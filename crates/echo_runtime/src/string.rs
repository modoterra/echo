use crate::encoding::{quoted_printable_decode_bytes, quoted_printable_encode_bytes};
use crate::{EchoString, EchoValue, echo_runtime_string};

mod compare;
mod pattern;
mod sequence;

pub use compare::{
    echo_php_strcasecmp, echo_php_strcmp, echo_php_strncasecmp, echo_php_strncmp,
    echo_php_substr_compare,
};

pub use pattern::{
    echo_php_str_contains, echo_php_str_ends_with, echo_php_str_ireplace, echo_php_str_replace,
    echo_php_str_starts_with, echo_php_strcspn, echo_php_stripos, echo_php_stristr,
    echo_php_strpbrk, echo_php_strpos, echo_php_strrchr, echo_php_strripos, echo_php_strrpos,
    echo_php_strspn, echo_php_strstr, echo_php_strtr, echo_php_substr_count,
};
pub use sequence::{echo_php_chunk_split, echo_php_explode, echo_php_implode, echo_php_str_split};

const PHP_DEFAULT_TRIM_BYTES: &[u8] = b" \n\r\t\x0b\0";

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
pub unsafe extern "C" fn echo_value_string(ptr: *const u8, len: usize) -> EchoValue {
    if ptr.is_null() && len != 0 {
        return EchoValue::error();
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_concat(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(mut bytes) = left.string_bytes() else {
        return EchoValue::error();
    };
    let Some(right) = right.string_bytes() else {
        return EchoValue::error();
    };

    bytes.extend_from_slice(&right);
    echo_runtime_string(bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_strlen(value: EchoValue) -> EchoValue {
    match value.string_bytes() {
        Some(bytes) => EchoValue::int(bytes.len() as i64),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_trim(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| trim_bytes(bytes, true, true))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ltrim(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| trim_bytes(bytes, true, false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rtrim(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| trim_bytes(bytes, false, true))
}

fn trim_bytes(bytes: &[u8], left: bool, right: bool) -> Vec<u8> {
    let mut start = 0;
    let mut end = bytes.len();

    if left {
        while start < end && PHP_DEFAULT_TRIM_BYTES.contains(&bytes[start]) {
            start += 1;
        }
    }

    if right {
        while end > start && PHP_DEFAULT_TRIM_BYTES.contains(&bytes[end - 1]) {
            end -= 1;
        }
    }

    bytes[start..end].to_vec()
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
pub extern "C" fn echo_php_addslashes(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| {
        let mut escaped = Vec::with_capacity(bytes.len());
        for byte in bytes {
            match byte {
                b'\'' | b'"' | b'\\' => {
                    escaped.push(b'\\');
                    escaped.push(*byte);
                }
                b'\0' => escaped.extend_from_slice(b"\\0"),
                other => escaped.push(*other),
            }
        }
        escaped
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stripslashes(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, |bytes| {
        let mut stripped = Vec::with_capacity(bytes.len());
        let mut index = 0;

        while index < bytes.len() {
            if bytes[index] != b'\\' || index + 1 == bytes.len() {
                stripped.push(bytes[index]);
                index += 1;
                continue;
            }

            match bytes[index + 1] {
                b'0' => stripped.push(b'\0'),
                other => stripped.push(other),
            }
            index += 2;
        }

        stripped
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_quoted_printable_encode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, quoted_printable_encode_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_quoted_printable_decode(value: EchoValue) -> EchoValue {
    php_string_map_builtin(value, quoted_printable_decode_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_nl2br(value: EchoValue, use_xhtml: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let use_xhtml = use_xhtml.bool_value().unwrap_or(true);
    let marker: &[u8] = if use_xhtml { b"<br />" } else { b"<br>" };

    echo_runtime_string(php_nl2br(&bytes, marker))
}

fn php_nl2br(bytes: &[u8], marker: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                result.extend_from_slice(marker);
                result.extend_from_slice(b"\r\n");
                index += 2;
            }
            b'\n' => {
                result.extend_from_slice(marker);
                result.push(b'\n');
                index += 1;
            }
            b'\r' => {
                result.extend_from_slice(marker);
                result.push(b'\r');
                index += 1;
            }
            other => {
                result.push(other);
                index += 1;
            }
        }
    }

    result
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_quotemeta(value: EchoValue) -> EchoValue {
    const META_BYTES: &[u8] = b".\\+*?[^]($)";

    match value.string_bytes() {
        Some(bytes) if bytes.is_empty() => EchoValue::bool(false),
        Some(bytes) => {
            let mut quoted = Vec::with_capacity(bytes.len());
            for byte in bytes {
                if META_BYTES.contains(&byte) {
                    quoted.push(b'\\');
                }
                quoted.push(byte);
            }
            echo_runtime_string(quoted)
        }
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_pad(
    value: EchoValue,
    length: EchoValue,
    pad_string: EchoValue,
    pad_type: EchoValue,
) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };
    let Some(pad_string) = pad_string.string_bytes() else {
        return EchoValue::error();
    };
    let Some(pad_type) = pad_type.php_int_value() else {
        return EchoValue::error();
    };
    let Ok(length) = usize::try_from(length) else {
        return echo_runtime_string(bytes);
    };
    if pad_string.is_empty() {
        return EchoValue::error();
    }

    echo_runtime_string(php_str_pad(&bytes, length, &pad_string, pad_type))
}

fn php_str_pad(bytes: &[u8], length: usize, pad_string: &[u8], pad_type: i64) -> Vec<u8> {
    let missing = length.saturating_sub(bytes.len());
    if missing == 0 {
        return bytes.to_vec();
    }

    let (left, right) = match pad_type {
        0 => (missing, 0),
        2 => (missing / 2, missing - (missing / 2)),
        _ => (0, missing),
    };

    let mut result = Vec::with_capacity(length);
    append_repeated_pad(&mut result, pad_string, left);
    result.extend_from_slice(bytes);
    let current_len = result.len();
    append_repeated_pad(&mut result, pad_string, current_len + right);
    result
}

fn append_repeated_pad(result: &mut Vec<u8>, pad_string: &[u8], target_len: usize) {
    while result.len() < target_len {
        let remaining = target_len - result.len();
        if remaining >= pad_string.len() {
            result.extend_from_slice(pad_string);
        } else {
            result.extend_from_slice(&pad_string[..remaining]);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_substr(value: EchoValue, offset: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(offset) = offset.int_value() else {
        return EchoValue::error();
    };

    let len = bytes.len() as i64;
    let start = if offset >= 0 { offset } else { len + offset }.clamp(0, len);

    echo_runtime_string(bytes[start as usize..].to_vec())
}
