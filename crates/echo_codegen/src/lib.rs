use echo_ast::{BinaryOp, Expr, Program, Stmt};
use echo_diagnostics::Diagnostic;
use echo_runtime::RuntimeFn;
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
    next_string_id: usize,
    next_call_id: usize,
}

impl IrModule {
    fn new() -> Self {
        Self {
            globals: String::new(),
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
                        match render_expr(expr) {
                            Ok(value) => self.write_call(&mut body, &value),
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
            RuntimeFn::Shutdown => unreachable!("shutdown is emitted at program exit"),
        }
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
