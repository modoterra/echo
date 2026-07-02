use super::*;

#[test]
fn set_time_limit_reports_unsupported_timer_control() {
    assert_eq!(
        echo_php_set_time_limit(EchoValue::int(0)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_set_time_limit(EchoValue::int(1)),
        EchoValue::bool(false)
    );
}
