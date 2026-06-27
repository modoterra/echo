use echo_ast::{
    AppendStmt, ArrayExpr, AssignStmt, ClassDeclStmt, ClassMember, Expr, ExprStmt, ForkExpr,
    IndexExpr, JoinExpr, LetStmt, ListExpr, MethodDecl, MethodVisibility, NumberLiteral,
    ObjectExpr, ObjectField, Program, ReceiverConst, ReceiverConstExpr, RunExpr, SpawnExpr, Stmt,
    StringLiteral, UnnamedExportStmt, VariableExpr,
};
use echo_source::Span;

use super::*;

fn program(statements: Vec<Stmt>) -> Program {
    Program {
        open_tag: None,
        statements,
        source_dir: None,
        span: Span::new(0, 0),
    }
}

fn receiver(kind: ReceiverConst) -> Expr {
    Expr::ReceiverConst(ReceiverConstExpr {
        kind,
        span: Span::new(0, 5),
    })
}

fn method(name: &str, is_static: bool, body: Vec<Stmt>) -> ClassMember {
    ClassMember::Method(MethodDecl {
        name: name.to_string(),
        params: Vec::new(),
        return_type: None,
        body,
        visibility: MethodVisibility::Public,
        is_static,
        is_intrinsic: false,
        span: Span::new(0, 0),
    })
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
fn rejects_duplicate_unnamed_exports() {
    let diagnostics = analyze(&program(vec![
        Stmt::UnnamedExport(UnnamedExportStmt {
            value: Expr::Object(ObjectExpr {
                name: String::new(),
                fields: Vec::new(),
                span: Span::new(0, 2),
            }),
            span: Span::new(0, 8),
        }),
        Stmt::UnnamedExport(UnnamedExportStmt {
            value: Expr::Object(ObjectExpr {
                name: String::new(),
                fields: Vec::new(),
                span: Span::new(9, 11),
            }),
            span: Span::new(9, 17),
        }),
    ]))
    .expect_err("duplicate unnamed exports should be diagnostic");

    assert_eq!(
        diagnostics[0].message,
        "only one unnamed export is allowed per module."
    );
}

#[test]
fn rejects_this_outside_instance_receiver_context() {
    let diagnostics = analyze(&program(vec![Stmt::Expr(ExprStmt {
        expr: receiver(ReceiverConst::This),
        span: Span::new(0, 5),
    })]))
    .expect_err("$this outside receiver context should be diagnostic");

    assert_eq!(
        diagnostics[0].message,
        "$this is only available inside instance receiver contexts."
    );
}

#[test]
fn rejects_receiver_constant_assignment() {
    let diagnostics = analyze(&program(vec![Stmt::Assign(AssignStmt {
        name: "this".to_string(),
        value: Expr::Variable(VariableExpr {
            name: "Other".to_string(),
            span: Span::new(8, 13),
        }),
        span: Span::new(0, 13),
    })]))
    .expect_err("receiver constant assignment should be diagnostic");

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message
        == "$this is a compiler-provided receiver constant and cannot be assigned."));
}

#[test]
fn rejects_static_receiver_until_late_static_binding_exists() {
    let diagnostics = analyze(&program(vec![Stmt::ClassDecl(ClassDeclStmt {
        name: "User".to_string(),
        parent: None,
        interfaces: Vec::new(),
        members: vec![method(
            "make",
            true,
            vec![Stmt::Expr(ExprStmt {
                expr: receiver(ReceiverConst::Static),
                span: Span::new(0, 7),
            })],
        )],
        span: Span::new(0, 0),
    })]))
    .expect_err("$static should be reserved");

    assert_eq!(
        diagnostics[0].message,
        "$static is reserved for late static binding and is not implemented yet."
    );
}

#[test]
fn rejects_parent_without_lexical_parent() {
    let diagnostics = analyze(&program(vec![Stmt::ClassDecl(ClassDeclStmt {
        name: "User".to_string(),
        parent: None,
        interfaces: Vec::new(),
        members: vec![method(
            "boot",
            false,
            vec![Stmt::Expr(ExprStmt {
                expr: receiver(ReceiverConst::Parent),
                span: Span::new(0, 7),
            })],
        )],
        span: Span::new(0, 0),
    })]))
    .expect_err("$parent without parent should be diagnostic");

    assert_eq!(
        diagnostics[0].message,
        "$parent is only available when the lexical type has a parent."
    );
}

#[test]
fn accepts_this_and_self_inside_instance_method() {
    analyze(&program(vec![Stmt::ClassDecl(ClassDeclStmt {
        name: "User".to_string(),
        parent: None,
        interfaces: Vec::new(),
        members: vec![method(
            "save",
            false,
            vec![
                Stmt::Expr(ExprStmt {
                    expr: receiver(ReceiverConst::This),
                    span: Span::new(0, 5),
                }),
                Stmt::Expr(ExprStmt {
                    expr: receiver(ReceiverConst::SelfType),
                    span: Span::new(6, 11),
                }),
            ],
        )],
        span: Span::new(0, 0),
    })]))
    .expect("$this and $self should be valid in instance methods");
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
            target: Expr::Variable(VariableExpr {
                name: "a".to_string(),
                span: Span::new(12, 14),
            }),
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
fn allows_php_append_syntax_for_unknown_targets() {
    analyze(&program(vec![
        Stmt::Let(LetStmt {
            name: "a".to_string(),
            ty: None,
            value: Expr::FunctionCall(echo_ast::FunctionCallExpr {
                name: "make_array".to_string(),
                args: vec![],
                span: Span::new(8, 20),
            }),
            span: Span::new(0, 20),
        }),
        Stmt::Append(AppendStmt {
            target: Expr::Variable(VariableExpr {
                name: "a".to_string(),
                span: Span::new(21, 23),
            }),
            value: Expr::Number(NumberLiteral {
                value: "1".to_string(),
                span: Span::new(28, 29),
            }),
            span: Span::new(21, 30),
        }),
    ]))
    .expect("unknown append target should analyze in PHP-compatible mode");
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
            target: Expr::Variable(VariableExpr {
                name: "a".to_string(),
                span: Span::new(12, 14),
            }),
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
            target: Expr::Variable(VariableExpr {
                name: "a".to_string(),
                span: Span::new(25, 27),
            }),
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
fn allows_index_access_for_strings() {
    analyze(&program(vec![Stmt::Expr(ExprStmt {
        expr: Expr::Index(Box::new(IndexExpr {
            collection: Expr::String(StringLiteral {
                value: "echo".to_string(),
                span: Span::new(0, 6),
            }),
            index: Expr::Number(NumberLiteral {
                value: "0".to_string(),
                span: Span::new(7, 8),
            }),
            span: Span::new(0, 9),
        })),
        span: Span::new(0, 9),
    })]))
    .expect("PHP string offset access should analyze");
}

#[test]
fn allows_index_access_for_named_strings() {
    analyze(&program(vec![
        Stmt::Let(LetStmt {
            name: "value".to_string(),
            ty: Some("string".to_string()),
            value: Expr::String(StringLiteral {
                value: "echo".to_string(),
                span: Span::new(20, 26),
            }),
            span: Span::new(0, 26),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::Index(Box::new(IndexExpr {
                collection: Expr::Variable(VariableExpr {
                    name: "value".to_string(),
                    span: Span::new(27, 33),
                }),
                index: Expr::Number(NumberLiteral {
                    value: "0".to_string(),
                    span: Span::new(34, 35),
                }),
                span: Span::new(27, 36),
            })),
            span: Span::new(27, 36),
        }),
    ]))
    .expect("PHP typed string offset access should analyze");
}

#[test]
fn allows_unset_index_targets_without_reading_collection_type() {
    analyze(&program(vec![
        Stmt::Let(LetStmt {
            name: "value".to_string(),
            ty: Some("bool".to_string()),
            value: Expr::Bool(echo_ast::BoolLiteral {
                value: true,
                span: Span::new(13, 17),
            }),
            span: Span::new(0, 17),
        }),
        Stmt::FunctionCall(echo_ast::FunctionCallStmt {
            name: "unset".to_string(),
            args: vec![echo_ast::CallArg::positional(Expr::Index(Box::new(
                IndexExpr {
                    collection: Expr::Variable(VariableExpr {
                        name: "value".to_string(),
                        span: Span::new(24, 30),
                    }),
                    index: Expr::Number(NumberLiteral {
                        value: "0".to_string(),
                        span: Span::new(31, 32),
                    }),
                    span: Span::new(24, 33),
                },
            )))],
            span: Span::new(18, 34),
        }),
    ]))
    .expect("unset index targets should not be analyzed as value reads");
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
