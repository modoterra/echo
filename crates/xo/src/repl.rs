use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::{Child, Command as ProcessCommand};

use echo_ast::{BinaryOp, EchoStmt, Expr, FunctionCallExpr, Program, Stmt};
use echo_source::SourceFile;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Editor, Helper};

use crate::source::{
    SourceOptions, parse_source_program, print_diagnostics, source_file_from_text,
};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_YELLOW: &str = "\x1b[33m";

pub fn run_repl(mode: SourceOptions) {
    let interactive = io::stdin().is_terminal();
    let mut session = ReplSession::default();

    if interactive {
        run_interactive_repl(&mut session, mode);
    } else {
        run_piped_repl(&mut session, mode);
    }
}

fn run_interactive_repl(session: &mut ReplSession, mode: SourceOptions) {
    let mut editor: Editor<ReplHelper, DefaultHistory> = Editor::new().unwrap_or_else(|err| {
        eprintln!("error: failed to initialize REPL editor: {err}");
        std::process::exit(1);
    });
    editor.set_helper(Some(ReplHelper::new(mode)));
    let history_path = repl_history_path();
    if let Some(path) = history_path.as_deref() {
        let _ = editor.load_history(path);
    }

    println!("{ANSI_DIM}Echo REPL. Use :quit or :exit to leave.{ANSI_RESET}");

    let mut pending = String::new();
    let mut pending_brace_depth = 0i32;

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

        if pending.is_empty() && (input == ":quit" || input == ":exit") {
            break;
        }

        let _ = editor.add_history_entry(input);
        if let Some(input) = buffer_repl_input(input, &mut pending, &mut pending_brace_depth) {
            run_repl_input(session, &input, mode, true);
            if let Some(helper) = editor.helper_mut() {
                helper.set_statements(session.statements.clone());
            }
        }
    }

    if let Some(path) = history_path.as_deref() {
        let _ = editor.save_history(path);
    }
}

struct ReplHelper {
    mode: SourceOptions,
    statements: Vec<Stmt>,
}

impl ReplHelper {
    fn new(mode: SourceOptions) -> Self {
        Self {
            mode,
            statements: Vec::new(),
        }
    }

    fn set_statements(&mut self, statements: Vec<Stmt>) {
        self.statements = statements;
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;
}

impl Helper for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        if pos < line.len() {
            return None;
        }

        repl_live_hint(line, self.mode, &self.statements)
    }
}

impl Highlighter for ReplHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("{ANSI_DIM}{hint}{ANSI_RESET}"))
    }
}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        if brace_delta(ctx.input()) > 0 {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

fn run_piped_repl(session: &mut ReplSession, mode: SourceOptions) {
    let mut line = String::new();
    let mut pending = String::new();
    let mut pending_brace_depth = 0i32;

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

        if pending.is_empty() && (input == ":quit" || input == ":exit") {
            break;
        }

        if let Some(input) = buffer_repl_input(input, &mut pending, &mut pending_brace_depth) {
            run_repl_input(session, &input, mode, false);
        }
    }
}

fn buffer_repl_input(input: &str, pending: &mut String, brace_depth: &mut i32) -> Option<String> {
    if pending.is_empty() && brace_delta(input) <= 0 {
        return Some(input.to_string());
    }

    if !pending.is_empty() {
        pending.push('\n');
    }
    pending.push_str(input);
    *brace_depth += brace_delta(input);

    if *brace_depth > 0 {
        return None;
    }

    *brace_depth = 0;
    Some(std::mem::take(pending))
}

fn brace_delta(input: &str) -> i32 {
    let mut delta = 0;
    let mut chars = input.chars();
    let mut quote = None;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }

        if quote.is_some() && ch == '\\' {
            escaped = true;
            continue;
        }

        match quote {
            Some(current) if ch == current => quote = None,
            Some(_) => {}
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch == '{' => delta += 1,
            None if ch == '}' => delta -= 1,
            None => {}
        }
    }

    delta
}

#[derive(Debug, Default)]
struct ReplSession {
    statements: Vec<Stmt>,
    processes: HashMap<String, Child>,
}

fn run_repl_input(session: &mut ReplSession, input: &str, mode: SourceOptions, interactive: bool) {
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
            | Stmt::Compile(_)
            | Stmt::Namespace(_)
            | Stmt::Use(_)
            | Stmt::Import(_)
            | Stmt::UnnamedExport(_)
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

fn repl_live_hint(line: &str, mode: SourceOptions, session_statements: &[Stmt]) -> Option<String> {
    let input = line.trim();
    if input.is_empty() || input.starts_with(':') || brace_delta(input) > 0 {
        return None;
    }

    let file = PathBuf::from("repl.echo");
    let source = source_file_from_text(file, input.to_string(), mode);
    let program = parse_source_program(&source).ok()?;
    let expr = repl_display_expr(program.statements.as_slice())?;
    let ty = live_expression_type(&program, &expr, session_statements);

    match const_eval_expr(&expr, session_statements) {
        Some(value) => Some(format!("  => {} {ty}", value.display())),
        None => Some(format!("  => {ty}")),
    }
}

fn live_expression_type(program: &Program, expr: &Expr, session_statements: &[Stmt]) -> String {
    let mut program = program.clone();
    let mut statements = session_statements.to_vec();
    statements.extend(program.statements);
    program.statements = statements;

    echo_semantics::analyze(&program)
        .map(|analysis| analysis.expression_type_at(expr.span()).display_name())
        .map(|ty| {
            if ty == "unknown" {
                expression_static_type(expr)
            } else {
                ty
            }
        })
        .unwrap_or_else(|_| expression_static_type(expr))
}

#[derive(Debug, Clone, PartialEq)]
enum ConstValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl ConstValue {
    fn display(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::String(value) => format!("{value:?}"),
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            Self::Int(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }
}

fn const_eval_expr(expr: &Expr, session_statements: &[Stmt]) -> Option<ConstValue> {
    match expr {
        Expr::Null(_) => Some(ConstValue::Null),
        Expr::Bool(value) => Some(ConstValue::Bool(value.value)),
        Expr::String(value) => Some(ConstValue::String(value.value.clone())),
        Expr::Number(value) => {
            if value.value.contains('.') || value.value.contains('e') || value.value.contains('E') {
                value.value.parse().ok().map(ConstValue::Float)
            } else {
                value.value.parse().ok().map(ConstValue::Int)
            }
        }
        Expr::Variable(variable) => const_eval_variable(&variable.name, session_statements),
        Expr::Constant(constant) if constant.name == "PHP_VERSION_ID" => {
            Some(ConstValue::Int(80200))
        }
        Expr::Unary(value) => {
            let value = const_eval_expr(&value.expr, session_statements)?;
            match value.as_number()? {
                number if value_is_int(&value) => {
                    let value = number as i64;
                    Some(ConstValue::Int(match value {
                        value if matches!(expr, Expr::Unary(unary) if unary.op == echo_ast::UnaryOp::Minus) => {
                            -value
                        }
                        value => value,
                    }))
                }
                number => Some(ConstValue::Float(match expr {
                    Expr::Unary(unary) if unary.op == echo_ast::UnaryOp::Minus => -number,
                    _ => number,
                })),
            }
        }
        Expr::Binary(value) => {
            let left = const_eval_expr(&value.left, session_statements)?;
            let right = const_eval_expr(&value.right, session_statements)?;
            const_eval_binary(left, value.op, right)
        }
        Expr::TypeAscription(value) => const_eval_expr(&value.expr, session_statements),
        _ => None,
    }
}

fn const_eval_variable(name: &str, session_statements: &[Stmt]) -> Option<ConstValue> {
    session_statements
        .iter()
        .rev()
        .find_map(|statement| match statement {
            Stmt::Assign(statement) if statement.name == name => {
                const_eval_expr(&statement.value, session_statements)
            }
            Stmt::Let(statement) if statement.name == name => {
                const_eval_expr(&statement.value, session_statements)
            }
            _ => None,
        })
}

fn const_eval_binary(left: ConstValue, op: BinaryOp, right: ConstValue) -> Option<ConstValue> {
    match op {
        BinaryOp::Add => const_eval_numeric(left, right, |left, right| left + right),
        BinaryOp::Sub => const_eval_numeric(left, right, |left, right| left - right),
        BinaryOp::Mul => const_eval_numeric(left, right, |left, right| left * right),
        BinaryOp::Div => const_eval_numeric(left, right, |left, right| left / right),
        BinaryOp::Mod => match (left, right) {
            (ConstValue::Int(left), ConstValue::Int(right)) if right != 0 => {
                Some(ConstValue::Int(left % right))
            }
            _ => None,
        },
        BinaryOp::Pow => const_eval_numeric(left, right, f64::powf),
        BinaryOp::Concat => Some(ConstValue::String(format!(
            "{}{}",
            const_value_as_string(left)?,
            const_value_as_string(right)?
        ))),
        BinaryOp::LessThan => match (left, right) {
            (ConstValue::Int(left), ConstValue::Int(right)) => Some(ConstValue::Bool(left < right)),
            _ => None,
        },
        BinaryOp::GreaterThanOrEqual => match (left, right) {
            (ConstValue::Int(left), ConstValue::Int(right)) => {
                Some(ConstValue::Bool(left >= right))
            }
            _ => None,
        },
        BinaryOp::Identical => Some(ConstValue::Bool(left == right)),
        BinaryOp::NotIdentical => Some(ConstValue::Bool(left != right)),
        BinaryOp::Equal => Some(ConstValue::Bool(left == right)),
        BinaryOp::NotEqual => Some(ConstValue::Bool(left != right)),
        BinaryOp::InstanceOf => None,
        BinaryOp::Coalesce => Some(left),
        BinaryOp::And => Some(ConstValue::Bool(
            const_value_as_bool(left)? && const_value_as_bool(right)?,
        )),
        BinaryOp::Or => Some(ConstValue::Bool(
            const_value_as_bool(left)? || const_value_as_bool(right)?,
        )),
        BinaryOp::Is => Some(ConstValue::Bool(
            matches!(right, ConstValue::Null) && left == right,
        )),
        BinaryOp::IsNot => Some(ConstValue::Bool(!matches!(
            const_eval_binary(left, BinaryOp::Is, right)?,
            ConstValue::Bool(true)
        ))),
    }
}

fn const_eval_numeric(
    left: ConstValue,
    right: ConstValue,
    op: impl FnOnce(f64, f64) -> f64,
) -> Option<ConstValue> {
    let both_int = value_is_int(&left) && value_is_int(&right);
    let result = op(left.as_number()?, right.as_number()?);

    if both_int && result.fract() == 0.0 {
        Some(ConstValue::Int(result as i64))
    } else {
        Some(ConstValue::Float(result))
    }
}

fn value_is_int(value: &ConstValue) -> bool {
    matches!(value, ConstValue::Int(_))
}

fn const_value_as_string(value: ConstValue) -> Option<String> {
    match value {
        ConstValue::Null => Some(String::new()),
        ConstValue::Bool(true) => Some("1".to_string()),
        ConstValue::Bool(false) => Some(String::new()),
        ConstValue::Int(value) => Some(value.to_string()),
        ConstValue::Float(value) => Some(value.to_string()),
        ConstValue::String(value) => Some(value),
    }
}

fn const_value_as_bool(value: ConstValue) -> Option<bool> {
    match value {
        ConstValue::Null => Some(false),
        ConstValue::Bool(value) => Some(value),
        ConstValue::Int(value) => Some(value != 0),
        ConstValue::Float(value) => Some(value != 0.0),
        ConstValue::String(value) => Some(!value.is_empty() && value != "0"),
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
        Expr::Constant(_) => "constant",
        Expr::ReceiverConst(_) => "receiver constant",
        Expr::StaticPropertyFetch(_) => "static property fetch",
        Expr::StaticPropertyAssign(_) => "static property assignment",
        Expr::StaticPropertyCoalesceAssign(_) => "static property coalesce assignment",
        Expr::ClassConstantFetch(_) => "class constant fetch",
        Expr::FunctionCall(_) => "function call",
        Expr::DynamicFunctionCall(_) => "dynamic function call",
        Expr::DynamicCall(_) => "dynamic call",
        Expr::MethodCall(_) => "method call",
        Expr::StaticCall(_) => "static call",
        Expr::New(_) => "constructor expression",
        Expr::Closure(_) => "closure expression",
        Expr::ArrowFunction(_) => "arrow function expression",
        Expr::Assign(_) => "assignment expression",
        Expr::MagicConstant(_) => "magic constant",
        Expr::Include(_) => "include expression",
        Expr::Defer(_) => "defer expression",
        Expr::Run(_) => "run expression",
        Expr::Fork(_) => "fork expression",
        Expr::Spawn(_) => "spawn expression",
        Expr::Join(_) => "join expression",
        Expr::Loop(_) => "loop expression",
        Expr::Unary(expr) => match expr.op {
            echo_ast::UnaryOp::Plus => "numeric identity expression",
            echo_ast::UnaryOp::Minus => "negate expression",
            echo_ast::UnaryOp::Not => "not expression",
        },
        Expr::Cast(_) => "cast expression",
        Expr::Binary(expr) => match expr.op {
            BinaryOp::Add => "add expression",
            BinaryOp::Sub => "subtract expression",
            BinaryOp::Mul => "multiply expression",
            BinaryOp::Div => "divide expression",
            BinaryOp::Mod => "modulo expression",
            BinaryOp::Pow => "exponent expression",
            BinaryOp::Concat => "concat expression",
            BinaryOp::LessThan => "less-than expression",
            BinaryOp::GreaterThanOrEqual => "greater-than-or-equal expression",
            BinaryOp::Identical => "identity comparison expression",
            BinaryOp::NotIdentical => "non-identity comparison expression",
            BinaryOp::Equal => "equality comparison expression",
            BinaryOp::NotEqual => "not-equal comparison expression",
            BinaryOp::InstanceOf => "instanceof expression",
            BinaryOp::Coalesce => "coalesce expression",
            BinaryOp::And => "and expression",
            BinaryOp::Or => "or expression",
            BinaryOp::Is | BinaryOp::IsNot => "null test expression",
        },
        Expr::Ternary(_) => "ternary expression",
        Expr::Match(_) => "match expression",
        Expr::TypeAscription(_) => "type ascription expression",
        Expr::Field(_) => "field expression",
        Expr::Index(_) => "index expression",
        Expr::TargetAssign(_) => "target assignment expression",
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
        Expr::Unary(expr) => match expr.op {
            echo_ast::UnaryOp::Plus | echo_ast::UnaryOp::Minus => "number".to_string(),
            echo_ast::UnaryOp::Not => "bool".to_string(),
        },
        Expr::Cast(expr) => expr.ty.clone(),
        Expr::Binary(expr) => match expr.op {
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Pow => "number".to_string(),
            BinaryOp::Concat => "string".to_string(),
            BinaryOp::LessThan => "bool".to_string(),
            BinaryOp::GreaterThanOrEqual => "bool".to_string(),
            BinaryOp::Identical => "bool".to_string(),
            BinaryOp::NotIdentical => "bool".to_string(),
            BinaryOp::Equal => "bool".to_string(),
            BinaryOp::NotEqual => "bool".to_string(),
            BinaryOp::InstanceOf => "bool".to_string(),
            BinaryOp::Coalesce => "unknown".to_string(),
            BinaryOp::And => "bool".to_string(),
            BinaryOp::Or => "bool".to_string(),
            BinaryOp::Is | BinaryOp::IsNot => "bool".to_string(),
        },
        Expr::Ternary(_) | Expr::Match(_) => "unknown".to_string(),
        Expr::FunctionCall(call) => echo_reflection::function(&call.name)
            .and_then(|function| function.return_type.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        Expr::DynamicFunctionCall(_)
        | Expr::DynamicCall(_)
        | Expr::MethodCall(_)
        | Expr::StaticCall(_)
        | Expr::StaticPropertyFetch(_)
        | Expr::StaticPropertyAssign(_)
        | Expr::StaticPropertyCoalesceAssign(_)
        | Expr::ClassConstantFetch(_)
        | Expr::TargetAssign(_) => "unknown".to_string(),
        Expr::New(expr) => match &expr.target {
            echo_ast::NewTarget::Class(class_name) => class_name.as_string(),
            echo_ast::NewTarget::Expr(_) => "unknown".to_string(),
            echo_ast::NewTarget::AnonymousClass(_) => "object".to_string(),
        },
        Expr::Closure(_) | Expr::ArrowFunction(_) => "unknown".to_string(),
        Expr::TypeAscription(expr) => expr.ty.clone(),
        Expr::Assign(expr) => expression_static_type(&expr.value),
        Expr::MagicConstant(_) => "string".to_string(),
        Expr::Include(_) => "bool".to_string(),
        Expr::Defer(_) | Expr::Run(_) => "task".to_string(),
        Expr::Fork(_) => "thread".to_string(),
        Expr::Spawn(_) => "process".to_string(),
        Expr::Variable(_)
        | Expr::Constant(_)
        | Expr::ReceiverConst(_)
        | Expr::Index(_)
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

#[cfg(test)]
#[path = "repl/tests.rs"]
mod tests;
