use super::*;

#[test]
fn ob_start_null_uses_named_echo_value_abi() {
    let ir = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
        name: "ob_start".to_string(),
        args: echo_ast::call_args![Expr::Null(NullLiteral {
            span: Span::new(0, 4),
        })],
        span: Span::new(0, 15),
    })]))
    .expect("IR");

    assert!(ir.contains("%EchoValue = type { i32, i64 }"));
    assert!(ir.contains("declare i1 @echo_php_ob_start_value(%EchoValue)"));
    assert!(
        ir.contains("call i1 @echo_php_ob_start_value(%EchoValue { i32 0, i64 0 })"),
        "{ir}"
    );
}

#[test]
fn ob_start_string_constructs_echo_value_callback() {
    let ir = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
        name: "ob_start".to_string(),
        args: echo_ast::call_args![Expr::String(StringLiteral {
            value: "filter".to_string(),
            span: Span::new(9, 17),
        })],
        span: Span::new(0, 19),
    })]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_value_string(ptr, i64)"));
    assert!(ir.contains("declare void @echo_write_value(%EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_"));
    assert!(ir.contains("call i1 @echo_php_ob_start_value(%EchoValue %runtime_call_"));
}

#[test]
fn strlen_lowers_to_php_builtin_with_echo_value_argument() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "strlen".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "hello".to_string(),
                span: Span::new(7, 14),
            })],
            span: Span::new(0, 15),
        })],
        span: Span::new(0, 16),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_strlen(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_strlen(%EchoValue %runtime_call_0)"),
        "{ir}"
    );
    assert!(
        ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
        "{ir}"
    );
}

#[test]
fn pi_lowers_to_php_builtin_with_no_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "pi".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(5, 9),
        })],
        span: Span::new(0, 10),
    })]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_php_pi()"), "{ir}");
    assert!(ir.contains("call %EchoValue @echo_php_pi()"), "{ir}");
    assert!(
        ir.contains("call void @echo_write_value(%EchoValue %runtime_call_0)"),
        "{ir}"
    );
}

#[test]
fn logarithm_builtins_lower_optional_base_to_e() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "log".to_string(),
            args: echo_ast::call_args![Expr::Number(NumberLiteral {
                value: "8".to_string(),
                span: Span::new(4, 5),
            })],
            span: Span::new(0, 6),
        })],
        span: Span::new(0, 7),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_log(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
            ir.contains("call %EchoValue @echo_php_log(%EchoValue { i32 2, i64 8 }, %EchoValue { i32 11, i64 4613303445314885481 })"),
            "{ir}"
        );
}

#[test]
fn digest_builtins_lower_optional_binary_flag_to_false() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "md5".to_string(),
                args: echo_ast::call_args![Expr::String(StringLiteral {
                    value: "Echo".to_string(),
                    span: Span::new(4, 10),
                })],
                span: Span::new(0, 11),
            })],
            span: Span::new(0, 12),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "sha1".to_string(),
                args: echo_ast::call_args![
                    Expr::String(StringLiteral {
                        value: "Echo".to_string(),
                        span: Span::new(17, 23),
                    }),
                    Expr::Bool(BoolLiteral {
                        value: true,
                        span: Span::new(25, 29),
                    }),
                ],
                span: Span::new(12, 30),
            })],
            span: Span::new(12, 31),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_md5(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_php_sha1(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %EchoValue @echo_php_md5(%EchoValue %runtime_call_0, %EchoValue { i32 1, i64 0 })"
        ),
        "{ir}"
    );
    assert!(
            ir.contains(
                "call %EchoValue @echo_php_sha1(%EchoValue %runtime_call_2, %EchoValue { i32 1, i64 1 })"
            ),
            "{ir}"
        );
}

#[test]
fn password_and_crypt_constants_lower_to_expected_php_compat_shapes() {
    let string_constants = [
        ("PASSWORD_DEFAULT", "2y"),
        ("PASSWORD_BCRYPT", "2y"),
        ("PASSWORD_ARGON2I", "argon2i"),
        ("PASSWORD_ARGON2ID", "argon2id"),
    ];

    for (constant_name, expected_payload) in string_constants {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::Constant(echo_ast::ConstantExpr {
                name: constant_name.to_string(),
                span: Span::new(0, 1),
            })],
            span: Span::new(0, 1),
        })]))
        .expect("crypto constants should lower to LLVM constants");

        assert!(
            ir.contains(&format!("c\"{expected_payload}\", ")),
            "{constant_name} should be materialized as a string literal"
        );
    }

    let int_constants = [
        ("HASH_HMAC", "{ i32 2, i64 1 }"),
        ("CRYPT_BLOWFISH", "{ i32 2, i64 1 }"),
        ("CRYPT_STD_DES", "{ i32 2, i64 1 }"),
        ("CRYPT_EXT_DES", "{ i32 2, i64 1 }"),
        ("CRYPT_MD5", "{ i32 2, i64 1 }"),
        ("CRYPT_SHA256", "{ i32 2, i64 1 }"),
        ("CRYPT_SHA512", "{ i32 2, i64 1 }"),
    ];

    for (constant_name, expected_payload) in int_constants {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::Constant(echo_ast::ConstantExpr {
                name: constant_name.to_string(),
                span: Span::new(0, 1),
            })],
            span: Span::new(0, 1),
        })]))
        .expect("crypto constants should lower to LLVM constants");

        assert!(
            ir.contains(expected_payload),
            "{constant_name} should include expected integer payload"
        );
    }
}
