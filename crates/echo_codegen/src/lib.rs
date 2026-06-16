use echo_ast::{BinaryOp, Expr, FunctionDeclStmt, Program, Stmt};
use echo_diagnostics::Diagnostic;
use echo_runtime::RuntimeFn;
use echo_source::Span;
use inkwell::context::Context;
use std::collections::HashMap;

#[derive(Clone)]
enum RuntimeValue {
    StaticString(String),
    I64(String),
    I64OrFalse(String),
    RuntimeString(String),
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
        RuntimeFn::Shutdown.symbol(),
    ))
}

struct IrModule {
    globals: String,
    functions_ir: String,
    aliases: HashMap<String, String>,
    locals: HashMap<String, RuntimeValue>,
    functions: HashMap<String, FunctionDeclStmt>,
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
        if !function.params.is_empty() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported parameters for userland function `{}` in LLVM codegen",
                    function.name
                ),
                function.span,
            ));
        }

        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let mut body = String::new();

        for statement in &function.body {
            if let Err(diagnostic) = self.render_stmt(&mut body, statement) {
                self.aliases = saved_aliases;
                self.locals = saved_locals;
                return Err(diagnostic);
            }
        }

        self.aliases = saved_aliases;
        self.locals = saved_locals;

        self.functions_ir.push_str(&format!(
            "define void @{}() {{\nentry:\n{}  ret void\n}}\n",
            userland_function_symbol(&function.name),
            body
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
            Stmt::FunctionCall(statement) => match runtime_function_for_call(&statement.name) {
                Some(function) => self.runtime_call(body, function, &statement.args)?,
                None => self.userland_call(body, statement)?,
            },
            Stmt::FunctionDecl(_) => {}
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

        if !statement.args.is_empty() || !function.params.is_empty() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported arguments for userland function `{}` in LLVM codegen",
                    statement.name
                ),
                statement.span,
            ));
        }

        body.push_str(&format!(
            "  call void @{}()\n",
            userland_function_symbol(&statement.name)
        ));

        Ok(())
    }

    fn write_call(&mut self, body: &mut String, value: &str) {
        let global = self.string_global(value);
        body.push_str(&format!(
            "  call void @{}(ptr @{global}, i64 {})\n",
            RuntimeFn::EchoWrite.symbol(),
            value.len()
        ));
    }

    fn write_value(&mut self, body: &mut String, value: RuntimeValue) {
        match value {
            RuntimeValue::StaticString(value) => self.write_call(body, &value),
            RuntimeValue::I64(name) => body.push_str(&format!(
                "  call void @{}(i64 {name})\n",
                RuntimeFn::EchoWriteI64.symbol()
            )),
            RuntimeValue::I64OrFalse(name) => body.push_str(&format!(
                "  call void @{}(i64 {name})\n",
                RuntimeFn::EchoWriteI64OrFalse.symbol()
            )),
            RuntimeValue::RuntimeString(name) => body.push_str(&format!(
                "  call void @{}(ptr {name})\n",
                RuntimeFn::EchoWriteString.symbol()
            )),
        }
    }

    fn runtime_call(
        &mut self,
        body: &mut String,
        function: RuntimeFn,
        args: &[Expr],
    ) -> Result<(), Diagnostic> {
        match function {
            RuntimeFn::ObStart => match args {
                [] => body.push_str(&format!("  call void @{}()\n", function.symbol())),
                [Expr::Null(_)] => {
                    let call_id = self.next_call_id;
                    self.next_call_id += 1;

                    body.push_str(&format!(
                        "  %runtime_call_{call_id} = call i1 @{}(%EchoValue {{ i32 0, i64 0 }})\n",
                        RuntimeFn::ObStartValue.symbol()
                    ));
                }
                [Expr::String(expr)] => {
                    let global = self.string_global(&expr.value);
                    let value_id = self.next_call_id;
                    self.next_call_id += 1;
                    let start_id = self.next_call_id;
                    self.next_call_id += 1;

                    body.push_str(&format!(
                        "  %runtime_call_{value_id} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
                        RuntimeFn::EchoValueString.symbol(),
                        expr.value.len()
                    ));
                    body.push_str(&format!(
                        "  %runtime_call_{start_id} = call i1 @{}(%EchoValue %runtime_call_{value_id})\n",
                        RuntimeFn::ObStartValue.symbol()
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
            RuntimeFn::ObClean
            | RuntimeFn::ObFlush
            | RuntimeFn::ObEndFlush
            | RuntimeFn::ObEndClean => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;

                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call i1 @{}()\n",
                    function.symbol()
                ));
            }
            RuntimeFn::ObGetClean | RuntimeFn::ObGetFlush => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;

                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call ptr @{}()\n",
                    function.symbol()
                ));
            }
            RuntimeFn::EchoWrite => unreachable!("echo_write needs string arguments"),
            RuntimeFn::EchoWriteI64 => unreachable!("echo_write_i64 needs an i64 argument"),
            RuntimeFn::EchoWriteI64OrFalse => {
                unreachable!("echo_write_i64_or_false needs an i64 argument")
            }
            RuntimeFn::EchoWriteString => unreachable!("echo_write_string needs a string argument"),
            RuntimeFn::EchoValueString => unreachable!("echo_value_string needs string arguments"),
            RuntimeFn::ObStartValue => unreachable!("ob_start_value needs an EchoValue argument"),
            RuntimeFn::ObGetContents => unreachable!("ob_get_contents is emitted as an expression"),
            RuntimeFn::ObGetLength => unreachable!("ob_get_length is emitted as an expression"),
            RuntimeFn::ObGetLevel => unreachable!("ob_get_level is emitted as an expression"),
            RuntimeFn::Shutdown => unreachable!("shutdown is emitted at program exit"),
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
            Expr::FunctionCall(expr) if expr.name == "ob_get_level" => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call i64 @{}()\n",
                    RuntimeFn::ObGetLevel.symbol()
                ));

                Ok(RuntimeValue::I64(name))
            }
            Expr::FunctionCall(expr) if expr.name == "ob_get_length" => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call i64 @{}()\n",
                    RuntimeFn::ObGetLength.symbol()
                ));

                Ok(RuntimeValue::I64OrFalse(name))
            }
            Expr::FunctionCall(expr) if expr.name == "ob_get_contents" => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call ptr @{}()\n",
                    RuntimeFn::ObGetContents.symbol()
                ));

                Ok(RuntimeValue::RuntimeString(name))
            }
            Expr::FunctionCall(expr) if expr.name == "ob_get_clean" => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call ptr @{}()\n",
                    RuntimeFn::ObGetClean.symbol()
                ));

                Ok(RuntimeValue::RuntimeString(name))
            }
            Expr::FunctionCall(expr) if expr.name == "ob_get_flush" => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call ptr @{}()\n",
                    RuntimeFn::ObGetFlush.symbol()
                ));

                Ok(RuntimeValue::RuntimeString(name))
            }
            Expr::Binary(expr) if expr.op == BinaryOp::Concat => {
                let RuntimeValue::StaticString(mut output) = self.render_expr(body, &expr.left)?
                else {
                    return Err(Diagnostic::new(
                        "unsupported dynamic concat expression in LLVM codegen",
                        expr.left.span(),
                    ));
                };
                let RuntimeValue::StaticString(right) = self.render_expr(body, &expr.right)? else {
                    return Err(Diagnostic::new(
                        "unsupported dynamic concat expression in LLVM codegen",
                        expr.right.span(),
                    ));
                };

                output.push_str(&right);
                Ok(RuntimeValue::StaticString(output))
            }
            _ => Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                expr.span(),
            )),
        }
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
    RuntimeFn::ALL
        .iter()
        .map(|function| function.llvm_decl())
        .collect::<Vec<_>>()
        .join("\n")
}

fn runtime_function_for_call(name: &str) -> Option<RuntimeFn> {
    match name {
        "ob_start" => Some(RuntimeFn::ObStart),
        "ob_clean" => Some(RuntimeFn::ObClean),
        "ob_flush" => Some(RuntimeFn::ObFlush),
        "ob_end_flush" => Some(RuntimeFn::ObEndFlush),
        "ob_end_clean" => Some(RuntimeFn::ObEndClean),
        "ob_get_clean" => Some(RuntimeFn::ObGetClean),
        "ob_get_flush" => Some(RuntimeFn::ObGetFlush),
        _ => None,
    }
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
    use echo_ast::{EchoStmt, FunctionCallStmt, FunctionDeclStmt, NullLiteral, StringLiteral};

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
        assert!(ir.contains("declare i1 @echo_ob_start_value(%EchoValue)"));
        assert!(
            ir.contains("call i1 @echo_ob_start_value(%EchoValue { i32 0, i64 0 })"),
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
        assert!(ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_0, i64 6)"));
        assert!(ir.contains("call i1 @echo_ob_start_value(%EchoValue %runtime_call_0)"));
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

        assert!(ir.contains("define void @echo_user_say_after()"), "{ir}");
        assert!(
            ir.contains("call void @echo_write(ptr @echo_str_0, i64 6)"),
            "{ir}"
        );
        assert!(ir.contains("call void @echo_user_say_after()"), "{ir}");
    }
}
