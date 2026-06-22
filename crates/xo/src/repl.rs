use std::collections::HashMap;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::{Child, Command as ProcessCommand};

use echo_ast::{BinaryOp, EchoStmt, Expr, FunctionCallExpr, Program, Stmt};
use echo_source::SourceFile;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::source::{ModeOverride, parse_source_program, print_diagnostics, source_file_from_text};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_YELLOW: &str = "\x1b[33m";

pub fn run_repl(mode: ModeOverride) {
    let interactive = io::stdin().is_terminal();
    let mut session = ReplSession::default();

    if interactive {
        run_interactive_repl(&mut session, mode);
    } else {
        run_piped_repl(&mut session, mode);
    }
}

fn run_interactive_repl(session: &mut ReplSession, mode: ModeOverride) {
    let mut editor = DefaultEditor::new().unwrap_or_else(|err| {
        eprintln!("error: failed to initialize REPL editor: {err}");
        std::process::exit(1);
    });
    let history_path = repl_history_path();
    if let Some(path) = history_path.as_deref() {
        let _ = editor.load_history(path);
    }

    println!("{ANSI_DIM}Echo REPL. Use :quit or :exit to leave.{ANSI_RESET}");

    loop {
        let line = match editor.readline(&repl_prompt()) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("error: failed to read stdin: {err}");
                std::process::exit(1);
            }
        };

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input == ":quit" || input == ":exit" {
            break;
        }

        let _ = editor.add_history_entry(input);
        run_repl_input(session, input, mode, true);
    }

    if let Some(path) = history_path.as_deref() {
        let _ = editor.save_history(path);
    }
}

fn run_piped_repl(session: &mut ReplSession, mode: ModeOverride) {
    let mut line = String::new();

    loop {
        line.clear();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(err) => {
                eprintln!("error: failed to read stdin: {err}");
                std::process::exit(1);
            }
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input == ":quit" || input == ":exit" {
            break;
        }

        run_repl_input(session, input, mode, false);
    }
}

#[derive(Debug, Default)]
struct ReplSession {
    statements: Vec<Stmt>,
    processes: HashMap<String, Child>,
}

fn run_repl_input(session: &mut ReplSession, input: &str, mode: ModeOverride, interactive: bool) {
    let file = PathBuf::from("repl.echo");
    let source = source_file_from_text(file.clone(), input.to_string(), mode);

    let mut parsed = match try_parse_repl_input(&source) {
        Ok(parsed) => parsed,
        Err(diagnostics) => {
            print_diagnostics(diagnostics);
            return;
        }
    };

    if let Some(output) = run_repl_process_handle_input(session, &parsed.program, &mut parsed.input)
    {
        print_repl_output(&output, parsed.input, interactive);
        return;
    }

    let mut program = parsed.program.clone();
    let current_statements = program.statements.clone();
    let mut statements = session.statements.clone();
    statements.extend(program.statements);
    program.statements = statements;

    let analysis = match echo_semantics::analyze(&program) {
        Ok(analysis) => analysis,
        Err(diagnostics) => {
            if interactive {
                print_repl_metadata(&parsed.input);
            }
            print_diagnostics(diagnostics);
            return;
        }
    };
    apply_repl_semantics(&mut parsed.input, &analysis);

    let ir = match echo_codegen::compile_to_ir(&program) {
        Ok(ir) => ir,
        Err(diagnostics) => {
            if interactive {
                print_repl_metadata(&parsed.input);
            }
            print_diagnostics(diagnostics);
            return;
        }
    };

    let output = match run_repl_jit(&ir, matches!(parsed.input, ReplInput::Expression(_))) {
        Ok(output) => output,
        Err(diagnostics) => {
            if interactive {
                print_repl_metadata(&parsed.input);
            }
            print_diagnostics(diagnostics);
            return;
        }
    };
    let should_store = output.status == 0 && matches!(parsed.input, ReplInput::Statement);
    print_repl_output(&output, parsed.input, interactive);
    if should_store {
        session.statements.extend(
            current_statements
                .into_iter()
                .filter(is_repl_persistent_statement),
        );
    }
}

fn repl_prompt() -> String {
    format!("{ANSI_GREEN}xo){ANSI_RESET} ")
}

fn repl_history_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .map(|home| home.join(".xo_history"))
}

fn run_repl_process_handle_input(
    session: &mut ReplSession,
    program: &Program,
    input: &mut ReplInput,
) -> Option<ReplExecutionOutput> {
    if let Some((name, command)) = repl_spawn_assignment(program) {
        match spawn_shell_command(&command) {
            Ok(child) => {
                session.processes.insert(name, child);
                return Some(ReplExecutionOutput {
                    status: 0,
                    stdout: Vec::new(),
                });
            }
            Err(err) => {
                eprintln!("error: failed to spawn process: {err}");
                return Some(ReplExecutionOutput {
                    status: 1,
                    stdout: Vec::new(),
                });
            }
        }
    }

    let ReplInput::Expression(_) = input else {
        return None;
    };

    match repl_process_handle_expr(program) {
        Some(ReplProcessInput::Join(name)) => {
            let Some(mut child) = session.processes.remove(&name) else {
                eprintln!("error: join target `${name}` is not a live process handle");
                return Some(ReplExecutionOutput {
                    status: 1,
                    stdout: Vec::new(),
                });
            };

            let status = match child.wait() {
                Ok(status) => status.code().unwrap_or_default(),
                Err(err) => {
                    eprintln!("error: failed to join process `${name}`: {err}");
                    return Some(ReplExecutionOutput {
                        status: 1,
                        stdout: Vec::new(),
                    });
                }
            };

            apply_repl_live_process_type(input, "int");
            Some(ReplExecutionOutput {
                status: 0,
                stdout: status.to_string().into_bytes(),
            })
        }
        Some(ReplProcessInput::Inspect(name)) if session.processes.contains_key(&name) => {
            apply_repl_live_process_type(input, "process");
            Some(ReplExecutionOutput {
                status: 0,
                stdout: b"Object".to_vec(),
            })
        }
        _ => None,
    }
}

fn apply_repl_live_process_type(input: &mut ReplInput, ty: &str) {
    if let ReplInput::Expression(info) = input {
        info.static_type = ty.to_string();
    }
}

fn repl_spawn_assignment(program: &Program) -> Option<(String, String)> {
    let [Stmt::Assign(statement)] = program.statements.as_slice() else {
        return None;
    };
    let Expr::Spawn(spawn) = &statement.value else {
        return None;
    };
    let Expr::String(command) = spawn.command.as_ref() else {
        return None;
    };

    Some((statement.name.clone(), command.value.clone()))
}

enum ReplProcessInput {
    Join(String),
    Inspect(String),
}

fn repl_process_handle_expr(program: &Program) -> Option<ReplProcessInput> {
    let [Stmt::Echo(statement)] = program.statements.as_slice() else {
        return None;
    };
    let [expr] = statement.exprs.as_slice() else {
        return None;
    };

    match expr {
        Expr::Join(join) => match join.handle.as_ref() {
            Expr::Variable(variable) => Some(ReplProcessInput::Join(variable.name.clone())),
            _ => None,
        },
        Expr::Variable(variable) => Some(ReplProcessInput::Inspect(variable.name.clone())),
        _ => None,
    }
}

fn spawn_shell_command(command: &str) -> std::io::Result<Child> {
    if cfg!(windows) {
        ProcessCommand::new("cmd").arg("/C").arg(command).spawn()
    } else {
        ProcessCommand::new("sh").arg("-c").arg(command).spawn()
    }
}

fn is_repl_persistent_statement(statement: &Stmt) -> bool {
    matches!(
        statement,
        Stmt::Assign(_)
            | Stmt::Let(_)
            | Stmt::AssignRef(_)
            | Stmt::Namespace(_)
            | Stmt::Use(_)
            | Stmt::Import(_)
            | Stmt::FunctionDecl(_)
            | Stmt::ClassDecl(_)
            | Stmt::TypeDecl(_)
            | Stmt::Append(_)
    )
}

#[derive(Debug)]
struct ReplParsed {
    program: Program,
    input: ReplInput,
}

#[derive(Debug)]
enum ReplInput {
    Statement,
    Expression(ExpressionInfo),
}

#[derive(Debug)]
struct ExpressionInfo {
    kind: &'static str,
    static_type: String,
    span: echo_source::Span,
}

fn try_parse_repl_input(
    source: &SourceFile,
) -> Result<ReplParsed, Vec<echo_diagnostics::Diagnostic>> {
    let mut program = parse_source_program(source)?;

    let mut input = ReplInput::Statement;
    if let Some(expr) = repl_display_expr(program.statements.as_slice()) {
        let span = expr.span();
        input = ReplInput::Expression(expression_info(&expr));
        program.statements = vec![Stmt::Echo(EchoStmt {
            exprs: vec![expr],
            span,
        })];
        program.span = span;
    }

    Ok(ReplParsed { program, input })
}

fn repl_display_expr(statements: &[Stmt]) -> Option<Expr> {
    match statements {
        [Stmt::Expr(statement)] => Some(statement.expr.clone()),
        [Stmt::FunctionCall(statement)] => Some(Expr::FunctionCall(FunctionCallExpr {
            name: statement.name.clone(),
            args: statement.args.clone(),
            span: statement.span,
        })),
        _ => None,
    }
}

#[derive(Debug)]
struct ReplExecutionOutput {
    status: i32,
    stdout: Vec<u8>,
}

fn run_repl_jit(
    ir: &str,
    inspect_expression: bool,
) -> Result<ReplExecutionOutput, Vec<echo_diagnostics::Diagnostic>> {
    let output = echo_codegen::run_ir_jit_with_options(
        ir,
        echo_codegen::JitOptions {
            capture_stdout: true,
            repl_inspect: inspect_expression,
        },
    )?;

    if output.status != 0 {
        eprintln!("error: JIT main exited with status {}", output.status);
    }

    Ok(ReplExecutionOutput {
        status: output.status,
        stdout: output.stdout,
    })
}

fn print_repl_output(output: &ReplExecutionOutput, input: ReplInput, interactive: bool) {
    if !interactive {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        return;
    }

    match input {
        ReplInput::Statement => {
            print!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stdout.is_empty() && !output.stdout.ends_with(b"\n") {
                println!();
            }
        }
        ReplInput::Expression(info) => {
            let value = String::from_utf8_lossy(&output.stdout);
            println!("{ANSI_CYAN}=>{ANSI_RESET} {value}");
            print_expression_metadata(&info);
        }
    }
}

fn print_repl_metadata(input: &ReplInput) {
    if let ReplInput::Expression(info) = input {
        print_expression_metadata(info);
    }
}

fn apply_repl_semantics(input: &mut ReplInput, analysis: &echo_semantics::Analysis) {
    if let ReplInput::Expression(info) = input {
        info.static_type = analysis.expression_type_at(info.span).display_name();
    }
}

fn print_expression_metadata(info: &ExpressionInfo) {
    println!(
        "{ANSI_DIM}   kind:{ANSI_RESET} {}  {ANSI_DIM}type:{ANSI_RESET} {}  {ANSI_DIM}span:{ANSI_RESET} {}..{}",
        colorize_expr_kind(info.kind),
        colorize_type(&info.static_type),
        info.span.start,
        info.span.end
    );
}

fn expression_info(expr: &Expr) -> ExpressionInfo {
    ExpressionInfo {
        kind: expression_kind(expr),
        static_type: expression_static_type(expr),
        span: expr.span(),
    }
}

fn expression_kind(expr: &Expr) -> &'static str {
    match expr {
        Expr::Null(_) => "null literal",
        Expr::Bool(_) => "bool literal",
        Expr::String(_) => "string literal",
        Expr::Number(_) => "number literal",
        Expr::Variable(_) => "variable",
        Expr::FunctionCall(_) => "function call",
        Expr::MethodCall(_) => "method call",
        Expr::StaticCall(_) => "static call",
        Expr::Assign(_) => "assignment expression",
        Expr::MagicConstant(_) => "magic constant",
        Expr::Require(_) => "require expression",
        Expr::Defer(_) => "defer expression",
        Expr::Run(_) => "run expression",
        Expr::Fork(_) => "fork expression",
        Expr::Spawn(_) => "spawn expression",
        Expr::Join(_) => "join expression",
        Expr::Loop(_) => "loop expression",
        Expr::Unary(expr) => match expr.op {
            echo_ast::UnaryOp::Plus => "numeric identity expression",
            echo_ast::UnaryOp::Minus => "negate expression",
        },
        Expr::Binary(expr) => match expr.op {
            BinaryOp::Add => "add expression",
            BinaryOp::Sub => "subtract expression",
            BinaryOp::Mul => "multiply expression",
            BinaryOp::Div => "divide expression",
            BinaryOp::Mod => "modulo expression",
            BinaryOp::Pow => "exponent expression",
            BinaryOp::Concat => "concat expression",
            BinaryOp::Is | BinaryOp::IsNot => "null test expression",
        },
        Expr::TypeAscription(_) => "type ascription expression",
        Expr::Field(_) => "field expression",
        Expr::Index(_) => "index expression",
        Expr::Object(_) => "object expression",
        Expr::List(_) => "list expression",
        Expr::Array(_) => "array expression",
    }
}

fn expression_static_type(expr: &Expr) -> String {
    match expr {
        Expr::Null(_) => "null".to_string(),
        Expr::Bool(_) => "bool".to_string(),
        Expr::String(_) => "string".to_string(),
        Expr::Number(_) => "int".to_string(),
        Expr::List(_) => "list".to_string(),
        Expr::Array(_) => "array".to_string(),
        Expr::Object(_) => "object".to_string(),
        Expr::Unary(_) => "number".to_string(),
        Expr::Binary(expr) => match expr.op {
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Pow => "number".to_string(),
            BinaryOp::Concat => "string".to_string(),
            BinaryOp::Is | BinaryOp::IsNot => "bool".to_string(),
        },
        Expr::FunctionCall(call) => echo_reflection::function(&call.name)
            .and_then(|function| function.return_type.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        Expr::MethodCall(_) | Expr::StaticCall(_) => "unknown".to_string(),
        Expr::TypeAscription(expr) => expr.ty.clone(),
        Expr::Assign(expr) => expression_static_type(&expr.value),
        Expr::MagicConstant(_) => "string".to_string(),
        Expr::Require(_) => "bool".to_string(),
        Expr::Defer(_) | Expr::Run(_) => "task".to_string(),
        Expr::Fork(_) => "thread".to_string(),
        Expr::Spawn(_) => "process".to_string(),
        Expr::Variable(_) | Expr::Index(_) | Expr::Join(_) | Expr::Loop(_) | Expr::Field(_) => {
            "unknown".to_string()
        }
    }
}

fn colorize_expr_kind(kind: &str) -> String {
    format!("{ANSI_MAGENTA}{kind}{ANSI_RESET}")
}

fn colorize_type(ty: &str) -> String {
    let color = match ty {
        "int" => ANSI_YELLOW,
        "string" => ANSI_GREEN,
        "bool" => ANSI_BLUE,
        "null" => ANSI_DIM,
        _ => ANSI_CYAN,
    };

    format!("{color}{ty}{ANSI_RESET}")
}

#[cfg(test)]
#[path = "repl/tests.rs"]
mod tests;
