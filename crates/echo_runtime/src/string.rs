use crate::{EchoValue, echo_runtime_string};

pub(crate) fn php_string_transform_builtin(
    value: EchoValue,
    f: impl FnOnce(&mut Vec<u8>),
) -> EchoValue {
    match value.string_bytes() {
        Some(mut bytes) => {
            f(&mut bytes);
            echo_runtime_string(bytes)
        }
        None => EchoValue::error(),
    }
}

pub(crate) fn php_string_map_builtin(
    value: EchoValue,
    f: impl FnOnce(&[u8]) -> Vec<u8>,
) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => echo_runtime_string(f(&bytes)),
        None => EchoValue::error(),
    }
}

pub(crate) fn php_int_to_string_builtin(
    value: EchoValue,
    f: impl FnOnce(i64) -> Vec<u8>,
) -> EchoValue {
    match value.php_int_value() {
        Some(number) => echo_runtime_string(f(number)),
        None => EchoValue::error(),
    }
}

pub(crate) fn php_string_to_number_builtin(
    value: EchoValue,
    f: impl FnOnce(&[u8]) -> EchoValue,
) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => f(&bytes),
        None => EchoValue::error(),
    }
}

pub(crate) fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let bytes = trim_ascii_start(bytes);
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(0, |index| index + 1);
    &bytes[..end]
}

pub(crate) fn trim_ascii_start(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    &bytes[start..]
}
