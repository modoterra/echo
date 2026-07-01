use crate::{EchoSymbol, EchoValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoError {
    InvalidCallable,
    UndefinedFunction(EchoSymbol),
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_error_handler() -> EchoValue {
    EchoValue::null()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_exception_handler() -> EchoValue {
    EchoValue::null()
}
