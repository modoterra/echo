use super::*;
use echo_ast::{
    AssignExpr, AssignStmt, BinaryExpr, BinaryOp, ClassDeclStmt, ClassMember, Expr, ExprStmt,
    FunctionCallExpr, FunctionDeclStmt, IfStmt, ImportSource, ImportStmt, IncludeExpr, IncludeKind,
    MagicConstantExpr, MagicConstantKind, MethodDecl, MethodVisibility, NamespaceSource,
    NamespaceStmt, QualifiedName, StaticCallExpr, Stmt, StringLiteral, TypeDeclStmt, UseStmt,
    VariableExpr,
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
                parent: None,
                interfaces: Vec::new(),
                members: vec![ClassMember::Method(MethodDecl {
                    name: "show".to_string(),
                    params: vec![],
                    return_type: None,
                    body: vec![],
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
    );

    assert_eq!(facts.file_id, FileId(7));
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
                args: echo_ast::call_args![Expr::Assign(Box::new(AssignExpr {
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
                expr: Expr::Include(Box::new(IncludeExpr {
                    kind: IncludeKind::Require,
                    path: Expr::Variable(VariableExpr {
                        name: "maintenance".to_string(),
                        span: Span::new(82, 94),
                    }),
                    span: Span::new(74, 94),
                })),
                span: Span::new(74, 95),
            })],
            elseif_clauses: Vec::new(),
            else_body: Vec::new(),
            span: Span::new(0, 100),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::Include(Box::new(IncludeExpr {
                kind: IncludeKind::Require,
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
            value: Expr::Include(Box::new(IncludeExpr {
                kind: IncludeKind::RequireOnce,
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

    let facts = index_facts(&source, FileId(10));

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
        expr: Expr::Include(Box::new(IncludeExpr {
            kind: IncludeKind::RequireOnce,
            path: Expr::String(StringLiteral {
                value: "../bootstrap/app.php".to_string(),
                span: Span::new(14, 36),
            }),
            span: Span::new(0, 36),
        })),
        span: Span::new(0, 37),
    })]);
    source.source_dir = Some("/project/public".to_string());

    let facts = index_facts(&source, FileId(13));

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
        value: Expr::Include(Box::new(IncludeExpr {
            kind: IncludeKind::RequireOnce,
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
