use super::*;

#[test]
fn dynamic_concat_uses_echo_value_concat() {
    let ir = compile_to_ir(&program(vec![
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "greet".to_string(),
            params: vec![param("name")],
            return_type: None,
            is_intrinsic: false,
            is_generator: false,
            body: vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::Binary(Box::new(echo_ast::BinaryExpr {
                    left: Expr::Binary(Box::new(echo_ast::BinaryExpr {
                        left: Expr::String(StringLiteral {
                            value: "Hello, ".to_string(),
                            span: Span::new(0, 9),
                        }),
                        op: BinaryOp::Concat,
                        right: Expr::Variable(echo_ast::VariableExpr {
                            name: "name".to_string(),
                            span: Span::new(12, 17),
                        }),
                        span: Span::new(0, 17),
                    })),
                    op: BinaryOp::Concat,
                    right: Expr::String(StringLiteral {
                        value: "!\n".to_string(),
                        span: Span::new(20, 24),
                    }),
                    span: Span::new(0, 24),
                }))],
                span: Span::new(0, 30),
            })],
            span: Span::new(0, 40),
        }),
        Stmt::FunctionCall(FunctionCallStmt {
            name: "greet".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "Echo".to_string(),
                span: Span::new(45, 51),
            })],
            span: Span::new(41, 53),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_value_concat(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_value_concat"), "{ir}");
}

#[test]
fn integer_subtraction_lowers_to_echo_value_sub() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Binary(Box::new(echo_ast::BinaryExpr {
            left: Expr::Number(NumberLiteral {
                value: "3".to_string(),
                span: Span::new(5, 6),
            }),
            op: BinaryOp::Sub,
            right: Expr::Number(NumberLiteral {
                value: "5".to_string(),
                span: Span::new(7, 8),
            }),
            span: Span::new(5, 8),
        }))],
        span: Span::new(0, 9),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_value_sub(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_value_sub"), "{ir}");
}

#[test]
fn instanceof_literal_class_name_does_not_lower_rhs_as_constant_lookup() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Binary(Box::new(echo_ast::BinaryExpr {
            left: Expr::Null(NullLiteral {
                span: Span::new(5, 9),
            }),
            op: BinaryOp::InstanceOf,
            right: Expr::Constant(echo_ast::ConstantExpr {
                name: "Closure".to_string(),
                span: Span::new(21, 28),
            }),
            span: Span::new(5, 28),
        }))],
        span: Span::new(0, 29),
    })]))
    .expect("instanceof literal class-name operand should compile");

    assert!(!ir.contains("Closure"), "{ir}");
}
