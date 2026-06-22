use super::*;

#[test]
fn filesystem_content_builtins_lower_optional_arguments() {
    let file_get_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "file_get_contents".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "report.txt".to_string(),
                span: Span::new(18, 30),
            })],
            span: Span::new(0, 31),
        })],
        span: Span::new(0, 32),
    })]))
    .expect("IR");

    assert!(
            file_get_ir.contains("declare %EchoValue @echo_php_file_get_contents(%EchoValue, %EchoValue, %EchoValue, %EchoValue, %EchoValue)"),
            "{file_get_ir}"
        );
    assert!(
            file_get_ir.contains("call %EchoValue @echo_php_file_get_contents(%EchoValue %runtime_call_0, %EchoValue { i32 1, i64 0 }, %EchoValue { i32 0, i64 0 }, %EchoValue { i32 2, i64 0 }, %EchoValue { i32 0, i64 0 })"),
            "{file_get_ir}"
        );

    let file_put_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "file_put_contents".to_string(),
            args: vec![
                Expr::String(StringLiteral {
                    value: "report.txt".to_string(),
                    span: Span::new(18, 30),
                }),
                Expr::String(StringLiteral {
                    value: "ready".to_string(),
                    span: Span::new(32, 39),
                }),
            ],
            span: Span::new(0, 40),
        })],
        span: Span::new(0, 41),
    })]))
    .expect("IR");

    assert!(
            file_put_ir.contains("declare %EchoValue @echo_php_file_put_contents(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"),
            "{file_put_ir}"
        );
    assert!(
            file_put_ir.contains("call %EchoValue @echo_php_file_put_contents(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1, %EchoValue { i32 2, i64 0 }, %EchoValue { i32 0, i64 0 })"),
            "{file_put_ir}"
        );

    let readfile_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "readfile".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "report.txt".to_string(),
                span: Span::new(9, 21),
            })],
            span: Span::new(0, 22),
        })],
        span: Span::new(0, 23),
    })]))
    .expect("IR");

    assert!(
        readfile_ir
            .contains("declare %EchoValue @echo_php_readfile(%EchoValue, %EchoValue, %EchoValue)"),
        "{readfile_ir}"
    );
    assert!(
            readfile_ir.contains("call %EchoValue @echo_php_readfile(%EchoValue %runtime_call_0, %EchoValue { i32 1, i64 0 }, %EchoValue { i32 0, i64 0 })"),
            "{readfile_ir}"
        );
}

#[test]
fn getcwd_lowers_to_php_builtin_with_no_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "getcwd".to_string(),
            args: vec![],
            span: Span::new(5, 13),
        })],
        span: Span::new(0, 14),
    })]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_php_getcwd()"), "{ir}");
    assert!(ir.contains("call %EchoValue @echo_php_getcwd()"), "{ir}");
    assert!(
        ir.contains("call void @echo_write_value(%EchoValue %runtime_call_0)"),
        "{ir}"
    );
}

#[test]
fn temporary_name_builtins_lower_to_php_runtime_calls() {
    let sys_temp_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "sys_get_temp_dir".to_string(),
            args: vec![],
            span: Span::new(0, 18),
        })],
        span: Span::new(0, 19),
    })]))
    .expect("IR");

    assert!(
        sys_temp_ir.contains("declare %EchoValue @echo_php_sys_get_temp_dir()"),
        "{sys_temp_ir}"
    );
    assert!(
        sys_temp_ir.contains("call %EchoValue @echo_php_sys_get_temp_dir()"),
        "{sys_temp_ir}"
    );

    let tempnam_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "tempnam".to_string(),
            args: vec![
                Expr::String(StringLiteral {
                    value: "/tmp".to_string(),
                    span: Span::new(8, 14),
                }),
                Expr::String(StringLiteral {
                    value: "exo".to_string(),
                    span: Span::new(16, 21),
                }),
            ],
            span: Span::new(0, 22),
        })],
        span: Span::new(0, 23),
    })]))
    .expect("IR");

    assert!(
        tempnam_ir.contains("declare %EchoValue @echo_php_tempnam(%EchoValue, %EchoValue)"),
        "{tempnam_ir}"
    );
    assert!(
            tempnam_ir.contains(
                "call %EchoValue @echo_php_tempnam(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1)"
            ),
            "{tempnam_ir}"
        );

    let uniqid_ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "uniqid".to_string(),
                args: vec![],
                span: Span::new(0, 8),
            })],
            span: Span::new(0, 9),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "uniqid".to_string(),
                args: vec![
                    Expr::String(StringLiteral {
                        value: "job_".to_string(),
                        span: Span::new(17, 23),
                    }),
                    Expr::Bool(BoolLiteral {
                        value: true,
                        span: Span::new(25, 29),
                    }),
                ],
                span: Span::new(10, 30),
            })],
            span: Span::new(10, 31),
        }),
    ]))
    .expect("IR");

    assert!(
        uniqid_ir.contains("declare %EchoValue @echo_php_uniqid(%EchoValue, %EchoValue)"),
        "{uniqid_ir}"
    );
    assert!(
            uniqid_ir.contains(
                "call %EchoValue @echo_php_uniqid(%EchoValue %runtime_call_0, %EchoValue { i32 1, i64 0 })"
            ),
            "{uniqid_ir}"
        );
    assert!(
            uniqid_ir.contains(
                "call %EchoValue @echo_php_uniqid(%EchoValue %runtime_call_2, %EchoValue { i32 1, i64 1 })"
            ),
            "{uniqid_ir}"
        );
}

#[test]
fn filesystem_mutation_builtins_lower_optional_arguments() {
    let touch_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "touch".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "cache.marker".to_string(),
                span: Span::new(6, 20),
            })],
            span: Span::new(0, 21),
        })],
        span: Span::new(0, 22),
    })]))
    .expect("IR");

    assert!(
        touch_ir.contains("declare %EchoValue @echo_php_touch(%EchoValue, %EchoValue, %EchoValue)"),
        "{touch_ir}"
    );
    assert!(
            touch_ir.contains("call %EchoValue @echo_php_touch(%EchoValue %runtime_call_0, %EchoValue { i32 0, i64 0 }, %EchoValue { i32 0, i64 0 })"),
            "{touch_ir}"
        );

    let copy_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "copy".to_string(),
            args: vec![
                Expr::String(StringLiteral {
                    value: "import.csv".to_string(),
                    span: Span::new(5, 17),
                }),
                Expr::String(StringLiteral {
                    value: "import.csv.bak".to_string(),
                    span: Span::new(19, 35),
                }),
            ],
            span: Span::new(0, 36),
        })],
        span: Span::new(0, 37),
    })]))
    .expect("IR");

    assert!(
        copy_ir.contains("declare %EchoValue @echo_php_copy(%EchoValue, %EchoValue, %EchoValue)"),
        "{copy_ir}"
    );
    assert!(
            copy_ir.contains("call %EchoValue @echo_php_copy(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1, %EchoValue { i32 0, i64 0 })"),
            "{copy_ir}"
        );

    let unlink_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "unlink".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "import.csv.bak".to_string(),
                span: Span::new(7, 23),
            })],
            span: Span::new(0, 24),
        })],
        span: Span::new(0, 25),
    })]))
    .expect("IR");

    assert!(
        unlink_ir.contains("declare %EchoValue @echo_php_unlink(%EchoValue, %EchoValue)"),
        "{unlink_ir}"
    );
    assert!(
            unlink_ir.contains("call %EchoValue @echo_php_unlink(%EchoValue %runtime_call_0, %EchoValue { i32 0, i64 0 })"),
            "{unlink_ir}"
        );

    let mkdir_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "mkdir".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "storage/cache".to_string(),
                span: Span::new(6, 21),
            })],
            span: Span::new(0, 22),
        })],
        span: Span::new(0, 23),
    })]))
    .expect("IR");

    assert!(
        mkdir_ir.contains(
            "declare %EchoValue @echo_php_mkdir(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
        ),
        "{mkdir_ir}"
    );
    assert!(
            mkdir_ir.contains("call %EchoValue @echo_php_mkdir(%EchoValue %runtime_call_0, %EchoValue { i32 2, i64 511 }, %EchoValue { i32 1, i64 0 }, %EchoValue { i32 0, i64 0 })"),
            "{mkdir_ir}"
        );

    let rmdir_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "rmdir".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "storage/cache".to_string(),
                span: Span::new(6, 21),
            })],
            span: Span::new(0, 22),
        })],
        span: Span::new(0, 23),
    })]))
    .expect("IR");

    assert!(
        rmdir_ir.contains("declare %EchoValue @echo_php_rmdir(%EchoValue, %EchoValue)"),
        "{rmdir_ir}"
    );
    assert!(
            rmdir_ir.contains("call %EchoValue @echo_php_rmdir(%EchoValue %runtime_call_0, %EchoValue { i32 0, i64 0 })"),
            "{rmdir_ir}"
        );
}

#[test]
fn basename_lowers_optional_suffix_to_empty_string() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "basename".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "/etc/passwd".to_string(),
                span: Span::new(9, 22),
            })],
            span: Span::new(0, 23),
        })],
        span: Span::new(0, 24),
    })]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_php_basename(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_php_basename("));
    assert!(
        ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_") && ir.contains(", i64 0)")
    );
}

#[test]
fn dirname_lowers_optional_levels_to_one() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "dirname".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "/etc/passwd".to_string(),
                span: Span::new(8, 21),
            })],
            span: Span::new(0, 22),
        })],
        span: Span::new(0, 23),
    })]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_php_dirname(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_php_dirname("));
    assert!(ir.contains("%EchoValue { i32 2, i64 1 }"));
}
