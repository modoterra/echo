pub(crate) fn echo_math_sin(value: f64) -> f64 {
    if !value.is_finite() {
        return f64::NAN;
    }

    let (x, sign) = reduce_to_half_pi(value);
    let x2 = x * x;
    sign * x
        * (1.0
            + x2 * (-1.0 / 6.0
                + x2 * (1.0 / 120.0
                    + x2 * (-1.0 / 5040.0
                        + x2 * (1.0 / 362880.0 + x2 * (-1.0 / 39916800.0 + x2 / 6227020800.0))))))
}

pub(crate) fn echo_math_cos(value: f64) -> f64 {
    echo_math_sin(std::f64::consts::FRAC_PI_2 - value)
}

pub(crate) fn echo_math_asin(value: f64) -> f64 {
    if !(-1.0..=1.0).contains(&value) {
        return f64::NAN;
    }
    echo_math_atan2(value, echo_math_sqrt(1.0 - value * value))
}

pub(crate) fn echo_math_acos(value: f64) -> f64 {
    if !(-1.0..=1.0).contains(&value) {
        return f64::NAN;
    }
    echo_math_atan2(echo_math_sqrt(1.0 - value * value), value)
}

pub(crate) fn echo_math_atan(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value.is_infinite() {
        return value.signum() * std::f64::consts::FRAC_PI_2;
    }

    let sign = if value < 0.0 { -1.0 } else { 1.0 };
    sign * echo_math_atan_positive(value.abs())
}

pub(crate) fn echo_math_atan2(y: f64, x: f64) -> f64 {
    if y.is_nan() || x.is_nan() {
        return f64::NAN;
    }

    if x > 0.0 {
        echo_math_atan(y / x)
    } else if x < 0.0 && y >= 0.0 {
        echo_math_atan(y / x) + std::f64::consts::PI
    } else if x < 0.0 {
        echo_math_atan(y / x) - std::f64::consts::PI
    } else if y > 0.0 {
        std::f64::consts::FRAC_PI_2
    } else if y < 0.0 {
        -std::f64::consts::FRAC_PI_2
    } else {
        0.0
    }
}

pub(crate) fn echo_math_sinh(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value.is_infinite() {
        return value;
    }

    let abs = value.abs();
    let result = if abs < 0.00000001 {
        value
    } else {
        let exp = echo_math_exp(abs);
        0.5 * (exp - 1.0 / exp)
    };
    result.copysign(value)
}

pub(crate) fn echo_math_cosh(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value.is_infinite() {
        return f64::INFINITY;
    }

    let exp = echo_math_exp(value.abs());
    0.5 * (exp + 1.0 / exp)
}

pub(crate) fn echo_math_tanh(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value == 0.0 {
        return value;
    }
    if value.is_infinite() {
        return value.signum();
    }

    let exp2 = echo_math_exp(2.0 * value.abs());
    let result = (exp2 - 1.0) / (exp2 + 1.0);
    result.copysign(value)
}

pub(crate) fn echo_math_asinh(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        return value;
    }
    let abs = value.abs();
    let result = echo_math_ln(abs + echo_math_sqrt(abs * abs + 1.0));
    result.copysign(value)
}

pub(crate) fn echo_math_acosh(value: f64) -> f64 {
    if value < 1.0 || value.is_nan() {
        return f64::NAN;
    }
    if value.is_infinite() {
        return f64::INFINITY;
    }

    echo_math_ln(value + echo_math_sqrt(value - 1.0) * echo_math_sqrt(value + 1.0))
}

pub(crate) fn echo_math_atanh(value: f64) -> f64 {
    if value.is_nan() || value < -1.0 || value > 1.0 {
        return f64::NAN;
    }
    if value == 1.0 {
        return f64::INFINITY;
    }
    if value == -1.0 {
        return f64::NEG_INFINITY;
    }

    0.5 * echo_math_ln((1.0 + value) / (1.0 - value))
}

pub(crate) fn echo_math_exp(value: f64) -> f64 {
    const LN_2: f64 = std::f64::consts::LN_2;
    const MAX_EXP_INPUT: f64 = 709.782712893384;
    const MIN_EXP_INPUT: f64 = -745.1332191019411;

    if value.is_nan() {
        return f64::NAN;
    }
    if value > MAX_EXP_INPUT {
        return f64::INFINITY;
    }
    if value < MIN_EXP_INPUT {
        return 0.0;
    }
    if value == 0.0 {
        return 1.0;
    }

    let scaled = value / LN_2;
    let k = if scaled >= 0.0 {
        (scaled + 0.5) as i32
    } else {
        (scaled - 0.5) as i32
    };
    let r = value - (k as f64) * LN_2;
    echo_math_pow2(k) * echo_math_exp_kernel(r)
}

pub(crate) fn echo_math_expm1(value: f64) -> f64 {
    if value.is_nan() {
        return f64::NAN;
    }
    if value == 0.0 {
        return value;
    }
    if value.abs() >= 0.000001 {
        return echo_math_exp(value) - 1.0;
    }

    let mut term = value;
    let mut sum = value;
    for n in 2..=24 {
        term *= value / n as f64;
        sum += term;
    }
    sum
}

pub(crate) fn echo_math_ln(value: f64) -> f64 {
    const LN_2: f64 = std::f64::consts::LN_2;

    if value.is_nan() || value < 0.0 {
        return f64::NAN;
    }
    if value == 0.0 {
        return f64::NEG_INFINITY;
    }
    if value.is_infinite() {
        return f64::INFINITY;
    }

    let (mantissa, exponent) = echo_math_frexp(value);
    let y = (mantissa - 1.0) / (mantissa + 1.0);
    let y2 = y * y;
    let mut term = y;
    let mut sum = 0.0;
    let mut denominator = 1.0;

    for _ in 0..48 {
        sum += term / denominator;
        term *= y2;
        denominator += 2.0;
    }

    2.0 * sum + (exponent as f64) * LN_2
}

pub(crate) fn echo_math_log1p(value: f64) -> f64 {
    if value.is_nan() || value < -1.0 {
        return f64::NAN;
    }
    if value == -1.0 {
        return f64::NEG_INFINITY;
    }
    if value.abs() >= 0.000001 {
        return echo_math_ln(1.0 + value);
    }

    let mut term = value;
    let mut sum = value;
    for n in 2..=48 {
        term *= -value;
        sum += term / n as f64;
    }
    sum
}

pub(crate) fn echo_math_pow_float(base: f64, exponent: f64) -> f64 {
    if base.is_nan() || exponent.is_nan() {
        return f64::NAN;
    }
    if exponent == 0.0 {
        return 1.0;
    }
    if base == 0.0 && exponent < 0.0 {
        return f64::INFINITY;
    }
    if base < 0.0 && exponent.fract() != 0.0 {
        return f64::NAN;
    }
    if base == 0.0 {
        return 0.0;
    }

    let magnitude = echo_math_exp(echo_math_ln(base.abs()) * exponent);
    if base < 0.0 && (exponent as i64) % 2 != 0 {
        -magnitude
    } else {
        magnitude
    }
}

pub(crate) fn echo_math_sqrt(value: f64) -> f64 {
    if value < 0.0 {
        return f64::NAN;
    }
    if value == 0.0 || value.is_infinite() {
        return value;
    }

    let mut estimate = if value >= 1.0 { value } else { 1.0 };
    for _ in 0..24 {
        estimate = 0.5 * (estimate + value / estimate);
    }
    estimate
}

pub(crate) fn echo_math_floor(value: f64) -> f64 {
    if !value.is_finite() || value.abs() >= i64::MAX as f64 {
        return value;
    }

    let truncated = value as i64 as f64;
    if value < truncated {
        truncated - 1.0
    } else {
        truncated
    }
}

pub(crate) fn echo_math_ceil(value: f64) -> f64 {
    if !value.is_finite() || value.abs() >= i64::MAX as f64 {
        return value;
    }

    let truncated = value as i64 as f64;
    if value < 0.0 && truncated == 0.0 {
        -0.0
    } else if value > truncated {
        truncated + 1.0
    } else {
        truncated
    }
}

pub(crate) fn echo_math_pow(base: f64, exponent: f64) -> f64 {
    if exponent == 0.0 {
        return 1.0;
    }
    if base == 0.0 {
        return if exponent.is_sign_negative() {
            f64::INFINITY
        } else {
            0.0
        };
    }
    if base < 0.0 {
        return f64::NAN;
    }
    if base.is_nan() || exponent.is_nan() {
        return f64::NAN;
    }
    if base.is_infinite() {
        return if exponent.is_sign_negative() {
            0.0
        } else {
            f64::INFINITY
        };
    }

    echo_math_exp(echo_math_ln(base) * exponent)
}

fn echo_math_atan_positive(value: f64) -> f64 {
    if value > 1.0 {
        return std::f64::consts::FRAC_PI_2 - echo_math_atan_positive(1.0 / value);
    }
    if value > 0.41421356237309503 {
        return std::f64::consts::FRAC_PI_4 + echo_math_atan_kernel((value - 1.0) / (value + 1.0));
    }

    echo_math_atan_kernel(value)
}

fn echo_math_atan_kernel(value: f64) -> f64 {
    let x2 = value * value;
    value
        * (1.0
            + x2 * (-1.0 / 3.0
                + x2 * (1.0 / 5.0
                    + x2 * (-1.0 / 7.0
                        + x2 * (1.0 / 9.0
                            + x2 * (-1.0 / 11.0
                                + x2 * (1.0 / 13.0 + x2 * (-1.0 / 15.0 + x2 / 17.0))))))))
}

fn echo_math_exp_kernel(value: f64) -> f64 {
    let mut term = 1.0;
    let mut sum = 1.0;
    for n in 1..=24 {
        term *= value / n as f64;
        sum += term;
    }
    sum
}

fn echo_math_pow2(exp: i32) -> f64 {
    if exp > 1023 {
        return f64::INFINITY;
    }
    if exp < -1074 {
        return 0.0;
    }
    if exp >= -1022 {
        return f64::from_bits(((exp + 1023) as u64) << 52);
    }
    f64::from_bits(1_u64 << (exp + 1074))
}

fn echo_math_frexp(value: f64) -> (f64, i32) {
    let bits = value.to_bits();
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    let fraction_bits = bits & 0x000f_ffff_ffff_ffff;

    if exponent_bits == 0 {
        let (mantissa, exponent) = echo_math_frexp(value * echo_math_pow2(52));
        return (mantissa, exponent - 52);
    }

    let mantissa_bits = (1023_u64 << 52) | fraction_bits;
    (f64::from_bits(mantissa_bits), exponent_bits - 1023)
}

fn reduce_to_half_pi(value: f64) -> (f64, f64) {
    let mut x = value - (value / std::f64::consts::TAU) as i64 as f64 * std::f64::consts::TAU;
    if x > std::f64::consts::PI {
        x -= std::f64::consts::TAU;
    } else if x < -std::f64::consts::PI {
        x += std::f64::consts::TAU;
    }

    if x > std::f64::consts::FRAC_PI_2 {
        (std::f64::consts::PI - x, 1.0)
    } else if x < -std::f64::consts::FRAC_PI_2 {
        (-std::f64::consts::PI - x, -1.0)
    } else {
        (x, 1.0)
    }
}
