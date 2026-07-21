//! Model A test suite: register cases during load, run after entry body.
//!
//! Enabled when the host sets `XO_TEST` in the process environment (AOT child
//! via `xo test`) or calls [`echo_runtime_test_enable`] (in-process JIT).
//! Registration is a no-op otherwise so co-located `test.it` calls do not run
//! under ordinary `xo run`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::{echo_runtime_fn_code, echo_runtime_fn_shape, string_data, FN_SHAPE_PLAIN};

/// Human-readable duration for suite / `xo test` lines.
fn fmt_dur(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms < 1000.0 {
        format!("{ms:.1}ms")
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}

struct Case {
    name: String,
    /// Native code pointer bits (from function value).
    code: i64,
    shape: i64,
}

struct Suite {
    cases: Vec<Case>,
    /// Failures in the case currently executing.
    case_failed: bool,
}

static SUITE: Mutex<Suite> = Mutex::new(Suite {
    cases: Vec::new(),
    case_failed: false,
});

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Turn on suite mode for this process (JIT / in-process hosts).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_test_enable() {
    ENABLED.store(true, Ordering::SeqCst);
}

fn suite_mode() -> bool {
    ENABLED.load(Ordering::SeqCst) || std::env::var_os("XO_TEST").is_some()
}

/// Register a named zero-arg function value as a test case.
///
/// No-op unless `XO_TEST` is set in the environment.
///
/// # Safety
/// `name` must be 0 or a string handle; `body` must be 0 or a function-value handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_runtime_test_register(name: i64, body: i64) {
    if !suite_mode() {
        return;
    }
    let label = string_data(name).unwrap_or_else(|| "<unnamed>".into());
    let code = unsafe { echo_runtime_fn_code(body) };
    let shape = unsafe { echo_runtime_fn_shape(body) };
    if code == 0 {
        eprintln!("xo test: skip register {label:?}: not a function value");
        return;
    }
    let mut g = SUITE.lock().expect("test suite lock");
    g.cases.push(Case {
        name: label,
        code,
        shape,
    });
}

/// Record a failure in the current case and print `msg` (string handle).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_test_fail(msg: i64) {
    if !suite_mode() {
        return;
    }
    {
        let mut g = SUITE.lock().expect("test suite lock");
        g.case_failed = true;
    }
    if let Some(s) = string_data(msg) {
        eprintln!("  fail: {s}");
    } else {
        eprintln!("  fail");
    }
}

/// Run all registered cases. Returns failure count, or `-1` when suite mode is off.
///
/// When suite mode is on and zero cases registered, prints a notice and returns 0.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_test_finish() -> i64 {
    if !suite_mode() {
        return -1;
    }

    let cases = {
        let mut g = SUITE.lock().expect("test suite lock");
        std::mem::take(&mut g.cases)
    };

    if cases.is_empty() {
        eprintln!("xo test: 0 cases registered");
        return 0;
    }

    let mut failed = 0i64;
    let total = cases.len();
    let suite_start = Instant::now();
    for case in &cases {
        {
            let mut g = SUITE.lock().expect("test suite lock");
            g.case_failed = false;
        }
        let case_start = Instant::now();
        // SAFETY: code bits come from fn values created by this process's LLVM.
        let call_ok = unsafe { invoke_case(case.code, case.shape) };
        let case_failed = SUITE.lock().expect("test suite lock").case_failed;
        let elapsed = fmt_dur(case_start.elapsed());
        if !call_ok || case_failed {
            failed += 1;
            eprintln!("FAIL  {} ({elapsed})", case.name);
        } else {
            eprintln!("ok    {} ({elapsed})", case.name);
        }
    }

    let passed = total as i64 - failed;
    let suite_elapsed = fmt_dur(suite_start.elapsed());
    eprintln!("xo test: {passed} passed, {failed} failed, {total} total ({suite_elapsed})");
    failed
}

/// Invoke a zero-arg function value. Returns false if the pointer was null.
///
/// # Safety
/// `code` must be a valid native function pointer for this process (or 0).
unsafe fn invoke_case(code: i64, shape: i64) -> bool {
    if code == 0 {
        return false;
    }
    if shape == FN_SHAPE_PLAIN {
        type Plain = unsafe extern "C" fn() -> i64;
        let f: Plain = unsafe { std::mem::transmute(code) };
        let _ = unsafe { f() };
        return true;
    }
    // Result / option shaped: ignore tag, just force the body to run.
    type Wide = unsafe extern "C" fn() -> i128;
    let f: Wide = unsafe { std::mem::transmute(code) };
    let _ = unsafe { f() };
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::echo_runtime_fn_new;

    #[test]
    fn register_noop_without_suite_mode() {
        ENABLED.store(false, Ordering::SeqCst);
        // SAFETY: nullish handles; register must no-op without suite mode.
        unsafe {
            echo_runtime_test_register(0, 0);
        }
        assert_eq!(echo_runtime_test_finish(), -1);
    }

    #[test]
    fn finish_zero_cases_when_enabled() {
        echo_runtime_test_enable();
        let _ = SUITE.lock().map(|mut g| {
            g.cases.clear();
            g.case_failed = false;
        });
        assert_eq!(echo_runtime_test_finish(), 0);
        ENABLED.store(false, Ordering::SeqCst);
    }

    #[test]
    fn fn_new_roundtrip_for_register() {
        extern "C" fn sample() -> i64 {
            42
        }
        let code = sample as *const () as i64;
        let handle = echo_runtime_fn_new(code, FN_SHAPE_PLAIN);
        let back = unsafe { echo_runtime_fn_code(handle) };
        assert_eq!(back, code);
    }
}
