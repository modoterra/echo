use crate::error::EchoError;
use crate::{EchoString, EchoValue, output};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoSymbol {
    name: String,
}

impl EchoSymbol {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoCallable {
    Function(EchoSymbol),
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_define(name: EchoValue, _value: EchoValue) -> EchoValue {
    match name.string_bytes() {
        Some(bytes) if !bytes.is_empty() => EchoValue::bool(true),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_defined(name: EchoValue) -> EchoValue {
    let is_defined = name.string_bytes().is_some_and(|bytes| {
        matches!(
            bytes.as_slice(),
            b"PHP_VERSION_ID"
                | b"PHP_VERSION"
                | b"PHP_BUILD_DATE"
                | b"PHP_SAPI"
                | b"PHP_EOL"
                | b"STDERR"
                | b"PASSWORD_DEFAULT"
                | b"PASSWORD_BCRYPT"
                | b"PASSWORD_ARGON2I"
                | b"PASSWORD_ARGON2ID"
                | b"PASSWORD_BCRYPT_DEFAULT_COST"
                | b"HASH_HMAC"
                | b"CRYPT_BLOWFISH"
                | b"CRYPT_STD_DES"
                | b"CRYPT_EXT_DES"
                | b"CRYPT_MD5"
                | b"CRYPT_SHA256"
                | b"CRYPT_SHA512"
        )
    });

    EchoValue::bool(is_defined)
}

pub fn echo_normalize_callable(value: EchoValue) -> Result<Option<EchoCallable>, EchoError> {
    if value.is_null() {
        return Ok(None);
    }

    if value.is_string() {
        let string = unsafe { (value.payload as *const EchoString).as_ref() }
            .ok_or(EchoError::InvalidCallable)?;
        let name = std::str::from_utf8(&string.bytes).map_err(|_| EchoError::InvalidCallable)?;

        return Ok(Some(EchoCallable::Function(EchoSymbol::new(name))));
    }

    Err(EchoError::InvalidCallable)
}

pub fn echo_call(callable: &EchoCallable, _args: &[EchoValue]) -> Result<EchoValue, EchoError> {
    match callable {
        EchoCallable::Function(symbol) if symbol.as_str() == "ob_start" => {
            output::output_ob_start();
            Ok(EchoValue::null())
        }
        EchoCallable::Function(symbol) => Err(EchoError::UndefinedFunction(symbol.clone())),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_call_function(ptr: *const u8, len: usize) -> EchoValue {
    if ptr.is_null() && len != 0 {
        return EchoValue::error();
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let Ok(name) = std::str::from_utf8(bytes) else {
        return EchoValue::error();
    };

    let callable = EchoCallable::Function(EchoSymbol::new(name));
    echo_call(&callable, &[]).unwrap_or_else(|_| EchoValue::error())
}
