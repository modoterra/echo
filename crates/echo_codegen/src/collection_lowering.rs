use crate::abi::CoreRuntimeSymbol;
use crate::{IrModule, RuntimeValue, stmt_span};
use echo_ast::Stmt;
use echo_diagnostics::Diagnostic;

impl IrModule {
    pub(super) fn render_mir_append_stmt(
        &mut self,
        body: &mut String,
        source: &Stmt,
        target: &str,
        value: &echo_mir::MirExpr,
    ) -> Result<(), Diagnostic> {
        let resolved_target = self.resolve_alias(target);
        let Some(list) = self.locals.get(&resolved_target).cloned() else {
            return Err(Diagnostic::new(
                format!("unsupported append to undefined variable `${target}` in LLVM codegen"),
                stmt_span(source),
            ));
        };
        let list = self.runtime_value_as_echo_value(body, list);
        let value = self.render_mir_expr_as_echo_value(body, value)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({list}, {value})\n",
            CoreRuntimeSymbol::ValueArrayAppend.symbol()
        ));

        self.locals
            .insert(resolved_target, RuntimeValue::EchoValue(name));

        Ok(())
    }

    pub(super) fn render_mir_method_call_expr(
        &mut self,
        body: &mut String,
        object: &echo_mir::MirExpr,
        method: &str,
        args: &[echo_mir::MirExpr],
    ) -> Result<RuntimeValue, Diagnostic> {
        match (method, args) {
            ("push", [value]) => self.render_mir_list_push_expr(body, object, value),
            _ => {
                self.render_mir_expr_as_echo_value(body, object)?;
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
        }
    }

    fn render_mir_list_push_expr(
        &mut self,
        body: &mut String,
        object: &echo_mir::MirExpr,
        value: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let target = match object {
            echo_mir::MirExpr::Variable { name, .. } => Some(self.resolve_alias(name)),
            _ => None,
        };
        let list = self.render_mir_expr_as_echo_value(body, object)?;
        let value = self.render_mir_expr_as_echo_value(body, value)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({list}, {value})\n",
            CoreRuntimeSymbol::ValueListAppend.symbol()
        ));

        if let Some(target) = target {
            self.locals
                .insert(target, RuntimeValue::EchoValue(name.clone()));
        }

        Ok(RuntimeValue::EchoValue(name))
    }

    pub(super) fn render_mir_field_expr(
        &mut self,
        body: &mut String,
        object: &echo_mir::MirExpr,
        field: &str,
    ) -> Result<RuntimeValue, Diagnostic> {
        let object = self.render_mir_expr_as_echo_value(body, object)?;
        let field_global = self.string_global(field);
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({object}, ptr @{field_global}, i64 {})\n",
            CoreRuntimeSymbol::ValueObjectGet.symbol(),
            field.len()
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    pub(super) fn render_mir_index_expr(
        &mut self,
        body: &mut String,
        collection: &echo_mir::MirExpr,
        index: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let collection = self.render_mir_expr_as_echo_value(body, collection)?;
        let index = self.render_mir_expr_as_echo_value(body, index)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({collection}, {index})\n",
            CoreRuntimeSymbol::ValueIndexGet.symbol()
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    pub(super) fn render_mir_array_expr(
        &mut self,
        body: &mut String,
        elements: &[echo_mir::MirArrayElement],
    ) -> Result<RuntimeValue, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut array = format!("%runtime_call_{call_id}");
        body.push_str(&format!(
            "  {array} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueArrayNew.symbol()
        ));

        for element in elements {
            let value = self.render_mir_expr_as_echo_value(body, &element.value)?;
            let append_id = self.next_call_id;
            self.next_call_id += 1;
            let appended = format!("%runtime_call_{append_id}");
            match &element.key {
                Some(key) => {
                    let key = self.render_mir_expr_as_echo_value(body, key)?;
                    body.push_str(&format!(
                        "  {appended} = call %EchoValue @{}(%EchoValue {array}, {key}, {value})\n",
                        CoreRuntimeSymbol::ValueArraySet.symbol()
                    ));
                }
                None => {
                    body.push_str(&format!(
                        "  {appended} = call %EchoValue @{}(%EchoValue {array}, {value})\n",
                        CoreRuntimeSymbol::ValueArrayAppend.symbol()
                    ));
                }
            }
            array = appended;
        }

        Ok(RuntimeValue::EchoValue(array))
    }

    pub(super) fn render_mir_call_args_as_array(
        &mut self,
        body: &mut String,
        args: &[echo_mir::MirExpr],
    ) -> Result<String, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut array = format!("%runtime_call_{call_id}");
        body.push_str(&format!(
            "  {array} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueArrayNew.symbol()
        ));

        for arg in args {
            let value = self.render_mir_expr_as_echo_value(body, arg)?;
            let append_id = self.next_call_id;
            self.next_call_id += 1;
            let appended = format!("%runtime_call_{append_id}");
            body.push_str(&format!(
                "  {appended} = call %EchoValue @{}(%EchoValue {array}, {value})\n",
                CoreRuntimeSymbol::ValueArrayAppend.symbol()
            ));
            array = appended;
        }

        Ok(array)
    }

    pub(super) fn render_mir_list_values(
        &mut self,
        body: &mut String,
        values: &[echo_mir::MirExpr],
    ) -> Result<RuntimeValue, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut list = format!("%runtime_call_{call_id}");
        body.push_str(&format!(
            "  {list} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueListNew.symbol()
        ));

        for value in values {
            let value = self.render_mir_expr_as_echo_value(body, value)?;
            let append_id = self.next_call_id;
            self.next_call_id += 1;
            let appended = format!("%runtime_call_{append_id}");
            body.push_str(&format!(
                "  {appended} = call %EchoValue @{}(%EchoValue {list}, {value})\n",
                CoreRuntimeSymbol::ValueListAppend.symbol()
            ));
            list = appended;
        }

        Ok(RuntimeValue::EchoValue(list))
    }

    pub(super) fn render_mir_object_expr(
        &mut self,
        body: &mut String,
        fields: &[echo_mir::MirObjectField],
    ) -> Result<RuntimeValue, Diagnostic> {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let mut object = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {object} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::ValueObjectNew.symbol()
        ));

        for field in fields {
            let field_global = self.string_global(&field.name);
            let value = self.render_mir_expr_as_echo_value(body, &field.value)?;
            let call_id = self.next_call_id;
            self.next_call_id += 1;
            let next_object = format!("%runtime_call_{call_id}");

            body.push_str(&format!(
                "  {next_object} = call %EchoValue @{}(%EchoValue {object}, ptr @{field_global}, i64 {}, {value})\n",
                CoreRuntimeSymbol::ValueObjectSet.symbol(),
                field.name.len()
            ));
            object = next_object;
        }

        Ok(RuntimeValue::EchoValue(object))
    }
}
