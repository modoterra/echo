use echo_ast::{BinaryOp, Expr, Program, Stmt};
use echo_diagnostics::Diagnostic;
use echo_runtime::RuntimeFn;
use inkwell::context::Context;
use std::collections::HashMap;

#[derive(Clone)]
enum RuntimeValue {
    StaticString(String),
    I64(String),
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
        body,
        RuntimeFn::Shutdown.symbol(),
    ))
}

struct IrModule {
    globals: String,
    aliases: HashMap<String, String>,
    locals: HashMap<String, RuntimeValue>,
    next_string_id: usize,
    next_call_id: usize,
}

impl IrModule {
    fn new() -> Self {
        Self {
            globals: String::new(),
            aliases: HashMap::new(),
            locals: HashMap::new(),
            next_string_id: 0,
            next_call_id: 0,
        }
    }

    fn render_program(&mut self, program: &Program) -> Result<String, Vec<Diagnostic>> {
        let mut body = String::new();
        let mut diagnostics = Vec::new();

        for statement in &program.statements {
            match statement {
                Stmt::Echo(statement) => {
                    for expr in &statement.exprs {
                        match self.render_expr(&mut body, expr) {
                            Ok(value) => self.write_value(&mut body, value),
                            Err(diagnostic) => diagnostics.push(diagnostic),
                        }
                    }
                }
                Stmt::FunctionCall(statement) => match runtime_function_for_call(&statement.name) {
                    Some(function) => self.runtime_call(&mut body, function),
                    None => diagnostics.push(Diagnostic::new(
                        format!("unsupported function `{}` in LLVM codegen", statement.name),
                        statement.span,
                    )),
                },
                Stmt::Assign(statement) => match self.render_expr(&mut body, &statement.value) {
                    Ok(value) => {
                        // PHP assignments copy values by default; references are handled separately.
                        // Source: https://www.php.net/manual/en/language.operators.assignment.php
                        let name = self.resolve_alias(&statement.name);
                        self.locals.insert(name, value);
                    }
                    Err(diagnostic) => diagnostics.push(diagnostic),
                },
                Stmt::AssignRef(statement) => {
                    let target = self.resolve_alias(&statement.target);
                    if self.locals.contains_key(&target) {
                        // PHP references make two variable names aliases for the same value cell.
                        // Source: https://www.php.net/manual/en/language.references.php
                        self.aliases.insert(statement.name.clone(), target);
                    } else {
                        diagnostics.push(Diagnostic::new(
                            format!(
                                "unsupported reference to undefined variable `${}` in LLVM codegen",
                                statement.target
                            ),
                            statement.span,
                        ));
                    }
                }
            }
        }

        if diagnostics.is_empty() {
            Ok(body)
        } else {
            Err(diagnostics)
        }
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
            RuntimeValue::RuntimeString(name) => body.push_str(&format!(
                "  call void @{}(ptr {name})\n",
                RuntimeFn::EchoWriteString.symbol()
            )),
        }
    }

    fn runtime_call(&mut self, body: &mut String, function: RuntimeFn) {
        match function {
            RuntimeFn::ObStart => {
                body.push_str(&format!("  call void @{}()\n", function.symbol()));
            }
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
            RuntimeFn::EchoWrite => unreachable!("echo_write needs string arguments"),
            RuntimeFn::EchoWriteI64 => unreachable!("echo_write_i64 needs an i64 argument"),
            RuntimeFn::EchoWriteString => unreachable!("echo_write_string needs a string argument"),
            RuntimeFn::ObGetContents => unreachable!("ob_get_contents is emitted as an expression"),
            RuntimeFn::ObGetLength => unreachable!("ob_get_length is emitted as an expression"),
            RuntimeFn::ObGetLevel => unreachable!("ob_get_level is emitted as an expression"),
            RuntimeFn::Shutdown => unreachable!("shutdown is emitted at program exit"),
        }
    }

    fn render_expr(&mut self, body: &mut String, expr: &Expr) -> Result<RuntimeValue, Diagnostic> {
        match expr {
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

                Ok(RuntimeValue::I64(name))
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
        _ => None,
    }
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
