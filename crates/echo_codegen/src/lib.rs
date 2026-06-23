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
#[cfg(test)]
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

pub fn compile_hir_to_ir(program: &echo_hir::HirProgram) -> Result<String, Vec<Diagnostic>> {
    let mir = echo_mir::lower_program(program)?;

    compile_mir_to_ir(&mir)
}

pub fn compile_mir_to_ir(program: &echo_mir::MirProgram) -> Result<String, Vec<Diagnostic>> {
    let mut module = IrModule::new();
    module.source_dir = program.source_dir().map(str::to_string);
    let body = module.render_program(program)?;

    Ok(format!(
        r#"target triple = "x86_64-pc-linux-gnu"

%EchoValue = type {{ i32, i64 }}

{}
{}

{}

define i32 @main() {{
entry:
{}  call void @{}()
  ret i32 0
}}
"#,
        module.globals,
        runtime_declarations(),
        module.functions_ir,
        body,
        CoreRuntimeSymbol::Shutdown.symbol(),
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

        for function in self.functions.clone().into_values() {
            if let Err(diagnostic) = self.render_userland_function(&function) {
                diagnostics.push(diagnostic);
            }
        }

        self.render_reflection_registrations(&mut body);

        for statement in program.statements() {
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
                .ok_or_else(|| {
                    Diagnostic::new(
                        format!("unsupported undefined variable `${name}` in LLVM codegen"),
                        source.span(),
                    )
                }),
            echo_mir::MirExpr::FunctionCall { call, .. } => {
                self.render_mir_function_call_expr(body, call)
            }
            echo_mir::MirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } => self.render_mir_method_call_expr(body, object, method, args),
            echo_mir::MirExpr::StaticCall { args, .. } => {
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::Assign { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let resolved = self.resolve_alias(name);
                self.locals.insert(resolved, value.clone());
                Ok(value)
            }
            echo_mir::MirExpr::MagicDir { .. } => Ok(RuntimeValue::StaticString(
                self.source_dir.clone().unwrap_or_else(|| ".".to_string()),
            )),
            echo_mir::MirExpr::Require { once, path, .. } => {
                let path = self.render_mir_expr_as_echo_value(body, path)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let symbol = if *once {
                    CoreRuntimeSymbol::RequireOnce
                } else {
                    CoreRuntimeSymbol::Require
                };
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}({path})\n",
                    symbol.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
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
            echo_mir::MirExpr::Unary { op, expr, .. } => self.render_mir_numeric_unary_expr(
                body,
                expr,
                match op {
                    UnaryOp::Plus => CoreRuntimeSymbol::ValueUnaryPlus,
                    UnaryOp::Minus => CoreRuntimeSymbol::ValueUnaryMinus,
                },
            ),
            echo_mir::MirExpr::Field { object, field, .. } => {
                self.render_mir_field_expr(body, object, field)
            }
            echo_mir::MirExpr::Index {
                collection, index, ..
            } => self.render_mir_index_expr(body, collection, index),
            echo_mir::MirExpr::Object { fields, .. } => self.render_mir_object_expr(body, fields),
            echo_mir::MirExpr::List { values, .. } => self.render_mir_list_values(body, values),
            echo_mir::MirExpr::Array { elements, .. } => self.render_mir_array_expr(body, elements),
            echo_mir::MirExpr::Binary { source, .. } => Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                source.span(),
            )),
        }
    }

    fn render_mir_function_call_expr(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<RuntimeValue, Diagnostic> {
        let name = self.resolve_std_call_name(&call.name, call.span)?;

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
                    [echo_mir::MirExpr::Null { .. }] => (
                        builtin
                            .helper_symbol
                            .expect("ob_start value helper must be declared"),
                        Some("%EchoValue { i32 0, i64 0 }".to_string()),
                    ),
                    [echo_mir::MirExpr::String { value, .. }] => {
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
