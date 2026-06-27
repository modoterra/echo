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
        self.continue_labels.push(loop_label.clone());
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
        self.continue_labels.pop();
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

    pub(super) fn render_mir_continue_stmt(
        &mut self,
        body: &mut String,
        source: &Stmt,
        value: Option<&echo_mir::MirExpr>,
    ) -> Result<(), Diagnostic> {
        if let Some(value) = value {
            self.render_mir_expr_as_echo_value(body, value)?;
        }

        let Some(label) = self.continue_labels.last().cloned() else {
            return Err(Diagnostic::new(
                "continue used outside of loop in LLVM codegen",
                stmt_span(source),
            ));
        };

        body.push_str(&format!("  br label %{label}\n"));
        self.terminated = true;

        Ok(())
    }

    pub(super) fn render_mir_if_stmt(
        &mut self,
        body: &mut String,
        condition_expr: &echo_mir::MirExpr,
        statements: &[echo_mir::MirStmt],
        elseif_clauses: &[echo_mir::MirElseIfClause],
        else_statements: &[echo_mir::MirStmt],
    ) -> Result<(), Diagnostic> {
        if let Some(selected_statements) =
            self.static_if_branch(condition_expr, statements, elseif_clauses, else_statements)
        {
            let saved_terminated = self.terminated;
            self.terminated = false;

            for statement in selected_statements {
                self.render_mir_stmt(body, statement)?;
                if self.terminated {
                    break;
                }
            }

            self.terminated = saved_terminated;
            return Ok(());
        }

        let condition = self.render_mir_condition(body, condition_expr)?;
        let if_id = self.next_if_id;
        self.next_if_id += 1;
        let then_label = format!("if_then_{if_id}");
        let elseif_labels = (0..elseif_clauses.len())
            .map(|index| format!("if_elseif_{if_id}_{index}"))
            .collect::<Vec<_>>();
        let else_label = format!("if_else_{if_id}");
        let after_label = format!("if_after_{if_id}");
        let false_label = if let Some(first_elseif_label) = elseif_labels.first() {
            first_elseif_label
        } else if else_statements.is_empty() {
            &after_label
        } else {
            &else_label
        };

        body.push_str(&format!(
            "  br i1 {condition}, label %{then_label}, label %{false_label}\n\n{then_label}:\n"
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

        for (index, clause) in elseif_clauses.iter().enumerate() {
            let elseif_label = &elseif_labels[index];
            body.push_str(&format!("\n{elseif_label}:\n"));
            self.terminated = false;
            let condition = self.render_mir_condition(body, &clause.condition)?;
            let elseif_then_label = format!("if_elseif_then_{if_id}_{index}");
            let false_label = if let Some(next_label) = elseif_labels.get(index + 1) {
                next_label
            } else if else_statements.is_empty() {
                &after_label
            } else {
                &else_label
            };
            body.push_str(&format!(
                "  br i1 {condition}, label %{elseif_then_label}, label %{false_label}\n\n{elseif_then_label}:\n"
            ));

            for statement in &clause.body {
                self.render_mir_stmt(body, statement)?;
                if self.terminated {
                    break;
                }
            }

            if !self.terminated {
                body.push_str(&format!("  br label %{after_label}\n"));
            }
        }

        if !else_statements.is_empty() {
            body.push_str(&format!("\n{else_label}:\n"));
            self.terminated = false;

            for statement in else_statements {
                self.render_mir_stmt(body, statement)?;
                if self.terminated {
                    break;
                }
            }

            if !self.terminated {
                body.push_str(&format!("  br label %{after_label}\n"));
            }
        }

        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));

        Ok(())
    }

    fn static_if_branch<'a>(
        &self,
        condition_expr: &echo_mir::MirExpr,
        statements: &'a [echo_mir::MirStmt],
        elseif_clauses: &'a [echo_mir::MirElseIfClause],
        else_statements: &'a [echo_mir::MirStmt],
    ) -> Option<&'a [echo_mir::MirStmt]> {
        match static_bool_for_pruning(condition_expr)? {
            true => Some(statements),
            false => {
                for clause in elseif_clauses {
                    match static_bool_for_pruning(&clause.condition)? {
                        true => return Some(&clause.body),
                        false => {}
                    }
                }

                Some(else_statements)
            }
        }
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
        self.continue_labels.push(loop_label.clone());
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
        self.continue_labels.pop();
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

fn static_bool_for_pruning(expr: &echo_mir::MirExpr) -> Option<bool> {
    contains_compile_time_constant(expr)
        .then(|| static_bool(expr))
        .flatten()
}

fn static_bool(expr: &echo_mir::MirExpr) -> Option<bool> {
    match expr {
        echo_mir::MirExpr::Bool { value, .. } => Some(*value),
        echo_mir::MirExpr::Number { value, .. } => {
            value.parse::<i64>().ok().map(|value| value != 0)
        }
        echo_mir::MirExpr::Constant { name, .. } if name == "PHP_VERSION_ID" => Some(true),
        echo_mir::MirExpr::Binary {
            left, op, right, ..
        } => match op {
            BinaryOp::LessThan => Some(static_int(left)? < static_int(right)?),
            BinaryOp::GreaterThanOrEqual => Some(static_int(left)? >= static_int(right)?),
            BinaryOp::Identical => Some(static_int(left)? == static_int(right)?),
            BinaryOp::NotEqual => Some(static_int(left)? != static_int(right)?),
            BinaryOp::And => Some(static_bool(left)? && static_bool(right)?),
            BinaryOp::Or => Some(static_bool(left)? || static_bool(right)?),
            _ => None,
        },
        echo_mir::MirExpr::Unary { op, expr, .. } if *op == echo_ast::UnaryOp::Not => {
            Some(!static_bool(expr)?)
        }
        _ => None,
    }
}

fn static_int(expr: &echo_mir::MirExpr) -> Option<i64> {
    match expr {
        echo_mir::MirExpr::Number { value, .. } => value.parse().ok(),
        echo_mir::MirExpr::Constant { name, .. } if name == "PHP_VERSION_ID" => Some(80200),
        echo_mir::MirExpr::Bool { value, .. } => Some(i64::from(*value)),
        _ => None,
    }
}

fn contains_compile_time_constant(expr: &echo_mir::MirExpr) -> bool {
    match expr {
        echo_mir::MirExpr::Constant { name, .. } if name == "PHP_VERSION_ID" => true,
        echo_mir::MirExpr::Binary { left, right, .. } => {
            contains_compile_time_constant(left) || contains_compile_time_constant(right)
        }
        echo_mir::MirExpr::Unary { expr, .. } => contains_compile_time_constant(expr),
        _ => false,
    }
}
