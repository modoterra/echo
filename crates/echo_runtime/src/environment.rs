use std::env;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::{
    EchoValue, echo_runtime_string, echo_value_array_append, echo_value_array_new,
    echo_value_array_set,
};

pub const PHP_COMPAT_VERSION: &str = "8.2.0";
pub const ZEND_COMPAT_VERSION: &str = "8.2.0";

static HTTP_RESPONSE_CODE: AtomicI64 = AtomicI64::new(0);
static IGNORE_USER_ABORT: AtomicI64 = AtomicI64::new(0);
static CLI_PROCESS_TITLE: Mutex<Option<Vec<u8>>> = Mutex::new(None);

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
pub extern "C" fn echo_php_sys_getloadavg() -> EchoValue {
    let Some(loads) = load_average_values() else {
        return EchoValue::bool(false);
    };

    let mut result = echo_value_array_new();
    for load in loads {
        result = echo_value_array_append(result, EchoValue::float(load));
    }
    result
}

#[cfg(target_os = "linux")]
fn load_average_values() -> Option<[f64; 3]> {
    let content = std::fs::read_to_string("/proc/loadavg").ok()?;
    let mut parts = content.split_whitespace();
    Some([
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ])
}

#[cfg(not(target_os = "linux"))]
fn load_average_values() -> Option<[f64; 3]> {
    None
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_cli_get_process_title() -> EchoValue {
    let Ok(title) = CLI_PROCESS_TITLE.lock() else {
        return EchoValue::null();
    };

    title
        .clone()
        .map(echo_runtime_string)
        .unwrap_or_else(EchoValue::null)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_cli_set_process_title(title: EchoValue) -> EchoValue {
    let Some(bytes) = title.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(mut stored_title) = CLI_PROCESS_TITLE.lock() else {
        return EchoValue::bool(false);
    };

    *stored_title = Some(bytes);
    EchoValue::bool(true)
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
pub extern "C" fn echo_php_php_uname(mode: EchoValue) -> EchoValue {
    let mode = mode
        .string_bytes()
        .and_then(|bytes| bytes.first().copied())
        .unwrap_or(b'a')
        .to_ascii_lowercase();

    let system = php_uname_system();
    let node = php_uname_node();
    let release = php_uname_file("/proc/sys/kernel/osrelease", "unknown");
    let version = php_uname_file("/proc/sys/kernel/version", "unknown");
    let machine = std::env::consts::ARCH.as_bytes().to_vec();

    let value = match mode {
        b's' => system,
        b'n' => node,
        b'r' => release,
        b'v' => version,
        b'm' => machine,
        _ => {
            let mut value = Vec::new();
            for (index, part) in [system, node, release, version, machine]
                .into_iter()
                .enumerate()
            {
                if index > 0 {
                    value.push(b' ');
                }
                value.extend(part);
            }
            value
        }
    };

    echo_runtime_string(value)
}

fn php_uname_system() -> Vec<u8> {
    if cfg!(target_os = "linux") {
        b"Linux".to_vec()
    } else if cfg!(target_os = "macos") {
        b"Darwin".to_vec()
    } else if cfg!(target_os = "windows") {
        b"Windows".to_vec()
    } else {
        std::env::consts::OS.as_bytes().to_vec()
    }
}

fn php_uname_node() -> Vec<u8> {
    env::var_os("HOSTNAME")
        .and_then(non_empty_os_string_bytes)
        .or_else(|| hostname_file_bytes(Path::new("/proc/sys/kernel/hostname")))
        .or_else(|| hostname_file_bytes(Path::new("/etc/hostname")))
        .unwrap_or_else(|| b"unknown".to_vec())
}

fn php_uname_file(path: &str, fallback: &str) -> Vec<u8> {
    std::fs::read(path)
        .ok()
        .map(|mut bytes| {
            while matches!(bytes.last(), Some(b'\n' | b'\r')) {
                bytes.pop();
            }
            bytes
        })
        .filter(|bytes| !bytes.is_empty())
        .unwrap_or_else(|| fallback.as_bytes().to_vec())
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
pub extern "C" fn echo_php_get_cfg_var(option: EchoValue) -> EchoValue {
    let Some(_option) = option.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_get(option: EchoValue) -> EchoValue {
    let Some(_option) = option.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_get_all(extension: EchoValue, _details: EchoValue) -> EchoValue {
    if extension.is_null() {
        return echo_value_array_new();
    }

    let Some(bytes) = extension.string_bytes() else {
        return EchoValue::bool(false);
    };

    if bytes.is_empty() {
        echo_value_array_new()
    } else {
        EchoValue::bool(false)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_parse_quantity(shorthand: EchoValue) -> EchoValue {
    let Some(bytes) = shorthand.string_bytes() else {
        return EchoValue::int(0);
    };

    EchoValue::int(parse_ini_quantity_bytes(&bytes))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_include_path() -> EchoValue {
    echo_php_ini_get(echo_runtime_string(b"include_path".to_vec()))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_connection_aborted() -> EchoValue {
    EchoValue::int(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_connection_status() -> EchoValue {
    EchoValue::int(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ignore_user_abort(enable: EchoValue) -> EchoValue {
    if enable.is_null() {
        return EchoValue::int(IGNORE_USER_ABORT.load(Ordering::SeqCst));
    }

    let previous = IGNORE_USER_ABORT.swap(
        enable.bool_value().unwrap_or(false) as i64,
        Ordering::SeqCst,
    );
    EchoValue::int(previous)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_headers_list() -> EchoValue {
    echo_value_array_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_headers_sent() -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_header(
    header: EchoValue,
    _replace: EchoValue,
    _response_code: EchoValue,
) {
    let Some(_header) = header.string_bytes() else {
        return;
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_header_remove(name: EchoValue) {
    if name.is_null() {
        return;
    }

    let Some(_name) = name.string_bytes() else {
        return;
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_http_response_code(response_code: EchoValue) -> EchoValue {
    if response_code.is_null() {
        let current = HTTP_RESPONSE_CODE.load(Ordering::SeqCst);

        if current == 0 {
            EchoValue::bool(false)
        } else {
            EchoValue::int(current)
        }
    } else if let Some(code) = response_code.int_value() {
        let previous = HTTP_RESPONSE_CODE.swap(code, Ordering::SeqCst);

        if previous == 0 {
            EchoValue::bool(true)
        } else {
            EchoValue::int(previous)
        }
    } else {
        EchoValue::bool(false)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_set(option: EchoValue, value: EchoValue) -> EchoValue {
    let Some(_option) = option.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Some(_value) = value.string_bytes() else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_alter(option: EchoValue, value: EchoValue) -> EchoValue {
    echo_php_ini_set(option, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_ini_restore(option: EchoValue) {
    let Some(_option) = option.string_bytes() else {
        return;
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_php_ini_loaded_file() -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_php_ini_scanned_files() -> EchoValue {
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

fn parse_ini_quantity_bytes(bytes: &[u8]) -> i64 {
    let bytes = trim_ascii_start(bytes);
    let Some((sign, after_sign)) = parse_ini_quantity_sign(bytes) else {
        return 0;
    };
    let Some((base, digits)) = parse_ini_quantity_base(after_sign) else {
        return 0;
    };

    let mut value = 0_i64;
    let mut consumed = 0;

    for &byte in digits {
        let Some(digit) = ascii_digit_value(byte) else {
            break;
        };
        if digit >= base {
            break;
        }

        value = value
            .saturating_mul(base as i64)
            .saturating_add(digit as i64);
        consumed += 1;
    }

    if consumed == 0 {
        return 0;
    }

    let mut value = if sign < 0 {
        value.saturating_neg()
    } else {
        value
    };

    if let Some(multiplier) = digits
        .get(consumed)
        .and_then(|byte| ini_quantity_multiplier(*byte))
    {
        value = value.saturating_mul(multiplier);
    }

    value
}

fn trim_ascii_start(mut bytes: &[u8]) -> &[u8] {
    while matches!(
        bytes.first(),
        Some(b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
    ) {
        bytes = &bytes[1..];
    }

    bytes
}

fn parse_ini_quantity_sign(bytes: &[u8]) -> Option<(i8, &[u8])> {
    match bytes.first() {
        Some(b'+') => Some((1, &bytes[1..])),
        Some(b'-') => Some((-1, &bytes[1..])),
        Some(_) => Some((1, bytes)),
        None => None,
    }
}

fn parse_ini_quantity_base(bytes: &[u8]) -> Option<(u8, &[u8])> {
    match bytes {
        [b'0', b'x' | b'X', rest @ ..] => Some((16, rest)),
        [b'0', b'b' | b'B', rest @ ..] => Some((2, rest)),
        [b'0', b'o' | b'O', rest @ ..] => Some((8, rest)),
        [b'0', rest @ ..] => Some((8, rest)),
        [_, ..] => Some((10, bytes)),
        [] => None,
    }
}

fn ascii_digit_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn ini_quantity_multiplier(byte: u8) -> Option<i64> {
    match byte {
        b'k' | b'K' => Some(1024),
        b'm' | b'M' => Some(1024 * 1024),
        b'g' | b'G' => Some(1024 * 1024 * 1024),
        _ => None,
    }
}
