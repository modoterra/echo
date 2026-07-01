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
            target: Expr::Variable(echo_ast::VariableExpr {
                name: "a".to_string(),
                span: Span::new(13, 15),
            }),
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
                method_span: Span::new(37, 41),
                args: echo_ast::call_args![Expr::String(StringLiteral {
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
                method_span: Span::new(34, 38),
                args: echo_ast::call_args![Expr::TypeAscription(Box::new(TypeAscriptionExpr {
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

#[test]
fn array_keys_lowers_optional_filter_and_strict_defaults() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "array_keys".to_string(),
            args: echo_ast::call_args![Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: Some(Expr::String(StringLiteral {
                        value: "id".to_string(),
                        span: Span::new(12, 16),
                    })),
                    value: Expr::Number(NumberLiteral {
                        value: "10".to_string(),
                        span: Span::new(20, 22),
                    }),
                    span: Span::new(12, 22),
                }],
                span: Span::new(11, 23),
            })],
            span: Span::new(0, 24),
        })],
        span: Span::new(0, 25),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_keys(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("%EchoValue { i32 6, i64 0 }, %EchoValue { i32 1, i64 0 })"),
        "{ir}"
    );
}

#[test]
fn array_first_and_last_lower_to_runtime_calls() {
    let array = Expr::Array(ArrayExpr {
        elements: vec![ArrayElement {
            key: Some(Expr::String(StringLiteral {
                value: "id".to_string(),
                span: Span::new(12, 16),
            })),
            value: Expr::Number(NumberLiteral {
                value: "10".to_string(),
                span: Span::new(20, 22),
            }),
            span: Span::new(12, 22),
        }],
        span: Span::new(11, 23),
    });
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![
            Expr::FunctionCall(FunctionCallExpr {
                name: "array_first".to_string(),
                args: echo_ast::call_args![array.clone()],
                span: Span::new(0, 24),
            }),
            Expr::FunctionCall(FunctionCallExpr {
                name: "array_last".to_string(),
                args: echo_ast::call_args![array],
                span: Span::new(25, 48),
            }),
        ],
        span: Span::new(0, 49),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_first(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_php_array_last(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_first("),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_php_array_last("), "{ir}");
}

#[test]
fn in_array_lowers_optional_strict_default() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "in_array".to_string(),
            args: echo_ast::call_args![
                Expr::Number(NumberLiteral {
                    value: "2".to_string(),
                    span: Span::new(9, 10),
                }),
                Expr::Array(ArrayExpr {
                    elements: vec![ArrayElement {
                        key: None,
                        value: Expr::String(StringLiteral {
                            value: "2".to_string(),
                            span: Span::new(13, 16),
                        }),
                        span: Span::new(13, 16),
                    }],
                    span: Span::new(12, 17),
                }),
            ],
            span: Span::new(0, 18),
        })],
        span: Span::new(0, 19),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_in_array(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("%EchoValue { i32 1, i64 0 })"), "{ir}");
}

#[test]
fn array_fill_lowers_to_three_echo_value_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "array_fill".to_string(),
            args: echo_ast::call_args![
                Expr::Number(NumberLiteral {
                    value: "-2".to_string(),
                    span: Span::new(11, 13),
                }),
                Expr::Number(NumberLiteral {
                    value: "3".to_string(),
                    span: Span::new(15, 16),
                }),
                Expr::String(StringLiteral {
                    value: "x".to_string(),
                    span: Span::new(18, 21),
                }),
            ],
            span: Span::new(0, 22),
        })],
        span: Span::new(0, 23),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_fill(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_php_array_fill("), "{ir}");
}

#[test]
fn array_reverse_lowers_optional_preserve_keys_default() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "array_reverse".to_string(),
            args: echo_ast::call_args![Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: None,
                    value: Expr::String(StringLiteral {
                        value: "x".to_string(),
                        span: Span::new(15, 18),
                    }),
                    span: Span::new(15, 18),
                }],
                span: Span::new(14, 19),
            })],
            span: Span::new(0, 20),
        })],
        span: Span::new(0, 21),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_reverse(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("%EchoValue { i32 1, i64 0 })"), "{ir}");
}

#[test]
fn array_combine_and_pad_lower_to_runtime_calls() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_combine".to_string(),
                args: echo_ast::call_args![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "sku".to_string(),
                                span: Span::new(15, 20),
                            }),
                            span: Span::new(15, 20),
                        }],
                        span: Span::new(14, 21),
                    }),
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "A-42".to_string(),
                                span: Span::new(24, 30),
                            }),
                            span: Span::new(24, 30),
                        }],
                        span: Span::new(23, 31),
                    }),
                ],
                span: Span::new(0, 32),
            })],
            span: Span::new(0, 33),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_pad".to_string(),
                args: echo_ast::call_args![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "x".to_string(),
                                span: Span::new(50, 53),
                            }),
                            span: Span::new(50, 53),
                        }],
                        span: Span::new(49, 54),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "3".to_string(),
                        span: Span::new(56, 57),
                    }),
                    Expr::String(StringLiteral {
                        value: "missing".to_string(),
                        span: Span::new(59, 68),
                    }),
                ],
                span: Span::new(35, 69),
            })],
            span: Span::new(35, 70),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_combine(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_php_array_pad(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_combine("),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_php_array_pad("), "{ir}");
}

#[test]
fn array_slice_and_chunk_lower_optional_arguments_to_runtime_calls() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_slice".to_string(),
                args: echo_ast::call_args![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "active".to_string(),
                                span: Span::new(13, 21),
                            }),
                            span: Span::new(13, 21),
                        }],
                        span: Span::new(12, 22),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "1".to_string(),
                        span: Span::new(24, 25),
                    }),
                ],
                span: Span::new(0, 26),
            })],
            span: Span::new(0, 27),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_chunk".to_string(),
                args: echo_ast::call_args![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "active".to_string(),
                                span: Span::new(41, 49),
                            }),
                            span: Span::new(41, 49),
                        }],
                        span: Span::new(40, 50),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "2".to_string(),
                        span: Span::new(52, 53),
                    }),
                ],
                span: Span::new(28, 54),
            })],
            span: Span::new(28, 55),
        }),
    ]))
    .expect("IR");

    assert!(
            ir.contains(
                "declare %EchoValue @echo_php_array_slice(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
            ),
            "{ir}"
        );
    assert!(
        ir.contains("declare %EchoValue @echo_php_array_chunk(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_slice(")
            && ir.contains("%EchoValue { i32 0, i64 0 }, %EchoValue { i32 1, i64 0 })"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_chunk(")
            && ir.contains("%EchoValue { i32 1, i64 0 })"),
        "{ir}"
    );
}

#[test]
fn array_merge_and_replace_pack_variadic_arguments_for_runtime_calls() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_merge".to_string(),
                args: echo_ast::call_args![],
                span: Span::new(0, 13),
            })],
            span: Span::new(0, 14),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_replace".to_string(),
                args: echo_ast::call_args![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: Some(Expr::String(StringLiteral {
                                value: "sku".to_string(),
                                span: Span::new(30, 35),
                            })),
                            value: Expr::String(StringLiteral {
                                value: "A-42".to_string(),
                                span: Span::new(39, 45),
                            }),
                            span: Span::new(30, 45),
                        }],
                        span: Span::new(29, 46),
                    }),
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: Some(Expr::String(StringLiteral {
                                value: "sku".to_string(),
                                span: Span::new(49, 54),
                            })),
                            value: Expr::String(StringLiteral {
                                value: "A-43".to_string(),
                                span: Span::new(58, 64),
                            }),
                            span: Span::new(49, 64),
                        }],
                        span: Span::new(48, 65),
                    }),
                ],
                span: Span::new(15, 66),
            })],
            span: Span::new(15, 67),
        }),
    ]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_php_array_merge(%EchoValue)"));
    assert!(ir.contains("declare %EchoValue @echo_php_array_replace(%EchoValue)"));
    assert!(
        ir.contains("call %EchoValue @echo_value_array_new()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_value_array_append("),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_merge("),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_replace("),
        "{ir}"
    );
}

#[test]
fn array_search_and_count_values_lower_to_runtime_calls() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_search".to_string(),
                args: echo_ast::call_args![
                    Expr::String(StringLiteral {
                        value: "active".to_string(),
                        span: Span::new(13, 21),
                    }),
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "active".to_string(),
                                span: Span::new(24, 32),
                            }),
                            span: Span::new(24, 32),
                        }],
                        span: Span::new(23, 33),
                    }),
                ],
                span: Span::new(0, 34),
            })],
            span: Span::new(0, 35),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_count_values".to_string(),
                args: echo_ast::call_args![Expr::Array(ArrayExpr {
                    elements: vec![ArrayElement {
                        key: None,
                        value: Expr::String(StringLiteral {
                            value: "active".to_string(),
                            span: Span::new(58, 66),
                        }),
                        span: Span::new(58, 66),
                    }],
                    span: Span::new(57, 67),
                })],
                span: Span::new(37, 68),
            })],
            span: Span::new(37, 69),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains(
            "declare %EchoValue @echo_php_array_search(%EchoValue, %EchoValue, %EchoValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_php_array_count_values(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_search("),
        "{ir}"
    );
    assert!(ir.contains("%EchoValue { i32 1, i64 0 })"), "{ir}");
    assert!(
        ir.contains("call %EchoValue @echo_php_array_count_values("),
        "{ir}"
    );
}
