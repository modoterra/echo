use super::*;

#[test]
fn userland_call_emits_function_definition_and_call() {
    let ir = compile_to_ir(&program(vec![
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "say_after".to_string(),
            params: vec![],
            return_type: None,
            is_intrinsic: false,
            is_generator: false,
            body: vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::String(StringLiteral {
                    value: "after\n".to_string(),
                    span: Span::new(0, 8),
                })],
                span: Span::new(0, 15),
            })],
            span: Span::new(0, 40),
        }),
        Stmt::FunctionCall(FunctionCallStmt {
            name: "say_after".to_string(),
            args: vec![],
            span: Span::new(41, 53),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("define %EchoValue @echo_user_say_after()"),
        "{ir}"
    );
    assert!(
        ir.contains("call void @echo_write(ptr @echo_str_0, i64 6)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_user_say_after()"),
        "{ir}"
    );
}

#[test]
fn userland_call_passes_string_argument_as_echo_value() {
    let ir = compile_to_ir(&program(vec![
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "say".to_string(),
            params: vec![param("message")],
            return_type: None,
            is_intrinsic: false,
            is_generator: false,
            body: vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::Variable(echo_ast::VariableExpr {
                    name: "message".to_string(),
                    span: Span::new(0, 8),
                })],
                span: Span::new(0, 15),
            })],
            span: Span::new(0, 40),
        }),
        Stmt::FunctionCall(FunctionCallStmt {
            name: "say".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "hello\n".to_string(),
                span: Span::new(45, 53),
            })],
            span: Span::new(41, 55),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("define %EchoValue @echo_user_say(%EchoValue %arg_message)"),
        "{ir}"
    );
    assert!(
        ir.contains("call void @echo_write_value(%EchoValue %arg_message)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_user_say(%EchoValue %runtime_call_"),
        "{ir}"
    );
}

#[test]
fn userland_return_value_can_be_echoed() {
    let ir = compile_to_ir(&program(vec![
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "greeting".to_string(),
            params: vec![],
            return_type: None,
            is_intrinsic: false,
            is_generator: false,
            body: vec![Stmt::Return(ReturnStmt {
                value: Expr::String(StringLiteral {
                    value: "hello\n".to_string(),
                    span: Span::new(0, 8),
                }),
                span: Span::new(0, 16),
            })],
            span: Span::new(0, 40),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "greeting".to_string(),
                args: vec![],
                span: Span::new(45, 55),
            })],
            span: Span::new(41, 56),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("define %EchoValue @echo_user_greeting()"),
        "{ir}"
    );
    assert!(ir.contains("ret %EchoValue %runtime_call_0"), "{ir}");
    assert!(ir.contains("call %EchoValue @echo_user_greeting()"), "{ir}");
    assert!(
        ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
        "{ir}"
    );
}
