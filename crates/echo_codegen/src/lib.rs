mod abi;

use abi::{
    BuiltinCodegen, BuiltinLowering, CoreRuntimeSymbol, PHP_BUILTINS, PHP_RUNTIME_HELPERS,
    PhpBuiltin, STD_INTRINSICS, StdIntrinsic, php_builtin, std_intrinsic,
};
use echo_ast::{BinaryOp, ImportSource, Program, Stmt, TypedParam, UnaryOp};
use echo_diagnostics::Diagnostic;
use echo_source::Span;
use inkwell::OptimizationLevel as LlvmOptimizationLevel;
use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
enum RuntimeValue {
    StaticString(String),
    EchoValue(String),
}

const REFLECTION_SOURCE_PHP_BUILTIN: i32 = 1;
const REFLECTION_SOURCE_STD: i32 = 2;
const REFLECTION_SOURCE_USERLAND: i32 = 3;

fn stmt_span(statement: &Stmt) -> Span {
    match statement {
        Stmt::Echo(statement) => statement.span,
        Stmt::FunctionCall(statement) => statement.span,
        Stmt::DynamicFunctionCall(statement) => statement.span,
        Stmt::FunctionDecl(statement) => statement.span,
        Stmt::Assign(statement) => statement.span,
        Stmt::Let(statement) => statement.span,
        Stmt::AssignRef(statement) => statement.span,
        Stmt::Return(statement) => statement.span,
        Stmt::Yield(statement) => statement.span,
        Stmt::Expr(statement) => statement.span,
        Stmt::Namespace(statement) => statement.span,
        Stmt::Use(statement) => statement.span,
        Stmt::Import(statement) => statement.span,
        Stmt::ClassDecl(statement) => statement.span,
        Stmt::TypeDecl(statement) => statement.span,
        Stmt::Loop(statement) => statement.span,
        Stmt::If(statement) => statement.span,
        Stmt::Break(statement) => statement.span,
        Stmt::Append(statement) => statement.span,
    }
}

pub fn backend_name() -> &'static str {
    "llvm"
}

pub fn smoke_test_module_ir() -> String {
    let context = Context::create();
    let module = context.create_module("echo_smoke");

    module.print_to_string().to_string()
}

pub fn compile_to_ir(program: &Program) -> Result<String, Vec<Diagnostic>> {
    let hir = echo_hir::lower_program(program)?;
    let mir = echo_mir::lower_program(&hir)?;

    compile_mir_to_ir(&mir)
}

pub fn compile_hir_to_ir(program: &echo_hir::HirProgram) -> Result<String, Vec<Diagnostic>> {
    let mir = echo_mir::lower_program(program)?;

    compile_mir_to_ir(&mir)
}

pub fn compile_mir_to_ir(program: &echo_mir::MirProgram) -> Result<String, Vec<Diagnostic>> {
    let mut module = IrModule::new();
    module.source_dir = program.source_dir().map(str::to_string);
    let body = module.render_program(program)?;

    Ok(format!(
        r#"target triple = "x86_64-pc-linux-gnu"

%EchoValue = type {{ i32, i64 }}

{}
{}

{}

define i32 @main() {{
entry:
{}  call void @{}()
  ret i32 0
}}
"#,
        module.globals,
        runtime_declarations(),
        module.functions_ir,
        body,
        CoreRuntimeSymbol::Shutdown.symbol(),
    ))
}

pub fn run_program_jit(program: &Program) -> Result<i32, Vec<Diagnostic>> {
    let ir = compile_to_ir(program)?;
    run_ir_jit(&ir)
}

pub fn run_mir_jit(program: &echo_mir::MirProgram) -> Result<i32, Vec<Diagnostic>> {
    let ir = compile_mir_to_ir(program)?;
    run_ir_jit(&ir)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JitOutput {
    pub status: i32,
    pub stdout: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JitOptions {
    pub capture_stdout: bool,
    pub repl_inspect: bool,
}

pub fn run_ir_jit_with_options(
    ir: &str,
    options: JitOptions,
) -> Result<JitOutput, Vec<Diagnostic>> {
    if options.capture_stdout {
        let (status, stdout) =
            echo_runtime::capture_stdout(options.repl_inspect, || run_ir_jit(ir));
        let status = status?;

        Ok(JitOutput { status, stdout })
    } else {
        Ok(JitOutput {
            status: run_ir_jit(ir)?,
            stdout: Vec::new(),
        })
    }
}

fn run_ir_jit(ir: &str) -> Result<i32, Vec<Diagnostic>> {
    let context = Context::create();
    let mut ir_bytes = ir.as_bytes().to_vec();
    ir_bytes.push(0);
    let memory_buffer = MemoryBuffer::create_from_memory_range_copy(&ir_bytes, "echo_jit");
    let module = context
        .create_module_from_ir(memory_buffer)
        .map_err(|err| jit_diagnostic(format!("failed to parse generated LLVM IR: {err}")))?;
    let execution_engine = module
        .create_jit_execution_engine(LlvmOptimizationLevel::None)
        .map_err(|err| jit_diagnostic(format!("failed to create LLVM JIT engine: {err}")))?;

    register_jit_runtime_symbols(&module, &execution_engine)?;

    type Main = unsafe extern "C" fn() -> i32;
    let main = unsafe {
        execution_engine
            .get_function::<Main>("main")
            .map_err(|err| jit_diagnostic(format!("failed to find JIT main function: {err:?}")))?
    };

    Ok(unsafe { main.call() })
}

fn register_jit_runtime_symbols(
    module: &inkwell::module::Module<'_>,
    execution_engine: &inkwell::execution_engine::ExecutionEngine<'_>,
) -> Result<(), Vec<Diagnostic>> {
    let addresses = jit_runtime_symbol_addresses()
        .into_iter()
        .collect::<HashMap<_, _>>();
    let mut missing = Vec::new();

    for function in module.get_functions() {
        if function.count_basic_blocks() != 0 {
            continue;
        }

        let Ok(symbol) = function.get_name().to_str() else {
            continue;
        };

        if !symbol.starts_with("echo_") {
            continue;
        }

        if let Some(address) = addresses.get(symbol) {
            execution_engine.add_global_mapping(&function, *address);
        } else {
            missing.push(symbol.to_string());
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        missing.sort();
        missing.dedup();
        Err(jit_diagnostic(format!(
            "missing JIT runtime symbol mappings: {}",
            missing.join(", ")
        )))
    }
}

fn jit_runtime_symbol_addresses() -> Vec<(&'static str, usize)> {
    vec![
        (
            "echo_write",
            echo_runtime::echo_write as unsafe extern "C" fn(*const u8, usize) as usize,
        ),
        (
            "echo_write_value",
            echo_runtime::echo_write_value as unsafe extern "C" fn(echo_runtime::EchoValue)
                as usize,
        ),
        (
            "echo_value_string",
            echo_runtime::echo_value_string
                as unsafe extern "C" fn(*const u8, usize) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_add",
            echo_runtime::echo_value_add
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_sub",
            echo_runtime::echo_value_sub
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_mul",
            echo_runtime::echo_value_mul
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_div",
            echo_runtime::echo_value_div
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_mod",
            echo_runtime::echo_value_mod
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_pow",
            echo_runtime::echo_value_pow
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_unary_plus",
            echo_runtime::echo_value_unary_plus
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_unary_minus",
            echo_runtime::echo_value_unary_minus
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_bool",
            echo_runtime::echo_value_bool as extern "C" fn(echo_runtime::EchoValue) -> bool
                as usize,
        ),
        (
            "echo_value_concat",
            echo_runtime::echo_value_concat
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_list_new",
            echo_runtime::echo_value_list_new as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_list_append",
            echo_runtime::echo_value_list_append
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_array_new",
            echo_runtime::echo_value_array_new as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_array_append",
            echo_runtime::echo_value_array_append
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_array_set",
            echo_runtime::echo_value_array_set
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_index_get",
            echo_runtime::echo_value_index_get
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_object_new",
            echo_runtime::echo_value_object_new as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_value_object_set",
            echo_runtime::echo_value_object_set
                as unsafe extern "C" fn(
                    echo_runtime::EchoValue,
                    *const u8,
                    usize,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_value_object_get",
            echo_runtime::echo_value_object_get
                as unsafe extern "C" fn(
                    echo_runtime::EchoValue,
                    *const u8,
                    usize,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_task_defer",
            echo_runtime::echo_task_defer
                as extern "C" fn(
                    Option<echo_runtime::task::EchoTaskCallback>,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_task_run",
            echo_runtime::echo_task_run
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_task_join",
            echo_runtime::echo_task_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_task_group_new",
            echo_runtime::echo_task_group_new as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_task_group_add",
            echo_runtime::echo_task_group_add
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_task_group_run_and_join",
            echo_runtime::echo_task_group_run_and_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_thread_fork",
            echo_runtime::echo_thread_fork
                as extern "C" fn(
                    Option<echo_runtime::task::EchoTaskCallback>,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_thread_fork_task",
            echo_runtime::echo_thread_fork_task
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_thread_join",
            echo_runtime::echo_thread_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_process_spawn",
            echo_runtime::echo_process_spawn
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_process_join",
            echo_runtime::echo_process_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_join",
            echo_runtime::echo_join
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_require",
            echo_runtime::echo_php_require
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_require_once",
            echo_runtime::echo_php_require_once
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_task_sleep_current",
            echo_runtime::echo_task_sleep_current
                as extern "C" fn(
                    i64,
                    Option<echo_runtime::task::EchoTaskCallback>,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_time_sleep",
            echo_runtime::echo_time_sleep as extern "C" fn(i64) as usize,
        ),
        (
            "echo_call_function",
            echo_runtime::echo_call_function
                as unsafe extern "C" fn(*const u8, usize) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_reflection_register_function",
            echo_runtime::echo_reflection_register_function
                as unsafe extern "C" fn(*const u8, usize, *const u8, usize, *const u8, usize, i32)
                as usize,
        ),
        (
            "echo_php_abs",
            echo_runtime::echo_php_abs
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_flush",
            echo_runtime::echo_php_flush as extern "C" fn() as usize,
        ),
        (
            "echo_php_ob_implicit_flush",
            echo_runtime::echo_php_ob_implicit_flush as extern "C" fn(echo_runtime::EchoValue)
                as usize,
        ),
        (
            "echo_php_ob_start",
            echo_runtime::echo_php_ob_start as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_start_value",
            echo_runtime::echo_php_ob_start_value as extern "C" fn(echo_runtime::EchoValue) -> bool
                as usize,
        ),
        (
            "echo_php_ob_flush",
            echo_runtime::echo_php_ob_flush as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_clean",
            echo_runtime::echo_php_ob_clean as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_end_flush",
            echo_runtime::echo_php_ob_end_flush as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_end_clean",
            echo_runtime::echo_php_ob_end_clean as extern "C" fn() -> bool as usize,
        ),
        (
            "echo_php_ob_get_clean",
            echo_runtime::echo_php_ob_get_clean as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ob_get_contents",
            echo_runtime::echo_php_ob_get_contents as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ob_get_flush",
            echo_runtime::echo_php_ob_get_flush as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ob_get_level",
            echo_runtime::echo_php_ob_get_level as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ob_get_length",
            echo_runtime::echo_php_ob_get_length as extern "C" fn() -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strlen",
            echo_runtime::echo_php_strlen
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_define",
            echo_runtime::echo_php_define
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_microtime",
            echo_runtime::echo_php_microtime
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_count",
            echo_runtime::echo_php_count
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_values",
            echo_runtime::echo_php_array_values
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_keys",
            echo_runtime::echo_php_array_keys
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_array_sum",
            echo_runtime::echo_php_array_sum
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_product",
            echo_runtime::echo_php_array_product
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_function_exists",
            echo_runtime::echo_php_function_exists
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_gettype",
            echo_runtime::echo_php_gettype
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_array_is_list",
            echo_runtime::echo_php_array_is_list
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_array",
            echo_runtime::echo_php_is_array
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_countable",
            echo_runtime::echo_php_is_countable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_iterable",
            echo_runtime::echo_php_is_iterable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_numeric",
            echo_runtime::echo_php_is_numeric
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_null",
            echo_runtime::echo_php_is_null
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_bool",
            echo_runtime::echo_php_is_bool
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_callable",
            echo_runtime::echo_php_is_callable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_int",
            echo_runtime::echo_php_is_int
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_float",
            echo_runtime::echo_php_is_float
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_finite",
            echo_runtime::echo_php_is_finite
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_infinite",
            echo_runtime::echo_php_is_infinite
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_nan",
            echo_runtime::echo_php_is_nan
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_object",
            echo_runtime::echo_php_is_object
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_resource",
            echo_runtime::echo_php_is_resource
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_string",
            echo_runtime::echo_php_is_string
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_scalar",
            echo_runtime::echo_php_is_scalar
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strval",
            echo_runtime::echo_php_strval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_boolval",
            echo_runtime::echo_php_boolval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_intval",
            echo_runtime::echo_php_intval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_floatval",
            echo_runtime::echo_php_floatval
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strtoupper",
            echo_runtime::echo_php_strtoupper
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strtolower",
            echo_runtime::echo_php_strtolower
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ucwords",
            echo_runtime::echo_php_ucwords
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_strrev",
            echo_runtime::echo_php_strrev
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ucfirst",
            echo_runtime::echo_php_ucfirst
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_lcfirst",
            echo_runtime::echo_php_lcfirst
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ord",
            echo_runtime::echo_php_ord
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_str_rot13",
            echo_runtime::echo_php_str_rot13
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_chr",
            echo_runtime::echo_php_chr
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_decbin",
            echo_runtime::echo_php_decbin
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_dechex",
            echo_runtime::echo_php_dechex
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_decoct",
            echo_runtime::echo_php_decoct
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_bindec",
            echo_runtime::echo_php_bindec
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_hexdec",
            echo_runtime::echo_php_hexdec
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_octdec",
            echo_runtime::echo_php_octdec
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_base_convert",
            echo_runtime::echo_php_base_convert
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_deg2rad",
            echo_runtime::echo_php_deg2rad
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_rad2deg",
            echo_runtime::echo_php_rad2deg
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_sin",
            echo_runtime::echo_php_sin
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_cos",
            echo_runtime::echo_php_cos
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_tan",
            echo_runtime::echo_php_tan
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_asin",
            echo_runtime::echo_php_asin
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_acos",
            echo_runtime::echo_php_acos
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_atan",
            echo_runtime::echo_php_atan
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_atan2",
            echo_runtime::echo_php_atan2
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_ceil",
            echo_runtime::echo_php_ceil
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_floor",
            echo_runtime::echo_php_floor
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_sqrt",
            echo_runtime::echo_php_sqrt
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_exp",
            echo_runtime::echo_php_exp
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_expm1",
            echo_runtime::echo_php_expm1
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_log",
            echo_runtime::echo_php_log
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_log10",
            echo_runtime::echo_php_log10
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_log1p",
            echo_runtime::echo_php_log1p
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_pow",
            echo_runtime::echo_php_pow
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hypot",
            echo_runtime::echo_php_hypot
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_pi",
            echo_runtime::echo_php_pi as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_fmod",
            echo_runtime::echo_php_fmod
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_bin2hex",
            echo_runtime::echo_php_bin2hex
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_base64_encode",
            echo_runtime::echo_php_base64_encode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_base64_decode",
            echo_runtime::echo_php_base64_decode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_rawurlencode",
            echo_runtime::echo_php_rawurlencode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_rawurldecode",
            echo_runtime::echo_php_rawurldecode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_urlencode",
            echo_runtime::echo_php_urlencode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_urldecode",
            echo_runtime::echo_php_urldecode
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_sinh",
            echo_runtime::echo_php_sinh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_cosh",
            echo_runtime::echo_php_cosh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_tanh",
            echo_runtime::echo_php_tanh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_asinh",
            echo_runtime::echo_php_asinh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_acosh",
            echo_runtime::echo_php_acosh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_atanh",
            echo_runtime::echo_php_atanh
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_basename",
            echo_runtime::echo_php_basename
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_dirname",
            echo_runtime::echo_php_dirname
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_hex2bin",
            echo_runtime::echo_php_hex2bin
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_escapeshellarg",
            echo_runtime::echo_php_escapeshellarg
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_escapeshellcmd",
            echo_runtime::echo_php_escapeshellcmd
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_explode",
            echo_runtime::echo_php_explode
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_implode",
            echo_runtime::echo_php_implode
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_file_exists",
            echo_runtime::echo_php_file_exists
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_chdir",
            echo_runtime::echo_php_chdir
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_getcwd",
            echo_runtime::echo_php_getcwd as extern "C" fn() -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_is_dir",
            echo_runtime::echo_php_is_dir
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_file",
            echo_runtime::echo_php_is_file
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_link",
            echo_runtime::echo_php_is_link
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_readable",
            echo_runtime::echo_php_is_readable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_writable",
            echo_runtime::echo_php_is_writable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_is_executable",
            echo_runtime::echo_php_is_executable
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_filesize",
            echo_runtime::echo_php_filesize
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_realpath",
            echo_runtime::echo_php_realpath
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_trim",
            echo_runtime::echo_php_trim
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_ltrim",
            echo_runtime::echo_php_ltrim
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_rtrim",
            echo_runtime::echo_php_rtrim
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_addslashes",
            echo_runtime::echo_php_addslashes
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_stripslashes",
            echo_runtime::echo_php_stripslashes
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_quotemeta",
            echo_runtime::echo_php_quotemeta
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_php_str_contains",
            echo_runtime::echo_php_str_contains
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_starts_with",
            echo_runtime::echo_php_str_starts_with
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_ends_with",
            echo_runtime::echo_php_str_ends_with
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_repeat",
            echo_runtime::echo_php_str_repeat
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_pad",
            echo_runtime::echo_php_str_pad
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_str_split",
            echo_runtime::echo_php_str_split
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_chunk_split",
            echo_runtime::echo_php_chunk_split
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_substr",
            echo_runtime::echo_php_substr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strpos",
            echo_runtime::echo_php_strpos
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stripos",
            echo_runtime::echo_php_stripos
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strrpos",
            echo_runtime::echo_php_strrpos
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strripos",
            echo_runtime::echo_php_strripos
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strstr",
            echo_runtime::echo_php_strstr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_stristr",
            echo_runtime::echo_php_stristr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strrchr",
            echo_runtime::echo_php_strrchr
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strpbrk",
            echo_runtime::echo_php_strpbrk
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strspn",
            echo_runtime::echo_php_strspn
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strcspn",
            echo_runtime::echo_php_strcspn
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_substr_count",
            echo_runtime::echo_php_substr_count
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_substr_compare",
            echo_runtime::echo_php_substr_compare
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strcmp",
            echo_runtime::echo_php_strcmp
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strcasecmp",
            echo_runtime::echo_php_strcasecmp
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strncmp",
            echo_runtime::echo_php_strncmp
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_php_strncasecmp",
            echo_runtime::echo_php_strncasecmp
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_std_assert_ok",
            echo_runtime::echo_std_assert_ok
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_assert_equals",
            echo_runtime::echo_std_assert_equals
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_std_http_response_text",
            echo_runtime::echo_std_http_response_text
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_http_read_request",
            echo_runtime::echo_std_http_read_request
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_net_listen",
            echo_runtime::echo_std_net_listen
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_net_connect",
            echo_runtime::echo_std_net_connect
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_net_accept",
            echo_runtime::echo_std_net_accept
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_net_read",
            echo_runtime::echo_std_net_read
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_std_net_write",
            echo_runtime::echo_std_net_write
                as extern "C" fn(
                    echo_runtime::EchoValue,
                    echo_runtime::EchoValue,
                ) -> echo_runtime::EchoValue as usize,
        ),
        (
            "echo_std_net_close",
            echo_runtime::echo_std_net_close
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_reflect_exists",
            echo_runtime::echo_std_reflect_exists
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_reflect_params",
            echo_runtime::echo_std_reflect_params
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_reflect_return_type",
            echo_runtime::echo_std_reflect_return_type
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_std_reflect_type_of",
            echo_runtime::echo_std_reflect_type_of
                as extern "C" fn(echo_runtime::EchoValue) -> echo_runtime::EchoValue
                as usize,
        ),
        (
            "echo_shutdown",
            echo_runtime::echo_shutdown as extern "C" fn() as usize,
        ),
    ]
}

fn jit_diagnostic(message: String) -> Vec<Diagnostic> {
    vec![Diagnostic::new(message, Span::new(0, 0))]
}

struct IrModule {
    globals: String,
    functions_ir: String,
    aliases: HashMap<String, String>,
    std_imports: HashMap<String, String>,
    locals: HashMap<String, RuntimeValue>,
    functions: HashMap<String, echo_mir::MirFunction>,
    source_dir: Option<String>,
    returned: bool,
    terminated: bool,
    break_labels: Vec<String>,
    break_value_slots: Vec<Option<String>>,
    next_string_id: usize,
    next_call_id: usize,
    next_defer_id: usize,
    next_loop_id: usize,
    next_if_id: usize,
}

impl IrModule {
    fn new() -> Self {
        Self {
            globals: String::new(),
            functions_ir: String::new(),
            aliases: HashMap::new(),
            std_imports: HashMap::new(),
            locals: HashMap::new(),
            functions: HashMap::new(),
            source_dir: None,
            returned: false,
            terminated: false,
            break_labels: Vec::new(),
            break_value_slots: Vec::new(),
            next_string_id: 0,
            next_call_id: 0,
            next_defer_id: 0,
            next_loop_id: 0,
            next_if_id: 0,
        }
    }

    fn render_program(
        &mut self,
        program: &echo_mir::MirProgram,
    ) -> Result<String, Vec<Diagnostic>> {
        let mut body = String::new();
        let mut diagnostics = Vec::new();

        for statement in program.imports() {
            if let Err(diagnostic) = self.register_std_import(statement) {
                diagnostics.push(diagnostic);
            }
        }

        for statement in program.functions() {
            self.functions
                .insert(statement.name.clone(), statement.clone());
        }

        for function in self.functions.clone().into_values() {
            if let Err(diagnostic) = self.render_userland_function(&function) {
                diagnostics.push(diagnostic);
            }
        }

        self.render_reflection_registrations(&mut body);

        for statement in program.statements() {
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                diagnostics.push(diagnostic);
            }
        }

        if diagnostics.is_empty() {
            Ok(body)
        } else {
            Err(diagnostics)
        }
    }

    fn render_reflection_registrations(&mut self, body: &mut String) {
        let php_builtins = echo_reflection::php_builtins()
            .iter()
            .map(|function| {
                (
                    function.qualified_name.clone(),
                    function.params_signature(),
                    function.return_type_signature().to_string(),
                    REFLECTION_SOURCE_PHP_BUILTIN,
                )
            })
            .collect::<Vec<_>>();
        for (name, params, return_type, source_kind) in php_builtins {
            self.register_function_reflection(body, &name, &params, &return_type, source_kind);
        }

        let std_functions = echo_reflection::functions()
            .iter()
            .filter(|function| function.source == echo_reflection::FunctionSource::Std)
            .map(|function| {
                (
                    function.qualified_name.clone(),
                    function.params_signature(),
                    function.return_type_signature().to_string(),
                    REFLECTION_SOURCE_STD,
                )
            })
            .collect::<Vec<_>>();
        for (name, params, return_type, source_kind) in std_functions {
            self.register_function_reflection(body, &name, &params, &return_type, source_kind);
        }

        let mut functions = self.functions.values().cloned().collect::<Vec<_>>();
        functions.sort_by(|left, right| left.name.cmp(&right.name));

        for function in &functions {
            let name = function.name.clone();
            let params = function_params_signature(&function.params);
            let return_type = function.return_type.clone().unwrap_or_default();

            self.register_function_reflection(
                body,
                &name,
                &params,
                &return_type,
                REFLECTION_SOURCE_USERLAND,
            );
        }
    }

    fn register_function_reflection(
        &mut self,
        body: &mut String,
        name: &str,
        params: &str,
        return_type: &str,
        source_kind: i32,
    ) {
        let name_global = self.string_global(name);
        let params_global = self.string_global(params);
        let return_type_global = self.string_global(return_type);

        body.push_str(&format!(
            "  call void @{}(ptr @{name_global}, i64 {}, ptr @{params_global}, i64 {}, ptr @{return_type_global}, i64 {}, i32 {source_kind})\n",
            CoreRuntimeSymbol::RegisterFunction.symbol(),
            name.len(),
            params.len(),
            return_type.len()
        ));
    }

    fn render_userland_function(
        &mut self,
        function: &echo_mir::MirFunction,
    ) -> Result<(), Diagnostic> {
        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_returned = self.returned;
        let saved_terminated = self.terminated;
        self.returned = false;
        self.terminated = false;

        for param in &function.params {
            self.locals.insert(
                param.name.clone(),
                RuntimeValue::EchoValue(format!("%arg_{}", param.name)),
            );
        }

        let mut body = String::new();

        for statement in &function.body {
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                self.aliases = saved_aliases;
                self.locals = saved_locals;
                self.returned = saved_returned;
                self.terminated = saved_terminated;
                return Err(diagnostic);
            }
        }

        let returned = self.returned;

        self.aliases = saved_aliases;
        self.locals = saved_locals;
        self.returned = saved_returned;
        self.terminated = saved_terminated;

        let params = function
            .params
            .iter()
            .map(|param| format!("%EchoValue %arg_{}", param.name))
            .collect::<Vec<_>>()
            .join(", ");

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{}({params}) {{\nentry:\n{}{}\n}}\n",
            userland_function_symbol(&function.name),
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(())
    }

    fn render_mir_stmt(
        &mut self,
        body: &mut String,
        statement: &echo_mir::MirStmt,
    ) -> Result<(), Diagnostic> {
        match statement {
            echo_mir::MirStmt::Echo { exprs, .. } => {
                for expr in exprs {
                    let value = self.render_mir_expr(body, expr)?;
                    self.write_value(body, value);
                }
                Ok(())
            }
            echo_mir::MirStmt::FunctionCall { call, .. } => {
                self.render_mir_function_call_stmt(body, call)
            }
            echo_mir::MirStmt::DynamicFunctionCall { source, name, args } => {
                self.render_mir_dynamic_function_call(body, source, name, args)
            }
            echo_mir::MirStmt::Assign { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let name = self.resolve_alias(name);
                self.locals.insert(name, value);
                Ok(())
            }
            echo_mir::MirStmt::Let { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let name = self.resolve_alias(name);
                self.locals.insert(name, value);
                Ok(())
            }
            echo_mir::MirStmt::Return { value, .. } => {
                let value = self.render_mir_expr_as_echo_value(body, value)?;
                body.push_str(&format!("  ret {value}\n"));
                self.returned = true;
                self.terminated = true;
                Ok(())
            }
            echo_mir::MirStmt::Expr { expr, .. } => {
                self.render_mir_expr(body, expr)?;
                Ok(())
            }
            echo_mir::MirStmt::Loop {
                body: loop_body, ..
            } => self.render_mir_loop_stmt(body, loop_body),
            echo_mir::MirStmt::If {
                condition,
                body: if_body,
                ..
            } => self.render_mir_if_stmt(body, condition, if_body),
            echo_mir::MirStmt::Break { source, value } => {
                self.render_mir_break_stmt(body, source, value.as_ref())
            }
            echo_mir::MirStmt::Append {
                source,
                target,
                value,
            } => self.render_mir_append_stmt(body, source, target, value),
            echo_mir::MirStmt::AssignRef {
                source,
                name,
                target,
            } => self.render_mir_assign_ref_stmt(source, name, target),
            echo_mir::MirStmt::Yield { source, .. } => Err(Diagnostic::new(
                "unsupported yield statement in LLVM codegen",
                stmt_span(source),
            )),
            echo_mir::MirStmt::Noop { .. } => Ok(()),
        }
    }

    fn render_mir_function_call_stmt(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<(), Diagnostic> {
        let name = self.resolve_std_call_name(&call.name, call.span)?;
        if name == "time.sleep" {
            self.mir_time_sleep_call(body, &call.args, call.span)?;
        } else if let Some(intrinsic) = std_intrinsic(&name) {
            self.mir_std_intrinsic_call(body, intrinsic, &call.args, call.span)?;
        } else {
            match php_builtin(&name) {
                Some(builtin) if builtin.lowering == BuiltinLowering::DirectRuntimeCall => {
                    self.mir_php_builtin_call(body, builtin, &call.args, call.span)?
                }
                None => self.mir_userland_call(body, call)?,
                Some(_) => self.mir_userland_call(body, call)?,
            }
        }

        Ok(())
    }

    fn render_mir_loop_stmt(
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
        self.break_value_slots.pop();
        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));

        Ok(())
    }

    fn render_mir_break_stmt(
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

    fn render_mir_if_stmt(
        &mut self,
        body: &mut String,
        condition_expr: &echo_mir::MirExpr,
        statements: &[echo_mir::MirStmt],
    ) -> Result<(), Diagnostic> {
        let condition = self.render_mir_condition(body, condition_expr)?;
        let if_id = self.next_if_id;
        self.next_if_id += 1;
        let then_label = format!("if_then_{if_id}");
        let after_label = format!("if_after_{if_id}");

        body.push_str(&format!(
            "  br i1 {condition}, label %{then_label}, label %{after_label}\n\n{then_label}:\n"
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

        self.terminated = saved_terminated;
        body.push_str(&format!("\n{after_label}:\n"));

        Ok(())
    }

    fn render_mir_append_stmt(
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

    fn render_mir_assign_ref_stmt(
        &mut self,
        source: &Stmt,
        name: &str,
        target: &str,
    ) -> Result<(), Diagnostic> {
        let resolved_target = self.resolve_alias(target);
        if self.locals.contains_key(&resolved_target) {
            self.aliases.insert(name.to_string(), resolved_target);
            Ok(())
        } else {
            Err(Diagnostic::new(
                format!("unsupported reference to undefined variable `${target}` in LLVM codegen"),
                stmt_span(source),
            ))
        }
    }

    fn render_mir_loop_expr(
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

    fn mir_userland_call(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<(), Diagnostic> {
        let Some(function) = self.functions.get(&call.name).cloned() else {
            return Err(Diagnostic::new(
                format!("unsupported function `{}` in LLVM codegen", call.name),
                call.span,
            ));
        };

        if call.args.len() != function.params.len() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for userland function `{}` in LLVM codegen",
                    call.name
                ),
                call.span,
            ));
        }

        let mut args = Vec::new();
        for arg in &call.args {
            args.push(self.render_mir_expr_as_echo_value(body, arg)?);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}({})\n",
            userland_function_symbol(&call.name),
            args.join(", ")
        ));

        Ok(())
    }

    fn mir_time_sleep_call(
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

    fn mir_std_intrinsic_call(
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

        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({})\n",
            intrinsic.symbol,
            rendered_args.join(", ")
        ));

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

    fn render_mir_expr_as_echo_value(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
    ) -> Result<String, Diagnostic> {
        let value = self.render_mir_expr(body, expr)?;
        Ok(self.runtime_value_as_echo_value(body, value))
    }

    fn runtime_value_as_echo_value(&mut self, body: &mut String, value: RuntimeValue) -> String {
        match value {
            RuntimeValue::EchoValue(name) => format!("%EchoValue {name}"),
            RuntimeValue::StaticString(value) => {
                let global = self.string_global(&value);
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
                    CoreRuntimeSymbol::ValueString.symbol(),
                    value.len()
                ));

                format!("%EchoValue {name}")
            }
        }
    }

    fn render_mir_dynamic_function_call(
        &mut self,
        body: &mut String,
        source: &Stmt,
        name: &str,
        args: &[echo_mir::MirExpr],
    ) -> Result<(), Diagnostic> {
        if !args.is_empty() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported arguments for dynamic function call `${name}` in LLVM codegen"
                ),
                stmt_span(source),
            ));
        }

        let RuntimeValue::StaticString(function_name) = self
            .locals
            .get(&self.resolve_alias(name))
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    format!("unsupported undefined dynamic function `${name}` in LLVM codegen"),
                    stmt_span(source),
                )
            })?
        else {
            return Err(Diagnostic::new(
                format!("unsupported non-string dynamic function `${name}` in LLVM codegen"),
                stmt_span(source),
            ));
        };

        let global = self.string_global(&function_name);
        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
            CoreRuntimeSymbol::CallFunction.symbol(),
            function_name.len()
        ));

        Ok(())
    }

    fn write_call(&mut self, body: &mut String, value: &str) {
        let global = self.string_global(value);
        body.push_str(&format!(
            "  call void @{}(ptr @{global}, i64 {})\n",
            CoreRuntimeSymbol::Write.symbol(),
            value.len()
        ));
    }

    fn write_value(&mut self, body: &mut String, value: RuntimeValue) {
        match value {
            RuntimeValue::StaticString(value) => self.write_call(body, &value),
            RuntimeValue::EchoValue(name) => body.push_str(&format!(
                "  call void @{}(%EchoValue {name})\n",
                CoreRuntimeSymbol::WriteValue.symbol()
            )),
        }
    }

    fn mir_php_builtin_call(
        &mut self,
        body: &mut String,
        builtin: PhpBuiltin,
        args: &[echo_mir::MirExpr],
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
                [echo_mir::MirExpr::Null { .. }] => {
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
                [echo_mir::MirExpr::String { value, .. }] => {
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
            | BuiltinCodegen::Basename
            | BuiltinCodegen::Dirname
            | BuiltinCodegen::ChunkSplit
            | BuiltinCodegen::Implode
            | BuiltinCodegen::Log
            | BuiltinCodegen::StrPad
            | BuiltinCodegen::StrSplit
            | BuiltinCodegen::ValueTernaryExpression
            | BuiltinCodegen::Explode
            | BuiltinCodegen::SubstrCompare => {
                unreachable!("expression builtin used as statement call")
            }
        }

        Ok(())
    }

    fn render_mir_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirExpr::Null { .. } => {
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::Bool { value, .. } => Ok(RuntimeValue::EchoValue(format!(
                "{{ i32 1, i64 {} }}",
                *value as u8
            ))),
            echo_mir::MirExpr::String { value, .. } => {
                Ok(RuntimeValue::StaticString(value.clone()))
            }
            echo_mir::MirExpr::Number { source, value } => {
                if let Ok(value) = value.parse::<i64>() {
                    Ok(RuntimeValue::EchoValue(format!("{{ i32 2, i64 {value} }}")))
                } else if let Ok(value) = value.parse::<f64>() {
                    Ok(RuntimeValue::EchoValue(format!(
                        "{{ i32 11, i64 {} }}",
                        value.to_bits() as i64
                    )))
                } else {
                    Err(Diagnostic::new(
                        "unsupported numeric literal in LLVM codegen",
                        source.span(),
                    ))
                }
            }
            echo_mir::MirExpr::Variable { source, name } => self
                .locals
                .get(&self.resolve_alias(name))
                .cloned()
                .ok_or_else(|| {
                    Diagnostic::new(
                        format!("unsupported undefined variable `${name}` in LLVM codegen"),
                        source.span(),
                    )
                }),
            echo_mir::MirExpr::FunctionCall { call, .. } => {
                self.render_mir_function_call_expr(body, call)
            }
            echo_mir::MirExpr::MethodCall { object, args, .. } => {
                self.render_mir_expr_as_echo_value(body, object)?;
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::StaticCall { args, .. } => {
                for arg in args {
                    self.render_mir_expr_as_echo_value(body, arg)?;
                }
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            echo_mir::MirExpr::Assign { name, value, .. } => {
                let value = self.render_mir_expr(body, value)?;
                let resolved = self.resolve_alias(name);
                self.locals.insert(resolved, value.clone());
                Ok(value)
            }
            echo_mir::MirExpr::MagicDir { .. } => Ok(RuntimeValue::StaticString(
                self.source_dir.clone().unwrap_or_else(|| ".".to_string()),
            )),
            echo_mir::MirExpr::Require { once, path, .. } => {
                let path = self.render_mir_expr_as_echo_value(body, path)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let symbol = if *once {
                    CoreRuntimeSymbol::RequireOnce
                } else {
                    CoreRuntimeSymbol::Require
                };
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}({path})\n",
                    symbol.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirExpr::Defer { body: block, .. } => {
                let function = self.render_mir_defer_function(body, block)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::TaskDefer.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirExpr::Run { source, expr } => self.render_mir_run_expr(body, source, expr),
            echo_mir::MirExpr::Join { handle, .. } => {
                self.render_mir_task_unary_expr(body, handle, CoreRuntimeSymbol::Join)
            }
            echo_mir::MirExpr::Loop { body: block, .. } => self.render_mir_loop_expr(body, block),
            echo_mir::MirExpr::Fork { source, expr } => {
                self.render_mir_fork_expr(body, source, expr)
            }
            echo_mir::MirExpr::Spawn { command, .. } => {
                self.render_mir_task_unary_expr(body, command, CoreRuntimeSymbol::ProcessSpawn)
            }
            echo_mir::MirExpr::Binary {
                source,
                left,
                op,
                right,
            } if *op == BinaryOp::Concat => self.render_mir_concat_expr(body, left, right),
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Add => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueAdd)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Sub => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueSub)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Mul => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueMul)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Div => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueDiv)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Mod => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValueMod)
            }
            echo_mir::MirExpr::Binary {
                left, op, right, ..
            } if *op == BinaryOp::Pow => {
                self.render_mir_numeric_binary_expr(body, left, right, CoreRuntimeSymbol::ValuePow)
            }
            echo_mir::MirExpr::Unary { op, expr, .. } => self.render_mir_numeric_unary_expr(
                body,
                expr,
                match op {
                    UnaryOp::Plus => CoreRuntimeSymbol::ValueUnaryPlus,
                    UnaryOp::Minus => CoreRuntimeSymbol::ValueUnaryMinus,
                },
            ),
            echo_mir::MirExpr::Field { object, field, .. } => {
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
            echo_mir::MirExpr::Index {
                collection, index, ..
            } => {
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
            echo_mir::MirExpr::Object { fields, .. } => self.render_mir_object_expr(body, fields),
            echo_mir::MirExpr::List { values, .. } => self.render_mir_list_values(body, values),
            echo_mir::MirExpr::Array { elements, .. } => self.render_mir_array_expr(body, elements),
            echo_mir::MirExpr::Binary { source, .. } => Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                source.span(),
            )),
        }
    }

    fn render_mir_array_expr(
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

    fn render_mir_list_values(
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

    fn render_mir_object_expr(
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

    fn render_mir_concat_expr(
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

    fn render_mir_numeric_binary_expr(
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

    fn render_mir_numeric_unary_expr(
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

    fn render_mir_defer_function(
        &mut self,
        caller_body: &mut String,
        statements: &[echo_mir::MirStmt],
    ) -> Result<String, Diagnostic> {
        let function = format!("echo_defer_{}", self.next_defer_id);
        self.next_defer_id += 1;

        let captures = self
            .locals
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();

        for (name, value) in &captures {
            let global = format!("echo_capture_{}_{}", function, name);
            self.globals
                .push_str(&format!("@{global} = global %EchoValue zeroinitializer\n"));
            let value = self.runtime_value_as_echo_value(caller_body, value.clone());
            caller_body.push_str(&format!("  store {value}, ptr @{global}\n"));
        }

        let saved_aliases = std::mem::take(&mut self.aliases);
        let saved_locals = std::mem::take(&mut self.locals);
        let saved_returned = self.returned;
        self.returned = false;

        let sleep = if let Some(echo_mir::MirStmt::FunctionCall { call, .. }) = statements.first()
            && self.resolve_std_call_name(&call.name, call.span)? == "time.sleep"
        {
            mir_task_sleep_millis(statements.first().expect("first statement exists"))
        } else {
            None
        };

        let mut body = String::new();
        for (name, _) in &captures {
            let global = format!("echo_capture_{}_{}", function, name);
            let local = format!("%capture_{}_{}", function, name);
            body.push_str(&format!("  {local} = load %EchoValue, ptr @{global}\n"));
            self.locals
                .insert(name.clone(), RuntimeValue::EchoValue(local));
        }
        if let Some(millis) = sleep {
            let continuation =
                self.render_mir_defer_continuation_function(&function, &statements[1..])?;
            body.push_str(&format!(
                "  %runtime_call_{} = call %EchoValue @{}(i64 {millis}, ptr @{continuation})\n",
                self.next_call_id,
                CoreRuntimeSymbol::TaskSleepCurrent.symbol()
            ));
            self.next_call_id += 1;
            body.push_str(&format!(
                "  ret %EchoValue %runtime_call_{}\n",
                self.next_call_id - 1
            ));
            self.returned = true;
        } else {
            for statement in statements {
                if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                    self.aliases = saved_aliases;
                    self.locals = saved_locals;
                    self.returned = saved_returned;
                    return Err(diagnostic);
                }
            }
        }

        let returned = self.returned;
        self.aliases = saved_aliases;
        self.locals = saved_locals;
        self.returned = saved_returned;

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{function}() {{\nentry:\n{}{}\n}}\n",
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(function)
    }

    fn render_mir_defer_continuation_function(
        &mut self,
        parent: &str,
        statements: &[echo_mir::MirStmt],
    ) -> Result<String, Diagnostic> {
        let function = format!("{parent}_cont");
        let saved_returned = self.returned;
        self.returned = false;

        let mut body = String::new();
        for statement in statements {
            if let Err(diagnostic) = self.render_mir_stmt(&mut body, statement) {
                self.returned = saved_returned;
                return Err(diagnostic);
            }
        }

        let returned = self.returned;
        self.returned = saved_returned;

        self.functions_ir.push_str(&format!(
            "define %EchoValue @{function}() {{\nentry:\n{}{}\n}}\n",
            body,
            if returned {
                "".to_string()
            } else {
                "  ret %EchoValue { i32 0, i64 0 }".to_string()
            }
        ));

        Ok(function)
    }

    fn render_mir_run_expr(
        &mut self,
        body: &mut String,
        source: &echo_ast::Expr,
        expr: &echo_mir::MirRunExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirRunExpr::Block { body: block } => {
                let function = self.render_mir_defer_function(body, block)?;
                let defer_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{defer_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::TaskDefer.symbol()
                ));

                let run_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{run_id} = call %EchoValue @{}(%EchoValue %runtime_call_{defer_id})\n",
                    CoreRuntimeSymbol::TaskRun.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{run_id}")))
            }
            echo_mir::MirRunExpr::Task { expr } => {
                self.render_mir_task_unary_expr(body, expr, CoreRuntimeSymbol::TaskRun)
            }
            echo_mir::MirRunExpr::Group { entries } => {
                self.render_mir_run_group_expr(body, source, entries)
            }
        }
    }

    fn render_mir_run_group_expr(
        &mut self,
        body: &mut String,
        _source: &echo_ast::Expr,
        entries: &[Vec<echo_mir::MirStmt>],
    ) -> Result<RuntimeValue, Diagnostic> {
        let group_id = self.next_call_id;
        self.next_call_id += 1;
        let mut group = format!("%runtime_call_{group_id}");
        body.push_str(&format!(
            "  {group} = call %EchoValue @{}()\n",
            CoreRuntimeSymbol::TaskGroupNew.symbol()
        ));

        for entry in entries {
            let function = self.render_mir_defer_function(body, entry)?;
            let defer_id = self.next_call_id;
            self.next_call_id += 1;
            let task = format!("%runtime_call_{defer_id}");
            body.push_str(&format!(
                "  {task} = call %EchoValue @{}(ptr @{function})\n",
                CoreRuntimeSymbol::TaskDefer.symbol()
            ));

            let add_id = self.next_call_id;
            self.next_call_id += 1;
            let added = format!("%runtime_call_{add_id}");
            body.push_str(&format!(
                "  {added} = call %EchoValue @{}(%EchoValue {group}, %EchoValue {task})\n",
                CoreRuntimeSymbol::TaskGroupAdd.symbol()
            ));
            group = added;
        }

        let run_id = self.next_call_id;
        self.next_call_id += 1;
        body.push_str(&format!(
            "  %runtime_call_{run_id} = call %EchoValue @{}(%EchoValue {group})\n",
            CoreRuntimeSymbol::TaskGroupRunAndJoin.symbol()
        ));
        Ok(RuntimeValue::EchoValue(format!("%runtime_call_{run_id}")))
    }

    fn render_mir_fork_expr(
        &mut self,
        body: &mut String,
        _source: &echo_ast::Expr,
        expr: &echo_mir::MirForkExpr,
    ) -> Result<RuntimeValue, Diagnostic> {
        match expr {
            echo_mir::MirForkExpr::Block { body: block } => {
                let function = self.render_mir_defer_function(body, block)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                body.push_str(&format!(
                    "  %runtime_call_{call_id} = call %EchoValue @{}(ptr @{function})\n",
                    CoreRuntimeSymbol::ThreadFork.symbol()
                ));
                Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
            }
            echo_mir::MirForkExpr::Task { expr } => {
                self.render_mir_task_unary_expr(body, expr, CoreRuntimeSymbol::ThreadForkTask)
            }
        }
    }

    fn render_mir_task_unary_expr(
        &mut self,
        body: &mut String,
        expr: &echo_mir::MirExpr,
        symbol: CoreRuntimeSymbol,
    ) -> Result<RuntimeValue, Diagnostic> {
        let task = self.render_mir_expr_as_echo_value(body, expr)?;
        let call_id = self.next_call_id;
        self.next_call_id += 1;

        body.push_str(&format!(
            "  %runtime_call_{call_id} = call %EchoValue @{}({task})\n",
            symbol.symbol()
        ));

        Ok(RuntimeValue::EchoValue(format!("%runtime_call_{call_id}")))
    }

    fn render_mir_function_call_expr(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<RuntimeValue, Diagnostic> {
        let name = self.resolve_std_call_name(&call.name, call.span)?;

        if name == "time.sleep" {
            self.mir_time_sleep_call(body, &call.args, call.span)?;
            return Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()));
        }

        if let Some(intrinsic) = std_intrinsic(&name) {
            return self.mir_std_intrinsic_call(body, intrinsic, &call.args, call.span);
        }

        let Some(builtin) = php_builtin(&name) else {
            return self.render_mir_userland_function_call_expr(body, call);
        };

        match builtin.codegen {
            BuiltinCodegen::ObStart => {
                let (symbol, arg) = match call.args.as_slice() {
                    [] => (builtin.symbol, None),
                    [echo_mir::MirExpr::Null { .. }] => (
                        builtin
                            .helper_symbol
                            .expect("ob_start value helper must be declared"),
                        Some("%EchoValue { i32 0, i64 0 }".to_string()),
                    ),
                    [echo_mir::MirExpr::String { value, .. }] => {
                        let helper = builtin
                            .helper_symbol
                            .expect("ob_start value helper must be declared");
                        let global = self.string_global(value);
                        let value_id = self.next_call_id;
                        self.next_call_id += 1;
                        let value_name = format!("%runtime_call_{value_id}");

                        body.push_str(&format!(
                            "  {value_name} = call %EchoValue @{}(ptr @{global}, i64 {})\n",
                            CoreRuntimeSymbol::ValueString.symbol(),
                            value.len()
                        ));

                        (helper, Some(format!("%EchoValue {value_name}")))
                    }
                    [arg] => {
                        return Err(Diagnostic::new(
                            "unsupported ob_start callback argument in LLVM codegen",
                            arg.syntax().span(),
                        ));
                    }
                    _ => {
                        return Err(Diagnostic::new(
                            "unsupported ob_start argument count in LLVM codegen",
                            call.span,
                        ));
                    }
                };

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let bool_name = format!("%runtime_bool_{call_id}");
                let payload_name = format!("%runtime_bool_payload_{call_id}");
                let value_name = format!("%runtime_call_{call_id}");

                match arg {
                    Some(arg) => {
                        body.push_str(&format!("  {bool_name} = call i1 @{symbol}({arg})\n"))
                    }
                    None => body.push_str(&format!("  {bool_name} = call i1 @{symbol}()\n")),
                }
                body.push_str(&format!("  {payload_name} = zext i1 {bool_name} to i64\n"));
                body.push_str(&format!(
                    "  {value_name} = insertvalue %EchoValue {{ i32 1, i64 0 }}, i64 {payload_name}, 1\n"
                ));

                Ok(RuntimeValue::EchoValue(value_name))
            }
            BuiltinCodegen::VoidStatement => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                body.push_str(&format!("  call void @{}()\n", builtin.symbol));
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            BuiltinCodegen::VoidUnaryStatement => {
                let [arg] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let arg = self.render_mir_expr_as_echo_value(body, arg)?;
                body.push_str(&format!("  call void @{}({arg})\n", builtin.symbol));
                Ok(RuntimeValue::EchoValue("{ i32 0, i64 0 }".to_string()))
            }
            BuiltinCodegen::BoolStatement => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let bool_name = format!("%runtime_bool_{call_id}");
                let payload_name = format!("%runtime_bool_payload_{call_id}");
                let value_name = format!("%runtime_call_{call_id}");

                body.push_str(&format!("  {bool_name} = call i1 @{}()\n", builtin.symbol));
                body.push_str(&format!("  {payload_name} = zext i1 {bool_name} to i64\n"));
                body.push_str(&format!(
                    "  {value_name} = insertvalue %EchoValue {{ i32 1, i64 0 }}, i64 {payload_name}, 1\n"
                ));

                Ok(RuntimeValue::EchoValue(value_name))
            }
            BuiltinCodegen::ValueExpression => {
                if !call.args.is_empty() {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported arguments for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}()\n",
                    builtin.symbol
                ));
                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueUnaryExpression => {
                let [arg] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let arg = self.render_mir_expr_as_echo_value(body, arg)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({arg})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Basename => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let path = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let suffix = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString(String::new()),
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({path}, {suffix})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Dirname => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let path = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let levels = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({path}, {levels})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueBinaryExpression => {
                let [left, right] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let left = self.render_mir_expr_as_echo_value(body, left)?;
                let right = self.render_mir_expr_as_echo_value(body, right)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({left}, {right})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ArrayKeys => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let array = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let filter_value = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 6, i64 0 }".to_string(),
                };
                let strict = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({array}, {filter_value}, {strict})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Log => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let value = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let base = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => format!(
                        "%EchoValue {{ i32 11, i64 {} }}",
                        std::f64::consts::E.to_bits() as i64
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({value}, {base})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ChunkSplit => {
                if !(1..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 76 }".to_string(),
                };
                let separator = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString("\r\n".to_string()),
                    ),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length}, {separator})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::StrPad => {
                if !(2..=4).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let pad_string = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => self.runtime_value_as_echo_value(
                        body,
                        RuntimeValue::StaticString(" ".to_string()),
                    ),
                };
                let pad_type = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length}, {pad_string}, {pad_type})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::StrSplit => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let string = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let length = match call.args.get(1) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 1 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({string}, {length})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::ValueTernaryExpression => {
                let [first, second, third] = call.args.as_slice() else {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                };

                let first = self.render_mir_expr_as_echo_value(body, first)?;
                let second = self.render_mir_expr_as_echo_value(body, second)?;
                let third = self.render_mir_expr_as_echo_value(body, third)?;
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({first}, {second}, {third})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Explode => {
                if !(2..=3).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let separator = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let string = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let limit = match call.args.get(2) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 2, i64 9223372036854775807 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({separator}, {string}, {limit})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::Implode => {
                if !(1..=2).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let (separator, array) = match call.args.as_slice() {
                    [array] => (
                        self.runtime_value_as_echo_value(
                            body,
                            RuntimeValue::StaticString(String::new()),
                        ),
                        self.render_mir_expr_as_echo_value(body, array)?,
                    ),
                    [separator, array] => (
                        self.render_mir_expr_as_echo_value(body, separator)?,
                        self.render_mir_expr_as_echo_value(body, array)?,
                    ),
                    _ => unreachable!("argument count checked above"),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({separator}, {array})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
            BuiltinCodegen::SubstrCompare => {
                if !(3..=5).contains(&call.args.len()) {
                    return Err(Diagnostic::new(
                        format!(
                            "unsupported argument count for builtin `{}` in LLVM codegen",
                            call.name
                        ),
                        call.span,
                    ));
                }

                let haystack = self.render_mir_expr_as_echo_value(body, &call.args[0])?;
                let needle = self.render_mir_expr_as_echo_value(body, &call.args[1])?;
                let offset = self.render_mir_expr_as_echo_value(body, &call.args[2])?;
                let length = match call.args.get(3) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 0, i64 0 }".to_string(),
                };
                let case_insensitive = match call.args.get(4) {
                    Some(expr) => self.render_mir_expr_as_echo_value(body, expr)?,
                    None => "%EchoValue { i32 1, i64 0 }".to_string(),
                };
                let call_id = self.next_call_id;
                self.next_call_id += 1;
                let name = format!("%runtime_call_{call_id}");

                body.push_str(&format!(
                    "  {name} = call %EchoValue @{}({haystack}, {needle}, {offset}, {length}, {case_insensitive})\n",
                    builtin.symbol
                ));

                Ok(RuntimeValue::EchoValue(name))
            }
        }
    }

    fn render_mir_userland_function_call_expr(
        &mut self,
        body: &mut String,
        call: &echo_mir::MirFunctionCall,
    ) -> Result<RuntimeValue, Diagnostic> {
        let Some(function) = self.functions.get(&call.name).cloned() else {
            return Err(Diagnostic::new(
                "unsupported expression in LLVM codegen",
                call.span,
            ));
        };

        if call.args.len() != function.params.len() {
            return Err(Diagnostic::new(
                format!(
                    "unsupported argument count for userland function `{}` in LLVM codegen",
                    call.name
                ),
                call.span,
            ));
        }

        let mut args = Vec::new();
        for arg in &call.args {
            args.push(self.render_mir_expr_as_echo_value(body, arg)?);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;
        let name = format!("%runtime_call_{call_id}");

        body.push_str(&format!(
            "  {name} = call %EchoValue @{}({})\n",
            userland_function_symbol(&call.name),
            args.join(", ")
        ));

        Ok(RuntimeValue::EchoValue(name))
    }

    fn resolve_alias(&self, name: &str) -> String {
        let mut current = name;

        while let Some(next) = self.aliases.get(current) {
            current = next;
        }

        current.to_string()
    }

    fn register_std_import(&mut self, statement: &echo_ast::ImportStmt) -> Result<(), Diagnostic> {
        if statement.source != ImportSource::Std {
            return Ok(());
        }

        let Some(module) = statement.name.parts.first() else {
            return Err(Diagnostic::new(
                "empty std import in LLVM codegen",
                statement.span,
            ));
        };

        if !is_known_std_module(module) {
            return Err(Diagnostic::new(
                format!("unknown std import `{}`", statement.name.as_string()),
                statement.span,
            ));
        }

        if statement.name.parts.len() == 1 {
            let local = statement.alias.as_deref().unwrap_or(module).to_string();
            self.std_imports.insert(local, module.clone());
        }

        Ok(())
    }

    fn resolve_std_call_name(&self, name: &str, span: Span) -> Result<String, Diagnostic> {
        let Some((module, rest)) = name.split_once('.') else {
            return Ok(name.to_string());
        };

        if let Some(imported) = self.std_imports.get(module) {
            return Ok(format!("{imported}.{rest}"));
        }

        if is_known_std_module(module) {
            return Err(Diagnostic::new(
                format!("std module `{module}` must be imported before use"),
                span,
            ));
        }

        Ok(name.to_string())
    }

    fn string_global(&mut self, value: &str) -> String {
        let name = format!("echo_str_{}", self.next_string_id);
        self.next_string_id += 1;

        self.globals.push_str(&format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{}\", align 1\n",
            value.len(),
            llvm_string_literal(value)
        ));

        name
    }
}

fn runtime_declarations() -> String {
    let mut seen = HashSet::new();

    CoreRuntimeSymbol::ALL
        .iter()
        .map(|function| function.llvm_decl())
        .chain(
            PHP_RUNTIME_HELPERS
                .iter()
                .map(|(symbol, signature)| signature.llvm_decl(symbol)),
        )
        .chain(PHP_BUILTINS.iter().map(|builtin| builtin.llvm_decl()))
        .chain(STD_INTRINSICS.iter().map(|intrinsic| intrinsic.llvm_decl()))
        .filter(|declaration| seen.insert(declaration.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn function_params_signature(params: &[TypedParam]) -> String {
    params
        .iter()
        .map(|param| match &param.ty {
            Some(ty) => format!("{ty} ${}", param.name),
            None => format!("${}", param.name),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_known_std_module(name: &str) -> bool {
    let module_name = format!("std.{name}");
    echo_std::modules()
        .iter()
        .any(|module| module.name == module_name)
}

fn userland_function_symbol(name: &str) -> String {
    format!("echo_user_{name}")
}

fn mir_task_sleep_millis(statement: &echo_mir::MirStmt) -> Option<i64> {
    let echo_mir::MirStmt::FunctionCall { call, .. } = statement else {
        return None;
    };
    if call.name != "time.sleep" {
        return None;
    }
    let [echo_mir::MirExpr::Number { value, .. }] = call.args.as_slice() else {
        return None;
    };

    value.parse().ok()
}

fn llvm_string_literal(value: &str) -> String {
    let mut output = String::new();

    for byte in value.bytes() {
        match byte {
            b'\\' => output.push_str(r#"\5C"#),
            b'"' => output.push_str(r#"\22"#),
            0x20..=0x7e => output.push(byte as char),
            _ => output.push_str(&format!(r#"\{byte:02X}"#)),
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_ast::{
        ArrayElement, ArrayExpr, AssignStmt, DeferExpr, EchoStmt, Expr, FunctionCallExpr,
        FunctionCallStmt, FunctionDeclStmt, ImportStmt, NullLiteral, NumberLiteral, QualifiedName,
        ReturnStmt, StringLiteral, TypedParam,
    };

    fn program(statements: Vec<Stmt>) -> Program {
        Program {
            open_tag: None,
            statements,
            source_dir: None,
            span: Span::new(0, 0),
        }
    }

    fn param(name: &str) -> TypedParam {
        TypedParam {
            name: name.to_string(),
            ty: None,
        }
    }

    fn std_import(module: &str) -> Stmt {
        Stmt::Import(ImportStmt {
            source: ImportSource::Std,
            name: QualifiedName::new(vec![module.to_string()]),
            alias: None,
            span: Span::new(0, 0),
        })
    }

    fn std_import_alias(module: &str, alias: &str) -> Stmt {
        Stmt::Import(ImportStmt {
            source: ImportSource::Std,
            name: QualifiedName::new(vec![module.to_string()]),
            alias: Some(alias.to_string()),
            span: Span::new(0, 0),
        })
    }

    #[test]
    fn ast_hir_and_mir_entrypoints_emit_same_ir() {
        let program = program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::String(StringLiteral {
                value: "hello".to_string(),
                span: Span::new(5, 12),
            })],
            span: Span::new(0, 13),
        })]);
        let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
        let mir = echo_mir::lower_program(&hir).expect("HIR should lower to MIR");

        let ast_ir = compile_to_ir(&program).expect("AST entrypoint should compile");
        let hir_ir = compile_hir_to_ir(&hir).expect("HIR entrypoint should compile");
        let mir_ir = compile_mir_to_ir(&mir).expect("MIR entrypoint should compile");

        assert_eq!(ast_ir, hir_ir);
        assert_eq!(hir_ir, mir_ir);
        assert!(mir_ir.contains("call void @echo_write("));
    }

    #[test]
    fn ast_to_hir_to_mir_to_llvm_ir_executes_with_jit() {
        let program = program(vec![]);
        let hir = echo_hir::lower_program(&program).expect("program should lower to HIR");
        let mir = echo_mir::lower_program(&hir).expect("HIR should lower to MIR");

        let status = run_mir_jit(&mir).expect("MIR should execute through LLVM JIT");

        assert_eq!(status, 0);
    }

    #[test]
    fn validates_known_std_import() {
        compile_to_ir(&program(vec![Stmt::Import(ImportStmt {
            source: ImportSource::Std,
            name: QualifiedName::new(vec!["net".to_string()]),
            alias: None,
            span: Span::new(0, 16),
        })]))
        .expect("known std import should compile");
    }

    #[test]
    fn rejects_unknown_std_import() {
        let diagnostics = compile_to_ir(&program(vec![Stmt::Import(ImportStmt {
            source: ImportSource::Std,
            name: QualifiedName::new(vec!["potato".to_string()]),
            alias: None,
            span: Span::new(0, 19),
        })]))
        .expect_err("unknown std import should fail");

        assert_eq!(diagnostics[0].message, "unknown std import `potato`");
        assert_eq!(diagnostics[0].span, Span::new(0, 19));
    }

    #[test]
    fn rejects_unimported_std_module_call() {
        let diagnostics = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
            name: "net.listen".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "127.0.0.1:39183".to_string(),
                span: Span::new(11, 30),
            })],
            span: Span::new(0, 31),
        })]))
        .expect_err("unimported std module should fail");

        assert_eq!(
            diagnostics[0].message,
            "std module `net` must be imported before use"
        );
    }

    #[test]
    fn aliases_std_module_calls() {
        let ir = compile_to_ir(&program(vec![
            std_import_alias("net", "socket"),
            Stmt::FunctionCall(FunctionCallStmt {
                name: "socket.close".to_string(),
                args: vec![Expr::Null(NullLiteral {
                    span: Span::new(13, 17),
                })],
                span: Span::new(0, 18),
            }),
        ]))
        .expect("aliased std call compiles");

        assert!(ir.contains("call %EchoValue @echo_std_net_close"), "{ir}");
    }

    #[test]
    fn ob_start_null_uses_named_echo_value_abi() {
        let ir = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
            name: "ob_start".to_string(),
            args: vec![Expr::Null(NullLiteral {
                span: Span::new(0, 4),
            })],
            span: Span::new(0, 15),
        })]))
        .expect("IR");

        assert!(ir.contains("%EchoValue = type { i32, i64 }"));
        assert!(ir.contains("declare i1 @echo_php_ob_start_value(%EchoValue)"));
        assert!(
            ir.contains("call i1 @echo_php_ob_start_value(%EchoValue { i32 0, i64 0 })"),
            "{ir}"
        );
    }

    #[test]
    fn ob_start_string_constructs_echo_value_callback() {
        let ir = compile_to_ir(&program(vec![Stmt::FunctionCall(FunctionCallStmt {
            name: "ob_start".to_string(),
            args: vec![Expr::String(StringLiteral {
                value: "filter".to_string(),
                span: Span::new(9, 17),
            })],
            span: Span::new(0, 19),
        })]))
        .expect("IR");

        assert!(ir.contains("declare %EchoValue @echo_value_string(ptr, i64)"));
        assert!(ir.contains("declare void @echo_write_value(%EchoValue)"));
        assert!(ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_"));
        assert!(ir.contains("call i1 @echo_php_ob_start_value(%EchoValue %runtime_call_"));
    }

    #[test]
    fn userland_call_emits_function_definition_and_call() {
        let ir = compile_to_ir(&program(vec![
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "say_after".to_string(),
                params: vec![],
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Echo(EchoStmt {
                    exprs: vec![Expr::String(StringLiteral {
                        value: "after\n".to_string(),
                        span: Span::new(0, 8),
                    })],
                    span: Span::new(0, 15),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::FunctionCall(FunctionCallStmt {
                name: "say_after".to_string(),
                args: vec![],
                span: Span::new(41, 53),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("define %EchoValue @echo_user_say_after()"),
            "{ir}"
        );
        assert!(
            ir.contains("call void @echo_write(ptr @echo_str_0, i64 6)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_user_say_after()"),
            "{ir}"
        );
    }

    #[test]
    fn userland_function_declaration_registers_reflection_metadata() {
        let ir = compile_to_ir(&program(vec![
            std_import("reflect"),
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "greet".to_string(),
                params: vec![TypedParam {
                    name: "name".to_string(),
                    ty: Some("string".to_string()),
                }],
                return_type: Some("string".to_string()),
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Return(ReturnStmt {
                    value: Expr::String(StringLiteral {
                        value: "hello\n".to_string(),
                        span: Span::new(0, 8),
                    }),
                    span: Span::new(0, 15),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "reflect.params".to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "greet".to_string(),
                        span: Span::new(50, 57),
                    })],
                    span: Span::new(35, 58),
                })],
                span: Span::new(35, 59),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains(
                "declare void @echo_reflection_register_function(ptr, i64, ptr, i64, ptr, i64, i32)"
            ),
            "{ir}"
        );
        assert!(ir.contains("c\"greet\""), "{ir}");
        assert!(ir.contains("c\"string $name\""), "{ir}");
        assert!(
            ir.contains("call void @echo_reflection_register_function("),
            "{ir}"
        );
    }

    #[test]
    fn userland_call_passes_string_argument_as_echo_value() {
        let ir = compile_to_ir(&program(vec![
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "say".to_string(),
                params: vec![param("message")],
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Echo(EchoStmt {
                    exprs: vec![Expr::Variable(echo_ast::VariableExpr {
                        name: "message".to_string(),
                        span: Span::new(0, 8),
                    })],
                    span: Span::new(0, 15),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::FunctionCall(FunctionCallStmt {
                name: "say".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "hello\n".to_string(),
                    span: Span::new(45, 53),
                })],
                span: Span::new(41, 55),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("define %EchoValue @echo_user_say(%EchoValue %arg_message)"),
            "{ir}"
        );
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %arg_message)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_user_say(%EchoValue %runtime_call_"),
            "{ir}"
        );
    }

    #[test]
    fn userland_return_value_can_be_echoed() {
        let ir = compile_to_ir(&program(vec![
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "greeting".to_string(),
                params: vec![],
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Return(ReturnStmt {
                    value: Expr::String(StringLiteral {
                        value: "hello\n".to_string(),
                        span: Span::new(0, 8),
                    }),
                    span: Span::new(0, 16),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "greeting".to_string(),
                    args: vec![],
                    span: Span::new(45, 55),
                })],
                span: Span::new(41, 56),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("define %EchoValue @echo_user_greeting()"),
            "{ir}"
        );
        assert!(ir.contains("ret %EchoValue %runtime_call_0"), "{ir}");
        assert!(ir.contains("call %EchoValue @echo_user_greeting()"), "{ir}");
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
            "{ir}"
        );
    }

    #[test]
    fn dynamic_concat_uses_echo_value_concat() {
        let ir = compile_to_ir(&program(vec![
            Stmt::FunctionDecl(FunctionDeclStmt {
                name: "greet".to_string(),
                params: vec![param("name")],
                return_type: None,
                is_intrinsic: false,
                is_generator: false,
                body: vec![Stmt::Echo(EchoStmt {
                    exprs: vec![Expr::Binary(Box::new(echo_ast::BinaryExpr {
                        left: Expr::Binary(Box::new(echo_ast::BinaryExpr {
                            left: Expr::String(StringLiteral {
                                value: "Hello, ".to_string(),
                                span: Span::new(0, 9),
                            }),
                            op: BinaryOp::Concat,
                            right: Expr::Variable(echo_ast::VariableExpr {
                                name: "name".to_string(),
                                span: Span::new(12, 17),
                            }),
                            span: Span::new(0, 17),
                        })),
                        op: BinaryOp::Concat,
                        right: Expr::String(StringLiteral {
                            value: "!\n".to_string(),
                            span: Span::new(20, 24),
                        }),
                        span: Span::new(0, 24),
                    }))],
                    span: Span::new(0, 30),
                })],
                span: Span::new(0, 40),
            }),
            Stmt::FunctionCall(FunctionCallStmt {
                name: "greet".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "Echo".to_string(),
                    span: Span::new(45, 51),
                })],
                span: Span::new(41, 53),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_value_concat(%EchoValue, %EchoValue)"),
            "{ir}"
        );
        assert!(ir.contains("call %EchoValue @echo_value_concat"), "{ir}");
    }

    #[test]
    fn integer_subtraction_lowers_to_echo_value_sub() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::Binary(Box::new(echo_ast::BinaryExpr {
                left: Expr::Number(NumberLiteral {
                    value: "3".to_string(),
                    span: Span::new(5, 6),
                }),
                op: BinaryOp::Sub,
                right: Expr::Number(NumberLiteral {
                    value: "5".to_string(),
                    span: Span::new(7, 8),
                }),
                span: Span::new(5, 8),
            }))],
            span: Span::new(0, 9),
        })]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_value_sub(%EchoValue, %EchoValue)"),
            "{ir}"
        );
        assert!(ir.contains("call %EchoValue @echo_value_sub"), "{ir}");
    }

    #[test]
    fn strlen_lowers_to_php_builtin_with_echo_value_argument() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "strlen".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "hello".to_string(),
                    span: Span::new(7, 14),
                })],
                span: Span::new(0, 15),
            })],
            span: Span::new(0, 16),
        })]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_php_strlen(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_php_strlen(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
            "{ir}"
        );
    }

    #[test]
    fn pi_lowers_to_php_builtin_with_no_arguments() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "pi".to_string(),
                args: vec![],
                span: Span::new(5, 9),
            })],
            span: Span::new(0, 10),
        })]))
        .expect("IR");

        assert!(ir.contains("declare %EchoValue @echo_php_pi()"), "{ir}");
        assert!(ir.contains("call %EchoValue @echo_php_pi()"), "{ir}");
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
    }

    #[test]
    fn logarithm_builtins_lower_optional_base_to_e() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "log".to_string(),
                args: vec![Expr::Number(NumberLiteral {
                    value: "8".to_string(),
                    span: Span::new(4, 5),
                })],
                span: Span::new(0, 6),
            })],
            span: Span::new(0, 7),
        })]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_php_log(%EchoValue, %EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_php_log(%EchoValue { i32 2, i64 8 }, %EchoValue { i32 11, i64 4613303445314885481 })"),
            "{ir}"
        );
    }

    #[test]
    fn getcwd_lowers_to_php_builtin_with_no_arguments() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "getcwd".to_string(),
                args: vec![],
                span: Span::new(5, 13),
            })],
            span: Span::new(0, 14),
        })]))
        .expect("IR");

        assert!(ir.contains("declare %EchoValue @echo_php_getcwd()"), "{ir}");
        assert!(ir.contains("call %EchoValue @echo_php_getcwd()"), "{ir}");
        assert!(
            ir.contains("call void @echo_write_value(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
    }

    #[test]
    fn string_case_builtins_lower_to_php_builtin_with_echo_value_argument() {
        for (php_name, symbol) in [
            ("strval", "echo_php_strval"),
            ("boolval", "echo_php_boolval"),
            ("intval", "echo_php_intval"),
            ("floatval", "echo_php_floatval"),
            ("doubleval", "echo_php_floatval"),
            ("strtoupper", "echo_php_strtoupper"),
            ("strtolower", "echo_php_strtolower"),
            ("ucwords", "echo_php_ucwords"),
            ("strrev", "echo_php_strrev"),
            ("ucfirst", "echo_php_ucfirst"),
            ("lcfirst", "echo_php_lcfirst"),
            ("ord", "echo_php_ord"),
            ("str_rot13", "echo_php_str_rot13"),
            ("chr", "echo_php_chr"),
            ("decbin", "echo_php_decbin"),
            ("dechex", "echo_php_dechex"),
            ("decoct", "echo_php_decoct"),
            ("bindec", "echo_php_bindec"),
            ("hexdec", "echo_php_hexdec"),
            ("octdec", "echo_php_octdec"),
            ("deg2rad", "echo_php_deg2rad"),
            ("rad2deg", "echo_php_rad2deg"),
            ("sin", "echo_php_sin"),
            ("cos", "echo_php_cos"),
            ("tan", "echo_php_tan"),
            ("asin", "echo_php_asin"),
            ("acos", "echo_php_acos"),
            ("atan", "echo_php_atan"),
            ("sinh", "echo_php_sinh"),
            ("cosh", "echo_php_cosh"),
            ("tanh", "echo_php_tanh"),
            ("asinh", "echo_php_asinh"),
            ("acosh", "echo_php_acosh"),
            ("atanh", "echo_php_atanh"),
            ("ceil", "echo_php_ceil"),
            ("floor", "echo_php_floor"),
            ("sqrt", "echo_php_sqrt"),
            ("bin2hex", "echo_php_bin2hex"),
            ("base64_encode", "echo_php_base64_encode"),
            ("base64_decode", "echo_php_base64_decode"),
            ("rawurlencode", "echo_php_rawurlencode"),
            ("rawurldecode", "echo_php_rawurldecode"),
            ("urlencode", "echo_php_urlencode"),
            ("urldecode", "echo_php_urldecode"),
            ("hex2bin", "echo_php_hex2bin"),
            ("escapeshellarg", "echo_php_escapeshellarg"),
            ("escapeshellcmd", "echo_php_escapeshellcmd"),
            ("file_exists", "echo_php_file_exists"),
            ("chdir", "echo_php_chdir"),
            ("is_dir", "echo_php_is_dir"),
            ("is_file", "echo_php_is_file"),
            ("is_link", "echo_php_is_link"),
            ("is_readable", "echo_php_is_readable"),
            ("is_writable", "echo_php_is_writable"),
            ("is_writeable", "echo_php_is_writable"),
            ("is_executable", "echo_php_is_executable"),
            ("filesize", "echo_php_filesize"),
            ("realpath", "echo_php_realpath"),
            ("trim", "echo_php_trim"),
            ("ltrim", "echo_php_ltrim"),
            ("rtrim", "echo_php_rtrim"),
            ("addslashes", "echo_php_addslashes"),
            ("stripslashes", "echo_php_stripslashes"),
            ("quotemeta", "echo_php_quotemeta"),
        ] {
            let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: php_name.to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "Echo".to_string(),
                        span: Span::new(11, 17),
                    })],
                    span: Span::new(0, 18),
                })],
                span: Span::new(0, 19),
            })]))
            .expect("IR");

            assert!(
                ir.contains(&format!("declare %EchoValue @{symbol}(%EchoValue)")),
                "{ir}"
            );
            assert!(
                ir.contains(&format!(
                    "call %EchoValue @{symbol}(%EchoValue %runtime_call_0)"
                )),
                "{ir}"
            );
            assert!(
                ir.contains("call void @echo_write_value(%EchoValue %runtime_call_1)"),
                "{ir}"
            );
        }
    }

    #[test]
    fn string_predicate_builtins_lower_to_php_builtin_with_two_echo_value_arguments() {
        for (php_name, symbol) in [
            ("str_contains", "echo_php_str_contains"),
            ("str_starts_with", "echo_php_str_starts_with"),
            ("str_ends_with", "echo_php_str_ends_with"),
            ("str_repeat", "echo_php_str_repeat"),
            ("substr", "echo_php_substr"),
            ("strpos", "echo_php_strpos"),
            ("stripos", "echo_php_stripos"),
            ("strrpos", "echo_php_strrpos"),
            ("strripos", "echo_php_strripos"),
            ("strstr", "echo_php_strstr"),
            ("strchr", "echo_php_strstr"),
            ("stristr", "echo_php_stristr"),
            ("strrchr", "echo_php_strrchr"),
            ("strpbrk", "echo_php_strpbrk"),
            ("strspn", "echo_php_strspn"),
            ("strcspn", "echo_php_strcspn"),
            ("substr_count", "echo_php_substr_count"),
            ("strcmp", "echo_php_strcmp"),
            ("strcasecmp", "echo_php_strcasecmp"),
            ("atan2", "echo_php_atan2"),
            ("hypot", "echo_php_hypot"),
            ("fmod", "echo_php_fmod"),
        ] {
            let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: php_name.to_string(),
                    args: vec![
                        Expr::String(StringLiteral {
                            value: "Echo PHP".to_string(),
                            span: Span::new(13, 23),
                        }),
                        Expr::String(StringLiteral {
                            value: "PHP".to_string(),
                            span: Span::new(25, 30),
                        }),
                    ],
                    span: Span::new(0, 31),
                })],
                span: Span::new(0, 32),
            })]))
            .expect("IR");

            assert!(
                ir.contains(&format!(
                    "declare %EchoValue @{symbol}(%EchoValue, %EchoValue)"
                )),
                "{ir}"
            );
            assert!(
                ir.contains(&format!(
                    "call %EchoValue @{symbol}(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1)"
                )),
                "{ir}"
            );
            assert!(
                ir.contains("call void @echo_write_value(%EchoValue %runtime_call_2)"),
                "{ir}"
            );
        }
    }

    #[test]
    fn runtime_declarations_deduplicate_alias_symbols() {
        let declarations = runtime_declarations();

        assert_eq!(declarations.matches("@echo_php_strstr(").count(), 1);
    }

    #[test]
    fn str_pad_lowers_optional_arguments_to_php_defaults() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "str_pad".to_string(),
                args: vec![
                    Expr::String(StringLiteral {
                        value: "ID".to_string(),
                        span: Span::new(8, 12),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "6".to_string(),
                        span: Span::new(14, 15),
                    }),
                ],
                span: Span::new(0, 16),
            })],
            span: Span::new(0, 17),
        })]))
        .expect("IR");

        assert!(ir.contains(
            "declare %EchoValue @echo_php_str_pad(%EchoValue, %EchoValue, %EchoValue, %EchoValue)"
        ));
        assert!(ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_"));
        assert!(ir.contains(", i64 1)"));
        assert!(ir.contains("call %EchoValue @echo_php_str_pad("), "{ir}");
        assert!(ir.contains("%EchoValue { i32 2, i64 1 }"));
    }

    #[test]
    fn string_chunk_builtins_lower_optional_arguments_to_php_defaults() {
        let split_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "str_split".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "Echo".to_string(),
                    span: Span::new(10, 16),
                })],
                span: Span::new(0, 17),
            })],
            span: Span::new(0, 18),
        })]))
        .expect("IR");

        assert!(
            split_ir.contains("declare %EchoValue @echo_php_str_split(%EchoValue, %EchoValue)")
        );
        assert!(split_ir.contains(
            "call %EchoValue @echo_php_str_split(%EchoValue %runtime_call_0, %EchoValue { i32 2, i64 1 })"
        ));

        let chunk_ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "chunk_split".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "Echo".to_string(),
                    span: Span::new(12, 18),
                })],
                span: Span::new(0, 19),
            })],
            span: Span::new(0, 20),
        })]))
        .expect("IR");

        assert!(chunk_ir.contains(
            "declare %EchoValue @echo_php_chunk_split(%EchoValue, %EchoValue, %EchoValue)"
        ));
        assert!(chunk_ir.contains("%EchoValue { i32 2, i64 76 }"));
        assert!(chunk_ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_"));
        assert!(chunk_ir.contains(", i64 2)"));
        assert!(chunk_ir.contains("call %EchoValue @echo_php_chunk_split("));
    }

    #[test]
    fn every_lowered_php_builtin_has_reflected_declaration() {
        for builtin in PHP_BUILTINS {
            assert!(
                echo_reflection::php_builtin(builtin.php_name).is_some(),
                "missing reflected declaration for {}",
                builtin.php_name
            );
        }
    }

    #[test]
    fn string_prefix_compare_builtins_lower_to_php_builtin_with_three_echo_value_arguments() {
        for (php_name, symbol) in [
            ("strncmp", "echo_php_strncmp"),
            ("strncasecmp", "echo_php_strncasecmp"),
        ] {
            let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: php_name.to_string(),
                    args: vec![
                        Expr::String(StringLiteral {
                            value: "Echo PHP".to_string(),
                            span: Span::new(13, 23),
                        }),
                        Expr::String(StringLiteral {
                            value: "PHP".to_string(),
                            span: Span::new(25, 30),
                        }),
                        Expr::Number(NumberLiteral {
                            value: "3".to_string(),
                            span: Span::new(32, 33),
                        }),
                    ],
                    span: Span::new(0, 34),
                })],
                span: Span::new(0, 35),
            })]))
            .expect("IR");

            assert!(
                ir.contains(&format!(
                    "call %EchoValue @{symbol}(%EchoValue %runtime_call_"
                )),
                "IR should call {symbol}: {ir}"
            );
            assert!(ir.contains("declare %EchoValue @"));
            assert!(ir.contains("(%EchoValue, %EchoValue, %EchoValue)"));
        }
    }

    #[test]
    fn explode_lowers_optional_limit_to_php_default() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "explode".to_string(),
                args: vec![
                    Expr::String(StringLiteral {
                        value: ",".to_string(),
                        span: Span::new(8, 11),
                    }),
                    Expr::String(StringLiteral {
                        value: "a,b".to_string(),
                        span: Span::new(13, 18),
                    }),
                ],
                span: Span::new(0, 19),
            })],
            span: Span::new(0, 20),
        })]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_php_explode(%EchoValue, %EchoValue, %EchoValue)")
        );
        assert!(
            ir.contains(
                "call %EchoValue @echo_php_explode(%EchoValue %runtime_call_0, %EchoValue %runtime_call_1, %EchoValue { i32 2, i64 9223372036854775807 })"
            ),
            "{ir}"
        );
    }

    #[test]
    fn implode_lowers_optional_separator_to_empty_string() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "implode".to_string(),
                args: vec![Expr::Array(ArrayExpr {
                    elements: vec![ArrayElement {
                        key: None,
                        value: Expr::String(StringLiteral {
                            value: "a".to_string(),
                            span: Span::new(9, 12),
                        }),
                        span: Span::new(9, 12),
                    }],
                    span: Span::new(8, 13),
                })],
                span: Span::new(0, 14),
            })],
            span: Span::new(0, 15),
        })]))
        .expect("IR");

        assert!(ir.contains("declare %EchoValue @echo_php_implode(%EchoValue, %EchoValue)"));
        assert!(
            ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_")
                && ir.contains(", i64 0)")
        );
        assert!(ir.contains("call %EchoValue @echo_php_implode("), "{ir}");
    }

    #[test]
    fn array_keys_lowers_optional_filter_and_strict_defaults() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "array_keys".to_string(),
                args: vec![Expr::Array(ArrayExpr {
                    elements: vec![ArrayElement {
                        key: Some(Expr::String(StringLiteral {
                            value: "id".to_string(),
                            span: Span::new(12, 16),
                        })),
                        value: Expr::Number(NumberLiteral {
                            value: "10".to_string(),
                            span: Span::new(20, 22),
                        }),
                        span: Span::new(12, 22),
                    }],
                    span: Span::new(11, 23),
                })],
                span: Span::new(0, 24),
            })],
            span: Span::new(0, 25),
        })]))
        .expect("IR");

        assert!(
            ir.contains(
                "declare %EchoValue @echo_php_array_keys(%EchoValue, %EchoValue, %EchoValue)"
            ),
            "{ir}"
        );
        assert!(
            ir.contains("%EchoValue { i32 6, i64 0 }, %EchoValue { i32 1, i64 0 })"),
            "{ir}"
        );
    }

    #[test]
    fn substr_compare_lowers_optional_arguments_to_php_defaults() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "substr_compare".to_string(),
                args: vec![
                    Expr::String(StringLiteral {
                        value: "abcde".to_string(),
                        span: Span::new(16, 23),
                    }),
                    Expr::String(StringLiteral {
                        value: "de".to_string(),
                        span: Span::new(25, 29),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "3".to_string(),
                        span: Span::new(31, 32),
                    }),
                ],
                span: Span::new(0, 33),
            })],
            span: Span::new(0, 34),
        })]))
        .expect("IR");

        assert!(ir.contains("declare %EchoValue @echo_php_substr_compare(%EchoValue, %EchoValue, %EchoValue, %EchoValue, %EchoValue)"));
        assert!(ir.contains("call %EchoValue @echo_php_substr_compare("));
        assert!(ir.contains("%EchoValue { i32 0, i64 0 }, %EchoValue { i32 1, i64 0 }"));
    }

    #[test]
    fn basename_lowers_optional_suffix_to_empty_string() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "basename".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "/etc/passwd".to_string(),
                    span: Span::new(9, 22),
                })],
                span: Span::new(0, 23),
            })],
            span: Span::new(0, 24),
        })]))
        .expect("IR");

        assert!(ir.contains("declare %EchoValue @echo_php_basename(%EchoValue, %EchoValue)"));
        assert!(ir.contains("call %EchoValue @echo_php_basename("));
        assert!(
            ir.contains("call %EchoValue @echo_value_string(ptr @echo_str_")
                && ir.contains(", i64 0)")
        );
    }

    #[test]
    fn base_convert_lowers_to_three_echo_value_arguments() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "base_convert".to_string(),
                args: vec![
                    Expr::String(StringLiteral {
                        value: "a37334".to_string(),
                        span: Span::new(13, 21),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "16".to_string(),
                        span: Span::new(23, 25),
                    }),
                    Expr::Number(NumberLiteral {
                        value: "2".to_string(),
                        span: Span::new(27, 28),
                    }),
                ],
                span: Span::new(0, 29),
            })],
            span: Span::new(0, 30),
        })]))
        .expect("IR");

        assert!(ir.contains(
            "declare %EchoValue @echo_php_base_convert(%EchoValue, %EchoValue, %EchoValue)"
        ));
        assert!(ir.contains("call %EchoValue @echo_php_base_convert("));
    }

    #[test]
    fn dirname_lowers_optional_levels_to_one() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                name: "dirname".to_string(),
                args: vec![Expr::String(StringLiteral {
                    value: "/etc/passwd".to_string(),
                    span: Span::new(8, 21),
                })],
                span: Span::new(0, 22),
            })],
            span: Span::new(0, 23),
        })]))
        .expect("IR");

        assert!(ir.contains("declare %EchoValue @echo_php_dirname(%EchoValue, %EchoValue)"));
        assert!(ir.contains("call %EchoValue @echo_php_dirname("));
        assert!(ir.contains("%EchoValue { i32 2, i64 1 }"));
    }

    #[test]
    fn time_sleep_lowers_to_core_runtime_call() {
        let ir = compile_to_ir(&program(vec![
            std_import("time"),
            Stmt::FunctionCall(FunctionCallStmt {
                name: "time.sleep".to_string(),
                args: vec![Expr::Number(echo_ast::NumberLiteral {
                    value: "50".to_string(),
                    span: Span::new(11, 13),
                })],
                span: Span::new(0, 14),
            }),
        ]))
        .expect("IR");

        assert!(ir.contains("declare void @echo_time_sleep(i64)"), "{ir}");
        assert!(ir.contains("call void @echo_time_sleep(i64 50)"), "{ir}");
    }

    #[test]
    fn net_listen_lowers_to_std_intrinsic_call() {
        let ir = compile_to_ir(&program(vec![
            std_import("net"),
            Stmt::Assign(AssignStmt {
                name: "server".to_string(),
                value: Expr::FunctionCall(FunctionCallExpr {
                    name: "net.listen".to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "127.0.0.1:39183".to_string(),
                        span: Span::new(11, 30),
                    })],
                    span: Span::new(0, 31),
                }),
                span: Span::new(0, 31),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_std_net_listen(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_std_net_listen(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
    }

    #[test]
    fn net_read_lowers_numeric_length_as_int_value() {
        let ir = compile_to_ir(&program(vec![
            std_import("net"),
            Stmt::Assign(AssignStmt {
                name: "connection".to_string(),
                value: Expr::FunctionCall(FunctionCallExpr {
                    name: "net.connect".to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "127.0.0.1:39183".to_string(),
                        span: Span::new(0, 17),
                    })],
                    span: Span::new(0, 18),
                }),
                span: Span::new(0, 18),
            }),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "net.read".to_string(),
                    args: vec![
                        Expr::Variable(echo_ast::VariableExpr {
                            name: "connection".to_string(),
                            span: Span::new(19, 30),
                        }),
                        Expr::Number(echo_ast::NumberLiteral {
                            value: "4".to_string(),
                            span: Span::new(31, 32),
                        }),
                    ],
                    span: Span::new(19, 33),
                })],
                span: Span::new(19, 33),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains(
                "call %EchoValue @echo_std_net_read(%EchoValue %runtime_call_1, %EchoValue { i32 2, i64 4 })"
            ),
            "{ir}"
        );
    }

    #[test]
    fn http_response_text_lowers_to_std_intrinsic_call() {
        let ir = compile_to_ir(&program(vec![
            std_import("http"),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "http.responseText".to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "hello".to_string(),
                        span: Span::new(18, 25),
                    })],
                    span: Span::new(0, 26),
                })],
                span: Span::new(0, 26),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_std_http_response_text(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_std_http_response_text(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
    }

    #[test]
    fn reflect_lowers_to_std_intrinsic_call() {
        let ir = compile_to_ir(&program(vec![
            std_import("reflect"),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "reflect.params".to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "strlen".to_string(),
                        span: Span::new(18, 26),
                    })],
                    span: Span::new(0, 27),
                })],
                span: Span::new(0, 27),
            }),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "reflect.returnType".to_string(),
                    args: vec![Expr::String(StringLiteral {
                        value: "strlen".to_string(),
                        span: Span::new(46, 54),
                    })],
                    span: Span::new(28, 55),
                })],
                span: Span::new(28, 55),
            }),
            Stmt::Echo(EchoStmt {
                exprs: vec![Expr::FunctionCall(FunctionCallExpr {
                    name: "reflect.typeOf".to_string(),
                    args: vec![Expr::Number(NumberLiteral {
                        value: "1".to_string(),
                        span: Span::new(72, 73),
                    })],
                    span: Span::new(56, 74),
                })],
                span: Span::new(56, 74),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_std_reflect_params(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("declare %EchoValue @echo_std_reflect_return_type(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("declare %EchoValue @echo_std_reflect_type_of(%EchoValue)"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_std_reflect_params(%EchoValue %runtime_call_0)"),
            "{ir}"
        );
        assert!(
            ir.contains(
                "call %EchoValue @echo_std_reflect_return_type(%EchoValue %runtime_call_2)"
            ),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_std_reflect_type_of(%EchoValue { i32 2, i64 1 })"),
            "{ir}"
        );
    }

    #[test]
    fn task_sleep_lowers_to_timer_continuation() {
        let ir = compile_to_ir(&program(vec![
            std_import("time"),
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(echo_ast::RunExpr::Block {
                    body: vec![
                        Stmt::FunctionCall(FunctionCallStmt {
                            name: "time.sleep".to_string(),
                            args: vec![Expr::Number(echo_ast::NumberLiteral {
                                value: "50".to_string(),
                                span: Span::new(0, 2),
                            })],
                            span: Span::new(0, 14),
                        }),
                        Stmt::Echo(EchoStmt {
                            exprs: vec![Expr::String(StringLiteral {
                                value: "done\n".to_string(),
                                span: Span::new(15, 22),
                            })],
                            span: Span::new(15, 28),
                        }),
                    ],
                    span: Span::new(0, 28),
                }),
                span: Span::new(0, 28),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_task_sleep_current(i64, ptr)"),
            "{ir}"
        );
        assert!(
            ir.contains("define %EchoValue @echo_defer_0_cont()"),
            "{ir}"
        );
        assert!(
            ir.contains("call %EchoValue @echo_task_sleep_current(i64 50, ptr @echo_defer_0_cont)"),
            "{ir}"
        );
    }

    #[test]
    fn expression_statement_evaluates_and_discards_value() {
        let ir = compile_to_ir(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Defer(DeferExpr {
                    body: vec![],
                    span: Span::new(0, 10),
                }),
                span: Span::new(0, 10),
            }),
            Stmt::Expr(echo_ast::ExprStmt {
                expr: Expr::Join(echo_ast::JoinExpr {
                    handle: Box::new(Expr::Variable(echo_ast::VariableExpr {
                        name: "task".to_string(),
                        span: Span::new(11, 16),
                    })),
                    span: Span::new(11, 16),
                }),
                span: Span::new(11, 16),
            }),
        ]))
        .expect("IR");

        assert!(ir.contains("call %EchoValue @echo_join"), "{ir}");
        assert!(!ir.contains("call void @echo_write_value"), "{ir}");
    }

    #[test]
    fn array_literals_lower_to_array_runtime() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: None,
                    value: Expr::Number(NumberLiteral {
                        value: "1".to_string(),
                        span: Span::new(1, 2),
                    }),
                    span: Span::new(1, 2),
                }],
                span: Span::new(0, 3),
            })],
            span: Span::new(0, 3),
        })]))
        .expect("array literal should lower");

        assert!(ir.contains("declare %EchoValue @echo_value_array_new()"));
        assert!(ir.contains("declare %EchoValue @echo_value_array_append(%EchoValue, %EchoValue)"));
        assert!(ir.contains("call %EchoValue @echo_value_array_new()"));
        assert!(ir.contains("call %EchoValue @echo_value_array_append"));
        assert!(!ir.contains("call %EchoValue @echo_value_list_append"));
    }

    #[test]
    fn keyed_array_literals_lower_to_array_set_runtime() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::Array(ArrayExpr {
                elements: vec![ArrayElement {
                    key: Some(Expr::String(StringLiteral {
                        value: "key".to_string(),
                        span: Span::new(1, 6),
                    })),
                    value: Expr::Number(NumberLiteral {
                        value: "1".to_string(),
                        span: Span::new(10, 11),
                    }),
                    span: Span::new(1, 11),
                }],
                span: Span::new(0, 12),
            })],
            span: Span::new(0, 12),
        })]))
        .expect("keyed array should lower");

        assert!(ir.contains(
            "declare %EchoValue @echo_value_array_set(%EchoValue, %EchoValue, %EchoValue)"
        ));
        assert!(ir.contains("call %EchoValue @echo_value_array_set"));
    }

    #[test]
    fn index_access_lowers_to_index_get_runtime() {
        let ir = compile_to_ir(&program(vec![Stmt::Echo(EchoStmt {
            exprs: vec![Expr::Index(Box::new(echo_ast::IndexExpr {
                collection: Expr::Array(ArrayExpr {
                    elements: vec![ArrayElement {
                        key: None,
                        value: Expr::Number(NumberLiteral {
                            value: "4".to_string(),
                            span: Span::new(1, 2),
                        }),
                        span: Span::new(1, 2),
                    }],
                    span: Span::new(0, 3),
                }),
                index: Expr::Number(NumberLiteral {
                    value: "0".to_string(),
                    span: Span::new(4, 5),
                }),
                span: Span::new(0, 6),
            }))],
            span: Span::new(0, 6),
        })]))
        .expect("index access should lower");

        assert!(ir.contains("declare %EchoValue @echo_value_index_get(%EchoValue, %EchoValue)"));
        assert!(ir.contains("call %EchoValue @echo_value_index_get"));
    }

    #[test]
    fn append_statement_lowers_to_array_append_runtime() {
        let ir = compile_to_ir(&program(vec![
            Stmt::Let(echo_ast::LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 12),
            }),
            Stmt::Append(echo_ast::AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "2".to_string(),
                    span: Span::new(20, 21),
                }),
                span: Span::new(13, 22),
            }),
        ]))
        .expect("array append should lower");

        assert!(ir.contains("declare %EchoValue @echo_value_array_append(%EchoValue, %EchoValue)"));
        assert!(ir.contains("call %EchoValue @echo_value_array_append"));
        assert!(!ir.contains("call %EchoValue @echo_value_list_append"));
    }

    #[test]
    fn defer_lowers_to_runtime_task_handle() {
        let ir = compile_to_ir(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "deferred".to_string(),
                value: Expr::Defer(DeferExpr {
                    body: vec![],
                    span: Span::new(0, 10),
                }),
                span: Span::new(0, 10),
            }),
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(echo_ast::RunExpr::Task {
                    expr: Box::new(Expr::Variable(echo_ast::VariableExpr {
                        name: "deferred".to_string(),
                        span: Span::new(11, 20),
                    })),
                    span: Span::new(11, 20),
                }),
                span: Span::new(11, 20),
            }),
            Stmt::Assign(AssignStmt {
                name: "value".to_string(),
                value: Expr::Join(echo_ast::JoinExpr {
                    handle: Box::new(Expr::Variable(echo_ast::VariableExpr {
                        name: "task".to_string(),
                        span: Span::new(21, 30),
                    })),
                    span: Span::new(21, 30),
                }),
                span: Span::new(21, 30),
            }),
        ]))
        .expect("IR");

        assert!(
            ir.contains("declare %EchoValue @echo_task_defer(ptr)"),
            "{ir}"
        );
        assert!(ir.contains("define %EchoValue @echo_defer_0()"), "{ir}");
        assert!(
            ir.contains("call %EchoValue @echo_task_defer(ptr @echo_defer_0)"),
            "{ir}"
        );
        assert!(
            ir.contains("declare %EchoValue @echo_task_run(%EchoValue)"),
            "{ir}"
        );
        assert!(ir.contains("call %EchoValue @echo_task_run"), "{ir}");
        assert!(
            ir.contains("declare %EchoValue @echo_join(%EchoValue)"),
            "{ir}"
        );
        assert!(ir.contains("call %EchoValue @echo_join"), "{ir}");
        assert!(ir.contains("ret i32 0"), "{ir}");
    }
}
