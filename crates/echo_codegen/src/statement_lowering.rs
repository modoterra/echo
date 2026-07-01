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
            echo_mir::MirStmt::CoalesceAssign { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let name = self.resolve_alias(name);
                self.locals.insert(name, value);
                Ok(())
            }
            echo_mir::MirStmt::ListAssign { targets, value, .. } => {
                self.render_mir_expr_as_echo_value(body, value)?;
                for target in targets {
                    let target = self.resolve_alias(target);
                    self.locals.insert(
                        target,
                        RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()),
                    );
                }
                Ok(())
            }
            echo_mir::MirStmt::Let { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let name = self.resolve_alias(name);
                self.locals.insert(name, value);
                Ok(())
            }
            echo_mir::MirStmt::Return { value, .. } => {
                let value = if let Some(value) = value {
                    self.render_mir_expr_as_echo_value(body, value)?
                } else {
                    "{ i32 0, i64 0 }".to_string()
                };
                body.push_str(&format!("  ret {value}\n"));
                self.returned = true;
                self.terminated = true;
                Ok(())
            }
            echo_mir::MirStmt::Throw { value, .. } => {
                self.render_mir_expr_as_echo_value(body, value)?;
                Ok(())
            }
            echo_mir::MirStmt::Goto { source, .. } => Err(Diagnostic::new(
                "unsupported goto statement in LLVM codegen",
                stmt_span(source),
            )),
            echo_mir::MirStmt::Label { .. } => Ok(()),
            echo_mir::MirStmt::PhpDeclare {
                source,
                body: declare_body,
            } => {
                if declare_body.is_empty() {
                    Ok(())
                } else {
                    Err(Diagnostic::new(
                        "unsupported declare block in LLVM codegen",
                        stmt_span(source),
                    ))
                }
            }
            echo_mir::MirStmt::PhpExit { value, .. } => self.render_mir_php_exit_stmt(body, value),
            echo_mir::MirStmt::Expr { expr, .. } => {
                self.render_mir_expr(body, expr)?;
                Ok(())
            }
            echo_mir::MirStmt::Loop {
                body: loop_body, ..
            } => self.render_mir_loop_stmt(body, loop_body),
            echo_mir::MirStmt::While {
                source, condition, ..
            } => {
                self.render_mir_expr_as_echo_value(body, condition)?;
                Err(Diagnostic::new(
                    "unsupported while statement in LLVM codegen",
                    stmt_span(source),
                ))
            }
            echo_mir::MirStmt::DoWhile {
                source, condition, ..
            } => {
                self.render_mir_expr_as_echo_value(body, condition)?;
                Err(Diagnostic::new(
                    "unsupported do-while statement in LLVM codegen",
                    stmt_span(source),
                ))
            }
            echo_mir::MirStmt::For {
                source,
                init,
                conditions,
                increments,
                ..
            } => {
                for expr in init {
                    self.render_mir_expr_as_echo_value(body, expr)?;
                }
                for expr in conditions {
                    self.render_mir_expr_as_echo_value(body, expr)?;
                }
                for expr in increments {
                    self.render_mir_expr_as_echo_value(body, expr)?;
                }
                Err(Diagnostic::new(
                    "unsupported for statement in LLVM codegen",
                    stmt_span(source),
                ))
            }
            echo_mir::MirStmt::Foreach {
                iterable,
                key,
                value,
                body: foreach_body,
                ..
            } => self.render_mir_foreach_stmt(body, iterable, key.as_deref(), value, foreach_body),
            echo_mir::MirStmt::Switch {
                source,
                expr,
                cases,
            } => {
                self.render_mir_expr_as_echo_value(body, expr)?;
                for case in cases {
                    if let Some(condition) = &case.condition {
                        self.render_mir_expr_as_echo_value(body, condition)?;
                    }
                }
                Err(Diagnostic::new(
                    "unsupported switch statement in LLVM codegen",
                    stmt_span(source),
                ))
            }
            echo_mir::MirStmt::If {
                condition,
                body: if_body,
                elseif_clauses,
                else_body,
                ..
            } => self.render_mir_if_stmt(body, condition, if_body, elseif_clauses, else_body),
            echo_mir::MirStmt::Try {
                body: try_body,
                finally_body,
                ..
            } => {
                for statement in try_body {
                    if self.terminated {
                        break;
                    }
                    self.render_mir_stmt(body, statement)?;
                }
                for statement in finally_body {
                    if self.terminated {
                        break;
                    }
                    self.render_mir_stmt(body, statement)?;
                }
                Ok(())
            }
            echo_mir::MirStmt::Break { source, value } => {
                self.render_mir_break_stmt(body, source, value.as_ref())
            }
            echo_mir::MirStmt::Continue { source, value } => {
                self.render_mir_continue_stmt(body, source, value.as_ref())
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
            echo_mir::MirStmt::Noop { source } => self.render_noop_source_stmt(body, source),
        }
    }

    fn render_noop_source_stmt(
        &mut self,
        body: &mut String,
        source: &Stmt,
    ) -> Result<(), Diagnostic> {
        let members = match source {
            Stmt::ClassDecl(class_decl) => &class_decl.members,
            Stmt::TraitDecl(trait_decl) => &trait_decl.members,
            _ => return Ok(()),
        };

        for member in members {
            let echo_ast::ClassMember::Property(property) = member else {
                continue;
            };
            if !property.is_static {
                continue;
            }

            let value = match &property.value {
                Some(value) => {
                    self.render_mir_expr_as_echo_value(body, &echo_mir::lower_expr(value))?
                }
                None => "%EchoValue { i32 0, i64 0 }".to_string(),
            };
            let type_name = match source {
                Stmt::ClassDecl(class_decl) => &class_decl.name,
                Stmt::TraitDecl(trait_decl) => &trait_decl.name,
                _ => unreachable!("guarded above"),
            };
            let key = format!("{}::${}", type_name, property.name);
            let global = self.ensure_static_property_global(&key);
            body.push_str(&format!("  store {value}, ptr @{global}\n"));
        }

        Ok(())
    }

    fn render_mir_function_call_stmt(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<(), Diagnostic> {
        let name = self.resolve_std_call_name(&call.name, call.span)?;
        if name == "exit" {
            self.render_mir_exit_stmt(body, call)?;
        } else if name == "time.sleep" {
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

    fn render_mir_exit_stmt(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<(), Diagnostic> {
        let status = match call.args.as_slice() {
            [] => "0".to_string(),
            [arg] => {
                let value = self.render_mir_expr_as_echo_value(body, arg)?;
                let status_name = self.next_runtime_call_name();
                body.push_str(&format!(
                    "  {status_name} = call i32 @{}({value})\n",
                    CoreRuntimeSymbol::ValueExitStatus.symbol()
                ));
                status_name
            }
            _ => {
                return Err(Diagnostic::new(
                    "unsupported exit argument count in LLVM codegen",
                    call.span,
                ));
            }
        };

        body.push_str(&format!(
            "  call void @{}()\n",
            CoreRuntimeSymbol::Shutdown.symbol()
        ));
        body.push_str(&format!("  ret i32 {status}\n"));
        self.terminated = true;

        Ok(())
    }

    fn render_mir_php_exit_stmt(
        &mut self,
        body: &mut String,
        value: &Option<echo_mir::MirExpr>,
    ) -> Result<(), Diagnostic> {
        let status = match value {
            None => "0".to_string(),
            Some(value) => {
                let value = self.render_mir_expr_as_echo_value(body, value)?;
                let status_name = self.next_runtime_call_name();
                body.push_str(&format!(
                    "  {status_name} = call i32 @{}({value})\n",
                    CoreRuntimeSymbol::ValueExitStatus.symbol()
                ));
                status_name
            }
        };

        body.push_str(&format!(
            "  call void @{}()\n",
            CoreRuntimeSymbol::Shutdown.symbol()
        ));
        body.push_str(&format!("  ret i32 {status}\n"));
        self.terminated = true;

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

    fn render_mir_foreach_stmt(
        &mut self,
        body: &mut String,
        iterable: &echo_mir::MirExpr,
        key: Option<&str>,
        value: &str,
        foreach_body: &[echo_mir::MirStmt],
    ) -> Result<(), Diagnostic> {
        let iterable = self.render_mir_expr_as_echo_value(body, iterable)?;
        let loop_id = self.next_loop_id;
        self.next_loop_id += 1;
        let index_slot = format!("%foreach_index_slot_{loop_id}");
        let len_name = format!("%foreach_len_{loop_id}");
        let index_name = format!("%foreach_index_{loop_id}");
        let condition_name = format!("%foreach_keep_going_{loop_id}");
        let next_name = format!("%foreach_next_{loop_id}");
        let value_name = format!("%foreach_value_{loop_id}");

        body.push_str(&format!("  {index_slot} = alloca i64\n"));
        body.push_str(&format!("  store i64 0, ptr {index_slot}\n"));
        body.push_str(&format!(
            "  {len_name} = call i64 @{}({iterable})\n",
            CoreRuntimeSymbol::ValueArrayLen.symbol()
        ));
        body.push_str(&format!("  br label %foreach_condition_{loop_id}\n"));
        body.push_str(&format!("foreach_condition_{loop_id}:\n"));
        body.push_str(&format!("  {index_name} = load i64, ptr {index_slot}\n"));
        body.push_str(&format!(
            "  {condition_name} = icmp slt i64 {index_name}, {len_name}\n"
        ));
        body.push_str(&format!(
            "  br i1 {condition_name}, label %foreach_body_{loop_id}, label %foreach_done_{loop_id}\n"
        ));
        body.push_str(&format!("foreach_body_{loop_id}:\n"));
        self.break_labels.push(format!("foreach_done_{loop_id}"));
        self.continue_labels
            .push(format!("foreach_continue_{loop_id}"));
        self.break_value_slots.push(None);

        if let Some(key) = key {
            let key_name = format!("%foreach_key_{loop_id}");
            body.push_str(&format!(
                "  {key_name} = call %EchoValue @{}({iterable}, i64 {index_name})\n",
                CoreRuntimeSymbol::ValueArrayKeyAt.symbol()
            ));
            self.locals
                .insert(self.resolve_alias(key), RuntimeValue::EchoValue(key_name));
        }

        body.push_str(&format!(
            "  {value_name} = call %EchoValue @{}({iterable}, i64 {index_name})\n",
            CoreRuntimeSymbol::ValueArrayValueAt.symbol()
        ));
        self.locals.insert(
            self.resolve_alias(value),
            RuntimeValue::EchoValue(value_name),
        );

        for statement in foreach_body {
            if self.terminated {
                break;
            }
            self.render_mir_stmt(body, statement)?;
        }

        if !self.terminated {
            body.push_str(&format!("  br label %foreach_continue_{loop_id}\n"));
        }
        self.break_labels.pop();
        self.continue_labels.pop();
        self.break_value_slots.pop();
        self.terminated = false;
        body.push_str(&format!("foreach_continue_{loop_id}:\n"));
        body.push_str(&format!("  {next_name} = add i64 {index_name}, 1\n"));
        body.push_str(&format!("  store i64 {next_name}, ptr {index_slot}\n"));
        body.push_str(&format!("  br label %foreach_condition_{loop_id}\n"));
        body.push_str(&format!("foreach_done_{loop_id}:\n"));

        Ok(())
    }

    fn render_mir_dynamic_function_call(
        &mut self,
        body: &mut String,
        source: &Stmt,
        name: &str,
        args: &[echo_mir::MirCallArg],
    ) -> Result<(), Diagnostic> {
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
                    stmt_span(source),
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

            for statement in &closure_body {
                if self.terminated {
                    break;
                }
                self.render_mir_stmt(body, statement)?;
            }

            self.locals = saved_locals;
            self.aliases = saved_aliases;
            return Ok(());
        }

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
