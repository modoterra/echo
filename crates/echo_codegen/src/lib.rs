mod abi;
mod jit;
mod php_lowering;
mod runtime_symbols;
mod std_lowering;

use abi::{
    BuiltinCodegen, BuiltinLowering, CoreRuntimeSymbol, PHP_BUILTINS, PHP_RUNTIME_HELPERS,
    STD_INTRINSICS, php_builtin, std_intrinsic,
};
use echo_ast::{BinaryOp, ImportSource, Program, Stmt, TypedParam, UnaryOp};
use echo_diagnostics::Diagnostic;
use echo_source::Span;
use inkwell::context::Context;
use std::collections::{HashMap, HashSet};

use jit::run_ir_jit;
pub use jit::{JitOptions, JitOutput, run_ir_jit_with_options};
pub(crate) use runtime_symbols::jit_runtime_symbol_addresses;

#[derive(Clone)]
enum RuntimeValue {
    StaticString(String),
    EchoValue(String),
}

const REFLECTION_SOURCE_PHP_BUILTIN: i32 = 1;
const REFLECTION_SOURCE_STD: i32 = 2;
const REFLECTION_SOURCE_USERLAND: i32 = 3;

fn stmt_span(statement: &Stmt) -> Span {
    match statement {
        Stmt::Echo(statement) => statement.span,
        Stmt::FunctionCall(statement) => statement.span,
        Stmt::DynamicFunctionCall(statement) => statement.span,
        Stmt::FunctionDecl(statement) => statement.span,
        Stmt::Assign(statement) => statement.span,
        Stmt::Let(statement) => statement.span,
        Stmt::AssignRef(statement) => statement.span,
        Stmt::Return(statement) => statement.span,
        Stmt::Yield(statement) => statement.span,
        Stmt::Expr(statement) => statement.span,
        Stmt::Namespace(statement) => statement.span,
        Stmt::Use(statement) => statement.span,
        Stmt::Import(statement) => statement.span,
        Stmt::ClassDecl(statement) => statement.span,
        Stmt::TypeDecl(statement) => statement.span,
        Stmt::Loop(statement) => statement.span,
        Stmt::If(statement) => statement.span,
        Stmt::Break(statement) => statement.span,
        Stmt::Append(statement) => statement.span,
    }
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
    let hir = echo_hir::lower_program(program)?;
    let mir = echo_mir::lower_program(&hir)?;

    compile_mir_to_ir(&mir)
}

pub fn compile_hir_to_ir(program: &echo_hir::HirProgram) -> Result<String, Vec<Diagnostic>> {
    let mir = echo_mir::lower_program(program)?;

    compile_mir_to_ir(&mir)
}

pub fn compile_mir_to_ir(program: &echo_mir::MirProgram) -> Result<String, Vec<Diagnostic>> {
    let mut module = IrModule::new();
    module.source_dir = program.source_dir().map(str::to_string);
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

pub fn run_program_jit(program: &Program) -> Result<i32, Vec<Diagnostic>> {
    let ir = compile_to_ir(program)?;
    run_ir_jit(&ir)
}

pub fn run_mir_jit(program: &echo_mir::MirProgram) -> Result<i32, Vec<Diagnostic>> {
    let ir = compile_mir_to_ir(program)?;
    run_ir_jit(&ir)
}

struct IrModule {
    globals: String,
    functions_ir: String,
    aliases: HashMap<String, String>,
    std_imports: HashMap<String, String>,
    locals: HashMap<String, RuntimeValue>,
    functions: HashMap<String, echo_mir::MirFunction>,
    source_dir: Option<String>,
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
            std_imports: HashMap::new(),
            locals: HashMap::new(),
            functions: HashMap::new(),
            source_dir: None,
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

    fn render_program(
        &mut self,
        program: &echo_mir::MirProgram,
    ) -> Result<String, Vec<Diagnostic>> {
        let mut body = String::new();
        let mut diagnostics = Vec::new();

        for statement in program.imports() {
            if let Err(diagnostic) = self.register_std_import(statement) {
                diagnostics.push(diagnostic);
            }
        }

        for statement in program.functions() {
            self.functions
                .insert(statement.name.clone(), statement.clone());
        }

        for function in self.functions.clone().into_values() {
            if let Err(diagnostic) = self.render_userland_function(&function) {
                diagnostics.push(diagnostic);
            }
        }

        self.render_reflection_registrations(&mut body);

        for statement in program.statements() {
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                diagnostics.push(diagnostic);
            }
        }

        if diagnostics.is_empty() {
            Ok(body)
        } else {
            Err(diagnostics)
        }
    }

    fn render_reflection_registrations(&mut self, body: &mut String) {
        let php_builtins = echo_reflection::php_builtins()
            .iter()
            .map(|function| {
                (
                    function.qualified_name.clone(),
                    function.params_signature(),
                    function.return_type_signature().to_string(),
                    REFLECTION_SOURCE_PHP_BUILTIN,
                )
            })
            .collect::<Vec<_>>();
        for (name, params, return_type, source_kind) in php_builtins {
            self.register_function_reflection(body, &name, &params, &return_type, source_kind);
        }

        let std_functions = echo_reflection::functions()
            .iter()
            .filter(|function| function.source == echo_reflection::FunctionSource::Std)
            .map(|function| {
                (
                    function.qualified_name.clone(),
                    function.params_signature(),
                    function.return_type_signature().to_string(),
                    REFLECTION_SOURCE_STD,
                )
            })
            .collect::<Vec<_>>();
        for (name, params, return_type, source_kind) in std_functions {
            self.register_function_reflection(body, &name, &params, &return_type, source_kind);
        }

        let mut functions = self.functions.values().cloned().collect::<Vec<_>>();
        functions.sort_by(|left, right| left.name.cmp(&right.name));

        for function in &functions {
            let name = function.name.clone();
            let params = function_params_signature(&function.params);
            let return_type = function.return_type.clone().unwrap_or_default();

            self.register_function_reflection(
                body,
                &name,
                &params,
                &return_type,
                REFLECTION_SOURCE_USERLAND,
            );
        }
    }

    fn register_function_reflection(
        &mut self,
        body: &mut String,
        name: &str,
        params: &str,
        return_type: &str,
        source_kind: i32,
    ) {
        let name_global = self.string_global(name);
        let params_global = self.string_global(params);
        let return_type_global = self.string_global(return_type);

        body.push_str(&format!(
            "  call void @{}(ptr @{name_global}, i64 {}, ptr @{params_global}, i64 {}, ptr @{return_type_global}, i64 {}, i32 {source_kind})\n",
            CoreRuntimeSymbol::RegisterFunction.symbol(),
            name.len(),
            params.len(),
            return_type.len()
        ));
    }

    fn render_userland_function(
        &mut self,
        function: &echo_mir::MirFunction,
    ) -> Result<(), Diagnostic> {
        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_returned = self.returned;
        let saved_terminated = self.terminated;
        self.returned = false;
        self.terminated = false;

        for param in &function.params {
            self.locals.insert(
                param.name.clone(),
                RuntimeValue::EchoValue(format!("%arg_{}", param.name)),
            );
        }

        let mut body = String::new();

        for statement in &function.body {
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
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
            .map(|param| format!("%EchoValue %arg_{}", param.name))
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

    fn render_mir_stmt(
        &mut self,
        body: &mut String,
        statement: &echo_mir::MirStmt,
    ) -> Result<(), Diagnostic> {
        match statement {
            echo_mir::MirStmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.render_mir_expr(body, expr)?;
                    self.write_value(body, value);
                }
                Ok(())
            }
            echo_mir::MirStmt::FunctionCall { call, .. } => {
                self.render_mir_function_call_stmt(body, call)
            }
            echo_mir::MirStmt::DynamicFunctionCall { source, name, args } => {
                self.render_mir_dynamic_function_call(body, source, name, args)
            }
            echo_mir::MirStmt::Assign { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let name = self.resolve_alias(name);
                self.locals.insert(name, value);
                Ok(())
            }
            echo_mir::MirStmt::Let { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let name = self.resolve_alias(name);
                self.locals.insert(name, value);
                Ok(())
            }
            echo_mir::MirStmt::Return { value, .. } => {
                let value = self.render_mir_expr_as_echo_value(body, value)?;
                body.push_str(&format!("  ret {value}\n"));
                self.returned = true;
                self.terminated = true;
                Ok(())
            }
            echo_mir::MirStmt::Expr { expr, .. } => {
                self.render_mir_expr(body, expr)?;
                Ok(())
            }
            echo_mir::MirStmt::Loop {
                body: loop_body, ..
            } => self.render_mir_loop_stmt(body, loop_body),
            echo_mir::MirStmt::If {
                condition,
                body: if_body,
                ..
            } => self.render_mir_if_stmt(body, condition, if_body),
            echo_mir::MirStmt::Break { source, value } => {
                self.render_mir_break_stmt(body, source, value.as_ref())
            }
            echo_mir::MirStmt::Append {
                source,
                target,
                value,
            } => self.render_mir_append_stmt(body, source, target, value),
            echo_mir::MirStmt::AssignRef {
                source,
                name,
                target,
            } => self.render_mir_assign_ref_stmt(source, name, target),
            echo_mir::MirStmt::Yield { source, .. } => Err(Diagnostic::new(
                "unsupported yield statement in LLVM codegen",
                stmt_span(source),
            )),
            echo_mir::MirStmt::Noop { .. } => Ok(()),
        }
    }

    fn render_mir_function_call_stmt(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<(), Diagnostic> {
        let name = self.resolve_std_call_name(&call.name, call.span)?;
        if name == "time.sleep" {
            self.mir_time_sleep_call(body, &call.args, call.span)?;
        } else if let Some(intrinsic) = std_intrinsic(&name) {
            self.mir_std_intrinsic_call(body, intrinsic, &call.args, call.span)?;
        } else {
            match php_builtin(&name) {
                Some(builtin) if builtin.lowering == BuiltinLowering::DirectRuntimeCall => {
                    self.mir_php_builtin_call(body, builtin, &call.args, call.span)?
                }
                None => self.mir_userland_call(body, call)?,
                Some(_) => self.mir_userland_call(body, call)?,
            }
        }

        Ok(())
    }

    fn render_mir_loop_stmt(
        &mut self,
        body: &mut String,
        statements: &[echo_mir::MirStmt],
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

        for statement in statements {
            self.render_mir_stmt(body, statement)?;
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

    fn render_mir_break_stmt(
        &mut self,
        body: &mut String,
        source: &Stmt,
        value: Option<&echo_mir::MirExpr>,
    ) -> Result<(), Diagnostic> {
        let Some(label) = self.break_labels.last().cloned() else {
            return Err(Diagnostic::new(
                "break used outside of loop in LLVM codegen",
                stmt_span(source),
            ));
        };

        if let Some(value) = value {
            let Some(Some(slot)) = self.break_value_slots.last().cloned() else {
                return Err(Diagnostic::new(
                    "break value used outside of expression loop in LLVM codegen",
                    stmt_span(source),
                ));
            };
            let value = self.render_mir_expr_as_echo_value(body, value)?;
            body.push_str(&format!("  store {value}, ptr {slot}\n"));
        }

        body.push_str(&format!("  br label %{label}\n"));
        self.terminated = true;

        Ok(())
    }

    fn render_mir_if_stmt(
        &mut self,
        body: &mut String,
        condition_expr: &echo_mir::MirExpr,
        statements: &[echo_mir::MirStmt],
    ) -> Result<(), Diagnostic> {
        let condition = self.render_mir_condition(body, condition_expr)?;
        let if_id = self.next_if_id;
        self.next_if_id += 1;
        let then_label = format!("if_then_{if_id}");
        let after_label = format!("if_after_{if_id}");

        body.push_str(&format!(
            "  br i1 {condition}, label %{then_label}, label %{after_label}\n\n{then_label}:\n"
        ));

        let saved_terminated = self.terminated;
        self.terminated = false;

        for statement in statements {
            self.render_mir_stmt(body, statement)?;
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

    fn render_mir_append_stmt(
        &mut self,
        body: &mut String,
        source: &Stmt,
        target: &str,
        value: &echo_mir::MirExpr,
    ) -> Result<(), Diagnostic> {
        let resolved_target = self.resolve_alias(target);
        let Some(list) = self.locals.get(&resolved_target).cloned() else {
            return Err(Diagnostic::new(
                format!("unsupported append to undefined variable `${target}` in LLVM codegen"),
                stmt_span(source),
            ));
        };
        let list = self.runtime_value_as_echo_value(body, list);
        let value = self.render_mir_expr_as_echo_value(body, value)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({list}, {value})\n",
            CoreRuntimeSymbol::ValueArrayAppend.symbol()
        ));

        self.locals
            .insert(resolved_target, RuntimeValue::EchoValue(name));

        Ok(())
    }

    fn render_mir_method_call_expr(
        &mut self,
        body: &mut String,
        object: &echo_mir::MirExpr,
        method: &str,
        args: &[echo_mir::MirExpr],
    ) -> Result<RuntimeValue, Diagnostic> {
        match (method, args) {
            ("push", [value]) => self.render_mir_list_push_expr(body, object, value),
            _ => {
                self.render_mir_expr_as_echo_value(body, object)?;
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
        }
    }

    fn render_mir_list_push_expr(
        &mut self,
        body: &mut String,
        object: &echo_mir::MirExpr,
        value: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let target = match object {
            echo_mir::MirExpr::Variable { name, .. } => Some(self.resolve_alias(name)),
            _ => None,
        };
        let list = self.render_mir_expr_as_echo_value(body, object)?;
        let value = self.render_mir_expr_as_echo_value(body, value)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({list}, {value})\n",
            CoreRuntimeSymbol::ValueListAppend.symbol()
        ));

        if let Some(target) = target {
            self.locals
                .insert(target, RuntimeValue::EchoValue(name.clone()));
        }

        Ok(RuntimeValue::EchoValue(name))
    }

    fn render_mir_assign_ref_stmt(
        &mut self,
        source: &Stmt,
        name: &str,
        target: &str,
    ) -> Result<(), Diagnostic> {
        let resolved_target = self.resolve_alias(target);
        if self.locals.contains_key(&resolved_target) {
            self.aliases.insert(name.to_string(), resolved_target);
            Ok(())
        } else {
            Err(Diagnostic::new(
                format!("unsupported reference to undefined variable `${target}` in LLVM codegen"),
                stmt_span(source),
            ))
        }
    }

    fn render_mir_loop_expr(
        &mut self,
        body: &mut String,
        statements: &[echo_mir::MirStmt],
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

        for statement in statements {
            self.render_mir_stmt(body, statement)?;
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

    fn render_mir_condition(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
    ) -> Result<String, Diagnostic> {
        match expr {
            echo_mir::MirExpr::Binary {
                source,
                left,
                op,
                right,
            } if matches!(op, BinaryOp::Is | BinaryOp::IsNot) => {
                if !matches!(right.as_ref(), echo_mir::MirExpr::Null { .. }) {
                    return Err(Diagnostic::new(
                        "unsupported non-null `is` comparison in LLVM codegen",
                        source.span(),
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, left)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let kind = format!("%value_kind_{call_id}");
                let is_null = format!("%is_null_{call_id}");
                let condition = format!("%condition_{call_id}");

                body.push_str(&format!("  {kind} = extractvalue {value}, 0\n"));
                body.push_str(&format!("  {is_null} = icmp eq i32 {kind}, 0\n"));

                if *op == BinaryOp::IsNot {
                    body.push_str(&format!("  {condition} = xor i1 {is_null}, true\n"));
                    Ok(condition)
                } else {
                    Ok(is_null)
                }
            }
            _ => {
                let value = self.render_mir_expr_as_echo_value(body, expr)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let condition = format!("%condition_{call_id}");
                body.push_str(&format!(
                    "  {condition} = call i1 @{}({value})\n",
                    CoreRuntimeSymbol::ValueBool.symbol()
                ));
                Ok(condition)
            }
        }
    }

    fn mir_userland_call(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<(), Diagnostic> {
        let Some(function) = self.functions.get(&call.name).cloned() else {
            return Err(Diagnostic::new(
                format!("unsupported function `{}` in LLVM codegen", call.name),
                call.span,
            ));
        };

        if call.args.len() != function.params.len() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for userland function `{}` in LLVM codegen",
                    call.name
                ),
                call.span,
            ));
        }

        let mut args = Vec::new();
        for arg in &call.args {
            args.push(self.render_mir_expr_as_echo_value(body, arg)?);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}({})\n",
            userland_function_symbol(&call.name),
            args.join(", ")
        ));

        Ok(())
    }

    fn render_mir_expr_as_echo_value(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
    ) -> Result<String, Diagnostic> {
        let value = self.render_mir_expr(body, expr)?;
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

    fn render_mir_dynamic_function_call(
        &mut self,
        body: &mut String,
        source: &Stmt,
        name: &str,
        args: &[echo_mir::MirExpr],
    ) -> Result<(), Diagnostic> {
        if !args.is_empty() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported arguments for dynamic function call `${name}` in LLVM codegen"
                ),
                stmt_span(source),
            ));
        }

        let RuntimeValue::StaticString(function_name) = self
            .locals
            .get(&self.resolve_alias(name))
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    format!("unsupported undefined dynamic function `${name}` in LLVM codegen"),
                    stmt_span(source),
                )
            })?
        else {
            return Err(Diagnostic::new(
                format!("unsupported non-string dynamic function `${name}` in LLVM codegen"),
                stmt_span(source),
            ));
        };

        let global = self.string_global(&function_name);
        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
            CoreRuntimeSymbol::CallFunction.symbol(),
            function_name.len()
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

    fn render_mir_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirExpr::Null { .. } => {
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::Bool { value, .. } => Ok(RuntimeValue::EchoValue(format!(
                "{{ i32 1, i64 {} }}",
                *value as u8
            ))),
            echo_mir::MirExpr::String { value, .. } => {
                Ok(RuntimeValue::StaticString(value.clone()))
            }
            echo_mir::MirExpr::Number { source, value } => {
                if let Ok(value) = value.parse::<i64>() {
                    Ok(RuntimeValue::EchoValue(format!("{{ i32 2, i64 {value} }}")))
                } else if let Ok(value) = value.parse::<f64>() {
                    Ok(RuntimeValue::EchoValue(format!(
                        "{{ i32 11, i64 {} }}",
                        value.to_bits() as i64
                    )))
                } else {
                    Err(Diagnostic::new(
                        "unsupported numeric literal in LLVM codegen",
                        source.span(),
                    ))
                }
            }
            echo_mir::MirExpr::Variable { source, name } => self
                .locals
                .get(&self.resolve_alias(name))
                .cloned()
                .ok_or_else(|| {
                    Diagnostic::new(
                        format!("unsupported undefined variable `${name}` in LLVM codegen"),
                        source.span(),
                    )
                }),
            echo_mir::MirExpr::FunctionCall { call, .. } => {
                self.render_mir_function_call_expr(body, call)
            }
            echo_mir::MirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } => self.render_mir_method_call_expr(body, object, method, args),
            echo_mir::MirExpr::StaticCall { args, .. } => {
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::Assign { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let resolved = self.resolve_alias(name);
                self.locals.insert(resolved, value.clone());
                Ok(value)
            }
            echo_mir::MirExpr::MagicDir { .. } => Ok(RuntimeValue::StaticString(
                self.source_dir.clone().unwrap_or_else(|| ".".to_string()),
            )),
            echo_mir::MirExpr::Require { once, path, .. } => {
                let path = self.render_mir_expr_as_echo_value(body, path)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let symbol = if *once {
                    CoreRuntimeSymbol::RequireOnce
                } else {
                    CoreRuntimeSymbol::Require
                };
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}({path})\n",
                    symbol.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirExpr::Defer { body: block, .. } => {
                let function = self.render_mir_defer_function(body, block)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::TaskDefer.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirExpr::Run { source, expr } => self.render_mir_run_expr(body, source, expr),
            echo_mir::MirExpr::Join { handle, .. } => {
                self.render_mir_task_unary_expr(body, handle, CoreRuntimeSymbol::Join)
            }
            echo_mir::MirExpr::Loop { body: block, .. } => self.render_mir_loop_expr(body, block),
            echo_mir::MirExpr::Fork { source, expr } => {
                self.render_mir_fork_expr(body, source, expr)
            }
            echo_mir::MirExpr::Spawn { command, .. } => {
                self.render_mir_task_unary_expr(body, command, CoreRuntimeSymbol::ProcessSpawn)
            }
            echo_mir::MirExpr::Binary {
                source,
                left,
                op,
                right,
            } if *op == BinaryOp::Concat => self.render_mir_concat_expr(body, left, right),
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Add => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueAdd)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Sub => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueSub)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Mul => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueMul)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Div => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueDiv)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Mod => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueMod)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Pow => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValuePow)
            }
            echo_mir::MirExpr::Unary { op, expr, .. } => self.render_mir_numeric_unary_expr(
                body,
                expr,
                match op {
                    UnaryOp::Plus => CoreRuntimeSymbol::ValueUnaryPlus,
                    UnaryOp::Minus => CoreRuntimeSymbol::ValueUnaryMinus,
                },
            ),
            echo_mir::MirExpr::Field { object, field, .. } => {
                let object = self.render_mir_expr_as_echo_value(body, object)?;
                let field_global = self.string_global(field);
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({object}, ptr @{field_global}, i64 {})\n",
                    CoreRuntimeSymbol::ValueObjectGet.symbol(),
                    field.len()
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            echo_mir::MirExpr::Index {
                collection, index, ..
            } => {
                let collection = self.render_mir_expr_as_echo_value(body, collection)?;
                let index = self.render_mir_expr_as_echo_value(body, index)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({collection}, {index})\n",
                    CoreRuntimeSymbol::ValueIndexGet.symbol()
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            echo_mir::MirExpr::Object { fields, .. } => self.render_mir_object_expr(body, fields),
            echo_mir::MirExpr::List { values, .. } => self.render_mir_list_values(body, values),
            echo_mir::MirExpr::Array { elements, .. } => self.render_mir_array_expr(body, elements),
            echo_mir::MirExpr::Binary { source, .. } => Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                source.span(),
            )),
        }
    }

    fn render_mir_array_expr(
        &mut self,
        body: &mut String,
        elements: &[echo_mir::MirArrayElement],
    ) -> Result<RuntimeValue, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut array = format!("%runtime_call_{call_id}");
        body.push_str(&format!(
            "  {array} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueArrayNew.symbol()
        ));

        for element in elements {
            let value = self.render_mir_expr_as_echo_value(body, &element.value)?;
            let append_id = self.next_call_id;
            self.next_call_id += 1;
            let appended = format!("%runtime_call_{append_id}");
            match &element.key {
                Some(key) => {
                    let key = self.render_mir_expr_as_echo_value(body, key)?;
                    body.push_str(&format!(
                        "  {appended} = call %EchoValue @{}(%EchoValue {array}, {key}, {value})\n",
                        CoreRuntimeSymbol::ValueArraySet.symbol()
                    ));
                }
                None => {
                    body.push_str(&format!(
                        "  {appended} = call %EchoValue @{}(%EchoValue {array}, {value})\n",
                        CoreRuntimeSymbol::ValueArrayAppend.symbol()
                    ));
                }
            }
            array = appended;
        }

        Ok(RuntimeValue::EchoValue(array))
    }

    fn render_mir_call_args_as_array(
        &mut self,
        body: &mut String,
        args: &[echo_mir::MirExpr],
    ) -> Result<String, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut array = format!("%runtime_call_{call_id}");
        body.push_str(&format!(
            "  {array} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueArrayNew.symbol()
        ));

        for arg in args {
            let value = self.render_mir_expr_as_echo_value(body, arg)?;
            let append_id = self.next_call_id;
            self.next_call_id += 1;
            let appended = format!("%runtime_call_{append_id}");
            body.push_str(&format!(
                "  {appended} = call %EchoValue @{}(%EchoValue {array}, {value})\n",
                CoreRuntimeSymbol::ValueArrayAppend.symbol()
            ));
            array = appended;
        }

        Ok(array)
    }

    fn render_mir_list_values(
        &mut self,
        body: &mut String,
        values: &[echo_mir::MirExpr],
    ) -> Result<RuntimeValue, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut list = format!("%runtime_call_{call_id}");
        body.push_str(&format!(
            "  {list} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueListNew.symbol()
        ));

        for value in values {
            let value = self.render_mir_expr_as_echo_value(body, value)?;
            let append_id = self.next_call_id;
            self.next_call_id += 1;
            let appended = format!("%runtime_call_{append_id}");
            body.push_str(&format!(
                "  {appended} = call %EchoValue @{}(%EchoValue {list}, {value})\n",
                CoreRuntimeSymbol::ValueListAppend.symbol()
            ));
            list = appended;
        }

        Ok(RuntimeValue::EchoValue(list))
    }

    fn render_mir_object_expr(
        &mut self,
        body: &mut String,
        fields: &[echo_mir::MirObjectField],
    ) -> Result<RuntimeValue, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut object = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {object} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueObjectNew.symbol()
        ));

        for field in fields {
            let field_global = self.string_global(&field.name);
            let value = self.render_mir_expr_as_echo_value(body, &field.value)?;
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

    fn render_mir_concat_expr(
        &mut self,
        body: &mut String,
        left: &echo_mir::MirExpr,
        right: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let left = self.render_mir_expr(body, left)?;
        let right = self.render_mir_expr(body, right)?;

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

    fn render_mir_numeric_binary_expr(
        &mut self,
        body: &mut String,
        left: &echo_mir::MirExpr,
        right: &echo_mir::MirExpr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let left = self.render_mir_expr_as_echo_value(body, left)?;
        let right = self.render_mir_expr_as_echo_value(body, right)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({left}, {right})\n",
            symbol.symbol()
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    fn render_mir_numeric_unary_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let value = self.render_mir_expr_as_echo_value(body, expr)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({value})\n",
            symbol.symbol()
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    fn render_mir_defer_function(
        &mut self,
        caller_body: &mut String,
        statements: &[echo_mir::MirStmt],
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

        let sleep = if let Some(echo_mir::MirStmt::FunctionCall { call, .. }) = statements.first()
            && self.resolve_std_call_name(&call.name, call.span)? == "time.sleep"
        {
            mir_task_sleep_millis(statements.first().expect("first statement exists"))
        } else {
            None
        };

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
                self.render_mir_defer_continuation_function(&function, &statements[1..])?;
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
                if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
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

    fn render_mir_defer_continuation_function(
        &mut self,
        parent: &str,
        statements: &[echo_mir::MirStmt],
    ) -> Result<String, Diagnostic> {
        let function = format!("{parent}_cont");
        let saved_returned = self.returned;
        self.returned = false;

        let mut body = String::new();
        for statement in statements {
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
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

    fn render_mir_run_expr(
        &mut self,
        body: &mut String,
        source: &echo_ast::Expr,
        expr: &echo_mir::MirRunExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirRunExpr::Block { body: block } => {
                let function = self.render_mir_defer_function(body, block)?;
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
            echo_mir::MirRunExpr::Task { expr } => {
                self.render_mir_task_unary_expr(body, expr, CoreRuntimeSymbol::TaskRun)
            }
            echo_mir::MirRunExpr::Group { entries } => {
                self.render_mir_run_group_expr(body, source, entries)
            }
        }
    }

    fn render_mir_run_group_expr(
        &mut self,
        body: &mut String,
        _source: &echo_ast::Expr,
        entries: &[Vec<echo_mir::MirStmt>],
    ) -> Result<RuntimeValue, Diagnostic> {
        let group_id = self.next_call_id;
        self.next_call_id += 1;
        let mut group = format!("%runtime_call_{group_id}");
        body.push_str(&format!(
            "  {group} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::TaskGroupNew.symbol()
        ));

        for entry in entries {
            let function = self.render_mir_defer_function(body, entry)?;
            let defer_id = self.next_call_id;
            self.next_call_id += 1;
            let task = format!("%runtime_call_{defer_id}");
            body.push_str(&format!(
                "  {task} = call %EchoValue @{}(ptr @{function})\n",
                CoreRuntimeSymbol::TaskDefer.symbol()
            ));

            let add_id = self.next_call_id;
            self.next_call_id += 1;
            let added = format!("%runtime_call_{add_id}");
            body.push_str(&format!(
                "  {added} = call %EchoValue @{}(%EchoValue {group}, %EchoValue {task})\n",
                CoreRuntimeSymbol::TaskGroupAdd.symbol()
            ));
            group = added;
        }

        let run_id = self.next_call_id;
        self.next_call_id += 1;
        body.push_str(&format!(
            "  %runtime_call_{run_id} = call %EchoValue @{}(%EchoValue {group})\n",
            CoreRuntimeSymbol::TaskGroupRunAndJoin.symbol()
        ));
        Ok(RuntimeValue::EchoValue(format!("%runtime_call_{run_id}")))
    }

    fn render_mir_fork_expr(
        &mut self,
        body: &mut String,
        _source: &echo_ast::Expr,
        expr: &echo_mir::MirForkExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirForkExpr::Block { body: block } => {
                let function = self.render_mir_defer_function(body, block)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::ThreadFork.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirForkExpr::Task { expr } => {
                self.render_mir_task_unary_expr(body, expr, CoreRuntimeSymbol::ThreadForkTask)
            }
        }
    }

    fn render_mir_task_unary_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let task = self.render_mir_expr_as_echo_value(body, expr)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}({task})\n",
            symbol.symbol()
        ));

        Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
    }

    fn render_mir_function_call_expr(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<RuntimeValue, Diagnostic> {
        let name = self.resolve_std_call_name(&call.name, call.span)?;

        if name == "time.sleep" {
            self.mir_time_sleep_call(body, &call.args, call.span)?;
            return Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()));
        }

        if let Some(intrinsic) = std_intrinsic(&name) {
            return self.mir_std_intrinsic_call(body, intrinsic, &call.args, call.span);
        }

        let Some(builtin) = php_builtin(&name) else {
            return self.render_mir_userland_function_call_expr(body, call);
        };

        match builtin.codegen {
            BuiltinCodegen::ObStart => {
                let (symbol, arg) = match call.args.as_slice() {
                    [] => (builtin.symbol, None),
                    [echo_mir::MirExpr::Null { .. }] => (
                        builtin
                            .helper_symbol
                            .expect("ob_start value helper must be declared"),
                        Some("%EchoValue { i32 0, i64 0 }".to_string()),
                    ),
                    [echo_mir::MirExpr::String { value, .. }] => {
                        let helper = builtin
                            .helper_symbol
                            .expect("ob_start value helper must be declared");
                        let global = self.string_global(value);
                        let value_id = self.next_call_id;
                        self.next_call_id += 1;
                        let value_name = format!("%runtime_call_{value_id}");

                        body.push_str(&format!(
                            "  {value_name} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
                            CoreRuntimeSymbol::ValueString.symbol(),
                            value.len()
                        ));

                        (helper, Some(format!("%EchoValue {value_name}")))
                    }
                    [arg] => {
                        return Err(Diagnostic::new(
                            "unsupported ob_start callback argument in LLVM codegen",
                            arg.syntax().span(),
                        ));
                    }
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported ob_start argument count in LLVM codegen",
                            call.span,
                        ));
                    }
                };

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let bool_name = format!("%runtime_bool_{call_id}");
                let payload_name = format!("%runtime_bool_payload_{call_id}");
                let value_name = format!("%runtime_call_{call_id}");

                match arg {
                    Some(arg) => {
                        body.push_str(&format!("  {bool_name} = call i1 @{symbol}({arg})\n"))
                    }
                    None => body.push_str(&format!("  {bool_name} = call i1 @{symbol}()\n")),
                }
                body.push_str(&format!("  {payload_name} = zext i1 {bool_name} to i64\n"));
                body.push_str(&format!(
                    "  {value_name} = insertvalue %EchoValue {{ i32 1, i64 0 }}, i64 {payload_name}, 1\n"
                ));

                Ok(RuntimeValue::EchoValue(value_name))
            }
            BuiltinCodegen::VoidStatement => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                body.push_str(&format!("  call void @{}()\n", builtin.symbol));
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            BuiltinCodegen::VoidUnaryStatement => {
                let [arg] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let arg = self.render_mir_expr_as_echo_value(body, arg)?;
                body.push_str(&format!("  call void @{}({arg})\n", builtin.symbol));
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            BuiltinCodegen::BoolStatement => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let bool_name = format!("%runtime_bool_{call_id}");
                let payload_name = format!("%runtime_bool_payload_{call_id}");
                let value_name = format!("%runtime_call_{call_id}");

                body.push_str(&format!("  {bool_name} = call i1 @{}()\n", builtin.symbol));
                body.push_str(&format!("  {payload_name} = zext i1 {bool_name} to i64\n"));
                body.push_str(&format!(
                    "  {value_name} = insertvalue %EchoValue {{ i32 1, i64 0 }}, i64 {payload_name}, 1\n"
                ));

                Ok(RuntimeValue::EchoValue(value_name))
            }
            BuiltinCodegen::ValueExpression => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
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
                let [arg] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let arg = self.render_mir_expr_as_echo_value(body, arg)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({arg})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Getenv => {
                if call.args.len() > 2 {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let name_arg = if let Some(arg) = call.args.first() {
                    self.render_mir_expr_as_echo_value(body, arg)?
                } else {
                    "%EchoValue { i32 0, i64 0 }".to_string()
                };
                let local_only_arg = if let Some(arg) = call.args.get(1) {
                    self.render_mir_expr_as_echo_value(body, arg)?
                } else {
                    "%EchoValue { i32 1, i64 0 }".to_string()
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({name_arg}, {local_only_arg})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Basename => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let path = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let suffix = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString(String::new()),
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({path}, {suffix})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Dirname => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let path = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let levels = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({path}, {levels})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueBinaryExpression => {
                let [left, right] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let left = self.render_mir_expr_as_echo_value(body, left)?;
                let right = self.render_mir_expr_as_echo_value(body, right)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({left}, {right})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryOptionalBoolExpression => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let flag = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {flag})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Nl2br => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let use_xhtml = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {use_xhtml})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryOptionalBoolContextExpression => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let flag = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let context = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {flag}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryOptionalContextExpression => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let context = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueBinaryOptionalContextExpression => {
                if !(2..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let left = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let right = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let context = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({left}, {right}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::FileGetContents => {
                if !(1..=5).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let filename = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let use_include_path = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let context = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let offset = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 0 }".to_string(),
                };
                let length = match call.args.get(4) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({filename}, {use_include_path}, {context}, {offset}, {length})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::FilePutContents => {
                if !(2..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let filename = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let data = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let flags = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 0 }".to_string(),
                };
                let context = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({filename}, {data}, {flags}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Uniqid => {
                if call.args.len() > 2 {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let prefix = match call.args.first() {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString(String::new()),
                    ),
                };
                let more_entropy = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({prefix}, {more_entropy})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Mkdir => {
                if !(1..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let directory = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let permissions = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 511 }".to_string(),
                };
                let recursive = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let context = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({directory}, {permissions}, {recursive}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Touch => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let filename = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let mtime = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let atime = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({filename}, {mtime}, {atime})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArrayKeys => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let array = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let filter_value = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 6, i64 0 }".to_string(),
                };
                let strict = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({array}, {filter_value}, {strict})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArrayReverse => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let array = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let preserve_keys = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({array}, {preserve_keys})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArraySlice => {
                if !(2..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let array = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let offset = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let length = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let preserve_keys = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({array}, {offset}, {length}, {preserve_keys})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArrayMerge | BuiltinCodegen::ArrayReplace => {
                if builtin.codegen == BuiltinCodegen::ArrayReplace && call.args.is_empty() {
                    return Err(Diagnostic::new(
                        "unsupported argument count for builtin `array_replace` in LLVM codegen",
                        call.span,
                    ));
                }

                let arrays = self.render_mir_call_args_as_array(body, &call.args)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}(%EchoValue {arrays})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::InArray => {
                if !(2..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let needle = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let haystack = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let strict = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({needle}, {haystack}, {strict})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Log => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let base = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => format!(
                        "%EchoValue {{ i32 11, i64 {} }}",
                        std::f64::consts::E.to_bits() as i64
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {base})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ChunkSplit => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 76 }".to_string(),
                };
                let separator = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString("\r\n".to_string()),
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length}, {separator})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::StrPad => {
                if !(2..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let pad_string = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString(" ".to_string()),
                    ),
                };
                let pad_type = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length}, {pad_string}, {pad_type})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::StrSplit => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueTernaryExpression => {
                let [first, second, third] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let first = self.render_mir_expr_as_echo_value(body, first)?;
                let second = self.render_mir_expr_as_echo_value(body, second)?;
                let third = self.render_mir_expr_as_echo_value(body, third)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({first}, {second}, {third})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Explode => {
                if !(2..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let separator = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let string = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let limit = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 9223372036854775807 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({separator}, {string}, {limit})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Implode => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let (separator, array) = match call.args.as_slice() {
                    [array] => (
                        self.runtime_value_as_echo_value(
                            body,
                            RuntimeValue::StaticString(String::new()),
                        ),
                        self.render_mir_expr_as_echo_value(body, array)?,
                    ),
                    [separator, array] => (
                        self.render_mir_expr_as_echo_value(body, separator)?,
                        self.render_mir_expr_as_echo_value(body, array)?,
                    ),
                    _ => unreachable!("argument count checked above"),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({separator}, {array})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::SubstrCompare => {
                if !(3..=5).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let haystack = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let needle = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let offset = self.render_mir_expr_as_echo_value(body, &call.args[2])?;
                let length = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let case_insensitive = match call.args.get(4) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({haystack}, {needle}, {offset}, {length}, {case_insensitive})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
        }
    }

    fn render_mir_userland_function_call_expr(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<RuntimeValue, Diagnostic> {
        let Some(function) = self.functions.get(&call.name).cloned() else {
            return Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                call.span,
            ));
        };

        if call.args.len() != function.params.len() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for userland function `{}` in LLVM codegen",
                    call.name
                ),
                call.span,
            ));
        }

        let mut args = Vec::new();
        for arg in &call.args {
            args.push(self.render_mir_expr_as_echo_value(body, arg)?);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({})\n",
            userland_function_symbol(&call.name),
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

    fn register_std_import(&mut self, statement: &echo_ast::ImportStmt) -> Result<(), Diagnostic> {
        if statement.source != ImportSource::Std {
            return Ok(());
        }

        let Some(module) = statement.name.parts.first() else {
            return Err(Diagnostic::new(
                "empty std import in LLVM codegen",
                statement.span,
            ));
        };

        if !is_known_std_module(module) {
            return Err(Diagnostic::new(
                format!("unknown std import `{}`", statement.name.as_string()),
                statement.span,
            ));
        }

        if statement.name.parts.len() == 1 {
            let local = statement.alias.as_deref().unwrap_or(module).to_string();
            self.std_imports.insert(local, module.clone());
        }

        Ok(())
    }

    fn resolve_std_call_name(&self, name: &str, span: Span) -> Result<String, Diagnostic> {
        let Some((module, rest)) = name.split_once('.') else {
            return Ok(name.to_string());
        };

        if let Some(imported) = self.std_imports.get(module) {
            return Ok(format!("{imported}.{rest}"));
        }

        if is_known_std_module(module) {
            return Err(Diagnostic::new(
                format!("std module `{module}` must be imported before use"),
                span,
            ));
        }

        Ok(name.to_string())
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
    let mut seen = HashSet::new();

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
        .filter(|declaration| seen.insert(declaration.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn function_params_signature(params: &[TypedParam]) -> String {
    params
        .iter()
        .map(|param| match &param.ty {
            Some(ty) => format!("{ty} ${}", param.name),
            None => format!("${}", param.name),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_known_std_module(name: &str) -> bool {
    let module_name = format!("std.{name}");
    echo_std::modules()
        .iter()
        .any(|module| module.name == module_name)
}

fn userland_function_symbol(name: &str) -> String {
    format!("echo_user_{name}")
}

fn mir_task_sleep_millis(statement: &echo_mir::MirStmt) -> Option<i64> {
    let echo_mir::MirStmt::FunctionCall { call, .. } = statement else {
        return None;
    };
    if call.name != "time.sleep" {
        return None;
    }
    let [echo_mir::MirExpr::Number { value, .. }] = call.args.as_slice() else {
        return None;
    };

    value.parse().ok()
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
mod tests;
