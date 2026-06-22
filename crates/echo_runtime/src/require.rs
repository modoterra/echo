use std::collections::HashSet;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::EchoValue;

static REQUIRED_ONCE_FILES: OnceLock<Mutex<HashSet<Vec<u8>>>> = OnceLock::new();

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_require(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => require_path(&bytes),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_require_once(filename: EchoValue) -> EchoValue {
    let Some(bytes) = filename.string_bytes() else {
        return EchoValue::error();
    };

    let key = canonical_require_key(&bytes);
    let files = REQUIRED_ONCE_FILES.get_or_init(|| Mutex::new(HashSet::new()));
    {
        let mut files = files.lock().expect("require_once set poisoned");
        if files.contains(&key) {
            return EchoValue::bool(true);
        }
        files.insert(key);
    }

    require_path(&bytes)
}

#[cfg(unix)]
fn require_path(bytes: &[u8]) -> EchoValue {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = Path::new(OsStr::from_bytes(bytes));
    if path.exists() {
        EchoValue::bool(true)
    } else {
        eprintln!(
            "PHP Fatal error: Failed opening required '{}'",
            String::from_utf8_lossy(bytes)
        );
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn canonical_require_key(bytes: &[u8]) -> Vec<u8> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = Path::new(OsStr::from_bytes(bytes));
    std::fs::canonicalize(path)
        .ok()
        .map(|path| path.as_os_str().as_bytes().to_vec())
        .unwrap_or_else(|| bytes.to_vec())
}

#[cfg(not(unix))]
fn require_path(bytes: &[u8]) -> EchoValue {
    let Ok(path) = std::str::from_utf8(bytes) else {
        return EchoValue::error();
    };
    if Path::new(path).exists() {
        EchoValue::bool(true)
    } else {
        eprintln!("PHP Fatal error: Failed opening required '{path}'");
        std::process::exit(1);
    }
}

#[cfg(not(unix))]
fn canonical_require_key(bytes: &[u8]) -> Vec<u8> {
    let Ok(path) = std::str::from_utf8(bytes) else {
        return bytes.to_vec();
    };
    std::fs::canonicalize(path)
        .ok()
        .and_then(|path| path.into_os_string().into_string().ok())
        .map(String::into_bytes)
        .unwrap_or_else(|| bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EchoString;

    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes.to_vec()))))
    }

    #[test]
    fn require_accepts_existing_file() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

        assert_eq!(
            echo_php_require(string_value(path.to_string_lossy().as_bytes())),
            EchoValue::bool(true)
        );
    }

    #[test]
    fn require_once_returns_true_for_repeated_existing_file() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let path = path.to_string_lossy();

        assert_eq!(
            echo_php_require_once(string_value(path.as_bytes())),
            EchoValue::bool(true)
        );
        assert_eq!(
            echo_php_require_once(string_value(path.as_bytes())),
            EchoValue::bool(true)
        );
    }
}
