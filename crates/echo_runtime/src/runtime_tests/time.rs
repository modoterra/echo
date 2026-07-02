use super::*;
use crate::time::unix_duration_now_or_zero;

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

#[test]
fn time_nanosleep_accepts_valid_delays_and_rejects_invalid_ranges() {
    assert_eq!(
        echo_php_time_nanosleep(EchoValue::int(0), EchoValue::int(0)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_time_nanosleep(EchoValue::int(0), EchoValue::int(1)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_time_nanosleep(EchoValue::int(0), EchoValue::int(1_000_000_000)),
        EchoValue::error()
    );
}

#[test]
fn time_sleep_until_accepts_future_timestamps_and_rejects_past_ones() {
    assert_eq!(
        echo_php_time_sleep_until(EchoValue::int(1)),
        EchoValue::bool(false)
    );
    let future = unix_duration_now_or_zero().as_secs_f64() + 0.001;
    assert_eq!(
        echo_php_time_sleep_until(EchoValue::float(future)),
        EchoValue::bool(true)
    );
}
