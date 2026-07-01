use crate::collections::{EchoArray, EchoArrayKey};
use crate::error::EchoError;
use crate::{EchoString, EchoValue, echo_runtime_string, output};

#[derive(Debug, Clone, Copy)]
enum PhpConstantValue {
    Int(i64),
    String(&'static [u8]),
}

const PHP_COMPAT_CONSTANTS: &[(&str, PhpConstantValue)] = &[
    ("PHP_VERSION_ID", PhpConstantValue::Int(80200)),
    ("PHP_VERSION", PhpConstantValue::String(b"8.2.0")),
    (
        "PHP_BUILD_DATE",
        PhpConstantValue::String(b"Jul  1 2026 00:00:00"),
    ),
    ("PHP_SAPI", PhpConstantValue::String(b"cli")),
    ("PHP_EOL", PhpConstantValue::String(b"\n")),
    ("STDERR", PhpConstantValue::String(b"php://stderr")),
    ("PASSWORD_DEFAULT", PhpConstantValue::String(b"2y")),
    ("PASSWORD_BCRYPT", PhpConstantValue::String(b"2y")),
    ("PASSWORD_ARGON2I", PhpConstantValue::String(b"argon2i")),
    ("PASSWORD_ARGON2ID", PhpConstantValue::String(b"argon2id")),
    ("PASSWORD_BCRYPT_DEFAULT_COST", PhpConstantValue::Int(10)),
    ("HASH_HMAC", PhpConstantValue::Int(1)),
    ("CRYPT_BLOWFISH", PhpConstantValue::Int(1)),
    ("CRYPT_STD_DES", PhpConstantValue::Int(1)),
    ("CRYPT_EXT_DES", PhpConstantValue::Int(1)),
    ("CRYPT_MD5", PhpConstantValue::Int(1)),
    ("CRYPT_SHA256", PhpConstantValue::Int(1)),
    ("CRYPT_SHA512", PhpConstantValue::Int(1)),
];

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
pub extern "C" fn echo_php_constant(name: EchoValue) -> EchoValue {
    let Some(name) = name.string_bytes() else {
        return EchoValue::error();
    };

    php_compat_constant_value(&name).unwrap_or_else(EchoValue::error)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_defined(name: EchoValue) -> EchoValue {
    let is_defined = name
        .string_bytes()
        .is_some_and(|bytes| php_compat_constant_value(&bytes).is_some());

    EchoValue::bool(is_defined)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_defined_constants(categorize: EchoValue) -> EchoValue {
    let constants = php_compat_constants_array();
    if categorize.bool_value().unwrap_or(false) {
        return php_array_from_pairs(vec![("Core", constants)]);
    }

    constants
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

fn php_compat_constants_array() -> EchoValue {
    let mut keys = Vec::with_capacity(PHP_COMPAT_CONSTANTS.len());
    let mut values = Vec::with_capacity(PHP_COMPAT_CONSTANTS.len());

    for (name, value) in PHP_COMPAT_CONSTANTS {
        keys.push(EchoArrayKey::String(name.as_bytes().to_vec()));
        values.push(match value {
            PhpConstantValue::Int(value) => EchoValue::int(*value),
            PhpConstantValue::String(bytes) => echo_runtime_string(bytes.to_vec()),
        });
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

fn php_compat_constant_value(name: &[u8]) -> Option<EchoValue> {
    PHP_COMPAT_CONSTANTS
        .iter()
        .find(|(constant_name, _)| constant_name.as_bytes() == name)
        .map(|(_, value)| match value {
            PhpConstantValue::Int(value) => EchoValue::int(*value),
            PhpConstantValue::String(bytes) => echo_runtime_string(bytes.to_vec()),
        })
}

fn php_array_from_pairs(pairs: Vec<(&str, EchoValue)>) -> EchoValue {
    let mut keys = Vec::with_capacity(pairs.len());
    let mut values = Vec::with_capacity(pairs.len());

    for (name, value) in pairs {
        keys.push(EchoArrayKey::String(name.as_bytes().to_vec()));
        values.push(value);
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}
