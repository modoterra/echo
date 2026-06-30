use std::env;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::{EchoValue, echo_runtime_string, echo_value_array_new, echo_value_array_set};

pub const PHP_COMPAT_VERSION: &str = "8.2.0";
pub const ZEND_COMPAT_VERSION: &str = "8.2.0";

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getenv(name: EchoValue, _local_only: EchoValue) -> EchoValue {
    if name.is_null() {
        let mut result = echo_value_array_new();

        for (key, value) in env::vars_os() {
            result = echo_value_array_set(
                result,
                echo_runtime_string(os_string_bytes(&key)),
                echo_runtime_string(os_string_bytes(&value)),
            );
        }

        return result;
    }

    let Some(bytes) = name.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(key) = String::from_utf8(bytes) else {
        return EchoValue::bool(false);
    };

    env::var_os(key)
        .map(|value| echo_runtime_string(os_string_bytes(&value)))
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gethostname() -> EchoValue {
    env::var_os("HOSTNAME")
        .and_then(non_empty_os_string_bytes)
        .or_else(|| hostname_file_bytes(Path::new("/proc/sys/kernel/hostname")))
        .or_else(|| hostname_file_bytes(Path::new("/etc/hostname")))
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getmypid() -> EchoValue {
    EchoValue::int(std::process::id() as i64)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_phpversion(extension: EchoValue) -> EchoValue {
    if extension.is_null() {
        return echo_runtime_string(PHP_COMPAT_VERSION.as_bytes().to_vec());
    }

    let Some(_extension) = extension.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_php_sapi_name() -> EchoValue {
    echo_runtime_string(b"cli".to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_zend_version() -> EchoValue {
    echo_runtime_string(ZEND_COMPAT_VERSION.as_bytes().to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_extension_loaded(extension: EchoValue) -> EchoValue {
    let Some(_extension) = extension.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_loaded_extensions(_zend_extensions: EchoValue) -> EchoValue {
    echo_value_array_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_extension_funcs(extension: EchoValue) -> EchoValue {
    let Some(_extension) = extension.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_putenv(assignment: EchoValue) -> EchoValue {
    let Some(bytes) = assignment.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(assignment) = String::from_utf8(bytes) else {
        return EchoValue::bool(false);
    };

    if let Some((name, value)) = assignment.split_once('=') {
        if name.is_empty() {
            return EchoValue::bool(false);
        }

        unsafe {
            env::set_var(name, value);
        }
    } else {
        if assignment.is_empty() {
            return EchoValue::bool(false);
        }

        unsafe {
            env::remove_var(assignment);
        }
    }

    EchoValue::bool(true)
}

#[cfg(unix)]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    value.as_bytes().to_vec()
}

#[cfg(not(unix))]
fn os_string_bytes(value: &OsStr) -> Vec<u8> {
    value.to_string_lossy().as_bytes().to_vec()
}

fn non_empty_os_string_bytes(value: std::ffi::OsString) -> Option<Vec<u8>> {
    let bytes = os_string_bytes(&value);

    if bytes.is_empty() { None } else { Some(bytes) }
}

fn hostname_file_bytes(path: &Path) -> Option<Vec<u8>> {
    let mut bytes = std::fs::read(path).ok()?;

    while matches!(bytes.last(), Some(b'\n' | b'\r')) {
        bytes.pop();
    }

    if bytes.is_empty() { None } else { Some(bytes) }
}
