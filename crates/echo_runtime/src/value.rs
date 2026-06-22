use crate::string::trim_ascii_start;
use crate::{
    ECHO_VALUE_ARRAY, ECHO_VALUE_BOOL, ECHO_VALUE_FLOAT, ECHO_VALUE_INT, ECHO_VALUE_LIST,
    ECHO_VALUE_NULL, ECHO_VALUE_OBJECT, ECHO_VALUE_PROCESS, ECHO_VALUE_STRING, ECHO_VALUE_TASK,
    ECHO_VALUE_TASK_GROUP, ECHO_VALUE_TCP_CONNECTION, ECHO_VALUE_TCP_LISTENER, ECHO_VALUE_THREAD,
    echo_runtime_string,
};
pub use crate::{EchoArray, EchoCallable, EchoList, EchoSymbol, EchoValue};

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gettype(value: EchoValue) -> EchoValue {
    let type_name = match value.kind {
        ECHO_VALUE_NULL => b"NULL".as_slice(),
        ECHO_VALUE_BOOL => b"boolean".as_slice(),
        ECHO_VALUE_INT => b"integer".as_slice(),
        ECHO_VALUE_FLOAT => b"double".as_slice(),
        ECHO_VALUE_STRING => b"string".as_slice(),
        ECHO_VALUE_ARRAY => b"array".as_slice(),
        ECHO_VALUE_LIST => b"list".as_slice(),
        ECHO_VALUE_TASK
        | ECHO_VALUE_TASK_GROUP
        | ECHO_VALUE_OBJECT
        | ECHO_VALUE_PROCESS
        | ECHO_VALUE_THREAD => b"object".as_slice(),
        ECHO_VALUE_TCP_LISTENER | ECHO_VALUE_TCP_CONNECTION => b"resource".as_slice(),
        _ => b"unknown type".as_slice(),
    };
    echo_runtime_string(type_name.to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_array(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_countable(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array() || value.is_list())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_iterable(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array() || value.is_list())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_null(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_null())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_bool(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_bool())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_int(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_int())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_float(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_float())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_object(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_object())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_resource(value: EchoValue) -> EchoValue {
    EchoValue::bool(matches!(
        value.kind,
        ECHO_VALUE_TCP_LISTENER
            | ECHO_VALUE_TCP_CONNECTION
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_THREAD
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_string(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_scalar(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_bool() || value.is_int() || value.is_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_boolval(value: EchoValue) -> EchoValue {
    match value.bool_value() {
        Some(value) => EchoValue::bool(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_intval(value: EchoValue) -> EchoValue {
    match value.php_int_value() {
        Some(value) => EchoValue::int(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_floatval(value: EchoValue) -> EchoValue {
    match php_float_cast(value) {
        Some(value) => EchoValue::float(value),
        None => EchoValue::error(),
    }
}

fn php_float_cast(value: EchoValue) -> Option<f64> {
    if value.is_null() || value.kind == EchoValue::error().kind {
        return Some(0.0);
    }
    if value.is_bool() {
        return Some(if value.payload == 0 { 0.0 } else { 1.0 });
    }
    if value.is_int() {
        return Some(value.payload as i64 as f64);
    }
    if value.is_float() {
        return Some(f64::from_bits(value.payload));
    }
    if value.is_string() {
        return Some(parse_php_decimal_float_prefix(&value.string_bytes()?));
    }

    None
}

fn parse_php_decimal_float_prefix(bytes: &[u8]) -> f64 {
    let bytes = trim_ascii_start(bytes);
    let mut index = match bytes.first().copied() {
        Some(b'-' | b'+') => 1,
        _ => 0,
    };

    let integer_digits = consume_ascii_digits(bytes, &mut index);
    let fraction_digits = if bytes.get(index) == Some(&b'.') {
        index += 1;
        consume_ascii_digits(bytes, &mut index)
    } else {
        0
    };

    if integer_digits + fraction_digits == 0 {
        return 0.0;
    }

    let mut end = index;
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent_start = index;
        index += 1;
        if matches!(bytes.get(index), Some(b'-' | b'+')) {
            index += 1;
        }
        if consume_ascii_digits(bytes, &mut index) > 0 {
            end = index;
        } else {
            end = exponent_start;
        }
    }

    std::str::from_utf8(&bytes[..end])
        .ok()
        .and_then(|text| text.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn consume_ascii_digits(bytes: &[u8], index: &mut usize) -> usize {
    let start = *index;
    while bytes.get(*index).is_some_and(u8::is_ascii_digit) {
        *index += 1;
    }
    *index - start
}

pub(crate) fn format_php_float(value: f64) -> String {
    if value.is_nan() {
        return "NAN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-INF".to_string()
        } else {
            "INF".to_string()
        };
    }

    let formatted = format!("{value:.14}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[derive(Debug)]
pub struct EchoString {
    pub(crate) bytes: Vec<u8>,
}

impl EchoString {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

#[derive(Debug)]
pub struct EchoObject {
    pub(crate) fields: Vec<(String, EchoValue)>,
}

impl EchoObject {
    pub(crate) fn new() -> Self {
        Self { fields: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn php_float_formatting_matches_runtime_scalar_strings() {
        assert_eq!(format_php_float(12.5), "12.5");
        assert_eq!(format_php_float(12.0), "12");
        assert_eq!(format_php_float(f64::INFINITY), "INF");
        assert_eq!(format_php_float(f64::NEG_INFINITY), "-INF");
        assert_eq!(format_php_float(f64::NAN), "NAN");
    }
}
