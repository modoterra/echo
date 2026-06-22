use crate::abi::CoreRuntimeSymbol;
use crate::{IrModule, RuntimeValue, stmt_span};
use echo_ast::{BinaryOp, Stmt};
use echo_diagnostics::Diagnostic;

impl IrModule {
    pub(super) fn render_mir_loop_stmt(
        &mut self,
        body: &mut String,
        statements: &[echo_mir::MirStmt],
    ) -> Result<(), Diagnostic> {
        let loop_id = self.next_loop_id;
        self.next_loop_id += 1;
        let loop_label = format!("loop_{loop_id}");
        let after_label = format!("loop_after_{loop_id}");

        body.push_str(&format!("  br label %{loop_label}\n\n{loop_label}:\n"));
        self.break_labels.push(after_label.clone());
        self.break_value_slots.push(None);

        let saved_terminated = self.terminated;
        self.terminated = false;

        for statement in statements {
            self.render_mir_stmt(body, statement)?;
            if self.terminated {
                break;
            }
        }

        if !self.terminated {
            body.push_str(&format!("  br label %{loop_label}\n"));
        }

        self.break_labels.pop();
        self.break_value_slots.pop();
        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));

        Ok(())
    }

    pub(super) fn render_mir_break_stmt(
        &mut self,
        body: &mut String,
        source: &Stmt,
        value: Option<&echo_mir::MirExpr>,
    ) -> Result<(), Diagnostic> {
        let Some(label) = self.break_labels.last().cloned() else {
            return Err(Diagnostic::new(
                "break used outside of loop in LLVM codegen",
                stmt_span(source),
            ));
        };

        if let Some(value) = value {
            let Some(Some(slot)) = self.break_value_slots.last().cloned() else {
                return Err(Diagnostic::new(
                    "break value used outside of expression loop in LLVM codegen",
                    stmt_span(source),
                ));
            };
            let value = self.render_mir_expr_as_echo_value(body, value)?;
            body.push_str(&format!("  store {value}, ptr {slot}\n"));
        }

        body.push_str(&format!("  br label %{label}\n"));
        self.terminated = true;

        Ok(())
    }

    pub(super) fn render_mir_if_stmt(
        &mut self,
        body: &mut String,
        condition_expr: &echo_mir::MirExpr,
        statements: &[echo_mir::MirStmt],
    ) -> Result<(), Diagnostic> {
        let condition = self.render_mir_condition(body, condition_expr)?;
        let if_id = self.next_if_id;
        self.next_if_id += 1;
        let then_label = format!("if_then_{if_id}");
        let after_label = format!("if_after_{if_id}");

        body.push_str(&format!(
            "  br i1 {condition}, label %{then_label}, label %{after_label}\n\n{then_label}:\n"
        ));

        let saved_terminated = self.terminated;
        self.terminated = false;

        for statement in statements {
            self.render_mir_stmt(body, statement)?;
            if self.terminated {
                break;
            }
        }

        if !self.terminated {
            body.push_str(&format!("  br label %{after_label}\n"));
        }

        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));

        Ok(())
    }

    pub(super) fn render_mir_loop_expr(
        &mut self,
        body: &mut String,
        statements: &[echo_mir::MirStmt],
    ) -> Result<RuntimeValue, Diagnostic> {
        let loop_id = self.next_loop_id;
        self.next_loop_id += 1;
        let loop_label = format!("loop_expr_{loop_id}");
        let after_label = format!("loop_expr_after_{loop_id}");
        let result_slot = format!("%loop_result_{loop_id}");
        let result_name = format!("%loop_value_{loop_id}");

        body.push_str(&format!("  {result_slot} = alloca %EchoValue\n"));
        body.push_str(&format!("  br label %{loop_label}\n\n{loop_label}:\n"));
        self.break_labels.push(after_label.clone());
        self.break_value_slots.push(Some(result_slot.clone()));

        let saved_terminated = self.terminated;
        self.terminated = false;

        for statement in statements {
            self.render_mir_stmt(body, statement)?;
            if self.terminated {
                break;
            }
        }

        if !self.terminated {
            body.push_str(&format!("  br label %{loop_label}\n"));
        }

        self.break_labels.pop();
        self.break_value_slots.pop();
        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));
        body.push_str(&format!(
            "  {result_name} = load %EchoValue, ptr {result_slot}\n"
        ));

        Ok(RuntimeValue::EchoValue(result_name))
    }

    fn render_mir_condition(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
    ) -> Result<String, Diagnostic> {
        match expr {
            echo_mir::MirExpr::Binary {
                source,
                left,
                op,
                right,
            } if matches!(op, BinaryOp::Is | BinaryOp::IsNot) => {
                if !matches!(right.as_ref(), echo_mir::MirExpr::Null { .. }) {
                    return Err(Diagnostic::new(
                        "unsupported non-null `is` comparison in LLVM codegen",
                        source.span(),
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, left)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let kind = format!("%value_kind_{call_id}");
                let is_null = format!("%is_null_{call_id}");
                let condition = format!("%condition_{call_id}");

                body.push_str(&format!("  {kind} = extractvalue {value}, 0\n"));
                body.push_str(&format!("  {is_null} = icmp eq i32 {kind}, 0\n"));

                if *op == BinaryOp::IsNot {
                    body.push_str(&format!("  {condition} = xor i1 {is_null}, true\n"));
                    Ok(condition)
                } else {
                    Ok(is_null)
                }
            }
            _ => {
                let value = self.render_mir_expr_as_echo_value(body, expr)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let condition = format!("%condition_{call_id}");
                body.push_str(&format!(
                    "  {condition} = call i1 @{}({value})\n",
                    CoreRuntimeSymbol::ValueBool.symbol()
                ));
                Ok(condition)
            }
        }
    }
}
