use crate::collections::php_array_union;
use crate::math::echo_math_pow;
use crate::string::{php_string_to_number_builtin, trim_ascii, trim_ascii_start};
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_add(left: EchoValue, right: EchoValue) -> EchoValue {
    if left.is_array() || right.is_array() {
        return php_array_union(left, right);
    }

    php_numeric_binary(
        left,
        right,
        |left, right| left + right,
        |left, right| left + right,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_sub(left: EchoValue, right: EchoValue) -> EchoValue {
    php_numeric_binary(
        left,
        right,
        |left, right| left - right,
        |left, right| left - right,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_mul(left: EchoValue, right: EchoValue) -> EchoValue {
    php_numeric_binary(
        left,
        right,
        |left, right| left * right,
        |left, right| left * right,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_div(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = PhpNumber::coerce(left) else {
        return EchoValue::error();
    };
    let Some(right) = PhpNumber::coerce(right) else {
        return EchoValue::error();
    };

    match (left, right) {
        (_, PhpNumber::Int(0)) | (_, PhpNumber::Float(0.0)) => EchoValue::error(),
        (PhpNumber::Int(left), PhpNumber::Int(right)) if left % right == 0 => {
            EchoValue::int(left / right)
        }
        _ => EchoValue::float(left.as_float() / right.as_float()),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_mod(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = left.php_int_value() else {
        return EchoValue::error();
    };
    let Some(right) = right.php_int_value() else {
        return EchoValue::error();
    };
    if right == 0 {
        return EchoValue::error();
    }

    EchoValue::int(left % right)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_pow(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = PhpNumber::coerce(left) else {
        return EchoValue::error();
    };
    let Some(right) = PhpNumber::coerce(right) else {
        return EchoValue::error();
    };

    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) if right >= 0 => {
            match u32::try_from(right)
                .ok()
                .and_then(|right| left.checked_pow(right))
            {
                Some(value) => EchoValue::int(value),
                None => EchoValue::float(pow_f64_int(left as f64, right)),
            }
        }
        (left, PhpNumber::Int(right)) => EchoValue::float(pow_f64_int(left.as_float(), right)),
        (left, PhpNumber::Float(right)) if right.fract() == 0.0 => {
            EchoValue::float(pow_f64_int(left.as_float(), right as i64))
        }
        (left, PhpNumber::Float(right)) => EchoValue::float(echo_math_pow(left.as_float(), right)),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_unary_plus(value: EchoValue) -> EchoValue {
    match PhpNumber::coerce(value) {
        Some(PhpNumber::Int(value)) => EchoValue::int(value),
        Some(PhpNumber::Float(value)) => EchoValue::float(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_unary_minus(value: EchoValue) -> EchoValue {
    match PhpNumber::coerce(value) {
        Some(PhpNumber::Int(value)) => EchoValue::int(value.saturating_neg()),
        Some(PhpNumber::Float(value)) => EchoValue::float(-value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_bool(value: EchoValue) -> bool {
    value.bool_value().unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_abs(value: EchoValue) -> EchoValue {
    match value.kind {
        ECHO_VALUE_INT => match (value.payload as i64).checked_abs() {
            Some(value) => EchoValue::int(value),
            None => EchoValue::error(),
        },
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_numeric(value: EchoValue) -> EchoValue {
    let is_numeric = match value.kind {
        ECHO_VALUE_INT => true,
        ECHO_VALUE_STRING => unsafe {
            (value.payload as *const EchoString)
                .as_ref()
                .is_some_and(|value| is_php_numeric_string(&value.bytes))
        },
        _ => false,
    };
    EchoValue::bool(is_numeric)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_bindec(value: EchoValue) -> EchoValue {
    php_string_to_number_builtin(value, |bytes| php_unsigned_base_to_decimal(bytes, 2))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hexdec(value: EchoValue) -> EchoValue {
    php_string_to_number_builtin(value, |bytes| php_unsigned_base_to_decimal(bytes, 16))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_octdec(value: EchoValue) -> EchoValue {
    php_string_to_number_builtin(value, |bytes| php_unsigned_base_to_decimal(bytes, 8))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_base_convert(
    value: EchoValue,
    from_base: EchoValue,
    to_base: EchoValue,
) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Some(from_base) = from_base.php_int_value() else {
        return EchoValue::error();
    };
    let Some(to_base) = to_base.php_int_value() else {
        return EchoValue::error();
    };
    if !(2..=36).contains(&from_base) || !(2..=36).contains(&to_base) {
        return EchoValue::error();
    }

    let Some(value) = php_unsigned_base_to_u128(&bytes, from_base as u32) else {
        return EchoValue::error();
    };

    echo_runtime_string(u128_to_base_bytes(value, to_base as u32))
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

fn php_unsigned_base_to_decimal(bytes: &[u8], base: u32) -> EchoValue {
    let mut integer = 0u64;
    let mut float = 0.0;
    let mut overflowed = false;

    for digit in bytes
        .iter()
        .copied()
        .filter_map(|byte| ascii_digit_value(byte))
    {
        if digit >= base {
            continue;
        }

        if overflowed {
            float = float * base as f64 + digit as f64;
            continue;
        }

        match integer
            .checked_mul(base as u64)
            .and_then(|value| value.checked_add(digit as u64))
        {
            Some(value) => integer = value,
            None => {
                overflowed = true;
                float = integer as f64 * base as f64 + digit as f64;
            }
        }
    }

    if overflowed || integer > i64::MAX as u64 {
        EchoValue::float(if overflowed { float } else { integer as f64 })
    } else {
        EchoValue::int(integer as i64)
    }
}

fn php_unsigned_base_to_u128(bytes: &[u8], base: u32) -> Option<u128> {
    let mut value = 0u128;
    for digit in bytes
        .iter()
        .copied()
        .filter_map(|byte| ascii_digit_value(byte))
    {
        if digit >= base {
            continue;
        }

        value = value
            .checked_mul(base as u128)?
            .checked_add(digit as u128)?;
    }

    Some(value)
}

fn u128_to_base_bytes(mut value: u128, base: u32) -> Vec<u8> {
    if value == 0 {
        return b"0".to_vec();
    }

    let mut bytes = Vec::new();
    while value > 0 {
        let digit = (value % base as u128) as u8;
        bytes.push(match digit {
            0..=9 => b'0' + digit,
            _ => b'a' + (digit - 10),
        });
        value /= base as u128;
    }
    bytes.reverse();
    bytes
}

fn ascii_digit_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some((byte - b'0') as u32),
        b'a'..=b'z' => Some((byte - b'a' + 10) as u32),
        b'A'..=b'Z' => Some((byte - b'A' + 10) as u32),
        _ => None,
    }
}

pub(crate) fn parse_php_decimal_int(bytes: &[u8]) -> i64 {
    let bytes = trim_ascii_start(bytes);
    let (negative, digits) = match bytes.first().copied() {
        Some(b'-') => (true, &bytes[1..]),
        Some(b'+') => (false, &bytes[1..]),
        _ => (false, bytes),
    };

    let mut value = 0i64;
    let mut found_digit = false;
    for byte in digits.iter().copied() {
        if !byte.is_ascii_digit() {
            break;
        }
        found_digit = true;
        value = value
            .saturating_mul(10)
            .saturating_add((byte - b'0') as i64);
    }

    if !found_digit {
        return 0;
    }

    if negative {
        value.saturating_neg()
    } else {
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PhpNumber {
    Int(i64),
    Float(f64),
}

impl PhpNumber {
    pub(crate) fn coerce(value: EchoValue) -> Option<Self> {
        match value.kind {
            ECHO_VALUE_NULL => Some(Self::Int(0)),
            ECHO_VALUE_BOOL => Some(Self::Int(if value.payload == 0 { 0 } else { 1 })),
            ECHO_VALUE_INT => Some(Self::Int(value.payload as i64)),
            ECHO_VALUE_FLOAT => Some(Self::Float(f64::from_bits(value.payload))),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(value.payload as *const EchoString).as_ref()?.bytes;
                parse_php_number(bytes)
            },
            _ => None,
        }
    }

    pub(crate) const fn as_float(self) -> f64 {
        match self {
            Self::Int(value) => value as f64,
            Self::Float(value) => value,
        }
    }

    pub(crate) fn into_echo_value(self) -> EchoValue {
        match self {
            Self::Int(value) => EchoValue::int(value),
            Self::Float(value) => EchoValue::float(value),
        }
    }
}

pub(crate) fn php_number_add(left: PhpNumber, right: PhpNumber) -> PhpNumber {
    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) => left
            .checked_add(right)
            .map(PhpNumber::Int)
            .unwrap_or_else(|| PhpNumber::Float(left as f64 + right as f64)),
        _ => PhpNumber::Float(left.as_float() + right.as_float()),
    }
}

pub(crate) fn php_number_mul(left: PhpNumber, right: PhpNumber) -> PhpNumber {
    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) => left
            .checked_mul(right)
            .map(PhpNumber::Int)
            .unwrap_or_else(|| PhpNumber::Float(left as f64 * right as f64)),
        _ => PhpNumber::Float(left.as_float() * right.as_float()),
    }
}

fn php_numeric_binary(
    left: EchoValue,
    right: EchoValue,
    int_op: impl FnOnce(i64, i64) -> i64,
    float_op: impl FnOnce(f64, f64) -> f64,
) -> EchoValue {
    let Some(left) = PhpNumber::coerce(left) else {
        return EchoValue::error();
    };
    let Some(right) = PhpNumber::coerce(right) else {
        return EchoValue::error();
    };

    match (left, right) {
        (PhpNumber::Int(left), PhpNumber::Int(right)) => EchoValue::int(int_op(left, right)),
        _ => EchoValue::float(float_op(left.as_float(), right.as_float())),
    }
}

fn parse_php_number(bytes: &[u8]) -> Option<PhpNumber> {
    let bytes = trim_ascii(bytes);
    if bytes.is_empty() {
        return None;
    }

    let text = std::str::from_utf8(bytes).ok()?;
    if text.contains(['.', 'e', 'E']) {
        text.parse::<f64>().ok().map(PhpNumber::Float)
    } else {
        text.parse::<i64>().ok().map(PhpNumber::Int)
    }
}

fn pow_f64_int(base: f64, exponent: i64) -> f64 {
    if exponent == 0 {
        return 1.0;
    }

    let negative = exponent < 0;
    let mut exponent = exponent.unsigned_abs();
    let mut factor = base;
    let mut value = 1.0;

    while exponent > 0 {
        if exponent & 1 == 1 {
            value *= factor;
        }
        exponent >>= 1;
        factor *= factor;
    }

    if negative { 1.0 / value } else { value }
}

fn is_php_numeric_string(bytes: &[u8]) -> bool {
    let bytes = trim_ascii(bytes);
    if bytes.is_empty() {
        return false;
    }

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
        return false;
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'-' | b'+')) {
            index += 1;
        }
        if consume_ascii_digits(bytes, &mut index) == 0 {
            return false;
        }
    }

    index == bytes.len()
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

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_object_new() -> EchoValue {
    EchoValue::object(Box::into_raw(Box::new(EchoObject::new())))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_object_set(
    object: EchoValue,
    field_ptr: *const u8,
    field_len: usize,
    value: EchoValue,
) -> EchoValue {
    if object.kind != ECHO_VALUE_OBJECT || (field_ptr.is_null() && field_len != 0) {
        return EchoValue::error();
    }

    let Some(fields) = (unsafe { (object.payload as *mut EchoObject).as_mut() }) else {
        return EchoValue::error();
    };
    let field_bytes = unsafe { std::slice::from_raw_parts(field_ptr, field_len) };
    let Ok(field) = std::str::from_utf8(field_bytes) else {
        return EchoValue::error();
    };

    fields.fields.push((field.to_string(), value));
    object
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_object_get(
    object: EchoValue,
    field_ptr: *const u8,
    field_len: usize,
) -> EchoValue {
    if object.kind != ECHO_VALUE_OBJECT || (field_ptr.is_null() && field_len != 0) {
        return EchoValue::error();
    }

    let Some(fields) = (unsafe { (object.payload as *const EchoObject).as_ref() }) else {
        return EchoValue::error();
    };
    let field_bytes = unsafe { std::slice::from_raw_parts(field_ptr, field_len) };
    let Ok(field) = std::str::from_utf8(field_bytes) else {
        return EchoValue::error();
    };

    fields
        .fields
        .iter()
        .rev()
        .find_map(|(name, value)| (name == field).then_some(*value))
        .unwrap_or_else(EchoValue::error)
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
