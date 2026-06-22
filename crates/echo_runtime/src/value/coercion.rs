use super::{
    ECHO_VALUE_BOOL, ECHO_VALUE_FLOAT, ECHO_VALUE_INT, ECHO_VALUE_NULL, ECHO_VALUE_STRING,
    EchoString, EchoValue,
};
use crate::string::{trim_ascii, trim_ascii_start};

pub(super) fn php_float_cast(value: EchoValue) -> Option<f64> {
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

pub(super) fn parse_php_decimal_int(bytes: &[u8]) -> i64 {
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

pub(super) fn php_numeric_binary(
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

pub(super) fn pow_f64_int(base: f64, exponent: i64) -> f64 {
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

pub(super) fn is_php_numeric_string(bytes: &[u8]) -> bool {
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
