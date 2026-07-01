use std::sync::atomic::{AtomicBool, Ordering};

use crate::{EchoValue, echo_runtime_string, echo_value_array_new, echo_value_array_set};

static GC_ENABLED: AtomicBool = AtomicBool::new(true);

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gc_enabled() -> EchoValue {
    EchoValue::bool(GC_ENABLED.load(Ordering::Relaxed))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gc_disable() {
    GC_ENABLED.store(false, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gc_enable() {
    GC_ENABLED.store(true, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gc_collect_cycles() -> EchoValue {
    EchoValue::int(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gc_mem_caches() -> EchoValue {
    EchoValue::int(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gc_status() -> EchoValue {
    let mut status = echo_value_array_new();
    status = set_status_bool(status, "running", false);
    status = set_status_bool(status, "protected", false);
    status = set_status_bool(status, "full", false);
    status = set_status_int(status, "runs", 0);
    status = set_status_int(status, "collected", 0);
    status = set_status_int(status, "threshold", 0);
    status = set_status_int(status, "buffer_size", 0);
    status = set_status_int(status, "roots", 0);
    status = set_status_int(status, "application_time", 0);
    status = set_status_int(status, "collector_time", 0);
    status = set_status_int(status, "destructor_time", 0);
    set_status_int(status, "free_time", 0)
}

fn set_status_bool(status: EchoValue, key: &str, value: bool) -> EchoValue {
    echo_value_array_set(
        status,
        echo_runtime_string(key.as_bytes().to_vec()),
        EchoValue::bool(value),
    )
}

fn set_status_int(status: EchoValue, key: &str, value: i64) -> EchoValue {
    echo_value_array_set(
        status,
        echo_runtime_string(key.as_bytes().to_vec()),
        EchoValue::int(value),
    )
}
