use std::path::Path;

#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

use crate::{EchoValue, echo_runtime_string};

use super::path_buf_from_bytes;

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_readlink(path: EchoValue) -> EchoValue {
    match path.string_bytes() {
        Some(bytes) => path_readlink(&bytes)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_linkinfo(path: EchoValue) -> EchoValue {
    match path.string_bytes() {
        Some(bytes) => EchoValue::int(path_linkinfo(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_link(target: EchoValue, link: EchoValue) -> EchoValue {
    match (target.string_bytes(), link.string_bytes()) {
        (Some(target), Some(link)) => EchoValue::bool(path_link(&target, &link)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_symlink(target: EchoValue, link: EchoValue) -> EchoValue {
    match (target.string_bytes(), link.string_bytes()) {
        (Some(target), Some(link)) => EchoValue::bool(path_symlink(&target, &link)),
        _ => EchoValue::error(),
    }
}

#[cfg(unix)]
fn path_linkinfo(bytes: &[u8]) -> i64 {
    std::fs::symlink_metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .and_then(|metadata| i64::try_from(metadata.dev()).ok())
        .unwrap_or(-1)
}

#[cfg(not(unix))]
fn path_linkinfo(_bytes: &[u8]) -> i64 {
    -1
}

#[cfg(unix)]
fn path_readlink(bytes: &[u8]) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    std::fs::read_link(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|path| path.into_os_string().into_vec())
}

#[cfg(not(unix))]
fn path_readlink(bytes: &[u8]) -> Option<Vec<u8>> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::read_link(Path::new(path)).ok())
        .and_then(|path| path.into_os_string().into_string().ok())
        .map(String::into_bytes)
}

fn path_link(target: &[u8], link: &[u8]) -> bool {
    match (path_buf_from_bytes(target), path_buf_from_bytes(link)) {
        (Some(target), Some(link)) => std::fs::hard_link(target, link).is_ok(),
        _ => false,
    }
}

#[cfg(unix)]
fn path_symlink(target: &[u8], link: &[u8]) -> bool {
    std::os::unix::fs::symlink(OsStr::from_bytes(target), OsStr::from_bytes(link)).is_ok()
}

#[cfg(windows)]
fn path_symlink(target: &[u8], link: &[u8]) -> bool {
    match (path_buf_from_bytes(target), path_buf_from_bytes(link)) {
        (Some(target), Some(link)) => {
            if target.is_dir() {
                std::os::windows::fs::symlink_dir(target, link).is_ok()
            } else {
                std::os::windows::fs::symlink_file(target, link).is_ok()
            }
        }
        _ => false,
    }
}

#[cfg(all(not(unix), not(windows)))]
fn path_symlink(_target: &[u8], _link: &[u8]) -> bool {
    false
}
