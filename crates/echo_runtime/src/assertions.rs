use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{EchoValue, echo_values_equal};

static ASSERT_FAILURES: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn reset() {
    ASSERT_FAILURES.store(0, Ordering::Relaxed);
}

pub(crate) fn has_failures() -> bool {
    ASSERT_FAILURES.load(Ordering::Relaxed) > 0
}

fn record_assertion(passed: bool, message: &str) {
    if passed {
        return;
    }

    ASSERT_FAILURES.fetch_add(1, Ordering::Relaxed);
    eprintln!("{message}");
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_assert_ok(condition: EchoValue) -> EchoValue {
    let passed = condition.is_true_bool();
    record_assertion(passed, "assert.ok failed");
    EchoValue::bool(passed)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_assert_equals(actual: EchoValue, expected: EchoValue) -> EchoValue {
    let passed = echo_values_equal(actual, expected);
    record_assertion(passed, "assert.equals failed");
    EchoValue::bool(passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_ok_records_bool_success() {
        reset();

        assert_eq!(
            echo_std_assert_ok(EchoValue::bool(true)),
            EchoValue::bool(true)
        );
        assert!(!has_failures());
    }

    #[test]
    fn assert_equals_uses_runtime_value_equality() {
        reset();

        assert_eq!(
            echo_std_assert_equals(EchoValue::int(42), EchoValue::int(42)),
            EchoValue::bool(true)
        );
        assert!(!has_failures());
    }

    #[test]
    fn failed_assertions_are_reported() {
        reset();

        assert_eq!(
            echo_std_assert_equals(EchoValue::int(1), EchoValue::int(2)),
            EchoValue::bool(false)
        );
        assert!(has_failures());
    }
}
