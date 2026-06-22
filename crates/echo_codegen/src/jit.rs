use echo_diagnostics::Diagnostic;
use echo_source::Span;
use inkwell::OptimizationLevel as LlvmOptimizationLevel;
use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use std::collections::HashMap;
use std::sync::OnceLock;

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

pub(crate) fn run_ir_jit(ir: &str) -> Result<i32, Vec<Diagnostic>> {
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
    let addresses = jit_runtime_symbol_address_map();
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

fn jit_runtime_symbol_address_map() -> &'static HashMap<&'static str, usize> {
    static ADDRESSES: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();

    ADDRESSES.get_or_init(|| crate::jit_runtime_symbol_addresses().into_iter().collect())
}

fn jit_diagnostic(message: String) -> Vec<Diagnostic> {
    vec![Diagnostic::new(message, Span::new(0, 0))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_symbol_address_map_is_cached() {
        let first = jit_runtime_symbol_address_map();
        let second = jit_runtime_symbol_address_map();

        assert!(std::ptr::eq(first, second));
        assert!(first.contains_key("echo_write"));
        assert!(first.contains_key("echo_shutdown"));
    }
}
