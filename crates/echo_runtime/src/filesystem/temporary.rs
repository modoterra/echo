use std::env;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::time::unix_duration_now_or_zero;
use crate::{EchoValue, echo_runtime_string};

use super::path_buf_from_bytes;

static NEXT_UNIQID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_sys_get_temp_dir() -> EchoValue {
    path_bytes(env::temp_dir())
        .map(echo_runtime_string)
        .unwrap_or_else(|| echo_runtime_string(Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_tempnam(directory: EchoValue, prefix: EchoValue) -> EchoValue {
    match (directory.string_bytes(), prefix.string_bytes()) {
        (Some(directory), Some(prefix)) => path_tempnam(&directory, &prefix)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_uniqid(prefix: EchoValue, more_entropy: EchoValue) -> EchoValue {
    let Some(prefix) = prefix.string_bytes() else {
        return EchoValue::error();
    };
    let more_entropy = more_entropy.bool_value().unwrap_or(false);

    echo_runtime_string(php_uniqid(&prefix, more_entropy))
}

fn path_tempnam(directory: &[u8], prefix: &[u8]) -> Option<Vec<u8>> {
    let requested = path_buf_from_bytes(directory)?;
    let fallback = env::temp_dir();
    create_temp_file_in(&requested, prefix).or_else(|| create_temp_file_in(&fallback, prefix))
}

fn create_temp_file_in(directory: &Path, prefix: &[u8]) -> Option<Vec<u8>> {
    if !directory.is_dir() {
        return None;
    }

    let prefix = &prefix[..prefix.len().min(63)];
    for _ in 0..128 {
        let mut name = Vec::with_capacity(prefix.len() + 16);
        name.extend_from_slice(prefix);
        let unique = php_uniqid(b"", false);
        name.extend_from_slice(&unique);

        let mut path = directory.to_path_buf();
        push_path_component_from_bytes(&mut path, &name)?;
        if create_temp_file(&path) {
            return path_bytes(path);
        }
    }

    None
}

fn create_temp_file(path: &Path) -> bool {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    configure_temp_file_mode(&mut options);
    options.open(path).is_ok()
}

#[cfg(unix)]
fn configure_temp_file_mode(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.mode(0o600);
}

#[cfg(not(unix))]
fn configure_temp_file_mode(_options: &mut OpenOptions) {}

fn php_uniqid(prefix: &[u8], more_entropy: bool) -> Vec<u8> {
    let duration = unix_duration_now_or_zero();
    let seconds = duration.as_secs() as u32;
    let micros = duration.subsec_micros();
    let counter = NEXT_UNIQID_COUNTER.fetch_add(1, Ordering::Relaxed) as u32;
    let micros = (micros.wrapping_add(counter)) % 0x100000;

    let mut id = prefix.to_vec();
    id.extend_from_slice(format!("{seconds:08x}{micros:05x}").as_bytes());
    if more_entropy {
        let entropy = ((duration.subsec_nanos() as u64)
            ^ ((counter as u64).wrapping_mul(1_103_515_245)))
            % 1_000_000_000;
        id.extend_from_slice(format!(".{entropy:09}").as_bytes());
    }
    id
}

#[cfg(unix)]
fn path_bytes(path: PathBuf) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    Some(path.into_os_string().into_vec())
}

#[cfg(not(unix))]
fn path_bytes(path: PathBuf) -> Option<Vec<u8>> {
    path.into_os_string()
        .into_string()
        .ok()
        .map(String::into_bytes)
}

#[cfg(unix)]
fn push_path_component_from_bytes(path: &mut PathBuf, component: &[u8]) -> Option<()> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    path.push(OsStr::from_bytes(component));
    Some(())
}

#[cfg(not(unix))]
fn push_path_component_from_bytes(path: &mut PathBuf, component: &[u8]) -> Option<()> {
    path.push(std::str::from_utf8(component).ok()?);
    Some(())
}
