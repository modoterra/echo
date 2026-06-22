use super::*;

#[test]
fn angle_conversion_builtins_preserve_php_float_coercion() {
    assert_float_value(echo_php_deg2rad(EchoValue::int(180)), std::f64::consts::PI);
    assert_float_value(
        echo_php_rad2deg(EchoValue::float(std::f64::consts::PI)),
        180.0,
    );
    assert_float_value(
        echo_php_deg2rad(test_string_value(b"-90")),
        -std::f64::consts::FRAC_PI_2,
    );
    assert_float_value(echo_php_rad2deg(EchoValue::bool(true)), 57.29577951308232);
    assert_float_value(echo_php_deg2rad(EchoValue::null()), 0.0);
    assert_eq!(
        echo_php_deg2rad(test_string_value(b"not numeric")),
        EchoValue::error()
    );
}

#[test]
fn trigonometric_builtins_preserve_php_float_coercion() {
    assert_float_value(
        echo_php_sin(EchoValue::float(std::f64::consts::FRAC_PI_6)),
        0.5,
    );
    assert_float_value(
        echo_php_cos(EchoValue::float(std::f64::consts::FRAC_PI_3)),
        0.5,
    );
    assert_float_value(
        echo_php_tan(EchoValue::float(std::f64::consts::FRAC_PI_4)),
        1.0,
    );
    assert_float_value(
        echo_php_asin(EchoValue::float(0.5)),
        std::f64::consts::FRAC_PI_6,
    );
    assert_float_value(
        echo_php_acos(EchoValue::float(0.5)),
        std::f64::consts::FRAC_PI_3,
    );
    assert_float_value(
        echo_php_atan(EchoValue::float(1.0)),
        std::f64::consts::FRAC_PI_4,
    );
    assert_float_value(
        echo_php_atan2(EchoValue::float(3.0), EchoValue::float(-3.0)),
        2.356194490192345,
    );
    assert_float_value(echo_php_sin(test_string_value(b"0.5")), 0.479425538604203);
    assert_float_value(echo_php_cos(EchoValue::bool(true)), 0.5403023058681398);
    assert!(f64::from_bits(echo_php_acos(EchoValue::int(2)).payload).is_nan());
}

#[test]
fn hyperbolic_builtins_preserve_php_float_behavior() {
    assert_float_value(echo_php_sinh(EchoValue::int(0)), 0.0);
    assert_float_value(echo_php_sinh(EchoValue::int(1)), 1.1752011936438014);
    assert_float_value(echo_php_cosh(EchoValue::int(0)), 1.0);
    assert_float_value(echo_php_cosh(EchoValue::int(1)), 1.5430806348152437);
    assert_float_value(echo_php_tanh(EchoValue::int(1)), 0.7615941559557649);
    assert_float_value(echo_php_asinh(EchoValue::int(1)), 0.881373587019543);
    assert_float_value(echo_php_acosh(EchoValue::int(1)), 0.0);
    assert_float_value(echo_php_atanh(EchoValue::float(0.5)), 0.5493061443340548);
    assert_float_value(echo_php_cosh(test_string_value(b"2.5")), 6.132289479663687);
    assert!(f64::from_bits(echo_php_acosh(EchoValue::int(0)).payload).is_nan());
    assert!(f64::from_bits(echo_php_atanh(EchoValue::int(2)).payload).is_nan());
}

#[test]
fn rounding_and_magnitude_builtins_preserve_php_float_behavior() {
    assert_float_value(echo_php_ceil(EchoValue::float(4.3)), 5.0);
    assert_float_value(echo_php_floor(EchoValue::float(9.999)), 9.0);
    assert_float_value(echo_php_floor(EchoValue::float(-3.14)), -4.0);
    assert_eq!(
        f64::from_bits(echo_php_ceil(EchoValue::float(-0.1)).payload).to_bits(),
        (-0.0f64).to_bits()
    );
    assert_float_value(echo_php_ceil(test_string_value(b"12.2")), 13.0);
    assert_float_value(echo_php_floor(EchoValue::bool(true)), 1.0);
    assert_float_value(echo_php_sqrt(EchoValue::int(9)), 3.0);
    assert_float_value(echo_php_sqrt(EchoValue::float(10.0)), 3.162277660168379);
    assert!(f64::from_bits(echo_php_sqrt(EchoValue::int(-1)).payload).is_nan());
    assert_float_value(echo_php_hypot(EchoValue::int(3), EchoValue::int(4)), 5.0);
    assert_float_value(
        echo_php_hypot(test_string_value(b"5"), test_string_value(b"12")),
        13.0,
    );
}

#[test]
fn exponential_and_logarithm_builtins_preserve_php_float_behavior() {
    assert_float_value(echo_php_exp(EchoValue::int(0)), 1.0);
    assert_float_value(echo_php_expm1(EchoValue::int(0)), 0.0);
    assert_float_value(echo_php_log(EchoValue::int(8), EchoValue::int(2)), 3.0);
    assert_float_value(echo_php_log10(EchoValue::int(1000)), 3.0);
    assert_float_value(echo_php_log1p(EchoValue::int(0)), 0.0);
    assert_eq!(
        f64::from_bits(
            echo_php_log(EchoValue::int(0), EchoValue::float(std::f64::consts::E)).payload
        ),
        f64::NEG_INFINITY
    );
    assert!(
        f64::from_bits(
            echo_php_log(EchoValue::int(-1), EchoValue::float(std::f64::consts::E)).payload
        )
        .is_nan()
    );
    assert!(f64::from_bits(echo_php_log1p(EchoValue::int(-2)).payload).is_nan());
    assert_eq!(
        echo_php_log(EchoValue::int(8), EchoValue::int(0)),
        EchoValue::error()
    );
    assert_eq!(
        echo_php_pow(EchoValue::int(2), EchoValue::int(8)),
        EchoValue::int(256)
    );
    assert_float_value(echo_php_pow(EchoValue::int(10), EchoValue::int(-1)), 0.1);
    assert_float_value(
        echo_php_fdiv(EchoValue::int(125), EchoValue::int(100)),
        1.25,
    );
    assert_eq!(
        f64::from_bits(echo_php_fdiv(EchoValue::int(1), EchoValue::int(0)).payload),
        f64::INFINITY
    );
    assert_float_value(
        echo_php_fpow(EchoValue::float(1.05), EchoValue::int(2)),
        1.1025,
    );
    assert_eq!(
        f64::from_bits(echo_php_fpow(EchoValue::int(0), EchoValue::int(-2)).payload),
        f64::INFINITY
    );
    assert!(
        f64::from_bits(echo_php_fpow(EchoValue::int(-1), EchoValue::float(5.5)).payload).is_nan()
    );
    assert!(
        f64::from_bits(echo_php_pow(EchoValue::int(-1), EchoValue::float(5.5)).payload).is_nan()
    );
    assert_eq!(
        f64::from_bits(echo_php_pow(EchoValue::int(0), EchoValue::int(-1)).payload),
        f64::INFINITY
    );
}
