use super::*;
use echo_ast::{
    ArrayElement, ArrayExpr, AssignStmt, BinaryExpr, BinaryOp, BoolLiteral, BreakStmt,
    ClassDeclStmt, ClassMember, DeferExpr, EchoStmt, Expr, ExprStmt, FunctionCallExpr,
    FunctionCallStmt, FunctionDeclStmt, IfStmt, ImportStmt, IncludeExpr, IncludeKind, LetStmt,
    ListExpr, LoopExpr, LoopStmt, MethodCallExpr, MethodDecl, MethodVisibility, NewExpr, NewTarget,
    NullLiteral, NumberLiteral, ObjectExpr, ObjectField, QualifiedName, ReturnStmt, StringLiteral,
    TraitDeclStmt, TypeAscriptionExpr, TypedParam, UseStmt, VariableExpr,
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
        default_value: None,
        promoted_visibility: None,
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
fn unsupported_method_call_reports_codegen_error() {
    let err = compile_to_ir(&program(vec![
        Stmt::Let(LetStmt {
            name: "app".to_string(),
            ty: None,
            value: Expr::Object(ObjectExpr {
                name: String::new(),
                fields: vec![],
                span: Span::new(11, 13),
            }),
            span: Span::new(0, 13),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Variable(VariableExpr {
                    name: "app".to_string(),
                    span: Span::new(14, 18),
                }),
                method: "doSomethingUnsupported".to_string(),
                method_span: Span::new(20, 42),
                args: echo_ast::call_args![],
                span: Span::new(14, 44),
            })),
            span: Span::new(14, 44),
        }),
    ]))
    .expect_err("unknown method calls should not lower to no-op values");

    assert_eq!(err.len(), 1);
    assert_eq!(
        err[0].message,
        "unsupported method call `doSomethingUnsupported` in LLVM codegen"
    );
    assert_eq!(err[0].span, Span::new(20, 42));
}

#[test]
fn instance_method_call_on_new_object_calls_matching_class_method() {
    let ir = compile_to_ir(&program(vec![
        Stmt::ClassDecl(ClassDeclStmt {
            name: "Worker".to_string(),
            parent: None,
            interfaces: Vec::new(),
            members: vec![ClassMember::Method(MethodDecl {
                name: "run".to_string(),
                params: Vec::new(),
                return_type: None,
                body: vec![Stmt::Echo(EchoStmt {
                    exprs: vec![Expr::String(StringLiteral {
                        value: "ran".to_string(),
                        span: Span::new(0, 5),
                    })],
                    span: Span::new(0, 6),
                })],
                visibility: MethodVisibility::Public,
                is_static: false,
                is_intrinsic: false,
                span: Span::new(0, 20),
            })],
            span: Span::new(0, 30),
        }),
        Stmt::Let(LetStmt {
            name: "worker".to_string(),
            ty: None,
            value: Expr::New(Box::new(NewExpr {
                target: NewTarget::Class(QualifiedName::new(vec!["Worker".to_string()])),
                args: Vec::new(),
                span: Span::new(31, 43),
            })),
            span: Span::new(31, 44),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Variable(VariableExpr {
                    name: "worker".to_string(),
                    span: Span::new(45, 52),
                }),
                method: "run".to_string(),
                method_span: Span::new(54, 57),
                args: echo_ast::call_args![],
                span: Span::new(45, 59),
            })),
            span: Span::new(45, 60),
        }),
    ]))
    .expect("known instance method should lower");

    assert!(
        ir.contains("define %EchoValue @echo_user_Worker__run()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_user_Worker__run()"),
        "{ir}"
    );
}

#[test]
fn instance_method_call_on_new_object_uses_parent_class_method() {
    let ir = compile_to_ir(&program(vec![
        Stmt::ClassDecl(ClassDeclStmt {
            name: "Container".to_string(),
            parent: None,
            interfaces: Vec::new(),
            members: vec![ClassMember::Method(MethodDecl {
                name: "singleton".to_string(),
                params: Vec::new(),
                return_type: None,
                body: vec![Stmt::Return(ReturnStmt {
                    value: None,
                    span: Span::new(0, 7),
                })],
                visibility: MethodVisibility::Public,
                is_static: false,
                is_intrinsic: false,
                span: Span::new(0, 20),
            })],
            span: Span::new(0, 30),
        }),
        Stmt::ClassDecl(ClassDeclStmt {
            name: "Application".to_string(),
            parent: Some(QualifiedName::new(vec!["Container".to_string()])),
            interfaces: Vec::new(),
            members: Vec::new(),
            span: Span::new(31, 60),
        }),
        Stmt::Let(LetStmt {
            name: "app".to_string(),
            ty: None,
            value: Expr::New(Box::new(NewExpr {
                target: NewTarget::Class(QualifiedName::new(vec!["Application".to_string()])),
                args: Vec::new(),
                span: Span::new(61, 78),
            })),
            span: Span::new(61, 79),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Variable(VariableExpr {
                    name: "app".to_string(),
                    span: Span::new(80, 84),
                }),
                method: "singleton".to_string(),
                method_span: Span::new(86, 95),
                args: echo_ast::call_args![],
                span: Span::new(80, 97),
            })),
            span: Span::new(80, 98),
        }),
    ]))
    .expect("inherited instance method should lower");

    assert!(
        ir.contains("define %EchoValue @echo_user_Container__singleton()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_user_Container__singleton()"),
        "{ir}"
    );
}

#[test]
fn instance_method_call_on_new_object_uses_trait_method_before_parent() {
    let ir = compile_to_ir(&program(vec![
        Stmt::TraitDecl(TraitDeclStmt {
            name: "ReflectsClosures".to_string(),
            members: vec![ClassMember::Method(MethodDecl {
                name: "closureReturnTypes".to_string(),
                params: Vec::new(),
                return_type: None,
                body: vec![Stmt::Return(ReturnStmt {
                    value: None,
                    span: Span::new(0, 7),
                })],
                visibility: MethodVisibility::Protected,
                is_static: false,
                is_intrinsic: false,
                span: Span::new(0, 30),
            })],
            span: Span::new(0, 40),
        }),
        Stmt::ClassDecl(ClassDeclStmt {
            name: "Container".to_string(),
            parent: None,
            interfaces: Vec::new(),
            members: vec![ClassMember::TraitUse(QualifiedName::new(vec![
                "ReflectsClosures".to_string(),
            ]))],
            span: Span::new(41, 80),
        }),
        Stmt::Let(LetStmt {
            name: "container".to_string(),
            ty: None,
            value: Expr::New(Box::new(NewExpr {
                target: NewTarget::Class(QualifiedName::new(vec!["Container".to_string()])),
                args: Vec::new(),
                span: Span::new(81, 96),
            })),
            span: Span::new(81, 97),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::Variable(VariableExpr {
                    name: "container".to_string(),
                    span: Span::new(98, 108),
                }),
                method: "closureReturnTypes".to_string(),
                method_span: Span::new(110, 128),
                args: echo_ast::call_args![],
                span: Span::new(98, 130),
            })),
            span: Span::new(98, 131),
        }),
    ]))
    .expect("trait instance method should lower");

    assert!(
        ir.contains("define %EchoValue @echo_user_ReflectsClosures__closureReturnTypes()"),
        "{ir}"
    );
    assert!(
        ir.contains("call %EchoValue @echo_user_ReflectsClosures__closureReturnTypes()"),
        "{ir}"
    );
}

#[test]
fn promoted_property_type_guides_method_dispatch() {
    let ir = compile_to_ir(&program(vec![
        Stmt::ClassDecl(ClassDeclStmt {
            name: "App".to_string(),
            parent: None,
            interfaces: Vec::new(),
            members: vec![ClassMember::Method(MethodDecl {
                name: "singleton".to_string(),
                params: Vec::new(),
                return_type: None,
                body: vec![Stmt::Return(ReturnStmt {
                    value: None,
                    span: Span::new(0, 7),
                })],
                visibility: MethodVisibility::Public,
                is_static: false,
                is_intrinsic: false,
                span: Span::new(0, 20),
            })],
            span: Span::new(0, 30),
        }),
        Stmt::ClassDecl(ClassDeclStmt {
            name: "Builder".to_string(),
            parent: None,
            interfaces: Vec::new(),
            members: vec![
                ClassMember::Method(MethodDecl {
                    name: "__construct".to_string(),
                    params: vec![TypedParam {
                        name: "app".to_string(),
                        ty: Some("App".to_string()),
                        default_value: None,
                        promoted_visibility: Some(MethodVisibility::Protected),
                    }],
                    return_type: None,
                    body: Vec::new(),
                    visibility: MethodVisibility::Public,
                    is_static: false,
                    is_intrinsic: false,
                    span: Span::new(31, 60),
                }),
                ClassMember::Method(MethodDecl {
                    name: "boot".to_string(),
                    params: Vec::new(),
                    return_type: None,
                    body: vec![Stmt::Expr(ExprStmt {
                        expr: Expr::MethodCall(Box::new(MethodCallExpr {
                            object: Expr::Field(Box::new(echo_ast::FieldExpr {
                                object: Expr::ReceiverConst(echo_ast::ReceiverConstExpr {
                                    kind: echo_ast::ReceiverConst::This,
                                    span: Span::new(61, 66),
                                }),
                                field: "app".to_string(),
                                span: Span::new(61, 71),
                            })),
                            method: "singleton".to_string(),
                            method_span: Span::new(73, 82),
                            args: echo_ast::call_args![],
                            span: Span::new(61, 84),
                        })),
                        span: Span::new(61, 85),
                    })],
                    visibility: MethodVisibility::Public,
                    is_static: false,
                    is_intrinsic: false,
                    span: Span::new(61, 90),
                }),
            ],
            span: Span::new(31, 90),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::MethodCall(Box::new(MethodCallExpr {
                object: Expr::New(Box::new(NewExpr {
                    target: NewTarget::Class(QualifiedName::new(vec!["Builder".to_string()])),
                    args: Vec::new(),
                    span: Span::new(91, 104),
                })),
                method: "boot".to_string(),
                method_span: Span::new(106, 110),
                args: echo_ast::call_args![],
                span: Span::new(91, 112),
            })),
            span: Span::new(91, 113),
        }),
    ]))
    .expect("promoted property type should guide method dispatch");

    assert!(
        ir.contains("call %EchoValue @echo_user_App__singleton()"),
        "{ir}"
    );
}

#[test]
fn userland_calls_fill_omitted_default_parameters() {
    let ir = compile_to_ir(&program(vec![
        Stmt::FunctionDecl(FunctionDeclStmt {
            name: "example".to_string(),
            params: vec![TypedParam {
                name: "value".to_string(),
                ty: None,
                default_value: Some(Expr::String(StringLiteral {
                    value: "fallback".to_string(),
                    span: Span::new(17, 27),
                })),
                promoted_visibility: None,
            }],
            return_type: None,
            is_intrinsic: false,
            is_generator: false,
            body: Vec::new(),
            span: Span::new(0, 30),
        }),
        Stmt::FunctionCall(FunctionCallStmt {
            name: "example".to_string(),
            args: echo_ast::call_args![],
            span: Span::new(31, 40),
        }),
    ]))
    .expect("omitted default parameter should lower");

    assert!(ir.contains("@echo_str_"), "{ir}");
    assert!(ir.contains("fallback"), "{ir}");
    assert!(ir.contains("call %EchoValue @echo_user_example("), "{ir}");
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
fn dynamic_new_dispatches_to_bundled_class_include_unit() {
    let entry = program(vec![
        Stmt::Let(LetStmt {
            name: "provider".to_string(),
            ty: None,
            value: Expr::String(StringLiteral {
                value: "App\\Provider".to_string(),
                span: Span::new(16, 30),
            }),
            span: Span::new(0, 31),
        }),
        Stmt::Let(LetStmt {
            name: "instance".to_string(),
            ty: None,
            value: Expr::New(Box::new(NewExpr {
                target: NewTarget::Expr(Box::new(Expr::Variable(VariableExpr {
                    name: "provider".to_string(),
                    span: Span::new(52, 61),
                }))),
                args: Vec::new(),
                span: Span::new(48, 63),
            })),
            span: Span::new(32, 64),
        }),
    ]);
    let include = program(Vec::new());
    let entry_hir = echo_hir::lower_program(&entry).expect("entry lowers to HIR");
    let include_hir = echo_hir::lower_program(&include).expect("include lowers to HIR");
    let entry_mir = echo_mir::lower_program(&entry_hir).expect("entry lowers to MIR");
    let include_mir = echo_mir::lower_program(&include_hir).expect("include lowers to MIR");

    let ir = compile_mir_bundle_to_ir_detailed(
        &entry_mir,
        &[MirIncludeUnit {
            path: "/app/Provider.php".to_string(),
            program: include_mir,
            dynamic_require: false,
            class_names: vec!["App\\Provider".to_string()],
        }],
    )
    .expect("IR");

    assert!(ir.contains("dynamic_class_autoload_is_match_"), "{ir}");
    assert!(ir.contains("call i1 @echo_value_string_equals_ptr"), "{ir}");
    assert!(
        ir.contains("call %EchoValue @echo_include_unit_0()"),
        "{ir}"
    );
}

#[test]
fn dynamic_require_dispatches_only_to_bundled_include_units() {
    let entry = program(vec![
        Stmt::Let(LetStmt {
            name: "file".to_string(),
            ty: None,
            value: Expr::String(StringLiteral {
                value: "/app/helpers.php".to_string(),
                span: Span::new(12, 30),
            }),
            span: Span::new(0, 31),
        }),
        Stmt::Expr(ExprStmt {
            expr: Expr::Include(Box::new(IncludeExpr {
                kind: IncludeKind::Require,
                path: Expr::Variable(VariableExpr {
                    name: "file".to_string(),
                    span: Span::new(40, 45),
                }),
                span: Span::new(32, 46),
            })),
            span: Span::new(32, 47),
        }),
    ]);
    let include = program(Vec::new());
    let entry_hir = echo_hir::lower_program(&entry).expect("entry lowers to HIR");
    let include_hir = echo_hir::lower_program(&include).expect("include lowers to HIR");
    let entry_mir = echo_mir::lower_program(&entry_hir).expect("entry lowers to MIR");
    let include_mir = echo_mir::lower_program(&include_hir).expect("include lowers to MIR");

    let ir = compile_mir_bundle_to_ir_detailed(
        &entry_mir,
        &[MirIncludeUnit {
            path: "/app/helpers.php".to_string(),
            program: include_mir,
            dynamic_require: true,
            class_names: Vec::new(),
        }],
    )
    .expect("IR");

    assert!(ir.contains("dynamic_include_is_match_"), "{ir}");
    assert!(ir.contains("call i1 @echo_value_string_equals_ptr"), "{ir}");
    assert!(
        ir.contains("call %EchoValue @echo_include_unit_0()"),
        "{ir}"
    );
    let fallback = ir
        .split("dynamic_include_fallback_")
        .nth(1)
        .expect("dynamic include fallback label should exist");
    let fallback = fallback
        .split("dynamic_include_done_")
        .next()
        .expect("dynamic include done label should follow fallback");
    assert!(
        !fallback.contains("echo_php_require"),
        "dynamic include miss must stay closed-world:\n{fallback}"
    );
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
        args: echo_ast::call_args![Expr::String(StringLiteral {
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
fn php_use_std_namespace_does_not_register_std_module_import() {
    let diagnostics = compile_to_ir(&program(vec![
        Stmt::Use(UseStmt {
            name: QualifiedName::new(vec!["std".to_string(), "net".to_string()]),
            alias: None,
            span: Span::new(0, 11),
        }),
        Stmt::FunctionCall(FunctionCallStmt {
            name: "net.listen".to_string(),
            args: echo_ast::call_args![Expr::String(StringLiteral {
                value: "127.0.0.1:39183".to_string(),
                span: Span::new(23, 42),
            })],
            span: Span::new(12, 43),
        }),
    ]))
    .expect_err("PHP use syntax must not satisfy std module imports");

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
            args: echo_ast::call_args![Expr::Null(NullLiteral {
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
            args: echo_ast::call_args![
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
            args: echo_ast::call_args![Expr::String(StringLiteral {
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
            args: echo_ast::call_args![Expr::String(StringLiteral {
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
                args: echo_ast::call_args![
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
            args: echo_ast::call_args![
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
            args: echo_ast::call_args![Expr::Array(ArrayExpr {
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
            args: echo_ast::call_args![
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
            args: echo_ast::call_args![
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
