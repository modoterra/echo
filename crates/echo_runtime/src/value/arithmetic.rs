use super::coercion::{PhpNumber, php_numeric_binary, pow_f64_int};
use super::{ECHO_VALUE_INT, EchoValue};
use crate::collections::php_array_union;
use crate::echo_runtime_string;
use crate::math::echo_math_pow;
use crate::string::php_string_to_number_builtin;

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

fn php_unsigned_base_to_decimal(bytes: &[u8], base: u32) -> EchoValue {
    let mut integer = 0u64;
    let mut float = 0.0;
    let mut overflowed = false;

    for digit in bytes.iter().copied().filter_map(ascii_digit_value) {
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
    for digit in bytes.iter().copied().filter_map(ascii_digit_value) {
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
