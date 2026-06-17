//! Compiler-facing runtime ABI policy.
//!
//! `echo_*` symbols are the small core compiler/runtime ABI used for language
//! semantics such as output writes and shutdown.
//! `echo_php_*` symbols are PHP builtin implementations used when codegen can
//! statically resolve a PHP function call.
//! `echo_ext_*` is reserved for a future extension/module ABI.
//! `echo_internal_*` symbols are runtime-private implementation details and must
//! not be emitted by codegen.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreRuntimeSymbol {
    Write,
    WriteValue,
    ValueString,
    ValueConcat,
    TaskDefer,
    CallFunction,
    Shutdown,
}

impl CoreRuntimeSymbol {
    pub const ALL: &'static [Self] = &[
        Self::Write,
        Self::WriteValue,
        Self::ValueString,
        Self::ValueConcat,
        Self::TaskDefer,
        Self::CallFunction,
        Self::Shutdown,
    ];

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Write => "echo_write",
            Self::WriteValue => "echo_write_value",
            Self::ValueString => "echo_value_string",
            Self::ValueConcat => "echo_value_concat",
            Self::TaskDefer => "echo_task_defer",
            Self::CallFunction => "echo_call_function",
            Self::Shutdown => "echo_shutdown",
        }
    }

    pub const fn signature(self) -> RuntimeSignature {
        match self {
            Self::Write => RuntimeSignature::VoidPtrI64,
            Self::WriteValue => RuntimeSignature::VoidEchoValue,
            Self::ValueString => RuntimeSignature::EchoValuePtrI64,
            Self::ValueConcat => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::TaskDefer => RuntimeSignature::EchoValuePtr,
            Self::CallFunction => RuntimeSignature::EchoValuePtrI64,
            Self::Shutdown => RuntimeSignature::VoidNoArgs,
        }
    }

    pub fn llvm_decl(self) -> String {
        self.signature().llvm_decl(self.symbol())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSignature {
    VoidNoArgs,
    VoidPtrI64,
    VoidEchoValue,
    BoolNoArgs,
    BoolEchoValue,
    EchoValueNoArgs,
    EchoValuePtr,
    EchoValuePtrI64,
    EchoValueEchoValue,
    EchoValueEchoValueEchoValue,
}

impl RuntimeSignature {
    pub fn llvm_decl(self, symbol: &str) -> String {
        match self {
            Self::VoidNoArgs => format!("declare void @{symbol}()"),
            Self::VoidPtrI64 => format!("declare void @{symbol}(ptr, i64)"),
            Self::VoidEchoValue => format!("declare void @{symbol}(%EchoValue)"),
            Self::BoolNoArgs => format!("declare i1 @{symbol}()"),
            Self::BoolEchoValue => format!("declare i1 @{symbol}(%EchoValue)"),
            Self::EchoValueNoArgs => format!("declare %EchoValue @{symbol}()"),
            Self::EchoValuePtr => format!("declare %EchoValue @{symbol}(ptr)"),
            Self::EchoValuePtrI64 => format!("declare %EchoValue @{symbol}(ptr, i64)"),
            Self::EchoValueEchoValue => format!("declare %EchoValue @{symbol}(%EchoValue)"),
            Self::EchoValueEchoValueEchoValue => {
                format!("declare %EchoValue @{symbol}(%EchoValue, %EchoValue)")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinLowering {
    DirectRuntimeCall,
    #[allow(dead_code)]
    GenericRuntimeCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinCodegen {
    ObStart,
    BoolStatement,
    ValueExpression,
    ValueUnaryExpression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhpBuiltin {
    pub php_name: &'static str,
    pub symbol: &'static str,
    pub helper_symbol: Option<&'static str>,
    pub signature: RuntimeSignature,
    pub lowering: BuiltinLowering,
    pub codegen: BuiltinCodegen,
}

impl PhpBuiltin {
    pub fn llvm_decl(self) -> String {
        self.signature.llvm_decl(self.symbol)
    }
}

pub const PHP_RUNTIME_HELPERS: &[(&str, RuntimeSignature)] =
    &[("echo_php_ob_start_value", RuntimeSignature::BoolEchoValue)];

pub const PHP_BUILTINS: &[PhpBuiltin] = &[
    PhpBuiltin {
        php_name: "ob_start",
        symbol: "echo_php_ob_start",
        helper_symbol: Some("echo_php_ob_start_value"),
        signature: RuntimeSignature::BoolNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ObStart,
    },
    PhpBuiltin {
        php_name: "ob_flush",
        symbol: "echo_php_ob_flush",
        helper_symbol: None,
        signature: RuntimeSignature::BoolNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::BoolStatement,
    },
    PhpBuiltin {
        php_name: "ob_clean",
        symbol: "echo_php_ob_clean",
        helper_symbol: None,
        signature: RuntimeSignature::BoolNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::BoolStatement,
    },
    PhpBuiltin {
        php_name: "ob_end_flush",
        symbol: "echo_php_ob_end_flush",
        helper_symbol: None,
        signature: RuntimeSignature::BoolNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::BoolStatement,
    },
    PhpBuiltin {
        php_name: "ob_end_clean",
        symbol: "echo_php_ob_end_clean",
        helper_symbol: None,
        signature: RuntimeSignature::BoolNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::BoolStatement,
    },
    PhpBuiltin {
        php_name: "ob_get_clean",
        symbol: "echo_php_ob_get_clean",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueExpression,
    },
    PhpBuiltin {
        php_name: "ob_get_contents",
        symbol: "echo_php_ob_get_contents",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueExpression,
    },
    PhpBuiltin {
        php_name: "ob_get_flush",
        symbol: "echo_php_ob_get_flush",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueExpression,
    },
    PhpBuiltin {
        php_name: "ob_get_length",
        symbol: "echo_php_ob_get_length",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueExpression,
    },
    PhpBuiltin {
        php_name: "ob_get_level",
        symbol: "echo_php_ob_get_level",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueNoArgs,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueExpression,
    },
    PhpBuiltin {
        php_name: "strlen",
        symbol: "echo_php_strlen",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
];

pub fn php_builtin(name: &str) -> Option<PhpBuiltin> {
    PHP_BUILTINS
        .iter()
        .copied()
        .find(|builtin| builtin.php_name == name)
}
