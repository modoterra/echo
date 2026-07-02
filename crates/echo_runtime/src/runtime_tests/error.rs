use super::*;

#[test]
fn last_error_state_defaults_to_null_and_can_be_cleared() {
    assert_eq!(echo_php_error_get_last(), EchoValue::null());
    assert_eq!(echo_php_error_clear_last(), EchoValue::null());
    assert_eq!(echo_php_error_get_last(), EchoValue::null());
}
