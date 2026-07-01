use crate::{EchoSymbol, EchoValue, echo_normalize_callable};
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoError {
    InvalidCallable,
    UndefinedFunction(EchoSymbol),
}

static ERROR_HANDLERS: Mutex<Vec<EchoValue>> = Mutex::new(Vec::new());

pub(crate) fn reset() {
    if let Ok(mut handlers) = ERROR_HANDLERS.lock() {
        handlers.clear();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_error_handler() -> EchoValue {
    current_error_handler()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_exception_handler() -> EchoValue {
    EchoValue::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_set_error_handler(callback: EchoValue) -> EchoValue {
    match echo_normalize_callable(callback) {
        Ok(Some(_)) => {}
        Ok(None) | Err(_) => return EchoValue::error(),
    }

    let previous = current_error_handler();

    let Ok(mut handlers) = ERROR_HANDLERS.lock() else {
        return EchoValue::error();
    };
    handlers.push(callback);

    previous
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_restore_error_handler() -> EchoValue {
    let Ok(mut handlers) = ERROR_HANDLERS.lock() else {
        return EchoValue::error();
    };

    handlers.pop();
    EchoValue::bool(true)
}

fn current_error_handler() -> EchoValue {
    let Ok(handlers) = ERROR_HANDLERS.lock() else {
        return EchoValue::error();
    };

    handlers.last().copied().unwrap_or_else(EchoValue::null)
}
