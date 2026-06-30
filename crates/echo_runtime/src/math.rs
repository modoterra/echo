use crate::string::trim_ascii;
use crate::{EchoValue, echo_runtime_string, echo_value_pow, php_float_cast};

mod elementary;

pub(crate) use elementary::*;

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_finite(value: EchoValue) -> EchoValue {
    php_float_predicate_builtin(value, f64::is_finite)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_infinite(value: EchoValue) -> EchoValue {
    php_float_predicate_builtin(value, f64::is_infinite)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_nan(value: EchoValue) -> EchoValue {
    php_float_predicate_builtin(value, f64::is_nan)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_deg2rad(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, f64::to_radians)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rad2deg(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, f64::to_degrees)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sin(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_sin)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_cos(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_cos)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_tan(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, |value| echo_math_sin(value) / echo_math_cos(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_asin(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_asin)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_acos(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_acos)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_atan(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_atan)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_atan2(y: EchoValue, x: EchoValue) -> EchoValue {
    php_binary_float_builtin(y, x, echo_math_atan2)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_intdiv(num1: EchoValue, num2: EchoValue) -> EchoValue {
    let Some(num1) = num1.php_int_value() else {
        return EchoValue::error();
    };
    let Some(num2) = num2.php_int_value() else {
        return EchoValue::error();
    };

    match num1.checked_div(num2) {
        Some(result) => EchoValue::int(result),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sinh(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_sinh)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_cosh(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_cosh)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_tanh(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_tanh)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_asinh(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_asinh)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_acosh(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_acosh)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_atanh(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_atanh)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ceil(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_ceil)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_floor(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_floor)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_round(value: EchoValue, precision: EchoValue) -> EchoValue {
    let Some(value) = php_float_coercion(value) else {
        return EchoValue::error();
    };
    let Some(precision) = precision.php_int_value() else {
        return EchoValue::error();
    };

    EchoValue::float(php_round_half_away_from_zero(value, precision))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_number_format(
    value: EchoValue,
    decimals: EchoValue,
    decimal_separator: EchoValue,
    thousands_separator: EchoValue,
) -> EchoValue {
    let Some(value) = php_float_cast(value) else {
        return EchoValue::error();
    };
    let Some(decimals) = decimals.php_int_value() else {
        return EchoValue::error();
    };
    let Some(decimal_separator) = decimal_separator.string_bytes() else {
        return EchoValue::error();
    };
    let Some(thousands_separator) = thousands_separator.string_bytes() else {
        return EchoValue::error();
    };

    echo_runtime_string(format_number(
        value,
        decimals.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        &decimal_separator,
        &thousands_separator,
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sqrt(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_sqrt)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_exp(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_exp)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_expm1(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_expm1)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_log(value: EchoValue, base: EchoValue) -> EchoValue {
    match (php_float_coercion(value), php_float_coercion(base)) {
        (Some(value), Some(base)) if base > 0.0 => {
            EchoValue::float(echo_math_ln(value) / echo_math_ln(base))
        }
        (Some(_), Some(_)) => EchoValue::error(),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_log10(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, |value| echo_math_ln(value) / std::f64::consts::LN_10)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_log1p(value: EchoValue) -> EchoValue {
    php_unary_float_builtin(value, echo_math_log1p)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_pow(base: EchoValue, exponent: EchoValue) -> EchoValue {
    echo_value_pow(base, exponent)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fdiv(num1: EchoValue, num2: EchoValue) -> EchoValue {
    php_binary_float_builtin(num1, num2, |num1, num2| num1 / num2)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fpow(base: EchoValue, exponent: EchoValue) -> EchoValue {
    php_binary_float_builtin(base, exponent, echo_math_pow_float)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_hypot(x: EchoValue, y: EchoValue) -> EchoValue {
    php_binary_float_builtin(x, y, |x, y| echo_math_sqrt(x * x + y * y))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_pi() -> EchoValue {
    EchoValue::float(std::f64::consts::PI)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fmod(num1: EchoValue, num2: EchoValue) -> EchoValue {
    php_binary_float_builtin(num1, num2, |num1, num2| num1 % num2)
}

fn php_float_predicate_builtin(value: EchoValue, f: impl FnOnce(f64) -> bool) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::bool(f(value)),
        None => EchoValue::error(),
    }
}

fn php_unary_float_builtin(value: EchoValue, f: impl FnOnce(f64) -> f64) -> EchoValue {
    match php_float_coercion(value) {
        Some(value) => EchoValue::float(f(value)),
        None => EchoValue::error(),
    }
}

fn php_binary_float_builtin(
    left: EchoValue,
    right: EchoValue,
    f: impl FnOnce(f64, f64) -> f64,
) -> EchoValue {
    match (php_float_coercion(left), php_float_coercion(right)) {
        (Some(left), Some(right)) => EchoValue::float(f(left, right)),
        _ => EchoValue::error(),
    }
}

fn php_round_half_away_from_zero(value: f64, precision: i64) -> f64 {
    if !value.is_finite() {
        return value;
    }

    let precision = precision.clamp(-308, 308) as i32;
    if precision >= 0 {
        let factor = 10_f64.powi(precision);
        (value * factor).round() / factor
    } else {
        let factor = 10_f64.powi(-precision);
        (value / factor).round() * factor
    }
}

fn format_number(
    value: f64,
    decimals: i32,
    decimal_separator: &[u8],
    thousands_separator: &[u8],
) -> Vec<u8> {
    let rounded = php_round_half_away_from_zero(value, decimals as i64);
    let display_decimals = decimals.max(0) as usize;
    let sign: &[u8] = if rounded.is_sign_negative() && rounded != 0.0 {
        b"-"
    } else {
        b""
    };
    let abs = rounded.abs();
    let formatted = format!("{abs:.display_decimals$}");
    let (integer, fraction) = formatted
        .split_once('.')
        .map_or((formatted.as_str(), ""), |(integer, fraction)| {
            (integer, fraction)
        });

    let mut bytes = Vec::new();
    bytes.extend_from_slice(sign);
    bytes.extend_from_slice(&group_integer_digits(
        integer.as_bytes(),
        thousands_separator,
    ));
    if display_decimals > 0 {
        bytes.extend_from_slice(decimal_separator);
        bytes.extend_from_slice(fraction.as_bytes());
    }
    bytes
}

fn group_integer_digits(digits: &[u8], thousands_separator: &[u8]) -> Vec<u8> {
    let mut grouped = Vec::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.iter().copied().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.extend_from_slice(thousands_separator);
        }
        grouped.push(digit);
    }
    grouped
}

fn php_float_coercion(value: EchoValue) -> Option<f64> {
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
        let bytes = value.string_bytes()?;
        let text = std::str::from_utf8(trim_ascii(&bytes)).ok()?;
        if text.is_empty() {
            return None;
        }
        return text.parse::<f64>().ok();
    }

    None
}
