use crate::IrModule;
use crate::abi::{BuiltinCodegen, CoreRuntimeSymbol, PhpBuiltin};
use echo_diagnostics::Diagnostic;
use echo_source::Span;

impl IrModule {
    pub(super) fn mir_php_builtin_call(
        &mut self,
        body: &mut String,
        builtin: PhpBuiltin,
        args: &[echo_mir::MirCallArg],
        span: Span,
    ) -> Result<(), Diagnostic> {
        match builtin.codegen {
            BuiltinCodegen::ObStart => match args {
                [] => {
                    let call_id = self.next_call_id;
                    self.next_call_id += 1;

                    body.push_str(&format!(
                        "  %runtime_call_{call_id} = call i1 @{}()\n",
                        builtin.symbol
                    ));
                }
                [arg] if matches!(&arg.value, echo_mir::MirExpr::Null { .. }) => {
                    let helper = builtin
                        .helper_symbol
                        .expect("ob_start value helper must be declared");
                    let call_id = self.next_call_id;
                    self.next_call_id += 1;

                    body.push_str(&format!(
                        "  %runtime_call_{call_id} = call i1 @{}(%EchoValue {{ i32 0, i64 0 }})\n",
                        helper
                    ));
                }
                [arg] if matches!(&arg.value, echo_mir::MirExpr::String { .. }) => {
                    let echo_mir::MirExpr::String { value, .. } = &arg.value else {
                        unreachable!("matches! checked string argument")
                    };
                    let helper = builtin
                        .helper_symbol
                        .expect("ob_start value helper must be declared");
                    let global = self.string_global(value);
                    let value_id = self.next_call_id;
                    self.next_call_id += 1;
                    let start_id = self.next_call_id;
                    self.next_call_id += 1;

                    body.push_str(&format!(
                        "  %runtime_call_{value_id} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
                        CoreRuntimeSymbol::ValueString.symbol(),
                        value.len()
                    ));
                    body.push_str(&format!(
                        "  %runtime_call_{start_id} = call i1 @{}(%EchoValue %runtime_call_{value_id})\n",
                        helper
                    ));
                }
                [expr] => {
                    return Err(Diagnostic::new(
                        "unsupported ob_start callback argument in LLVM codegen",
                        expr.syntax().span(),
                    ));
                }
                _ => {
                    return Err(Diagnostic::new(
                        "unsupported ob_start argument count in LLVM codegen",
                        args.first().map_or(span, |expr| expr.syntax().span()),
                    ));
                }
            },
            BuiltinCodegen::VoidStatement => {
                if !args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            builtin.php_name
                        ),
                        args.first().map_or(span, |expr| expr.syntax().span()),
                    ));
                }

                body.push_str(&format!("  call void @{}()\n", builtin.symbol));
            }
            BuiltinCodegen::VoidUnaryStatement => {
                let [arg] = args else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            builtin.php_name
                        ),
                        args.first().map_or(span, |expr| expr.syntax().span()),
                    ));
                };

                let arg = self.render_mir_expr_as_echo_value(body, arg)?;
                body.push_str(&format!("  call void @{}({arg})\n", builtin.symbol));
            }
            BuiltinCodegen::BoolStatement => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;

                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call i1 @{}()\n",
                    builtin.symbol
                ));
            }
            BuiltinCodegen::ValueExpression => {
                let call_id = self.next_call_id;
                self.next_call_id += 1;

                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}()\n",
                    builtin.symbol
                ));
            }
            BuiltinCodegen::ValueUnaryExpression => {
                let [arg] = args else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            builtin.php_name
                        ),
                        span,
                    ));
                };
                let arg = self.render_mir_expr_as_echo_value(body, arg)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}({arg})\n",
                    builtin.symbol
                ));
            }
            BuiltinCodegen::ValueBinaryExpression => {
                let [left, right] = args else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            builtin.php_name
                        ),
                        span,
                    ));
                };
                let left = self.render_mir_expr_as_echo_value(body, left)?;
                let right = self.render_mir_expr_as_echo_value(body, right)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}({left}, {right})\n",
                    builtin.symbol
                ));
            }
            BuiltinCodegen::ArrayKeys
            | BuiltinCodegen::ArrayMerge
            | BuiltinCodegen::ArrayReplace
            | BuiltinCodegen::ArrayReverse
            | BuiltinCodegen::ArraySlice
            | BuiltinCodegen::Basename
            | BuiltinCodegen::Dirname
            | BuiltinCodegen::ChunkSplit
            | BuiltinCodegen::Getenv
            | BuiltinCodegen::InArray
            | BuiltinCodegen::Implode
            | BuiltinCodegen::Levenshtein
            | BuiltinCodegen::Log
            | BuiltinCodegen::Nl2br
            | BuiltinCodegen::NumberFormat
            | BuiltinCodegen::Round
            | BuiltinCodegen::StrPad
            | BuiltinCodegen::StrSplit
            | BuiltinCodegen::ValueUnaryOptionalContextExpression
            | BuiltinCodegen::ValueBinaryOptionalContextExpression
            | BuiltinCodegen::ValueUnaryOptionalBoolExpression
            | BuiltinCodegen::ValueUnaryOptionalBoolContextExpression
            | BuiltinCodegen::ValueTernaryExpression
            | BuiltinCodegen::FileGetContents
            | BuiltinCodegen::FilePutContents
            | BuiltinCodegen::Mkdir
            | BuiltinCodegen::Touch
            | BuiltinCodegen::Uniqid
            | BuiltinCodegen::Explode
            | BuiltinCodegen::SubstrCompare
            | BuiltinCodegen::SubstrReplace => {
                unreachable!("expression builtin used as statement call")
            }
        }

        Ok(())
    }
}
