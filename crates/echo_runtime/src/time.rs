use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
}
