use super::*;

#[test]
fn integer_arithmetic_core_abi_adds_and_subtracts() {
    assert_eq!(
        echo_value_add(EchoValue::int(3), EchoValue::int(5)),
        EchoValue::int(8)
    );
    assert_eq!(
        echo_value_sub(EchoValue::int(3), EchoValue::int(5)),
        EchoValue::int(-2)
    );
    assert_eq!(
        echo_value_sub(EchoValue::int(3), test_string_value(b"not numeric")),
        EchoValue::error()
    );
}

#[test]
fn php_numeric_arithmetic_coerces_strings_bools_and_null() {
    assert_float_value(
        echo_value_add(test_string_value(b"3.2"), test_string_value(b"3.4")),
        6.6,
    );
    assert_eq!(
        echo_value_add(EchoValue::null(), EchoValue::int(5)),
        EchoValue::int(5)
    );
    assert_eq!(
        echo_value_add(EchoValue::bool(true), EchoValue::int(2)),
        EchoValue::int(3)
    );
}

#[test]
fn php_arithmetic_core_abi_handles_remaining_operators() {
    assert_eq!(
        echo_value_mul(EchoValue::int(3), EchoValue::int(5)),
        EchoValue::int(15)
    );
    assert_eq!(
        echo_value_div(EchoValue::int(5), EchoValue::int(2)),
        EchoValue::float(2.5)
    );
    assert_eq!(
        echo_value_div(EchoValue::int(6), EchoValue::int(3)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_value_mod(EchoValue::int(-5), EchoValue::int(3)),
        EchoValue::int(-2)
    );
    assert_eq!(
        echo_value_pow(EchoValue::int(2), EchoValue::int(3)),
        EchoValue::int(8)
    );
    assert_eq!(
        echo_value_unary_minus(EchoValue::float(2.5)),
        EchoValue::float(-2.5)
    );
}
