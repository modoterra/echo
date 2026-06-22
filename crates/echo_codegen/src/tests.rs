use super::*;
use echo_ast::{
    ArrayElement, ArrayExpr, AssignStmt, BinaryExpr, BinaryOp, BoolLiteral, BreakStmt, DeferExpr,
    EchoStmt, Expr, ExprStmt, FunctionCallExpr, FunctionCallStmt, FunctionDeclStmt, IfStmt,
    ImportStmt, LetStmt, ListExpr, LoopExpr, LoopStmt, MethodCallExpr, NullLiteral, NumberLiteral,
    ObjectExpr, ObjectField, QualifiedName, ReturnStmt, StringLiteral, TypeAscriptionExpr,
    TypedParam, VariableExpr,
};

fn program(statements: Vec<Stmt>) -> Program {
    Program {
        open_tag: None,
        statements,
        source_dir: None,
        span: Span::new(0, 0),
    }
}

fn param(name: &str) -> TypedParam {
    TypedParam {
        name: name.to_string(),
        ty: None,
    }
}

fn std_import(module: &str) -> Stmt {
    Stmt::Import(ImportStmt {
        source: ImportSource::Std,
        name: QualifiedName::new(vec![module.to_string()]),
        alias: None,
        span: Span::new(0, 0),
    })
}

fn std_import_alias(module: &str, alias: &str) -> Stmt {
    Stmt::Import(ImportStmt {
        source: ImportSource::Std,
        name: QualifiedName::new(vec![module.to_string()]),
        alias: Some(alias.to_string()),
        span: Span::new(0, 0),
    })
}

mod collections;
mod control_flow;
mod expressions;
mod reflection;
mod stdlib;
mod task;
mod userland;

#[test]
fn ast_hir_and_mir_entrypoints_emit_same_ir() {
    let program = program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::String(StringLiteral {
            value: "hello".to_string(),
            span: Span::new(5, 12),
        })],
        span: Span::new(0, 13),
    })]);
    let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
    let mir = echo_mir::lower_program(&hir).expect("HIR should lower to MIR");

    let ast_ir = compile_to_ir(&program).expect("AST entrypoint should compile");
    let hir_ir = compile_hir_to_ir(&hir).expect("HIR entrypoint should compile");
    let mir_ir = compile_mir_to_ir(&mir).expect("MIR entrypoint should compile");

    assert_eq!(ast_ir, hir_ir);
    assert_eq!(hir_ir, mir_ir);
    assert!(mir_ir.contains("call void @echo_write("));
}

#[test]
fn ast_to_hir_to_mir_to_llvm_ir_executes_with_jit() {
    let program = program(vec![]);
    let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
    let mir = echo_mir::lower_program(&hir).expect("HIR should lower to MIR");

    let status = run_mir_jit(&mir).expect("MIR should execute through LLVM JIT");

    assert_eq!(status, 0);
}

#[test]
fn validates_known_std_import() {
    compile_to_ir(&program(vec![Stmt::Import(ImportStmt {
        source: ImportSource::Std,
        name: QualifiedName::new(vec!["net".to_string()]),
        alias: None,
        span: Span::new(0, 16),
    })]))
    .expect("known std import should compile");
}

#[test]
fn rejects_unknown_std_import() {
    let diagnostics = compile_to_ir(&program(vec![Stmt::Import(ImportStmt {
        source: ImportSource::Std,
        name: QualifiedName::new(vec!["potato".to_string()]),
        alias: None,
        span: Span::new(0, 19),
    })]))
    .expect_err("unknown std import should fail");

    assert_eq!(diagnostics[0].message, "unknown std import `potato`");
    assert_eq!(diagnostics[0].span, Span::new(0, 19));
}

#[test]
fn rejects_unimported_std_module_call() {
    let diagnostics = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
        name: "net.listen".to_string(),
        args: vec![Expr::String(StringLiteral {
            value: "127.0.0.1:39183".to_string(),
            span: Span::new(11, 30),
        })],
        span: Span::new(0, 31),
    })]))
    .expect_err("unimported std module should fail");

    assert_eq!(
        diagnostics[0].message,
        "std module `net` must be imported before use"
    );
}

#[test]
fn aliases_std_module_calls() {
    let ir = compile_to_ir(&program(vec![
        std_import_alias("net", "socket"),
        Stmt::FunctionCall(FunctionCallStmt {
            name: "socket.close".to_string(),
            args: vec![Expr::Null(NullLiteral {
                span: Span::new(13, 17),
            })],
            span: Span::new(0, 18),
        }),
    ]))
    .expect("aliased std call compiles");

    assert!(ir.contains("call %EchoValue @echo_std_net_close"), "{ir}");
}

#[test]
fn ob_start_null_uses_named_echo_value_abi() {
    let ir = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
        name: "ob_start".to_string(),
        args: vec![Expr::Null(NullLiteral {
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
        args: vec![Expr::String(StringLiteral {
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
            args: vec![Expr::String(StringLiteral {
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
            args: vec![],
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
            args: vec![Expr::Number(NumberLiteral {
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
                args: vec![Expr::String(StringLiteral {
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
                args: vec![
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
fn environment_process_builtins_lower_to_php_runtime_calls() {
    let getenv_ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "getenv".to_string(),
                args: vec![],
                span: Span::new(0, 8),
            })],
            span: Span::new(0, 9),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "getenv".to_string(),
                args: vec![Expr::String(StringLiteral {
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
                args: vec![
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
                args: vec![],
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
            args: vec![Expr::String(StringLiteral {
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
fn string_case_builtins_lower_to_php_builtin_with_echo_value_argument() {
    for (php_name, symbol) in [
        ("strval", "echo_php_strval"),
        ("boolval", "echo_php_boolval"),
        ("intval", "echo_php_intval"),
        ("floatval", "echo_php_floatval"),
        ("doubleval", "echo_php_floatval"),
        ("strtoupper", "echo_php_strtoupper"),
        ("strtolower", "echo_php_strtolower"),
        ("ucwords", "echo_php_ucwords"),
        ("strrev", "echo_php_strrev"),
        ("ucfirst", "echo_php_ucfirst"),
        ("lcfirst", "echo_php_lcfirst"),
        ("ord", "echo_php_ord"),
        ("str_rot13", "echo_php_str_rot13"),
        ("chr", "echo_php_chr"),
        ("decbin", "echo_php_decbin"),
        ("dechex", "echo_php_dechex"),
        ("decoct", "echo_php_decoct"),
        ("crc32", "echo_php_crc32"),
        ("bindec", "echo_php_bindec"),
        ("hexdec", "echo_php_hexdec"),
        ("octdec", "echo_php_octdec"),
        ("deg2rad", "echo_php_deg2rad"),
        ("rad2deg", "echo_php_rad2deg"),
        ("sin", "echo_php_sin"),
        ("cos", "echo_php_cos"),
        ("tan", "echo_php_tan"),
        ("asin", "echo_php_asin"),
        ("acos", "echo_php_acos"),
        ("atan", "echo_php_atan"),
        ("sinh", "echo_php_sinh"),
        ("cosh", "echo_php_cosh"),
        ("tanh", "echo_php_tanh"),
        ("asinh", "echo_php_asinh"),
        ("acosh", "echo_php_acosh"),
        ("atanh", "echo_php_atanh"),
        ("ceil", "echo_php_ceil"),
        ("floor", "echo_php_floor"),
        ("sqrt", "echo_php_sqrt"),
        ("bin2hex", "echo_php_bin2hex"),
        ("base64_encode", "echo_php_base64_encode"),
        ("base64_decode", "echo_php_base64_decode"),
        ("rawurlencode", "echo_php_rawurlencode"),
        ("rawurldecode", "echo_php_rawurldecode"),
        ("urlencode", "echo_php_urlencode"),
        ("urldecode", "echo_php_urldecode"),
        ("hex2bin", "echo_php_hex2bin"),
        ("escapeshellarg", "echo_php_escapeshellarg"),
        ("escapeshellcmd", "echo_php_escapeshellcmd"),
        ("file_exists", "echo_php_file_exists"),
        ("chdir", "echo_php_chdir"),
        ("is_dir", "echo_php_is_dir"),
        ("is_file", "echo_php_is_file"),
        ("is_link", "echo_php_is_link"),
        ("is_readable", "echo_php_is_readable"),
        ("is_writable", "echo_php_is_writable"),
        ("is_writeable", "echo_php_is_writable"),
        ("is_executable", "echo_php_is_executable"),
        ("filesize", "echo_php_filesize"),
        ("fileatime", "echo_php_fileatime"),
        ("filectime", "echo_php_filectime"),
        ("filemtime", "echo_php_filemtime"),
        ("fileinode", "echo_php_fileinode"),
        ("fileowner", "echo_php_fileowner"),
        ("filegroup", "echo_php_filegroup"),
        ("fileperms", "echo_php_fileperms"),
        ("filetype", "echo_php_filetype"),
        ("readlink", "echo_php_readlink"),
        ("realpath", "echo_php_realpath"),
        ("trim", "echo_php_trim"),
        ("ltrim", "echo_php_ltrim"),
        ("rtrim", "echo_php_rtrim"),
        ("chop", "echo_php_rtrim"),
        ("addslashes", "echo_php_addslashes"),
        ("stripslashes", "echo_php_stripslashes"),
        (
            "quoted_printable_encode",
            "echo_php_quoted_printable_encode",
        ),
        (
            "quoted_printable_decode",
            "echo_php_quoted_printable_decode",
        ),
        ("quotemeta", "echo_php_quotemeta"),
    ] {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: php_name.to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "Echo".to_string(),
                    span: Span::new(11, 17),
                })],
                span: Span::new(0, 18),
            })],
            span: Span::new(0, 19),
        })]))
        .expect("IR");

        assert!(
            ir.contains(&format!("declare %EchoValue @{symbol}(%EchoValue)")),
            "{ir}"
        );
        assert!(
            ir.contains(&format!(
                "call %EchoValue @{symbol}(%EchoValue %runtime_call_0)"
            )),
            "{ir}"
        );
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
            "{ir}"
        );
    }
}

#[test]
fn nl2br_lowers_optional_xhtml_flag_to_true() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "nl2br".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "line\nnext".to_string(),
                span: Span::new(7, 19),
            })],
            span: Span::new(0, 20),
        })],
        span: Span::new(0, 21),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_nl2br(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
            ir.contains(
                "call %EchoValue @echo_php_nl2br(%EchoValue %runtime_call_0, %EchoValue { i32 1, i64 1 })"
            ),
            "{ir}"
        );
}

#[test]
fn string_predicate_builtins_lower_to_php_builtin_with_two_echo_value_arguments() {
    for (php_name, symbol) in [
        ("str_contains", "echo_php_str_contains"),
        ("str_starts_with", "echo_php_str_starts_with"),
        ("str_ends_with", "echo_php_str_ends_with"),
        ("str_repeat", "echo_php_str_repeat"),
        ("substr", "echo_php_substr"),
        ("strpos", "echo_php_strpos"),
        ("stripos", "echo_php_stripos"),
        ("strrpos", "echo_php_strrpos"),
        ("strripos", "echo_php_strripos"),
        ("strstr", "echo_php_strstr"),
        ("strchr", "echo_php_strstr"),
        ("stristr", "echo_php_stristr"),
        ("strrchr", "echo_php_strrchr"),
        ("strpbrk", "echo_php_strpbrk"),
        ("strspn", "echo_php_strspn"),
        ("strcspn", "echo_php_strcspn"),
        ("substr_count", "echo_php_substr_count"),
        ("strcmp", "echo_php_strcmp"),
        ("strcasecmp", "echo_php_strcasecmp"),
        ("atan2", "echo_php_atan2"),
        ("fdiv", "echo_php_fdiv"),
        ("fpow", "echo_php_fpow"),
        ("hypot", "echo_php_hypot"),
        ("fmod", "echo_php_fmod"),
        ("link", "echo_php_link"),
        ("symlink", "echo_php_symlink"),
    ] {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: php_name.to_string(),
                args: vec![
                    Expr::String(StringLiteral {
                        value: "Echo PHP".to_string(),
                        span: Span::new(13, 23),
                    }),
                    Expr::String(StringLiteral {
                        value: "PHP".to_string(),
                        span: Span::new(25, 30),
                    }),
                ],
                span: Span::new(0, 31),
            })],
            span: Span::new(0, 32),
        })]))
        .expect("IR");

        assert!(
            ir.contains(&format!(
                "declare %EchoValue @{symbol}(%EchoValue, %EchoValue)"
            )),
            "{ir}"
        );
        assert!(
            ir.contains(&format!(
                "call %EchoValue @{symbol}(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1)"
            )),
            "{ir}"
        );
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %runtime_call_2)"),
            "{ir}"
        );
    }
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
fn runtime_declarations_deduplicate_alias_symbols() {
    let declarations = runtime_declarations();

    assert_eq!(declarations.matches("@echo_php_strstr(").count(), 1);
}

#[test]
fn str_pad_lowers_optional_arguments_to_php_defaults() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "str_pad".to_string(),
            args: vec![
                Expr::String(StringLiteral {
                    value: "ID".to_string(),
                    span: Span::new(8, 12),
                }),
                Expr::Number(NumberLiteral {
                    value: "6".to_string(),
                    span: Span::new(14, 15),
                }),
            ],
            span: Span::new(0, 16),
        })],
        span: Span::new(0, 17),
    })]))
    .expect("IR");

    assert!(ir.contains(
        "declare %EchoValue @echo_php_str_pad(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
    ));
    assert!(ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_"));
    assert!(ir.contains(", i64 1)"));
    assert!(ir.contains("call %EchoValue @echo_php_str_pad("), "{ir}");
    assert!(ir.contains("%EchoValue { i32 2, i64 1 }"));
}

#[test]
fn string_chunk_builtins_lower_optional_arguments_to_php_defaults() {
    let split_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "str_split".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "Echo".to_string(),
                span: Span::new(10, 16),
            })],
            span: Span::new(0, 17),
        })],
        span: Span::new(0, 18),
    })]))
    .expect("IR");

    assert!(split_ir.contains("declare %EchoValue @echo_php_str_split(%EchoValue, %EchoValue)"));
    assert!(split_ir.contains(
            "call %EchoValue @echo_php_str_split(%EchoValue %runtime_call_0, %EchoValue { i32 2, i64 1 })"
        ));

    let chunk_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "chunk_split".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "Echo".to_string(),
                span: Span::new(12, 18),
            })],
            span: Span::new(0, 19),
        })],
        span: Span::new(0, 20),
    })]))
    .expect("IR");

    assert!(
        chunk_ir.contains(
            "declare %EchoValue @echo_php_chunk_split(%EchoValue, %EchoValue, %EchoValue)"
        )
    );
    assert!(chunk_ir.contains("%EchoValue { i32 2, i64 76 }"));
    assert!(chunk_ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_"));
    assert!(chunk_ir.contains(", i64 2)"));
    assert!(chunk_ir.contains("call %EchoValue @echo_php_chunk_split("));
}

#[test]
fn every_lowered_php_builtin_has_reflected_declaration() {
    for builtin in PHP_BUILTINS.iter() {
        assert!(
            echo_reflection::php_builtin(builtin.php_name).is_some(),
            "missing reflected declaration for {}",
            builtin.php_name
        );
    }
}

#[test]
fn string_prefix_compare_builtins_lower_to_php_builtin_with_three_echo_value_arguments() {
    for (php_name, symbol) in [
        ("strncmp", "echo_php_strncmp"),
        ("strncasecmp", "echo_php_strncasecmp"),
        ("str_replace", "echo_php_str_replace"),
        ("str_ireplace", "echo_php_str_ireplace"),
        ("strtr", "echo_php_strtr"),
    ] {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: php_name.to_string(),
                args: vec![
                    Expr::String(StringLiteral {
                        value: "Echo PHP".to_string(),
                        span: Span::new(13, 23),
                    }),
                    Expr::String(StringLiteral {
                        value: "PHP".to_string(),
                        span: Span::new(25, 30),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "3".to_string(),
                        span: Span::new(32, 33),
                    }),
                ],
                span: Span::new(0, 34),
            })],
            span: Span::new(0, 35),
        })]))
        .expect("IR");

        assert!(
            ir.contains(&format!(
                "call %EchoValue @{symbol}(%EchoValue %runtime_call_"
            )),
            "IR should call {symbol}: {ir}"
        );
        assert!(ir.contains("declare %EchoValue @"));
        assert!(ir.contains("(%EchoValue, %EchoValue, %EchoValue)"));
    }
}

#[test]
fn explode_lowers_optional_limit_to_php_default() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "explode".to_string(),
            args: vec![
                Expr::String(StringLiteral {
                    value: ",".to_string(),
                    span: Span::new(8, 11),
                }),
                Expr::String(StringLiteral {
                    value: "a,b".to_string(),
                    span: Span::new(13, 18),
                }),
            ],
            span: Span::new(0, 19),
        })],
        span: Span::new(0, 20),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_explode(%EchoValue, %EchoValue, %EchoValue)")
    );
    assert!(
            ir.contains(
                "call %EchoValue @echo_php_explode(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1, %EchoValue { i32 2, i64 9223372036854775807 })"
            ),
            "{ir}"
        );
}

#[test]
fn implode_lowers_optional_separator_to_empty_string() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "implode".to_string(),
            args: vec![Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: None,
                    value: Expr::String(StringLiteral {
                        value: "a".to_string(),
                        span: Span::new(9, 12),
                    }),
                    span: Span::new(9, 12),
                }],
                span: Span::new(8, 13),
            })],
            span: Span::new(0, 14),
        })],
        span: Span::new(0, 15),
    })]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_php_implode(%EchoValue, %EchoValue)"));
    assert!(
        ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_") && ir.contains(", i64 0)")
    );
    assert!(ir.contains("call %EchoValue @echo_php_implode("), "{ir}");
}

#[test]
fn array_keys_lowers_optional_filter_and_strict_defaults() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "array_keys".to_string(),
            args: vec![Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: Some(Expr::String(StringLiteral {
                        value: "id".to_string(),
                        span: Span::new(12, 16),
                    })),
                    value: Expr::Number(NumberLiteral {
                        value: "10".to_string(),
                        span: Span::new(20, 22),
                    }),
                    span: Span::new(12, 22),
                }],
                span: Span::new(11, 23),
            })],
            span: Span::new(0, 24),
        })],
        span: Span::new(0, 25),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_keys(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("%EchoValue { i32 6, i64 0 }, %EchoValue { i32 1, i64 0 })"),
        "{ir}"
    );
}

#[test]
fn in_array_lowers_optional_strict_default() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "in_array".to_string(),
            args: vec![
                Expr::Number(NumberLiteral {
                    value: "2".to_string(),
                    span: Span::new(9, 10),
                }),
                Expr::Array(ArrayExpr {
                    elements: vec![ArrayElement {
                        key: None,
                        value: Expr::String(StringLiteral {
                            value: "2".to_string(),
                            span: Span::new(13, 16),
                        }),
                        span: Span::new(13, 16),
                    }],
                    span: Span::new(12, 17),
                }),
            ],
            span: Span::new(0, 18),
        })],
        span: Span::new(0, 19),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_in_array(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("%EchoValue { i32 1, i64 0 })"), "{ir}");
}

#[test]
fn array_fill_lowers_to_three_echo_value_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "array_fill".to_string(),
            args: vec![
                Expr::Number(NumberLiteral {
                    value: "-2".to_string(),
                    span: Span::new(11, 13),
                }),
                Expr::Number(NumberLiteral {
                    value: "3".to_string(),
                    span: Span::new(15, 16),
                }),
                Expr::String(StringLiteral {
                    value: "x".to_string(),
                    span: Span::new(18, 21),
                }),
            ],
            span: Span::new(0, 22),
        })],
        span: Span::new(0, 23),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_fill(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_php_array_fill("), "{ir}");
}

#[test]
fn array_reverse_lowers_optional_preserve_keys_default() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "array_reverse".to_string(),
            args: vec![Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: None,
                    value: Expr::String(StringLiteral {
                        value: "x".to_string(),
                        span: Span::new(15, 18),
                    }),
                    span: Span::new(15, 18),
                }],
                span: Span::new(14, 19),
            })],
            span: Span::new(0, 20),
        })],
        span: Span::new(0, 21),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_reverse(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("%EchoValue { i32 1, i64 0 })"), "{ir}");
}

#[test]
fn array_combine_and_pad_lower_to_runtime_calls() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_combine".to_string(),
                args: vec![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "sku".to_string(),
                                span: Span::new(15, 20),
                            }),
                            span: Span::new(15, 20),
                        }],
                        span: Span::new(14, 21),
                    }),
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "A-42".to_string(),
                                span: Span::new(24, 30),
                            }),
                            span: Span::new(24, 30),
                        }],
                        span: Span::new(23, 31),
                    }),
                ],
                span: Span::new(0, 32),
            })],
            span: Span::new(0, 33),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_pad".to_string(),
                args: vec![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "x".to_string(),
                                span: Span::new(50, 53),
                            }),
                            span: Span::new(50, 53),
                        }],
                        span: Span::new(49, 54),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "3".to_string(),
                        span: Span::new(56, 57),
                    }),
                    Expr::String(StringLiteral {
                        value: "missing".to_string(),
                        span: Span::new(59, 68),
                    }),
                ],
                span: Span::new(35, 69),
            })],
            span: Span::new(35, 70),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_array_combine(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_php_array_pad(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_combine("),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_php_array_pad("), "{ir}");
}

#[test]
fn array_slice_and_chunk_lower_optional_arguments_to_runtime_calls() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_slice".to_string(),
                args: vec![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "active".to_string(),
                                span: Span::new(13, 21),
                            }),
                            span: Span::new(13, 21),
                        }],
                        span: Span::new(12, 22),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "1".to_string(),
                        span: Span::new(24, 25),
                    }),
                ],
                span: Span::new(0, 26),
            })],
            span: Span::new(0, 27),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_chunk".to_string(),
                args: vec![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "active".to_string(),
                                span: Span::new(41, 49),
                            }),
                            span: Span::new(41, 49),
                        }],
                        span: Span::new(40, 50),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "2".to_string(),
                        span: Span::new(52, 53),
                    }),
                ],
                span: Span::new(28, 54),
            })],
            span: Span::new(28, 55),
        }),
    ]))
    .expect("IR");

    assert!(
            ir.contains(
                "declare %EchoValue @echo_php_array_slice(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
            ),
            "{ir}"
        );
    assert!(
        ir.contains("declare %EchoValue @echo_php_array_chunk(%EchoValue, %EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_slice(")
            && ir.contains("%EchoValue { i32 0, i64 0 }, %EchoValue { i32 1, i64 0 })"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_chunk(")
            && ir.contains("%EchoValue { i32 1, i64 0 })"),
        "{ir}"
    );
}

#[test]
fn array_merge_and_replace_pack_variadic_arguments_for_runtime_calls() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_merge".to_string(),
                args: vec![],
                span: Span::new(0, 13),
            })],
            span: Span::new(0, 14),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_replace".to_string(),
                args: vec![
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: Some(Expr::String(StringLiteral {
                                value: "sku".to_string(),
                                span: Span::new(30, 35),
                            })),
                            value: Expr::String(StringLiteral {
                                value: "A-42".to_string(),
                                span: Span::new(39, 45),
                            }),
                            span: Span::new(30, 45),
                        }],
                        span: Span::new(29, 46),
                    }),
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: Some(Expr::String(StringLiteral {
                                value: "sku".to_string(),
                                span: Span::new(49, 54),
                            })),
                            value: Expr::String(StringLiteral {
                                value: "A-43".to_string(),
                                span: Span::new(58, 64),
                            }),
                            span: Span::new(49, 64),
                        }],
                        span: Span::new(48, 65),
                    }),
                ],
                span: Span::new(15, 66),
            })],
            span: Span::new(15, 67),
        }),
    ]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_php_array_merge(%EchoValue)"));
    assert!(ir.contains("declare %EchoValue @echo_php_array_replace(%EchoValue)"));
    assert!(
        ir.contains("call %EchoValue @echo_value_array_new()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_value_array_append("),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_merge("),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_replace("),
        "{ir}"
    );
}

#[test]
fn array_search_and_count_values_lower_to_runtime_calls() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_search".to_string(),
                args: vec![
                    Expr::String(StringLiteral {
                        value: "active".to_string(),
                        span: Span::new(13, 21),
                    }),
                    Expr::Array(ArrayExpr {
                        elements: vec![ArrayElement {
                            key: None,
                            value: Expr::String(StringLiteral {
                                value: "active".to_string(),
                                span: Span::new(24, 32),
                            }),
                            span: Span::new(24, 32),
                        }],
                        span: Span::new(23, 33),
                    }),
                ],
                span: Span::new(0, 34),
            })],
            span: Span::new(0, 35),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_count_values".to_string(),
                args: vec![Expr::Array(ArrayExpr {
                    elements: vec![ArrayElement {
                        key: None,
                        value: Expr::String(StringLiteral {
                            value: "active".to_string(),
                            span: Span::new(58, 66),
                        }),
                        span: Span::new(58, 66),
                    }],
                    span: Span::new(57, 67),
                })],
                span: Span::new(37, 68),
            })],
            span: Span::new(37, 69),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains(
            "declare %EchoValue @echo_php_array_search(%EchoValue, %EchoValue, %EchoValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("declare %EchoValue @echo_php_array_count_values(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_array_search("),
        "{ir}"
    );
    assert!(ir.contains("%EchoValue { i32 1, i64 0 })"), "{ir}");
    assert!(
        ir.contains("call %EchoValue @echo_php_array_count_values("),
        "{ir}"
    );
}

#[test]
fn substr_compare_lowers_optional_arguments_to_php_defaults() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "substr_compare".to_string(),
            args: vec![
                Expr::String(StringLiteral {
                    value: "abcde".to_string(),
                    span: Span::new(16, 23),
                }),
                Expr::String(StringLiteral {
                    value: "de".to_string(),
                    span: Span::new(25, 29),
                }),
                Expr::Number(NumberLiteral {
                    value: "3".to_string(),
                    span: Span::new(31, 32),
                }),
            ],
            span: Span::new(0, 33),
        })],
        span: Span::new(0, 34),
    })]))
    .expect("IR");

    assert!(ir.contains("declare %EchoValue @echo_php_substr_compare(%EchoValue, %EchoValue, %EchoValue, %EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_php_substr_compare("));
    assert!(ir.contains("%EchoValue { i32 0, i64 0 }, %EchoValue { i32 1, i64 0 }"));
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
fn base_convert_lowers_to_three_echo_value_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "base_convert".to_string(),
            args: vec![
                Expr::String(StringLiteral {
                    value: "a37334".to_string(),
                    span: Span::new(13, 21),
                }),
                Expr::Number(NumberLiteral {
                    value: "16".to_string(),
                    span: Span::new(23, 25),
                }),
                Expr::Number(NumberLiteral {
                    value: "2".to_string(),
                    span: Span::new(27, 28),
                }),
            ],
            span: Span::new(0, 29),
        })],
        span: Span::new(0, 30),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "declare %EchoValue @echo_php_base_convert(%EchoValue, %EchoValue, %EchoValue)"
        )
    );
    assert!(ir.contains("call %EchoValue @echo_php_base_convert("));
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
