use echo_ast::{BinaryOp, Expr, Program, Stmt};
use echo_diagnostics::Diagnostic;
use inkwell::context::Context;

pub fn backend_name() -> &'static str {
    "llvm"
}

pub fn smoke_test_module_ir() -> String {
    let context = Context::create();
    let module = context.create_module("echo_smoke");

    module.print_to_string().to_string()
}

pub fn compile_to_ir(program: &Program) -> Result<String, Vec<Diagnostic>> {
    let output = render_program_output(program)?;
    let escaped = llvm_string_literal(&output);
    let byte_len = output.len() + 1;

    Ok(format!(
        r#"@echo_output = private unnamed_addr constant [{byte_len} x i8] c"{escaped}\00", align 1

declare i32 @printf(ptr, ...)

define i32 @main() {{
entry:
  %printed = call i32 (ptr, ...) @printf(ptr @echo_output)
  ret i32 0
}}
"#
    ))
}

fn render_program_output(program: &Program) -> Result<String, Vec<Diagnostic>> {
    let mut runtime = OutputRuntime::default();
    let mut diagnostics = Vec::new();

    for statement in &program.statements {
        match statement {
            Stmt::Echo(statement) => {
                for expr in &statement.exprs {
                    match render_expr(expr) {
                        Ok(value) => runtime.echo(&value),
                        Err(diagnostic) => diagnostics.push(diagnostic),
                    }
                }
            }
            Stmt::FunctionCall(statement) => {
                if let Err(diagnostic) = runtime.call(&statement.name, statement.span) {
                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(runtime.finish())
    } else {
        Err(diagnostics)
    }
}

#[derive(Default)]
struct OutputRuntime {
    output: String,
    buffer: Option<String>,
}

impl OutputRuntime {
    fn echo(&mut self, value: &str) {
        match &mut self.buffer {
            Some(buffer) => buffer.push_str(value),
            None => self.output.push_str(value),
        }
    }

    fn call(&mut self, name: &str, span: echo_source::Span) -> Result<(), Diagnostic> {
        match name {
            "ob_start" => {
                if self.buffer.is_some() {
                    return Err(Diagnostic::new(
                        "nested output buffers are not supported yet",
                        span,
                    ));
                }

                self.buffer = Some(String::new());
                Ok(())
            }
            "ob_flush" => {
                let Some(buffer) = &self.buffer else {
                    return Err(Diagnostic::new("no active output buffer", span));
                };

                self.output.push_str(buffer);
                Ok(())
            }
            "ob_end_flush" => {
                let Some(buffer) = self.buffer.take() else {
                    return Err(Diagnostic::new("no active output buffer", span));
                };

                self.output.push_str(&buffer);
                Ok(())
            }
            _ => Err(Diagnostic::new(
                format!("unsupported function `{name}` in LLVM codegen"),
                span,
            )),
        }
    }

    fn finish(self) -> String {
        self.output
    }
}

fn render_expr(expr: &Expr) -> Result<String, Diagnostic> {
    match expr {
        Expr::String(expr) => Ok(expr.value.clone()),
        Expr::Number(expr) => Ok(expr.value.clone()),
        Expr::Binary(expr) if expr.op == BinaryOp::Concat => {
            let mut output = render_expr(&expr.left)?;
            output.push_str(&render_expr(&expr.right)?);
            Ok(output)
        }
        _ => Err(Diagnostic::new(
            "unsupported expression in LLVM codegen",
            expr.span(),
        )),
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
