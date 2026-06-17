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
    TaskRun,
    TaskJoin,
    TaskSleepCurrent,
    TimeSleep,
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
        Self::TaskRun,
        Self::TaskJoin,
        Self::TaskSleepCurrent,
        Self::TimeSleep,
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
            Self::TaskRun => "echo_task_run",
            Self::TaskJoin => "echo_task_join",
            Self::TaskSleepCurrent => "echo_task_sleep_current",
            Self::TimeSleep => "echo_time_sleep",
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
            Self::TaskRun => RuntimeSignature::EchoValueEchoValue,
            Self::TaskJoin => RuntimeSignature::EchoValueEchoValue,
            Self::TaskSleepCurrent => RuntimeSignature::EchoValueI64Ptr,
            Self::TimeSleep => RuntimeSignature::VoidI64,
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
    VoidI64,
    VoidPtrI64,
    VoidEchoValue,
    BoolNoArgs,
    BoolEchoValue,
    EchoValueNoArgs,
    EchoValueI64Ptr,
    EchoValuePtr,
    EchoValuePtrI64,
    EchoValueEchoValue,
    EchoValueEchoValueEchoValue,
}

impl RuntimeSignature {
    pub fn llvm_decl(self, symbol: &str) -> String {
        match self {
            Self::VoidNoArgs => format!("declare void @{symbol}()"),
            Self::VoidI64 => format!("declare void @{symbol}(i64)"),
            Self::VoidPtrI64 => format!("declare void @{symbol}(ptr, i64)"),
            Self::VoidEchoValue => format!("declare void @{symbol}(%EchoValue)"),
            Self::BoolNoArgs => format!("declare i1 @{symbol}()"),
            Self::BoolEchoValue => format!("declare i1 @{symbol}(%EchoValue)"),
            Self::EchoValueNoArgs => format!("declare %EchoValue @{symbol}()"),
            Self::EchoValueI64Ptr => format!("declare %EchoValue @{symbol}(i64, ptr)"),
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
pub struct StdIntrinsic {
    pub echo_name: &'static str,
    pub symbol: &'static str,
    pub signature: RuntimeSignature,
    pub arity: usize,
}

impl StdIntrinsic {
    pub fn llvm_decl(self) -> String {
        self.signature.llvm_decl(self.symbol)
    }
}

pub const STD_INTRINSICS: &[StdIntrinsic] = &[
    StdIntrinsic {
        echo_name: "http.responseText",
        symbol: "echo_std_http_response_text",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "net.listen",
        symbol: "echo_std_net_listen",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "net.connect",
        symbol: "echo_std_net_connect",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "net.accept",
        symbol: "echo_std_net_accept",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "net.read",
        symbol: "echo_std_net_read",
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        arity: 2,
    },
    StdIntrinsic {
        echo_name: "net.write",
        symbol: "echo_std_net_write",
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        arity: 2,
    },
    StdIntrinsic {
        echo_name: "net.close",
        symbol: "echo_std_net_close",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
];

pub fn std_intrinsic(name: &str) -> Option<StdIntrinsic> {
    STD_INTRINSICS
        .iter()
        .copied()
        .find(|intrinsic| intrinsic.echo_name == name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinCodegen {
    ObStart,
    BoolStatement,
    ValueExpression,
    ValueUnaryExpression,
    ValueBinaryExpression,
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
    PhpBuiltin {
        php_name: "strtoupper",
        symbol: "echo_php_strtoupper",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "strtolower",
        symbol: "echo_php_strtolower",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "strrev",
        symbol: "echo_php_strrev",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "ucfirst",
        symbol: "echo_php_ucfirst",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "lcfirst",
        symbol: "echo_php_lcfirst",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "ord",
        symbol: "echo_php_ord",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "str_rot13",
        symbol: "echo_php_str_rot13",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "chr",
        symbol: "echo_php_chr",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "bin2hex",
        symbol: "echo_php_bin2hex",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "hex2bin",
        symbol: "echo_php_hex2bin",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "trim",
        symbol: "echo_php_trim",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "ltrim",
        symbol: "echo_php_ltrim",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "rtrim",
        symbol: "echo_php_rtrim",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "addslashes",
        symbol: "echo_php_addslashes",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "stripslashes",
        symbol: "echo_php_stripslashes",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "quotemeta",
        symbol: "echo_php_quotemeta",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueUnaryExpression,
    },
    PhpBuiltin {
        php_name: "str_contains",
        symbol: "echo_php_str_contains",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
    PhpBuiltin {
        php_name: "str_starts_with",
        symbol: "echo_php_str_starts_with",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
    PhpBuiltin {
        php_name: "str_ends_with",
        symbol: "echo_php_str_ends_with",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
    PhpBuiltin {
        php_name: "str_repeat",
        symbol: "echo_php_str_repeat",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
    PhpBuiltin {
        php_name: "substr",
        symbol: "echo_php_substr",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
    PhpBuiltin {
        php_name: "strpos",
        symbol: "echo_php_strpos",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
    PhpBuiltin {
        php_name: "strstr",
        symbol: "echo_php_strstr",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
    PhpBuiltin {
        php_name: "strcmp",
        symbol: "echo_php_strcmp",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
    PhpBuiltin {
        php_name: "strcasecmp",
        symbol: "echo_php_strcasecmp",
        helper_symbol: None,
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        lowering: BuiltinLowering::DirectRuntimeCall,
        codegen: BuiltinCodegen::ValueBinaryExpression,
    },
];

pub fn php_builtin(name: &str) -> Option<PhpBuiltin> {
    PHP_BUILTINS
        .iter()
        .copied()
        .find(|builtin| builtin.php_name == name)
}
