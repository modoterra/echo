//! Process args, environment, exit, and spawn+wait (`std/process`).

use std::process::Command;

use crate::{
    echo_runtime_list_new, echo_runtime_list_push, string_data, string_to_handle, EchoList,
    HEAP_MAGIC, KIND_LIST,
};

/// Current process arguments as a list of string handles (`argv[0]` is the program).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_process_args() -> i64 {
    let list = echo_runtime_list_new();
    for a in std::env::args() {
        let h = string_to_handle(a);
        unsafe {
            echo_runtime_list_push(list, h);
        }
    }
    list
}

/// 1 if environment variable `name` is set, else 0. Non-string name → 0.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_process_env_has(name: i64) -> i64 {
    let Some(key) = string_data(name) else {
        return 0;
    };
    if std::env::var_os(&key).is_some() {
        1
    } else {
        0
    }
}

/// Value of environment variable `name`, or empty string if unset / bad name.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_process_env_get(name: i64) -> i64 {
    let Some(key) = string_data(name) else {
        return string_to_handle(String::new());
    };
    match std::env::var(&key) {
        Ok(v) => string_to_handle(v),
        Err(_) => string_to_handle(String::new()),
    }
}

/// Set environment variable. No-op if name/value are not strings.
///
/// # Safety
/// Uses `std::env::set_var` (not thread-safe vs concurrent env reads in other threads).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_process_env_set(name: i64, value: i64) {
    let Some(key) = string_data(name) else {
        return;
    };
    let Some(val) = string_data(value) else {
        return;
    };
    // SAFETY: Echo single-threaded runtime default; std documents this as unsafe in 2024.
    unsafe {
        std::env::set_var(key, val);
    }
}

/// Unset environment variable. No-op if name is not a string.
///
/// # Safety
/// Uses `std::env::remove_var` (not thread-safe vs concurrent env reads).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_process_env_unset(name: i64) {
    let Some(key) = string_data(name) else {
        return;
    };
    unsafe {
        std::env::remove_var(key);
    }
}

/// Terminate the current process with `code` (truncated to process exit status).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_process_exit(code: i64) {
    let status = if code < 0 {
        255
    } else if code > 255 {
        (code & 0xff) as i32
    } else {
        code as i32
    };
    std::process::exit(status);
}

/// Spawn `program` with string args from `args` list, wait, return exit status.
///
/// Returns:
/// - `0..` — process exit code (or 128+signal convention is left as OS-reported wait code bits)
/// - `-1` — spawn failed or bad handles
///
/// `args` is a list of string handles (may be empty). Does not include the program name unless
/// the caller puts it there; `program` is the executable path/name only.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_process_run(program: i64, args: i64) -> i64 {
    let Some(prog) = string_data(program) else {
        return -1;
    };
    if prog.is_empty() {
        return -1;
    }
    let mut cmd = Command::new(&prog);
    if args != 0 {
        if let Some(list) = list_string_args(args) {
            for a in list {
                cmd.arg(a);
            }
        } else {
            return -1;
        }
    }
    match cmd.status() {
        Ok(st) => st.code().unwrap_or(1) as i64,
        Err(_) => -1,
    }
}

fn list_string_args(list: i64) -> Option<Vec<String>> {
    let h = unsafe { crate::header_at(list)? };
    if unsafe { (*h).magic } != HEAP_MAGIC || unsafe { (*h).kind } != KIND_LIST {
        return None;
    }
    let lst = unsafe { &*(list as *const EchoList) };
    let mut out = Vec::with_capacity(lst.elems.len());
    for &el in &lst.elems {
        out.push(string_data(el)?);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::echo_runtime_string_from_utf8;

    fn s(text: &str) -> i64 {
        unsafe { echo_runtime_string_from_utf8(text.as_ptr(), text.len()) }
    }

    #[test]
    fn args_nonempty() {
        let list = echo_runtime_process_args();
        let n = unsafe { crate::echo_runtime_list_len(list) };
        assert!(n >= 1, "expected at least argv0, got {n}");
        let a0 = unsafe { crate::echo_runtime_list_get(list, 0) };
        let text = string_data(a0).expect("argv0 string");
        assert!(!text.is_empty());
    }

    #[test]
    fn env_roundtrip() {
        let key = s("ECHO_PROCESS_TEST_KEY");
        echo_runtime_process_env_unset(key);
        assert_eq!(echo_runtime_process_env_has(key), 0);
        let empty = echo_runtime_process_env_get(key);
        assert_eq!(string_data(empty).as_deref(), Some(""));

        echo_runtime_process_env_set(key, s("hello"));
        assert_eq!(echo_runtime_process_env_has(key), 1);
        let v = echo_runtime_process_env_get(key);
        assert_eq!(string_data(v).as_deref(), Some("hello"));

        echo_runtime_process_env_unset(key);
        assert_eq!(echo_runtime_process_env_has(key), 0);
    }

    #[test]
    fn run_true_or_echo() {
        // Portable enough: `true` on Unix; on Windows try `cmd /C exit 0` via run of cmd.
        #[cfg(unix)]
        {
            let code = echo_runtime_process_run(s("true"), echo_runtime_list_new());
            assert_eq!(code, 0, "true should exit 0, got {code}");
            let args = echo_runtime_list_new();
            unsafe {
                echo_runtime_list_push(args, s("not-a-real-command-xyz-echo-test"));
            }
            // false exits 1
            let code = echo_runtime_process_run(s("false"), echo_runtime_list_new());
            assert_eq!(code, 1);
        }
        #[cfg(windows)]
        {
            let args = echo_runtime_list_new();
            unsafe {
                echo_runtime_list_push(args, s("/C"));
                echo_runtime_list_push(args, s("exit 0"));
            }
            let code = echo_runtime_process_run(s("cmd"), args);
            assert_eq!(code, 0, "cmd /C exit 0, got {code}");
        }
    }

    #[test]
    fn run_missing_program() {
        let code = echo_runtime_process_run(s(""), echo_runtime_list_new());
        assert_eq!(code, -1);
        let code = echo_runtime_process_run(
            s("__echo_definitely_missing_binary_xyz__"),
            echo_runtime_list_new(),
        );
        assert_eq!(code, -1);
    }
}
