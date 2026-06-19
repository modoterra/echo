use std::time::Duration;

pub fn sleep(millis: i64) {
    if millis > 0 {
        std::thread::sleep(Duration::from_millis(millis as u64));
    }
}
