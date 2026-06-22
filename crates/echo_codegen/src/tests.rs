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
fn userland_function_declaration_registers_reflection_metadata() {
    let ir = compile_to_ir(&program(vec![
        std_import("reflect"),
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "greet".to_string(),
            params: vec![TypedParam {
                name: "name".to_string(),
                ty: Some("string".to_string()),
            }],
            return_type: Some("string".to_string()),
            is_intrinsic: false,
            is_generator: false,
            body: vec![Stmt::Return(ReturnStmt {
                value: Expr::String(StringLiteral {
                    value: "hello\n".to_string(),
                    span: Span::new(0, 8),
                }),
                span: Span::new(0, 15),
            })],
            span: Span::new(0, 40),
        }),
        Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "reflect.params".to_string(),
                args: vec![Expr::String(StringLiteral {
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

#[test]
fn dynamic_concat_uses_echo_value_concat() {
    let ir = compile_to_ir(&program(vec![
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "greet".to_string(),
            params: vec![param("name")],
            return_type: None,
            is_intrinsic: false,
            is_generator: false,
            body: vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::Binary(Box::new(echo_ast::BinaryExpr {
                    left: Expr::Binary(Box::new(echo_ast::BinaryExpr {
                        left: Expr::String(StringLiteral {
                            value: "Hello, ".to_string(),
                            span: Span::new(0, 9),
                        }),
                        op: BinaryOp::Concat,
                        right: Expr::Variable(echo_ast::VariableExpr {
                            name: "name".to_string(),
                            span: Span::new(12, 17),
                        }),
                        span: Span::new(0, 17),
                    })),
                    op: BinaryOp::Concat,
                    right: Expr::String(StringLiteral {
                        value: "!\n".to_string(),
                        span: Span::new(20, 24),
                    }),
                    span: Span::new(0, 24),
                }))],
                span: Span::new(0, 30),
            })],
            span: Span::new(0, 40),
        }),
        Stmt::FunctionCall(FunctionCallStmt {
            name: "greet".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "Echo".to_string(),
                span: Span::new(45, 51),
            })],
            span: Span::new(41, 53),
        }),
    ]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_value_concat(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_value_concat"), "{ir}");
}

#[test]
fn integer_subtraction_lowers_to_echo_value_sub() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Binary(Box::new(echo_ast::BinaryExpr {
            left: Expr::Number(NumberLiteral {
                value: "3".to_string(),
                span: Span::new(5, 6),
            }),
            op: BinaryOp::Sub,
            right: Expr::Number(NumberLiteral {
                value: "5".to_string(),
                span: Span::new(7, 8),
            }),
            span: Span::new(5, 8),
        }))],
        span: Span::new(0, 9),
    })]))
    .expect("IR");

    assert!(
        ir.contains("declare %EchoValue @echo_value_sub(%EchoValue, %EchoValue)"),
        "{ir}"
    );
    assert!(ir.contains("call %EchoValue @echo_value_sub"), "{ir}");
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
    for builtin in PHP_BUILTINS {
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
fn array_literals_lower_to_array_runtime() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Array(ArrayExpr {
            elements: vec![ArrayElement {
                key: None,
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(1, 2),
                }),
                span: Span::new(1, 2),
            }],
            span: Span::new(0, 3),
        })],
        span: Span::new(0, 3),
    })]))
    .expect("array literal should lower");

    assert!(ir.contains("declare %EchoValue @echo_value_array_new()"));
    assert!(ir.contains("declare %EchoValue @echo_value_array_append(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_array_new()"));
    assert!(ir.contains("call %EchoValue @echo_value_array_append"));
    assert!(!ir.contains("call %EchoValue @echo_value_list_append"));
}

#[test]
fn keyed_array_literals_lower_to_array_set_runtime() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Array(ArrayExpr {
            elements: vec![ArrayElement {
                key: Some(Expr::String(StringLiteral {
                    value: "key".to_string(),
                    span: Span::new(1, 6),
                })),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(10, 11),
                }),
                span: Span::new(1, 11),
            }],
            span: Span::new(0, 12),
        })],
        span: Span::new(0, 12),
    })]))
    .expect("keyed array should lower");

    assert!(
        ir.contains("declare %EchoValue @echo_value_array_set(%EchoValue, %EchoValue, %EchoValue)")
    );
    assert!(ir.contains("call %EchoValue @echo_value_array_set"));
}

#[test]
fn index_access_lowers_to_index_get_runtime() {
    let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
        exprs: vec![Expr::Index(Box::new(echo_ast::IndexExpr {
            collection: Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: None,
                    value: Expr::Number(NumberLiteral {
                        value: "4".to_string(),
                        span: Span::new(1, 2),
                    }),
                    span: Span::new(1, 2),
                }],
                span: Span::new(0, 3),
            }),
            index: Expr::Number(NumberLiteral {
                value: "0".to_string(),
                span: Span::new(4, 5),
            }),
            span: Span::new(0, 6),
        }))],
        span: Span::new(0, 6),
    })]))
    .expect("index access should lower");

    assert!(ir.contains("declare %EchoValue @echo_value_index_get(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_index_get"));
}

#[test]
fn append_statement_lowers_to_array_append_runtime() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Let(echo_ast::LetStmt {
            name: "a".to_string(),
            ty: None,
            value: Expr::Array(ArrayExpr {
                elements: vec![],
                span: Span::new(9, 11),
            }),
            span: Span::new(0, 12),
        }),
        Stmt::Append(echo_ast::AppendStmt {
            target: "a".to_string(),
            value: Expr::Number(NumberLiteral {
                value: "2".to_string(),
                span: Span::new(20, 21),
            }),
            span: Span::new(13, 22),
        }),
    ]))
    .expect("array append should lower");

    assert!(ir.contains("declare %EchoValue @echo_value_array_append(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_array_append"));
    assert!(!ir.contains("call %EchoValue @echo_value_list_append"));
}

#[test]
fn list_push_method_lowers_to_list_append_runtime() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Let(LetStmt {
            name: "items".to_string(),
            ty: Some("list<string>".to_string()),
            value: Expr::List(ListExpr {
                values: vec![],
                span: Span::new(27, 29),
            }),
            span: Span::new(0, 29),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Variable(VariableExpr {
                    name: "items".to_string(),
                    span: Span::new(30, 36),
                }),
                method: "push".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "first".to_string(),
                    span: Span::new(42, 49),
                })],
                span: Span::new(30, 50),
            })),
            span: Span::new(30, 50),
        }),
    ]))
    .expect("list push should lower");

    assert!(ir.contains("declare %EchoValue @echo_value_list_append(%EchoValue, %EchoValue)"));
    assert!(ir.contains("call %EchoValue @echo_value_list_append"));
    assert!(!ir.contains("call %EchoValue @echo_value_array_append"));
}

#[test]
fn list_push_lowers_type_ascribed_structural_literal() {
    let ir = compile_to_ir(&program(vec![
        Stmt::Let(LetStmt {
            name: "users".to_string(),
            ty: Some("list<User>".to_string()),
            value: Expr::List(ListExpr {
                values: vec![],
                span: Span::new(24, 26),
            }),
            span: Span::new(0, 26),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Variable(VariableExpr {
                    name: "users".to_string(),
                    span: Span::new(27, 33),
                }),
                method: "push".to_string(),
                args: vec![Expr::TypeAscription(Box::new(TypeAscriptionExpr {
                    expr: Expr::Object(ObjectExpr {
                        name: String::new(),
                        fields: vec![ObjectField {
                            name: "email".to_string(),
                            value: Expr::String(StringLiteral {
                                value: "first@example.test".to_string(),
                                span: Span::new(50, 70),
                            }),
                        }],
                        span: Span::new(39, 72),
                    }),
                    ty: "User".to_string(),
                    span: Span::new(39, 78),
                }))],
                span: Span::new(27, 79),
            })),
            span: Span::new(27, 79),
        }),
    ]))
    .expect("type-ascribed structural literal push should lower");

    assert!(ir.contains("call %EchoValue @echo_value_object_new()"));
    assert!(ir.contains("call %EchoValue @echo_value_object_set"));
    assert!(ir.contains("call %EchoValue @echo_value_list_append"));
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
