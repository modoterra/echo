use crate::{EchoValue, echo_runtime_string};
#[cfg(unix)]
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

mod content;
mod links;
mod metadata;
mod mutation;
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
pub use mutation::{
    echo_php_copy, echo_php_mkdir, echo_php_rename, echo_php_rmdir, echo_php_touch, echo_php_unlink,
};
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
pub extern "C" fn echo_php_realpath(path: EchoValue) -> EchoValue {
    match path.string_bytes() {
        Some(bytes) => path_realpath(&bytes)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
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
