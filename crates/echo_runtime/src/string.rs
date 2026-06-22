use crate::{EchoArray, EchoString, EchoValue, echo_runtime_string};

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
pub extern "C" fn echo_php_chr(value: EchoValue) -> EchoValue {
    match value.int_value() {
        Some(codepoint) => {
            let byte = codepoint.rem_euclid(256) as u8;
            echo_runtime_string(vec![byte])
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_dechex(value: EchoValue) -> EchoValue {
    php_int_to_string_builtin(value, |number| format!("{:x}", number as u64).into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_decbin(value: EchoValue) -> EchoValue {
    php_int_to_string_builtin(value, |number| format!("{:b}", number as u64).into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_decoct(value: EchoValue) -> EchoValue {
    php_int_to_string_builtin(value, |number| format!("{:o}", number as u64).into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strval(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => echo_runtime_string(bytes),
        None => EchoValue::error(),
    }
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_repeat(value: EchoValue, times: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(times) = times.int_value() else {
        return EchoValue::error();
    };
    let Ok(times) = usize::try_from(times) else {
        return EchoValue::error();
    };

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        bytes.repeat(times),
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_explode(
    separator: EchoValue,
    string: EchoValue,
    limit: EchoValue,
) -> EchoValue {
    let Some(separator) = separator.string_bytes() else {
        return EchoValue::error();
    };
    let Some(string) = string.string_bytes() else {
        return EchoValue::error();
    };
    let Some(limit) = limit.php_int_value() else {
        return EchoValue::error();
    };
    if separator.is_empty() {
        return EchoValue::error();
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        explode_bytes(&separator, &string, limit)
            .into_iter()
            .map(echo_runtime_string)
            .collect(),
    ))))
}

fn explode_bytes(separator: &[u8], string: &[u8], limit: i64) -> Vec<Vec<u8>> {
    let limit = if limit == 0 { 1 } else { limit };
    if limit > 0 {
        return explode_bytes_positive_limit(separator, string, limit as usize);
    }

    let mut parts = explode_bytes_all(separator, string);
    let omit = limit.unsigned_abs() as usize;
    let keep = parts.len().saturating_sub(omit);
    parts.truncate(keep);
    parts
}

fn explode_bytes_positive_limit(separator: &[u8], string: &[u8], limit: usize) -> Vec<Vec<u8>> {
    if limit == 1 {
        return vec![string.to_vec()];
    }

    let mut parts = Vec::new();
    let mut start = 0;
    while parts.len() + 1 < limit {
        let Some(offset) = find_subslice(&string[start..], separator) else {
            break;
        };
        let end = start + offset;
        parts.push(string[start..end].to_vec());
        start = end + separator.len();
    }
    parts.push(string[start..].to_vec());
    parts
}

fn explode_bytes_all(separator: &[u8], string: &[u8]) -> Vec<Vec<u8>> {
    let mut parts = Vec::new();
    let mut start = 0;
    while let Some(offset) = find_subslice(&string[start..], separator) {
        let end = start + offset;
        parts.push(string[start..end].to_vec());
        start = end + separator.len();
    }
    parts.push(string[start..].to_vec());
    parts
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_implode(separator: EchoValue, array: EchoValue) -> EchoValue {
    let Some(separator) = separator.string_bytes() else {
        return EchoValue::error();
    };
    if !array.is_array() {
        return EchoValue::error();
    }
    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return EchoValue::error();
    };

    let mut joined = Vec::new();
    for (index, value) in array.values.iter().enumerate() {
        if index > 0 {
            joined.extend_from_slice(&separator);
        }
        let Some(bytes) = value.string_bytes() else {
            return EchoValue::error();
        };
        joined.extend_from_slice(&bytes);
    }

    echo_runtime_string(joined)
}
