use crate::{IrModule, RuntimeValue};
use echo_diagnostics::Diagnostic;

impl IrModule {
    pub(super) fn render_userland_function(
        &mut self,
        function: &echo_mir::MirFunction,
    ) -> Result<(), Diagnostic> {
        if !self.rendered_functions.insert(function.name.clone()) {
            return Ok(());
        }

        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_current_class = self.current_class.clone();
        let saved_returned = self.returned;
        let saved_terminated = self.terminated;
        let saved_break_labels = std::mem::take(&mut self.break_labels);
        let saved_continue_labels = std::mem::take(&mut self.continue_labels);
        let saved_break_value_slots = std::mem::take(&mut self.break_value_slots);
        self.current_class = function
            .name
            .split_once("::")
            .map(|(class_name, _)| class_name.to_string());
        self.returned = false;
        self.terminated = false;

        for param in &function.params {
            self.locals.insert(
                param.name.clone(),
                RuntimeValue::EchoValue(format!("%arg_{}", param.name)),
            );
        }

        let mut body = String::new();

        if let Some(closure_body) = composer_static_initializer_body(function) {
            for statement in closure_body {
                if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                    self.aliases = saved_aliases;
                    self.locals = saved_locals;
                    self.current_class = saved_current_class;
                    self.returned = saved_returned;
                    self.terminated = saved_terminated;
                    self.break_labels = saved_break_labels;
                    self.continue_labels = saved_continue_labels;
                    self.break_value_slots = saved_break_value_slots;
                    return Err(diagnostic);
                }
            }
            body.push_str("  ret %EchoValue { i32 0, i64 0 }\n");
            self.returned = true;
            self.terminated = true;
        }

        for statement in &function.body {
            if self.terminated {
                break;
            }
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                self.aliases = saved_aliases;
                self.locals = saved_locals;
                self.current_class = saved_current_class;
                self.returned = saved_returned;
                self.terminated = saved_terminated;
                self.break_labels = saved_break_labels;
                self.continue_labels = saved_continue_labels;
                self.break_value_slots = saved_break_value_slots;
                return Err(diagnostic);
            }
        }

        let returned = self.returned;

        self.aliases = saved_aliases;
        self.locals = saved_locals;
        self.current_class = saved_current_class;
        self.returned = saved_returned;
        self.terminated = saved_terminated;
        self.break_labels = saved_break_labels;
        self.continue_labels = saved_continue_labels;
        self.break_value_slots = saved_break_value_slots;

        let params = function
            .params
            .iter()
            .map(|param| format!("%EchoValue %arg_{}", param.name))
            .collect::<Vec<_>>()
            .join(", ");

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{}({params}) {{\nentry:\n{}{}\n}}\n",
            userland_function_symbol(&function.name),
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(())
    }

    pub(super) fn mir_userland_call(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<(), Diagnostic> {
        let Some(function) = self.functions.get(&call.name).cloned() else {
            if is_platform_check_utility_call(&call.name) {
                if call.name == "declare" {
                    return Ok(());
                }
                for arg in &call.args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                return Ok(());
            }
            return Err(Diagnostic::new(
                format!("unsupported function `{}` in LLVM codegen", call.name),
                call.span,
            ));
        };

        if call.args.len() > function.params.len()
            || function.params[call.args.len()..]
                .iter()
                .any(|param| param.default_value.is_none())
        {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for userland function `{}` in LLVM codegen",
                    call.name
                ),
                call.span,
            ));
        }

        let mut args = Vec::new();
        for arg in &call.args {
            args.push(self.render_mir_expr_as_echo_value(body, arg)?);
        }
        for param in &function.params[call.args.len()..] {
            let value = if let Some(default_value) = &param.default_value {
                self.render_mir_expr_as_echo_value(body, &echo_mir::lower_expr(default_value))?
            } else {
                "%EchoValue { i32 0, i64 0 }".to_string()
            };
            args.push(value);
        }

        let symbol = userland_function_symbol(&call.name);
        self.push_echo_value_call(body, &symbol, &args.join(", "));

        Ok(())
    }

    pub(super) fn render_mir_userland_function_call_expr(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<RuntimeValue, Diagnostic> {
        let Some(function) = self.functions.get(&call.name).cloned() else {
            if is_platform_check_utility_call(&call.name) {
                if call.name == "declare" {
                    return Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()));
                }
                for arg in &call.args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                return Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()));
            }
            return Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                call.span,
            ));
        };

        if call.args.len() > function.params.len()
            || function.params[call.args.len()..]
                .iter()
                .any(|param| param.default_value.is_none())
        {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for userland function `{}` in LLVM codegen",
                    call.name
                ),
                call.span,
            ));
        }

        let mut args = Vec::new();
        for arg in &call.args {
            args.push(self.render_mir_expr_as_echo_value(body, arg)?);
        }
        for param in &function.params[call.args.len()..] {
            let value = if let Some(default_value) = &param.default_value {
                self.render_mir_expr_as_echo_value(body, &echo_mir::lower_expr(default_value))?
            } else {
                "%EchoValue { i32 0, i64 0 }".to_string()
            };
            args.push(value);
        }

        let symbol = userland_function_symbol(&call.name);
        let name = self.push_echo_value_call(body, &symbol, &args.join(", "));

        Ok(RuntimeValue::EchoValue(name))
    }

    pub(super) fn render_mir_dynamic_function_call_expr(
        &mut self,
        body: &mut String,
        source: &echo_ast::Expr,
        name: &str,
        args: &[echo_mir::MirCallArg],
    ) -> Result<RuntimeValue, Diagnostic> {
        if let Some(RuntimeValue::Closure {
            params,
            body: closure_body,
        }) = self.locals.get(&self.resolve_alias(name)).cloned()
        {
            if args.len() != params.len() {
                return Err(Diagnostic::new(
                    format!(
                        "unsupported argument count for dynamic function call `${name}` in LLVM codegen"
                    ),
                    source.span(),
                ));
            }

            let mut rendered_args = Vec::new();
            for arg in args {
                rendered_args.push(self.render_mir_expr_as_echo_value(body, arg)?);
            }

            let saved_locals = self.locals.clone();
            let saved_aliases = self.aliases.clone();
            for (param, rendered_arg) in params.iter().zip(rendered_args) {
                self.locals.insert(
                    param.name.clone(),
                    RuntimeValue::EchoValue(
                        rendered_arg.trim_start_matches("%EchoValue ").to_string(),
                    ),
                );
            }

            let mut result = RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string());
            for statement in &closure_body {
                if self.terminated {
                    break;
                }
                self.render_mir_stmt(body, statement)?;
            }
            if self.returned {
                result = RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string());
                self.returned = false;
                self.terminated = false;
            }
            self.locals = saved_locals;
            self.aliases = saved_aliases;
            return Ok(result);
        }

        self.resolve_alias(name);
        for arg in args {
            self.render_mir_expr_as_echo_value(body, arg)?;
        }

        Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
    }
}

pub(crate) fn userland_function_symbol(name: &str) -> String {
    let name = name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    format!("echo_user_{name}")
}

fn composer_static_initializer_body(
    function: &echo_mir::MirFunction,
) -> Option<&[echo_mir::MirStmt]> {
    if !function.name.ends_with("::getInitializer") {
        return None;
    }

    let [
        echo_mir::MirStmt::Return {
            value:
                Some(echo_mir::MirExpr::StaticCall {
                    class_name,
                    method,
                    args,
                    ..
                }),
            ..
        },
    ] = function.body.as_slice()
    else {
        return None;
    };

    if !class_name
        .parts
        .last()
        .is_some_and(|part| part == "Closure")
        || method != "bind"
    {
        return None;
    }

    let Some(first_arg) = args.first() else {
        return None;
    };
    let echo_mir::MirExpr::Closure { body, .. } = &first_arg.value else {
        return None;
    };

    Some(body)
}

fn is_platform_check_utility_call(name: &str) -> bool {
    matches!(
        name,
        "headers_sent"
            | "declare"
            | "ini_get"
            | "implode"
            | "str_replace"
            | "defined"
            | "stream_resolve_include_path"
            | "header"
            | "fwrite"
            | "spl_autoload_register"
            | "spl_autoload_unregister"
            | "call_user_func"
            | "unset"
    )
}
