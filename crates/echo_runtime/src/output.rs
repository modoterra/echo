use std::cell::RefCell;
use std::io::{self as std_io, Write};

use crate::{
    ECHO_VALUE_ARRAY, ECHO_VALUE_BOOL, ECHO_VALUE_ERROR, ECHO_VALUE_FLOAT, ECHO_VALUE_INT,
    ECHO_VALUE_LIST, ECHO_VALUE_NULL, ECHO_VALUE_STRING, EchoString, EchoValue, assertions,
    echo_normalize_callable, echo_runtime_string, echo_value_array_append, echo_value_array_new,
    echo_value_array_set,
    execution::{repl_inspect_enabled, write_stdout},
    format_php_float,
};

mod buffer;

pub use buffer::OutputRuntime;

thread_local! {
    static OUTPUT: RefCell<OutputRuntime> = RefCell::new(OutputRuntime::new());
}

pub(crate) fn reset_output_runtime() {
    OUTPUT.with(|runtime| {
        *runtime.borrow_mut() = OutputRuntime::new();
    });
}

pub(crate) fn output_ob_start() {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_start());
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_write(ptr: *const u8, len: usize) {
    if ptr.is_null() && len != 0 {
        return;
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    write_runtime_output(bytes);
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_write_i64(value: i64) {
    let bytes = value.to_string();
    write_runtime_output(bytes.as_bytes());
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_write_i64_or_false(value: i64) {
    // PHP echoes `false` as an empty string. Echo uses -1 as the current sentinel
    // for int|false runtime calls where supported integer results cannot be negative.
    if value >= 0 {
        echo_write_i64(value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_write_string(value: *const EchoString) {
    if value.is_null() {
        return;
    }

    let bytes = unsafe { &(*value).bytes };
    write_runtime_output(bytes);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_write_value(value: EchoValue) {
    if repl_inspect_enabled() {
        if let Some(bytes) = value.inspect_bytes() {
            unsafe { echo_write(bytes.as_ptr(), bytes.len()) };
        }
        return;
    }

    match value.kind {
        ECHO_VALUE_NULL | ECHO_VALUE_ERROR => {}
        ECHO_VALUE_BOOL => {
            if value.payload != 0 {
                unsafe { echo_write(c"1".as_ptr().cast(), 1) };
            }
        }
        ECHO_VALUE_INT => echo_write_i64(value.payload as i64),
        ECHO_VALUE_FLOAT => {
            let bytes = format_php_float(f64::from_bits(value.payload)).into_bytes();
            unsafe { echo_write(bytes.as_ptr(), bytes.len()) };
        }
        ECHO_VALUE_STRING => unsafe { echo_write_string(value.payload as *const EchoString) },
        ECHO_VALUE_ARRAY => unsafe { echo_write(c"Array".as_ptr().cast(), 5) },
        ECHO_VALUE_LIST => unsafe { echo_write(c"List".as_ptr().cast(), 4) },
        _ => {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_flush() {
    let _ = std_io::stdout().flush();
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_implicit_flush(value: EchoValue) {
    OUTPUT.with(|runtime| {
        runtime
            .borrow_mut()
            .ob_implicit_flush(value.bool_value().unwrap_or(false));
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_start() -> bool {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_start());
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_start_value(callback: EchoValue) -> bool {
    let Ok(callback) = echo_normalize_callable(callback) else {
        return false;
    };

    OUTPUT.with(|runtime| runtime.borrow_mut().ob_start_with_callback(callback));
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_clean() -> bool {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_clean())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_flush() -> bool {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let ok = runtime.borrow_mut().ob_flush(&mut stdout);
        write_stdout(&stdout);
        ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_end_flush() -> bool {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let ok = runtime.borrow_mut().ob_end_flush(&mut stdout);
        write_stdout(&stdout);
        ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_end_clean() -> bool {
    OUTPUT.with(|runtime| runtime.borrow_mut().ob_end_clean())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_clean() -> EchoValue {
    OUTPUT.with(|runtime| match runtime.borrow_mut().ob_get_clean() {
        Some(value) => EchoValue::string(Box::into_raw(Box::new(value))),
        None => EchoValue::bool(false),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_contents() -> EchoValue {
    OUTPUT.with(|runtime| match runtime.borrow().ob_get_contents() {
        Some(value) => EchoValue::string(Box::into_raw(Box::new(value))),
        None => EchoValue::bool(false),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_flush() -> EchoValue {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        let value = runtime.borrow_mut().ob_get_flush(&mut stdout);
        write_stdout(&stdout);
        match value {
            Some(value) => EchoValue::string(Box::into_raw(Box::new(value))),
            None => EchoValue::bool(false),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_level() -> EchoValue {
    // PHP `ob_get_level()` returns zero when inactive; the first active buffer is level 1.
    // Source: https://www.php.net/manual/en/function.ob-get-level.php
    OUTPUT.with(|runtime| EchoValue::int(runtime.borrow().level() as i64))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_length() -> EchoValue {
    OUTPUT.with(|runtime| match runtime.borrow().ob_get_length() {
        Some(len) => EchoValue::int(len as i64),
        None => EchoValue::bool(false),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_list_handlers() -> EchoValue {
    OUTPUT.with(|runtime| {
        let mut result = echo_value_array_new();
        for handler in runtime.borrow().ob_list_handlers() {
            result = echo_value_array_append(result, echo_runtime_string(handler));
        }
        result
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ob_get_status(full_status: EchoValue) -> EchoValue {
    OUTPUT.with(|runtime| {
        let statuses = runtime
            .borrow()
            .ob_get_status(full_status.bool_value().unwrap_or(false));
        let full_status = full_status.bool_value().unwrap_or(false);

        if full_status {
            let mut result = echo_value_array_new();
            for status in statuses {
                result = echo_value_array_append(result, output_buffer_status_array(status));
            }
            return result;
        }

        statuses
            .into_iter()
            .next()
            .map(output_buffer_status_array)
            .unwrap_or_else(|| echo_value_array_new())
    })
}

fn output_buffer_status_array(status: buffer::OutputBufferStatus) -> EchoValue {
    let mut result = echo_value_array_new();
    result = echo_value_array_set(
        result,
        echo_runtime_string(b"name".to_vec()),
        echo_runtime_string(status.name),
    );
    result = echo_value_array_set(
        result,
        echo_runtime_string(b"type".to_vec()),
        EchoValue::int(status.r#type),
    );
    result = echo_value_array_set(
        result,
        echo_runtime_string(b"flags".to_vec()),
        EchoValue::int(status.flags),
    );
    result = echo_value_array_set(
        result,
        echo_runtime_string(b"level".to_vec()),
        EchoValue::int(status.level),
    );
    result = echo_value_array_set(
        result,
        echo_runtime_string(b"chunk_size".to_vec()),
        EchoValue::int(status.chunk_size),
    );
    result = echo_value_array_set(
        result,
        echo_runtime_string(b"buffer_size".to_vec()),
        EchoValue::int(status.buffer_size),
    );
    echo_value_array_set(
        result,
        echo_runtime_string(b"buffer_used".to_vec()),
        EchoValue::int(status.buffer_used),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_shutdown() {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        runtime.borrow_mut().shutdown(&mut stdout);
        write_stdout(&stdout);
    });

    if assertions::has_failures() {
        std::process::exit(1);
    }
}

pub(crate) fn write_runtime_output(bytes: &[u8]) {
    OUTPUT.with(|runtime| {
        let mut stdout = Vec::new();
        runtime.borrow_mut().write(bytes, &mut stdout);
        write_stdout(&stdout);
    });
}
