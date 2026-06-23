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
mod php_builtins;
mod php_compat;
mod php_environment;
mod php_filesystem;
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
