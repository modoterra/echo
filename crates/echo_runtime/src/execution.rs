use std::cell::RefCell;
use std::io::{self, Write};

use crate::{
    ECHO_VALUE_PROCESS, ECHO_VALUE_TASK, ECHO_VALUE_THREAD, EchoValue, echo_process_join,
    echo_task_join, echo_thread_join,
};

#[derive(Debug, Default)]
struct RuntimeExecution {
    stdout: Option<Vec<u8>>,
    repl_inspect: bool,
}

thread_local! {
    static EXECUTION: RefCell<RuntimeExecution> = RefCell::new(RuntimeExecution::default());
}

pub(crate) fn reset() {
    EXECUTION.with(|execution| {
        *execution.borrow_mut() = RuntimeExecution::default();
    });
}

pub(crate) fn begin_capture(repl_inspect: bool) {
    EXECUTION.with(|execution| {
        *execution.borrow_mut() = RuntimeExecution {
            stdout: Some(Vec::new()),
            repl_inspect,
        };
    });
}

pub(crate) fn finish_capture() -> Vec<u8> {
    EXECUTION.with(|execution| {
        let mut execution = execution.borrow_mut();
        execution.repl_inspect = false;
        execution.stdout.take().unwrap_or_default()
    })
}

pub(crate) fn repl_inspect_enabled() -> bool {
    EXECUTION.with(|execution| execution.borrow().repl_inspect)
        || std::env::var_os("ECHO_REPL_INSPECT").is_some()
}

pub(crate) fn write_stdout(bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }

    if EXECUTION.with(|execution| {
        let mut execution = execution.borrow_mut();
        if let Some(stdout) = execution.stdout.as_mut() {
            stdout.extend_from_slice(bytes);
            true
        } else {
            false
        }
    }) {
        return;
    }

    let mut stdout = io::stdout().lock();
    stdout
        .write_all(bytes)
        .expect("failed to write Echo runtime output");
    stdout.flush().expect("failed to flush Echo runtime output");
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_join(handle: EchoValue) -> EchoValue {
    match handle.kind {
        ECHO_VALUE_TASK => echo_task_join(handle),
        ECHO_VALUE_THREAD => echo_thread_join(handle),
        ECHO_VALUE_PROCESS => echo_process_join(handle),
        _ => {
            eprintln!("error: join target is not a task, thread, or process handle");
            EchoValue::error()
        }
    }
}
