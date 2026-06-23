use crate::abi::CoreRuntimeSymbol;
use crate::{IrModule, RuntimeValue};
use echo_diagnostics::Diagnostic;

impl IrModule {
    pub(super) fn render_mir_defer_function(
        &mut self,
        caller_body: &mut String,
        statements: &[echo_mir::MirStmt],
    ) -> Result<String, Diagnostic> {
        let function = format!("echo_defer_{}", self.next_defer_id);
        self.next_defer_id += 1;

        let captures = self
            .locals
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();

        for (name, value) in &captures {
            let global = format!("echo_capture_{}_{}", function, name);
            self.globals
                .push_str(&format!("@{global} = global %EchoValue zeroinitializer\n"));
            let value = self.runtime_value_as_echo_value(caller_body, value.clone());
            caller_body.push_str(&format!("  store {value}, ptr @{global}\n"));
        }

        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_returned = self.returned;
        let saved_terminated = self.terminated;
        self.returned = false;
        self.terminated = false;

        let sleep = if let Some(echo_mir::MirStmt::FunctionCall { call, .. }) = statements.first()
            && self.resolve_std_call_name(&call.name, call.span)? == "time.sleep"
        {
            mir_task_sleep_millis(statements.first().expect("first statement exists"))
        } else {
            None
        };

        let mut body = String::new();
        for (name, _) in &captures {
            let global = format!("echo_capture_{}_{}", function, name);
            let local = format!("%capture_{}_{}", function, name);
            body.push_str(&format!("  {local} = load %EchoValue, ptr @{global}\n"));
            self.locals
                .insert(name.clone(), RuntimeValue::EchoValue(local));
        }
        if let Some(millis) = sleep {
            let continuation =
                self.render_mir_defer_continuation_function(&function, &statements[1..])?;
            body.push_str(&format!(
                "  %runtime_call_{} = call %EchoValue @{}(i64 {millis}, ptr @{continuation})\n",
                self.next_call_id,
                CoreRuntimeSymbol::TaskSleepCurrent.symbol()
            ));
            self.next_call_id += 1;
            body.push_str(&format!(
                "  ret %EchoValue %runtime_call_{}\n",
                self.next_call_id - 1
            ));
            self.returned = true;
        } else {
            for statement in statements {
                if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                    self.aliases = saved_aliases;
                    self.locals = saved_locals;
                    self.returned = saved_returned;
                    self.terminated = saved_terminated;
                    return Err(diagnostic);
                }
            }
        }

        let returned = self.returned;
        self.aliases = saved_aliases;
        self.locals = saved_locals;
        self.returned = saved_returned;
        self.terminated = saved_terminated;

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{function}() {{\nentry:\n{}{}\n}}\n",
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(function)
    }

    fn render_mir_defer_continuation_function(
        &mut self,
        parent: &str,
        statements: &[echo_mir::MirStmt],
    ) -> Result<String, Diagnostic> {
        let function = format!("{parent}_cont");
        let saved_returned = self.returned;
        let saved_terminated = self.terminated;
        self.returned = false;
        self.terminated = false;

        let mut body = String::new();
        for statement in statements {
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                self.returned = saved_returned;
                self.terminated = saved_terminated;
                return Err(diagnostic);
            }
        }

        let returned = self.returned;
        self.returned = saved_returned;
        self.terminated = saved_terminated;

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{function}() {{\nentry:\n{}{}\n}}\n",
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(function)
    }

    pub(super) fn render_mir_run_expr(
        &mut self,
        body: &mut String,
        source: &echo_ast::Expr,
        expr: &echo_mir::MirRunExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirRunExpr::Block { body: block } => {
                let function = self.render_mir_defer_function(body, block)?;
                let defer_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{defer_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::TaskDefer.symbol()
                ));

                let run_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{run_id} = call %EchoValue @{}(%EchoValue %runtime_call_{defer_id})\n",
                    CoreRuntimeSymbol::TaskRun.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{run_id}")))
            }
            echo_mir::MirRunExpr::Task { expr } => {
                self.render_mir_task_unary_expr(body, expr, CoreRuntimeSymbol::TaskRun)
            }
            echo_mir::MirRunExpr::Group { entries } => {
                self.render_mir_run_group_expr(body, source, entries)
            }
        }
    }

    fn render_mir_run_group_expr(
        &mut self,
        body: &mut String,
        _source: &echo_ast::Expr,
        entries: &[Vec<echo_mir::MirStmt>],
    ) -> Result<RuntimeValue, Diagnostic> {
        let group_id = self.next_call_id;
        self.next_call_id += 1;
        let mut group = format!("%runtime_call_{group_id}");
        body.push_str(&format!(
            "  {group} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::TaskGroupNew.symbol()
        ));

        for entry in entries {
            let function = self.render_mir_defer_function(body, entry)?;
            let defer_id = self.next_call_id;
            self.next_call_id += 1;
            let task = format!("%runtime_call_{defer_id}");
            body.push_str(&format!(
                "  {task} = call %EchoValue @{}(ptr @{function})\n",
                CoreRuntimeSymbol::TaskDefer.symbol()
            ));

            let add_id = self.next_call_id;
            self.next_call_id += 1;
            let added = format!("%runtime_call_{add_id}");
            body.push_str(&format!(
                "  {added} = call %EchoValue @{}(%EchoValue {group}, %EchoValue {task})\n",
                CoreRuntimeSymbol::TaskGroupAdd.symbol()
            ));
            group = added;
        }

        let run_id = self.next_call_id;
        self.next_call_id += 1;
        body.push_str(&format!(
            "  %runtime_call_{run_id} = call %EchoValue @{}(%EchoValue {group})\n",
            CoreRuntimeSymbol::TaskGroupRunAndJoin.symbol()
        ));
        Ok(RuntimeValue::EchoValue(format!("%runtime_call_{run_id}")))
    }

    pub(super) fn render_mir_fork_expr(
        &mut self,
        body: &mut String,
        _source: &echo_ast::Expr,
        expr: &echo_mir::MirForkExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirForkExpr::Block { body: block } => {
                let function = self.render_mir_defer_function(body, block)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::ThreadFork.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirForkExpr::Task { expr } => {
                self.render_mir_task_unary_expr(body, expr, CoreRuntimeSymbol::ThreadForkTask)
            }
        }
    }

    pub(super) fn render_mir_task_unary_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let task = self.render_mir_expr_as_echo_value(body, expr)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}({task})\n",
            symbol.symbol()
        ));

        Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
    }
}

fn mir_task_sleep_millis(statement: &echo_mir::MirStmt) -> Option<i64> {
    let echo_mir::MirStmt::FunctionCall { call, .. } = statement else {
        return None;
    };
    if call.name != "time.sleep" {
        return None;
    }
    let [echo_mir::MirExpr::Number { value, .. }] = call.args.as_slice() else {
        return None;
    };

    value.parse().ok()
}
