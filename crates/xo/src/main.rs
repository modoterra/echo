use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use echo_ast::{BinaryOp, EchoStmt, Expr, FunctionCallExpr, Program, Stmt};
use echo_source::{SourceFile, SourceMode};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

static TEMP_PATH_COUNTER: AtomicU64 = AtomicU64::new(0);

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_YELLOW: &str = "\x1b[33m";

#[derive(Debug, Parser)]
#[command(name = "xo")]
#[command(about = "Echo language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Lex {
        file: PathBuf,
    },
    Ast {
        #[command(flatten)]
        mode: ModeOverride,
        file: PathBuf,
    },
    Ir {
        #[command(flatten)]
        mode: ModeOverride,
        file: PathBuf,
    },
    Run {
        #[command(flatten)]
        mode: ModeOverride,
        file: PathBuf,
    },
    Repl {
        #[command(flatten)]
        mode: ModeOverride,
    },
    Test {
        #[command(flatten)]
        mode: ModeOverride,
        path: PathBuf,
    },
    Build {
        #[command(flatten)]
        mode: ModeOverride,
        #[command(flatten)]
        optimization: OptimizationOptions,
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        emit_ir: bool,
    },
}

#[derive(Debug, Clone, Copy, clap::Args)]
struct ModeOverride {
    /// Force strict mode, rejecting unsafe PHP compatibility patterns.
    #[arg(long, conflicts_with = "unsafe_mode")]
    strict: bool,

    /// Force Echo unsafe/superset mode, allowing PHP compatibility patterns.
    #[arg(long = "unsafe", conflicts_with = "strict")]
    unsafe_mode: bool,
}

impl ModeOverride {
    fn apply(self, default: SourceMode) -> SourceMode {
        if self.strict {
            SourceMode::Strict
        } else if self.unsafe_mode {
            SourceMode::Echo
        } else {
            default
        }
    }
}

#[derive(Debug, Clone, Copy, clap::Args)]
struct OptimizationOptions {
    /// LLVM optimization level for build output.
    #[arg(short = 'O', default_value = "0", value_parser = parse_optimization_level)]
    level: OptimizationLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptimizationLevel {
    O0,
    O1,
    O2,
    O3,
    Oz,
}

impl OptimizationLevel {
    const fn clang_arg(self) -> &'static str {
        match self {
            Self::O0 => "-O0",
            Self::O1 => "-O1",
            Self::O2 => "-O2",
            Self::O3 => "-O3",
            Self::Oz => "-Oz",
        }
    }

    const fn opt_pipeline(self) -> Option<&'static str> {
        match self {
            Self::O0 => None,
            Self::O1 => Some("default<O1>"),
            Self::O2 => Some("default<O2>"),
            Self::O3 => Some("default<O3>"),
            Self::Oz => Some("default<Oz>"),
        }
    }
}

fn parse_optimization_level(value: &str) -> Result<OptimizationLevel, String> {
    match value {
        "0" => Ok(OptimizationLevel::O0),
        "1" => Ok(OptimizationLevel::O1),
        "2" => Ok(OptimizationLevel::O2),
        "3" => Ok(OptimizationLevel::O3),
        "z" | "Z" => Ok(OptimizationLevel::Oz),
        _ => Err(format!("expected one of 0, 1, 2, 3, or z, got `{value}`")),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Lex { file } => {
            let source = read_source(&file);

            match echo_lexer::lex(&source) {
                Ok(tokens) => {
                    for token in tokens {
                        println!("{token:#?}");
                    }
                }
                Err(diagnostics) => {
                    print_diagnostics(diagnostics);
                    std::process::exit(1);
                }
            }
        }
        Command::Ast { mode, file } => {
            let source = read_source_file(&file, mode);

            match echo_parser::parse_with_mode(&source.text, source.mode) {
                Ok(program) => {
                    println!("{program:#?}");
                }
                Err(diagnostics) => {
                    print_diagnostics(diagnostics);
                    std::process::exit(1);
                }
            }
        }
        Command::Ir { mode, file } => {
            let ir = compile_ir(&file, mode);
            print!("{ir}");
        }
        Command::Run { mode, file } => {
            let binary_path = temp_path(&file, "program");
            build_binary(&file, mode, OptimizationLevel::O0, &binary_path);
            run_command(&mut ProcessCommand::new(&binary_path));
            let _ = fs::remove_file(binary_path);
        }
        Command::Repl { mode } => {
            run_repl(mode);
        }
        Command::Test { mode, path } => {
            run_tests(&path, mode);
        }
        Command::Build {
            mode,
            optimization,
            file,
            output,
            emit_ir,
        } => {
            if emit_ir {
                let ir = compile_ir(&file, mode);
                verify_ir(&file, &ir);
                print!("{}", optimize_ir(&file, &ir, optimization.level));
            } else {
                let Some(output) = output else {
                    eprintln!("error: xo build requires -o/--output unless --emit-ir is used");
                    std::process::exit(1);
                };

                build_binary(&file, mode, optimization.level, &output);
            }
        }
    }
}

fn run_repl(mode: ModeOverride) {
    let interactive = io::stdin().is_terminal();
    let mut session = ReplSession::default();

    ensure_runtime_library_quiet();

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
}

fn run_repl_input(session: &mut ReplSession, input: &str, mode: ModeOverride, interactive: bool) {
    let file = PathBuf::from("repl.echo");
    let source = source_file_from_text(file.clone(), input.to_string(), mode);

    let parsed = match try_parse_repl_input(&source) {
        Ok(parsed) => parsed,
        Err(diagnostics) => {
            print_diagnostics(diagnostics);
            return;
        }
    };

    let mut program = parsed.program.clone();
    let current_statements = program.statements.clone();
    let mut statements = session.statements.clone();
    statements.extend(program.statements);
    program.statements = statements;

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

    verify_ir(&file, &ir);
    let binary_path = temp_path(&file, "program");
    build_ir_binary_quiet(&file, &ir, OptimizationLevel::O0, &binary_path);
    let output = run_repl_binary(&binary_path);
    let should_store = output.status.success() && matches!(parsed.input, ReplInput::Statement);
    print_repl_output(&output, parsed.input, interactive);
    if should_store {
        session.statements.extend(
            current_statements
                .into_iter()
                .filter(is_repl_persistent_statement),
        );
    }
    let _ = fs::remove_file(binary_path);
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

fn run_tests(path: &PathBuf, mode: ModeOverride) {
    let tests = collect_test_files(path);
    if tests.is_empty() {
        eprintln!("error: no .echo tests found in {}", path.display());
        std::process::exit(1);
    }

    let mut failures = 0;
    for test in tests {
        let binary_path = temp_path(&test, "test");
        build_binary(&test, mode, OptimizationLevel::O0, &binary_path);
        let output = ProcessCommand::new(&binary_path)
            .output()
            .unwrap_or_else(|err| {
                eprintln!("error: failed to run {}: {err}", binary_path.display());
                std::process::exit(1);
            });
        let _ = fs::remove_file(&binary_path);

        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));

        if output.status.success() {
            println!("ok {}", test.display());
        } else {
            failures += 1;
            eprintln!("FAILED {}", test.display());
        }
    }

    if failures > 0 {
        eprintln!("{failures} test(s) failed");
        std::process::exit(1);
    }
}

fn collect_test_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut tests = Vec::new();
    collect_test_files_into(path, &mut tests);
    tests.sort();
    tests
}

fn collect_test_files_into(path: &PathBuf, tests: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().and_then(|extension| extension.to_str()) == Some("echo") {
            tests.push(path.clone());
        }
        return;
    }

    let entries = fs::read_dir(path).unwrap_or_else(|err| {
        eprintln!("error: failed to read {}: {err}", path.display());
        std::process::exit(1);
    });

    for entry in entries {
        let path = entry
            .unwrap_or_else(|err| {
                eprintln!("error: failed to read test directory entry: {err}");
                std::process::exit(1);
            })
            .path();
        if path.is_dir() {
            collect_test_files_into(&path, tests);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("echo") {
            tests.push(path);
        }
    }
}

fn build_binary(
    file: &PathBuf,
    mode: ModeOverride,
    optimization: OptimizationLevel,
    output: &PathBuf,
) {
    let ir = compile_ir(file, mode);
    verify_ir(file, &ir);
    build_ir_binary(file, &ir, optimization, output);
}

fn build_ir_binary(file: &PathBuf, ir: &str, optimization: OptimizationLevel, output: &PathBuf) {
    ensure_runtime_library();
    build_ir_binary_quiet(file, ir, optimization, output);
}

fn build_ir_binary_quiet(
    file: &PathBuf,
    ir: &str,
    optimization: OptimizationLevel,
    output: &PathBuf,
) {
    let ir_path = write_temp_ir(file, &ir);
    run_command(
        ProcessCommand::new("clang")
            .arg(optimization.clang_arg())
            .arg("-x")
            .arg("ir")
            .arg(&ir_path)
            .arg("-x")
            .arg("none")
            .arg(runtime_library_path())
            .arg("-o")
            .arg(output),
    );
    let _ = fs::remove_file(ir_path);
}

fn verify_ir(file: &PathBuf, ir: &str) {
    let ir_path = write_temp_ir(file, ir);
    let output = ProcessCommand::new("opt")
        .arg("-disable-output")
        .arg("-passes=verify")
        .arg(&ir_path)
        .output()
        .unwrap_or_else(|err| {
            eprintln!("error: failed to run opt verifier: {err}");
            std::process::exit(1);
        });
    let _ = fs::remove_file(ir_path);

    if !output.status.success() {
        eprintln!("error: generated LLVM IR failed verification");
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn optimize_ir(file: &PathBuf, ir: &str, optimization: OptimizationLevel) -> String {
    let Some(pipeline) = optimization.opt_pipeline() else {
        return ir.to_string();
    };

    let ir_path = write_temp_ir(file, ir);
    let output = ProcessCommand::new("opt")
        .arg("-S")
        .arg(format!("-passes={pipeline}"))
        .arg(&ir_path)
        .output()
        .unwrap_or_else(|err| {
            eprintln!("error: failed to run opt: {err}");
            std::process::exit(1);
        });
    let _ = fs::remove_file(ir_path);

    if !output.status.success() {
        eprintln!("error: opt exited with {}", output.status);
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }

    String::from_utf8(output.stdout).unwrap_or_else(|err| {
        eprintln!("error: opt emitted non-UTF-8 IR: {err}");
        std::process::exit(1);
    })
}

fn compile_ir(file: &PathBuf, mode: ModeOverride) -> String {
    let source = read_source_file(file, mode);

    match try_compile_ir(&source) {
        Ok(ir) => ir,
        Err(diagnostics) => {
            print_diagnostics(diagnostics);
            std::process::exit(1);
        }
    }
}

fn try_parse_repl_input(
    source: &SourceFile,
) -> Result<ReplParsed, Vec<echo_diagnostics::Diagnostic>> {
    let mut program = match echo_parser::parse_with_mode(&source.text, source.mode) {
        Ok(program) => program,
        Err(diagnostics) => return Err(diagnostics),
    };

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

fn try_compile_ir(source: &SourceFile) -> Result<String, Vec<echo_diagnostics::Diagnostic>> {
    let program = match echo_parser::parse_with_mode(&source.text, source.mode) {
        Ok(program) => program,
        Err(diagnostics) => return Err(diagnostics),
    };

    echo_codegen::compile_to_ir(&program)
}

fn write_temp_ir(file: &PathBuf, ir: &str) -> PathBuf {
    let path = create_temp_file(file, "ll", ir.as_bytes());

    path
}

fn create_temp_file(file: &PathBuf, extension: &str, contents: &[u8]) -> PathBuf {
    for _ in 0..100 {
        let path = temp_path(file, extension);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut temp_file) => {
                temp_file.write_all(contents).unwrap_or_else(|err| {
                    eprintln!("error: failed to write {}: {err}", path.display());
                    std::process::exit(1);
                });
                return path;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                eprintln!("error: failed to create {}: {err}", path.display());
                std::process::exit(1);
            }
        }
    }

    eprintln!("error: failed to allocate unique temporary {extension} path");
    std::process::exit(1);
}

fn temp_path(file: &PathBuf, extension: &str) -> PathBuf {
    let stem = file
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("program");

    let counter = TEMP_PATH_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);

    std::env::temp_dir().join(format!(
        "echo-{stem}-{}-{nanos}-{counter}.{extension}",
        std::process::id(),
    ))
}

fn runtime_library_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xo should be two levels below the workspace root")
        .join("target/debug/libecho_runtime.a")
}

fn ensure_runtime_library() {
    run_command(
        ProcessCommand::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("echo_runtime"),
    );
}

fn ensure_runtime_library_quiet() {
    let output = ProcessCommand::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("echo_runtime")
        .output()
        .unwrap_or_else(|err| {
            eprintln!("error: failed to run cargo build -p echo_runtime: {err}");
            std::process::exit(1);
        });

    if !output.status.success() {
        eprintln!(
            "error: cargo build -p echo_runtime exited with {}",
            output.status
        );
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn run_repl_binary(binary_path: &PathBuf) -> std::process::Output {
    let output = ProcessCommand::new(binary_path)
        .output()
        .unwrap_or_else(|err| {
            eprintln!("error: failed to run {}: {err}", binary_path.display());
            std::process::exit(1);
        });

    if !output.status.success() {
        eprintln!(
            "error: {} exited with {}",
            binary_path.display(),
            output.status
        );
    }

    output
}

fn print_repl_output(output: &std::process::Output, input: ReplInput, interactive: bool) {
    eprint!("{}", String::from_utf8_lossy(&output.stderr));

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
        Expr::Defer(_) => "defer expression",
        Expr::Run(_) => "run expression",
        Expr::Fork(_) => "fork expression",
        Expr::Spawn(_) => "spawn expression",
        Expr::Join(_) => "join expression",
        Expr::Loop(_) => "loop expression",
        Expr::Binary(expr) => match expr.op {
            BinaryOp::Add => "add expression",
            BinaryOp::Sub => "subtract expression",
            BinaryOp::Mul => "multiply expression",
            BinaryOp::Div => "divide expression",
            BinaryOp::Concat => "concat expression",
            BinaryOp::Is | BinaryOp::IsNot => "null test expression",
        },
        Expr::Field(_) => "field expression",
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
        Expr::Binary(expr) => match expr.op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => "int".to_string(),
            BinaryOp::Concat => "string".to_string(),
            BinaryOp::Is | BinaryOp::IsNot => "bool".to_string(),
        },
        Expr::FunctionCall(call) => echo_reflection::function(&call.name)
            .and_then(|function| function.return_type.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        Expr::Variable(_)
        | Expr::Defer(_)
        | Expr::Run(_)
        | Expr::Fork(_)
        | Expr::Spawn(_)
        | Expr::Join(_)
        | Expr::Loop(_)
        | Expr::Field(_) => "unknown".to_string(),
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

fn run_command(command: &mut ProcessCommand) {
    let status = command.status().unwrap_or_else(|err| {
        eprintln!("error: failed to run {command:?}: {err}");
        std::process::exit(1);
    });

    if !status.success() {
        eprintln!("error: {command:?} exited with {status}");
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn read_source(file: &PathBuf) -> String {
    fs::read_to_string(file).unwrap_or_else(|err| {
        eprintln!("error: failed to read {}: {err}", file.display());
        std::process::exit(1);
    })
}

fn read_source_file(file: &PathBuf, mode: ModeOverride) -> SourceFile {
    source_file_from_text(file.clone(), read_source(file), mode)
}

fn source_file_from_text(file: PathBuf, text: String, mode: ModeOverride) -> SourceFile {
    let mut source = SourceFile::new(file, text);
    source.mode = mode.apply(source.mode);
    source
}

fn print_diagnostics(diagnostics: Vec<echo_diagnostics::Diagnostic>) {
    for diagnostic in diagnostics {
        eprintln!(
            "error: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_ir_files_are_unique_for_repeated_builds() {
        let source = PathBuf::from("program.php");
        let first = create_temp_file(&source, "ll", b"first");
        let second = create_temp_file(&source, "ll", b"second");

        assert_ne!(first, second);
        assert_eq!(fs::read(&first).expect("first temp file"), b"first");
        assert_eq!(fs::read(&second).expect("second temp file"), b"second");

        let _ = fs::remove_file(first);
        let _ = fs::remove_file(second);
    }

    #[test]
    fn parses_optimization_levels() {
        assert_eq!(parse_optimization_level("0"), Ok(OptimizationLevel::O0));
        assert_eq!(parse_optimization_level("1"), Ok(OptimizationLevel::O1));
        assert_eq!(parse_optimization_level("2"), Ok(OptimizationLevel::O2));
        assert_eq!(parse_optimization_level("3"), Ok(OptimizationLevel::O3));
        assert_eq!(parse_optimization_level("z"), Ok(OptimizationLevel::Oz));
        assert!(parse_optimization_level("4").is_err());
    }

    #[test]
    fn repl_source_defaults_to_strict_mode() {
        let source = source_file_from_text(
            PathBuf::from("repl.echo"),
            "echo \"hello\";".to_string(),
            ModeOverride {
                strict: false,
                unsafe_mode: false,
            },
        );

        assert_eq!(source.mode, SourceMode::Strict);
    }

    #[test]
    fn repl_source_can_use_unsafe_mode() {
        let source = source_file_from_text(
            PathBuf::from("repl.echo"),
            "<?php echo \"hello\";".to_string(),
            ModeOverride {
                strict: false,
                unsafe_mode: true,
            },
        );

        assert_eq!(source.mode, SourceMode::Echo);
    }

    #[test]
    fn repl_prompt_uses_xo_paren_with_ansi_color() {
        assert_eq!(repl_prompt(), "\x1b[32mxo)\x1b[0m ");
    }

    #[test]
    fn repl_expression_info_describes_addition() {
        let source = source_file_from_text(
            PathBuf::from("repl.echo"),
            "5+3".to_string(),
            ModeOverride {
                strict: false,
                unsafe_mode: false,
            },
        );
        let parsed = try_parse_repl_input(&source).expect("expression should parse");
        let ReplInput::Expression(info) = parsed.input else {
            panic!("bare expression should be classified as expression input");
        };

        assert_eq!(info.kind, "add expression");
        assert_eq!(info.static_type, "int");
        assert_eq!(info.span.start, 0);
        assert_eq!(info.span.end, 3);
    }

    #[test]
    fn repl_expression_info_reflects_bare_function_call_return_type() {
        let source = source_file_from_text(
            PathBuf::from("repl.echo"),
            "is_float(42)".to_string(),
            ModeOverride {
                strict: false,
                unsafe_mode: false,
            },
        );
        let parsed = try_parse_repl_input(&source).expect("function call should parse");
        let ReplInput::Expression(info) = parsed.input else {
            panic!("bare function call should be classified as expression input");
        };

        assert_eq!(info.kind, "function call");
        assert_eq!(info.static_type, "bool");
        assert_eq!(info.span.start, 0);
        assert_eq!(info.span.end, 12);
    }

    #[test]
    fn repl_expression_info_distinguishes_collection_literals() {
        let cases = [
            ("{1, 2}", "list expression", "list"),
            ("[1, 2]", "array expression", "array"),
            ("{ test: 5 }", "object expression", "object"),
        ];

        for (source, expected_kind, expected_type) in cases {
            let source = source_file_from_text(
                PathBuf::from("repl.echo"),
                source.to_string(),
                ModeOverride {
                    strict: false,
                    unsafe_mode: false,
                },
            );
            let parsed = try_parse_repl_input(&source).expect("expression should parse");
            let ReplInput::Expression(info) = parsed.input else {
                panic!("bare expression should be classified as expression input");
            };

            assert_eq!(info.kind, expected_kind);
            assert_eq!(info.static_type, expected_type);
        }
    }
}
