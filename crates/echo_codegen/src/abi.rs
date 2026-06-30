//! Compiler-facing runtime ABI policy.
//!
//! `echo_*` symbols are the small core compiler/runtime ABI used for language
//! semantics such as output writes and shutdown.
//! `echo_php_*` symbols are PHP builtin implementations used when codegen can
//! statically resolve a PHP function call.
//! `echo_ext_*` is reserved for a future extension/module ABI.
//! `echo_internal_*` symbols are runtime-private implementation details and must
//! not be emitted by codegen.

mod php_builtins;
mod std_intrinsics;

pub use php_builtins::{
    BuiltinCodegen, BuiltinLowering, PHP_BUILTINS, PHP_RUNTIME_HELPERS, PhpBuiltin, php_builtin,
};
pub use std_intrinsics::{STD_INTRINSICS, StdIntrinsic, std_intrinsic};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreRuntimeSymbol {
    Write,
    WriteValue,
    ValueString,
    ValueAdd,
    ValueSub,
    ValueMul,
    ValueDiv,
    ValueMod,
    ValuePow,
    ValueUnaryPlus,
    ValueUnaryMinus,
    ValueNot,
    ValueBool,
    ValueConcat,
    ValueLessThan,
    ValueIdentical,
    ValueOr,
    ValueListNew,
    ValueListAppend,
    ValueArrayNew,
    ValueArrayAppend,
    ValueArraySet,
    ValueArrayLen,
    ValueArrayKeyAt,
    ValueArrayValueAt,
    ValueIndexGet,
    ValueStringEqualsPtr,
    ValueObjectNew,
    ValueObjectSet,
    ValueObjectGet,
    TaskDefer,
    TaskRun,
    TaskJoin,
    TaskGroupNew,
    TaskGroupAdd,
    TaskGroupRunAndJoin,
    ThreadFork,
    ThreadForkTask,
    ThreadJoin,
    ProcessSpawn,
    ProcessJoin,
    Join,
    Require,
    RequireOnce,
    TaskSleepCurrent,
    TimeSleep,
    CallFunction,
    RegisterFunction,
    ValueExitStatus,
    Shutdown,
}

impl CoreRuntimeSymbol {
    pub const ALL: &'static [Self] = &[
        Self::Write,
        Self::WriteValue,
        Self::ValueString,
        Self::ValueAdd,
        Self::ValueSub,
        Self::ValueMul,
        Self::ValueDiv,
        Self::ValueMod,
        Self::ValuePow,
        Self::ValueUnaryPlus,
        Self::ValueUnaryMinus,
        Self::ValueNot,
        Self::ValueBool,
        Self::ValueConcat,
        Self::ValueLessThan,
        Self::ValueIdentical,
        Self::ValueOr,
        Self::ValueListNew,
        Self::ValueListAppend,
        Self::ValueArrayNew,
        Self::ValueArrayAppend,
        Self::ValueArraySet,
        Self::ValueArrayLen,
        Self::ValueArrayKeyAt,
        Self::ValueArrayValueAt,
        Self::ValueIndexGet,
        Self::ValueStringEqualsPtr,
        Self::ValueObjectNew,
        Self::ValueObjectSet,
        Self::ValueObjectGet,
        Self::TaskDefer,
        Self::TaskRun,
        Self::TaskJoin,
        Self::TaskGroupNew,
        Self::TaskGroupAdd,
        Self::TaskGroupRunAndJoin,
        Self::ThreadFork,
        Self::ThreadForkTask,
        Self::ThreadJoin,
        Self::ProcessSpawn,
        Self::ProcessJoin,
        Self::Join,
        Self::Require,
        Self::RequireOnce,
        Self::TaskSleepCurrent,
        Self::TimeSleep,
        Self::CallFunction,
        Self::RegisterFunction,
        Self::ValueExitStatus,
        Self::Shutdown,
    ];

    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Write => "echo_write",
            Self::WriteValue => "echo_write_value",
            Self::ValueString => "echo_value_string",
            Self::ValueAdd => "echo_value_add",
            Self::ValueSub => "echo_value_sub",
            Self::ValueMul => "echo_value_mul",
            Self::ValueDiv => "echo_value_div",
            Self::ValueMod => "echo_value_mod",
            Self::ValuePow => "echo_value_pow",
            Self::ValueUnaryPlus => "echo_value_unary_plus",
            Self::ValueUnaryMinus => "echo_value_unary_minus",
            Self::ValueNot => "echo_value_not",
            Self::ValueBool => "echo_value_bool",
            Self::ValueConcat => "echo_value_concat",
            Self::ValueLessThan => "echo_value_less_than",
            Self::ValueIdentical => "echo_value_identical",
            Self::ValueOr => "echo_value_or",
            Self::ValueListNew => "echo_value_list_new",
            Self::ValueListAppend => "echo_value_list_append",
            Self::ValueArrayNew => "echo_value_array_new",
            Self::ValueArrayAppend => "echo_value_array_append",
            Self::ValueArraySet => "echo_value_array_set",
            Self::ValueArrayLen => "echo_value_array_len",
            Self::ValueArrayKeyAt => "echo_value_array_key_at",
            Self::ValueArrayValueAt => "echo_value_array_value_at",
            Self::ValueIndexGet => "echo_value_index_get",
            Self::ValueStringEqualsPtr => "echo_value_string_equals_ptr",
            Self::ValueObjectNew => "echo_value_object_new",
            Self::ValueObjectSet => "echo_value_object_set",
            Self::ValueObjectGet => "echo_value_object_get",
            Self::TaskDefer => "echo_task_defer",
            Self::TaskRun => "echo_task_run",
            Self::TaskJoin => "echo_task_join",
            Self::TaskGroupNew => "echo_task_group_new",
            Self::TaskGroupAdd => "echo_task_group_add",
            Self::TaskGroupRunAndJoin => "echo_task_group_run_and_join",
            Self::ThreadFork => "echo_thread_fork",
            Self::ThreadForkTask => "echo_thread_fork_task",
            Self::ThreadJoin => "echo_thread_join",
            Self::ProcessSpawn => "echo_process_spawn",
            Self::ProcessJoin => "echo_process_join",
            Self::Join => "echo_join",
            Self::Require => "echo_php_require",
            Self::RequireOnce => "echo_php_require_once",
            Self::TaskSleepCurrent => "echo_task_sleep_current",
            Self::TimeSleep => "echo_time_sleep",
            Self::CallFunction => "echo_call_function",
            Self::RegisterFunction => "echo_reflection_register_function",
            Self::ValueExitStatus => "echo_value_exit_status",
            Self::Shutdown => "echo_shutdown",
        }
    }

    pub const fn signature(self) -> RuntimeSignature {
        match self {
            Self::Write => RuntimeSignature::VoidPtrI64,
            Self::WriteValue => RuntimeSignature::VoidEchoValue,
            Self::ValueString => RuntimeSignature::EchoValuePtrI64,
            Self::ValueAdd => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueSub => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueMul => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueDiv => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueMod => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValuePow => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueUnaryPlus => RuntimeSignature::EchoValueEchoValue,
            Self::ValueUnaryMinus => RuntimeSignature::EchoValueEchoValue,
            Self::ValueNot => RuntimeSignature::EchoValueEchoValue,
            Self::ValueBool => RuntimeSignature::BoolEchoValue,
            Self::ValueConcat => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueLessThan => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueIdentical => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueOr => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueListNew => RuntimeSignature::EchoValueNoArgs,
            Self::ValueListAppend => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueArrayNew => RuntimeSignature::EchoValueNoArgs,
            Self::ValueArrayAppend => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueArraySet => RuntimeSignature::EchoValueEchoValueEchoValueEchoValue,
            Self::ValueArrayLen => RuntimeSignature::I64EchoValue,
            Self::ValueArrayKeyAt => RuntimeSignature::EchoValueEchoValueI64,
            Self::ValueArrayValueAt => RuntimeSignature::EchoValueEchoValueI64,
            Self::ValueIndexGet => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::ValueStringEqualsPtr => RuntimeSignature::BoolEchoValuePtrI64,
            Self::ValueObjectNew => RuntimeSignature::EchoValueNoArgs,
            Self::ValueObjectSet => RuntimeSignature::EchoValueEchoValuePtrI64EchoValue,
            Self::ValueObjectGet => RuntimeSignature::EchoValueEchoValuePtrI64,
            Self::TaskDefer => RuntimeSignature::EchoValuePtr,
            Self::TaskRun => RuntimeSignature::EchoValueEchoValue,
            Self::TaskJoin => RuntimeSignature::EchoValueEchoValue,
            Self::TaskGroupNew => RuntimeSignature::EchoValueNoArgs,
            Self::TaskGroupAdd => RuntimeSignature::EchoValueEchoValueEchoValue,
            Self::TaskGroupRunAndJoin => RuntimeSignature::EchoValueEchoValue,
            Self::ThreadFork => RuntimeSignature::EchoValuePtr,
            Self::ThreadForkTask => RuntimeSignature::EchoValueEchoValue,
            Self::ThreadJoin => RuntimeSignature::EchoValueEchoValue,
            Self::ProcessSpawn => RuntimeSignature::EchoValueEchoValue,
            Self::ProcessJoin => RuntimeSignature::EchoValueEchoValue,
            Self::Join => RuntimeSignature::EchoValueEchoValue,
            Self::Require => RuntimeSignature::EchoValueEchoValue,
            Self::RequireOnce => RuntimeSignature::EchoValueEchoValue,
            Self::TaskSleepCurrent => RuntimeSignature::EchoValueI64Ptr,
            Self::TimeSleep => RuntimeSignature::VoidI64,
            Self::CallFunction => RuntimeSignature::EchoValuePtrI64,
            Self::RegisterFunction => RuntimeSignature::VoidPtrI64PtrI64PtrI64I32,
            Self::ValueExitStatus => RuntimeSignature::I32EchoValue,
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
    VoidPtrI64PtrI64PtrI64I32,
    VoidEchoValue,
    VoidEchoValueEchoValueEchoValue,
    BoolNoArgs,
    BoolEchoValue,
    BoolEchoValuePtrI64,
    I32EchoValue,
    I64EchoValue,
    EchoValueNoArgs,
    EchoValueI64Ptr,
    EchoValuePtr,
    EchoValuePtrI64,
    EchoValueEchoValue,
    EchoValueEchoValueI64,
    EchoValueEchoValuePtrI64,
    EchoValueEchoValueEchoValue,
    EchoValueEchoValueEchoValueEchoValue,
    EchoValueEchoValueEchoValueEchoValueEchoValue,
    EchoValueEchoValueEchoValueEchoValueEchoValueEchoValue,
    EchoValueEchoValuePtrI64EchoValue,
}

impl RuntimeSignature {
    pub fn llvm_decl(self, symbol: &str) -> String {
        match self {
            Self::VoidNoArgs => format!("declare void @{symbol}()"),
            Self::VoidI64 => format!("declare void @{symbol}(i64)"),
            Self::VoidPtrI64 => format!("declare void @{symbol}(ptr, i64)"),
            Self::VoidPtrI64PtrI64PtrI64I32 => {
                format!("declare void @{symbol}(ptr, i64, ptr, i64, ptr, i64, i32)")
            }
            Self::VoidEchoValue => format!("declare void @{symbol}(%EchoValue)"),
            Self::VoidEchoValueEchoValueEchoValue => {
                format!("declare void @{symbol}(%EchoValue, %EchoValue, %EchoValue)")
            }
            Self::BoolNoArgs => format!("declare i1 @{symbol}()"),
            Self::BoolEchoValue => format!("declare i1 @{symbol}(%EchoValue)"),
            Self::BoolEchoValuePtrI64 => format!("declare i1 @{symbol}(%EchoValue, ptr, i64)"),
            Self::I32EchoValue => format!("declare i32 @{symbol}(%EchoValue)"),
            Self::I64EchoValue => format!("declare i64 @{symbol}(%EchoValue)"),
            Self::EchoValueNoArgs => format!("declare %EchoValue @{symbol}()"),
            Self::EchoValueI64Ptr => format!("declare %EchoValue @{symbol}(i64, ptr)"),
            Self::EchoValuePtr => format!("declare %EchoValue @{symbol}(ptr)"),
            Self::EchoValuePtrI64 => format!("declare %EchoValue @{symbol}(ptr, i64)"),
            Self::EchoValueEchoValue => format!("declare %EchoValue @{symbol}(%EchoValue)"),
            Self::EchoValueEchoValueI64 => {
                format!("declare %EchoValue @{symbol}(%EchoValue, i64)")
            }
            Self::EchoValueEchoValuePtrI64 => {
                format!("declare %EchoValue @{symbol}(%EchoValue, ptr, i64)")
            }
            Self::EchoValueEchoValueEchoValue => {
                format!("declare %EchoValue @{symbol}(%EchoValue, %EchoValue)")
            }
            Self::EchoValueEchoValueEchoValueEchoValue => {
                format!("declare %EchoValue @{symbol}(%EchoValue, %EchoValue, %EchoValue)")
            }
            Self::EchoValueEchoValueEchoValueEchoValueEchoValue => {
                format!(
                    "declare %EchoValue @{symbol}(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
                )
            }
            Self::EchoValueEchoValueEchoValueEchoValueEchoValueEchoValue => {
                format!(
                    "declare %EchoValue @{symbol}(%EchoValue, %EchoValue, %EchoValue, %EchoValue, %EchoValue)"
                )
            }
            Self::EchoValueEchoValuePtrI64EchoValue => {
                format!("declare %EchoValue @{symbol}(%EchoValue, ptr, i64, %EchoValue)")
            }
        }
    }
}
