use std::time::Duration;

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
