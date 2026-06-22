use crate::string::trim_ascii;
use crate::{EchoValue, echo_value_pow};

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
