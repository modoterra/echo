use std::collections::HashMap;

use echo_ast::{BinaryOp, Expr, FunctionDeclStmt, Program, Stmt, UnaryOp};
use echo_diagnostics::Diagnostic;
use echo_source::Span;

mod index_facts;

pub use index_facts::{index_facts, index_facts_from_source};

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
            Expr::MethodCall(expr) => {
                self.analyze_expr(&expr.object);
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                Type::Unknown
            }
            Expr::StaticCall(expr) => {
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                Type::Unknown
            }
            Expr::Assign(expr) => {
                let ty = self.analyze_expr(&expr.value);
                self.variables.insert(
                    expr.name.clone(),
                    VariableInfo {
                        name: expr.name.clone(),
                        ty: ty.clone(),
                        span: expr.span,
                    },
                );
                ty
            }
            Expr::MagicConstant(_) => Type::String,
            Expr::Require(expr) => {
                self.analyze_expr(&expr.path);
                Type::Bool
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
            Expr::TypeAscription(expr) => {
                self.analyze_expr(&expr.expr);
                Type::Named(expr.ty.clone())
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
        AppendStmt, ArrayExpr, AssignExpr, AssignStmt, BinaryExpr, ClassDeclStmt, ClassMember,
        ExprStmt, ForkExpr, FunctionCallExpr, FunctionDeclStmt, IfStmt, ImportSource, ImportStmt,
        IndexExpr, JoinExpr, LetStmt, ListExpr, MagicConstantExpr, MagicConstantKind, MethodDecl,
        MethodVisibility, NamespaceSource, NamespaceStmt, NumberLiteral, ObjectExpr, ObjectField,
        QualifiedName, RequireExpr, RequireKind, RunExpr, SpawnExpr, StaticCallExpr, StringLiteral,
        TypeDeclStmt, UseStmt, VariableExpr,
    };

    use super::*;
    use echo_index::{
        DependencyKind, EchoFileMode, FileId, FqName, ReferenceKind, SymbolKind, TextRange,
    };

    fn program(statements: Vec<Stmt>) -> Program {
        Program {
            open_tag: None,
            statements,
            source_dir: None,
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
                        visibility: MethodVisibility::Private,
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

    #[test]
    fn extracts_require_dependency_facts_from_expressions() {
        let mut source = program(vec![
            Stmt::If(IfStmt {
                condition: Expr::FunctionCall(FunctionCallExpr {
                    name: "file_exists".to_string(),
                    args: vec![Expr::Assign(Box::new(AssignExpr {
                        name: "maintenance".to_string(),
                        value: Expr::Binary(Box::new(BinaryExpr {
                            left: Expr::MagicConstant(MagicConstantExpr {
                                kind: MagicConstantKind::Dir,
                                span: Span::new(20, 27),
                            }),
                            op: BinaryOp::Concat,
                            right: Expr::String(StringLiteral {
                                value: "/../storage/framework/maintenance.php".to_string(),
                                span: Span::new(28, 67),
                            }),
                            span: Span::new(20, 67),
                        })),
                        span: Span::new(5, 67),
                    }))],
                    span: Span::new(0, 68),
                }),
                body: vec![Stmt::Expr(ExprStmt {
                    expr: Expr::Require(Box::new(RequireExpr {
                        kind: RequireKind::Require,
                        path: Expr::Variable(VariableExpr {
                            name: "maintenance".to_string(),
                            span: Span::new(82, 94),
                        }),
                        span: Span::new(74, 94),
                    })),
                    span: Span::new(74, 95),
                })],
                span: Span::new(0, 100),
            }),
            Stmt::Expr(ExprStmt {
                expr: Expr::Require(Box::new(RequireExpr {
                    kind: RequireKind::Require,
                    path: Expr::Binary(Box::new(BinaryExpr {
                        left: Expr::MagicConstant(MagicConstantExpr {
                            kind: MagicConstantKind::Dir,
                            span: Span::new(108, 115),
                        }),
                        op: BinaryOp::Concat,
                        right: Expr::String(StringLiteral {
                            value: "/../vendor/autoload.php".to_string(),
                            span: Span::new(116, 141),
                        }),
                        span: Span::new(108, 141),
                    })),
                    span: Span::new(100, 141),
                })),
                span: Span::new(100, 142),
            }),
            Stmt::Assign(AssignStmt {
                name: "app".to_string(),
                value: Expr::Require(Box::new(RequireExpr {
                    kind: RequireKind::RequireOnce,
                    path: Expr::Binary(Box::new(BinaryExpr {
                        left: Expr::MagicConstant(MagicConstantExpr {
                            kind: MagicConstantKind::Dir,
                            span: Span::new(163, 170),
                        }),
                        op: BinaryOp::Concat,
                        right: Expr::String(StringLiteral {
                            value: "/../bootstrap/app.php".to_string(),
                            span: Span::new(171, 194),
                        }),
                        span: Span::new(163, 194),
                    })),
                    span: Span::new(150, 194),
                })),
                span: Span::new(143, 195),
            }),
        ]);
        source.source_dir = Some("/project/public".to_string());

        let facts = index_facts(&source, FileId(10), EchoFileMode::PhpCompat);

        assert_eq!(
            facts
                .dependencies
                .iter()
                .map(|dependency| (dependency.kind, dependency.target.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (DependencyKind::Require, "$maintenance"),
                (
                    DependencyKind::ComposerAutoload,
                    "/project/public/../vendor/autoload.php"
                ),
                (
                    DependencyKind::RequireOnce,
                    "/project/public/../bootstrap/app.php"
                ),
            ]
        );
        assert_eq!(facts.dependencies[1].range, TextRange::new(100, 141));
        assert_eq!(facts.dependencies[1].target_range, TextRange::new(108, 141));
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::FilePath
                && reference.name == "/project/public/../storage/framework/maintenance.php"
                && reference.range == TextRange::new(20, 67)
        }));
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::FilePath
                && reference.name == "/project/public/../vendor/autoload.php"
                && reference.range == TextRange::new(108, 141)
        }));
    }

    #[test]
    fn extracts_static_class_reference_facts() {
        let facts = index_facts(
            &program(vec![
                Stmt::Use(UseStmt {
                    name: QualifiedName::new(vec![
                        "Illuminate".to_string(),
                        "Http".to_string(),
                        "Request".to_string(),
                    ]),
                    alias: None,
                    span: Span::new(0, 30),
                }),
                Stmt::Expr(ExprStmt {
                    expr: Expr::StaticCall(StaticCallExpr {
                        class_name: QualifiedName::new(vec!["Request".to_string()]),
                        method: "capture".to_string(),
                        args: Vec::new(),
                        span: Span::new(31, 49),
                    }),
                    span: Span::new(31, 50),
                }),
            ]),
            FileId(11),
            EchoFileMode::PhpCompat,
        );

        assert_eq!(facts.references.len(), 2);
        assert_eq!(facts.references[0].kind, ReferenceKind::ClassLike);
        assert_eq!(facts.references[0].name, "Request");
        assert_eq!(facts.references[0].range, TextRange::new(31, 38));
        assert_eq!(facts.references[1].kind, ReferenceKind::StaticMethod);
        assert_eq!(facts.references[1].name, "capture");
        assert_eq!(facts.references[1].qualifier.as_deref(), Some("Request"));
        assert_eq!(facts.references[1].range, TextRange::new(40, 47));
    }

    #[test]
    fn resolves_plain_require_string_relative_to_source_dir() {
        let mut source = program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Require(Box::new(RequireExpr {
                kind: RequireKind::RequireOnce,
                path: Expr::String(StringLiteral {
                    value: "../bootstrap/app.php".to_string(),
                    span: Span::new(14, 36),
                }),
                span: Span::new(0, 36),
            })),
            span: Span::new(0, 37),
        })]);
        source.source_dir = Some("/project/public".to_string());

        let facts = index_facts(&source, FileId(13), EchoFileMode::PhpCompat);

        assert_eq!(facts.dependencies.len(), 1);
        assert_eq!(facts.dependencies[0].kind, DependencyKind::RequireOnce);
        assert_eq!(
            facts.dependencies[0].target,
            "/project/public/../bootstrap/app.php"
        );
        assert_eq!(facts.dependencies[0].target_range, TextRange::new(14, 36));
    }

    #[test]
    fn extracts_phpdoc_var_local_variable_fact_from_source() {
        let program = program(vec![Stmt::Assign(AssignStmt {
            name: "app".to_string(),
            value: Expr::Require(Box::new(RequireExpr {
                kind: RequireKind::RequireOnce,
                path: Expr::String(StringLiteral {
                    value: "/bootstrap/app.php".to_string(),
                    span: Span::new(49, 69),
                }),
                span: Span::new(36, 69),
            })),
            span: Span::new(29, 70),
        })]);
        let facts = index_facts_from_source(
            "/** @var Application $app */\n$app = require_once '/bootstrap/app.php';",
            &program,
            FileId(12),
            EchoFileMode::PhpCompat,
        );

        let symbol = facts
            .declarations
            .iter()
            .find(|symbol| symbol.kind == SymbolKind::LocalVariable)
            .expect("phpdoc variable symbol");

        assert_eq!(symbol.name.text.as_str(), "app");
        assert_eq!(
            symbol
                .signature
                .as_ref()
                .map(|signature| signature.text.as_str()),
            Some("Application")
        );
        assert_eq!(symbol.selection_range, TextRange::new(21, 25));
    }
}
