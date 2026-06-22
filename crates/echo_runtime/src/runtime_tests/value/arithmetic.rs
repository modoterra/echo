use super::*;

#[test]
fn abs_preserves_php_integer_absolute_value_behavior() {
    assert_eq!(echo_php_abs(EchoValue::int(42)), EchoValue::int(42));
    assert_eq!(echo_php_abs(EchoValue::int(-42)), EchoValue::int(42));
    assert_eq!(echo_php_abs(EchoValue::int(0)), EchoValue::int(0));
    assert_eq!(echo_php_abs(EchoValue::int(i64::MIN)), EchoValue::error());
    assert_eq!(echo_php_abs(EchoValue::bool(true)), EchoValue::error());
}
