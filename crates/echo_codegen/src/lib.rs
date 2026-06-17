mod abi;

use abi::{
    BuiltinCodegen, BuiltinLowering, CoreRuntimeSymbol, PHP_BUILTINS, PHP_RUNTIME_HELPERS,
    PhpBuiltin, STD_INTRINSICS, StdIntrinsic, php_builtin, std_intrinsic,
};
use echo_ast::{
    BinaryOp, BreakStmt, Expr, FunctionDeclStmt, IfStmt, LoopExpr, LoopStmt, ObjectExpr, Program,
    Stmt,
};
use echo_diagnostics::Diagnostic;
use echo_source::Span;
use inkwell::context::Context;
use std::collections::HashMap;

#[derive(Clone)]
enum RuntimeValue {
    StaticString(String),
    EchoValue(String),
}

pub fn backend_name() -> &'static str {
    "llvm"
}

pub fn smoke_test_module_ir() -> String {
    let context = Context::create();
    let module = context.create_module("echo_smoke");

    module.print_to_string().to_string()
}

pub fn compile_to_ir(program: &Program) -> Result<String, Vec<Diagnostic>> {
    let mut module = IrModule::new();
    let body = module.render_program(program)?;

    Ok(format!(
        r#"target triple = "x86_64-pc-linux-gnu"

%EchoValue = type {{ i32, i64 }}

{}
{}

{}

define i32 @main() {{
entry:
{}  call void @{}()
  ret i32 0
}}
"#,
        module.globals,
        runtime_declarations(),
        module.functions_ir,
        body,
        CoreRuntimeSymbol::Shutdown.symbol(),
    ))
}

struct IrModule {
    globals: String,
    functions_ir: String,
    aliases: HashMap<String, String>,
    locals: HashMap<String, RuntimeValue>,
    functions: HashMap<String, FunctionDeclStmt>,
    returned: bool,
    terminated: bool,
    break_labels: Vec<String>,
    break_value_slots: Vec<Option<String>>,
    next_string_id: usize,
    next_call_id: usize,
    next_defer_id: usize,
    next_loop_id: usize,
    next_if_id: usize,
}

impl IrModule {
    fn new() -> Self {
        Self {
            globals: String::new(),
            functions_ir: String::new(),
            aliases: HashMap::new(),
            locals: HashMap::new(),
            functions: HashMap::new(),
            returned: false,
            terminated: false,
            break_labels: Vec::new(),
            break_value_slots: Vec::new(),
            next_string_id: 0,
            next_call_id: 0,
            next_defer_id: 0,
            next_loop_id: 0,
            next_if_id: 0,
        }
    }

    fn render_program(&mut self, program: &Program) -> Result<String, Vec<Diagnostic>> {
        let mut body = String::new();
        let mut diagnostics = Vec::new();

        for statement in &program.statements {
            if let Stmt::FunctionDecl(statement) = statement
                && !statement.is_intrinsic
                && !statement.is_generator
            {
                self.functions
                    .insert(statement.name.clone(), statement.clone());
            }
        }

        for function in self.functions.clone().into_values() {
            if let Err(diagnostic) = self.render_userland_function(&function) {
                diagnostics.push(diagnostic);
            }
        }

        for statement in &program.statements {
            if let Err(diagnostic) = self.render_stmt(&mut body, statement) {
                diagnostics.push(diagnostic);
            }
        }

        if diagnostics.is_empty() {
            Ok(body)
        } else {
            Err(diagnostics)
        }
    }

    fn render_userland_function(&mut self, function: &FunctionDeclStmt) -> Result<(), Diagnostic> {
        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_returned = self.returned;
        let saved_terminated = self.terminated;
        self.returned = false;
        self.terminated = false;

        for param in &function.params {
            self.locals.insert(
                param.clone(),
                RuntimeValue::EchoValue(format!("%arg_{param}")),
            );
        }

        let mut body = String::new();

        for statement in &function.body {
            if let Err(diagnostic) = self.render_stmt(&mut body, statement) {
                self.aliases = saved_aliases;
                self.locals = saved_locals;
                self.returned = saved_returned;
                self.terminated = saved_terminated;
                return Err(diagnostic);
            }
        }

        let returned = self.returned;

        self.aliases = saved_aliases;
        self.locals = saved_locals;
        self.returned = saved_returned;
        self.terminated = saved_terminated;

        let params = function
            .params
            .iter()
            .map(|param| format!("%EchoValue %arg_{param}"))
            .collect::<Vec<_>>()
            .join(", ");

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{}({params}) {{\nentry:\n{}{}\n}}\n",
            userland_function_symbol(&function.name),
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(())
    }

    fn render_stmt(&mut self, body: &mut String, statement: &Stmt) -> Result<(), Diagnostic> {
        match statement {
            Stmt::Echo(statement) => {
                for expr in &statement.exprs {
                    let value = self.render_expr(body, expr)?;
                    self.write_value(body, value);
                }
            }
            Stmt::FunctionCall(statement) => {
                if statement.name == "time.sleep" {
                    self.time_sleep_call(body, &statement.args, statement.span)?;
                } else if let Some(intrinsic) = std_intrinsic(&statement.name) {
                    self.std_intrinsic_call(body, intrinsic, &statement.args, statement.span)?;
                } else {
                    match php_builtin(&statement.name) {
                        Some(builtin) if builtin.lowering == BuiltinLowering::DirectRuntimeCall => {
                            self.php_builtin_call(body, builtin, &statement.args)?
                        }
                        None => self.userland_call(body, statement)?,
                        Some(_) => self.userland_call(body, statement)?,
                    }
                }
            }
            Stmt::DynamicFunctionCall(statement) => self.dynamic_function_call(body, statement)?,
            Stmt::FunctionDecl(_) => {}
            Stmt::Yield(statement) => {
                return Err(Diagnostic::new(
                    "unsupported yield statement in LLVM codegen",
                    statement.span,
                ));
            }
            Stmt::Namespace(_)
            | Stmt::Use(_)
            | Stmt::Import(_)
            | Stmt::ClassDecl(_)
            | Stmt::TypeDecl(_) => {}
            Stmt::Loop(statement) => self.render_loop_stmt(body, statement)?,
            Stmt::If(statement) => {
                self.render_if_stmt(body, statement)?;
            }
            Stmt::Break(statement) => self.render_break_stmt(body, statement)?,
            Stmt::Append(statement) => {
                let target = self.resolve_alias(&statement.target);
                let Some(list) = self.locals.get(&target).cloned() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported append to undefined variable `${}` in LLVM codegen",
                            statement.target
                        ),
                        statement.span,
                    ));
                };
                let list = self.runtime_value_as_echo_value(body, list);
                let value = self.render_expr_as_echo_value(body, &statement.value)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({list}, {value})\n",
                    CoreRuntimeSymbol::ValueListAppend.symbol()
                ));

                self.locals.insert(target, RuntimeValue::EchoValue(name));
            }
            Stmt::Return(statement) => {
                let value = self.render_expr_as_echo_value(body, &statement.value)?;
                body.push_str(&format!("  ret {value}\n"));
                self.returned = true;
                self.terminated = true;
            }
            Stmt::Expr(statement) => {
                self.render_expr(body, &statement.expr)?;
            }
            Stmt::Assign(statement) => {
                let value = self.render_expr(body, &statement.value)?;
                // PHP assignments copy values by default; references are handled separately.
                // Source: https://www.php.net/manual/en/language.operators.assignment.php
                let name = self.resolve_alias(&statement.name);
                self.locals.insert(name, value);
            }
            Stmt::Let(statement) => {
                let value = self.render_expr(body, &statement.value)?;
                let name = self.resolve_alias(&statement.name);
                self.locals.insert(name, value);
            }
            Stmt::AssignRef(statement) => {
                let target = self.resolve_alias(&statement.target);
                if self.locals.contains_key(&target) {
                    // PHP references make two variable names aliases for the same value cell.
                    // Source: https://www.php.net/manual/en/language.references.php
                    self.aliases.insert(statement.name.clone(), target);
                } else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported reference to undefined variable `${}` in LLVM codegen",
                            statement.target
                        ),
                        statement.span,
                    ));
                }
            }
        }

        Ok(())
    }

    fn render_loop_stmt(
        &mut self,
        body: &mut String,
        statement: &LoopStmt,
    ) -> Result<(), Diagnostic> {
        let loop_id = self.next_loop_id;
        self.next_loop_id += 1;
        let loop_label = format!("loop_{loop_id}");
        let after_label = format!("loop_after_{loop_id}");

        body.push_str(&format!("  br label %{loop_label}\n\n{loop_label}:\n"));
        self.break_labels.push(after_label.clone());
        self.break_value_slots.push(None);

        let saved_terminated = self.terminated;
        self.terminated = false;

        for statement in &statement.body {
            self.render_stmt(body, statement)?;
            if self.terminated {
                break;
            }
        }

        if !self.terminated {
            body.push_str(&format!("  br label %{loop_label}\n"));
        }

        self.break_labels.pop();
        self.break_value_slots.pop();
        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));

        Ok(())
    }

    fn render_break_stmt(
        &mut self,
        body: &mut String,
        statement: &BreakStmt,
    ) -> Result<(), Diagnostic> {
        let Some(label) = self.break_labels.last().cloned() else {
            return Err(Diagnostic::new(
                "break used outside of loop in LLVM codegen",
                statement.span,
            ));
        };

        if let Some(value) = &statement.value {
            let Some(Some(slot)) = self.break_value_slots.last().cloned() else {
                return Err(Diagnostic::new(
                    "break value used outside of expression loop in LLVM codegen",
                    statement.span,
                ));
            };
            let value = self.render_expr_as_echo_value(body, value)?;
            body.push_str(&format!("  store {value}, ptr {slot}\n"));
        }

        body.push_str(&format!("  br label %{label}\n"));
        self.terminated = true;

        Ok(())
    }

    fn render_if_stmt(&mut self, body: &mut String, statement: &IfStmt) -> Result<(), Diagnostic> {
        let condition = self.render_condition(body, &statement.condition)?;
        let if_id = self.next_if_id;
        self.next_if_id += 1;
        let then_label = format!("if_then_{if_id}");
        let after_label = format!("if_after_{if_id}");

        body.push_str(&format!(
            "  br i1 {condition}, label %{then_label}, label %{after_label}\n\n{then_label}:\n"
        ));

        let saved_terminated = self.terminated;
        self.terminated = false;

        for statement in &statement.body {
            self.render_stmt(body, statement)?;
            if self.terminated {
                break;
            }
        }

        if !self.terminated {
            body.push_str(&format!("  br label %{after_label}\n"));
        }

        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));

        Ok(())
    }

    fn render_loop_expr(
        &mut self,
        body: &mut String,
        expr: &LoopExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let loop_id = self.next_loop_id;
        self.next_loop_id += 1;
        let loop_label = format!("loop_expr_{loop_id}");
        let after_label = format!("loop_expr_after_{loop_id}");
        let result_slot = format!("%loop_result_{loop_id}");
        let result_name = format!("%loop_value_{loop_id}");

        body.push_str(&format!("  {result_slot} = alloca %EchoValue\n"));
        body.push_str(&format!("  br label %{loop_label}\n\n{loop_label}:\n"));
        self.break_labels.push(after_label.clone());
        self.break_value_slots.push(Some(result_slot.clone()));

        let saved_terminated = self.terminated;
        self.terminated = false;

        for statement in &expr.body {
            self.render_stmt(body, statement)?;
            if self.terminated {
                break;
            }
        }

        if !self.terminated {
            body.push_str(&format!("  br label %{loop_label}\n"));
        }

        self.break_labels.pop();
        self.break_value_slots.pop();
        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));
        body.push_str(&format!(
            "  {result_name} = load %EchoValue, ptr {result_slot}\n"
        ));

        Ok(RuntimeValue::EchoValue(result_name))
    }

    fn render_object_expr(
        &mut self,
        body: &mut String,
        expr: &ObjectExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut object = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {object} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueObjectNew.symbol()
        ));

        for field in &expr.fields {
            let field_global = self.string_global(&field.name);
            let value = self.render_expr_as_echo_value(body, &field.value)?;
            let call_id = self.next_call_id;
            self.next_call_id += 1;
            let next_object = format!("%runtime_call_{call_id}");

            body.push_str(&format!(
                "  {next_object} = call %EchoValue @{}(%EchoValue {object}, ptr @{field_global}, i64 {}, {value})\n",
                CoreRuntimeSymbol::ValueObjectSet.symbol(),
                field.name.len()
            ));
            object = next_object;
        }

        Ok(RuntimeValue::EchoValue(object))
    }

    fn render_condition(&mut self, body: &mut String, expr: &Expr) -> Result<String, Diagnostic> {
        match expr {
            Expr::Binary(expr) if matches!(expr.op, BinaryOp::Is | BinaryOp::IsNot) => {
                if !matches!(expr.right, Expr::Null(_)) {
                    return Err(Diagnostic::new(
                        "unsupported non-null `is` comparison in LLVM codegen",
                        expr.span,
                    ));
                }

                let value = self.render_expr_as_echo_value(body, &expr.left)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let kind = format!("%value_kind_{call_id}");
                let is_null = format!("%is_null_{call_id}");
                let condition = format!("%condition_{call_id}");

                body.push_str(&format!("  {kind} = extractvalue {value}, 0\n"));
                body.push_str(&format!("  {is_null} = icmp eq i32 {kind}, 0\n"));

                if expr.op == BinaryOp::IsNot {
                    body.push_str(&format!("  {condition} = xor i1 {is_null}, true\n"));
                    Ok(condition)
                } else {
                    Ok(is_null)
                }
            }
            _ => Err(Diagnostic::new(
                "unsupported if condition in LLVM codegen",
                expr.span(),
            )),
        }
    }

    fn userland_call(
        &mut self,
        body: &mut String,
        statement: &echo_ast::FunctionCallStmt,
    ) -> Result<(), Diagnostic> {
        let Some(function) = self.functions.get(&statement.name).cloned() else {
            return Err(Diagnostic::new(
                format!("unsupported function `{}` in LLVM codegen", statement.name),
                statement.span,
            ));
        };

        if statement.args.len() != function.params.len() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for userland function `{}` in LLVM codegen",
                    statement.name
                ),
                statement.span,
            ));
        }

        let mut args = Vec::new();
        for arg in &statement.args {
            args.push(self.render_expr_as_echo_value(body, arg)?);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}({})\n",
            userland_function_symbol(&statement.name),
            args.join(", ")
        ));

        Ok(())
    }

    fn time_sleep_call(
        &mut self,
        body: &mut String,
        args: &[Expr],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let [Expr::Number(expr)] = args else {
            return Err(Diagnostic::new(
                "unsupported argument for time.sleep in LLVM codegen",
                span,
            ));
        };

        let millis = expr.value.parse::<i64>().map_err(|_| {
            Diagnostic::new(
                "unsupported duration for time.sleep in LLVM codegen",
                expr.span,
            )
        })?;

        body.push_str(&format!(
            "  call void @{}(i64 {millis})\n",
            CoreRuntimeSymbol::TimeSleep.symbol()
        ));

        Ok(())
    }

    fn std_intrinsic_call(
        &mut self,
        body: &mut String,
        intrinsic: StdIntrinsic,
        args: &[Expr],
        span: Span,
    ) -> Result<RuntimeValue, Diagnostic> {
        if args.len() != intrinsic.arity {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for std intrinsic `{}` in LLVM codegen",
                    intrinsic.echo_name
                ),
                span,
            ));
        }

        let mut rendered_args = Vec::new();
        for arg in args {
            rendered_args.push(self.render_std_intrinsic_arg(body, arg)?);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({})\n",
            intrinsic.symbol,
            rendered_args.join(", ")
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    fn render_std_intrinsic_arg(
        &mut self,
        body: &mut String,
        arg: &Expr,
    ) -> Result<String, Diagnostic> {
        if let Expr::Number(expr) = arg {
            let value = expr.value.parse::<i64>().map_err(|_| {
                Diagnostic::new(
                    "unsupported numeric std intrinsic argument in LLVM codegen",
                    expr.span,
                )
            })?;
            return Ok(format!("%EchoValue {{ i32 2, i64 {value} }}"));
        }

        self.render_expr_as_echo_value(body, arg)
    }

    fn render_expr_as_echo_value(
        &mut self,
        body: &mut String,
        expr: &Expr,
    ) -> Result<String, Diagnostic> {
        let value = self.render_expr(body, expr)?;
        Ok(self.runtime_value_as_echo_value(body, value))
    }

    fn runtime_value_as_echo_value(&mut self, body: &mut String, value: RuntimeValue) -> String {
        match value {
            RuntimeValue::EchoValue(name) => format!("%EchoValue {name}"),
            RuntimeValue::StaticString(value) => {
                let global = self.string_global(&value);
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
                    CoreRuntimeSymbol::ValueString.symbol(),
                    value.len()
                ));

                format!("%EchoValue {name}")
            }
        }
    }

    fn dynamic_function_call(
        &mut self,
        body: &mut String,
        statement: &echo_ast::DynamicFunctionCallStmt,
    ) -> Result<(), Diagnostic> {
        if !statement.args.is_empty() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported arguments for dynamic function call `${}` in LLVM codegen",
                    statement.name
                ),
                statement.span,
            ));
        }

        let RuntimeValue::StaticString(name) = self
            .locals
            .get(&self.resolve_alias(&statement.name))
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    format!(
                        "unsupported undefined dynamic function `${}` in LLVM codegen",
                        statement.name
                    ),
                    statement.span,
                )
            })?
        else {
            return Err(Diagnostic::new(
                format!(
                    "unsupported non-string dynamic function `${}` in LLVM codegen",
                    statement.name
                ),
                statement.span,
            ));
        };

        let global = self.string_global(&name);
        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
            CoreRuntimeSymbol::CallFunction.symbol(),
            name.len()
        ));

        Ok(())
    }

    fn write_call(&mut self, body: &mut String, value: &str) {
        let global = self.string_global(value);
        body.push_str(&format!(
            "  call void @{}(ptr @{global}, i64 {})\n",
            CoreRuntimeSymbol::Write.symbol(),
            value.len()
        ));
    }

    fn write_value(&mut self, body: &mut String, value: RuntimeValue) {
        match value {
            RuntimeValue::StaticString(value) => self.write_call(body, &value),
            RuntimeValue::EchoValue(name) => body.push_str(&format!(
                "  call void @{}(%EchoValue {name})\n",
                CoreRuntimeSymbol::WriteValue.symbol()
            )),
        }
    }

    fn php_builtin_call(
        &mut self,
        body: &mut String,
        builtin: PhpBuiltin,
        args: &[Expr],
    ) -> Result<(), Diagnostic> {
        match builtin.codegen {
            BuiltinCodegen::ObStart => match args {
                [] => {
                    let call_id = self.next_call_id;
                    self.next_call_id += 1;

                    body.push_str(&format!(
                        "  %runtime_call_{call_id} = call i1 @{}()\n",
                        builtin.symbol
                    ));
                }
                [Expr::Null(_)] => {
                    let helper = builtin
                        .helper_symbol
                        .expect("ob_start value helper must be declared");
                    let call_id = self.next_call_id;
                    self.next_call_id += 1;

                    body.push_str(&format!(
                        "  %runtime_call_{call_id} = call i1 @{}(%EchoValue {{ i32 0, i64 0 }})\n",
                        helper
                    ));
                }
                [Expr::String(expr)] => {
                    let helper = builtin
                        .helper_symbol
                        .expect("ob_start value helper must be declared");
                    let global = self.string_global(&expr.value);
                    let value_id = self.next_call_id;
                    self.next_call_id += 1;
                    let start_id = self.next_call_id;
                    self.next_call_id += 1;

                    body.push_str(&format!(
                        "  %runtime_call_{value_id} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
                        CoreRuntimeSymbol::ValueString.symbol(),
                        expr.value.len()
                    ));
                    body.push_str(&format!(
                        "  %runtime_call_{start_id} = call i1 @{}(%EchoValue %runtime_call_{value_id})\n",
                        helper
                    ));
                }
                [expr] => {
                    return Err(Diagnostic::new(
                        "unsupported ob_start callback argument in LLVM codegen",
                        expr.span(),
                    ));
                }
                _ => {
                    return Err(Diagnostic::new(
                        "unsupported ob_start argument count in LLVM codegen",
                        args.first().map_or_else(|| Span::new(0, 0), Expr::span),
                    ));
                }
            },
            BuiltinCodegen::BoolStatement => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;

                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call i1 @{}()\n",
                    builtin.symbol
                ));
            }
            BuiltinCodegen::ValueExpression => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;

                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}()\n",
                    builtin.symbol
                ));
            }
            BuiltinCodegen::ValueUnaryExpression => {
                unreachable!("expression builtin used as statement call")
            }
            BuiltinCodegen::ValueBinaryExpression => {
                unreachable!("expression builtin used as statement call")
            }
        }

        Ok(())
    }

    fn render_expr(&mut self, body: &mut String, expr: &Expr) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            Expr::Null(_) => Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string())),
            Expr::String(expr) => Ok(RuntimeValue::StaticString(expr.value.clone())),
            Expr::Number(expr) => {
                let Ok(value) = expr.value.parse::<i64>() else {
                    return Err(Diagnostic::new(
                        "unsupported numeric literal in LLVM codegen",
                        expr.span,
                    ));
                };
                Ok(RuntimeValue::EchoValue(format!("{{ i32 2, i64 {value} }}")))
            }
            Expr::Variable(expr) => self
                .locals
                .get(&self.resolve_alias(&expr.name))
                .cloned()
                .ok_or_else(|| {
                    Diagnostic::new(
                        format!(
                            "unsupported undefined variable `${}` in LLVM codegen",
                            expr.name
                        ),
                        expr.span,
                    )
                }),
            Expr::FunctionCall(expr) => self.render_function_call_expr(body, expr),
            Expr::Defer(expr) => {
                let function = self.render_defer_function(body, &expr.body)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::TaskDefer.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            Expr::Run(expr) => self.render_run_expr(body, expr),
            Expr::Join(expr) => {
                self.render_task_unary_expr(body, &expr.handle, CoreRuntimeSymbol::TaskJoin)
            }
            Expr::Loop(expr) => self.render_loop_expr(body, expr),
            Expr::Fork(_) | Expr::Spawn(_) => {
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            Expr::Binary(expr) if expr.op == BinaryOp::Concat => {
                self.render_concat_expr(body, &expr.left, &expr.right)
            }
            Expr::Binary(expr) if expr.op == BinaryOp::Add => {
                self.render_add_expr(body, &expr.left, &expr.right)
            }
            Expr::Field(expr) => {
                let object = self.render_expr_as_echo_value(body, &expr.object)?;
                let field_global = self.string_global(&expr.field);
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({object}, ptr @{field_global}, i64 {})\n",
                    CoreRuntimeSymbol::ValueObjectGet.symbol(),
                    expr.field.len()
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            Expr::Object(expr) => self.render_object_expr(body, expr),
            Expr::List(expr) => {
                if !expr.values.is_empty() {
                    return Err(Diagnostic::new(
                        "unsupported non-empty list expression in LLVM codegen",
                        expr.span,
                    ));
                }

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}()\n",
                    CoreRuntimeSymbol::ValueListNew.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            _ => Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                expr.span(),
            )),
        }
    }

    fn render_concat_expr(
        &mut self,
        body: &mut String,
        left: &Expr,
        right: &Expr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let left = self.render_expr(body, left)?;
        let right = self.render_expr(body, right)?;

        match (left, right) {
            (RuntimeValue::StaticString(mut left), RuntimeValue::StaticString(right)) => {
                left.push_str(&right);
                Ok(RuntimeValue::StaticString(left))
            }
            (left, right) => {
                let left = self.runtime_value_as_echo_value(body, left);
                let right = self.runtime_value_as_echo_value(body, right);
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({left}, {right})\n",
                    CoreRuntimeSymbol::ValueConcat.symbol()
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
        }
    }

    fn render_add_expr(
        &mut self,
        body: &mut String,
        left: &Expr,
        right: &Expr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let left = self.render_expr_as_echo_value(body, left)?;
        let right = self.render_expr_as_echo_value(body, right)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({left}, {right})\n",
            CoreRuntimeSymbol::ValueAdd.symbol()
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    fn render_defer_function(
        &mut self,
        caller_body: &mut String,
        statements: &[Stmt],
    ) -> Result<String, Diagnostic> {
        let function = format!("echo_defer_{}", self.next_defer_id);
        self.next_defer_id += 1;

        let captures = self
            .locals
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();

        for (name, value) in &captures {
            let global = format!("echo_capture_{}_{}", function, name);
            self.globals
                .push_str(&format!("@{global} = global %EchoValue zeroinitializer\n"));
            let value = self.runtime_value_as_echo_value(caller_body, value.clone());
            caller_body.push_str(&format!("  store {value}, ptr @{global}\n"));
        }

        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_returned = self.returned;
        self.returned = false;

        let sleep = statements.first().and_then(task_sleep_millis);

        let mut body = String::new();
        for (name, _) in &captures {
            let global = format!("echo_capture_{}_{}", function, name);
            let local = format!("%capture_{}_{}", function, name);
            body.push_str(&format!("  {local} = load %EchoValue, ptr @{global}\n"));
            self.locals
                .insert(name.clone(), RuntimeValue::EchoValue(local));
        }
        if let Some(millis) = sleep {
            let continuation =
                self.render_defer_continuation_function(&function, &statements[1..])?;
            body.push_str(&format!(
                "  %runtime_call_{} = call %EchoValue @{}(i64 {millis}, ptr @{continuation})\n",
                self.next_call_id,
                CoreRuntimeSymbol::TaskSleepCurrent.symbol()
            ));
            self.next_call_id += 1;
            body.push_str(&format!(
                "  ret %EchoValue %runtime_call_{}\n",
                self.next_call_id - 1
            ));
            self.returned = true;
        } else {
            for statement in statements {
                if let Err(diagnostic) = self.render_stmt(&mut body, statement) {
                    self.aliases = saved_aliases;
                    self.locals = saved_locals;
                    self.returned = saved_returned;
                    return Err(diagnostic);
                }
            }
        }

        let returned = self.returned;
        self.aliases = saved_aliases;
        self.locals = saved_locals;
        self.returned = saved_returned;

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{function}() {{\nentry:\n{}{}\n}}\n",
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(function)
    }

    fn render_defer_continuation_function(
        &mut self,
        parent: &str,
        statements: &[Stmt],
    ) -> Result<String, Diagnostic> {
        let function = format!("{parent}_cont");
        let saved_returned = self.returned;
        self.returned = false;

        let mut body = String::new();
        for statement in statements {
            if let Err(diagnostic) = self.render_stmt(&mut body, statement) {
                self.returned = saved_returned;
                return Err(diagnostic);
            }
        }

        let returned = self.returned;
        self.returned = saved_returned;

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{function}() {{\nentry:\n{}{}\n}}\n",
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(function)
    }

    fn render_run_expr(
        &mut self,
        body: &mut String,
        expr: &echo_ast::RunExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_ast::RunExpr::Block { body: block, .. } => {
                let function = self.render_defer_function(body, block)?;
                let defer_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{defer_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::TaskDefer.symbol()
                ));

                let run_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{run_id} = call %EchoValue @{}(%EchoValue %runtime_call_{defer_id})\n",
                    CoreRuntimeSymbol::TaskRun.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{run_id}")))
            }
            echo_ast::RunExpr::Task { expr, .. } => {
                self.render_task_unary_expr(body, expr, CoreRuntimeSymbol::TaskRun)
            }
        }
    }

    fn render_task_unary_expr(
        &mut self,
        body: &mut String,
        expr: &Expr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let task = self.render_expr_as_echo_value(body, expr)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}({task})\n",
            symbol.symbol()
        ));

        Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
    }

    fn render_function_call_expr(
        &mut self,
        body: &mut String,
        expr: &echo_ast::FunctionCallExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        if expr.name == "time.sleep" {
            self.time_sleep_call(body, &expr.args, expr.span)?;
            return Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()));
        }

        if let Some(intrinsic) = std_intrinsic(&expr.name) {
            return self.std_intrinsic_call(body, intrinsic, &expr.args, expr.span);
        }

        let Some(builtin) = php_builtin(&expr.name) else {
            return self.render_userland_function_call_expr(body, expr);
        };

        match builtin.codegen {
            BuiltinCodegen::ValueExpression => {
                if !expr.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            expr.name
                        ),
                        expr.span,
                    ));
                }

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}()\n",
                    builtin.symbol
                ));
                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryExpression => {
                let [arg] = expr.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            expr.name
                        ),
                        expr.span,
                    ));
                };

                let arg = self.render_expr_as_echo_value(body, arg)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({arg})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueBinaryExpression => {
                let [left, right] = expr.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            expr.name
                        ),
                        expr.span,
                    ));
                };

                let left = self.render_expr_as_echo_value(body, left)?;
                let right = self.render_expr_as_echo_value(body, right)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({left}, {right})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            _ => Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                expr.span,
            )),
        }
    }

    fn render_userland_function_call_expr(
        &mut self,
        body: &mut String,
        expr: &echo_ast::FunctionCallExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let Some(function) = self.functions.get(&expr.name).cloned() else {
            return Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                expr.span,
            ));
        };

        if expr.args.len() != function.params.len() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for userland function `{}` in LLVM codegen",
                    expr.name
                ),
                expr.span,
            ));
        }

        let mut args = Vec::new();
        for arg in &expr.args {
            args.push(self.render_expr_as_echo_value(body, arg)?);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({})\n",
            userland_function_symbol(&expr.name),
            args.join(", ")
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    fn resolve_alias(&self, name: &str) -> String {
        let mut current = name;

        while let Some(next) = self.aliases.get(current) {
            current = next;
        }

        current.to_string()
    }

    fn string_global(&mut self, value: &str) -> String {
        let name = format!("echo_str_{}", self.next_string_id);
        self.next_string_id += 1;

        self.globals.push_str(&format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{}\", align 1\n",
            value.len(),
            llvm_string_literal(value)
        ));

        name
    }
}

fn runtime_declarations() -> String {
    CoreRuntimeSymbol::ALL
        .iter()
        .map(|function| function.llvm_decl())
        .chain(
            PHP_RUNTIME_HELPERS
                .iter()
                .map(|(symbol, signature)| signature.llvm_decl(symbol)),
        )
        .chain(PHP_BUILTINS.iter().map(|builtin| builtin.llvm_decl()))
        .chain(STD_INTRINSICS.iter().map(|intrinsic| intrinsic.llvm_decl()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn userland_function_symbol(name: &str) -> String {
    format!("echo_user_{name}")
}

fn task_sleep_millis(statement: &Stmt) -> Option<i64> {
    let Stmt::FunctionCall(statement) = statement else {
        return None;
    };
    if statement.name != "time.sleep" {
        return None;
    }
    let [Expr::Number(expr)] = statement.args.as_slice() else {
        return None;
    };

    expr.value.parse().ok()
}

fn llvm_string_literal(value: &str) -> String {
    let mut output = String::new();

    for byte in value.bytes() {
        match byte {
            b'\\' => output.push_str(r#"\5C"#),
            b'"' => output.push_str(r#"\22"#),
            0x20..=0x7e => output.push(byte as char),
            _ => output.push_str(&format!(r#"\{byte:02X}"#)),
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_ast::{
        AssignStmt, DeferExpr, EchoStmt, FunctionCallExpr, FunctionCallStmt, FunctionDeclStmt,
        NullLiteral, ReturnStmt, StringLiteral,
    };

    fn program(statements: Vec<Stmt>) -> Program {
        Program {
            open_tag: None,
            statements,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn ob_start_null_uses_named_echo_value_abi() {
        let ir = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
            name: "ob_start".to_string(),
            args: vec![Expr::Null(NullLiteral {
                span: Span::new(0, 4),
            })],
            span: Span::new(0, 15),
        })]))
        .expect("IR");

        assert!(ir.contains("%EchoValue = type { i32, i64 }"));
        assert!(ir.contains("declare i1 @echo_php_ob_start_value(%EchoValue)"));
        assert!(
            ir.contains("call i1 @echo_php_ob_start_value(%EchoValue { i32 0, i64 0 })"),
            "{ir}"
        );
    }

    #[test]
    fn ob_start_string_constructs_echo_value_callback() {
        let ir = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
            name: "ob_start".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "filter".to_string(),
                span: Span::new(9, 17),
            })],
            span: Span::new(0, 19),
        })]))
        .expect("IR");

        assert!(ir.contains("declare %EchoValue @echo_value_string(ptr, i64)"));
        assert!(ir.contains("declare void @echo_write_value(%EchoValue)"));
        assert!(ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_0, i64 6)"));
        assert!(ir.contains("call i1 @echo_php_ob_start_value(%EchoValue %runtime_call_0)"));
    }

    #[test]
    fn userland_call_emits_function_definition_and_call() {
        let ir = compile_to_ir(&program(vec![
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "say_after".to_string(),
                params: vec![],
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Echo(EchoStmt {
                    exprs: vec![Expr::String(StringLiteral {
                        value: "after\n".to_string(),
                        span: Span::new(0, 8),
                    })],
                    span: Span::new(0, 15),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::FunctionCall(FunctionCallStmt {
                name: "say_after".to_string(),
                args: vec![],
                span: Span::new(41, 53),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("define %EchoValue @echo_user_say_after()"),
            "{ir}"
        );
        assert!(
            ir.contains("call void @echo_write(ptr @echo_str_0, i64 6)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_user_say_after()"),
            "{ir}"
        );
    }

    #[test]
    fn userland_call_passes_string_argument_as_echo_value() {
        let ir = compile_to_ir(&program(vec![
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "say".to_string(),
                params: vec!["message".to_string()],
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Echo(EchoStmt {
                    exprs: vec![Expr::Variable(echo_ast::VariableExpr {
                        name: "message".to_string(),
                        span: Span::new(0, 8),
                    })],
                    span: Span::new(0, 15),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::FunctionCall(FunctionCallStmt {
                name: "say".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "hello\n".to_string(),
                    span: Span::new(45, 53),
                })],
                span: Span::new(41, 55),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("define %EchoValue @echo_user_say(%EchoValue %arg_message)"),
            "{ir}"
        );
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %arg_message)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_0, i64 6)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_user_say(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
    }

    #[test]
    fn userland_return_value_can_be_echoed() {
        let ir = compile_to_ir(&program(vec![
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "greeting".to_string(),
                params: vec![],
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Return(ReturnStmt {
                    value: Expr::String(StringLiteral {
                        value: "hello\n".to_string(),
                        span: Span::new(0, 8),
                    }),
                    span: Span::new(0, 16),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "greeting".to_string(),
                    args: vec![],
                    span: Span::new(45, 55),
                })],
                span: Span::new(41, 56),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("define %EchoValue @echo_user_greeting()"),
            "{ir}"
        );
        assert!(ir.contains("ret %EchoValue %runtime_call_0"), "{ir}");
        assert!(ir.contains("call %EchoValue @echo_user_greeting()"), "{ir}");
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
            "{ir}"
        );
    }

    #[test]
    fn dynamic_concat_uses_echo_value_concat() {
        let ir = compile_to_ir(&program(vec![
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "greet".to_string(),
                params: vec!["name".to_string()],
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Echo(EchoStmt {
                    exprs: vec![Expr::Binary(Box::new(echo_ast::BinaryExpr {
                        left: Expr::Binary(Box::new(echo_ast::BinaryExpr {
                            left: Expr::String(StringLiteral {
                                value: "Hello, ".to_string(),
                                span: Span::new(0, 9),
                            }),
                            op: BinaryOp::Concat,
                            right: Expr::Variable(echo_ast::VariableExpr {
                                name: "name".to_string(),
                                span: Span::new(12, 17),
                            }),
                            span: Span::new(0, 17),
                        })),
                        op: BinaryOp::Concat,
                        right: Expr::String(StringLiteral {
                            value: "!\n".to_string(),
                            span: Span::new(20, 24),
                        }),
                        span: Span::new(0, 24),
                    }))],
                    span: Span::new(0, 30),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::FunctionCall(FunctionCallStmt {
                name: "greet".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "Echo".to_string(),
                    span: Span::new(45, 51),
                })],
                span: Span::new(41, 53),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_value_concat(%EchoValue, %EchoValue)"),
            "{ir}"
        );
        assert!(ir.contains("call %EchoValue @echo_value_concat"), "{ir}");
    }

    #[test]
    fn strlen_lowers_to_php_builtin_with_echo_value_argument() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "strlen".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "hello".to_string(),
                    span: Span::new(7, 14),
                })],
                span: Span::new(0, 15),
            })],
            span: Span::new(0, 16),
        })]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_php_strlen(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_php_strlen(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
            "{ir}"
        );
    }

    #[test]
    fn string_case_builtins_lower_to_php_builtin_with_echo_value_argument() {
        for (php_name, symbol) in [
            ("strval", "echo_php_strval"),
            ("boolval", "echo_php_boolval"),
            ("intval", "echo_php_intval"),
            ("strtoupper", "echo_php_strtoupper"),
            ("strtolower", "echo_php_strtolower"),
            ("ucwords", "echo_php_ucwords"),
            ("strrev", "echo_php_strrev"),
            ("ucfirst", "echo_php_ucfirst"),
            ("lcfirst", "echo_php_lcfirst"),
            ("ord", "echo_php_ord"),
            ("str_rot13", "echo_php_str_rot13"),
            ("chr", "echo_php_chr"),
            ("bin2hex", "echo_php_bin2hex"),
            ("base64_encode", "echo_php_base64_encode"),
            ("base64_decode", "echo_php_base64_decode"),
            ("hex2bin", "echo_php_hex2bin"),
            ("trim", "echo_php_trim"),
            ("ltrim", "echo_php_ltrim"),
            ("rtrim", "echo_php_rtrim"),
            ("addslashes", "echo_php_addslashes"),
            ("stripslashes", "echo_php_stripslashes"),
            ("quotemeta", "echo_php_quotemeta"),
        ] {
            let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: php_name.to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "Echo".to_string(),
                        span: Span::new(11, 17),
                    })],
                    span: Span::new(0, 18),
                })],
                span: Span::new(0, 19),
            })]))
            .expect("IR");

            assert!(
                ir.contains(&format!("declare %EchoValue @{symbol}(%EchoValue)")),
                "{ir}"
            );
            assert!(
                ir.contains(&format!(
                    "call %EchoValue @{symbol}(%EchoValue %runtime_call_0)"
                )),
                "{ir}"
            );
            assert!(
                ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
                "{ir}"
            );
        }
    }

    #[test]
    fn string_predicate_builtins_lower_to_php_builtin_with_two_echo_value_arguments() {
        for (php_name, symbol) in [
            ("str_contains", "echo_php_str_contains"),
            ("str_starts_with", "echo_php_str_starts_with"),
            ("str_ends_with", "echo_php_str_ends_with"),
            ("str_repeat", "echo_php_str_repeat"),
            ("substr", "echo_php_substr"),
            ("strpos", "echo_php_strpos"),
            ("stripos", "echo_php_stripos"),
            ("strrpos", "echo_php_strrpos"),
            ("strripos", "echo_php_strripos"),
            ("strstr", "echo_php_strstr"),
            ("stristr", "echo_php_stristr"),
            ("strrchr", "echo_php_strrchr"),
            ("strpbrk", "echo_php_strpbrk"),
            ("strspn", "echo_php_strspn"),
            ("strcspn", "echo_php_strcspn"),
            ("substr_count", "echo_php_substr_count"),
            ("strcmp", "echo_php_strcmp"),
            ("strcasecmp", "echo_php_strcasecmp"),
        ] {
            let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: php_name.to_string(),
                    args: vec![
                        Expr::String(StringLiteral {
                            value: "Echo PHP".to_string(),
                            span: Span::new(13, 23),
                        }),
                        Expr::String(StringLiteral {
                            value: "PHP".to_string(),
                            span: Span::new(25, 30),
                        }),
                    ],
                    span: Span::new(0, 31),
                })],
                span: Span::new(0, 32),
            })]))
            .expect("IR");

            assert!(
                ir.contains(&format!(
                    "declare %EchoValue @{symbol}(%EchoValue, %EchoValue)"
                )),
                "{ir}"
            );
            assert!(
                ir.contains(&format!(
                    "call %EchoValue @{symbol}(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1)"
                )),
                "{ir}"
            );
            assert!(
                ir.contains("call void @echo_write_value(%EchoValue %runtime_call_2)"),
                "{ir}"
            );
        }
    }

    #[test]
    fn time_sleep_lowers_to_core_runtime_call() {
        let ir = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
            name: "time.sleep".to_string(),
            args: vec![Expr::Number(echo_ast::NumberLiteral {
                value: "50".to_string(),
                span: Span::new(11, 13),
            })],
            span: Span::new(0, 14),
        })]))
        .expect("IR");

        assert!(ir.contains("declare void @echo_time_sleep(i64)"), "{ir}");
        assert!(ir.contains("call void @echo_time_sleep(i64 50)"), "{ir}");
    }

    #[test]
    fn net_listen_lowers_to_std_intrinsic_call() {
        let ir = compile_to_ir(&program(vec![Stmt::Assign(AssignStmt {
            name: "server".to_string(),
            value: Expr::FunctionCall(FunctionCallExpr {
                name: "net.listen".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "127.0.0.1:39183".to_string(),
                    span: Span::new(11, 30),
                })],
                span: Span::new(0, 31),
            }),
            span: Span::new(0, 31),
        })]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_std_net_listen(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_std_net_listen(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
    }

    #[test]
    fn net_read_lowers_numeric_length_as_int_value() {
        let ir = compile_to_ir(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "connection".to_string(),
                value: Expr::FunctionCall(FunctionCallExpr {
                    name: "net.connect".to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "127.0.0.1:39183".to_string(),
                        span: Span::new(0, 17),
                    })],
                    span: Span::new(0, 18),
                }),
                span: Span::new(0, 18),
            }),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "net.read".to_string(),
                    args: vec![
                        Expr::Variable(echo_ast::VariableExpr {
                            name: "connection".to_string(),
                            span: Span::new(19, 30),
                        }),
                        Expr::Number(echo_ast::NumberLiteral {
                            value: "4".to_string(),
                            span: Span::new(31, 32),
                        }),
                    ],
                    span: Span::new(19, 33),
                })],
                span: Span::new(19, 33),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains(
                "call %EchoValue @echo_std_net_read(%EchoValue %runtime_call_1, %EchoValue { i32 2, i64 4 })"
            ),
            "{ir}"
        );
    }

    #[test]
    fn http_response_text_lowers_to_std_intrinsic_call() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "http.responseText".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "hello".to_string(),
                    span: Span::new(18, 25),
                })],
                span: Span::new(0, 26),
            })],
            span: Span::new(0, 26),
        })]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_std_http_response_text(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_std_http_response_text(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
    }

    #[test]
    fn task_sleep_lowers_to_timer_continuation() {
        let ir = compile_to_ir(&program(vec![Stmt::Assign(AssignStmt {
            name: "task".to_string(),
            value: Expr::Run(echo_ast::RunExpr::Block {
                body: vec![
                    Stmt::FunctionCall(FunctionCallStmt {
                        name: "time.sleep".to_string(),
                        args: vec![Expr::Number(echo_ast::NumberLiteral {
                            value: "50".to_string(),
                            span: Span::new(0, 2),
                        })],
                        span: Span::new(0, 14),
                    }),
                    Stmt::Echo(EchoStmt {
                        exprs: vec![Expr::String(StringLiteral {
                            value: "done\n".to_string(),
                            span: Span::new(15, 22),
                        })],
                        span: Span::new(15, 28),
                    }),
                ],
                span: Span::new(0, 28),
            }),
            span: Span::new(0, 28),
        })]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_task_sleep_current(i64, ptr)"),
            "{ir}"
        );
        assert!(
            ir.contains("define %EchoValue @echo_defer_0_cont()"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_task_sleep_current(i64 50, ptr @echo_defer_0_cont)"),
            "{ir}"
        );
    }

    #[test]
    fn expression_statement_evaluates_and_discards_value() {
        let ir = compile_to_ir(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Defer(DeferExpr {
                    body: vec![],
                    span: Span::new(0, 10),
                }),
                span: Span::new(0, 10),
            }),
            Stmt::Expr(echo_ast::ExprStmt {
                expr: Expr::Join(echo_ast::JoinExpr {
                    handle: Box::new(Expr::Variable(echo_ast::VariableExpr {
                        name: "task".to_string(),
                        span: Span::new(11, 16),
                    })),
                    span: Span::new(11, 16),
                }),
                span: Span::new(11, 16),
            }),
        ]))
        .expect("IR");

        assert!(ir.contains("call %EchoValue @echo_task_join"), "{ir}");
        assert!(!ir.contains("call void @echo_write_value"), "{ir}");
    }

    #[test]
    fn defer_lowers_to_runtime_task_handle() {
        let ir = compile_to_ir(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "deferred".to_string(),
                value: Expr::Defer(DeferExpr {
                    body: vec![],
                    span: Span::new(0, 10),
                }),
                span: Span::new(0, 10),
            }),
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(echo_ast::RunExpr::Task {
                    expr: Box::new(Expr::Variable(echo_ast::VariableExpr {
                        name: "deferred".to_string(),
                        span: Span::new(11, 20),
                    })),
                    span: Span::new(11, 20),
                }),
                span: Span::new(11, 20),
            }),
            Stmt::Assign(AssignStmt {
                name: "value".to_string(),
                value: Expr::Join(echo_ast::JoinExpr {
                    handle: Box::new(Expr::Variable(echo_ast::VariableExpr {
                        name: "task".to_string(),
                        span: Span::new(21, 30),
                    })),
                    span: Span::new(21, 30),
                }),
                span: Span::new(21, 30),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_task_defer(ptr)"),
            "{ir}"
        );
        assert!(ir.contains("define %EchoValue @echo_defer_0()"), "{ir}");
        assert!(
            ir.contains("call %EchoValue @echo_task_defer(ptr @echo_defer_0)"),
            "{ir}"
        );
        assert!(
            ir.contains("declare %EchoValue @echo_task_run(%EchoValue)"),
            "{ir}"
        );
        assert!(ir.contains("call %EchoValue @echo_task_run"), "{ir}");
        assert!(
            ir.contains("declare %EchoValue @echo_task_join(%EchoValue)"),
            "{ir}"
        );
        assert!(ir.contains("call %EchoValue @echo_task_join"), "{ir}");
        assert!(ir.contains("ret i32 0"), "{ir}");
    }
}
