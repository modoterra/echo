use super::*;

#[test]
fn time_sleep_lowers_to_core_runtime_call() {
    let ir = compile_to_ir(&program(vec![
        std_import("time"),
        Stmt::FunctionCall(FunctionCallStmt {
            name: "time.sleep".to_string(),
            args: vec![Expr::Number(echo_ast::NumberLiteral {
                value: "50".to_string(),
                span: Span::new(11, 13),
            })],
            span: Span::new(0, 14),
        }),
    ]))
    .expect("IR");

    assert!(ir.contains("declare void @echo_time_sleep(i64)"), "{ir}");
    assert!(ir.contains("call void @echo_time_sleep(i64 50)"), "{ir}");
}

#[test]
fn net_listen_lowers_to_std_intrinsic_call() {
    let ir = compile_to_ir(&program(vec![
        std_import("net"),
        Stmt::Assign(AssignStmt {
            name: "server".to_string(),
            value: Expr::FunctionCall(FunctionCallExpr {
                name: "net.listen".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "127.0.0.1:39183".to_string(),
                    span: Span::new(11, 30),
                })],
                span: Span::new(0, 31),
            }),
            span: Span::new(0, 31),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_std_net_listen(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_std_net_listen(%EchoValue %runtime_call_0)"),
        "{ir}"
    );
}

#[test]
fn net_read_lowers_numeric_length_as_int_value() {
    let ir = compile_to_ir(&program(vec![
        std_import("net"),
        Stmt::Assign(AssignStmt {
            name: "connection".to_string(),
            value: Expr::FunctionCall(FunctionCallExpr {
                name: "net.connect".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "127.0.0.1:39183".to_string(),
                    span: Span::new(0, 17),
                })],
                span: Span::new(0, 18),
            }),
            span: Span::new(0, 18),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "net.read".to_string(),
                args: vec![
                    Expr::Variable(echo_ast::VariableExpr {
                        name: "connection".to_string(),
                        span: Span::new(19, 30),
                    }),
                    Expr::Number(echo_ast::NumberLiteral {
                        value: "4".to_string(),
                        span: Span::new(31, 32),
                    }),
                ],
                span: Span::new(19, 33),
            })],
            span: Span::new(19, 33),
        }),
    ]))
    .expect("IR");

    assert!(
            ir.contains(
                "call %EchoValue @echo_std_net_read(%EchoValue %runtime_call_1, %EchoValue { i32 2, i64 4 })"
            ),
            "{ir}"
        );
}

#[test]
fn http_response_text_lowers_to_std_intrinsic_call() {
    let ir = compile_to_ir(&program(vec![
        std_import("http"),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "http.responseText".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "hello".to_string(),
                    span: Span::new(18, 25),
                })],
                span: Span::new(0, 26),
            })],
            span: Span::new(0, 26),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_std_http_response_text(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_std_http_response_text(%EchoValue %runtime_call_0)"),
        "{ir}"
    );
}

#[test]
fn reflect_lowers_to_std_intrinsic_call() {
    let ir = compile_to_ir(&program(vec![
        std_import("reflect"),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "reflect.params".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "strlen".to_string(),
                    span: Span::new(18, 26),
                })],
                span: Span::new(0, 27),
            })],
            span: Span::new(0, 27),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "reflect.returnType".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "strlen".to_string(),
                    span: Span::new(46, 54),
                })],
                span: Span::new(28, 55),
            })],
            span: Span::new(28, 55),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "reflect.typeOf".to_string(),
                args: vec![Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(72, 73),
                })],
                span: Span::new(56, 74),
            })],
            span: Span::new(56, 74),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_std_reflect_params(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_std_reflect_return_type(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_std_reflect_type_of(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_std_reflect_params(%EchoValue %runtime_call_0)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_std_reflect_return_type(%EchoValue %runtime_call_2)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_std_reflect_type_of(%EchoValue { i32 2, i64 1 })"),
        "{ir}"
    );
}
