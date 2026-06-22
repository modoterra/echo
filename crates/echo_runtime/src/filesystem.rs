use crate::{EchoValue, echo_runtime_string};
use filetime::FileTime;
#[cfg(unix)]
use std::ffi::OsStr;
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use crate::time::unix_duration_now_or_zero;

mod content;
mod links;
mod metadata;
mod temporary;

#[cfg(test)]
pub(crate) use content::PHP_FILE_APPEND;
pub use content::{echo_php_file_get_contents, echo_php_file_put_contents, echo_php_readfile};
pub use links::{echo_php_link, echo_php_readlink, echo_php_symlink};
pub use metadata::{
    echo_php_fileatime, echo_php_filectime, echo_php_filegroup, echo_php_fileinode,
    echo_php_filemtime, echo_php_fileowner, echo_php_fileperms, echo_php_filesize,
    echo_php_filetype, echo_php_is_dir, echo_php_is_executable, echo_php_is_file, echo_php_is_link,
    echo_php_is_readable, echo_php_is_writable,
};
pub(crate) use metadata::{path_chdir, path_exists, path_getcwd};
#[cfg(test)]
pub(crate) use metadata::{path_is_dir, path_is_file};
pub use temporary::{echo_php_sys_get_temp_dir, echo_php_tempnam, echo_php_uniqid};

#[cfg(unix)]
pub(crate) fn path_buf_from_bytes(bytes: &[u8]) -> Option<PathBuf> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    Some(PathBuf::from(OsStr::from_bytes(bytes)))
}

#[cfg(not(unix))]
pub(crate) fn path_buf_from_bytes(bytes: &[u8]) -> Option<PathBuf> {
    std::str::from_utf8(bytes).ok().map(PathBuf::from)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_file_exists(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(path_exists(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_chdir(directory: EchoValue) -> EchoValue {
    match directory.string_bytes() {
        Some(bytes) => EchoValue::bool(path_chdir(&bytes)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_getcwd() -> EchoValue {
    path_getcwd()
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_realpath(path: EchoValue) -> EchoValue {
    match path.string_bytes() {
        Some(bytes) => path_realpath(&bytes)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
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

#[cfg(unix)]
pub(crate) fn path_realpath(bytes: &[u8]) -> Option<Vec<u8>> {
    let path = if bytes.is_empty() {
        Path::new(".")
    } else {
        Path::new(OsStr::from_bytes(bytes))
    };
    std::fs::canonicalize(path)
        .ok()
        .map(|path| path.as_os_str().as_bytes().to_vec())
}

#[cfg(not(unix))]
pub(crate) fn path_realpath(bytes: &[u8]) -> Option<Vec<u8>> {
    let path = if bytes.is_empty() {
        "."
    } else {
        std::str::from_utf8(bytes).ok()?
    };
    std::fs::canonicalize(path)
        .ok()
        .and_then(|path| path.into_os_string().into_string().ok())
        .map(String::into_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_basename(path: EchoValue, suffix: EchoValue) -> EchoValue {
    let Some(path) = path.string_bytes() else {
        return EchoValue::error();
    };
    let Some(suffix) = suffix.string_bytes() else {
        return EchoValue::error();
    };

    echo_runtime_string(php_basename(&path, &suffix))
}

fn php_basename(path: &[u8], suffix: &[u8]) -> Vec<u8> {
    let trimmed_end = path
        .iter()
        .rposition(|byte| *byte != b'/')
        .map_or(0, |position| position + 1);
    let path = &path[..trimmed_end];
    let start = path
        .iter()
        .rposition(|byte| *byte == b'/')
        .map_or(0, |position| position + 1);
    let mut basename = path[start..].to_vec();

    if !suffix.is_empty() && basename.ends_with(suffix) {
        basename.truncate(basename.len() - suffix.len());
    }

    basename
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_dirname(path: EchoValue, levels: EchoValue) -> EchoValue {
    let Some(path) = path.string_bytes() else {
        return EchoValue::error();
    };
    let Some(levels) = levels.php_int_value() else {
        return EchoValue::error();
    };
    if levels <= 0 {
        return EchoValue::error();
    }

    let mut dirname = path;
    for _ in 0..levels {
        dirname = php_dirname_once(&dirname);
    }

    echo_runtime_string(dirname)
}

fn php_dirname_once(path: &[u8]) -> Vec<u8> {
    let Some(last_non_slash) = path.iter().rposition(|byte| *byte != b'/') else {
        return b"/".to_vec();
    };
    let path = &path[..=last_non_slash];
    let Some(last_slash) = path.iter().rposition(|byte| *byte == b'/') else {
        return b".".to_vec();
    };
    if last_slash == 0 {
        return b"/".to_vec();
    }

    let parent = &path[..last_slash];
    let parent_end = parent
        .iter()
        .rposition(|byte| *byte != b'/')
        .map_or(0, |position| position + 1);
    if parent_end == 0 {
        b"/".to_vec()
    } else {
        parent[..parent_end].to_vec()
    }
}
