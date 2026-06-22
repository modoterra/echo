use crate::abi::{BuiltinLowering, CoreRuntimeSymbol, php_builtin, std_intrinsic};
use crate::{IrModule, RuntimeValue, stmt_span};
use echo_ast::Stmt;
use echo_diagnostics::Diagnostic;

impl IrModule {
    pub(super) fn render_mir_stmt(
        &mut self,
        body: &mut String,
        statement: &echo_mir::MirStmt,
    ) -> Result<(), Diagnostic> {
        match statement {
            echo_mir::MirStmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.render_mir_expr(body, expr)?;
                    self.write_value(body, value);
                }
                Ok(())
            }
            echo_mir::MirStmt::FunctionCall { call, .. } => {
                self.render_mir_function_call_stmt(body, call)
            }
            echo_mir::MirStmt::DynamicFunctionCall { source, name, args } => {
                self.render_mir_dynamic_function_call(body, source, name, args)
            }
            echo_mir::MirStmt::Assign { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let name = self.resolve_alias(name);
                self.locals.insert(name, value);
                Ok(())
            }
            echo_mir::MirStmt::Let { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let name = self.resolve_alias(name);
                self.locals.insert(name, value);
                Ok(())
            }
            echo_mir::MirStmt::Return { value, .. } => {
                let value = self.render_mir_expr_as_echo_value(body, value)?;
                body.push_str(&format!("  ret {value}\n"));
                self.returned = true;
                self.terminated = true;
                Ok(())
            }
            echo_mir::MirStmt::Expr { expr, .. } => {
                self.render_mir_expr(body, expr)?;
                Ok(())
            }
            echo_mir::MirStmt::Loop {
                body: loop_body, ..
            } => self.render_mir_loop_stmt(body, loop_body),
            echo_mir::MirStmt::If {
                condition,
                body: if_body,
                ..
            } => self.render_mir_if_stmt(body, condition, if_body),
            echo_mir::MirStmt::Break { source, value } => {
                self.render_mir_break_stmt(body, source, value.as_ref())
            }
            echo_mir::MirStmt::Append {
                source,
                target,
                value,
            } => self.render_mir_append_stmt(body, source, target, value),
            echo_mir::MirStmt::AssignRef {
                source,
                name,
                target,
            } => self.render_mir_assign_ref_stmt(source, name, target),
            echo_mir::MirStmt::Yield { source, .. } => Err(Diagnostic::new(
                "unsupported yield statement in LLVM codegen",
                stmt_span(source),
            )),
            echo_mir::MirStmt::Noop { .. } => Ok(()),
        }
    }

    fn render_mir_function_call_stmt(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<(), Diagnostic> {
        let name = self.resolve_std_call_name(&call.name, call.span)?;
        if name == "time.sleep" {
            self.mir_time_sleep_call(body, &call.args, call.span)?;
        } else if let Some(intrinsic) = std_intrinsic(&name) {
            self.mir_std_intrinsic_call(body, intrinsic, &call.args, call.span)?;
        } else {
            match php_builtin(&name) {
                Some(builtin) if builtin.lowering == BuiltinLowering::DirectRuntimeCall => {
                    self.mir_php_builtin_call(body, builtin, &call.args, call.span)?
                }
                None => self.mir_userland_call(body, call)?,
                Some(_) => self.mir_userland_call(body, call)?,
            }
        }

        Ok(())
    }

    fn render_mir_assign_ref_stmt(
        &mut self,
        source: &Stmt,
        name: &str,
        target: &str,
    ) -> Result<(), Diagnostic> {
        let resolved_target = self.resolve_alias(target);
        if self.locals.contains_key(&resolved_target) {
            self.aliases.insert(name.to_string(), resolved_target);
            Ok(())
        } else {
            Err(Diagnostic::new(
                format!("unsupported reference to undefined variable `${target}` in LLVM codegen"),
                stmt_span(source),
            ))
        }
    }

    fn render_mir_dynamic_function_call(
        &mut self,
        body: &mut String,
        source: &Stmt,
        name: &str,
        args: &[echo_mir::MirExpr],
    ) -> Result<(), Diagnostic> {
        if !args.is_empty() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported arguments for dynamic function call `${name}` in LLVM codegen"
                ),
                stmt_span(source),
            ));
        }

        let RuntimeValue::StaticString(function_name) = self
            .locals
            .get(&self.resolve_alias(name))
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    format!("unsupported undefined dynamic function `${name}` in LLVM codegen"),
                    stmt_span(source),
                )
            })?
        else {
            return Err(Diagnostic::new(
                format!("unsupported non-string dynamic function `${name}` in LLVM codegen"),
                stmt_span(source),
            ));
        };

        let global = self.string_global(&function_name);
        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
            CoreRuntimeSymbol::CallFunction.symbol(),
            function_name.len()
        ));

        Ok(())
    }
}
