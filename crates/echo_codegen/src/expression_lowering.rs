use crate::abi::CoreRuntimeSymbol;
use crate::{IrModule, RuntimeValue};
use echo_diagnostics::Diagnostic;

impl IrModule {
    pub(super) fn render_mir_concat_expr(
        &mut self,
        body: &mut String,
        left: &echo_mir::MirExpr,
        right: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        let left = self.render_mir_expr(body, left)?;
        let right = self.render_mir_expr(body, right)?;

        match (left, right) {
            (RuntimeValue::StaticString(mut left), RuntimeValue::StaticString(right)) => {
                left.push_str(&right);
                Ok(RuntimeValue::StaticString(left))
            }
            (left, right) => {
                let left = self.runtime_value_as_echo_value(body, left);
                let right = self.runtime_value_as_echo_value(body, right);
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({left}, {right})\n",
                    CoreRuntimeSymbol::ValueConcat.symbol()
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
        }
    }

    pub(super) fn render_mir_numeric_binary_expr(
        &mut self,
        body: &mut String,
        left: &echo_mir::MirExpr,
        right: &echo_mir::MirExpr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let left = self.render_mir_expr_as_echo_value(body, left)?;
        let right = self.render_mir_expr_as_echo_value(body, right)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({left}, {right})\n",
            symbol.symbol()
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    pub(super) fn render_mir_numeric_unary_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let value = self.render_mir_expr_as_echo_value(body, expr)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({value})\n",
            symbol.symbol()
        ));

        Ok(RuntimeValue::EchoValue(name))
    }
}
