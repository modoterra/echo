mod abi;
mod collection_lowering;
mod control_flow_lowering;
mod expression_lowering;
mod jit;
mod module;
mod php_lowering;
mod reflection_lowering;
mod runtime_symbols;
mod statement_lowering;
mod std_lowering;
mod task_lowering;
mod userland_lowering;

use abi::{
    BuiltinCodegen, CoreRuntimeSymbol, PHP_BUILTINS, PHP_RUNTIME_HELPERS, STD_INTRINSICS,
    php_builtin, std_intrinsic,
};
use echo_ast::{BinaryOp, Program, UnaryOp};
#[cfg(test)]
use echo_ast::{ImportSource, Stmt};
use echo_diagnostics::Diagnostic;
use echo_source::Span;
use inkwell::context::Context;
use std::collections::HashSet;

use jit::run_ir_jit;
pub use jit::{JitOptions, JitOutput, run_ir_jit_with_options};
pub(crate) use module::{IrModule, RuntimeValue, stmt_span};
pub(crate) use runtime_symbols::jit_runtime_symbol_addresses;

pub fn backend_name() -> &'static str {
    "llvm"
}

pub fn smoke_test_module_ir() -> String {
    let context = Context::create();
    let module = context.create_module("echo_smoke");

    module.print_to_string().to_string()
}

pub fn compile_to_ir(program: &Program) -> Result<String, Vec<Diagnostic>> {
    let hir = echo_hir::lower_program(program)?;
    let mir = echo_mir::lower_program(&hir)?;

    compile_mir_to_ir(&mir)
}

pub struct IncludeUnit<'a> {
    pub path: String,
    pub program: &'a Program,
}

pub struct MirIncludeUnit {
    pub path: String,
    pub program: echo_mir::MirProgram,
    pub dynamic_require: bool,
    pub class_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenDiagnostic {
    pub unit_path: Option<String>,
    pub diagnostic: Diagnostic,
}

pub fn compile_bundle_to_ir(
    entry: &Program,
    includes: &[IncludeUnit<'_>],
) -> Result<String, Vec<Diagnostic>> {
    let entry_hir = echo_hir::lower_program(entry)?;
    let entry_mir = echo_mir::lower_program(&entry_hir)?;
    let mut lowered_includes = Vec::new();

    for include in includes {
        let hir = echo_hir::lower_program(include.program)?;
        let mir = echo_mir::lower_program(&hir)?;
        lowered_includes.push(MirIncludeUnit {
            path: include.path.clone(),
            program: mir,
            dynamic_require: true,
            class_names: Vec::new(),
        });
    }

    compile_mir_bundle_to_ir(&entry_mir, &lowered_includes)
}

pub fn compile_hir_to_ir(program: &echo_hir::HirProgram) -> Result<String, Vec<Diagnostic>> {
    let mir = echo_mir::lower_program(program)?;

    compile_mir_to_ir(&mir)
}

pub fn compile_mir_to_ir(program: &echo_mir::MirProgram) -> Result<String, Vec<Diagnostic>> {
    compile_mir_bundle_to_ir(program, &[])
}

fn compile_mir_bundle_to_ir(
    program: &echo_mir::MirProgram,
    includes: &[MirIncludeUnit],
) -> Result<String, Vec<Diagnostic>> {
    compile_mir_bundle_to_ir_detailed(program, includes).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.diagnostic)
            .collect()
    })
}

pub fn compile_mir_bundle_to_ir_detailed(
    program: &echo_mir::MirProgram,
    includes: &[MirIncludeUnit],
) -> Result<String, Vec<CodegenDiagnostic>> {
    let mut module = IrModule::new();
    module.source_dir = program.source_dir().map(str::to_string);

    for (index, include) in includes.iter().enumerate() {
        module
            .include_units
            .insert(include.path.clone(), format!("echo_include_unit_{index}"));
        if include.dynamic_require {
            module
                .dynamic_include_units
                .insert(include.path.clone(), format!("echo_include_unit_{index}"));
        }
        for class_name in &include.class_names {
            module
                .class_include_units
                .insert(class_name.clone(), include.path.clone());
        }
        module
            .include_once_globals
            .insert(include.path.clone(), format!("echo_include_once_{index}"));
    }

    for include in includes {
        module.index_class_parents(include.program.statements());
        for function in include.program.functions() {
            module
                .functions
                .insert(function.name.clone(), function.clone());
        }
    }

    let mut diagnostics = Vec::new();
    for include in includes {
        if let Err(diagnostic) = module.render_include_unit(&include.path, &include.program) {
            diagnostics.push(CodegenDiagnostic {
                unit_path: Some(include.path.clone()),
                diagnostic,
            });
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let body = module.render_program(program).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| CodegenDiagnostic {
                unit_path: None,
                diagnostic,
            })
            .collect::<Vec<_>>()
    })?;
    let main_tail = if module.terminated {
        String::new()
    } else {
        format!(
            "  call void @{}()\n  ret i32 0\n",
            CoreRuntimeSymbol::Shutdown.symbol()
        )
    };

    Ok(format!(
        r#"target triple = "x86_64-pc-linux-gnu"

%EchoValue = type {{ i32, i64 }}

{}
{}

{}

define i32 @main() {{
entry:
{}{}
}}
"#,
        module.globals,
        runtime_declarations(),
        module.functions_ir,
        body,
        main_tail,
    ))
}

pub fn run_program_jit(program: &Program) -> Result<i32, Vec<Diagnostic>> {
    let ir = compile_to_ir(program)?;
    run_ir_jit(&ir)
}

pub fn run_mir_jit(program: &echo_mir::MirProgram) -> Result<i32, Vec<Diagnostic>> {
    let ir = compile_mir_to_ir(program)?;
    run_ir_jit(&ir)
}

impl IrModule {
    fn render_include_unit(
        &mut self,
        path: &str,
        program: &echo_mir::MirProgram,
    ) -> Result<(), Diagnostic> {
        let Some(function_name) = self.include_units.get(path).cloned() else {
            return Err(Diagnostic::new(
                format!("internal error: missing include unit for `{path}`"),
                echo_source::Span::new(0, 0),
            ));
        };

        if let Some(global_name) = self.include_once_globals.get(path) {
            self.globals
                .push_str(&format!("@{global_name} = internal global i1 false\n"));
        }

        let saved_source_dir = self.source_dir.clone();
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_returned = self.returned;
        let saved_terminated = self.terminated;
        let saved_break_labels = std::mem::take(&mut self.break_labels);
        let saved_continue_labels = std::mem::take(&mut self.continue_labels);
        let saved_break_value_slots = std::mem::take(&mut self.break_value_slots);

        self.source_dir = program.source_dir().map(str::to_string);
        self.returned = false;
        self.terminated = false;

        let mut body = String::new();
        for statement in program.statements() {
            if self.terminated {
                break;
            }
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                self.source_dir = saved_source_dir;
                self.locals = saved_locals;
                self.returned = saved_returned;
                self.terminated = saved_terminated;
                self.break_labels = saved_break_labels;
                self.continue_labels = saved_continue_labels;
                self.break_value_slots = saved_break_value_slots;
                return Err(diagnostic);
            }
        }

        if !self.terminated {
            body.push_str("  ret %EchoValue { i32 2, i64 1 }\n");
        }

        self.functions_ir.push_str(&format!(
            "\ndefine internal %EchoValue @{function_name}() {{\nentry:\n{body}}}\n"
        ));

        self.source_dir = saved_source_dir;
        self.locals = saved_locals;
        self.returned = saved_returned;
        self.terminated = saved_terminated;
        self.break_labels = saved_break_labels;
        self.continue_labels = saved_continue_labels;
        self.break_value_slots = saved_break_value_slots;

        Ok(())
    }

    fn render_program(
        &mut self,
        program: &echo_mir::MirProgram,
    ) -> Result<String, Vec<Diagnostic>> {
        let mut body = String::new();
        let mut diagnostics = Vec::new();

        for statement in program.imports() {
            if let Err(diagnostic) = self.register_std_import(statement) {
                diagnostics.push(diagnostic);
            }
        }

        for statement in program.functions() {
            self.functions
                .insert(statement.name.clone(), statement.clone());
        }

        for function in self
            .functions
            .clone()
            .into_values()
            .filter(|function| !function.name.contains("::"))
        {
            if let Err(diagnostic) = self.render_userland_function(&function) {
                diagnostics.push(diagnostic);
            }
        }

        self.render_reflection_registrations(&mut body);

        for statement in program.statements() {
            if self.terminated {
                break;
            }
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                diagnostics.push(diagnostic);
            }
        }

        if diagnostics.is_empty() {
            Ok(body)
        } else {
            Err(diagnostics)
        }
    }

    fn render_mir_expr_as_echo_value(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
    ) -> Result<String, Diagnostic> {
        let value = self.render_mir_expr(body, expr)?;
        Ok(self.runtime_value_as_echo_value(body, value))
    }

    fn render_mir_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirExpr::Null { .. } => {
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::Bool { value, .. } => Ok(RuntimeValue::EchoValue(format!(
                "{{ i32 1, i64 {} }}",
                *value as u8
            ))),
            echo_mir::MirExpr::String { value, .. } => {
                Ok(RuntimeValue::StaticString(value.clone()))
            }
            echo_mir::MirExpr::Number { source, value } => {
                if let Ok(value) = value.parse::<i64>() {
                    Ok(RuntimeValue::EchoValue(format!("{{ i32 2, i64 {value} }}")))
                } else if let Ok(value) = value.parse::<f64>() {
                    Ok(RuntimeValue::EchoValue(format!(
                        "{{ i32 11, i64 {} }}",
                        value.to_bits() as i64
                    )))
                } else {
                    Err(Diagnostic::new(
                        "unsupported numeric literal in LLVM codegen",
                        source.span(),
                    ))
                }
            }
            echo_mir::MirExpr::Variable { source, name } => self
                .locals
                .get(&self.resolve_alias(name))
                .cloned()
                .or_else(|| {
                    if name == "GLOBALS" {
                        Some(
                            self.render_mir_array_expr(body, &[])
                                .expect("empty array renders"),
                        )
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    Diagnostic::new(
                        format!("unsupported undefined variable `${name}` in LLVM codegen"),
                        source.span(),
                    )
                }),
            echo_mir::MirExpr::Constant { source, name } if name == "PHP_VERSION_ID" => {
                Ok(RuntimeValue::EchoValue("{ i32 2, i64 80200 }".to_string()))
            }
            echo_mir::MirExpr::Constant { name, .. } if name == "PHP_VERSION" => {
                Ok(RuntimeValue::StaticString("8.2.0".to_string()))
            }
            echo_mir::MirExpr::Constant { name, .. } if name == "PHP_SAPI" => {
                Ok(RuntimeValue::StaticString("cli".to_string()))
            }
            echo_mir::MirExpr::Constant { name, .. } if name == "PHP_EOL" => {
                Ok(RuntimeValue::StaticString("\n".to_string()))
            }
            echo_mir::MirExpr::Constant { name, .. } if name == "STDERR" => {
                Ok(RuntimeValue::StaticString("php://stderr".to_string()))
            }
            echo_mir::MirExpr::Constant { source, name } => Err(Diagnostic::new(
                format!("unsupported constant `{name}` in LLVM codegen"),
                source.span(),
            )),
            echo_mir::MirExpr::ReceiverConst { source, kind } => match kind {
                echo_ast::ReceiverConst::This => Ok(self.render_runtime_object_new_with_class(
                    body,
                    self.current_class.clone(),
                )),
                echo_ast::ReceiverConst::Static => Err(Diagnostic::new(
                    "$static is reserved for late static binding and is not implemented yet.",
                    source.span(),
                )),
                echo_ast::ReceiverConst::SelfType => Err(Diagnostic::new(
                    "$self receiver lowering is not implemented yet.",
                    source.span(),
                )),
                echo_ast::ReceiverConst::Parent => Err(Diagnostic::new(
                    "$parent receiver lowering is not implemented yet.",
                    source.span(),
                )),
            },
            echo_mir::MirExpr::StaticPropertyFetch {
                class_name,
                property,
                ..
            } => {
                let Some(global) = self.static_property_global_for_fetch(class_name, property)
                else {
                    return Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()));
                };
                let name = self.next_runtime_call_name();
                body.push_str(&format!("  {name} = load %EchoValue, ptr @{global}\n"));
                Ok(RuntimeValue::EchoValue(name))
            }
            echo_mir::MirExpr::StaticPropertyAssign {
                class_name,
                property,
                value,
                ..
            } => {
                let value = self.render_mir_expr_as_echo_value(body, value)?;
                let global =
                    self.ensure_static_property_global(&static_property_key(class_name, property));
                body.push_str(&format!("  store {value}, ptr @{global}\n"));
                Ok(RuntimeValue::EchoValue(
                    value.trim_start_matches("%EchoValue ").to_string(),
                ))
            }
            echo_mir::MirExpr::ClassConstantFetch {
                class_name,
                constant,
                ..
            } if constant == "class" => Ok(RuntimeValue::StaticString(class_name.as_string())),
            echo_mir::MirExpr::ClassConstantFetch {
                class_name,
                constant,
                ..
            } => Ok(RuntimeValue::StaticString(format!(
                "{}::{}",
                class_name.as_string(),
                constant
            ))),
            echo_mir::MirExpr::FunctionCall { call, .. } => {
                self.render_mir_function_call_expr(body, call)
            }
            echo_mir::MirExpr::DynamicFunctionCall {
                source, name, args, ..
            } => self.render_mir_dynamic_function_call_expr(body, source, name, args),
            echo_mir::MirExpr::DynamicCall { callee, args, .. } => {
                self.render_mir_expr_as_echo_value(body, callee)?;
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::MethodCall {
                source,
                object,
                method,
                method_span,
                args,
                ..
            } => self.render_mir_method_call_expr(body, source, object, method, *method_span, args),
            echo_mir::MirExpr::StaticCall {
                source,
                class_name,
                method,
                args,
            } if self
                .static_method_function_name(class_name, method)
                .is_some() =>
            {
                let name = self
                    .static_method_function_name(class_name, method)
                    .expect("guard checked static method function");
                if let Some(function) = self.functions.get(&name).cloned() {
                    self.render_userland_function(&function)?;
                }
                self.render_mir_userland_function_call_expr(
                    body,
                    &echo_mir::MirFunctionCall {
                        name,
                        args: args.clone(),
                        span: source.span(),
                    },
                )
            }
            echo_mir::MirExpr::StaticCall {
                class_name,
                method,
                args,
                ..
            } if class_name
                .parts
                .last()
                .is_some_and(|part| part == "Closure")
                && method == "bind"
                && !args
                    .first()
                    .is_some_and(|arg| matches!(arg.value, echo_mir::MirExpr::Closure { .. })) =>
            {
                Ok(self.render_runtime_object_new(body))
            }
            echo_mir::MirExpr::StaticCall {
                class_name,
                method,
                args,
                ..
            } if class_name
                .parts
                .last()
                .is_some_and(|part| part == "Closure")
                && method == "bind"
                && args
                    .first()
                    .is_some_and(|arg| matches!(arg.value, echo_mir::MirExpr::Closure { .. })) =>
            {
                let Some(first_arg) = args.first() else {
                    unreachable!("guard checked first closure argument")
                };
                let echo_mir::MirExpr::Closure { params, body, .. } = &first_arg.value else {
                    unreachable!("guard checked first closure argument")
                };
                Ok(RuntimeValue::Closure {
                    params: params.clone(),
                    body: body.clone(),
                })
            }
            echo_mir::MirExpr::StaticCall {
                class_name,
                method: _,
                args,
                ..
            } => {
                self.render_class_autoload_if_known(body, class_name)?;
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::New { target, args, .. } => {
                let class_name = match target {
                    echo_mir::MirNewTarget::Class(class_name) => {
                        self.render_class_autoload_if_known(body, class_name)?;
                        class_name.parts.last().cloned()
                    }
                    echo_mir::MirNewTarget::Expr(target) => {
                        let target = self.render_mir_expr_as_echo_value(body, target)?;
                        self.render_dynamic_class_autoload_if_known(body, &target)?;
                        None
                    }
                };
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(self.render_runtime_object_new_with_class(body, class_name))
            }
            echo_mir::MirExpr::Closure { source, .. } => Err(Diagnostic::new(
                "unsupported closure expression in LLVM codegen",
                source.span(),
            )),
            echo_mir::MirExpr::ArrowFunction { source, .. } => Err(Diagnostic::new(
                "unsupported arrow function expression in LLVM codegen",
                source.span(),
            )),
            echo_mir::MirExpr::Assign { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let resolved = self.resolve_alias(name);
                self.locals.insert(resolved, value.clone());
                Ok(value)
            }
            echo_mir::MirExpr::MagicDir { .. } => Ok(RuntimeValue::StaticString(
                self.source_dir.clone().unwrap_or_else(|| ".".to_string()),
            )),
            echo_mir::MirExpr::Include { kind, path, .. } => {
                if let echo_mir::MirExpr::String { value, .. } = path.as_ref()
                    && let Some(function_name) = self.include_units.get(value).cloned()
                {
                    let call_id = self.next_call_id;
                    self.next_call_id += 1;

                    if matches!(
                        kind,
                        echo_ast::IncludeKind::RequireOnce | echo_ast::IncludeKind::IncludeOnce
                    ) {
                        let Some(global_name) = self.include_once_globals.get(value).cloned()
                        else {
                            return Err(Diagnostic::new(
                                format!("internal error: missing include_once guard for `{value}`"),
                                path.syntax().span(),
                            ));
                        };
                        let loaded_name = format!("%include_once_loaded_value_{call_id}");
                        let call_name = format!("%include_once_call_{call_id}");
                        let result_name = format!("%include_once_result_{call_id}");
                        body.push_str(&format!("  {loaded_name} = load i1, ptr @{global_name}\n"));
                        body.push_str(&format!(
                            "  br i1 {loaded_name}, label %include_once_loaded_{call_id}, label %include_once_run_{call_id}\n"
                        ));
                        body.push_str(&format!("include_once_loaded_{call_id}:\n"));
                        body.push_str(&format!("  br label %include_once_done_{call_id}\n"));
                        body.push_str(&format!("include_once_run_{call_id}:\n"));
                        body.push_str(&format!("  store i1 true, ptr @{global_name}\n"));
                        body.push_str(&format!(
                            "  {call_name} = call %EchoValue @{function_name}()\n"
                        ));
                        body.push_str(&format!("  br label %include_once_done_{call_id}\n"));
                        body.push_str(&format!("include_once_done_{call_id}:\n"));
                        body.push_str(&format!(
                            "  {result_name} = phi %EchoValue [ {{ i32 2, i64 1 }}, %include_once_loaded_{call_id} ], [ {call_name}, %include_once_run_{call_id} ]\n"
                        ));
                        return Ok(RuntimeValue::EchoValue(result_name));
                    }

                    let name = format!("%include_call_{call_id}");
                    body.push_str(&format!("  {name} = call %EchoValue @{function_name}()\n"));
                    return Ok(RuntimeValue::EchoValue(name));
                }

                let path = self.render_mir_expr_as_echo_value(body, path)?;
                self.render_dynamic_include_expr(body, kind.clone(), &path)
            }
            echo_mir::MirExpr::Defer { body: block, .. } => {
                let function = self.render_mir_defer_function(body, block)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::TaskDefer.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirExpr::Run { source, expr } => self.render_mir_run_expr(body, source, expr),
            echo_mir::MirExpr::Join { handle, .. } => {
                self.render_mir_task_unary_expr(body, handle, CoreRuntimeSymbol::Join)
            }
            echo_mir::MirExpr::Loop { body: block, .. } => self.render_mir_loop_expr(body, block),
            echo_mir::MirExpr::Fork { source, expr } => {
                self.render_mir_fork_expr(body, source, expr)
            }
            echo_mir::MirExpr::Spawn { command, .. } => {
                self.render_mir_task_unary_expr(body, command, CoreRuntimeSymbol::ProcessSpawn)
            }
            echo_mir::MirExpr::Binary {
                source,
                left,
                op,
                right,
            } if *op == BinaryOp::Concat => self.render_mir_concat_expr(body, left, right),
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Add => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueAdd)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Sub => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueSub)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Mul => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueMul)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Div => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueDiv)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Mod => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueMod)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Pow => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValuePow)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if matches!(op, BinaryOp::Identical | BinaryOp::Equal) => self
                .render_mir_numeric_binary_expr(
                    body,
                    left,
                    right,
                    CoreRuntimeSymbol::ValueIdentical,
                ),
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if matches!(op, BinaryOp::NotIdentical | BinaryOp::NotEqual) => {
                let identical = self.render_mir_numeric_binary_expr(
                    body,
                    left,
                    right,
                    CoreRuntimeSymbol::ValueIdentical,
                )?;
                let identical = self.runtime_value_as_echo_value(body, identical);
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}({identical})\n",
                    CoreRuntimeSymbol::ValueNot.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Coalesce => {
                let left = self.render_mir_expr(body, left)?;
                self.render_mir_expr_as_echo_value(body, right)?;
                Ok(left)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::LessThan => self.render_mir_numeric_binary_expr(
                body,
                left,
                right,
                CoreRuntimeSymbol::ValueLessThan,
            ),
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::GreaterThanOrEqual => {
                let less_than = self.render_mir_numeric_binary_expr(
                    body,
                    left,
                    right,
                    CoreRuntimeSymbol::ValueLessThan,
                )?;
                let less_than = self.runtime_value_as_echo_value(body, less_than);
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}({less_than})\n",
                    CoreRuntimeSymbol::ValueNot.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Or => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueOr)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::InstanceOf => {
                self.render_mir_expr_as_echo_value(body, left)?;
                if !matches!(right.as_ref(), echo_mir::MirExpr::Constant { .. }) {
                    self.render_mir_expr_as_echo_value(body, right)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 2, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::Binary { source, op, .. } if *op == BinaryOp::And => Err(
                Diagnostic::new("unsupported && expression in LLVM codegen", source.span()),
            ),
            echo_mir::MirExpr::Ternary { source, .. } => Err(Diagnostic::new(
                "unsupported ternary expression in LLVM codegen",
                source.span(),
            )),
            echo_mir::MirExpr::Unary { op, expr, .. } => self.render_mir_numeric_unary_expr(
                body,
                expr,
                match op {
                    UnaryOp::Plus => CoreRuntimeSymbol::ValueUnaryPlus,
                    UnaryOp::Minus => CoreRuntimeSymbol::ValueUnaryMinus,
                    UnaryOp::Not => CoreRuntimeSymbol::ValueNot,
                },
            ),
            echo_mir::MirExpr::Cast { expr, .. } => self.render_mir_expr(body, expr),
            echo_mir::MirExpr::Field { object, field, .. } => {
                self.render_mir_field_expr(body, object, field)
            }
            echo_mir::MirExpr::Index {
                collection, index, ..
            } => self.render_mir_index_expr(body, collection, index),
            echo_mir::MirExpr::TargetAssign { target, value, .. } => match target.as_ref() {
                echo_mir::MirExpr::Field { object, field, .. } => {
                    self.render_mir_field_assign_expr(body, object, field, value)
                }
                _ => {
                    self.render_mir_expr_as_echo_value(body, target)?;
                    let value = self.render_mir_expr(body, value)?;
                    Ok(value)
                }
            },
            echo_mir::MirExpr::Object { fields, .. } => self.render_mir_object_expr(body, fields),
            echo_mir::MirExpr::List { values, .. } => self.render_mir_list_values(body, values),
            echo_mir::MirExpr::Array { elements, .. } => self.render_mir_array_expr(body, elements),
            echo_mir::MirExpr::Binary { source, .. } => Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                source.span(),
            )),
        }
    }

    pub(crate) fn render_runtime_object_new(&mut self, body: &mut String) -> RuntimeValue {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueObjectNew.symbol()
        ));
        RuntimeValue::EchoValue(format!("%runtime_call_{call_id}"))
    }

    pub(crate) fn render_runtime_object_new_with_class(
        &mut self,
        body: &mut String,
        class_name: Option<String>,
    ) -> RuntimeValue {
        let RuntimeValue::EchoValue(value) = self.render_runtime_object_new(body) else {
            unreachable!("runtime object constructor returns an EchoValue")
        };
        RuntimeValue::Object { value, class_name }
    }

    fn render_dynamic_include_expr(
        &mut self,
        body: &mut String,
        _kind: echo_ast::IncludeKind,
        path: &str,
    ) -> Result<RuntimeValue, Diagnostic> {
        let mut include_units = self
            .dynamic_include_units
            .iter()
            .map(|(path, function)| (path.clone(), function.clone()))
            .collect::<Vec<_>>();
        include_units.sort_by(|left, right| left.0.cmp(&right.0));

        if include_units.is_empty() {
            return Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()));
        }

        let dispatch_id = self.next_call_id;
        self.next_call_id += 1;
        let result_name = format!("%dynamic_include_result_{dispatch_id}");
        let fallback_label = format!("dynamic_include_fallback_{dispatch_id}");
        let done_label = format!("dynamic_include_done_{dispatch_id}");
        let mut incoming = Vec::new();

        for (index, (include_path, function_name)) in include_units.iter().enumerate() {
            let path_global = self.string_global(include_path);
            let compare_name = format!("%dynamic_include_is_match_{dispatch_id}_{index}");
            let match_label = format!("dynamic_include_match_{dispatch_id}_{index}");
            let next_label = if index + 1 == include_units.len() {
                fallback_label.clone()
            } else {
                format!("dynamic_include_check_{dispatch_id}_{}", index + 1)
            };

            if index > 0 {
                body.push_str(&format!("dynamic_include_check_{dispatch_id}_{index}:\n"));
            }
            body.push_str(&format!(
                "  {compare_name} = call i1 @{}({path}, ptr @{path_global}, i64 {})\n",
                CoreRuntimeSymbol::ValueStringEqualsPtr.symbol(),
                include_path.len()
            ));
            body.push_str(&format!(
                "  br i1 {compare_name}, label %{match_label}, label %{next_label}\n"
            ));
            body.push_str(&format!("{match_label}:\n"));
            let call_name = format!("%dynamic_include_call_{dispatch_id}_{index}");
            body.push_str(&format!(
                "  {call_name} = call %EchoValue @{function_name}()\n"
            ));
            body.push_str(&format!("  br label %{done_label}\n"));
            incoming.push((call_name, match_label));
        }

        body.push_str(&format!("{fallback_label}:\n"));
        let fallback_name = "{ i32 0, i64 0 }".to_string();
        body.push_str(&format!("  br label %{done_label}\n"));
        incoming.push((fallback_name, fallback_label));

        body.push_str(&format!("{done_label}:\n"));
        let incoming = incoming
            .into_iter()
            .map(|(value, label)| format!("[ {value}, %{label} ]"))
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!("  {result_name} = phi %EchoValue {incoming}\n"));

        Ok(RuntimeValue::EchoValue(result_name))
    }

    fn render_class_autoload_if_known(
        &mut self,
        body: &mut String,
        class_name: &echo_ast::QualifiedName,
    ) -> Result<(), Diagnostic> {
        let keys = [
            class_name.as_string(),
            class_name.parts.join("."),
            class_name.parts.last().cloned().unwrap_or_default(),
        ];
        let Some(path) = keys
            .iter()
            .find_map(|key| self.class_include_units.get(key).cloned())
        else {
            return Ok(());
        };
        let Some(function_name) = self.include_units.get(&path).cloned() else {
            return Err(Diagnostic::new(
                format!(
                    "internal error: missing include unit for autoload class `{}`",
                    keys[0]
                ),
                Span::new(0, 0),
            ));
        };
        let Some(global_name) = self.include_once_globals.get(&path).cloned() else {
            return Err(Diagnostic::new(
                format!(
                    "internal error: missing include_once guard for autoload class `{}`",
                    keys[0]
                ),
                Span::new(0, 0),
            ));
        };

        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let loaded_name = format!("%class_autoload_loaded_value_{call_id}");
        body.push_str(&format!("  {loaded_name} = load i1, ptr @{global_name}\n"));
        body.push_str(&format!(
            "  br i1 {loaded_name}, label %class_autoload_loaded_{call_id}, label %class_autoload_run_{call_id}\n"
        ));
        body.push_str(&format!("class_autoload_loaded_{call_id}:\n"));
        body.push_str(&format!("  br label %class_autoload_done_{call_id}\n"));
        body.push_str(&format!("class_autoload_run_{call_id}:\n"));
        body.push_str(&format!("  store i1 true, ptr @{global_name}\n"));
        body.push_str(&format!("  call %EchoValue @{function_name}()\n"));
        body.push_str(&format!("  br label %class_autoload_done_{call_id}\n"));
        body.push_str(&format!("class_autoload_done_{call_id}:\n"));

        Ok(())
    }

    fn render_dynamic_class_autoload_if_known(
        &mut self,
        body: &mut String,
        class_value: &str,
    ) -> Result<(), Diagnostic> {
        let mut class_units = self
            .class_include_units
            .iter()
            .map(|(class_name, path)| (class_name.clone(), path.clone()))
            .collect::<Vec<_>>();
        class_units.sort_by(|left, right| left.0.cmp(&right.0));

        if class_units.is_empty() {
            return Ok(());
        }

        let dispatch_id = self.next_call_id;
        self.next_call_id += 1;
        let done_label = format!("dynamic_class_autoload_done_{dispatch_id}");

        for (index, (class_name, path)) in class_units.iter().enumerate() {
            let Some(function_name) = self.include_units.get(path).cloned() else {
                return Err(Diagnostic::new(
                    format!(
                        "internal error: missing include unit for autoload class `{class_name}`"
                    ),
                    Span::new(0, 0),
                ));
            };
            let Some(global_name) = self.include_once_globals.get(path).cloned() else {
                return Err(Diagnostic::new(
                    format!(
                        "internal error: missing include_once guard for autoload class `{class_name}`"
                    ),
                    Span::new(0, 0),
                ));
            };
            let class_global = self.string_global(class_name);
            let compare_name = format!("%dynamic_class_autoload_is_match_{dispatch_id}_{index}");
            let match_label = format!("dynamic_class_autoload_match_{dispatch_id}_{index}");
            let loaded_label = format!("dynamic_class_autoload_loaded_{dispatch_id}_{index}");
            let run_label = format!("dynamic_class_autoload_run_{dispatch_id}_{index}");
            let next_label = if index + 1 == class_units.len() {
                done_label.clone()
            } else {
                format!("dynamic_class_autoload_check_{dispatch_id}_{}", index + 1)
            };

            if index > 0 {
                body.push_str(&format!(
                    "dynamic_class_autoload_check_{dispatch_id}_{index}:\n"
                ));
            }
            body.push_str(&format!(
                "  {compare_name} = call i1 @{}({class_value}, ptr @{class_global}, i64 {})\n",
                CoreRuntimeSymbol::ValueStringEqualsPtr.symbol(),
                class_name.len()
            ));
            body.push_str(&format!(
                "  br i1 {compare_name}, label %{match_label}, label %{next_label}\n"
            ));
            body.push_str(&format!("{match_label}:\n"));
            let loaded_name = format!("%dynamic_class_autoload_loaded_value_{dispatch_id}_{index}");
            body.push_str(&format!("  {loaded_name} = load i1, ptr @{global_name}\n"));
            body.push_str(&format!(
                "  br i1 {loaded_name}, label %{loaded_label}, label %{run_label}\n"
            ));
            body.push_str(&format!("{loaded_label}:\n"));
            body.push_str(&format!("  br label %{done_label}\n"));
            body.push_str(&format!("{run_label}:\n"));
            body.push_str(&format!("  store i1 true, ptr @{global_name}\n"));
            body.push_str(&format!("  call %EchoValue @{function_name}()\n"));
            body.push_str(&format!("  br label %{done_label}\n"));
        }

        body.push_str(&format!("{done_label}:\n"));

        Ok(())
    }

    fn render_mir_function_call_expr(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<RuntimeValue, Diagnostic> {
        let name = self.resolve_std_call_name(&call.name, call.span)?;

        if name == "empty" {
            return self.render_mir_empty_expr(body, call);
        }

        if name == "time.sleep" {
            self.mir_time_sleep_call(body, &call.args, call.span)?;
            return Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()));
        }

        if let Some(intrinsic) = std_intrinsic(&name) {
            return self.mir_std_intrinsic_call(body, intrinsic, &call.args, call.span);
        }

        let Some(builtin) = php_builtin(&name) else {
            return self.render_mir_userland_function_call_expr(body, call);
        };

        match builtin.codegen {
            BuiltinCodegen::ObStart => {
                let (symbol, arg) = match call.args.as_slice() {
                    [] => (builtin.symbol, None),
                    [arg] if matches!(&arg.value, echo_mir::MirExpr::Null { .. }) => (
                        builtin
                            .helper_symbol
                            .expect("ob_start value helper must be declared"),
                        Some("%EchoValue { i32 0, i64 0 }".to_string()),
                    ),
                    [arg] if matches!(&arg.value, echo_mir::MirExpr::String { .. }) => {
                        let echo_mir::MirExpr::String { value, .. } = &arg.value else {
                            unreachable!("matches! checked string argument")
                        };
                        let helper = builtin
                            .helper_symbol
                            .expect("ob_start value helper must be declared");
                        let global = self.string_global(value);
                        let value_id = self.next_call_id;
                        self.next_call_id += 1;
                        let value_name = format!("%runtime_call_{value_id}");

                        body.push_str(&format!(
                            "  {value_name} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
                            CoreRuntimeSymbol::ValueString.symbol(),
                            value.len()
                        ));

                        (helper, Some(format!("%EchoValue {value_name}")))
                    }
                    [arg] => {
                        return Err(Diagnostic::new(
                            "unsupported ob_start callback argument in LLVM codegen",
                            arg.syntax().span(),
                        ));
                    }
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported ob_start argument count in LLVM codegen",
                            call.span,
                        ));
                    }
                };

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let bool_name = format!("%runtime_bool_{call_id}");
                let payload_name = format!("%runtime_bool_payload_{call_id}");
                let value_name = format!("%runtime_call_{call_id}");

                match arg {
                    Some(arg) => {
                        body.push_str(&format!("  {bool_name} = call i1 @{symbol}({arg})\n"))
                    }
                    None => body.push_str(&format!("  {bool_name} = call i1 @{symbol}()\n")),
                }
                body.push_str(&format!("  {payload_name} = zext i1 {bool_name} to i64\n"));
                body.push_str(&format!(
                    "  {value_name} = insertvalue %EchoValue {{ i32 1, i64 0 }}, i64 {payload_name}, 1\n"
                ));

                Ok(RuntimeValue::EchoValue(value_name))
            }
            BuiltinCodegen::VoidStatement => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                body.push_str(&format!("  call void @{}()\n", builtin.symbol));
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            BuiltinCodegen::VoidUnaryStatement => {
                let [arg] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let arg = self.render_mir_expr_as_echo_value(body, arg)?;
                body.push_str(&format!("  call void @{}({arg})\n", builtin.symbol));
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            BuiltinCodegen::BoolStatement => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let bool_name = format!("%runtime_bool_{call_id}");
                let payload_name = format!("%runtime_bool_payload_{call_id}");
                let value_name = format!("%runtime_call_{call_id}");

                body.push_str(&format!("  {bool_name} = call i1 @{}()\n", builtin.symbol));
                body.push_str(&format!("  {payload_name} = zext i1 {bool_name} to i64\n"));
                body.push_str(&format!(
                    "  {value_name} = insertvalue %EchoValue {{ i32 1, i64 0 }}, i64 {payload_name}, 1\n"
                ));

                Ok(RuntimeValue::EchoValue(value_name))
            }
            BuiltinCodegen::ValueExpression => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}()\n",
                    builtin.symbol
                ));
                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryExpression => {
                let [arg] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let arg = self.render_mir_expr_as_echo_value(body, arg)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({arg})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Getenv => {
                if call.args.len() > 2 {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let name_arg = if let Some(arg) = call.args.first() {
                    self.render_mir_expr_as_echo_value(body, arg)?
                } else {
                    "%EchoValue { i32 0, i64 0 }".to_string()
                };
                let local_only_arg = if let Some(arg) = call.args.get(1) {
                    self.render_mir_expr_as_echo_value(body, arg)?
                } else {
                    "%EchoValue { i32 1, i64 0 }".to_string()
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({name_arg}, {local_only_arg})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Basename => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let path = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let suffix = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString(String::new()),
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({path}, {suffix})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Dirname => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let path = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let levels = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({path}, {levels})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueBinaryExpression => {
                let [left, right] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let left = self.render_mir_expr_as_echo_value(body, left)?;
                let right = self.render_mir_expr_as_echo_value(body, right)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({left}, {right})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryOptionalBoolExpression => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let flag = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {flag})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Nl2br => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let use_xhtml = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {use_xhtml})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryOptionalBoolContextExpression => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let flag = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let context = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {flag}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryOptionalContextExpression => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let context = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueBinaryOptionalContextExpression => {
                if !(2..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let left = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let right = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let context = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({left}, {right}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::FileGetContents => {
                if !(1..=5).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let filename = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let use_include_path = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let context = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let offset = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 0 }".to_string(),
                };
                let length = match call.args.get(4) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({filename}, {use_include_path}, {context}, {offset}, {length})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::FilePutContents => {
                if !(2..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let filename = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let data = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let flags = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 0 }".to_string(),
                };
                let context = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({filename}, {data}, {flags}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Uniqid => {
                if call.args.len() > 2 {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let prefix = match call.args.first() {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString(String::new()),
                    ),
                };
                let more_entropy = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({prefix}, {more_entropy})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Mkdir => {
                if !(1..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let directory = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let permissions = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 511 }".to_string(),
                };
                let recursive = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let context = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({directory}, {permissions}, {recursive}, {context})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Touch => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let filename = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let mtime = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let atime = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({filename}, {mtime}, {atime})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArrayKeys => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let array = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let filter_value = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 6, i64 0 }".to_string(),
                };
                let strict = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({array}, {filter_value}, {strict})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArrayReverse => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let array = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let preserve_keys = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({array}, {preserve_keys})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArraySlice => {
                if !(2..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let array = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let offset = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let length = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let preserve_keys = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({array}, {offset}, {length}, {preserve_keys})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArrayMerge | BuiltinCodegen::ArrayReplace => {
                if builtin.codegen == BuiltinCodegen::ArrayReplace && call.args.is_empty() {
                    return Err(Diagnostic::new(
                        "unsupported argument count for builtin `array_replace` in LLVM codegen",
                        call.span,
                    ));
                }

                let arrays = self.render_mir_call_args_as_array(body, &call.args)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}(%EchoValue {arrays})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::InArray => {
                if !(2..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let needle = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let haystack = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let strict = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({needle}, {haystack}, {strict})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Log => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let base = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => format!(
                        "%EchoValue {{ i32 11, i64 {} }}",
                        std::f64::consts::E.to_bits() as i64
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {base})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ChunkSplit => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 76 }".to_string(),
                };
                let separator = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString("\r\n".to_string()),
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length}, {separator})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::StrPad => {
                if !(2..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let pad_string = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString(" ".to_string()),
                    ),
                };
                let pad_type = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length}, {pad_string}, {pad_type})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::StrSplit => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueTernaryExpression => {
                let [first, second, third] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let first = self.render_mir_expr_as_echo_value(body, first)?;
                let second = self.render_mir_expr_as_echo_value(body, second)?;
                let third = self.render_mir_expr_as_echo_value(body, third)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({first}, {second}, {third})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Explode => {
                if !(2..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let separator = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let string = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let limit = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 9223372036854775807 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({separator}, {string}, {limit})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Implode => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let (separator, array) = match call.args.as_slice() {
                    [array] => (
                        self.runtime_value_as_echo_value(
                            body,
                            RuntimeValue::StaticString(String::new()),
                        ),
                        self.render_mir_expr_as_echo_value(body, array)?,
                    ),
                    [separator, array] => (
                        self.render_mir_expr_as_echo_value(body, separator)?,
                        self.render_mir_expr_as_echo_value(body, array)?,
                    ),
                    _ => unreachable!("argument count checked above"),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({separator}, {array})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::SubstrCompare => {
                if !(3..=5).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let haystack = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let needle = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let offset = self.render_mir_expr_as_echo_value(body, &call.args[2])?;
                let length = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let case_insensitive = match call.args.get(4) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({haystack}, {needle}, {offset}, {length}, {case_insensitive})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
        }
    }

    fn render_mir_empty_expr(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<RuntimeValue, Diagnostic> {
        let [value] = call.args.as_slice() else {
            return Err(Diagnostic::new(
                "unsupported empty argument count in LLVM codegen",
                call.span,
            ));
        };

        self.render_mir_expr_as_echo_value(body, value)?;

        Ok(RuntimeValue::EchoValue("{ i32 1, i64 1 }".to_string()))
    }
}

fn static_property_key(class_name: &echo_ast::QualifiedName, property: &str) -> String {
    format!("{}::${property}", class_name.as_string())
}

impl IrModule {
    fn index_class_parents(&mut self, statements: &[echo_mir::MirStmt]) {
        for statement in statements {
            if let echo_ast::Stmt::ClassDecl(class_decl) = statement.syntax() {
                if let Some(parent) = &class_decl.parent
                    && let Some(parent_name) = parent.parts.last()
                {
                    self.class_parents
                        .insert(class_decl.name.clone(), parent_name.clone());
                }

                let mut traits = Vec::new();
                for member in &class_decl.members {
                    if let echo_ast::ClassMember::TraitUse(name) = member {
                        traits.push(name.as_string());
                        if let Some(short_name) = name.parts.last()
                            && short_name != &name.as_string()
                        {
                            traits.push(short_name.clone());
                        }
                        continue;
                    }

                    let echo_ast::ClassMember::Method(method) = member else {
                        continue;
                    };
                    for param in &method.params {
                        if param.promoted_visibility.is_some()
                            && let Some(ty) = &param.ty
                        {
                            self.property_types.insert(
                                format!("{}::${}", class_decl.name, param.name),
                                ty.clone(),
                            );
                        }
                    }
                }
                if !traits.is_empty() {
                    self.class_traits.insert(class_decl.name.clone(), traits);
                }
            }
        }
    }

    fn ensure_static_property_global(&mut self, key: &str) -> String {
        if let Some(global) = self.static_property_globals.get(key) {
            return global.clone();
        }

        let id = self.next_static_property_id;
        self.next_static_property_id += 1;
        let global = format!("echo_static_property_{id}");
        self.globals.push_str(&format!(
            "@{global} = internal global %EchoValue {{ i32 0, i64 0 }}\n"
        ));
        self.static_property_globals
            .insert(key.to_string(), global.clone());
        global
    }

    fn static_property_global_for_fetch(
        &self,
        class_name: &echo_ast::QualifiedName,
        property: &str,
    ) -> Option<String> {
        let full = static_property_key(class_name, property);
        if let Some(global) = self.static_property_globals.get(&full) {
            return Some(global.clone());
        }

        let short = format!("{}::${property}", class_name.parts.last()?);
        self.static_property_globals.get(&short).cloned()
    }

    fn static_method_function_name(
        &self,
        class_name: &echo_ast::QualifiedName,
        method: &str,
    ) -> Option<String> {
        let mut candidates = vec![class_name.as_string(), class_name.parts.last()?.clone()];
        let mut seen = std::collections::HashSet::new();
        while let Some(candidate_class) = candidates.pop() {
            if !seen.insert(candidate_class.clone()) {
                continue;
            }
            let candidate = format!("{candidate_class}::{method}");
            if self.functions.contains_key(&candidate) {
                return Some(candidate);
            }
            if let Some(parent) = self.class_parents.get(&candidate_class) {
                candidates.push(parent.clone());
            }
        }
        None
    }
}

fn runtime_declarations() -> String {
    let mut seen = HashSet::new();

    CoreRuntimeSymbol::ALL
        .iter()
        .map(|function| function.llvm_decl())
        .chain(
            PHP_RUNTIME_HELPERS
                .iter()
                .map(|(symbol, signature)| signature.llvm_decl(symbol)),
        )
        .chain(PHP_BUILTINS.iter().map(|builtin| builtin.llvm_decl()))
        .chain(STD_INTRINSICS.iter().map(|intrinsic| intrinsic.llvm_decl()))
        .filter(|declaration| seen.insert(declaration.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
