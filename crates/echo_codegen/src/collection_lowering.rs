use crate::abi::CoreRuntimeSymbol;
use crate::{IrModule, RuntimeValue, stmt_span};
use echo_ast::Stmt;
use echo_diagnostics::Diagnostic;

impl IrModule {
    pub(super) fn render_mir_append_stmt(
        &mut self,
        body: &mut String,
        source: &Stmt,
        target: &echo_mir::MirExpr,
        value: &echo_mir::MirExpr,
    ) -> Result<(), Diagnostic> {
        let echo_mir::MirExpr::Variable { name: target, .. } = target else {
            self.render_mir_expr_as_echo_value(body, target)?;
            self.render_mir_expr_as_echo_value(body, value)?;
            return Ok(());
        };
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
        _source: &echo_ast::Expr,
        object: &echo_mir::MirExpr,
        method: &str,
        _method_span: echo_source::Span,
        args: &[echo_mir::MirCallArg],
    ) -> Result<RuntimeValue, Diagnostic> {
        match (method, args) {
            ("push", [value]) => self.render_mir_list_push_expr(body, object, value),
            _ => {
                let receiver = self.render_mir_expr(body, object)?;
                let function_name = self.method_function_name(receiver.clone(), method);
                if let Some(function_name) = function_name {
                    return self.render_known_method_call(body, &function_name, args, _method_span);
                }
                self.runtime_value_as_echo_value(body, receiver);
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Err(Diagnostic::new(
                    format!("unsupported method call `{method}` in LLVM codegen"),
                    _method_span,
                ))
            }
        }
    }

    fn render_known_method_call(
        &mut self,
        body: &mut String,
        function_name: &str,
        args: &[echo_mir::MirCallArg],
        span: echo_source::Span,
    ) -> Result<RuntimeValue, Diagnostic> {
        if let Some(function) = self.functions.get(function_name).cloned() {
            self.render_userland_function(&function)?;
        }
        self.render_mir_userland_function_call_expr(
            body,
            &echo_mir::MirFunctionCall {
                name: function_name.to_string(),
                args: args.to_vec(),
                span,
            },
        )
    }

    fn method_function_name(&self, receiver: RuntimeValue, method: &str) -> Option<String> {
        if let RuntimeValue::Object {
            class_name: Some(class_name),
            ..
        } = receiver
        {
            let mut current = Some(class_name);
            let mut seen = std::collections::HashSet::new();
            while let Some(class_name) = current {
                if !seen.insert(class_name.clone()) {
                    break;
                }
                let candidate = format!("{class_name}::{method}");
                if self.functions.contains_key(&candidate) {
                    return Some(candidate);
                }
                if let Some(function_name) = self.trait_method_function_name(&class_name, method) {
                    return Some(function_name);
                }
                current = self.class_parents.get(&class_name).cloned();
            }
        }

        let suffix = format!("::{method}");
        let mut matches = self
            .functions
            .keys()
            .filter(|name| name.ends_with(&suffix))
            .cloned()
            .collect::<Vec<_>>();
        matches.sort();
        (matches.len() == 1).then(|| matches.remove(0))
    }

    fn trait_method_function_name(&self, class_name: &str, method: &str) -> Option<String> {
        let traits = self.class_traits.get(class_name)?;
        for trait_name in traits {
            let candidates = [
                format!("{trait_name}::{method}"),
                trait_name
                    .rsplit(['\\', '.'])
                    .next()
                    .map(|short_name| format!("{short_name}::{method}"))
                    .unwrap_or_default(),
            ];
            for candidate in candidates {
                if self.functions.contains_key(&candidate) {
                    return Some(candidate);
                }
            }
        }
        None
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
        let object_value = self.render_mir_expr(body, object)?;
        let class_name = match &object_value {
            RuntimeValue::Object {
                class_name: Some(class_name),
                ..
            } => self
                .property_types
                .get(&format!("{class_name}::${field}"))
                .cloned(),
            _ => None,
        };
        let object = self.runtime_value_as_echo_value(body, object_value);
        let field_global = self.string_global(field);
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({object}, ptr @{field_global}, i64 {})\n",
            CoreRuntimeSymbol::ValueObjectGet.symbol(),
            field.len()
        ));

        Ok(RuntimeValue::Object {
            value: name,
            class_name,
        })
    }

    pub(super) fn render_mir_field_assign_expr(
        &mut self,
        body: &mut String,
        object: &echo_mir::MirExpr,
        field: &str,
        value: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let object = self.render_mir_expr_as_echo_value(body, object)?;
        let value = self.render_mir_expr_as_echo_value(body, value)?;
        let field_global = self.string_global(field);
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({object}, ptr @{field_global}, i64 {}, {value})\n",
            CoreRuntimeSymbol::ValueObjectSet.symbol(),
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
        args: &[echo_mir::MirCallArg],
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
