use std::collections::HashMap;

use echo_ast::{
    BinaryOp, ClassMember, Expr, FunctionDeclStmt, ImportSource, NamespaceSource, Program, Stmt,
    UnaryOp,
};
use echo_diagnostics::Diagnostic;
use echo_index::{
    DependencyFact, DependencyKind, EchoFileMode, FileId, FqName, IndexFacts, Signature,
    SymbolFact, SymbolKind, SymbolName, TextRange,
};
use echo_source::Span;
use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub struct Analysis {
    expression_types: HashMap<SpanKey, Type>,
    variables: HashMap<String, VariableInfo>,
}

impl Analysis {
    pub fn expression_type(&self, expr: &Expr) -> Type {
        self.expression_type_at(expr.span())
    }

    pub fn expression_type_at(&self, span: Span) -> Type {
        self.expression_types
            .get(&SpanKey::from(span))
            .cloned()
            .unwrap_or(Type::Unknown)
    }

    pub fn variable(&self, name: &str) -> Option<&VariableInfo> {
        self.variables.get(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableInfo {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Null,
    Bool,
    Int,
    Float,
    Number,
    String,
    Array,
    List,
    Object(Option<String>),
    Task,
    Thread,
    Process,
    Never,
    Unknown,
    Named(String),
}

impl Type {
    pub fn display_name(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::Number => "number".to_string(),
            Self::String => "string".to_string(),
            Self::Array => "array".to_string(),
            Self::List => "list".to_string(),
            Self::Object(Some(name)) if !name.is_empty() => name.clone(),
            Self::Object(Some(_)) => "object".to_string(),
            Self::Object(None) => "object".to_string(),
            Self::Task => "task".to_string(),
            Self::Thread => "thread".to_string(),
            Self::Process => "process".to_string(),
            Self::Never => "never".to_string(),
            Self::Unknown => "unknown".to_string(),
            Self::Named(name) => name.clone(),
        }
    }
}

pub fn analyze(program: &Program) -> Result<Analysis, Vec<Diagnostic>> {
    let mut analyzer = Analyzer::default();
    analyzer.analyze_statements(&program.statements);
    if analyzer.diagnostics.is_empty() {
        Ok(Analysis {
            expression_types: analyzer.expression_types,
            variables: analyzer.variables,
        })
    } else {
        Err(analyzer.diagnostics)
    }
}

pub fn index_facts(program: &Program, file_id: FileId, mode: EchoFileMode) -> IndexFacts {
    let mut extractor = IndexFactExtractor::new(file_id, mode);
    extractor.extract(program);
    extractor.into_facts()
}

struct IndexFactExtractor {
    file_id: FileId,
    mode: EchoFileMode,
    namespace: Vec<SmolStr>,
    declarations: Vec<SymbolFact>,
    dependencies: Vec<DependencyFact>,
}

impl IndexFactExtractor {
    fn new(file_id: FileId, mode: EchoFileMode) -> Self {
        Self {
            file_id,
            mode,
            namespace: Vec::new(),
            declarations: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    fn extract(&mut self, program: &Program) {
        self.extract_statements(&program.statements);
    }

    fn extract_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.extract_statement(statement);
        }
    }

    fn extract_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Namespace(statement) => {
                self.namespace = match statement.source {
                    NamespaceSource::Php => statement
                        .name
                        .parts
                        .iter()
                        .map(|part| SmolStr::new(part.as_str()))
                        .collect(),
                    NamespaceSource::Std => {
                        let mut namespace = vec![SmolStr::new("std")];
                        namespace.extend(
                            statement
                                .name
                                .parts
                                .iter()
                                .map(|part| SmolStr::new(part.as_str())),
                        );
                        namespace
                    }
                };

                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_string()),
                    fq_name: Some(FqName::new(Vec::new(), namespace_display(&self.namespace))),
                    kind: SymbolKind::Namespace,
                    range: span_range(statement.span),
                    selection_range: span_range(statement.span),
                    visibility: None,
                    signature: None,
                });
            }
            Stmt::Use(statement) => {
                self.dependencies.push(DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: statement.name.as_string(),
                    alias: statement.alias.clone(),
                    range: span_range(statement.span),
                });
            }
            Stmt::Import(statement) => {
                let (kind, target) = match &statement.source {
                    ImportSource::Std => {
                        (DependencyKind::EchoStdImport, statement.name.as_string())
                    }
                    ImportSource::File(path) => (
                        DependencyKind::EchoFileImport,
                        format!("{path}#{}", statement.name.as_string()),
                    ),
                };
                self.dependencies.push(DependencyFact {
                    kind,
                    target,
                    alias: statement.alias.clone(),
                    range: span_range(statement.span),
                });
            }
            Stmt::FunctionDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::Function,
                    range: span_range(statement.span),
                    selection_range: span_range(statement.span),
                    visibility: None,
                    signature: function_signature(statement),
                });
                self.extract_statements(&statement.body);
            }
            Stmt::ClassDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::Class,
                    range: span_range(statement.span),
                    selection_range: span_range(statement.span),
                    visibility: None,
                    signature: None,
                });

                for member in &statement.members {
                    let ClassMember::Method(method) = member;
                    self.declarations.push(SymbolFact {
                        name: SymbolName::new(method.name.as_str()),
                        fq_name: Some(
                            self.fq_name(&format!("{}::{}", statement.name, method.name)),
                        ),
                        kind: SymbolKind::Method,
                        range: span_range(method.span),
                        selection_range: span_range(method.span),
                        visibility: None,
                        signature: Some(Signature {
                            text: method_signature(&method.params, method.return_type.as_deref()),
                        }),
                    });
                }
            }
            Stmt::TypeDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::TypeAlias,
                    range: span_range(statement.span),
                    selection_range: span_range(statement.span),
                    visibility: None,
                    signature: None,
                });
            }
            Stmt::Loop(statement) => self.extract_statements(&statement.body),
            Stmt::If(statement) => self.extract_statements(&statement.body),
            Stmt::Echo(_)
            | Stmt::FunctionCall(_)
            | Stmt::DynamicFunctionCall(_)
            | Stmt::Assign(_)
            | Stmt::Let(_)
            | Stmt::AssignRef(_)
            | Stmt::Return(_)
            | Stmt::Yield(_)
            | Stmt::Expr(_)
            | Stmt::Break(_)
            | Stmt::Append(_) => {}
        }
    }

    fn fq_name(&self, name: &str) -> FqName {
        FqName::new(self.namespace.clone(), name)
    }

    fn into_facts(self) -> IndexFacts {
        IndexFacts {
            file_id: self.file_id,
            mode: self.mode,
            declarations: self.declarations,
            dependencies: self.dependencies,
        }
    }
}

fn span_range(span: Span) -> TextRange {
    TextRange::new(span.start as u32, span.end as u32)
}

fn namespace_display(namespace: &[SmolStr]) -> String {
    namespace
        .iter()
        .map(SmolStr::as_str)
        .collect::<Vec<_>>()
        .join("\\")
}

fn function_signature(statement: &FunctionDeclStmt) -> Option<Signature> {
    Some(Signature {
        text: method_signature(&statement.params, statement.return_type.as_deref()),
    })
}

fn method_signature(params: &[echo_ast::TypedParam], return_type: Option<&str>) -> String {
    let params = params
        .iter()
        .map(|param| match &param.ty {
            Some(ty) => format!("{ty} ${}", param.name),
            None => format!("${}", param.name),
        })
        .collect::<Vec<_>>()
        .join(", ");

    match return_type {
        Some(return_type) => format!("({params}): {return_type}"),
        None => format!("({params})"),
    }
}

#[derive(Default)]
struct Analyzer {
    expression_types: HashMap<SpanKey, Type>,
    variables: HashMap<String, VariableInfo>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SpanKey {
    start: usize,
    end: usize,
}

impl From<Span> for SpanKey {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl Analyzer {
    fn analyze_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.analyze_statement(statement);
        }
    }

    fn analyze_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Echo(statement) => {
                for expr in &statement.exprs {
                    self.analyze_expr(expr);
                }
            }
            Stmt::FunctionCall(statement) => {
                for expr in &statement.args {
                    self.analyze_expr(expr);
                }
            }
            Stmt::DynamicFunctionCall(statement) => {
                self.resolve_variable(&statement.name, statement.span);
                for expr in &statement.args {
                    self.analyze_expr(expr);
                }
            }
            Stmt::FunctionDecl(statement) => self.analyze_function_decl(statement),
            Stmt::Assign(statement) => {
                let ty = self.analyze_expr(&statement.value);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::Let(statement) => {
                let value_ty = self.analyze_expr(&statement.value);
                let ty = statement
                    .ty
                    .as_ref()
                    .map(|ty| Type::Named(ty.clone()))
                    .unwrap_or(value_ty);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::AssignRef(statement) => {
                let ty = self
                    .resolve_variable(&statement.target, statement.span)
                    .unwrap_or(Type::Unknown);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::Return(statement) => {
                self.analyze_expr(&statement.value);
            }
            Stmt::Yield(statement) => {
                self.analyze_expr(&statement.value);
            }
            Stmt::Expr(statement) => {
                self.analyze_expr(&statement.expr);
            }
            Stmt::Namespace(_)
            | Stmt::Use(_)
            | Stmt::Import(_)
            | Stmt::ClassDecl(_)
            | Stmt::TypeDecl(_)
            | Stmt::Break(_) => {}
            Stmt::Loop(statement) => self.analyze_statements(&statement.body),
            Stmt::If(statement) => {
                self.analyze_expr(&statement.condition);
                self.analyze_statements(&statement.body);
            }
            Stmt::Append(statement) => {
                let target_ty = self.resolve_variable(&statement.target, statement.span);
                if let Some(ty) = target_ty {
                    if !ty.allows_php_append_syntax() {
                        self.diagnostics.push(Diagnostic::new(
                            format!(
                                "PHP array append syntax requires array target, found {}",
                                ty.display_name()
                            ),
                            statement.span,
                        ));
                    }
                }
                self.analyze_expr(&statement.value);
            }
        }
    }

    fn analyze_function_decl(&mut self, statement: &FunctionDeclStmt) {
        let saved_variables = self.variables.clone();
        for param in &statement.params {
            let ty = param
                .ty
                .as_ref()
                .map(|ty| Type::Named(ty.clone()))
                .unwrap_or(Type::Unknown);
            self.bind_variable(&param.name, ty, statement.span);
        }
        self.analyze_statements(&statement.body);
        self.variables = saved_variables;
    }

    fn analyze_expr(&mut self, expr: &Expr) -> Type {
        let ty = match expr {
            Expr::Null(_) => Type::Null,
            Expr::Bool(_) => Type::Bool,
            Expr::String(_) => Type::String,
            Expr::Number(expr) if expr.value.contains(['.', 'e', 'E']) => Type::Float,
            Expr::Number(_) => Type::Int,
            Expr::Variable(expr) => self
                .resolve_variable(&expr.name, expr.span)
                .unwrap_or(Type::Unknown),
            Expr::FunctionCall(expr) => {
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                echo_reflection::function(&expr.name)
                    .and_then(|function| function.return_type.clone())
                    .map(Type::Named)
                    .unwrap_or(Type::Unknown)
            }
            Expr::Defer(expr) => {
                self.analyze_statements(&expr.body);
                Type::Task
            }
            Expr::Run(expr) => {
                match expr {
                    echo_ast::RunExpr::Block { body, .. } => self.analyze_statements(body),
                    echo_ast::RunExpr::Task { expr, .. } => {
                        self.analyze_expr(expr);
                    }
                    echo_ast::RunExpr::Group { entries, .. } => {
                        for entry in entries {
                            self.analyze_statements(entry);
                        }
                        return Type::List;
                    }
                }
                Type::Task
            }
            Expr::Fork(expr) => {
                match expr {
                    echo_ast::ForkExpr::Block { body, .. } => self.analyze_statements(body),
                    echo_ast::ForkExpr::Task { expr, .. } => {
                        self.analyze_expr(expr);
                    }
                }
                Type::Thread
            }
            Expr::Spawn(expr) => {
                self.analyze_expr(&expr.command);
                Type::Process
            }
            Expr::Join(expr) => match self.analyze_expr(&expr.handle) {
                Type::Process => Type::Int,
                Type::Task | Type::Thread | Type::Unknown => Type::Unknown,
                _ => Type::Unknown,
            },
            Expr::Loop(expr) => {
                self.analyze_statements(&expr.body);
                Type::Unknown
            }
            Expr::Unary(expr) => {
                self.analyze_expr(&expr.expr);
                match expr.op {
                    UnaryOp::Plus | UnaryOp::Minus => Type::Number,
                }
            }
            Expr::Binary(expr) => {
                self.analyze_expr(&expr.left);
                self.analyze_expr(&expr.right);
                match expr.op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod
                    | BinaryOp::Pow => Type::Number,
                    BinaryOp::Concat => Type::String,
                    BinaryOp::Is | BinaryOp::IsNot => Type::Bool,
                }
            }
            Expr::Field(expr) => {
                self.analyze_expr(&expr.object);
                Type::Unknown
            }
            Expr::Index(expr) => {
                let collection_ty = self.analyze_expr(&expr.collection);
                self.analyze_expr(&expr.index);
                if !collection_ty.allows_index_access() {
                    self.diagnostics.push(Diagnostic::new(
                        format!(
                            "index access requires array or list target, found {}",
                            collection_ty.display_name()
                        ),
                        expr.span,
                    ));
                }
                Type::Unknown
            }
            Expr::Object(expr) => {
                for field in &expr.fields {
                    self.analyze_expr(&field.value);
                }
                Type::Object(Some(expr.name.clone()))
            }
            Expr::List(expr) => {
                for value in &expr.values {
                    self.analyze_expr(value);
                }
                Type::List
            }
            Expr::Array(expr) => {
                for element in &expr.elements {
                    if let Some(key) = &element.key {
                        self.analyze_expr(key);
                    }
                    self.analyze_expr(&element.value);
                }
                Type::Array
            }
        };
        self.expression_types
            .insert(SpanKey::from(expr.span()), ty.clone());
        ty
    }

    fn bind_variable(&mut self, name: &str, ty: Type, span: Span) {
        self.variables.insert(
            name.to_string(),
            VariableInfo {
                name: name.to_string(),
                ty,
                span,
            },
        );
    }

    fn resolve_variable(&mut self, name: &str, span: Span) -> Option<Type> {
        let ty = self.variables.get(name).map(|variable| variable.ty.clone());
        if ty.is_none() {
            self.diagnostics.push(Diagnostic::new(
                format!("undefined variable `${name}`"),
                span,
            ));
        }
        ty
    }
}

impl Type {
    fn allows_php_append_syntax(&self) -> bool {
        match self {
            Self::Array => true,
            Self::Named(name) => !name.contains('[') && name.starts_with("array"),
            _ => false,
        }
    }

    fn allows_index_access(&self) -> bool {
        match self {
            Self::Array | Self::List => true,
            Self::Named(name) => name.starts_with("array") || name.starts_with("list"),
            Self::Unknown => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use echo_ast::{
        AppendStmt, ArrayExpr, AssignStmt, ClassDeclStmt, ExprStmt, ForkExpr, FunctionDeclStmt,
        ImportStmt, IndexExpr, JoinExpr, LetStmt, ListExpr, MethodDecl, NamespaceStmt,
        NumberLiteral, ObjectExpr, ObjectField, QualifiedName, RunExpr, SpawnExpr, StringLiteral,
        TypeDeclStmt, UseStmt, VariableExpr,
    };

    use super::*;

    fn program(statements: Vec<Stmt>) -> Program {
        Program {
            open_tag: None,
            statements,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn tracks_variable_type_from_let_initializer() {
        let variable = Expr::Variable(VariableExpr {
            name: "a".to_string(),
            span: Span::new(12, 14),
        });
        let analysis = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Expr(ExprStmt {
                expr: variable.clone(),
                span: Span::new(12, 14),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(
            analysis.variable("a").map(|info| &info.ty),
            Some(&Type::Array)
        );
        assert_eq!(analysis.expression_type(&variable), Type::Array);
    }

    #[test]
    fn reports_undefined_variable() {
        let diagnostics = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Variable(VariableExpr {
                name: "missing".to_string(),
                span: Span::new(0, 8),
            }),
            span: Span::new(0, 8),
        })]))
        .expect_err("undefined variable should be diagnostic");

        assert_eq!(diagnostics[0].message, "undefined variable `$missing`");
        assert_eq!(diagnostics[0].span, Span::new(0, 8));
    }

    #[test]
    fn infers_literal_and_operator_types() {
        let analysis = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::String(StringLiteral {
                value: "x".to_string(),
                span: Span::new(0, 3),
            }),
            span: Span::new(0, 3),
        })]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type_at(Span::new(0, 3)), Type::String);

        let analysis = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Number(NumberLiteral {
                value: "1.5".to_string(),
                span: Span::new(0, 3),
            }),
            span: Span::new(0, 3),
        })]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type_at(Span::new(0, 3)), Type::Float);
    }

    #[test]
    fn infers_concurrency_handle_types() {
        let analysis = analyze(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(RunExpr::Block {
                    body: vec![],
                    span: Span::new(0, 8),
                }),
                span: Span::new(0, 8),
            }),
            Stmt::Assign(AssignStmt {
                name: "thread".to_string(),
                value: Expr::Fork(ForkExpr::Block {
                    body: vec![],
                    span: Span::new(9, 18),
                }),
                span: Span::new(9, 18),
            }),
            Stmt::Assign(AssignStmt {
                name: "process".to_string(),
                value: Expr::Spawn(SpawnExpr {
                    command: Box::new(Expr::String(StringLiteral {
                        value: "true".to_string(),
                        span: Span::new(25, 31),
                    })),
                    span: Span::new(19, 31),
                }),
                span: Span::new(19, 31),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(
            analysis.variable("task").map(|info| &info.ty),
            Some(&Type::Task)
        );
        assert_eq!(
            analysis.variable("thread").map(|info| &info.ty),
            Some(&Type::Thread)
        );
        assert_eq!(
            analysis.variable("process").map(|info| &info.ty),
            Some(&Type::Process)
        );
    }

    #[test]
    fn infers_join_type_from_handle_variable_type() {
        let task_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "task".to_string(),
                span: Span::new(32, 37),
            })),
            span: Span::new(27, 37),
        });
        let thread_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "thread".to_string(),
                span: Span::new(48, 55),
            })),
            span: Span::new(43, 55),
        });
        let process_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "process".to_string(),
                span: Span::new(66, 75),
            })),
            span: Span::new(61, 75),
        });
        let analysis = analyze(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(RunExpr::Block {
                    body: vec![],
                    span: Span::new(0, 8),
                }),
                span: Span::new(0, 8),
            }),
            Stmt::Assign(AssignStmt {
                name: "thread".to_string(),
                value: Expr::Fork(ForkExpr::Block {
                    body: vec![],
                    span: Span::new(9, 18),
                }),
                span: Span::new(9, 18),
            }),
            Stmt::Assign(AssignStmt {
                name: "process".to_string(),
                value: Expr::Spawn(SpawnExpr {
                    command: Box::new(Expr::String(StringLiteral {
                        value: "true".to_string(),
                        span: Span::new(25, 31),
                    })),
                    span: Span::new(19, 31),
                }),
                span: Span::new(19, 31),
            }),
            Stmt::Expr(ExprStmt {
                expr: task_join.clone(),
                span: Span::new(27, 37),
            }),
            Stmt::Expr(ExprStmt {
                expr: thread_join.clone(),
                span: Span::new(43, 55),
            }),
            Stmt::Expr(ExprStmt {
                expr: process_join.clone(),
                span: Span::new(61, 75),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type(&task_join), Type::Unknown);
        assert_eq!(analysis.expression_type(&thread_join), Type::Unknown);
        assert_eq!(analysis.expression_type(&process_join), Type::Int);
    }

    #[test]
    fn allows_php_append_syntax_for_arrays() {
        analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(19, 20),
                }),
                span: Span::new(12, 21),
            }),
        ]))
        .expect("array append should analyze");
    }

    #[test]
    fn rejects_php_append_syntax_for_lists() {
        let diagnostics = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::List(ListExpr {
                    values: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(19, 20),
                }),
                span: Span::new(12, 21),
            }),
        ]))
        .expect_err("list append should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "PHP array append syntax requires array target, found list"
        );
    }

    #[test]
    fn rejects_php_append_syntax_for_fixed_size_arrays() {
        let diagnostics = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: Some("array<int>[3]".to_string()),
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(22, 24),
                }),
                span: Span::new(0, 24),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(32, 33),
                }),
                span: Span::new(25, 34),
            }),
        ]))
        .expect_err("fixed-size array append should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "PHP array append syntax requires array target, found array<int>[3]"
        );
    }

    #[test]
    fn allows_index_access_for_arrays_and_lists() {
        analyze(&program(vec![
            Stmt::Expr(ExprStmt {
                expr: Expr::Index(Box::new(IndexExpr {
                    collection: Expr::Array(ArrayExpr {
                        elements: vec![],
                        span: Span::new(0, 2),
                    }),
                    index: Expr::Number(NumberLiteral {
                        value: "0".to_string(),
                        span: Span::new(3, 4),
                    }),
                    span: Span::new(0, 5),
                })),
                span: Span::new(0, 5),
            }),
            Stmt::Expr(ExprStmt {
                expr: Expr::Index(Box::new(IndexExpr {
                    collection: Expr::List(ListExpr {
                        values: vec![],
                        span: Span::new(6, 8),
                    }),
                    index: Expr::Number(NumberLiteral {
                        value: "0".to_string(),
                        span: Span::new(9, 10),
                    }),
                    span: Span::new(6, 11),
                })),
                span: Span::new(6, 11),
            }),
        ]))
        .expect("array and list index access should analyze");
    }

    #[test]
    fn rejects_index_access_for_objects() {
        let diagnostics = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Index(Box::new(IndexExpr {
                collection: Expr::Object(ObjectExpr {
                    name: String::new(),
                    fields: vec![ObjectField {
                        name: "a".to_string(),
                        value: Expr::Number(NumberLiteral {
                            value: "1".to_string(),
                            span: Span::new(5, 6),
                        }),
                    }],
                    span: Span::new(0, 7),
                }),
                index: Expr::String(StringLiteral {
                    value: "a".to_string(),
                    span: Span::new(8, 11),
                }),
                span: Span::new(0, 12),
            })),
            span: Span::new(0, 12),
        })]))
        .expect_err("object index access should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "index access requires array or list target, found object"
        );
    }

    #[test]
    fn extracts_declaration_facts_with_namespace() {
        let facts = index_facts(
            &program(vec![
                Stmt::Namespace(NamespaceStmt {
                    source: NamespaceSource::Php,
                    name: QualifiedName::new(vec!["App".to_string(), "Http".to_string()]),
                    span: Span::new(0, 19),
                }),
                Stmt::FunctionDecl(FunctionDeclStmt {
                    name: "handler".to_string(),
                    params: vec![],
                    return_type: Some("string".to_string()),
                    is_intrinsic: false,
                    is_generator: false,
                    body: vec![],
                    span: Span::new(20, 52),
                }),
                Stmt::ClassDecl(ClassDeclStmt {
                    name: "UserController".to_string(),
                    members: vec![ClassMember::Method(MethodDecl {
                        name: "show".to_string(),
                        params: vec![],
                        return_type: None,
                        is_static: false,
                        is_intrinsic: false,
                        span: Span::new(76, 88),
                    })],
                    span: Span::new(53, 90),
                }),
                Stmt::TypeDecl(TypeDeclStmt {
                    name: "Payload".to_string(),
                    fields: vec![],
                    span: Span::new(91, 106),
                }),
            ]),
            FileId(7),
            EchoFileMode::PhpCompat,
        );

        assert_eq!(facts.file_id, FileId(7));
        assert_eq!(facts.mode, EchoFileMode::PhpCompat);
        assert_eq!(
            facts
                .declarations
                .iter()
                .map(|symbol| (symbol.name.text.as_str(), symbol.kind))
                .collect::<Vec<_>>(),
            vec![
                ("App\\Http", SymbolKind::Namespace),
                ("handler", SymbolKind::Function),
                ("UserController", SymbolKind::Class),
                ("show", SymbolKind::Method),
                ("Payload", SymbolKind::TypeAlias),
            ]
        );
        assert_eq!(
            facts.declarations[1]
                .fq_name
                .as_ref()
                .map(FqName::as_string),
            Some("App\\Http\\handler".to_string())
        );
        assert_eq!(
            facts.declarations[3]
                .fq_name
                .as_ref()
                .map(FqName::as_string),
            Some("App\\Http\\UserController::show".to_string())
        );
    }

    #[test]
    fn extracts_import_dependency_facts() {
        let facts = index_facts(
            &program(vec![
                Stmt::Use(UseStmt {
                    name: QualifiedName::new(vec![
                        "Psr".to_string(),
                        "Log".to_string(),
                        "LoggerInterface".to_string(),
                    ]),
                    alias: Some("Logger".to_string()),
                    span: Span::new(0, 36),
                }),
                Stmt::Import(ImportStmt {
                    source: ImportSource::Std,
                    name: QualifiedName::new(vec!["net".to_string(), "TcpServer".to_string()]),
                    alias: None,
                    span: Span::new(37, 63),
                }),
                Stmt::Import(ImportStmt {
                    source: ImportSource::File("./routes.echo".to_string()),
                    name: QualifiedName::new(vec!["route".to_string()]),
                    alias: Some("appRoute".to_string()),
                    span: Span::new(64, 112),
                }),
            ]),
            FileId(9),
            EchoFileMode::Echo,
        );

        assert_eq!(
            facts
                .dependencies
                .iter()
                .map(|dependency| (
                    dependency.kind,
                    dependency.target.as_str(),
                    dependency.alias.as_deref()
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    DependencyKind::PhpUse,
                    "Psr\\Log\\LoggerInterface",
                    Some("Logger")
                ),
                (DependencyKind::EchoStdImport, "net\\TcpServer", None),
                (
                    DependencyKind::EchoFileImport,
                    "./routes.echo#route",
                    Some("appRoute")
                ),
            ]
        );
    }
}
