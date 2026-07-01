use std::fs::OpenOptions;

use filetime::FileTime;

use crate::EchoValue;
use crate::time::unix_duration_now_or_zero;

use super::path_buf_from_bytes;

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_touch(
    filename: EchoValue,
    mtime: EchoValue,
    atime: EchoValue,
) -> EchoValue {
    let Some(bytes) = filename.string_bytes() else {
        return EchoValue::error();
    };
    let mtime = if mtime.is_null() {
        None
    } else {
        mtime.php_int_value()
    };
    let atime = if atime.is_null() {
        None
    } else {
        atime.php_int_value()
    };

    EchoValue::bool(path_touch(&bytes, mtime, atime))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_copy(from: EchoValue, to: EchoValue, _context: EchoValue) -> EchoValue {
    match (from.string_bytes(), to.string_bytes()) {
        (Some(from), Some(to)) => EchoValue::bool(path_copy(&from, &to)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rename(
    from: EchoValue,
    to: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    match (from.string_bytes(), to.string_bytes()) {
        (Some(from), Some(to)) => EchoValue::bool(path_rename(&from, &to)),
        _ => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_unlink(filename: EchoValue, _context: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_unlink(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_chmod(filename: EchoValue, permissions: EchoValue) -> EchoValue {
    let Some(bytes) = filename.string_bytes() else {
        return EchoValue::error();
    };
    let Some(permissions) = permissions.php_int_value() else {
        return EchoValue::error();
    };

    EchoValue::bool(path_chmod(&bytes, permissions))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_mkdir(
    directory: EchoValue,
    permissions: EchoValue,
    recursive: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(bytes) = directory.string_bytes() else {
        return EchoValue::error();
    };
    let permissions = permissions.php_int_value().unwrap_or(0o777);
    let recursive = recursive.bool_value().unwrap_or(false);

    EchoValue::bool(path_mkdir(&bytes, permissions, recursive))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_rmdir(directory: EchoValue, _context: EchoValue) -> EchoValue {
    match directory.string_bytes() {
        Some(bytes) => EchoValue::bool(path_rmdir(&bytes)),
        None => EchoValue::error(),
    }
}

fn path_touch(bytes: &[u8], mtime: Option<i64>, atime: Option<i64>) -> bool {
    let Some(path) = path_buf_from_bytes(bytes) else {
        return false;
    };

    if OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .is_err()
    {
        return false;
    }

    let now = i64::try_from(unix_duration_now_or_zero().as_secs()).unwrap_or(0);
    let modified = mtime.unwrap_or(now);
    let accessed = atime.unwrap_or(modified);

    filetime::set_file_times(
        &path,
        FileTime::from_unix_time(accessed, 0),
        FileTime::from_unix_time(modified, 0),
    )
    .is_ok()
}

fn path_copy(from: &[u8], to: &[u8]) -> bool {
    match (path_buf_from_bytes(from), path_buf_from_bytes(to)) {
        (Some(from), Some(to)) => std::fs::copy(from, to).is_ok(),
        _ => false,
    }
}

fn path_rename(from: &[u8], to: &[u8]) -> bool {
    match (path_buf_from_bytes(from), path_buf_from_bytes(to)) {
        (Some(from), Some(to)) => std::fs::rename(from, to).is_ok(),
        _ => false,
    }
}

fn path_unlink(bytes: &[u8]) -> bool {
    path_buf_from_bytes(bytes)
        .map(std::fs::remove_file)
        .is_some_and(|result| result.is_ok())
}

#[cfg(unix)]
fn path_chmod(bytes: &[u8], permissions: i64) -> bool {
    use std::os::unix::fs::PermissionsExt;

    let Some(path) = path_buf_from_bytes(bytes) else {
        return false;
    };

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(permissions as u32)).is_ok()
}

#[cfg(not(unix))]
fn path_chmod(_bytes: &[u8], _permissions: i64) -> bool {
    false
}

fn path_mkdir(bytes: &[u8], permissions: i64, recursive: bool) -> bool {
    let Some(path) = path_buf_from_bytes(bytes) else {
        return false;
    };
    if path.exists() {
        return false;
    }

    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(recursive);
    configure_dir_builder_mode(&mut builder, permissions);
    builder.create(path).is_ok()
}

#[cfg(unix)]
fn configure_dir_builder_mode(builder: &mut std::fs::DirBuilder, permissions: i64) {
    use std::os::unix::fs::DirBuilderExt;

    builder.mode(permissions as u32);
}

#[cfg(not(unix))]
fn configure_dir_builder_mode(_builder: &mut std::fs::DirBuilder, _permissions: i64) {}

fn path_rmdir(bytes: &[u8]) -> bool {
    path_buf_from_bytes(bytes)
        .map(std::fs::remove_dir)
        .is_some_and(|result| result.is_ok())
}
