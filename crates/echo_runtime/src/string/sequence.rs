use crate::{EchoArray, EchoValue, echo_runtime_string};

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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_str_split(value: EchoValue, length: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };
    let Ok(length) = usize::try_from(length) else {
        return EchoValue::error();
    };
    if length == 0 {
        return EchoValue::error();
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(
        bytes
            .chunks(length)
            .map(|chunk| echo_runtime_string(chunk.to_vec()))
            .collect(),
    ))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_chunk_split(
    value: EchoValue,
    length: EchoValue,
    separator: EchoValue,
) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(length) = length.php_int_value() else {
        return EchoValue::error();
    };
    let Some(separator) = separator.string_bytes() else {
        return EchoValue::error();
    };
    let Ok(length) = usize::try_from(length) else {
        return EchoValue::error();
    };
    if length == 0 {
        return EchoValue::error();
    }

    echo_runtime_string(php_chunk_split(&bytes, length, &separator))
}

fn php_chunk_split(bytes: &[u8], length: usize, separator: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len() + separator.len());
    for chunk in bytes.chunks(length) {
        result.extend_from_slice(chunk);
        result.extend_from_slice(separator);
    }
    if bytes.is_empty() {
        result.extend_from_slice(separator);
    }
    result
}
