use super::*;

#[test]
fn userland_function_declaration_registers_reflection_metadata() {
    let ir = compile_to_ir(&program(vec![
        std_import("reflect"),
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "greet".to_string(),
            attributes: Vec::new(),
            params: vec![TypedParam {
                name: "name".to_string(),
                attributes: Vec::new(),
                ty: Some("string".to_string()),
                default_value: None,
                promoted_visibility: None,
            }],
            return_type: Some("string".to_string()),
            is_intrinsic: false,
            is_generator: false,
            body: vec![Stmt::Return(ReturnStmt {
                value: Some(Expr::String(StringLiteral {
                    value: "hello\n".to_string(),
                    span: Span::new(0, 8),
                })),
                span: Span::new(0, 15),
            })],
            span: Span::new(0, 40),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "reflect.params".to_string(),
                args: echo_ast::call_args![Expr::String(StringLiteral {
                    value: "greet".to_string(),
                    span: Span::new(50, 57),
                })],
                span: Span::new(35, 58),
            })],
            span: Span::new(35, 59),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains(
            "declare void @echo_reflection_register_function(ptr, i64, ptr, i64, ptr, i64, i32)"
        ),
        "{ir}"
    );
    assert!(ir.contains("c\"greet\""), "{ir}");
    assert!(ir.contains("c\"string $name\""), "{ir}");
    assert!(
        ir.contains("call void @echo_reflection_register_function("),
        "{ir}"
    );
}
