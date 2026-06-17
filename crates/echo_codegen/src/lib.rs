mod abi;

use abi::{
    BuiltinCodegen, BuiltinLowering, CoreRuntimeSymbol, PHP_BUILTINS, PHP_RUNTIME_HELPERS,
    PhpBuiltin, php_builtin,
};
use echo_ast::{BinaryOp, Expr, FunctionDeclStmt, Program, Stmt};
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
    next_string_id: usize,
    next_call_id: usize,
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
            next_string_id: 0,
            next_call_id: 0,
        }
    }

    fn render_program(&mut self, program: &Program) -> Result<String, Vec<Diagnostic>> {
        let mut body = String::new();
        let mut diagnostics = Vec::new();

        for statement in &program.statements {
            if let Stmt::FunctionDecl(statement) = statement {
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
        self.returned = false;

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
                return Err(diagnostic);
            }
        }

        let returned = self.returned;

        self.aliases = saved_aliases;
        self.locals = saved_locals;
        self.returned = saved_returned;

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
            Stmt::FunctionCall(statement) => match php_builtin(&statement.name) {
                Some(builtin) if builtin.lowering == BuiltinLowering::DirectRuntimeCall => {
                    self.php_builtin_call(body, builtin, &statement.args)?
                }
                None => self.userland_call(body, statement)?,
                Some(_) => self.userland_call(body, statement)?,
            },
            Stmt::DynamicFunctionCall(statement) => self.dynamic_function_call(body, statement)?,
            Stmt::FunctionDecl(_) => {}
            Stmt::Namespace(_) | Stmt::Use(_) | Stmt::Import(_) | Stmt::ClassDecl(_) => {}
            Stmt::Return(statement) => {
                let value = self.render_expr_as_echo_value(body, &statement.value)?;
                body.push_str(&format!("  ret {value}\n"));
                self.returned = true;
            }
            Stmt::Assign(statement) => {
                let value = self.render_expr(body, &statement.value)?;
                // PHP assignments copy values by default; references are handled separately.
                // Source: https://www.php.net/manual/en/language.operators.assignment.php
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
        }

        Ok(())
    }

    fn render_expr(&mut self, body: &mut String, expr: &Expr) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            Expr::Null(expr) => Err(Diagnostic::new(
                "unsupported null expression in LLVM codegen",
                expr.span,
            )),
            Expr::String(expr) => Ok(RuntimeValue::StaticString(expr.value.clone())),
            Expr::Number(expr) => Ok(RuntimeValue::StaticString(expr.value.clone())),
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
            Expr::Defer(_) => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}()\n",
                    CoreRuntimeSymbol::TaskDefer.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            Expr::Run(_) | Expr::Fork(_) | Expr::Spawn(_) | Expr::Join(_) => {
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            Expr::Binary(expr) if expr.op == BinaryOp::Concat => {
                self.render_concat_expr(body, &expr.left, &expr.right)
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

    fn render_function_call_expr(
        &mut self,
        body: &mut String,
        expr: &echo_ast::FunctionCallExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
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
        .collect::<Vec<_>>()
        .join("\n")
}

fn userland_function_symbol(name: &str) -> String {
    format!("echo_user_{name}")
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

        assert!(ir.contains("declare %EchoValue @echo_task_defer()"), "{ir}");
        assert!(ir.contains("call %EchoValue @echo_task_defer()"), "{ir}");
        assert!(ir.contains("ret i32 0"), "{ir}");
    }
}
