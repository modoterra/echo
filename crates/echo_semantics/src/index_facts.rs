use echo_ast::{
    BinaryOp, ClassMember, Expr, FunctionDeclStmt, ImportSource, MagicConstantKind,
    NamespaceSource, Program, RequireKind, Stmt,
};
use echo_index::{
    DependencyFact, DependencyKind, EchoFileMode, FileId, FqName, IndexFacts, ReferenceFact,
    ReferenceKind, Signature, SymbolFact, SymbolKind, SymbolName, TextRange,
};
use echo_source::Span;
use smol_str::SmolStr;

mod phpdoc;

use phpdoc::phpdoc_var_annotations;

pub fn index_facts(program: &Program, file_id: FileId, mode: EchoFileMode) -> IndexFacts {
    let mut extractor = IndexFactExtractor::new(file_id, mode);
    extractor.extract(program);
    extractor.into_facts()
}

pub fn index_facts_from_source(
    source: &str,
    program: &Program,
    file_id: FileId,
    mode: EchoFileMode,
) -> IndexFacts {
    let mut extractor = IndexFactExtractor::new(file_id, mode);
    extractor.source_text = Some(source.to_string());
    extractor.source_dir.clone_from(&program.source_dir);
    extractor.extract_phpdoc_var_source_facts(source);
    extractor.extract_statements(&program.statements);
    extractor.into_facts()
}

struct IndexFactExtractor {
    file_id: FileId,
    mode: EchoFileMode,
    source_text: Option<String>,
    source_dir: Option<String>,
    namespace: Vec<SmolStr>,
    declarations: Vec<SymbolFact>,
    dependencies: Vec<DependencyFact>,
    references: Vec<ReferenceFact>,
}

impl IndexFactExtractor {
    fn new(file_id: FileId, mode: EchoFileMode) -> Self {
        Self {
            file_id,
            mode,
            source_text: None,
            source_dir: None,
            namespace: Vec::new(),
            declarations: Vec::new(),
            dependencies: Vec::new(),
            references: Vec::new(),
        }
    }

    fn extract(&mut self, program: &Program) {
        self.source_dir.clone_from(&program.source_dir);
        self.extract_statements(&program.statements);
    }

    fn extract_phpdoc_var_source_facts(&mut self, source: &str) {
        for annotation in phpdoc_var_annotations(source) {
            self.declarations.push(SymbolFact {
                name: SymbolName::new(annotation.variable.as_str()),
                fq_name: Some(self.fq_name(&annotation.variable)),
                kind: SymbolKind::LocalVariable,
                range: annotation.range,
                selection_range: annotation.selection_range,
                visibility: None,
                signature: Some(Signature {
                    text: annotation.ty.clone(),
                }),
            });
            self.references.push(ReferenceFact {
                kind: ReferenceKind::ClassLike,
                name: annotation.ty,
                qualifier: None,
                range: annotation.ty_range,
            });
        }
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
                let target = statement.name.as_string();
                self.dependencies.push(DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: target.clone(),
                    alias: statement.alias.clone(),
                    range: span_range(statement.span),
                    target_range: self
                        .span_range_for_text(statement.span, &target)
                        .unwrap_or_else(|| span_range(statement.span)),
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
                    target_range: span_range(statement.span),
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
                    selection_range: self
                        .span_range_for_text(statement.span, &statement.name)
                        .unwrap_or_else(|| span_range(statement.span)),
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
                        selection_range: self
                            .span_range_for_text(method.span, &method.name)
                            .unwrap_or_else(|| span_range(method.span)),
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
            Stmt::If(statement) => {
                self.extract_expr_dependencies(&statement.condition);
                self.extract_statements(&statement.body);
            }
            Stmt::Echo(statement) => {
                for expr in &statement.exprs {
                    self.extract_expr_dependencies(expr);
                }
            }
            Stmt::FunctionCall(statement) => {
                for expr in &statement.args {
                    self.extract_expr_dependencies(expr);
                }
            }
            Stmt::DynamicFunctionCall(statement) => {
                for expr in &statement.args {
                    self.extract_expr_dependencies(expr);
                }
            }
            Stmt::Assign(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Let(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::AssignRef(_) | Stmt::Break(_) => {}
            Stmt::Return(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Yield(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Expr(statement) => self.extract_expr_dependencies(&statement.expr),
            Stmt::Append(statement) => self.extract_expr_dependencies(&statement.value),
        }
    }

    fn extract_expr_dependencies(&mut self, expr: &Expr) {
        match expr {
            Expr::FunctionCall(expr) => {
                for arg in &expr.args {
                    self.extract_expr_dependencies(arg);
                }
            }
            Expr::MethodCall(expr) => {
                if let Expr::Variable(variable) = &expr.object {
                    let method_start = expr.object.span().end + 2;
                    self.references.push(ReferenceFact {
                        kind: ReferenceKind::Method,
                        name: expr.method.clone(),
                        qualifier: Some(variable.name.clone()),
                        range: TextRange::new(
                            method_start as u32,
                            method_start.saturating_add(expr.method.len()) as u32,
                        ),
                    });
                }
                self.extract_expr_dependencies(&expr.object);
                for arg in &expr.args {
                    self.extract_expr_dependencies(arg);
                }
            }
            Expr::StaticCall(expr) => {
                let name = expr.class_name.as_string();
                self.references.push(ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    range: TextRange::new(
                        expr.span.start as u32,
                        expr.span.start.saturating_add(name.len()) as u32,
                    ),
                    name,
                    qualifier: None,
                });
                let method_start = expr.span.start + expr.class_name.as_string().len() + 2;
                self.references.push(ReferenceFact {
                    kind: ReferenceKind::StaticMethod,
                    name: expr.method.clone(),
                    qualifier: Some(expr.class_name.as_string()),
                    range: TextRange::new(
                        method_start as u32,
                        method_start.saturating_add(expr.method.len()) as u32,
                    ),
                });
                for arg in &expr.args {
                    self.extract_expr_dependencies(arg);
                }
            }
            Expr::Assign(expr) => self.extract_expr_dependencies(&expr.value),
            Expr::Require(expr) => {
                let target = self
                    .const_string_expr(&expr.path)
                    .map(|target| self.resolve_source_path(&target))
                    .unwrap_or_else(|| expr_path_label(&expr.path));
                self.dependencies.push(DependencyFact {
                    kind: require_dependency_kind(expr.kind, &target),
                    target,
                    alias: None,
                    range: span_range(expr.span),
                    target_range: span_range(expr.path.span()),
                });
                self.extract_expr_dependencies(&expr.path);
            }
            Expr::Defer(expr) => self.extract_statements(&expr.body),
            Expr::Run(expr) => match expr {
                echo_ast::RunExpr::Block { body, .. } => self.extract_statements(body),
                echo_ast::RunExpr::Task { expr, .. } => self.extract_expr_dependencies(expr),
                echo_ast::RunExpr::Group { entries, .. } => {
                    for entry in entries {
                        self.extract_statements(entry);
                    }
                }
            },
            Expr::Fork(expr) => match expr {
                echo_ast::ForkExpr::Block { body, .. } => self.extract_statements(body),
                echo_ast::ForkExpr::Task { expr, .. } => self.extract_expr_dependencies(expr),
            },
            Expr::Spawn(expr) => self.extract_expr_dependencies(&expr.command),
            Expr::Join(expr) => self.extract_expr_dependencies(&expr.handle),
            Expr::Loop(expr) => self.extract_statements(&expr.body),
            Expr::Unary(expr) => self.extract_expr_dependencies(&expr.expr),
            Expr::Binary(expr) => {
                if let Some(target) = self.const_dir_path_binary(expr) {
                    self.references.push(ReferenceFact {
                        kind: ReferenceKind::FilePath,
                        name: target,
                        qualifier: None,
                        range: span_range(expr.span),
                    });
                }
                self.extract_expr_dependencies(&expr.left);
                self.extract_expr_dependencies(&expr.right);
            }
            Expr::TypeAscription(expr) => self.extract_expr_dependencies(&expr.expr),
            Expr::Field(expr) => self.extract_expr_dependencies(&expr.object),
            Expr::Index(expr) => {
                self.extract_expr_dependencies(&expr.collection);
                self.extract_expr_dependencies(&expr.index);
            }
            Expr::Object(expr) => {
                for field in &expr.fields {
                    self.extract_expr_dependencies(&field.value);
                }
            }
            Expr::List(expr) => {
                for value in &expr.values {
                    self.extract_expr_dependencies(value);
                }
            }
            Expr::Array(expr) => {
                for element in &expr.elements {
                    if let Some(key) = &element.key {
                        self.extract_expr_dependencies(key);
                    }
                    self.extract_expr_dependencies(&element.value);
                }
            }
            Expr::Null(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Number(_)
            | Expr::Variable(_)
            | Expr::MagicConstant(_) => {}
        }
    }

    fn const_string_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::String(expr) => Some(expr.value.clone()),
            Expr::MagicConstant(expr) if expr.kind == MagicConstantKind::Dir => {
                self.source_dir.clone()
            }
            Expr::Binary(expr) if expr.op == BinaryOp::Concat => self.const_string_binary(expr),
            _ => None,
        }
    }

    fn const_string_binary(&self, expr: &echo_ast::BinaryExpr) -> Option<String> {
        if expr.op != BinaryOp::Concat {
            return None;
        }
        let left = self.const_string_expr(&expr.left)?;
        let right = self.const_string_expr(&expr.right)?;
        Some(format!("{left}{right}"))
    }

    fn const_dir_path_binary(&self, expr: &echo_ast::BinaryExpr) -> Option<String> {
        if !binary_contains_dir_magic_constant(expr) {
            return None;
        }
        self.const_string_binary(expr)
    }

    fn resolve_source_path(&self, target: &str) -> String {
        if std::path::Path::new(target).is_absolute() {
            return target.to_string();
        }
        self.source_dir
            .as_ref()
            .map(|source_dir| {
                std::path::Path::new(source_dir)
                    .join(target)
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| target.to_string())
    }

    fn fq_name(&self, name: &str) -> FqName {
        FqName::new(self.namespace.clone(), name)
    }

    fn span_range_for_text(&self, span: Span, needle: &str) -> Option<TextRange> {
        let source = self.source_text.as_ref()?;
        let haystack = source.get(span.start..span.end)?;
        let start = haystack.find(needle)? + span.start;
        Some(TextRange::new(start as u32, (start + needle.len()) as u32))
    }

    fn into_facts(self) -> IndexFacts {
        IndexFacts {
            file_id: self.file_id,
            mode: self.mode,
            declarations: self.declarations,
            dependencies: self.dependencies,
            references: self.references,
        }
    }
}

fn span_range(span: Span) -> TextRange {
    TextRange::new(span.start as u32, span.end as u32)
}

fn expr_contains_dir_magic_constant(expr: &Expr) -> bool {
    match expr {
        Expr::MagicConstant(expr) => expr.kind == MagicConstantKind::Dir,
        Expr::Binary(expr) => binary_contains_dir_magic_constant(expr),
        _ => false,
    }
}

fn binary_contains_dir_magic_constant(expr: &echo_ast::BinaryExpr) -> bool {
    expr_contains_dir_magic_constant(&expr.left) || expr_contains_dir_magic_constant(&expr.right)
}

fn require_dependency_kind(kind: RequireKind, target: &str) -> DependencyKind {
    if target.ends_with("/vendor/autoload.php") || target.ends_with("\\vendor\\autoload.php") {
        DependencyKind::ComposerAutoload
    } else {
        match kind {
            RequireKind::Require => DependencyKind::Require,
            RequireKind::RequireOnce => DependencyKind::RequireOnce,
        }
    }
}

fn expr_path_label(expr: &Expr) -> String {
    match expr {
        Expr::Variable(expr) => format!("${}", expr.name),
        _ => "<dynamic path>".to_string(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use echo_ast::{
        AssignExpr, AssignStmt, BinaryExpr, ClassDeclStmt, ClassMember, Expr, ExprStmt,
        FunctionCallExpr, FunctionDeclStmt, IfStmt, ImportSource, ImportStmt, MagicConstantExpr,
        MagicConstantKind, MethodDecl, MethodVisibility, NamespaceSource, NamespaceStmt,
        QualifiedName, RequireExpr, RequireKind, StaticCallExpr, Stmt, StringLiteral, TypeDeclStmt,
        UseStmt, VariableExpr,
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
