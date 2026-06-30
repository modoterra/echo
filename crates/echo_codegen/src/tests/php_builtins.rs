use super::*;

#[test]
fn string_case_builtins_lower_to_php_builtin_with_echo_value_argument() {
    for (php_name, symbol) in [
        ("strval", "echo_php_strval"),
        ("boolval", "echo_php_boolval"),
        ("intval", "echo_php_intval"),
        ("floatval", "echo_php_floatval"),
        ("doubleval", "echo_php_floatval"),
        ("str_word_count", "echo_php_str_word_count"),
        ("strtoupper", "echo_php_strtoupper"),
        ("strtolower", "echo_php_strtolower"),
        ("ucwords", "echo_php_ucwords"),
        ("strrev", "echo_php_strrev"),
        ("ucfirst", "echo_php_ucfirst"),
        ("lcfirst", "echo_php_lcfirst"),
        ("ord", "echo_php_ord"),
        ("str_rot13", "echo_php_str_rot13"),
        ("soundex", "echo_php_soundex"),
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
        ("extension_loaded", "echo_php_extension_loaded"),
        ("get_extension_funcs", "echo_php_get_extension_funcs"),
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
        ("stripcslashes", "echo_php_stripcslashes"),
        (
            "quoted_printable_encode",
            "echo_php_quoted_printable_encode",
        ),
        (
            "quoted_printable_decode",
            "echo_php_quoted_printable_decode",
        ),
        ("htmlspecialchars", "echo_php_htmlspecialchars"),
        (
            "htmlspecialchars_decode",
            "echo_php_htmlspecialchars_decode",
        ),
        ("strip_tags", "echo_php_strip_tags"),
        ("quotemeta", "echo_php_quotemeta"),
    ] {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: php_name.to_string(),
                args: echo_ast::call_args![Expr::String(StringLiteral {
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
fn round_lowers_optional_precision_to_zero() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "round".to_string(),
            args: echo_ast::call_args![Expr::Number(NumberLiteral {
                value: "3.5".to_string(),
                span: Span::new(6, 9),
            })],
            span: Span::new(0, 10),
        })],
        span: Span::new(0, 11),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_round(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %EchoValue @echo_php_round(%EchoValue { i32 11, i64 4615063718147915776 }, %EchoValue { i32 2, i64 0 })"
        ),
        "{ir}"
    );
}

#[test]
fn round_lowers_explicit_precision_argument() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "round".to_string(),
            args: echo_ast::call_args![
                Expr::Number(NumberLiteral {
                    value: "5.055".to_string(),
                    span: Span::new(6, 11),
                }),
                Expr::Number(NumberLiteral {
                    value: "2".to_string(),
                    span: Span::new(13, 14),
                }),
            ],
            span: Span::new(0, 15),
        })],
        span: Span::new(0, 16),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "call %EchoValue @echo_php_round(%EchoValue { i32 11, i64 4617377442456477368 }, %EchoValue { i32 2, i64 2 })"
        ),
        "{ir}"
    );
}

#[test]
fn phpversion_lowers_default_null_extension_argument() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "phpversion".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 12),
        })],
        span: Span::new(0, 13),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_phpversion(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_phpversion(%EchoValue { i32 0, i64 0 })"),
        "{ir}"
    );
}

#[test]
fn phpversion_lowers_explicit_extension_argument() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "phpversion".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "json".to_string(),
                span: Span::new(11, 17),
            })],
            span: Span::new(0, 18),
        })],
        span: Span::new(0, 19),
    })]))
    .expect("IR");

    assert!(
        ir.contains("call %EchoValue @echo_php_phpversion(%EchoValue %runtime_call_0)"),
        "{ir}"
    );
}

#[test]
fn php_sapi_name_lowers_to_no_argument_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "php_sapi_name".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 15),
        })],
        span: Span::new(0, 16),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_php_sapi_name()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_php_sapi_name()"),
        "{ir}"
    );
}

#[test]
fn zend_version_lowers_to_no_argument_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "zend_version".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 14),
        })],
        span: Span::new(0, 15),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_zend_version()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_zend_version()"),
        "{ir}"
    );
}

#[test]
fn php_ini_loaded_file_lowers_to_no_argument_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "php_ini_loaded_file".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 21),
        })],
        span: Span::new(0, 22),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_php_ini_loaded_file()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_php_ini_loaded_file()"),
        "{ir}"
    );
}

#[test]
fn php_ini_scanned_files_lowers_to_no_argument_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "php_ini_scanned_files".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 23),
        })],
        span: Span::new(0, 24),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_php_ini_scanned_files()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_php_ini_scanned_files()"),
        "{ir}"
    );
}

#[test]
fn get_cfg_var_lowers_to_unary_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "get_cfg_var".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "include_path".to_string(),
                span: Span::new(12, 26),
            })],
            span: Span::new(0, 27),
        })],
        span: Span::new(0, 28),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_get_cfg_var(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_get_cfg_var("),
        "{ir}"
    );
}

#[test]
fn ini_get_lowers_to_unary_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "ini_get".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "include_path".to_string(),
                span: Span::new(8, 22),
            })],
            span: Span::new(0, 23),
        })],
        span: Span::new(0, 24),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_ini_get(%EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_php_ini_get("), "{ir}");
}

#[test]
fn ini_get_all_lowers_default_arguments_to_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "ini_get_all".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 13),
        })],
        span: Span::new(0, 14),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_ini_get_all(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_ini_get_all("),
        "{ir}"
    );
    assert!(ir.contains("%EchoValue { i32 0, i64 0 }"), "{ir}");
    assert!(ir.contains("%EchoValue { i32 1, i64 1 }"), "{ir}");
}

#[test]
fn ini_parse_quantity_lowers_to_unary_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "ini_parse_quantity".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "256M".to_string(),
                span: Span::new(19, 25),
            })],
            span: Span::new(0, 26),
        })],
        span: Span::new(0, 27),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_ini_parse_quantity(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_ini_parse_quantity("),
        "{ir}"
    );
}

#[test]
fn get_include_path_lowers_to_no_argument_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "get_include_path".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 18),
        })],
        span: Span::new(0, 19),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_get_include_path()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_get_include_path()"),
        "{ir}"
    );
}

#[test]
fn headers_list_lowers_to_no_argument_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "headers_list".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 14),
        })],
        span: Span::new(0, 15),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_headers_list()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_headers_list()"),
        "{ir}"
    );
}

#[test]
fn headers_sent_lowers_to_no_argument_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "headers_sent".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 14),
        })],
        span: Span::new(0, 15),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_headers_sent()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_headers_sent()"),
        "{ir}"
    );
}

#[test]
fn ini_set_lowers_to_binary_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "ini_set".to_string(),
            args: echo_ast::call_args![
                Expr::String(StringLiteral {
                    value: "memory_limit".to_string(),
                    span: Span::new(8, 22),
                }),
                Expr::String(StringLiteral {
                    value: "128M".to_string(),
                    span: Span::new(24, 30),
                })
            ],
            span: Span::new(0, 31),
        })],
        span: Span::new(0, 32),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_ini_set(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_php_ini_set("), "{ir}");
}

#[test]
fn ini_alter_lowers_to_binary_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "ini_alter".to_string(),
            args: echo_ast::call_args![
                Expr::String(StringLiteral {
                    value: "memory_limit".to_string(),
                    span: Span::new(10, 24),
                }),
                Expr::String(StringLiteral {
                    value: "128M".to_string(),
                    span: Span::new(26, 32),
                })
            ],
            span: Span::new(0, 33),
        })],
        span: Span::new(0, 34),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_ini_alter(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_php_ini_alter("), "{ir}");
}

#[test]
fn ini_restore_lowers_to_void_unary_runtime_call() {
    let ir = compile_to_ir(&program(vec![Stmt::Expr(echo_ast::ExprStmt {
        expr: Expr::FunctionCall(FunctionCallExpr {
            name: "ini_restore".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "memory_limit".to_string(),
                span: Span::new(12, 26),
            })],
            span: Span::new(0, 27),
        }),
        span: Span::new(0, 28),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare void @echo_php_ini_restore(%EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call void @echo_php_ini_restore("), "{ir}");
}

#[test]
fn get_loaded_extensions_lowers_default_false_argument() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "get_loaded_extensions".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(0, 23),
        })],
        span: Span::new(0, 24),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_php_get_loaded_extensions(%EchoValue)"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_php_get_loaded_extensions(%EchoValue { i32 1, i64 0 })"),
        "{ir}"
    );
}

#[test]
fn get_loaded_extensions_lowers_explicit_zend_flag() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "get_loaded_extensions".to_string(),
            args: echo_ast::call_args![Expr::Bool(BoolLiteral {
                value: true,
                span: Span::new(22, 26),
            })],
            span: Span::new(0, 27),
        })],
        span: Span::new(0, 28),
    })]))
    .expect("IR");

    assert!(
        ir.contains("call %EchoValue @echo_php_get_loaded_extensions(%EchoValue { i32 1, i64 1 })"),
        "{ir}"
    );
}

#[test]
fn number_format_lowers_php_default_separators() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "number_format".to_string(),
            args: echo_ast::call_args![Expr::Number(NumberLiteral {
                value: "1234.56".to_string(),
                span: Span::new(14, 21),
            })],
            span: Span::new(0, 22),
        })],
        span: Span::new(0, 23),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "declare %EchoValue @echo_php_number_format(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains("%runtime_call_0 = call %EchoValue @echo_value_string("),
        "{ir}"
    );
    assert!(
        ir.contains("%runtime_call_1 = call %EchoValue @echo_value_string("),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %EchoValue @echo_php_number_format(%EchoValue { i32 11, i64 4653144467747100426 }, %EchoValue { i32 2, i64 0 }, %EchoValue %runtime_call_0, %EchoValue %runtime_call_1)"
        ),
        "{ir}"
    );
}

#[test]
fn number_format_lowers_custom_separator_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "number_format".to_string(),
            args: echo_ast::call_args![
                Expr::Number(NumberLiteral {
                    value: "1234.5".to_string(),
                    span: Span::new(14, 20),
                }),
                Expr::Number(NumberLiteral {
                    value: "2".to_string(),
                    span: Span::new(22, 23),
                }),
                Expr::String(StringLiteral {
                    value: ",".to_string(),
                    span: Span::new(25, 28),
                }),
                Expr::String(StringLiteral {
                    value: " ".to_string(),
                    span: Span::new(30, 33),
                }),
            ],
            span: Span::new(0, 34),
        })],
        span: Span::new(0, 35),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "call %EchoValue @echo_php_number_format(%EchoValue { i32 11, i64 4653144203864309760 }, %EchoValue { i32 2, i64 2 }, %EchoValue %runtime_call_0, %EchoValue %runtime_call_1)"
        ),
        "{ir}"
    );
}

#[test]
fn levenshtein_lowers_php_default_costs() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "levenshtein".to_string(),
            args: echo_ast::call_args![
                Expr::String(StringLiteral {
                    value: "kitten".to_string(),
                    span: Span::new(12, 20),
                }),
                Expr::String(StringLiteral {
                    value: "sitting".to_string(),
                    span: Span::new(22, 31),
                }),
            ],
            span: Span::new(0, 32),
        })],
        span: Span::new(0, 33),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "declare %EchoValue @echo_php_levenshtein(%EchoValue, %EchoValue, %EchoValue, %EchoValue, %EchoValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %EchoValue @echo_php_levenshtein(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1, %EchoValue { i32 2, i64 1 }, %EchoValue { i32 2, i64 1 }, %EchoValue { i32 2, i64 1 })"
        ),
        "{ir}"
    );
}

#[test]
fn levenshtein_lowers_explicit_cost_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "levenshtein".to_string(),
            args: echo_ast::call_args![
                Expr::String(StringLiteral {
                    value: "abc".to_string(),
                    span: Span::new(12, 17),
                }),
                Expr::String(StringLiteral {
                    value: "adc".to_string(),
                    span: Span::new(19, 24),
                }),
                Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(26, 27),
                }),
                Expr::Number(NumberLiteral {
                    value: "5".to_string(),
                    span: Span::new(29, 30),
                }),
            ],
            span: Span::new(0, 31),
        })],
        span: Span::new(0, 32),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "call %EchoValue @echo_php_levenshtein(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1, %EchoValue { i32 2, i64 1 }, %EchoValue { i32 2, i64 5 }, %EchoValue { i32 2, i64 1 })"
        ),
        "{ir}"
    );
}

#[test]
fn wordwrap_lowers_php_default_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "wordwrap".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "The quick brown fox".to_string(),
                span: Span::new(9, 30),
            })],
            span: Span::new(0, 31),
        })],
        span: Span::new(0, 32),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "declare %EchoValue @echo_php_wordwrap(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %EchoValue @echo_php_wordwrap(%EchoValue %runtime_call_0, %EchoValue { i32 2, i64 75 }, %EchoValue %runtime_call_1, %EchoValue { i32 1, i64 0 })"
        ),
        "{ir}"
    );
}

#[test]
fn wordwrap_lowers_explicit_arguments() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "wordwrap".to_string(),
            args: echo_ast::call_args![
                Expr::String(StringLiteral {
                    value: "abcdefghij".to_string(),
                    span: Span::new(9, 21),
                }),
                Expr::Number(NumberLiteral {
                    value: "4".to_string(),
                    span: Span::new(23, 24),
                }),
                Expr::String(StringLiteral {
                    value: "|".to_string(),
                    span: Span::new(26, 29),
                }),
                Expr::Bool(BoolLiteral {
                    value: true,
                    span: Span::new(31, 35),
                }),
            ],
            span: Span::new(0, 36),
        })],
        span: Span::new(0, 37),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "call %EchoValue @echo_php_wordwrap(%EchoValue %runtime_call_0, %EchoValue { i32 2, i64 4 }, %EchoValue %runtime_call_1, %EchoValue { i32 1, i64 1 })"
        ),
        "{ir}"
    );
}

#[test]
fn nl2br_lowers_optional_xhtml_flag_to_true() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "nl2br".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
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
        ("strnatcmp", "echo_php_strnatcmp"),
        ("strnatcasecmp", "echo_php_strnatcasecmp"),
        ("atan2", "echo_php_atan2"),
        ("intdiv", "echo_php_intdiv"),
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
                args: echo_ast::call_args![
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
fn substr_replace_lowers_optional_length_to_null() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "substr_replace".to_string(),
            args: echo_ast::call_args![
                Expr::String(StringLiteral {
                    value: "Bearer abc123".to_string(),
                    span: Span::new(15, 30),
                }),
                Expr::String(StringLiteral {
                    value: "redacted".to_string(),
                    span: Span::new(32, 42),
                }),
                Expr::Number(NumberLiteral {
                    value: "7".to_string(),
                    span: Span::new(44, 45),
                }),
            ],
            span: Span::new(0, 46),
        })],
        span: Span::new(0, 47),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "declare %EchoValue @echo_php_substr_replace(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
        ),
        "{ir}"
    );
    assert!(
        ir.contains(
            "call %EchoValue @echo_php_substr_replace(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1, %EchoValue { i32 2, i64 7 }, %EchoValue { i32 0, i64 0 })"
        ),
        "{ir}"
    );
}

#[test]
fn substr_replace_lowers_explicit_length_argument() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::FunctionCall(FunctionCallExpr {
            name: "substr_replace".to_string(),
            args: echo_ast::call_args![
                Expr::String(StringLiteral {
                    value: "abcdef".to_string(),
                    span: Span::new(15, 23),
                }),
                Expr::String(StringLiteral {
                    value: "XX".to_string(),
                    span: Span::new(25, 29),
                }),
                Expr::Number(NumberLiteral {
                    value: "2".to_string(),
                    span: Span::new(31, 32),
                }),
                Expr::Number(NumberLiteral {
                    value: "3".to_string(),
                    span: Span::new(34, 35),
                }),
            ],
            span: Span::new(0, 36),
        })],
        span: Span::new(0, 37),
    })]))
    .expect("IR");

    assert!(
        ir.contains(
            "call %EchoValue @echo_php_substr_replace(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1, %EchoValue { i32 2, i64 2 }, %EchoValue { i32 2, i64 3 })"
        ),
        "{ir}"
    );
}

#[test]
fn runtime_declarations_deduplicate_alias_symbols() {
    let declarations = runtime_declarations();

    assert_eq!(declarations.matches("@echo_php_strstr(").count(), 1);
}
