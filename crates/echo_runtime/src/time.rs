use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{EchoValue, echo_runtime_string};

pub(crate) fn unix_duration_now_or_zero() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
}

#[cfg(not(unix))]
pub(crate) fn system_time_unix_timestamp(time: SystemTime) -> Option<i64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
}

pub fn sleep(millis: i64) {
    if millis > 0 {
        std::thread::sleep(Duration::from_millis(millis as u64));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_time_sleep(millis: i64) {
    sleep(millis);
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_microtime(as_float: EchoValue) -> EchoValue {
    let now = unix_duration_now_or_zero();

    if as_float.bool_value().unwrap_or(false) {
        return EchoValue::float(now.as_secs_f64());
    }

    let micros = now.subsec_micros();
    echo_runtime_string(format!("0.{micros:06} {}", now.as_secs()).into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn sleep_ignores_non_positive_durations() {
        let started = Instant::now();

        echo_time_sleep(0);
        echo_time_sleep(-1);

        assert!(started.elapsed() < Duration::from_millis(50));
    }

    #[test]
    fn microtime_reports_float_seconds_when_requested() {
        let value = echo_php_microtime(EchoValue::bool(true));

        assert!(value.is_float());
        assert!(f64::from_bits(value.payload) > 0.0);
    }

    #[test]
    fn microtime_reports_php_string_shape_by_default() {
        let value = echo_php_microtime(EchoValue::bool(false));
        let bytes = value.string_bytes().expect("microtime string");
        let text = std::str::from_utf8(&bytes).expect("utf8 microtime");
        let mut parts = text.split(' ');
        let fraction = parts.next().expect("fraction");
        let seconds = parts.next().expect("seconds");

        assert!(parts.next().is_none());
        assert!(fraction.starts_with("0."));
        assert_eq!(fraction.len(), 8);
        assert!(seconds.parse::<u64>().is_ok());
    }
}
