use super::*;

#[test]
fn last_error_state_defaults_to_null_and_can_be_cleared() {
    assert_eq!(echo_php_error_get_last(), EchoValue::null());
    assert_eq!(echo_php_error_clear_last(), EchoValue::null());
    assert_eq!(echo_php_error_get_last(), EchoValue::null());
}

#[test]
fn error_log_accepts_default_logger_message() {
    assert_eq!(
        echo_php_error_log(test_string_value(b"echo compatibility error log")),
        EchoValue::bool(true)
    );
}
