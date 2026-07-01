use crate::{EchoSymbol, EchoValue, echo_normalize_callable};
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoError {
    InvalidCallable,
    UndefinedFunction(EchoSymbol),
}

static ERROR_HANDLERS: Mutex<Vec<EchoValue>> = Mutex::new(Vec::new());
static EXCEPTION_HANDLERS: Mutex<Vec<EchoValue>> = Mutex::new(Vec::new());

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
