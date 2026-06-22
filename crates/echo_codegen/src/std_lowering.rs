use crate::abi::{CoreRuntimeSymbol, StdIntrinsic};
use crate::{IrModule, RuntimeValue};
use echo_diagnostics::Diagnostic;
use echo_source::Span;

impl IrModule {
    pub(super) fn mir_time_sleep_call(
        &mut self,
        body: &mut String,
        args: &[echo_mir::MirExpr],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let [
            echo_mir::MirExpr::Number {
                source: expr,
                value,
            },
        ] = args
        else {
            return Err(Diagnostic::new(
                "unsupported argument for time.sleep in LLVM codegen",
                span,
            ));
        };

        let millis = value.parse::<i64>().map_err(|_| {
            Diagnostic::new(
                "unsupported duration for time.sleep in LLVM codegen",
                expr.span(),
            )
        })?;

        body.push_str(&format!(
            "  call void @{}(i64 {millis})\n",
            CoreRuntimeSymbol::TimeSleep.symbol()
        ));

        Ok(())
    }

    pub(super) fn mir_std_intrinsic_call(
        &mut self,
        body: &mut String,
        intrinsic: StdIntrinsic,
        args: &[echo_mir::MirExpr],
        span: Span,
    ) -> Result<RuntimeValue, Diagnostic> {
        if args.len() != intrinsic.arity {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for std intrinsic `{}` in LLVM codegen",
                    intrinsic.echo_name
                ),
                span,
            ));
        }

        let mut rendered_args = Vec::new();
        for arg in args {
            rendered_args.push(self.render_mir_std_intrinsic_arg(body, arg)?);
        }

        let name = self.push_echo_value_call(body, intrinsic.symbol, &rendered_args.join(", "));

        Ok(RuntimeValue::EchoValue(name))
    }

    fn render_mir_std_intrinsic_arg(
        &mut self,
        body: &mut String,
        arg: &echo_mir::MirExpr,
    ) -> Result<String, Diagnostic> {
        if let echo_mir::MirExpr::Number { source, value } = arg {
            let value = value.parse::<i64>().map_err(|_| {
                Diagnostic::new(
                    "unsupported numeric std intrinsic argument in LLVM codegen",
                    source.span(),
                )
            })?;
            return Ok(format!("%EchoValue {{ i32 2, i64 {value} }}"));
        }

        self.render_mir_expr_as_echo_value(body, arg)
    }
}
