use crate::{EchoSymbol, EchoValue, echo_normalize_callable};
use std::sync::{
    Mutex,
    atomic::{AtomicI64, Ordering},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoError {
    InvalidCallable,
    UndefinedFunction(EchoSymbol),
}

static ERROR_HANDLERS: Mutex<Vec<EchoValue>> = Mutex::new(Vec::new());
static EXCEPTION_HANDLERS: Mutex<Vec<EchoValue>> = Mutex::new(Vec::new());
static ERROR_REPORTING_LEVEL: AtomicI64 = AtomicI64::new(22527);

pub(crate) fn reset() {
    if let Ok(mut handlers) = ERROR_HANDLERS.lock() {
        handlers.clear();
    }
    if let Ok(mut handlers) = EXCEPTION_HANDLERS.lock() {
        handlers.clear();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_error_handler() -> EchoValue {
    current_handler(&ERROR_HANDLERS)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_exception_handler() -> EchoValue {
    current_handler(&EXCEPTION_HANDLERS)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_error_reporting(level: EchoValue) -> EchoValue {
    let previous = ERROR_REPORTING_LEVEL.load(Ordering::Relaxed);
    if level.is_null() {
        return EchoValue::int(previous);
    }

    let Some(next) = level.php_int_value() else {
        return EchoValue::error();
    };
    ERROR_REPORTING_LEVEL.store(next, Ordering::Relaxed);
    EchoValue::int(previous)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_error_get_last() -> EchoValue {
    EchoValue::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_error_clear_last() -> EchoValue {
    EchoValue::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_error_log(message: EchoValue) -> EchoValue {
    if message.string_bytes().is_none() {
        return EchoValue::error();
    }

    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_trigger_error(message: EchoValue, error_level: EchoValue) -> EchoValue {
    if message.string_bytes().is_none() || error_level.php_int_value().is_none() {
        return EchoValue::error();
    }

    EchoValue::bool(true)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_user_error(message: EchoValue, error_level: EchoValue) -> EchoValue {
    echo_php_trigger_error(message, error_level)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_set_error_handler(callback: EchoValue) -> EchoValue {
    push_handler(&ERROR_HANDLERS, callback)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_restore_error_handler() -> EchoValue {
    pop_handler(&ERROR_HANDLERS)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_set_exception_handler(callback: EchoValue) -> EchoValue {
    push_handler(&EXCEPTION_HANDLERS, callback)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_restore_exception_handler() -> EchoValue {
    pop_handler(&EXCEPTION_HANDLERS)
}

fn push_handler(registry: &Mutex<Vec<EchoValue>>, callback: EchoValue) -> EchoValue {
    match echo_normalize_callable(callback) {
        Ok(Some(_)) => {}
        Ok(None) | Err(_) => return EchoValue::error(),
    }

    let previous = current_handler(registry);

    let Ok(mut handlers) = registry.lock() else {
        return EchoValue::error();
    };
    handlers.push(callback);

    previous
}

fn pop_handler(registry: &Mutex<Vec<EchoValue>>) -> EchoValue {
    let Ok(mut handlers) = registry.lock() else {
        return EchoValue::error();
    };

    handlers.pop();
    EchoValue::bool(true)
}

fn current_handler(registry: &Mutex<Vec<EchoValue>>) -> EchoValue {
    let Ok(handlers) = registry.lock() else {
        return EchoValue::error();
    };

    handlers.last().copied().unwrap_or_else(EchoValue::null)
}
