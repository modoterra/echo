use super::*;

#[test]
fn task_sleep_lowers_to_timer_continuation() {
    let ir = compile_to_ir(&program(vec![
        std_import("time"),
        Stmt::Assign(AssignStmt {
            name: "task".to_string(),
            value: Expr::Run(echo_ast::RunExpr::Block {
                body: vec![
                    Stmt::FunctionCall(FunctionCallStmt {
                        name: "time.sleep".to_string(),
                        args: echo_ast::call_args![Expr::Number(echo_ast::NumberLiteral {
                            value: "50".to_string(),
                            span: Span::new(0, 2),
                        })],
                        span: Span::new(0, 14),
                    }),
                    Stmt::Echo(EchoStmt {
                        exprs: vec![Expr::String(StringLiteral {
                            value: "done\n".to_string(),
                            span: Span::new(15, 22),
                        })],
                        span: Span::new(15, 28),
                    }),
                ],
                span: Span::new(0, 28),
            }),
            span: Span::new(0, 28),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_task_sleep_current(i64, ptr)"),
        "{ir}"
    );
    assert!(
        ir.contains("define %EchoValue @echo_defer_0_cont()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_task_sleep_current(i64 50, ptr @echo_defer_0_cont)"),
        "{ir}"
    );
}

#[test]
fn expression_statement_evaluates_and_discards_value() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Assign(AssignStmt {
            name: "task".to_string(),
            value: Expr::Defer(DeferExpr {
                body: vec![],
                span: Span::new(0, 10),
            }),
            span: Span::new(0, 10),
        }),
        Stmt::Expr(echo_ast::ExprStmt {
            expr: Expr::Join(echo_ast::JoinExpr {
                handle: Box::new(Expr::Variable(echo_ast::VariableExpr {
                    name: "task".to_string(),
                    span: Span::new(11, 16),
                })),
                span: Span::new(11, 16),
            }),
            span: Span::new(11, 16),
        }),
    ]))
    .expect("IR");

    assert!(ir.contains("call %EchoValue @echo_join"), "{ir}");
    assert!(!ir.contains("call void @echo_write_value"), "{ir}");
}

#[test]
fn defer_lowers_to_runtime_task_handle() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Assign(AssignStmt {
            name: "deferred".to_string(),
            value: Expr::Defer(DeferExpr {
                body: vec![],
                span: Span::new(0, 10),
            }),
            span: Span::new(0, 10),
        }),
        Stmt::Assign(AssignStmt {
            name: "task".to_string(),
            value: Expr::Run(echo_ast::RunExpr::Task {
                expr: Box::new(Expr::Variable(echo_ast::VariableExpr {
                    name: "deferred".to_string(),
                    span: Span::new(11, 20),
                })),
                span: Span::new(11, 20),
            }),
            span: Span::new(11, 20),
        }),
        Stmt::Assign(AssignStmt {
            name: "value".to_string(),
            value: Expr::Join(echo_ast::JoinExpr {
                handle: Box::new(Expr::Variable(echo_ast::VariableExpr {
                    name: "task".to_string(),
                    span: Span::new(21, 30),
                })),
                span: Span::new(21, 30),
            }),
            span: Span::new(21, 30),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_task_defer(ptr)"),
        "{ir}"
    );
    assert!(ir.contains("define %EchoValue @echo_defer_0()"), "{ir}");
    assert!(
        ir.contains("call %EchoValue @echo_task_defer(ptr @echo_defer_0)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_task_run(%EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_task_run"), "{ir}");
    assert!(
        ir.contains("declare %EchoValue @echo_join(%EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_join"), "{ir}");
    assert!(ir.contains("ret i32 0"), "{ir}");
}
