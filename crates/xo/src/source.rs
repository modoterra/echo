use std::fs;
use std::path::{Path, PathBuf};

use echo_ast::{BinaryOp, Expr, Program, QualifiedName, Stmt, StringLiteral};
use echo_diagnostics::Diagnostic;
use echo_source::{SourceFile, Span};

#[derive(Debug, Clone)]
pub struct SourceDiagnostic {
    pub diagnostic: Diagnostic,
    pub phase: String,
    pub path: PathBuf,
    pub source: String,
    pub include_stack: Vec<IncludeFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticReport {
    pub kind: String,
    pub phase: String,
    pub file: String,
    pub groups: Vec<DiagnosticGroup>,
    pub stack: Vec<DiagnosticStackFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticGroup {
    pub message: String,
    pub count: usize,
    pub occurrences: Vec<DiagnosticOccurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticOccurrence {
    pub line: usize,
    pub column: usize,
    pub span: Span,
    pub source: String,
    pub marker: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticStackFrame {
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub span: Span,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IncludeFrame {
    pub path: PathBuf,
    pub source: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Default, clap::Args)]
pub struct SourceOptions {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DiagnosticFormat {
    Human,
    Json,
}

pub fn read_source(file: &Path) -> String {
    fs::read_to_string(file).unwrap_or_else(|err| {
        eprintln!("error: failed to read {}: {err}", file.display());
        std::process::exit(1);
    })
}

pub fn read_source_file(file: &Path, mode: SourceOptions) -> SourceFile {
    let path = fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
    source_file_from_text(path, read_source(file), mode)
}

pub fn source_file_from_text(file: PathBuf, text: String, _mode: SourceOptions) -> SourceFile {
    SourceFile::new(file, text)
}

pub fn compile_ir(file: &Path, mode: SourceOptions) -> String {
    compile_ir_with_diagnostics(file, mode, DiagnosticFormat::Human)
}

pub fn compile_ir_with_diagnostics(
    file: &Path,
    mode: SourceOptions,
    diagnostic_format: DiagnosticFormat,
) -> String {
    let source = read_source_file(file, mode);

    match try_compile_ir_bundle(&source, mode) {
        Ok(ir) => ir,
        Err(diagnostics) => {
            print_source_diagnostics_with_format(diagnostics, diagnostic_format);
            std::process::exit(1);
        }
    }
}

pub fn run_jit(file: &Path, mode: SourceOptions) {
    let source = read_source_file(file, mode);

    match try_run_jit(&source) {
        Ok(status) => {
            if status != 0 {
                std::process::exit(status);
            }
        }
        Err(diagnostics) => {
            print_diagnostics(diagnostics);
            std::process::exit(1);
        }
    }
}

pub fn try_run_jit(source: &SourceFile) -> Result<i32, Vec<echo_diagnostics::Diagnostic>> {
    let ir = try_compile_ir_bundle(source, SourceOptions::default()).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.diagnostic)
            .collect::<Vec<_>>()
    })?;
    let output = echo_codegen::run_ir_jit_with_options(
        &ir,
        echo_codegen::JitOptions {
            capture_stdout: false,
            repl_inspect: false,
        },
    )?;

    Ok(output.status)
}

pub fn try_compile_ir_bundle(
    source: &SourceFile,
    mode: SourceOptions,
) -> Result<String, Vec<SourceDiagnostic>> {
    let bundle = parse_source_bundle(source, mode)?;
    let entry_hir = echo_hir::lower_program(&bundle.entry)
        .map_err(|diagnostics| compile_source_diagnostics(source, diagnostics, Vec::new()))?;
    let entry_mir = echo_mir::lower_program(&entry_hir)
        .map_err(|diagnostics| compile_source_diagnostics(source, diagnostics, Vec::new()))?;
    let mut includes = Vec::new();

    for include in &bundle.includes {
        let hir = echo_hir::lower_program(&include.program).map_err(|diagnostics| {
            compile_source_diagnostics(&include.source, diagnostics, include.include_stack.clone())
        })?;
        let mir = echo_mir::lower_program(&hir).map_err(|diagnostics| {
            compile_source_diagnostics(&include.source, diagnostics, include.include_stack.clone())
        })?;
        includes.push(echo_codegen::MirIncludeUnit {
            path: include.path.clone(),
            program: mir,
            dynamic_require: include.dynamic_require,
            class_names: include.class_names.clone(),
        });
    }

    echo_codegen::compile_mir_bundle_to_ir_detailed(&entry_mir, &includes)
        .map_err(|diagnostics| bundle.codegen_source_diagnostics(source, diagnostics))
}

struct SourceBundle {
    entry: Program,
    includes: Vec<IncludeProgram>,
}

struct IncludeProgram {
    path: String,
    source: SourceFile,
    program: Program,
    include_stack: Vec<IncludeFrame>,
    dynamic_require: bool,
    class_names: Vec<String>,
}

fn class_names_for_program(program: &Program) -> Vec<String> {
    let mut names = Vec::new();
    let mut namespace: Option<&QualifiedName> = None;

    for statement in &program.statements {
        match statement {
            Stmt::Namespace(statement) => namespace = Some(&statement.name),
            Stmt::ClassDecl(statement) => {
                names.push(statement.name.clone());
                if let Some(namespace) = namespace {
                    let dot_name = format!("{}.{}", namespace.parts.join("."), statement.name);
                    let php_name = format!("{}\\{}", namespace.as_string(), statement.name);
                    names.push(dot_name);
                    names.push(php_name);
                    if let Some(php_namespace) = echo_module_php_namespace(namespace) {
                        names.push(format!("{php_namespace}\\{}", statement.name));
                    }
                }
            }
            Stmt::InterfaceDecl(statement) => {
                names.push(statement.name.clone());
                if let Some(namespace) = namespace {
                    let dot_name = format!("{}.{}", namespace.parts.join("."), statement.name);
                    let php_name = format!("{}\\{}", namespace.as_string(), statement.name);
                    names.push(dot_name);
                    names.push(php_name);
                    if let Some(php_namespace) = echo_module_php_namespace(namespace) {
                        names.push(format!("{php_namespace}\\{}", statement.name));
                    }
                }
            }
            Stmt::TraitDecl(statement) => {
                names.push(statement.name.clone());
                if let Some(namespace) = namespace {
                    let dot_name = format!("{}.{}", namespace.parts.join("."), statement.name);
                    let php_name = format!("{}\\{}", namespace.as_string(), statement.name);
                    names.push(dot_name);
                    names.push(php_name);
                    if let Some(php_namespace) = echo_module_php_namespace(namespace) {
                        names.push(format!("{php_namespace}\\{}", statement.name));
                    }
                }
            }
            Stmt::EnumDecl(statement) => {
                names.push(statement.name.clone());
                if let Some(namespace) = namespace {
                    let dot_name = format!("{}.{}", namespace.parts.join("."), statement.name);
                    let php_name = format!("{}\\{}", namespace.as_string(), statement.name);
                    names.push(dot_name);
                    names.push(php_name);
                    if let Some(php_namespace) = echo_module_php_namespace(namespace) {
                        names.push(format!("{php_namespace}\\{}", statement.name));
                    }
                }
            }
            _ => {}
        }
    }

    names.sort();
    names.dedup();
    names
}

fn program_has_type_declaration(program: &Program) -> bool {
    program
        .statements
        .iter()
        .any(|statement| matches!(statement, Stmt::ClassDecl(_) | Stmt::TraitDecl(_)))
}

fn referenced_class_names_for_program(program: &Program) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    collect_contextual_class_references(program, &mut names);
    for statement in &program.statements {
        collect_statement_class_references(statement, &mut names);
    }
    names
}

fn collect_contextual_class_references(
    program: &Program,
    names: &mut std::collections::HashSet<String>,
) {
    let mut namespace: Option<QualifiedName> = None;
    let mut uses = std::collections::HashMap::new();

    for statement in &program.statements {
        match statement {
            Stmt::Namespace(statement) => namespace = Some(statement.name.clone()),
            Stmt::Use(statement) => {
                collect_qualified_name_references(&statement.name, names);
                if let Some(alias) = &statement.alias {
                    uses.insert(alias.clone(), statement.name.clone());
                } else if let Some(short) = statement.name.parts.last() {
                    uses.insert(short.clone(), statement.name.clone());
                }
            }
            _ => collect_statement_contextual_class_references(
                statement,
                namespace.as_ref(),
                &uses,
                names,
            ),
        }
    }
}

fn collect_contextual_name_references(
    name: &QualifiedName,
    namespace: Option<&QualifiedName>,
    uses: &std::collections::HashMap<String, QualifiedName>,
    names: &mut std::collections::HashSet<String>,
) {
    collect_qualified_name_references(name, names);

    let Some(first) = name.parts.first() else {
        return;
    };
    if let Some(imported) = uses.get(first) {
        let mut parts = imported.parts.clone();
        parts.extend(name.parts.iter().skip(1).cloned());
        collect_qualified_name_references(&QualifiedName::new(parts), names);
        return;
    }

    if let Some(namespace) = namespace {
        let mut parts = namespace.parts.clone();
        parts.extend(name.parts.iter().cloned());
        collect_qualified_name_references(&QualifiedName::new(parts), names);
    }
}

fn collect_call_arg_contextual_class_references(
    arg: &echo_ast::CallArg,
    namespace: Option<&QualifiedName>,
    uses: &std::collections::HashMap<String, QualifiedName>,
    names: &mut std::collections::HashSet<String>,
) {
    collect_expr_contextual_class_references(&arg.value, namespace, uses, names);
}

fn collect_class_member_contextual_class_references(
    member: &echo_ast::ClassMember,
    namespace: Option<&QualifiedName>,
    uses: &std::collections::HashMap<String, QualifiedName>,
    names: &mut std::collections::HashSet<String>,
) {
    match member {
        echo_ast::ClassMember::Method(method) => {
            for statement in &method.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        echo_ast::ClassMember::Property(property) => {
            if let Some(value) = &property.value {
                collect_expr_contextual_class_references(value, namespace, uses, names);
            }
        }
        echo_ast::ClassMember::Const(constant) => {
            collect_expr_contextual_class_references(&constant.value, namespace, uses, names);
        }
        echo_ast::ClassMember::TraitUse(name) => {
            collect_contextual_name_references(name, namespace, uses, names);
        }
    }
}

fn collect_enum_member_contextual_class_references(
    member: &echo_ast::EnumMember,
    namespace: Option<&QualifiedName>,
    uses: &std::collections::HashMap<String, QualifiedName>,
    names: &mut std::collections::HashSet<String>,
) {
    match member {
        echo_ast::EnumMember::Case(case) => {
            if let Some(value) = &case.value {
                collect_expr_contextual_class_references(value, namespace, uses, names);
            }
        }
        echo_ast::EnumMember::Method(method) => {
            for statement in &method.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        echo_ast::EnumMember::TraitUse(name) => {
            collect_contextual_name_references(name, namespace, uses, names);
        }
    }
}

fn collect_interface_member_contextual_class_references(
    member: &echo_ast::InterfaceMember,
    namespace: Option<&QualifiedName>,
    uses: &std::collections::HashMap<String, QualifiedName>,
    names: &mut std::collections::HashSet<String>,
) {
    match member {
        echo_ast::InterfaceMember::Method(method) => {
            for param in &method.params {
                if let Some(default_value) = &param.default_value {
                    collect_expr_contextual_class_references(default_value, namespace, uses, names);
                }
            }
        }
        echo_ast::InterfaceMember::Const(constant) => {
            collect_expr_contextual_class_references(&constant.value, namespace, uses, names);
        }
    }
}

fn collect_statement_contextual_class_references(
    statement: &Stmt,
    namespace: Option<&QualifiedName>,
    uses: &std::collections::HashMap<String, QualifiedName>,
    names: &mut std::collections::HashSet<String>,
) {
    match statement {
        Stmt::ClassDecl(statement) => {
            if let Some(parent) = &statement.parent {
                collect_contextual_name_references(parent, namespace, uses, names);
            }
            for interface in &statement.interfaces {
                collect_contextual_name_references(interface, namespace, uses, names);
            }
            for member in &statement.members {
                collect_class_member_contextual_class_references(member, namespace, uses, names);
            }
        }
        Stmt::InterfaceDecl(statement) => {
            for parent in &statement.parents {
                collect_contextual_name_references(parent, namespace, uses, names);
            }
            for member in &statement.members {
                collect_interface_member_contextual_class_references(
                    member, namespace, uses, names,
                );
            }
        }
        Stmt::TraitDecl(statement) => {
            for member in &statement.members {
                collect_class_member_contextual_class_references(member, namespace, uses, names);
            }
        }
        Stmt::EnumDecl(statement) => {
            for interface in &statement.interfaces {
                collect_contextual_name_references(interface, namespace, uses, names);
            }
            for member in &statement.members {
                collect_enum_member_contextual_class_references(member, namespace, uses, names);
            }
        }
        Stmt::FacetDecl(statement) => {
            for member in &statement.members {
                collect_class_member_contextual_class_references(member, namespace, uses, names);
            }
        }
        Stmt::FunctionDecl(statement) => {
            for statement in &statement.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Stmt::Echo(statement) => {
            for expr in &statement.exprs {
                collect_expr_contextual_class_references(expr, namespace, uses, names);
            }
        }
        Stmt::FunctionCall(statement) => {
            for arg in &statement.args {
                collect_call_arg_contextual_class_references(arg, namespace, uses, names);
            }
        }
        Stmt::DynamicFunctionCall(statement) => {
            for arg in &statement.args {
                collect_call_arg_contextual_class_references(arg, namespace, uses, names);
            }
        }
        Stmt::Assign(statement) => {
            collect_expr_contextual_class_references(&statement.value, namespace, uses, names)
        }
        Stmt::CoalesceAssign(statement) => {
            collect_expr_contextual_class_references(&statement.value, namespace, uses, names)
        }
        Stmt::ListAssign(statement) => {
            collect_expr_contextual_class_references(&statement.value, namespace, uses, names)
        }
        Stmt::Let(statement) => {
            collect_expr_contextual_class_references(&statement.value, namespace, uses, names)
        }
        Stmt::Return(statement) => {
            if let Some(value) = &statement.value {
                collect_expr_contextual_class_references(value, namespace, uses, names);
            }
        }
        Stmt::Throw(statement) => {
            collect_expr_contextual_class_references(&statement.value, namespace, uses, names)
        }
        Stmt::Yield(statement) => {
            collect_expr_contextual_class_references(&statement.value, namespace, uses, names)
        }
        Stmt::Goto(_) | Stmt::Label(_) => {}
        Stmt::Global(_) => {}
        Stmt::StaticVar(statement) => {
            for var in &statement.vars {
                if let Some(value) = &var.value {
                    collect_expr_contextual_class_references(value, namespace, uses, names);
                }
            }
        }
        Stmt::Expr(statement) => {
            collect_expr_contextual_class_references(&statement.expr, namespace, uses, names)
        }
        Stmt::Loop(statement) => {
            for statement in &statement.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Stmt::While(statement) => {
            collect_expr_contextual_class_references(&statement.condition, namespace, uses, names);
            for statement in &statement.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Stmt::DoWhile(statement) => {
            for statement in &statement.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
            collect_expr_contextual_class_references(&statement.condition, namespace, uses, names);
        }
        Stmt::For(statement) => {
            for expr in &statement.init {
                collect_expr_contextual_class_references(expr, namespace, uses, names);
            }
            for expr in &statement.conditions {
                collect_expr_contextual_class_references(expr, namespace, uses, names);
            }
            for expr in &statement.increments {
                collect_expr_contextual_class_references(expr, namespace, uses, names);
            }
            for statement in &statement.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Stmt::Foreach(statement) => {
            collect_expr_contextual_class_references(&statement.iterable, namespace, uses, names);
            for statement in &statement.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Stmt::Switch(statement) => {
            collect_expr_contextual_class_references(&statement.expr, namespace, uses, names);
            for case in &statement.cases {
                if let Some(condition) = &case.condition {
                    collect_expr_contextual_class_references(condition, namespace, uses, names);
                }
                for statement in &case.body {
                    collect_statement_contextual_class_references(
                        statement, namespace, uses, names,
                    );
                }
            }
        }
        Stmt::If(statement) => {
            collect_expr_contextual_class_references(&statement.condition, namespace, uses, names);
            for statement in &statement.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
            for clause in &statement.elseif_clauses {
                collect_expr_contextual_class_references(&clause.condition, namespace, uses, names);
                for statement in &clause.body {
                    collect_statement_contextual_class_references(
                        statement, namespace, uses, names,
                    );
                }
            }
            for statement in &statement.else_body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Stmt::Try(statement) => {
            for statement in &statement.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
            for catch in &statement.catches {
                for ty in &catch.types {
                    collect_contextual_name_references(ty, namespace, uses, names);
                }
                for statement in &catch.body {
                    collect_statement_contextual_class_references(
                        statement, namespace, uses, names,
                    );
                }
            }
            for statement in &statement.finally_body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Stmt::Break(statement) => {
            if let Some(value) = &statement.value {
                collect_expr_contextual_class_references(value, namespace, uses, names);
            }
        }
        Stmt::Continue(statement) => {
            if let Some(value) = &statement.value {
                collect_expr_contextual_class_references(value, namespace, uses, names);
            }
        }
        Stmt::Append(statement) => {
            collect_expr_contextual_class_references(&statement.target, namespace, uses, names);
            collect_expr_contextual_class_references(&statement.value, namespace, uses, names);
        }
        Stmt::AssignRef(_)
        | Stmt::Compile(_)
        | Stmt::Namespace(_)
        | Stmt::Use(_)
        | Stmt::Import(_)
        | Stmt::UnnamedExport(_)
        | Stmt::TypeDecl(_) => {}
    }
}

fn collect_expr_contextual_class_references(
    expr: &Expr,
    namespace: Option<&QualifiedName>,
    uses: &std::collections::HashMap<String, QualifiedName>,
    names: &mut std::collections::HashSet<String>,
) {
    match expr {
        Expr::StaticPropertyFetch(expr) => {
            collect_contextual_name_references(&expr.class_name, namespace, uses, names)
        }
        Expr::StaticPropertyAssign(expr) => {
            collect_contextual_name_references(&expr.class_name, namespace, uses, names);
            collect_expr_contextual_class_references(&expr.value, namespace, uses, names);
        }
        Expr::StaticPropertyCoalesceAssign(expr) => {
            collect_contextual_name_references(&expr.class_name, namespace, uses, names);
            collect_expr_contextual_class_references(&expr.value, namespace, uses, names);
        }
        Expr::ClassConstantFetch(expr) => {
            collect_contextual_name_references(&expr.class_name, namespace, uses, names)
        }
        Expr::StaticCall(expr) => {
            collect_contextual_name_references(&expr.class_name, namespace, uses, names);
            for arg in &expr.args {
                collect_call_arg_contextual_class_references(arg, namespace, uses, names);
            }
        }
        Expr::New(expr) => {
            match &expr.target {
                echo_ast::NewTarget::Class(name) => {
                    collect_contextual_name_references(name, namespace, uses, names)
                }
                echo_ast::NewTarget::Expr(target) => {
                    collect_expr_contextual_class_references(target, namespace, uses, names)
                }
                echo_ast::NewTarget::AnonymousClass(class) => {
                    if let Some(parent) = &class.parent {
                        collect_contextual_name_references(parent, namespace, uses, names);
                    }
                    for interface in &class.interfaces {
                        collect_contextual_name_references(interface, namespace, uses, names);
                    }
                    for member in &class.members {
                        collect_class_member_contextual_class_references(
                            member, namespace, uses, names,
                        );
                    }
                }
            }
            for arg in &expr.args {
                collect_call_arg_contextual_class_references(arg, namespace, uses, names);
            }
        }
        Expr::FunctionCall(expr) => {
            for arg in &expr.args {
                collect_call_arg_contextual_class_references(arg, namespace, uses, names);
            }
        }
        Expr::Print(expr) => {
            collect_expr_contextual_class_references(&expr.value, namespace, uses, names);
        }
        Expr::DynamicFunctionCall(expr) => {
            for arg in &expr.args {
                collect_call_arg_contextual_class_references(arg, namespace, uses, names);
            }
        }
        Expr::DynamicCall(expr) => {
            collect_expr_contextual_class_references(&expr.callee, namespace, uses, names);
            for arg in &expr.args {
                collect_call_arg_contextual_class_references(arg, namespace, uses, names);
            }
        }
        Expr::MethodCall(expr) => {
            collect_expr_contextual_class_references(&expr.object, namespace, uses, names);
            for arg in &expr.args {
                collect_call_arg_contextual_class_references(arg, namespace, uses, names);
            }
        }
        Expr::Closure(expr) => {
            for statement in &expr.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Expr::ArrowFunction(expr) => {
            collect_expr_contextual_class_references(&expr.body, namespace, uses, names)
        }
        Expr::Assign(expr) => {
            collect_expr_contextual_class_references(&expr.value, namespace, uses, names)
        }
        Expr::Include(expr) => {
            collect_expr_contextual_class_references(&expr.path, namespace, uses, names)
        }
        Expr::Defer(expr) => {
            for statement in &expr.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Expr::Run(expr) => match expr {
            echo_ast::RunExpr::Block { body, .. } => {
                for statement in body {
                    collect_statement_contextual_class_references(
                        statement, namespace, uses, names,
                    );
                }
            }
            echo_ast::RunExpr::Group { entries, .. } => {
                for entry in entries {
                    for statement in entry {
                        collect_statement_contextual_class_references(
                            statement, namespace, uses, names,
                        );
                    }
                }
            }
            echo_ast::RunExpr::Task { expr, .. } => {
                collect_expr_contextual_class_references(expr, namespace, uses, names)
            }
        },
        Expr::Fork(expr) => match expr {
            echo_ast::ForkExpr::Block { body, .. } => {
                for statement in body {
                    collect_statement_contextual_class_references(
                        statement, namespace, uses, names,
                    );
                }
            }
            echo_ast::ForkExpr::Task { expr, .. } => {
                collect_expr_contextual_class_references(expr, namespace, uses, names)
            }
        },
        Expr::Spawn(expr) => {
            collect_expr_contextual_class_references(&expr.command, namespace, uses, names)
        }
        Expr::Join(expr) => {
            collect_expr_contextual_class_references(&expr.handle, namespace, uses, names)
        }
        Expr::Loop(expr) => {
            for statement in &expr.body {
                collect_statement_contextual_class_references(statement, namespace, uses, names);
            }
        }
        Expr::Unary(expr) => {
            collect_expr_contextual_class_references(&expr.expr, namespace, uses, names)
        }
        Expr::Cast(expr) => {
            collect_expr_contextual_class_references(&expr.expr, namespace, uses, names)
        }
        Expr::Binary(expr) => {
            collect_expr_contextual_class_references(&expr.left, namespace, uses, names);
            collect_expr_contextual_class_references(&expr.right, namespace, uses, names);
        }
        Expr::Ternary(expr) => {
            collect_expr_contextual_class_references(&expr.condition, namespace, uses, names);
            collect_expr_contextual_class_references(&expr.if_true, namespace, uses, names);
            collect_expr_contextual_class_references(&expr.if_false, namespace, uses, names);
        }
        Expr::Match(expr) => {
            collect_expr_contextual_class_references(&expr.subject, namespace, uses, names);
            for arm in &expr.arms {
                for condition in &arm.conditions {
                    collect_expr_contextual_class_references(condition, namespace, uses, names);
                }
                collect_expr_contextual_class_references(&arm.value, namespace, uses, names);
            }
        }
        Expr::TypeAscription(expr) => {
            collect_expr_contextual_class_references(&expr.expr, namespace, uses, names)
        }
        Expr::Field(expr) => {
            collect_expr_contextual_class_references(&expr.object, namespace, uses, names)
        }
        Expr::Index(expr) => {
            collect_expr_contextual_class_references(&expr.collection, namespace, uses, names);
            collect_expr_contextual_class_references(&expr.index, namespace, uses, names);
        }
        Expr::TargetAssign(expr) => {
            collect_expr_contextual_class_references(&expr.target, namespace, uses, names);
            collect_expr_contextual_class_references(&expr.value, namespace, uses, names);
        }
        Expr::Object(expr) => {
            for field in &expr.fields {
                collect_expr_contextual_class_references(&field.value, namespace, uses, names);
            }
        }
        Expr::List(expr) => {
            for value in &expr.values {
                collect_expr_contextual_class_references(value, namespace, uses, names);
            }
        }
        Expr::Array(expr) => {
            for element in &expr.elements {
                if let Some(key) = &element.key {
                    collect_expr_contextual_class_references(key, namespace, uses, names);
                }
                collect_expr_contextual_class_references(&element.value, namespace, uses, names);
            }
        }
        Expr::Null(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Variable(_)
        | Expr::Constant(_)
        | Expr::ReceiverConst(_)
        | Expr::MagicConstant(_) => {}
    }
}

fn collect_qualified_name_references(
    name: &QualifiedName,
    names: &mut std::collections::HashSet<String>,
) {
    names.insert(name.as_string());
    names.insert(name.parts.join("."));
    if let Some(short) = name.parts.last() {
        names.insert(short.clone());
    }
}

fn collect_call_arg_class_references(
    arg: &echo_ast::CallArg,
    names: &mut std::collections::HashSet<String>,
) {
    collect_expr_class_references(&arg.value, names);
}

fn collect_class_member_class_references(
    member: &echo_ast::ClassMember,
    names: &mut std::collections::HashSet<String>,
) {
    match member {
        echo_ast::ClassMember::Method(method) => {
            for statement in &method.body {
                collect_statement_class_references(statement, names);
            }
            for param in &method.params {
                if let Some(default_value) = &param.default_value {
                    collect_expr_class_references(default_value, names);
                }
            }
        }
        echo_ast::ClassMember::Property(property) => {
            if let Some(value) = &property.value {
                collect_expr_class_references(value, names);
            }
        }
        echo_ast::ClassMember::Const(constant) => {
            collect_expr_class_references(&constant.value, names);
        }
        echo_ast::ClassMember::TraitUse(name) => collect_qualified_name_references(name, names),
    }
}

fn collect_enum_member_class_references(
    member: &echo_ast::EnumMember,
    names: &mut std::collections::HashSet<String>,
) {
    match member {
        echo_ast::EnumMember::Case(case) => {
            if let Some(value) = &case.value {
                collect_expr_class_references(value, names);
            }
        }
        echo_ast::EnumMember::Method(method) => {
            for statement in &method.body {
                collect_statement_class_references(statement, names);
            }
            for param in &method.params {
                if let Some(default_value) = &param.default_value {
                    collect_expr_class_references(default_value, names);
                }
            }
        }
        echo_ast::EnumMember::TraitUse(name) => collect_qualified_name_references(name, names),
    }
}

fn collect_interface_member_class_references(
    member: &echo_ast::InterfaceMember,
    names: &mut std::collections::HashSet<String>,
) {
    match member {
        echo_ast::InterfaceMember::Method(method) => {
            for param in &method.params {
                if let Some(default_value) = &param.default_value {
                    collect_expr_class_references(default_value, names);
                }
            }
        }
        echo_ast::InterfaceMember::Const(constant) => {
            collect_expr_class_references(&constant.value, names);
        }
    }
}

fn collect_statement_class_references(
    statement: &Stmt,
    names: &mut std::collections::HashSet<String>,
) {
    match statement {
        Stmt::Echo(statement) => {
            for expr in &statement.exprs {
                collect_expr_class_references(expr, names);
            }
        }
        Stmt::FunctionCall(statement) => {
            for arg in &statement.args {
                collect_call_arg_class_references(arg, names);
            }
        }
        Stmt::DynamicFunctionCall(statement) => {
            for arg in &statement.args {
                collect_call_arg_class_references(arg, names);
            }
        }
        Stmt::FunctionDecl(statement) => {
            for statement in &statement.body {
                collect_statement_class_references(statement, names);
            }
            for param in &statement.params {
                if let Some(default_value) = &param.default_value {
                    collect_expr_class_references(default_value, names);
                }
            }
        }
        Stmt::Assign(statement) => collect_expr_class_references(&statement.value, names),
        Stmt::CoalesceAssign(statement) => collect_expr_class_references(&statement.value, names),
        Stmt::ListAssign(statement) => collect_expr_class_references(&statement.value, names),
        Stmt::Let(statement) => collect_expr_class_references(&statement.value, names),
        Stmt::Compile(_) => {}
        Stmt::AssignRef(_) => {}
        Stmt::Return(statement) => {
            if let Some(value) = &statement.value {
                collect_expr_class_references(value, names);
            }
        }
        Stmt::Throw(statement) => collect_expr_class_references(&statement.value, names),
        Stmt::Yield(statement) => collect_expr_class_references(&statement.value, names),
        Stmt::Global(_) => {}
        Stmt::StaticVar(statement) => {
            for var in &statement.vars {
                if let Some(value) = &var.value {
                    collect_expr_class_references(value, names);
                }
            }
        }
        Stmt::Expr(statement) => collect_expr_class_references(&statement.expr, names),
        Stmt::Namespace(_) => {}
        Stmt::Use(statement) => collect_qualified_name_references(&statement.name, names),
        Stmt::Import(_) => {}
        Stmt::UnnamedExport(statement) => collect_expr_class_references(&statement.value, names),
        Stmt::ClassDecl(statement) => {
            if let Some(parent) = &statement.parent {
                collect_qualified_name_references(parent, names);
            }
            for interface in &statement.interfaces {
                collect_qualified_name_references(interface, names);
            }
            for member in &statement.members {
                collect_class_member_class_references(member, names);
            }
        }
        Stmt::InterfaceDecl(statement) => {
            for parent in &statement.parents {
                collect_qualified_name_references(parent, names);
            }
            for member in &statement.members {
                collect_interface_member_class_references(member, names);
            }
        }
        Stmt::TraitDecl(statement) => {
            for member in &statement.members {
                collect_class_member_class_references(member, names);
            }
        }
        Stmt::EnumDecl(statement) => {
            for interface in &statement.interfaces {
                collect_qualified_name_references(interface, names);
            }
            for member in &statement.members {
                collect_enum_member_class_references(member, names);
            }
        }
        Stmt::FacetDecl(statement) => {
            for member in &statement.members {
                collect_class_member_class_references(member, names);
            }
        }
        Stmt::TypeDecl(_) => {}
        Stmt::Goto(_) | Stmt::Label(_) => {}
        Stmt::Loop(statement) => {
            for statement in &statement.body {
                collect_statement_class_references(statement, names);
            }
        }
        Stmt::While(statement) => {
            collect_expr_class_references(&statement.condition, names);
            for statement in &statement.body {
                collect_statement_class_references(statement, names);
            }
        }
        Stmt::DoWhile(statement) => {
            for statement in &statement.body {
                collect_statement_class_references(statement, names);
            }
            collect_expr_class_references(&statement.condition, names);
        }
        Stmt::For(statement) => {
            for expr in &statement.init {
                collect_expr_class_references(expr, names);
            }
            for expr in &statement.conditions {
                collect_expr_class_references(expr, names);
            }
            for expr in &statement.increments {
                collect_expr_class_references(expr, names);
            }
            for statement in &statement.body {
                collect_statement_class_references(statement, names);
            }
        }
        Stmt::Foreach(statement) => {
            collect_expr_class_references(&statement.iterable, names);
            for statement in &statement.body {
                collect_statement_class_references(statement, names);
            }
        }
        Stmt::Switch(statement) => {
            collect_expr_class_references(&statement.expr, names);
            for case in &statement.cases {
                if let Some(condition) = &case.condition {
                    collect_expr_class_references(condition, names);
                }
                for statement in &case.body {
                    collect_statement_class_references(statement, names);
                }
            }
        }
        Stmt::If(statement) => {
            collect_expr_class_references(&statement.condition, names);
            for statement in &statement.body {
                collect_statement_class_references(statement, names);
            }
            for clause in &statement.elseif_clauses {
                collect_expr_class_references(&clause.condition, names);
                for statement in &clause.body {
                    collect_statement_class_references(statement, names);
                }
            }
            for statement in &statement.else_body {
                collect_statement_class_references(statement, names);
            }
        }
        Stmt::Try(statement) => {
            for statement in &statement.body {
                collect_statement_class_references(statement, names);
            }
            for catch in &statement.catches {
                for ty in &catch.types {
                    collect_qualified_name_references(ty, names);
                }
                for statement in &catch.body {
                    collect_statement_class_references(statement, names);
                }
            }
            for statement in &statement.finally_body {
                collect_statement_class_references(statement, names);
            }
        }
        Stmt::Break(statement) => {
            if let Some(value) = &statement.value {
                collect_expr_class_references(value, names);
            }
        }
        Stmt::Continue(statement) => {
            if let Some(value) = &statement.value {
                collect_expr_class_references(value, names);
            }
        }
        Stmt::Append(statement) => {
            collect_expr_class_references(&statement.target, names);
            collect_expr_class_references(&statement.value, names);
        }
    }
}

fn collect_expr_class_references(expr: &Expr, names: &mut std::collections::HashSet<String>) {
    match expr {
        Expr::Null(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Variable(_)
        | Expr::Constant(_)
        | Expr::ReceiverConst(_)
        | Expr::MagicConstant(_) => {}
        Expr::StaticPropertyFetch(expr) => {
            collect_qualified_name_references(&expr.class_name, names)
        }
        Expr::StaticPropertyAssign(expr) => {
            collect_qualified_name_references(&expr.class_name, names);
            collect_expr_class_references(&expr.value, names);
        }
        Expr::StaticPropertyCoalesceAssign(expr) => {
            collect_qualified_name_references(&expr.class_name, names);
            collect_expr_class_references(&expr.value, names);
        }
        Expr::ClassConstantFetch(expr) => {
            collect_qualified_name_references(&expr.class_name, names)
        }
        Expr::FunctionCall(expr) => {
            for arg in &expr.args {
                collect_call_arg_class_references(arg, names);
            }
        }
        Expr::Print(expr) => collect_expr_class_references(&expr.value, names),
        Expr::DynamicFunctionCall(expr) => {
            for arg in &expr.args {
                collect_call_arg_class_references(arg, names);
            }
        }
        Expr::DynamicCall(expr) => {
            collect_expr_class_references(&expr.callee, names);
            for arg in &expr.args {
                collect_call_arg_class_references(arg, names);
            }
        }
        Expr::MethodCall(expr) => {
            collect_expr_class_references(&expr.object, names);
            for arg in &expr.args {
                collect_call_arg_class_references(arg, names);
            }
        }
        Expr::StaticCall(expr) => {
            collect_qualified_name_references(&expr.class_name, names);
            for arg in &expr.args {
                collect_call_arg_class_references(arg, names);
            }
        }
        Expr::New(expr) => {
            match &expr.target {
                echo_ast::NewTarget::Class(name) => collect_qualified_name_references(name, names),
                echo_ast::NewTarget::Expr(target) => collect_expr_class_references(target, names),
                echo_ast::NewTarget::AnonymousClass(class) => {
                    if let Some(parent) = &class.parent {
                        collect_qualified_name_references(parent, names);
                    }
                    for interface in &class.interfaces {
                        collect_qualified_name_references(interface, names);
                    }
                    for member in &class.members {
                        collect_class_member_class_references(member, names);
                    }
                }
            }
            for arg in &expr.args {
                collect_call_arg_class_references(arg, names);
            }
        }
        Expr::Closure(expr) => {
            for statement in &expr.body {
                collect_statement_class_references(statement, names);
            }
            for param in &expr.params {
                if let Some(default_value) = &param.default_value {
                    collect_expr_class_references(default_value, names);
                }
            }
        }
        Expr::ArrowFunction(expr) => {
            collect_expr_class_references(&expr.body, names);
            for param in &expr.params {
                if let Some(default_value) = &param.default_value {
                    collect_expr_class_references(default_value, names);
                }
            }
        }
        Expr::Assign(expr) => collect_expr_class_references(&expr.value, names),
        Expr::Include(expr) => collect_expr_class_references(&expr.path, names),
        Expr::Defer(expr) => {
            for statement in &expr.body {
                collect_statement_class_references(statement, names);
            }
        }
        Expr::Run(expr) => match expr {
            echo_ast::RunExpr::Block { body, .. } => {
                for statement in body {
                    collect_statement_class_references(statement, names);
                }
            }
            echo_ast::RunExpr::Group { entries, .. } => {
                for entry in entries {
                    for statement in entry {
                        collect_statement_class_references(statement, names);
                    }
                }
            }
            echo_ast::RunExpr::Task { expr, .. } => collect_expr_class_references(expr, names),
        },
        Expr::Fork(expr) => match expr {
            echo_ast::ForkExpr::Block { body, .. } => {
                for statement in body {
                    collect_statement_class_references(statement, names);
                }
            }
            echo_ast::ForkExpr::Task { expr, .. } => collect_expr_class_references(expr, names),
        },
        Expr::Spawn(expr) => collect_expr_class_references(&expr.command, names),
        Expr::Join(expr) => collect_expr_class_references(&expr.handle, names),
        Expr::Loop(expr) => {
            for statement in &expr.body {
                collect_statement_class_references(statement, names);
            }
        }
        Expr::Unary(expr) => collect_expr_class_references(&expr.expr, names),
        Expr::Cast(expr) => collect_expr_class_references(&expr.expr, names),
        Expr::Binary(expr) => {
            collect_expr_class_references(&expr.left, names);
            if matches!(expr.op, BinaryOp::InstanceOf)
                && let Expr::Constant(constant) = &expr.right
            {
                names.insert(constant.name.clone());
            }
            collect_expr_class_references(&expr.right, names);
        }
        Expr::Ternary(expr) => {
            collect_expr_class_references(&expr.condition, names);
            collect_expr_class_references(&expr.if_true, names);
            collect_expr_class_references(&expr.if_false, names);
        }
        Expr::Match(expr) => {
            collect_expr_class_references(&expr.subject, names);
            for arm in &expr.arms {
                for condition in &arm.conditions {
                    collect_expr_class_references(condition, names);
                }
                collect_expr_class_references(&arm.value, names);
            }
        }
        Expr::TypeAscription(expr) => collect_expr_class_references(&expr.expr, names),
        Expr::Field(expr) => collect_expr_class_references(&expr.object, names),
        Expr::Index(expr) => {
            collect_expr_class_references(&expr.collection, names);
            collect_expr_class_references(&expr.index, names);
        }
        Expr::TargetAssign(expr) => {
            collect_expr_class_references(&expr.target, names);
            collect_expr_class_references(&expr.value, names);
        }
        Expr::Object(expr) => {
            for field in &expr.fields {
                collect_expr_class_references(&field.value, names);
            }
        }
        Expr::List(expr) => {
            for value in &expr.values {
                collect_expr_class_references(value, names);
            }
        }
        Expr::Array(expr) => {
            for element in &expr.elements {
                if let Some(key) = &element.key {
                    collect_expr_class_references(key, names);
                }
                collect_expr_class_references(&element.value, names);
            }
        }
    }
}

fn echo_module_php_namespace(namespace: &QualifiedName) -> Option<String> {
    let mut parts = Vec::with_capacity(namespace.parts.len());
    let mut changed = false;

    for part in &namespace.parts {
        let php_part = snake_or_lower_segment_to_pascal(part)?;
        changed |= php_part != *part;
        parts.push(php_part);
    }

    changed.then(|| parts.join("\\"))
}

fn snake_or_lower_segment_to_pascal(segment: &str) -> Option<String> {
    if segment.is_empty() {
        return None;
    }

    let mut output = String::new();
    for piece in segment.split('_') {
        if piece.is_empty() {
            return None;
        }
        let mut chars = piece.chars();
        let first = chars.next()?;
        output.extend(first.to_uppercase());
        output.push_str(chars.as_str());
    }
    Some(output)
}

fn class_name_aliases(class_name: &str) -> Vec<String> {
    let mut names = vec![class_name.to_string()];
    if let Some(short) = class_name.rsplit('\\').next()
        && short != class_name
    {
        names.push(short.to_string());
    }
    names
}

impl SourceBundle {
    fn codegen_source_diagnostics(
        &self,
        entry_source: &SourceFile,
        diagnostics: Vec<echo_codegen::CodegenDiagnostic>,
    ) -> Vec<SourceDiagnostic> {
        diagnostics
            .into_iter()
            .flat_map(|diagnostic| {
                if let Some(unit_path) = diagnostic.unit_path {
                    if let Some(include) = self
                        .includes
                        .iter()
                        .find(|include| include.path == unit_path)
                    {
                        return compile_source_diagnostics(
                            &include.source,
                            vec![diagnostic.diagnostic],
                            include.include_stack.clone(),
                        );
                    }
                }

                compile_source_diagnostics(entry_source, vec![diagnostic.diagnostic], Vec::new())
            })
            .collect()
    }
}

struct StaticIncludePath {
    path: PathBuf,
    span: Span,
}

fn parse_source_bundle(
    source: &SourceFile,
    mode: SourceOptions,
) -> Result<SourceBundle, Vec<SourceDiagnostic>> {
    let mut seen = std::collections::HashSet::new();
    let mut includes = Vec::new();
    let mut entry = parse_source_program(source)
        .map_err(|diagnostics| source_diagnostics(source, diagnostics, Vec::new()))?;
    resolve_static_includes(
        &mut entry,
        source,
        mode,
        &mut seen,
        &mut includes,
        Vec::new(),
    )?;
    discover_compile_entries(source, &entry, mode, &mut seen, &mut includes)?;
    discover_composer_autoload_files(source, mode, &mut seen, &mut includes)?;
    discover_composer_classmap_files(source, &entry, mode, &mut seen, &mut includes)?;
    discover_composer_psr4_roots(source, &entry, mode, &mut seen, &mut includes)?;

    Ok(SourceBundle { entry, includes })
}

fn discover_compile_entries(
    entry_source: &SourceFile,
    entry_program: &Program,
    mode: SourceOptions,
    seen: &mut std::collections::HashSet<PathBuf>,
    includes: &mut Vec<IncludeProgram>,
) -> Result<(), Vec<SourceDiagnostic>> {
    let entries = echo_resolver::compile_entries(entry_source, entry_program)
        .map_err(|diagnostics| source_diagnostics(entry_source, diagnostics, Vec::new()))?;

    for entry in entries {
        let canonical = fs::canonicalize(&entry.path).map_err(|err| {
            source_diagnostics(
                entry_source,
                vec![Diagnostic::new(
                    format!(
                        "failed to resolve compile entry `{}`: {err}",
                        entry.path.display()
                    ),
                    entry.span,
                )],
                Vec::new(),
            )
        })?;
        if !seen.insert(canonical.clone()) {
            continue;
        }

        let include_source = read_source_file(&canonical, mode);
        let include_program = parse_source_program(&include_source)
            .map_err(|diagnostics| source_diagnostics(&include_source, diagnostics, Vec::new()))?;
        let class_names = class_names_for_program(&include_program);
        includes.push(IncludeProgram {
            path: entry.dispatch_path,
            source: include_source,
            program: include_program,
            include_stack: Vec::new(),
            dynamic_require: true,
            class_names,
        });
    }

    Ok(())
}

fn discover_composer_autoload_files(
    entry_source: &SourceFile,
    mode: SourceOptions,
    seen: &mut std::collections::HashSet<PathBuf>,
    includes: &mut Vec<IncludeProgram>,
) -> Result<(), Vec<SourceDiagnostic>> {
    let Some(composer_dir) = find_composer_dir(&entry_source.path) else {
        return Ok(());
    };
    let Some((metadata_path, metadata)) = read_composer_autoload_metadata(&composer_dir) else {
        return Ok(());
    };
    let metadata = metadata.map_err(|err| {
        source_diagnostics(
            entry_source,
            vec![Diagnostic::new(
                format!(
                    "failed to read Composer autoload metadata `{}`: {err}",
                    metadata_path.display()
                ),
                Span::new(0, 0),
            )],
            Vec::new(),
        )
    })?;
    let helper_paths = composer_autoload_file_paths(&metadata, &composer_dir);

    for path in helper_paths {
        let dispatch_path = path.display().to_string();
        let canonical = fs::canonicalize(&path).map_err(|err| {
            source_diagnostics(
                entry_source,
                vec![Diagnostic::new(
                    format!(
                        "failed to resolve Composer autoload file `{}`: {err}",
                        path.display()
                    ),
                    Span::new(0, 0),
                )],
                Vec::new(),
            )
        })?;

        if !seen.insert(canonical.clone()) {
            continue;
        }

        let include_source = read_source_file(&canonical, mode);
        let include_program = Program {
            open_tag: None,
            statements: Vec::new(),
            source_id: include_source.id,
            source_dir: source_dir_for(&include_source.path),
            span: Span::new(0, 0),
        };

        includes.push(IncludeProgram {
            path: dispatch_path,
            source: include_source,
            program: include_program,
            include_stack: Vec::new(),
            dynamic_require: true,
            class_names: Vec::new(),
        });
    }

    Ok(())
}

fn discover_composer_classmap_files(
    entry_source: &SourceFile,
    entry_program: &Program,
    mode: SourceOptions,
    seen: &mut std::collections::HashSet<PathBuf>,
    includes: &mut Vec<IncludeProgram>,
) -> Result<(), Vec<SourceDiagnostic>> {
    let Some(composer_dir) = find_composer_dir(&entry_source.path) else {
        return Ok(());
    };
    let Some(classmap_path) = composer_classmap_metadata_path(&composer_dir) else {
        return Ok(());
    };

    let metadata = fs::read_to_string(&classmap_path).map_err(|err| {
        source_diagnostics(
            entry_source,
            vec![Diagnostic::new(
                format!(
                    "failed to read Composer autoload metadata `{}`: {err}",
                    classmap_path.display()
                ),
                Span::new(0, 0),
            )],
            Vec::new(),
        )
    })?;
    let classmap_paths = composer_classmap_paths(&metadata, &composer_dir);
    let classmap_short_counts = classmap_short_name_counts(&classmap_paths);
    let mut referenced = referenced_class_names_for_program(entry_program);
    for include in includes.iter() {
        referenced.extend(referenced_class_names_for_program(&include.program));
    }
    let mut consumed = std::collections::HashSet::new();

    loop {
        let mut progressed = false;
        for (class_name, path) in &classmap_paths {
            if consumed.contains(class_name) {
                continue;
            }
            let aliases = classmap_match_aliases(class_name, &classmap_short_counts);
            if !aliases.iter().any(|alias| referenced.contains(alias)) {
                continue;
            }

            consumed.insert(class_name.clone());
            let Some(include) = classmap_include_program(class_name, path, mode, seen, true) else {
                continue;
            };
            referenced.extend(referenced_class_names_for_program(&include.program));
            includes.push(include);
            progressed = true;
        }

        if !progressed {
            break;
        }
    }

    for (class_name, path) in &classmap_paths {
        if consumed.contains(class_name) {
            continue;
        }
        let Some(include) = classmap_include_program(class_name, path, mode, seen, false) else {
            continue;
        };
        includes.push(include);
    }

    Ok(())
}

fn classmap_short_name_counts(
    classmap_paths: &[(String, PathBuf)],
) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    for (class_name, _) in classmap_paths {
        if let Some(short) = class_name.rsplit('\\').next() {
            *counts.entry(short.to_string()).or_insert(0) += 1;
        }
    }
    counts
}

fn classmap_match_aliases(
    class_name: &str,
    short_counts: &std::collections::HashMap<String, usize>,
) -> Vec<String> {
    let mut aliases = vec![class_name.to_string(), class_name.replace('\\', ".")];
    if let Some(short) = class_name.rsplit('\\').next()
        && short != class_name
        && short_counts.get(short).copied() == Some(1)
    {
        aliases.push(short.to_string());
    }
    aliases.sort();
    aliases.dedup();
    aliases
}

fn classmap_include_program(
    class_name: &str,
    path: &Path,
    mode: SourceOptions,
    seen: &mut std::collections::HashSet<PathBuf>,
    parse_body: bool,
) -> Option<IncludeProgram> {
    let dispatch_path = path.display().to_string();
    let canonical = match fs::canonicalize(path) {
        Ok(canonical) => canonical,
        Err(_) => return None,
    };

    if !seen.insert(canonical.clone()) {
        return None;
    }

    let include_source = read_source_file(&canonical, mode);
    let include_program = if parse_body {
        parse_source_program(&include_source)
            .ok()
            .filter(program_has_type_declaration)
            .unwrap_or_else(|| Program {
                open_tag: None,
                statements: Vec::new(),
                source_id: include_source.id,
                source_dir: source_dir_for(&include_source.path),
                span: Span::new(0, 0),
            })
    } else {
        Program {
            open_tag: None,
            statements: Vec::new(),
            source_id: include_source.id,
            source_dir: source_dir_for(&include_source.path),
            span: Span::new(0, 0),
        }
    };
    let mut class_names = class_name_aliases(class_name);
    for parsed_name in class_names_for_program(&include_program) {
        if !class_names.contains(&parsed_name) {
            class_names.push(parsed_name);
        }
    }

    Some(IncludeProgram {
        path: dispatch_path,
        source: include_source,
        program: include_program,
        include_stack: Vec::new(),
        dynamic_require: false,
        class_names,
    })
}

fn discover_composer_psr4_roots(
    entry_source: &SourceFile,
    entry_program: &Program,
    mode: SourceOptions,
    seen: &mut std::collections::HashSet<PathBuf>,
    includes: &mut Vec<IncludeProgram>,
) -> Result<(), Vec<SourceDiagnostic>> {
    let Some(composer_dir) = find_composer_dir(&entry_source.path) else {
        return Ok(());
    };
    let autoload_psr4 = composer_dir.join("autoload_psr4.php");
    if !autoload_psr4.exists() {
        return Ok(());
    }

    let metadata = fs::read_to_string(&autoload_psr4).map_err(|err| {
        source_diagnostics(
            entry_source,
            vec![Diagnostic::new(
                format!(
                    "failed to read Composer PSR-4 metadata `{}`: {err}",
                    autoload_psr4.display()
                ),
                Span::new(0, 0),
            )],
            Vec::new(),
        )
    })?;
    let vendor_dir = composer_dir.parent().unwrap_or(&composer_dir);
    let base_dir = vendor_dir.parent().unwrap_or(vendor_dir);
    let canonical_base_dir = fs::canonicalize(base_dir).unwrap_or_else(|_| base_dir.to_path_buf());
    let mut referenced = referenced_class_names_for_program(entry_program);
    referenced = includes.iter().fold(referenced, |mut referenced, include| {
        referenced.extend(referenced_class_names_for_program(&include.program));
        referenced
    });

    for (prefix, root) in composer_psr4_roots(&metadata, &composer_dir) {
        let Ok(canonical_root) = fs::canonicalize(&root) else {
            continue;
        };
        if !is_project_psr4_root(&canonical_root, &canonical_base_dir) {
            for class_name in referenced.iter().filter(|name| name.starts_with(&prefix)) {
                let Some(include) =
                    psr4_include_program_for_class(&prefix, class_name, &root, mode, seen)
                else {
                    continue;
                };
                includes.push(include);
            }
            continue;
        }

        for (dispatch_path, canonical, relative) in
            source_files_under_psr4_root(&root, &canonical_root)
        {
            if !seen.insert(canonical.clone()) {
                continue;
            }

            let include_source = read_source_file(&canonical, mode);
            let include_program = match include_source
                .path
                .extension()
                .and_then(|extension| extension.to_str())
            {
                Some("echo" | "xo") => {
                    let program = parse_source_program(&include_source).map_err(|diagnostics| {
                        source_diagnostics(&include_source, diagnostics, Vec::new())
                    })?;
                    if program_has_type_declaration(&program) {
                        program
                    } else {
                        Program {
                            open_tag: None,
                            statements: Vec::new(),
                            source_id: include_source.id,
                            source_dir: source_dir_for(&include_source.path),
                            span: Span::new(0, 0),
                        }
                    }
                }
                _ => Program {
                    open_tag: None,
                    statements: Vec::new(),
                    source_id: include_source.id,
                    source_dir: source_dir_for(&include_source.path),
                    span: Span::new(0, 0),
                },
            };

            let mut class_names = class_names_for_program(&include_program);
            if class_names.is_empty()
                && include_source
                    .path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("php")
                && let Some(class_name) = psr4_class_name_for_relative_path(&prefix, &relative)
            {
                class_names.extend(class_name_aliases(&class_name));
            }

            includes.push(IncludeProgram {
                path: dispatch_path,
                source: include_source,
                program: include_program,
                include_stack: Vec::new(),
                dynamic_require: false,
                class_names,
            });
            if let Some(include) = includes.last() {
                referenced.extend(referenced_class_names_for_program(&include.program));
            }
        }
    }

    Ok(())
}

fn psr4_include_program_for_class(
    prefix: &str,
    class_name: &str,
    root: &Path,
    mode: SourceOptions,
    seen: &mut std::collections::HashSet<PathBuf>,
) -> Option<IncludeProgram> {
    let suffix = class_name.strip_prefix(prefix)?;
    let mut relative = PathBuf::new();
    for segment in suffix.split('\\').filter(|segment| !segment.is_empty()) {
        relative.push(segment);
    }
    relative.set_extension("php");

    let dispatch_path = root.join(&relative).display().to_string();
    let canonical = fs::canonicalize(root.join(&relative)).ok()?;
    if !seen.insert(canonical.clone()) {
        return None;
    }

    let include_source = read_source_file(&canonical, mode);
    let include_program = parse_source_program(&include_source)
        .ok()
        .filter(program_has_type_declaration)
        .unwrap_or_else(|| Program {
            open_tag: None,
            statements: Vec::new(),
            source_id: include_source.id,
            source_dir: source_dir_for(&include_source.path),
            span: Span::new(0, 0),
        });
    let mut class_names = class_name_aliases(class_name);
    for parsed_name in class_names_for_program(&include_program) {
        if !class_names.contains(&parsed_name) {
            class_names.push(parsed_name);
        }
    }

    Some(IncludeProgram {
        path: dispatch_path,
        source: include_source,
        program: include_program,
        include_stack: Vec::new(),
        dynamic_require: false,
        class_names,
    })
}

fn composer_psr4_roots(metadata: &str, composer_dir: &Path) -> Vec<(String, PathBuf)> {
    let vendor_dir = composer_dir.parent().unwrap_or(composer_dir);
    let base_dir = vendor_dir.parent().unwrap_or(vendor_dir);
    metadata
        .lines()
        .filter_map(|line| {
            let (prefix, roots) = line.split_once("=> array(")?;
            let prefix = php_generated_string_literal(prefix.trim())?;
            Some((prefix, roots.trim().trim_end_matches("),").to_string()))
        })
        .flat_map(|(prefix, roots)| {
            roots
                .trim()
                .trim_start_matches("array(")
                .trim_end_matches("),")
                .split(", ")
                .filter_map(move |expr| {
                    composer_generated_path_expr(expr, composer_dir, vendor_dir, base_dir)
                        .map(|path| (prefix.clone(), path))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn is_project_psr4_root(path: &Path, base_dir: &Path) -> bool {
    ["app", "database", "packages", "tests"]
        .iter()
        .map(|segment| base_dir.join(segment))
        .any(|root| path.starts_with(root))
}

fn source_files_under_psr4_root(
    dispatch_root: &Path,
    canonical_root: &Path,
) -> Vec<(String, PathBuf, PathBuf)> {
    let mut files = Vec::new();
    collect_source_files_under_psr4_root(dispatch_root, canonical_root, canonical_root, &mut files);
    files
}

fn collect_source_files_under_psr4_root(
    dispatch_root: &Path,
    canonical_root: &Path,
    current: &Path,
    files: &mut Vec<(String, PathBuf, PathBuf)>,
) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_source_files_under_psr4_root(dispatch_root, canonical_root, &path, files);
            continue;
        }

        if !matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("php" | "echo" | "xo")
        ) {
            continue;
        }

        let Ok(canonical) = fs::canonicalize(&path) else {
            continue;
        };
        let Ok(relative) = canonical.strip_prefix(canonical_root) else {
            continue;
        };
        let relative = relative.to_path_buf();
        let dispatch_path = dispatch_root.join(&relative).display().to_string();
        files.push((dispatch_path, canonical, relative));
    }
}

fn psr4_class_name_for_relative_path(prefix: &str, relative: &Path) -> Option<String> {
    let mut relative = relative.to_path_buf();
    relative.set_extension("");
    let suffix = relative
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?
        .join("\\");
    Some(format!("{prefix}{suffix}"))
}

fn read_composer_autoload_metadata(
    composer_dir: &Path,
) -> Option<(PathBuf, std::io::Result<String>)> {
    let autoload_static = composer_dir.join("autoload_static.php");
    if autoload_static.exists() {
        return Some((autoload_static.clone(), fs::read_to_string(autoload_static)));
    }

    let autoload_files = composer_dir.join("autoload_files.php");
    autoload_files
        .exists()
        .then(|| (autoload_files.clone(), fs::read_to_string(autoload_files)))
}

fn composer_classmap_metadata_path(composer_dir: &Path) -> Option<PathBuf> {
    let autoload_static = composer_dir.join("autoload_static.php");
    if autoload_static.exists() {
        return Some(autoload_static);
    }

    let autoload_classmap = composer_dir.join("autoload_classmap.php");
    autoload_classmap.exists().then_some(autoload_classmap)
}

fn find_composer_dir(entry_path: &Path) -> Option<PathBuf> {
    let mut current = entry_path.parent();
    while let Some(path) = current {
        let composer_dir = path.join("vendor").join("composer");
        if composer_dir.is_dir() {
            return Some(fs::canonicalize(&composer_dir).unwrap_or(composer_dir));
        }
        current = path.parent();
    }
    None
}

fn composer_autoload_file_paths(metadata: &str, composer_dir: &Path) -> Vec<PathBuf> {
    let vendor_dir = composer_dir.parent().unwrap_or(composer_dir);
    let base_dir = vendor_dir.parent().unwrap_or(vendor_dir);
    let file_entries = composer_static_files_block(metadata).unwrap_or(metadata);
    file_entries
        .lines()
        .filter_map(|line| line.split_once("=>").map(|(_, expr)| expr))
        .filter_map(|expr| composer_generated_path_expr(expr, composer_dir, vendor_dir, base_dir))
        .collect()
}

fn composer_static_files_block(metadata: &str) -> Option<&str> {
    let start = metadata.find("public static $files")?;
    let rest = &metadata[start..];
    let end = rest.find("public static $prefixLengthsPsr4")?;
    Some(&rest[..end])
}

fn composer_classmap_paths(metadata: &str, composer_dir: &Path) -> Vec<(String, PathBuf)> {
    let vendor_dir = composer_dir.parent().unwrap_or(composer_dir);
    let base_dir = vendor_dir.parent().unwrap_or(vendor_dir);
    let classmap_entries = composer_static_classmap_block(metadata).unwrap_or(metadata);
    classmap_entries
        .lines()
        .filter_map(|line| {
            let (class_name, expr) = line.split_once("=>")?;
            let class_name = php_generated_string_literal(class_name.trim())?;
            let path = composer_generated_path_expr(expr, composer_dir, vendor_dir, base_dir)?;
            Some((class_name, path))
        })
        .collect()
}

fn composer_static_classmap_block(metadata: &str) -> Option<&str> {
    let start = metadata.find("public static $classMap")?;
    let rest = &metadata[start..];
    let end = rest.find("public static function getInitializer")?;
    Some(&rest[..end])
}

fn composer_generated_path_expr(
    expr: &str,
    composer_dir: &Path,
    vendor_dir: &Path,
    base_dir: &Path,
) -> Option<PathBuf> {
    let mut output = String::new();
    for part in expr.trim().trim_end_matches(',').split(" . ") {
        let part = part.trim();
        match part {
            "__DIR__" => output.push_str(&composer_dir.display().to_string()),
            "$vendorDir" => output.push_str(&vendor_dir.display().to_string()),
            "$baseDir" => output.push_str(&base_dir.display().to_string()),
            _ => {
                let value = php_generated_string_literal(part)?;
                output.push_str(&value);
            }
        }
    }
    Some(PathBuf::from(output))
}

fn php_generated_string_literal(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    if bytes.len() < 2 || bytes.first() != Some(&b'\'') || bytes.last() != Some(&b'\'') {
        return None;
    }
    let mut output = String::new();
    let mut chars = value[1..value.len() - 1].chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }

        match chars.peek().copied() {
            Some('\\' | '\'') => {
                output.push(chars.next()?);
            }
            Some(_) | None => output.push(ch),
        }
    }
    Some(output)
}

fn resolve_static_includes(
    program: &mut Program,
    source: &SourceFile,
    mode: SourceOptions,
    seen: &mut std::collections::HashSet<PathBuf>,
    includes: &mut Vec<IncludeProgram>,
    include_stack: Vec<IncludeFrame>,
) -> Result<(), Vec<SourceDiagnostic>> {
    let source_dir = source
        .path
        .parent()
        .map(|path| fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let mut paths = Vec::new();

    collect_static_include_paths(&mut program.statements, &source_dir, &mut paths);

    for path in paths {
        let canonical = fs::canonicalize(&path.path).map_err(|err| {
            source_diagnostics(
                source,
                vec![Diagnostic::new(
                    format!(
                        "failed to resolve include path `{}`: {err}",
                        path.path.display()
                    ),
                    path.span,
                )],
                include_stack.clone(),
            )
        })?;

        if !seen.insert(canonical.clone()) {
            continue;
        }

        let include_source = read_source_file(&canonical, mode);
        let next_stack = {
            let mut stack = include_stack.clone();
            stack.push(IncludeFrame {
                path: source.path.clone(),
                source: source.text.clone(),
                span: path.span,
            });
            stack
        };
        let mut include_program = parse_source_program(&include_source).map_err(|diagnostics| {
            source_diagnostics(&include_source, diagnostics, next_stack.clone())
        })?;
        resolve_static_includes(
            &mut include_program,
            &include_source,
            mode,
            seen,
            includes,
            next_stack.clone(),
        )?;

        includes.push(IncludeProgram {
            path: canonical.display().to_string(),
            source: include_source,
            program: include_program,
            include_stack: next_stack,
            dynamic_require: true,
            class_names: Vec::new(),
        });
    }

    Ok(())
}

fn collect_static_include_class_member(
    member: &mut echo_ast::ClassMember,
    source_dir: &std::path::Path,
    paths: &mut Vec<StaticIncludePath>,
) {
    match member {
        echo_ast::ClassMember::Method(method) => {
            collect_static_include_paths(&mut method.body, source_dir, paths);
        }
        echo_ast::ClassMember::Property(property) => {
            if let Some(value) = &mut property.value {
                collect_static_include_expr(value, source_dir, paths);
            }
        }
        echo_ast::ClassMember::Const(constant) => {
            collect_static_include_expr(&mut constant.value, source_dir, paths);
        }
        echo_ast::ClassMember::TraitUse(_) => {}
    }
}

fn collect_static_include_paths(
    statements: &mut [Stmt],
    source_dir: &Path,
    paths: &mut Vec<StaticIncludePath>,
) {
    for statement in statements {
        match statement {
            Stmt::Echo(statement) => {
                for expr in &mut statement.exprs {
                    collect_static_include_expr(expr, source_dir, paths);
                }
            }
            Stmt::FunctionCall(statement) => {
                for arg in &mut statement.args {
                    collect_static_include_expr(&mut arg.value, source_dir, paths);
                }
            }
            Stmt::DynamicFunctionCall(statement) => {
                for arg in &mut statement.args {
                    collect_static_include_expr(&mut arg.value, source_dir, paths);
                }
            }
            Stmt::Assign(statement) => {
                collect_static_include_expr(&mut statement.value, source_dir, paths)
            }
            Stmt::CoalesceAssign(statement) => {
                collect_static_include_expr(&mut statement.value, source_dir, paths)
            }
            Stmt::ListAssign(statement) => {
                collect_static_include_expr(&mut statement.value, source_dir, paths)
            }
            Stmt::Let(statement) => {
                collect_static_include_expr(&mut statement.value, source_dir, paths)
            }
            Stmt::Return(statement) => {
                if let Some(value) = &mut statement.value {
                    collect_static_include_expr(value, source_dir, paths);
                }
            }
            Stmt::Throw(statement) => {
                collect_static_include_expr(&mut statement.value, source_dir, paths)
            }
            Stmt::Goto(_) | Stmt::Label(_) => {}
            Stmt::Expr(statement) => {
                collect_static_include_expr(&mut statement.expr, source_dir, paths)
            }
            Stmt::Loop(statement) => {
                collect_static_include_paths(&mut statement.body, source_dir, paths)
            }
            Stmt::While(statement) => {
                collect_static_include_expr(&mut statement.condition, source_dir, paths);
                collect_static_include_paths(&mut statement.body, source_dir, paths);
            }
            Stmt::DoWhile(statement) => {
                collect_static_include_paths(&mut statement.body, source_dir, paths);
                collect_static_include_expr(&mut statement.condition, source_dir, paths);
            }
            Stmt::For(statement) => {
                for expr in &mut statement.init {
                    collect_static_include_expr(expr, source_dir, paths);
                }
                for expr in &mut statement.conditions {
                    collect_static_include_expr(expr, source_dir, paths);
                }
                for expr in &mut statement.increments {
                    collect_static_include_expr(expr, source_dir, paths);
                }
                collect_static_include_paths(&mut statement.body, source_dir, paths);
            }
            Stmt::Foreach(statement) => {
                collect_static_include_expr(&mut statement.iterable, source_dir, paths);
                collect_static_include_paths(&mut statement.body, source_dir, paths);
            }
            Stmt::Switch(statement) => {
                collect_static_include_expr(&mut statement.expr, source_dir, paths);
                for case in &mut statement.cases {
                    if let Some(condition) = &mut case.condition {
                        collect_static_include_expr(condition, source_dir, paths);
                    }
                    collect_static_include_paths(&mut case.body, source_dir, paths);
                }
            }
            Stmt::If(statement) => {
                collect_static_include_expr(&mut statement.condition, source_dir, paths);
                collect_static_include_paths(&mut statement.body, source_dir, paths);
                for clause in &mut statement.elseif_clauses {
                    collect_static_include_expr(&mut clause.condition, source_dir, paths);
                    collect_static_include_paths(&mut clause.body, source_dir, paths);
                }
                collect_static_include_paths(&mut statement.else_body, source_dir, paths);
            }
            Stmt::Try(statement) => {
                collect_static_include_paths(&mut statement.body, source_dir, paths);
                for catch in &mut statement.catches {
                    collect_static_include_paths(&mut catch.body, source_dir, paths);
                }
                collect_static_include_paths(&mut statement.finally_body, source_dir, paths);
            }
            Stmt::Break(statement) => {
                if let Some(value) = &mut statement.value {
                    collect_static_include_expr(value, source_dir, paths);
                }
            }
            Stmt::Continue(statement) => {
                if let Some(value) = &mut statement.value {
                    collect_static_include_expr(value, source_dir, paths);
                }
            }
            Stmt::Append(statement) => {
                collect_static_include_expr(&mut statement.target, source_dir, paths);
                collect_static_include_expr(&mut statement.value, source_dir, paths)
            }
            Stmt::Yield(statement) => {
                collect_static_include_expr(&mut statement.value, source_dir, paths)
            }
            Stmt::Global(_) => {}
            Stmt::StaticVar(statement) => {
                for var in &mut statement.vars {
                    if let Some(value) = &mut var.value {
                        collect_static_include_expr(value, source_dir, paths);
                    }
                }
            }
            Stmt::AssignRef(_)
            | Stmt::Compile(_)
            | Stmt::Namespace(_)
            | Stmt::Use(_)
            | Stmt::Import(_)
            | Stmt::TypeDecl(_) => {}
            Stmt::UnnamedExport(statement) => {
                collect_static_include_expr(&mut statement.value, source_dir, paths)
            }
            Stmt::FunctionDecl(statement) => {
                collect_static_include_paths(&mut statement.body, source_dir, paths)
            }
            Stmt::ClassDecl(statement) => {
                for member in &mut statement.members {
                    match member {
                        echo_ast::ClassMember::Method(method) => {
                            collect_static_include_paths(&mut method.body, source_dir, paths);
                        }
                        echo_ast::ClassMember::Property(property) => {
                            if let Some(value) = &mut property.value {
                                collect_static_include_expr(value, source_dir, paths);
                            }
                        }
                        echo_ast::ClassMember::Const(constant) => {
                            collect_static_include_expr(&mut constant.value, source_dir, paths);
                        }
                        echo_ast::ClassMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::InterfaceDecl(statement) => {
                for member in &mut statement.members {
                    match member {
                        echo_ast::InterfaceMember::Method(method) => {
                            for param in &mut method.params {
                                if let Some(default_value) = &mut param.default_value {
                                    collect_static_include_expr(default_value, source_dir, paths);
                                }
                            }
                        }
                        echo_ast::InterfaceMember::Const(constant) => {
                            collect_static_include_expr(&mut constant.value, source_dir, paths);
                        }
                    }
                }
            }
            Stmt::TraitDecl(statement) => {
                for member in &mut statement.members {
                    match member {
                        echo_ast::ClassMember::Method(method) => {
                            collect_static_include_paths(&mut method.body, source_dir, paths);
                        }
                        echo_ast::ClassMember::Property(property) => {
                            if let Some(value) = &mut property.value {
                                collect_static_include_expr(value, source_dir, paths);
                            }
                        }
                        echo_ast::ClassMember::Const(constant) => {
                            collect_static_include_expr(&mut constant.value, source_dir, paths);
                        }
                        echo_ast::ClassMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::EnumDecl(statement) => {
                for member in &mut statement.members {
                    match member {
                        echo_ast::EnumMember::Case(case) => {
                            if let Some(value) = &mut case.value {
                                collect_static_include_expr(value, source_dir, paths);
                            }
                        }
                        echo_ast::EnumMember::Method(method) => {
                            collect_static_include_paths(&mut method.body, source_dir, paths);
                        }
                        echo_ast::EnumMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::FacetDecl(statement) => {
                for member in &mut statement.members {
                    match member {
                        echo_ast::ClassMember::Method(method) => {
                            collect_static_include_paths(&mut method.body, source_dir, paths);
                        }
                        echo_ast::ClassMember::Property(property) => {
                            if let Some(value) = &mut property.value {
                                collect_static_include_expr(value, source_dir, paths);
                            }
                        }
                        echo_ast::ClassMember::Const(constant) => {
                            collect_static_include_expr(&mut constant.value, source_dir, paths);
                        }
                        echo_ast::ClassMember::TraitUse(_) => {}
                    }
                }
            }
        }
    }
}

fn collect_static_include_expr(
    expr: &mut Expr,
    source_dir: &Path,
    paths: &mut Vec<StaticIncludePath>,
) {
    match expr {
        Expr::Include(include) => {
            collect_static_include_expr(&mut include.path, source_dir, paths);
            if let Some(path) = static_path_expr(&include.path, source_dir) {
                let canonical = fs::canonicalize(&path).unwrap_or(path);
                let value = canonical.display().to_string();
                let span = include.span;
                include.path = Expr::String(StringLiteral {
                    value,
                    span: include.path.span(),
                });
                paths.push(StaticIncludePath {
                    path: canonical,
                    span,
                });
            }
        }
        Expr::FunctionCall(expr) => {
            for arg in &mut expr.args {
                collect_static_include_expr(&mut arg.value, source_dir, paths);
            }
        }
        Expr::Print(expr) => collect_static_include_expr(&mut expr.value, source_dir, paths),
        Expr::DynamicFunctionCall(expr) => {
            for arg in &mut expr.args {
                collect_static_include_expr(&mut arg.value, source_dir, paths);
            }
        }
        Expr::DynamicCall(expr) => {
            collect_static_include_expr(&mut expr.callee, source_dir, paths);
            for arg in &mut expr.args {
                collect_static_include_expr(&mut arg.value, source_dir, paths);
            }
        }
        Expr::MethodCall(expr) => {
            collect_static_include_expr(&mut expr.object, source_dir, paths);
            for arg in &mut expr.args {
                collect_static_include_expr(&mut arg.value, source_dir, paths);
            }
        }
        Expr::StaticCall(expr) => {
            for arg in &mut expr.args {
                collect_static_include_expr(&mut arg.value, source_dir, paths);
            }
        }
        Expr::New(expr) => {
            match &mut expr.target {
                echo_ast::NewTarget::Expr(target) => {
                    collect_static_include_expr(target, source_dir, paths);
                }
                echo_ast::NewTarget::AnonymousClass(class) => {
                    for member in &mut class.members {
                        collect_static_include_class_member(member, source_dir, paths);
                    }
                }
                echo_ast::NewTarget::Class(_) => {}
            }
            for arg in &mut expr.args {
                collect_static_include_expr(&mut arg.value, source_dir, paths);
            }
        }
        Expr::Closure(expr) => collect_static_include_paths(&mut expr.body, source_dir, paths),
        Expr::ArrowFunction(expr) => collect_static_include_expr(&mut expr.body, source_dir, paths),
        Expr::Assign(expr) => collect_static_include_expr(&mut expr.value, source_dir, paths),
        Expr::StaticPropertyAssign(expr) => {
            collect_static_include_expr(&mut expr.value, source_dir, paths)
        }
        Expr::StaticPropertyCoalesceAssign(expr) => {
            collect_static_include_expr(&mut expr.value, source_dir, paths)
        }
        Expr::Defer(expr) => collect_static_include_paths(&mut expr.body, source_dir, paths),
        Expr::Run(expr) => match expr {
            echo_ast::RunExpr::Block { body, .. } => {
                collect_static_include_paths(body, source_dir, paths)
            }
            echo_ast::RunExpr::Task { expr, .. } => {
                collect_static_include_expr(expr, source_dir, paths)
            }
            echo_ast::RunExpr::Group { entries, .. } => {
                for entry in entries {
                    collect_static_include_paths(entry, source_dir, paths);
                }
            }
        },
        Expr::Fork(expr) => match expr {
            echo_ast::ForkExpr::Block { body, .. } => {
                collect_static_include_paths(body, source_dir, paths)
            }
            echo_ast::ForkExpr::Task { expr, .. } => {
                collect_static_include_expr(expr, source_dir, paths)
            }
        },
        Expr::Spawn(expr) => collect_static_include_expr(&mut expr.command, source_dir, paths),
        Expr::Join(expr) => collect_static_include_expr(&mut expr.handle, source_dir, paths),
        Expr::Loop(expr) => collect_static_include_paths(&mut expr.body, source_dir, paths),
        Expr::Unary(expr) => collect_static_include_expr(&mut expr.expr, source_dir, paths),
        Expr::Cast(expr) => collect_static_include_expr(&mut expr.expr, source_dir, paths),
        Expr::Binary(expr) => {
            collect_static_include_expr(&mut expr.left, source_dir, paths);
            collect_static_include_expr(&mut expr.right, source_dir, paths);
        }
        Expr::Ternary(expr) => {
            collect_static_include_expr(&mut expr.condition, source_dir, paths);
            collect_static_include_expr(&mut expr.if_true, source_dir, paths);
            collect_static_include_expr(&mut expr.if_false, source_dir, paths);
        }
        Expr::Match(expr) => {
            collect_static_include_expr(&mut expr.subject, source_dir, paths);
            for arm in &mut expr.arms {
                for condition in &mut arm.conditions {
                    collect_static_include_expr(condition, source_dir, paths);
                }
                collect_static_include_expr(&mut arm.value, source_dir, paths);
            }
        }
        Expr::TypeAscription(expr) => {
            collect_static_include_expr(&mut expr.expr, source_dir, paths)
        }
        Expr::Field(expr) => collect_static_include_expr(&mut expr.object, source_dir, paths),
        Expr::Index(expr) => {
            collect_static_include_expr(&mut expr.collection, source_dir, paths);
            collect_static_include_expr(&mut expr.index, source_dir, paths);
        }
        Expr::TargetAssign(expr) => {
            collect_static_include_expr(&mut expr.target, source_dir, paths);
            collect_static_include_expr(&mut expr.value, source_dir, paths);
        }
        Expr::Object(expr) => {
            for field in &mut expr.fields {
                collect_static_include_expr(&mut field.value, source_dir, paths);
            }
        }
        Expr::List(expr) => {
            for value in &mut expr.values {
                collect_static_include_expr(value, source_dir, paths);
            }
        }
        Expr::Array(expr) => {
            for element in &mut expr.elements {
                if let Some(key) = &mut element.key {
                    collect_static_include_expr(key, source_dir, paths);
                }
                collect_static_include_expr(&mut element.value, source_dir, paths);
            }
        }
        Expr::Null(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::Variable(_)
        | Expr::Constant(_)
        | Expr::ReceiverConst(_)
        | Expr::StaticPropertyFetch(_)
        | Expr::ClassConstantFetch(_)
        | Expr::MagicConstant(_) => {}
    }
}

fn static_path_expr(expr: &Expr, source_dir: &Path) -> Option<PathBuf> {
    let text = static_string_expr(expr, source_dir)?;
    let path = PathBuf::from(text);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(source_dir.join(path))
    }
}

fn static_string_expr(expr: &Expr, source_dir: &Path) -> Option<String> {
    match expr {
        Expr::String(expr) => Some(expr.value.clone()),
        Expr::MagicConstant(expr) if expr.kind == echo_ast::MagicConstantKind::Dir => {
            Some(source_dir.display().to_string())
        }
        Expr::Binary(expr) if expr.op == BinaryOp::Concat => {
            let left = static_string_expr(&expr.left, source_dir)?;
            let right = static_string_expr(&expr.right, source_dir)?;
            Some(format!("{left}{right}"))
        }
        _ => None,
    }
}

pub fn parse_source_program(
    source: &SourceFile,
) -> Result<Program, Vec<echo_diagnostics::Diagnostic>> {
    let mut program = echo_parser::parse_source_file(source)?;
    program.source_dir = source_dir_for(&source.path);
    Ok(program)
}

pub fn print_diagnostics(diagnostics: Vec<echo_diagnostics::Diagnostic>) {
    for diagnostic in diagnostics {
        eprintln!(
            "error: {} at {}..{}",
            diagnostic.message,
            diagnostic.span().start,
            diagnostic.span().end
        );
    }
}

pub fn print_source_diagnostics_with_format(
    diagnostics: Vec<SourceDiagnostic>,
    format: DiagnosticFormat,
) {
    let reports = diagnostic_reports(diagnostics);
    match format {
        DiagnosticFormat::Human => {
            for (index, report) in reports.iter().enumerate() {
                if index > 0 {
                    eprintln!();
                }
                eprint!("{}", render_diagnostic_report(report));
            }
        }
        DiagnosticFormat::Json => eprint!("{}", render_diagnostic_reports_json(&reports)),
    }
}

fn source_diagnostics(
    source: &SourceFile,
    diagnostics: Vec<Diagnostic>,
    include_stack: Vec<IncludeFrame>,
) -> Vec<SourceDiagnostic> {
    source_diagnostics_with_phase(source, diagnostics, include_stack, "parse")
}

fn compile_source_diagnostics(
    source: &SourceFile,
    diagnostics: Vec<Diagnostic>,
    include_stack: Vec<IncludeFrame>,
) -> Vec<SourceDiagnostic> {
    source_diagnostics_with_phase(source, diagnostics, include_stack, "compile")
}

fn source_diagnostics_with_phase(
    source: &SourceFile,
    diagnostics: Vec<Diagnostic>,
    include_stack: Vec<IncludeFrame>,
    phase: &str,
) -> Vec<SourceDiagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| SourceDiagnostic {
            diagnostic,
            phase: phase.to_string(),
            path: source.path.clone(),
            source: source.text.clone(),
            include_stack: include_stack.clone(),
        })
        .collect()
}

fn diagnostic_reports(diagnostics: Vec<SourceDiagnostic>) -> Vec<DiagnosticReport> {
    let mut reports = Vec::new();
    let mut index = 0;
    while index < diagnostics.len() {
        let diagnostic = &diagnostics[index];
        let mut report = DiagnosticReport {
            kind: format!("{}_failed", diagnostic.phase),
            phase: diagnostic.phase.clone(),
            file: display_path(&diagnostic.path),
            groups: Vec::new(),
            stack: diagnostic_stack(&diagnostic.include_stack),
        };

        let mut report_end = index;
        while report_end < diagnostics.len()
            && diagnostics[report_end].path == diagnostic.path
            && diagnostics[report_end].include_stack == diagnostic.include_stack
            && diagnostics[report_end].phase == diagnostic.phase
        {
            let message = normalized_message(&diagnostics[report_end].diagnostic.message);
            let mut group = DiagnosticGroup {
                message: message.clone(),
                count: 0,
                occurrences: Vec::new(),
            };

            while report_end < diagnostics.len()
                && diagnostics[report_end].path == diagnostic.path
                && diagnostics[report_end].include_stack == diagnostic.include_stack
                && diagnostics[report_end].phase == diagnostic.phase
                && normalized_message(&diagnostics[report_end].diagnostic.message) == message
            {
                group
                    .occurrences
                    .push(diagnostic_occurrence(&diagnostics[report_end]));
                group.count += 1;
                report_end += 1;
            }

            report.groups.push(group);
        }

        reports.push(report);
        index = report_end;
    }

    reports
}

fn normalized_message(message: &str) -> String {
    message.to_string()
}

fn diagnostic_occurrence(diagnostic: &SourceDiagnostic) -> DiagnosticOccurrence {
    let span = diagnostic.diagnostic.span();
    let (line, column) = line_column(&diagnostic.source, span.start);
    DiagnosticOccurrence {
        line,
        column,
        span,
        source: source_line(&diagnostic.source, span.start).to_string(),
        marker: marker_for_span(&diagnostic.source, span),
    }
}

fn diagnostic_stack(include_stack: &[IncludeFrame]) -> Vec<DiagnosticStackFrame> {
    include_stack
        .iter()
        .rev()
        .map(|frame| {
            let (line, column) = line_column(&frame.source, frame.span.start);
            DiagnosticStackFrame {
                kind: "include".to_string(),
                file: display_path(&frame.path),
                line,
                column,
                span: frame.span,
                source: source_line(&frame.source, frame.span.start).to_string(),
            }
        })
        .collect()
}

fn render_diagnostic_report(report: &DiagnosticReport) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{}{} failed{}: {}\n\n",
        ansi_bold(),
        report.phase,
        ansi_reset(),
        report.file
    ));
    let line_width = report
        .groups
        .iter()
        .flat_map(|group| group.occurrences.iter())
        .map(|occurrence| occurrence.line.to_string().len())
        .max()
        .unwrap_or(1);
    for group in &report.groups {
        if group.count == 1 {
            output.push_str(&format!("{}\n\n", group.message));
        } else {
            output.push_str(&format!(
                "{} occurrences of: {}\n\n",
                group.count, group.message
            ));
        }
        for occurrence in &group.occurrences {
            output.push_str(&render_occurrence(&report.file, occurrence, line_width));
            output.push('\n');
        }
    }

    if !report.stack.is_empty() {
        output.push_str(&format!("{}include stack:{}\n", ansi_bold(), ansi_reset()));
        for frame in &report.stack {
            output.push_str(&format!(
                "  {}{}:{}:{}{}",
                ansi_dim(),
                frame.file,
                frame.line,
                frame.column,
                ansi_reset()
            ));
            output.push('\n');
            output.push_str(&format!("    {}\n", frame.source.trim()));
        }
    }

    output
}

fn render_diagnostic_reports_json(reports: &[DiagnosticReport]) -> String {
    let mut output = String::from("{\"reports\":[");
    for (index, report) in reports.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&render_diagnostic_report_json(report));
    }
    output.push_str("]}\n");
    output
}

fn render_diagnostic_report_json(report: &DiagnosticReport) -> String {
    let mut output = String::new();
    output.push('{');
    output.push_str(&json_field("kind", &report.kind));
    output.push(',');
    output.push_str(&json_field("phase", &report.phase));
    output.push(',');
    output.push_str(&json_field("file", &report.file));
    output.push_str(",\"groups\":[");
    for (index, group) in report.groups.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&render_diagnostic_group_json(group));
    }
    output.push_str("],\"stack\":[");
    for (index, frame) in report.stack.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&render_diagnostic_stack_frame_json(frame));
    }
    output.push_str("]}");
    output
}

fn render_diagnostic_group_json(group: &DiagnosticGroup) -> String {
    let mut output = String::new();
    output.push('{');
    output.push_str(&json_field("message", &group.message));
    output.push_str(&format!(",\"count\":{}", group.count));
    output.push_str(",\"occurrences\":[");
    for (index, occurrence) in group.occurrences.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&render_diagnostic_occurrence_json(occurrence));
    }
    output.push_str("]}");
    output
}

fn render_diagnostic_occurrence_json(occurrence: &DiagnosticOccurrence) -> String {
    format!(
        "{{{},{},\"span\":{}, {}, {}}}",
        json_usize_field("line", occurrence.line),
        json_usize_field("column", occurrence.column),
        render_span_json(occurrence.span),
        json_field("source", &occurrence.source),
        json_field("marker", &occurrence.marker)
    )
}

fn render_diagnostic_stack_frame_json(frame: &DiagnosticStackFrame) -> String {
    format!(
        "{{{},{},{},\"span\":{}, {}, {}}}",
        json_field("kind", &frame.kind),
        json_field("file", &frame.file),
        json_usize_field("line", frame.line),
        render_span_json(frame.span),
        json_usize_field("column", frame.column),
        json_field("source", &frame.source)
    )
}

fn render_span_json(span: Span) -> String {
    format!("{{\"start\":{},\"end\":{}}}", span.start, span.end)
}

fn json_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", json_escape(name), json_escape(value))
}

fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{}\":{}", json_escape(name), value)
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn render_occurrence(file: &str, occurrence: &DiagnosticOccurrence, line_width: usize) -> String {
    let width = line_width.max(1);
    format!(
        "  {}-->{} {}:{}:{}\n   {}|\n{line:>width$} {}|{} {}\n   {}|{} {}{}{}",
        ansi_blue(),
        ansi_reset(),
        file,
        occurrence.line,
        occurrence.column,
        ansi_blue(),
        ansi_blue(),
        ansi_reset(),
        occurrence.source,
        ansi_blue(),
        ansi_reset(),
        ansi_red(),
        occurrence.marker,
        ansi_reset(),
        line = occurrence.line,
        width = width
    )
}

fn source_line(source: &str, offset: usize) -> &str {
    let offset = offset.min(source.len());
    let line_start = source[..offset]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let line_end = source[offset..]
        .find('\n')
        .map(|index| offset + index)
        .unwrap_or(source.len());
    &source[line_start..line_end]
}

fn marker_for_span(source: &str, span: Span) -> String {
    let (_, column) = line_column(source, span.start);
    let underline_len = span.end.saturating_sub(span.start).max(1);
    format!(
        "{}{}",
        " ".repeat(column.saturating_sub(1)),
        "^".repeat(underline_len)
    )
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut line_start = 0;
    for (index, byte) in source.bytes().enumerate() {
        if index >= offset {
            break;
        }
        if byte == b'\n' {
            line += 1;
            line_start = index + 1;
        }
    }
    (line, offset.saturating_sub(line_start) + 1)
}

fn display_path(path: &Path) -> String {
    let cwd = std::env::current_dir().ok();
    if let Some(cwd) = cwd
        && let Ok(stripped) = path.strip_prefix(cwd)
    {
        return stripped.display().to_string();
    }
    path.display().to_string()
}

fn ansi_bold() -> &'static str {
    "\x1b[1m"
}

fn ansi_dim() -> &'static str {
    "\x1b[2m"
}

fn ansi_blue() -> &'static str {
    "\x1b[34m"
}

fn ansi_red() -> &'static str {
    "\x1b[31m"
}

fn ansi_reset() -> &'static str {
    "\x1b[0m"
}

fn source_dir_for(path: &Path) -> Option<String> {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    path.parent().map(|path| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_module_classes_have_php_namespace_aliases_for_composer() {
        let program = Program {
            open_tag: None,
            source_id: None,
            statements: vec![
                Stmt::Namespace(echo_ast::NamespaceStmt {
                    source: echo_ast::NamespaceSource::Php,
                    name: QualifiedName::new(vec!["acme".to_string(), "http_server".to_string()]),
                    span: Span::new(0, 30),
                }),
                Stmt::ClassDecl(echo_ast::ClassDeclStmt {
                    name: "ServerProvider".to_string(),
                    modifiers: Vec::new(),
                    parent: None,
                    interfaces: Vec::new(),
                    members: Vec::new(),
                    span: Span::new(31, 60),
                }),
            ],
            source_dir: None,
            span: Span::new(0, 60),
        };

        let class_names = class_names_for_program(&program);

        assert!(class_names.contains(&"ServerProvider".to_string()));
        assert!(class_names.contains(&"acme.http_server.ServerProvider".to_string()));
        assert!(class_names.contains(&"acme\\http_server\\ServerProvider".to_string()));
        assert!(class_names.contains(&"Acme\\HttpServer\\ServerProvider".to_string()));
    }

    #[test]
    fn composer_psr4_discovery_bundles_echo_package_class_aliases() {
        let root =
            std::env::temp_dir().join(format!("echo-composer-psr4-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let composer_dir = root.join("vendor/composer");
        let package_src = root.join("packages/acme/http-server/src");
        fs::create_dir_all(&composer_dir).expect("composer dir");
        fs::create_dir_all(&package_src).expect("package src");
        fs::create_dir_all(root.join("public")).expect("public dir");

        fs::write(root.join("public/index.php"), "<?php echo \"\";\n").expect("entry source");
        fs::write(
            composer_dir.join("autoload_psr4.php"),
            "<?php\nreturn array(\n    'Acme\\\\HttpServer\\\\' => array($vendorDir . '/../packages/acme/http-server/src'),\n);\n",
        )
        .expect("autoload_psr4 metadata");
        fs::write(
            package_src.join("server_provider.echo"),
            "module acme.http_server\n\nclass ServerProvider {}\n",
        )
        .expect("echo provider source");

        let mode = SourceOptions::default();
        let entry = read_source_file(&root.join("public/index.php"), mode);
        let bundle = parse_source_bundle(&entry, mode).expect("source bundle parses");

        let provider = bundle
            .includes
            .iter()
            .find(|include| include.path.ends_with("server_provider.echo"))
            .expect("Composer PSR-4 should discover the Echo package class");

        assert!(
            provider
                .class_names
                .contains(&"acme.http_server.ServerProvider".to_string())
        );
        assert!(
            provider
                .class_names
                .contains(&"Acme\\HttpServer\\ServerProvider".to_string())
        );
        assert!(!provider.dynamic_require);

        fs::remove_dir_all(&root).expect("cleanup temp composer tree");
    }

    #[test]
    #[cfg(unix)]
    fn composer_psr4_discovery_follows_project_package_symlink() {
        let root = std::env::temp_dir().join(format!(
            "echo-composer-psr4-symlink-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);

        let composer_dir = root.join("vendor/composer");
        let vendor_package_dir = root.join("vendor/acme");
        let package_src = root.join("packages/acme/http-server/src");
        fs::create_dir_all(&composer_dir).expect("composer dir");
        fs::create_dir_all(&vendor_package_dir).expect("vendor package dir");
        fs::create_dir_all(&package_src).expect("package src");
        fs::create_dir_all(root.join("public")).expect("public dir");

        std::os::unix::fs::symlink(
            "../../packages/acme/http-server",
            vendor_package_dir.join("http-server"),
        )
        .expect("package symlink");

        fs::write(root.join("public/index.php"), "<?php echo \"\";\n").expect("entry source");
        fs::write(
            composer_dir.join("autoload_psr4.php"),
            "<?php\nreturn array(\n    'Acme\\\\HttpServer\\\\' => array($vendorDir . '/acme/http-server/src'),\n);\n",
        )
        .expect("autoload_psr4 metadata");
        fs::write(
            package_src.join("server_provider.echo"),
            "module acme.http_server\n\nclass ServerProvider {}\n",
        )
        .expect("echo provider source");

        let mode = SourceOptions::default();
        let entry = read_source_file(&root.join("public/index.php"), mode);
        let bundle = parse_source_bundle(&entry, mode).expect("source bundle parses");

        let provider = bundle
            .includes
            .iter()
            .find(|include| include.path.ends_with("server_provider.echo"))
            .expect("Composer PSR-4 should discover the symlinked Echo package class");

        assert!(
            provider
                .class_names
                .contains(&"Acme\\HttpServer\\ServerProvider".to_string())
        );

        fs::remove_dir_all(&root).expect("cleanup temp composer tree");
    }

    #[test]
    fn composer_psr4_discovery_loads_referenced_vendor_class_on_demand() {
        let root = std::env::temp_dir().join(format!(
            "echo-composer-vendor-psr4-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);

        let composer_dir = root.join("vendor/composer");
        let framework_dir = root.join("vendor/laravel/framework/src/Illuminate/Foundation");
        fs::create_dir_all(&composer_dir).expect("composer dir");
        fs::create_dir_all(&framework_dir).expect("framework dir");
        fs::create_dir_all(root.join("public")).expect("public dir");

        fs::write(
            root.join("public/index.php"),
            "<?php\nuse Illuminate\\Foundation\\Application;\n$app = require_once __DIR__.'/../bootstrap.php';\n$app->handleRequest();\n",
        )
        .expect("entry source");
        fs::write(
            composer_dir.join("autoload_psr4.php"),
            "<?php\nreturn array(\n    'Illuminate\\\\' => array($vendorDir . '/laravel/framework/src/Illuminate'),\n);\n",
        )
        .expect("autoload_psr4 metadata");
        fs::write(
            root.join("bootstrap.php"),
            "<?php return new Illuminate\\Foundation\\Application();\n",
        )
        .expect("bootstrap source");
        fs::write(
            framework_dir.join("Application.php"),
            "<?php namespace Illuminate\\Foundation; class Application { public function handleRequest() {} }\n",
        )
        .expect("application source");

        let mode = SourceOptions::default();
        let entry = read_source_file(&root.join("public/index.php"), mode);
        let bundle = parse_source_bundle(&entry, mode).expect("source bundle parses");

        let application = bundle
            .includes
            .iter()
            .find(|include| {
                include
                    .class_names
                    .contains(&"Illuminate\\Foundation\\Application".to_string())
            })
            .expect("referenced vendor PSR-4 class should be included");

        assert!(
            application
                .program
                .statements
                .iter()
                .any(|statement| matches!(
                    statement,
                    Stmt::ClassDecl(class)
                        if class.name == "Application"
                            && class.members.iter().any(|member| matches!(
                                member,
                                echo_ast::ClassMember::Method(method)
                                    if method.name == "handleRequest"
                            ))
                ))
        );
        assert!(!application.dynamic_require);

        fs::remove_dir_all(&root).expect("cleanup temp composer tree");
    }

    #[test]
    fn composer_classmap_discovery_reads_autoload_classmap_file() {
        let root = std::env::temp_dir().join(format!(
            "echo-composer-classmap-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);

        let composer_dir = root.join("vendor/composer");
        let app_dir = root.join("app");
        fs::create_dir_all(&composer_dir).expect("composer dir");
        fs::create_dir_all(&app_dir).expect("app dir");
        fs::create_dir_all(root.join("public")).expect("public dir");

        fs::write(root.join("public/index.php"), "<?php echo \"\";\n").expect("entry source");
        fs::write(
            composer_dir.join("autoload_classmap.php"),
            "<?php\n$vendorDir = dirname(__DIR__);\n$baseDir = dirname($vendorDir);\nreturn array(\n    'App\\\\Providers\\\\AppServiceProvider' => $baseDir . '/app/AppServiceProvider.php',\n);\n",
        )
        .expect("autoload_classmap metadata");
        fs::write(app_dir.join("AppServiceProvider.php"), "<?php\n").expect("classmap source");

        let mode = SourceOptions::default();
        let entry = read_source_file(&root.join("public/index.php"), mode);
        let bundle = parse_source_bundle(&entry, mode).expect("source bundle parses");

        let provider = bundle
            .includes
            .iter()
            .find(|include| include.path.ends_with("app/AppServiceProvider.php"))
            .expect("Composer classmap should discover the provider file");

        assert!(
            provider
                .class_names
                .contains(&"App\\Providers\\AppServiceProvider".to_string())
        );
        assert!(
            provider
                .class_names
                .contains(&"AppServiceProvider".to_string())
        );
        assert!(!provider.dynamic_require);

        fs::remove_dir_all(&root).expect("cleanup temp composer tree");
    }

    #[test]
    fn diagnostic_reports_group_same_file_message_and_stack() {
        let source = "if (!ready()) {\n}\nif (!done()) {\n}\n".to_string();
        let include_source = "require __DIR__ . \"/child.php\";\n".to_string();
        let diagnostics = vec![
            SourceDiagnostic {
                diagnostic: Diagnostic::new("unexpected token `!`", Span::new(4, 5)),
                phase: "parse".to_string(),
                path: PathBuf::from("child.php"),
                source: source.clone(),
                include_stack: vec![IncludeFrame {
                    path: PathBuf::from("main.php"),
                    source: include_source.clone(),
                    span: Span::new(0, include_source.trim_end().len()),
                }],
            },
            SourceDiagnostic {
                diagnostic: Diagnostic::new("unexpected token `!`", Span::new(22, 23)),
                phase: "parse".to_string(),
                path: PathBuf::from("child.php"),
                source,
                include_stack: vec![IncludeFrame {
                    path: PathBuf::from("main.php"),
                    source: include_source,
                    span: Span::new(0, 31),
                }],
            },
        ];

        let reports = diagnostic_reports(diagnostics);

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].kind, "parse_failed");
        assert_eq!(reports[0].phase, "parse");
        assert_eq!(reports[0].groups.len(), 1);
        assert_eq!(reports[0].groups[0].message, "unexpected token `!`");
        assert_eq!(reports[0].groups[0].count, 2);
        assert_eq!(reports[0].groups[0].occurrences[0].line, 1);
        assert_eq!(reports[0].groups[0].occurrences[1].line, 3);
        assert_eq!(reports[0].stack.len(), 1);
        assert_eq!(reports[0].stack[0].kind, "include");
    }

    #[test]
    fn diagnostic_renderer_pads_line_numbers_to_report_width() {
        let report = DiagnosticReport {
            kind: "parse_failed".to_string(),
            phase: "parse".to_string(),
            file: "child.php".to_string(),
            groups: vec![DiagnosticGroup {
                message: "unexpected token".to_string(),
                count: 2,
                occurrences: vec![
                    DiagnosticOccurrence {
                        line: 2,
                        column: 1,
                        span: Span::new(0, 1),
                        source: "bad".to_string(),
                        marker: "^".to_string(),
                    },
                    DiagnosticOccurrence {
                        line: 10,
                        column: 1,
                        span: Span::new(0, 1),
                        source: "worse".to_string(),
                        marker: "^".to_string(),
                    },
                ],
            }],
            stack: Vec::new(),
        };

        let rendered = render_diagnostic_report(&report);

        assert!(rendered.contains("\n 2 \x1b[34m|\x1b[0m bad"));
        assert!(rendered.contains("\n10 \x1b[34m|\x1b[0m worse"));
    }

    #[test]
    fn diagnostic_json_renderer_outputs_report_structure() {
        let report = DiagnosticReport {
            kind: "compile_failed".to_string(),
            phase: "compile".to_string(),
            file: "child.php".to_string(),
            groups: vec![DiagnosticGroup {
                message: "unsupported \"thing\"".to_string(),
                count: 1,
                occurrences: vec![DiagnosticOccurrence {
                    line: 2,
                    column: 6,
                    span: Span::new(10, 17),
                    source: "if (!ready()) {".to_string(),
                    marker: "     ^^^^^^^".to_string(),
                }],
            }],
            stack: vec![DiagnosticStackFrame {
                kind: "include".to_string(),
                file: "main.php".to_string(),
                line: 1,
                column: 1,
                span: Span::new(0, 31),
                source: "require __DIR__ . \"/child.php\";".to_string(),
            }],
        };

        let rendered = render_diagnostic_reports_json(&[report]);

        assert!(rendered.starts_with("{\"reports\":["));
        assert!(rendered.contains("\"phase\":\"compile\""));
        assert!(rendered.contains("\"message\":\"unsupported \\\"thing\\\"\""));
        assert!(rendered.contains("\"span\":{\"start\":10,\"end\":17}"));
        assert!(rendered.contains("\"marker\":\"     ^^^^^^^\""));
        assert!(rendered.contains("require __DIR__ . \\\"/child.php\\\";"));
    }
}
