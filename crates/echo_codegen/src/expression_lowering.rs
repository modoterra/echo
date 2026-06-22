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
                let name = self.push_echo_value_call(
                    body,
                    CoreRuntimeSymbol::ValueConcat.symbol(),
                    &format!("{left}, {right}"),
                );

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
        let name = self.push_echo_value_call(body, symbol.symbol(), &format!("{left}, {right}"));

        Ok(RuntimeValue::EchoValue(name))
    }

    pub(super) fn render_mir_numeric_unary_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let value = self.render_mir_expr_as_echo_value(body, expr)?;
        let name = self.push_echo_value_call(body, symbol.symbol(), &value);

        Ok(RuntimeValue::EchoValue(name))
    }
}
