use super::*;

#[test]
fn environment_process_builtins_lower_to_php_runtime_calls() {
    let getenv_ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "getenv".to_string(),
                args: echo_ast::call_args![],
                span: Span::new(0, 8),
            })],
            span: Span::new(0, 9),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "getenv".to_string(),
                args: echo_ast::call_args![Expr::String(StringLiteral {
                    value: "APP_ENV".to_string(),
                    span: Span::new(18, 27),
                })],
                span: Span::new(10, 28),
            })],
            span: Span::new(10, 29),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "getenv".to_string(),
                args: echo_ast::call_args![
                    Expr::String(StringLiteral {
                        value: "APP_ENV".to_string(),
                        span: Span::new(38, 47),
                    }),
                    Expr::Bool(BoolLiteral {
                        value: true,
                        span: Span::new(49, 53),
                    }),
                ],
                span: Span::new(30, 54),
            })],
            span: Span::new(30, 55),
        }),
    ]))
    .expect("IR");

    assert!(
        getenv_ir.contains("declare %EchoValue @echo_php_getenv(%EchoValue, %EchoValue)"),
        "{getenv_ir}"
    );
    assert!(
            getenv_ir.contains("call %EchoValue @echo_php_getenv(%EchoValue { i32 0, i64 0 }, %EchoValue { i32 1, i64 0 })"),
            "{getenv_ir}"
        );
    assert!(
            getenv_ir.contains("call %EchoValue @echo_php_getenv(%EchoValue %runtime_call_1, %EchoValue { i32 1, i64 0 })"),
            "{getenv_ir}"
        );
    assert!(
            getenv_ir.contains("call %EchoValue @echo_php_getenv(%EchoValue %runtime_call_3, %EchoValue { i32 1, i64 1 })"),
            "{getenv_ir}"
        );

    for name in ["gethostname", "getmypid"] {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: name.to_string(),
                args: echo_ast::call_args![],
                span: Span::new(0, name.len()),
            })],
            span: Span::new(0, name.len() + 1),
        })]))
        .expect("IR");
        let symbol = format!("echo_php_{name}");

        assert!(
            ir.contains(&format!("declare %EchoValue @{symbol}()")),
            "{ir}"
        );
        assert!(ir.contains(&format!("call %EchoValue @{symbol}()")), "{ir}");
    }

    let putenv_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "putenv".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "APP_ENV=local".to_string(),
                span: Span::new(7, 22),
            })],
            span: Span::new(0, 23),
        })],
        span: Span::new(0, 24),
    })]))
    .expect("IR");

    assert!(
        putenv_ir.contains("declare %EchoValue @echo_php_putenv(%EchoValue)"),
        "{putenv_ir}"
    );
    assert!(
        putenv_ir.contains("call %EchoValue @echo_php_putenv(%EchoValue %runtime_call_0)"),
        "{putenv_ir}"
    );
}
