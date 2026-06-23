use super::*;
use crate::lowering::lower_syntax_statement;
use echo_ast::{
    AppendStmt, ArrayElement, ArrayExpr, AssignRefStmt, BinaryExpr, BinaryOp, BreakStmt,
    DynamicFunctionCallStmt, EchoStmt, Expr, FieldExpr, FunctionDeclStmt, IfStmt, ImportSource,
    ImportStmt, LetStmt, ListExpr, LoopStmt, NamespaceSource, NamespaceStmt, NullLiteral,
    NumberLiteral, ObjectExpr, ObjectField, Program, QualifiedName, Stmt, StringLiteral,
    TypeDeclStmt, VariableExpr, YieldStmt,
};
use echo_source::Span;

fn program(statements: Vec<Stmt>) -> Program {
    Program {
        open_tag: None,
        statements,
        source_dir: None,
        span: Span::new(0, 0),
    }
}

#[test]
fn lower_program_preserves_hir_syntax_for_codegen() {
    let program = program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::String(StringLiteral {
            value: "hello".to_string(),
            span: Span::new(5, 12),
        })],
        span: Span::new(0, 13),
    })]);
    let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
    let mir = lower_program(&hir).expect("HIR should lower to MIR");

    assert_eq!(mir.statements().len(), program.statements.len());
    assert_eq!(mir.statements()[0].syntax(), &program.statements[0]);
    assert!(matches!(mir.statements()[0], MirStmt::Echo { .. }));
    assert!(mir.imports().is_empty());
    assert!(mir.functions().is_empty());
}

#[test]
fn lower_program_extracts_import_and_user_function_sections() {
    let program = program(vec![
        Stmt::Import(ImportStmt {
            source: ImportSource::Std,
            name: QualifiedName::new(vec!["net".to_string()]),
            alias: None,
            span: Span::new(0, 15),
        }),
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "greet".to_string(),
            params: Vec::new(),
            return_type: None,
            is_intrinsic: false,
            is_generator: false,
            body: vec![Stmt::Return(echo_ast::ReturnStmt {
                value: Expr::String(StringLiteral {
                    value: "hi".to_string(),
                    span: Span::new(26, 30),
                }),
                span: Span::new(19, 31),
            })],
            span: Span::new(16, 29),
        }),
    ]);
    let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
    let mir = lower_program(&hir).expect("HIR should lower to MIR");

    assert_eq!(mir.imports().len(), 1);
    assert_eq!(mir.imports()[0].name.as_string(), "net");
    assert_eq!(mir.functions().len(), 1);
    assert_eq!(mir.functions()[0].name, "greet");
    assert!(matches!(
        mir.functions()[0].body.first(),
        Some(MirStmt::Return { .. })
    ));
    assert_eq!(mir.statements().len(), 2);
}

#[test]
fn lower_program_lowers_common_expression_shapes() {
    let program = program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Binary(Box::new(BinaryExpr {
            left: Expr::Number(NumberLiteral {
                value: "2".to_string(),
                span: Span::new(5, 6),
            }),
            op: BinaryOp::Add,
            right: Expr::Number(NumberLiteral {
                value: "3".to_string(),
                span: Span::new(9, 10),
            }),
            span: Span::new(5, 10),
        }))],
        span: Span::new(0, 11),
    })]);
    let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
    let mir = lower_program(&hir).expect("HIR should lower to MIR");

    let MirStmt::Echo { exprs, .. } = &mir.statements()[0] else {
        panic!("echo statement should lower to MIR echo");
    };
    assert!(matches!(
        &exprs[0],
        MirExpr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
}

#[test]
fn lower_program_lowers_collection_and_access_expression_shapes() {
    let program = program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![
            Expr::List(ListExpr {
                values: vec![Expr::String(StringLiteral {
                    value: "a".to_string(),
                    span: Span::new(1, 4),
                })],
                span: Span::new(0, 5),
            }),
            Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: Some(Expr::String(StringLiteral {
                        value: "k".to_string(),
                        span: Span::new(7, 10),
                    })),
                    value: Expr::Number(NumberLiteral {
                        value: "1".to_string(),
                        span: Span::new(14, 15),
                    }),
                    span: Span::new(7, 15),
                }],
                span: Span::new(6, 16),
            }),
            Expr::Field(Box::new(FieldExpr {
                object: Expr::Object(ObjectExpr {
                    name: "object".to_string(),
                    fields: vec![ObjectField {
                        name: "name".to_string(),
                        value: Expr::String(StringLiteral {
                            value: "Echo".to_string(),
                            span: Span::new(26, 32),
                        }),
                    }],
                    span: Span::new(18, 33),
                }),
                field: "name".to_string(),
                span: Span::new(18, 38),
            })),
        ],
        span: Span::new(0, 39),
    })]);
    let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
    let mir = lower_program(&hir).expect("HIR should lower to MIR");

    let MirStmt::Echo { exprs, .. } = &mir.statements()[0] else {
        panic!("echo statement should lower to MIR echo");
    };

    assert!(matches!(&exprs[0], MirExpr::List { .. }));
    assert!(matches!(&exprs[1], MirExpr::Array { .. }));
    assert!(matches!(
        &exprs[2],
        MirExpr::Field {
            object,
            field,
            ..
        } if field == "name" && matches!(object.as_ref(), MirExpr::Object { .. })
    ));
}

#[test]
fn lower_program_lowers_control_statement_shapes() {
    let program = program(vec![
        Stmt::Let(LetStmt {
            name: "items".to_string(),
            ty: None,
            value: Expr::Array(ArrayExpr {
                elements: Vec::new(),
                span: Span::new(12, 14),
            }),
            span: Span::new(0, 15),
        }),
        Stmt::Let(LetStmt {
            name: "item".to_string(),
            ty: None,
            value: Expr::Null(NullLiteral {
                span: Span::new(27, 31),
            }),
            span: Span::new(17, 32),
        }),
        Stmt::Loop(LoopStmt {
            body: vec![
                Stmt::Append(AppendStmt {
                    target: "items".to_string(),
                    value: Expr::Variable(VariableExpr {
                        name: "item".to_string(),
                        span: Span::new(42, 47),
                    }),
                    span: Span::new(34, 48),
                }),
                Stmt::If(IfStmt {
                    condition: Expr::Binary(Box::new(BinaryExpr {
                        left: Expr::Variable(VariableExpr {
                            name: "item".to_string(),
                            span: Span::new(53, 58),
                        }),
                        op: BinaryOp::Is,
                        right: Expr::Null(NullLiteral {
                            span: Span::new(62, 66),
                        }),
                        span: Span::new(53, 66),
                    })),
                    body: vec![Stmt::Break(BreakStmt {
                        value: None,
                        span: Span::new(69, 75),
                    })],
                    span: Span::new(50, 77),
                }),
            ],
            span: Span::new(34, 78),
        }),
    ]);
    let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
    let mir = lower_program(&hir).expect("HIR should lower to MIR");

    let MirStmt::Loop { body, .. } = &mir.statements()[2] else {
        panic!("loop statement should lower to MIR loop");
    };

    assert!(matches!(&body[0], MirStmt::Append { target, .. } if target == "items"));
    assert!(matches!(
        &body[1],
        MirStmt::If {
            condition: MirExpr::Binary {
                op: BinaryOp::Is,
                ..
            },
            body,
            ..
        } if matches!(body.first(), Some(MirStmt::Break { value: None, .. }))
    ));
}

#[test]
fn lower_syntax_statement_lowers_remaining_statement_shapes() {
    let mut imports = Vec::new();
    let mut functions = Vec::new();

    assert!(matches!(
        lower_syntax_statement(
            &Stmt::DynamicFunctionCall(DynamicFunctionCallStmt {
                name: "handler".to_string(),
                args: Vec::new(),
                span: Span::new(0, 10),
            }),
            &mut imports,
            &mut functions,
        ),
        MirStmt::DynamicFunctionCall { name, .. } if name == "handler"
    ));

    assert!(matches!(
        lower_syntax_statement(
            &Stmt::AssignRef(AssignRefStmt {
                name: "alias".to_string(),
                target: "target".to_string(),
                span: Span::new(11, 22),
            }),
            &mut imports,
            &mut functions,
        ),
        MirStmt::AssignRef { name, target, .. } if name == "alias" && target == "target"
    ));

    assert!(matches!(
        lower_syntax_statement(
            &Stmt::Yield(YieldStmt {
                value: Expr::Null(NullLiteral {
                    span: Span::new(29, 33),
                }),
                span: Span::new(23, 34),
            }),
            &mut imports,
            &mut functions,
        ),
        MirStmt::Yield { .. }
    ));

    assert!(matches!(
        lower_syntax_statement(
            &Stmt::Namespace(NamespaceStmt {
                source: NamespaceSource::Php,
                name: QualifiedName::new(vec!["App".to_string()]),
                span: Span::new(35, 49),
            }),
            &mut imports,
            &mut functions,
        ),
        MirStmt::Noop { .. }
    ));

    assert!(matches!(
        lower_syntax_statement(
            &Stmt::TypeDecl(TypeDeclStmt {
                name: "User".to_string(),
                fields: Vec::new(),
                span: Span::new(50, 62),
            }),
            &mut imports,
            &mut functions,
        ),
        MirStmt::Noop { .. }
    ));
}
