use crate::{IrModule, RuntimeValue};
use echo_diagnostics::Diagnostic;

impl IrModule {
    pub(super) fn render_userland_function(
        &mut self,
        function: &echo_mir::MirFunction,
    ) -> Result<(), Diagnostic> {
        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_returned = self.returned;
        let saved_terminated = self.terminated;
        self.returned = false;
        self.terminated = false;

        for param in &function.params {
            self.locals.insert(
                param.name.clone(),
                RuntimeValue::EchoValue(format!("%arg_{}", param.name)),
            );
        }

        let mut body = String::new();

        for statement in &function.body {
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                self.aliases = saved_aliases;
                self.locals = saved_locals;
                self.returned = saved_returned;
                self.terminated = saved_terminated;
                return Err(diagnostic);
            }
        }

        let returned = self.returned;

        self.aliases = saved_aliases;
        self.locals = saved_locals;
        self.returned = saved_returned;
        self.terminated = saved_terminated;

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
            return Err(Diagnostic::new(
                format!("unsupported function `{}` in LLVM codegen", call.name),
                call.span,
            ));
        };

        if call.args.len() != function.params.len() {
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
            return Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                call.span,
            ));
        };

        if call.args.len() != function.params.len() {
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

        let symbol = userland_function_symbol(&call.name);
        let name = self.push_echo_value_call(body, &symbol, &args.join(", "));

        Ok(RuntimeValue::EchoValue(name))
    }
}

fn userland_function_symbol(name: &str) -> String {
    format!("echo_user_{name}")
}
