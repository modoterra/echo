use super::*;

#[test]
fn loop_statement_lowers_to_control_flow_labels() {
    let ir = compile_to_ir(&program(vec![Stmt::Loop(LoopStmt {
        body: vec![Stmt::If(IfStmt {
            condition: Expr::Bool(BoolLiteral {
                value: true,
                span: Span::new(5, 9),
            }),
            body: vec![Stmt::Break(BreakStmt {
                value: None,
                span: Span::new(12, 17),
            })],
            elseif_clauses: Vec::new(),
            else_body: Vec::new(),
            span: Span::new(2, 19),
        })],
        span: Span::new(0, 20),
    })]))
    .expect("loop statement should compile to LLVM IR");

    assert!(ir.contains("br label %loop_0"));
    assert!(ir.contains("loop_0:"));
    assert!(ir.contains("if_then_0:"));
    assert!(ir.contains("br label %loop_after_0"));
    assert!(ir.contains("loop_after_0:"));
}

#[test]
fn loop_expression_break_value_lowers_to_result_slot() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Loop(LoopExpr {
            body: vec![Stmt::Break(BreakStmt {
                value: Some(Expr::String(StringLiteral {
                    value: "done".to_string(),
                    span: Span::new(12, 18),
                })),
                span: Span::new(6, 19),
            })],
            span: Span::new(0, 20),
        })],
        span: Span::new(0, 21),
    })]))
    .expect("loop expression should compile to LLVM IR");

    assert!(ir.contains("%loop_result_0 = alloca %EchoValue"));
    assert!(ir.contains("store %EchoValue %runtime_call_0, ptr %loop_result_0"));
    assert!(ir.contains("loop_expr_after_0:"));
    assert!(ir.contains("%loop_value_0 = load %EchoValue, ptr %loop_result_0"));
}

#[test]
fn is_null_condition_lowers_to_kind_check() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Let(LetStmt {
            name: "value".to_string(),
            ty: None,
            value: Expr::Null(NullLiteral {
                span: Span::new(12, 16),
            }),
            span: Span::new(0, 16),
        }),
        Stmt::If(IfStmt {
            condition: Expr::Binary(Box::new(BinaryExpr {
                left: Expr::Variable(VariableExpr {
                    name: "value".to_string(),
                    span: Span::new(20, 26),
                }),
                op: BinaryOp::Is,
                right: Expr::Null(NullLiteral {
                    span: Span::new(30, 34),
                }),
                span: Span::new(20, 34),
            })),
            body: vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::String(StringLiteral {
                    value: "null".to_string(),
                    span: Span::new(38, 44),
                })],
                span: Span::new(35, 45),
            })],
            elseif_clauses: Vec::new(),
            else_body: Vec::new(),
            span: Span::new(17, 46),
        }),
    ]))
    .expect("null condition should compile to LLVM IR");

    assert!(ir.contains("%value_kind_0 = extractvalue %EchoValue { i32 0, i64 0 }, 0"));
    assert!(ir.contains("%is_null_0 = icmp eq i32 %value_kind_0, 0"));
    assert!(ir.contains("br i1 %is_null_0, label %if_then_0, label %if_after_0"));
    assert!(!ir.contains("call i1 @echo_value_bool(%EchoValue { i32 0, i64 0 })"));
}

#[test]
fn false_static_condition_prunes_unreachable_branch() {
    let ir = compile_to_ir(&program(vec![
        Stmt::If(IfStmt {
            condition: Expr::Binary(Box::new(BinaryExpr {
                left: Expr::Constant(echo_ast::ConstantExpr {
                    name: "PHP_VERSION_ID".to_string(),
                    span: Span::new(4, 18),
                }),
                op: BinaryOp::LessThan,
                right: Expr::Number(NumberLiteral {
                    value: "50600".to_string(),
                    span: Span::new(21, 26),
                }),
                span: Span::new(4, 26),
            })),
            body: vec![Stmt::Expr(ExprStmt {
                expr: Expr::MethodCall(Box::new(MethodCallExpr {
                    object: Expr::Object(ObjectExpr {
                        name: String::new(),
                        fields: Vec::new(),
                        span: Span::new(31, 35),
                    }),
                    method: "unsupported".to_string(),
                    method_span: Span::new(37, 48),
                    args: echo_ast::call_args![],
                    span: Span::new(31, 50),
                })),
                span: Span::new(31, 51),
            })],
            elseif_clauses: Vec::new(),
            else_body: Vec::new(),
            span: Span::new(0, 53),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::String(StringLiteral {
                value: "loaded".to_string(),
                span: Span::new(59, 67),
            })],
            span: Span::new(54, 68),
        }),
    ]))
    .expect("false static branch should not lower unreachable unsupported statements");

    assert!(!ir.contains("if_then_"));
    assert!(ir.contains("loaded"));
}
