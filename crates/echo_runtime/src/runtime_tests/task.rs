use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn task_defer_returns_task_value() {
    unsafe extern "C" fn callback() -> EchoValue {
        EchoValue::int(1)
    }

    let value = echo_task_defer(Some(callback));

    assert!(value.is_task());
    assert_ne!(value.payload, 0);
}

#[test]
fn task_run_starts_callback_before_join_collects_it() {
    static CALLBACK_RUNS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn callback() -> EchoValue {
        CALLBACK_RUNS.fetch_add(1, Ordering::Relaxed);
        EchoValue::int(42)
    }

    CALLBACK_RUNS.store(0, Ordering::Relaxed);
    let task = echo_task_defer(Some(callback));
    let task = echo_task_run(task);

    let deadline = Instant::now() + Duration::from_secs(1);
    while CALLBACK_RUNS.load(Ordering::Relaxed) == 0 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
    }

    assert_eq!(CALLBACK_RUNS.load(Ordering::Relaxed), 1);

    let result = echo_task_join(task);

    assert_eq!(CALLBACK_RUNS.load(Ordering::Relaxed), 1);
    assert_eq!(result, EchoValue::int(42));
}
