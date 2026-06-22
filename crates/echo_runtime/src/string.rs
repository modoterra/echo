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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strtoupper(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| bytes.make_ascii_uppercase())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strtolower(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| bytes.make_ascii_lowercase())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ucwords(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| {
        let mut uppercase_next = true;
        for byte in bytes {
            if uppercase_next {
                byte.make_ascii_uppercase();
            }
            uppercase_next = matches!(*byte, b' ' | b'\t' | b'\r' | b'\n' | 0x0c | 0x0b);
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strrev(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| bytes.reverse())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ucfirst(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| {
        if let Some(first) = bytes.first_mut() {
            first.make_ascii_uppercase();
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_lcfirst(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| {
        if let Some(first) = bytes.first_mut() {
            first.make_ascii_lowercase();
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ord(value: EchoValue) -> EchoValue {
    match value
        .string_bytes()
        .and_then(|bytes| bytes.first().copied())
    {
        Some(byte) => EchoValue::int(byte as i64),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_rot13(value: EchoValue) -> EchoValue {
    php_string_transform_builtin(value, |bytes| {
        for byte in bytes {
            *byte = match *byte {
                b'a'..=b'm' | b'A'..=b'M' => *byte + 13,
                b'n'..=b'z' | b'N'..=b'Z' => *byte - 13,
                other => other,
            };
        }
    })
}
