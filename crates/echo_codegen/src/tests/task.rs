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
                        args: vec![Expr::Number(echo_ast::NumberLiteral {
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
