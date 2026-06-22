use super::*;

#[test]
fn array_literals_lower_to_array_runtime() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Array(ArrayExpr {
            elements: vec![ArrayElement {
                key: None,
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(1, 2),
                }),
                span: Span::new(1, 2),
            }],
            span: Span::new(0, 3),
        })],
        span: Span::new(0, 3),
    })]))
    .expect("array literal should lower");

    assert!(ir.contains("declare %EchoValue @echo_value_array_new()"));
    assert!(ir.contains("declare %EchoValue @echo_value_array_append(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_array_new()"));
    assert!(ir.contains("call %EchoValue @echo_value_array_append"));
    assert!(!ir.contains("call %EchoValue @echo_value_list_append"));
}

#[test]
fn keyed_array_literals_lower_to_array_set_runtime() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Array(ArrayExpr {
            elements: vec![ArrayElement {
                key: Some(Expr::String(StringLiteral {
                    value: "key".to_string(),
                    span: Span::new(1, 6),
                })),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(10, 11),
                }),
                span: Span::new(1, 11),
            }],
            span: Span::new(0, 12),
        })],
        span: Span::new(0, 12),
    })]))
    .expect("keyed array should lower");

    assert!(
        ir.contains("declare %EchoValue @echo_value_array_set(%EchoValue, %EchoValue, %EchoValue)")
    );
    assert!(ir.contains("call %EchoValue @echo_value_array_set"));
}

#[test]
fn index_access_lowers_to_index_get_runtime() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Index(Box::new(echo_ast::IndexExpr {
            collection: Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: None,
                    value: Expr::Number(NumberLiteral {
                        value: "4".to_string(),
                        span: Span::new(1, 2),
                    }),
                    span: Span::new(1, 2),
                }],
                span: Span::new(0, 3),
            }),
            index: Expr::Number(NumberLiteral {
                value: "0".to_string(),
                span: Span::new(4, 5),
            }),
            span: Span::new(0, 6),
        }))],
        span: Span::new(0, 6),
    })]))
    .expect("index access should lower");

    assert!(ir.contains("declare %EchoValue @echo_value_index_get(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_index_get"));
}

#[test]
fn append_statement_lowers_to_array_append_runtime() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Let(echo_ast::LetStmt {
            name: "a".to_string(),
            ty: None,
            value: Expr::Array(ArrayExpr {
                elements: vec![],
                span: Span::new(9, 11),
            }),
            span: Span::new(0, 12),
        }),
        Stmt::Append(echo_ast::AppendStmt {
            target: "a".to_string(),
            value: Expr::Number(NumberLiteral {
                value: "2".to_string(),
                span: Span::new(20, 21),
            }),
            span: Span::new(13, 22),
        }),
    ]))
    .expect("array append should lower");

    assert!(ir.contains("declare %EchoValue @echo_value_array_append(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_array_append"));
    assert!(!ir.contains("call %EchoValue @echo_value_list_append"));
}

#[test]
fn list_push_method_lowers_to_list_append_runtime() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Let(LetStmt {
            name: "items".to_string(),
            ty: Some("list<string>".to_string()),
            value: Expr::List(ListExpr {
                values: vec![],
                span: Span::new(27, 29),
            }),
            span: Span::new(0, 29),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Variable(VariableExpr {
                    name: "items".to_string(),
                    span: Span::new(30, 36),
                }),
                method: "push".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "first".to_string(),
                    span: Span::new(42, 49),
                })],
                span: Span::new(30, 50),
            })),
            span: Span::new(30, 50),
        }),
    ]))
    .expect("list push should lower");

    assert!(ir.contains("declare %EchoValue @echo_value_list_append(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_list_append"));
    assert!(!ir.contains("call %EchoValue @echo_value_array_append"));
}

#[test]
fn list_push_lowers_type_ascribed_structural_literal() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Let(LetStmt {
            name: "users".to_string(),
            ty: Some("list<User>".to_string()),
            value: Expr::List(ListExpr {
                values: vec![],
                span: Span::new(24, 26),
            }),
            span: Span::new(0, 26),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Variable(VariableExpr {
                    name: "users".to_string(),
                    span: Span::new(27, 33),
                }),
                method: "push".to_string(),
                args: vec![Expr::TypeAscription(Box::new(TypeAscriptionExpr {
                    expr: Expr::Object(ObjectExpr {
                        name: String::new(),
                        fields: vec![ObjectField {
                            name: "email".to_string(),
                            value: Expr::String(StringLiteral {
                                value: "first@example.test".to_string(),
                                span: Span::new(50, 70),
                            }),
                        }],
                        span: Span::new(39, 72),
                    }),
                    ty: "User".to_string(),
                    span: Span::new(39, 78),
                }))],
                span: Span::new(27, 79),
            })),
            span: Span::new(27, 79),
        }),
    ]))
    .expect("type-ascribed structural literal push should lower");

    assert!(ir.contains("call %EchoValue @echo_value_object_new()"));
    assert!(ir.contains("call %EchoValue @echo_value_object_set"));
    assert!(ir.contains("call %EchoValue @echo_value_list_append"));
}
