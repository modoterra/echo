use crate::{EchoValue, echo_runtime_string, write_runtime_output};
use filetime::FileTime;
use std::env;
#[cfg(unix)]
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::time::unix_duration_now_or_zero;

static NEXT_UNIQID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[cfg(unix)]
pub(crate) fn path_bytes(path: PathBuf) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    Some(path.into_os_string().into_vec())
}

#[cfg(not(unix))]
pub(crate) fn path_bytes(path: PathBuf) -> Option<Vec<u8>> {
    path.into_os_string()
        .into_string()
        .ok()
        .map(String::into_bytes)
}

#[cfg(unix)]
pub(crate) fn push_path_component_from_bytes(path: &mut PathBuf, component: &[u8]) -> Option<()> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    path.push(OsStr::from_bytes(component));
    Some(())
}

#[cfg(not(unix))]
pub(crate) fn push_path_component_from_bytes(path: &mut PathBuf, component: &[u8]) -> Option<()> {
    path.push(std::str::from_utf8(component).ok()?);
    Some(())
}

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
pub extern "C" fn echo_php_is_dir(filename: EchoValue) -> EchoValue {
    path_bool_builtin(filename, path_is_dir)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_file(filename: EchoValue) -> EchoValue {
    path_bool_builtin(filename, path_is_file)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_link(filename: EchoValue) -> EchoValue {
    path_bool_builtin(filename, path_is_link)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_readable(filename: EchoValue) -> EchoValue {
    path_bool_builtin(filename, path_is_readable)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_writable(filename: EchoValue) -> EchoValue {
    path_bool_builtin(filename, path_is_writable)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_executable(filename: EchoValue) -> EchoValue {
    path_bool_builtin(filename, path_is_executable)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filesize(filename: EchoValue) -> EchoValue {
    path_u64_builtin(filename, path_filesize)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fileatime(filename: EchoValue) -> EchoValue {
    path_i64_builtin(filename, path_fileatime)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filectime(filename: EchoValue) -> EchoValue {
    path_i64_builtin(filename, path_filectime)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filemtime(filename: EchoValue) -> EchoValue {
    path_i64_builtin(filename, path_filemtime)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fileinode(filename: EchoValue) -> EchoValue {
    path_i64_builtin(filename, path_fileinode)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fileowner(filename: EchoValue) -> EchoValue {
    path_i64_builtin(filename, path_fileowner)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filegroup(filename: EchoValue) -> EchoValue {
    path_i64_builtin(filename, path_filegroup)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_fileperms(filename: EchoValue) -> EchoValue {
    path_i64_builtin(filename, path_fileperms)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_filetype(filename: EchoValue) -> EchoValue {
    path_bytes_builtin(filename, path_filetype)
}

fn path_bool_builtin(filename: EchoValue, f: impl FnOnce(&[u8]) -> bool) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => EchoValue::bool(f(&bytes)),
        None => EchoValue::error(),
    }
}

fn path_i64_builtin(filename: EchoValue, f: impl FnOnce(&[u8]) -> Option<i64>) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => f(&bytes)
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

fn path_u64_builtin(filename: EchoValue, f: impl FnOnce(&[u8]) -> Option<u64>) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => f(&bytes)
            .and_then(|value| i64::try_from(value).ok())
            .map(EchoValue::int)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

fn path_bytes_builtin(filename: EchoValue, f: impl FnOnce(&[u8]) -> Option<Vec<u8>>) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => f(&bytes)
            .map(echo_runtime_string)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[cfg(unix)]
fn path_exists(bytes: &[u8]) -> bool {
    Path::new(OsStr::from_bytes(bytes)).exists()
}

#[cfg(unix)]
fn path_chdir(bytes: &[u8]) -> bool {
    env::set_current_dir(Path::new(OsStr::from_bytes(bytes))).is_ok()
}

#[cfg(unix)]
pub(crate) fn path_getcwd() -> Option<Vec<u8>> {
    env::current_dir()
        .ok()
        .map(|path| path.into_os_string().as_bytes().to_vec())
}

#[cfg(not(unix))]
fn path_exists(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(|path| Path::new(path).exists())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn path_chdir(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(Path::new)
        .map(|path| env::set_current_dir(path).is_ok())
        .unwrap_or(false)
}

#[cfg(not(unix))]
pub(crate) fn path_getcwd() -> Option<Vec<u8>> {
    env::current_dir()
        .ok()
        .map(|path| path.to_string_lossy().as_bytes().to_vec())
}

#[cfg(unix)]
pub(crate) fn path_is_dir(bytes: &[u8]) -> bool {
    Path::new(OsStr::from_bytes(bytes)).is_dir()
}

#[cfg(not(unix))]
pub(crate) fn path_is_dir(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(|path| Path::new(path).is_dir())
        .unwrap_or(false)
}

#[cfg(unix)]
pub(crate) fn path_is_file(bytes: &[u8]) -> bool {
    Path::new(OsStr::from_bytes(bytes)).is_file()
}

#[cfg(not(unix))]
pub(crate) fn path_is_file(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(|path| Path::new(path).is_file())
        .unwrap_or(false)
}

#[cfg(unix)]
fn path_is_link(bytes: &[u8]) -> bool {
    std::fs::symlink_metadata(Path::new(OsStr::from_bytes(bytes)))
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn path_is_link(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::symlink_metadata(Path::new(path)).ok())
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(unix)]
fn path_is_readable(bytes: &[u8]) -> bool {
    let path = Path::new(OsStr::from_bytes(bytes));
    if path.is_dir() {
        return std::fs::read_dir(path).is_ok();
    }
    std::fs::File::open(path).is_ok()
}

#[cfg(not(unix))]
fn path_is_readable(bytes: &[u8]) -> bool {
    let Ok(path) = std::str::from_utf8(bytes) else {
        return false;
    };
    let path = Path::new(path);
    if path.is_dir() {
        return std::fs::read_dir(path).is_ok();
    }
    std::fs::File::open(path).is_ok()
}

#[cfg(unix)]
fn path_is_writable(bytes: &[u8]) -> bool {
    let path = Path::new(OsStr::from_bytes(bytes));
    path_is_writable_path(path)
}

#[cfg(not(unix))]
fn path_is_writable(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(Path::new)
        .map(path_is_writable_path)
        .unwrap_or(false)
}

fn path_is_writable_path(path: &Path) -> bool {
    if path.is_dir() {
        let nanos = unix_duration_now_or_zero().as_nanos();
        let probe = path.join(format!(".echo_writable_probe_{nanos}"));
        return OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)
            .map(|_| {
                let _ = std::fs::remove_file(&probe);
                true
            })
            .unwrap_or(false);
    }

    OpenOptions::new().append(true).open(path).is_ok()
}

#[cfg(unix)]
fn path_is_executable(bytes: &[u8]) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn path_is_executable(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .ok()
        .map(Path::new)
        .filter(|path| path.is_file())
        .and_then(|path| path.extension())
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "exe" | "bat" | "cmd" | "com"
            )
        })
        .unwrap_or(false)
}

#[cfg(unix)]
fn path_filesize(bytes: &[u8]) -> Option<u64> {
    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.len())
}

#[cfg(not(unix))]
fn path_filesize(bytes: &[u8]) -> Option<u64> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::metadata(Path::new(path)).ok())
        .map(|metadata| metadata.len())
}

#[cfg(unix)]
fn path_fileatime(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.atime())
}

#[cfg(not(unix))]
fn path_fileatime(bytes: &[u8]) -> Option<i64> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::metadata(Path::new(path)).ok())
        .and_then(|metadata| metadata.accessed().ok())
        .and_then(crate::time::system_time_unix_timestamp)
}

#[cfg(unix)]
fn path_filectime(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.ctime())
}

#[cfg(not(unix))]
fn path_filectime(bytes: &[u8]) -> Option<i64> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::metadata(Path::new(path)).ok())
        .and_then(|metadata| metadata.created().ok())
        .and_then(crate::time::system_time_unix_timestamp)
}

#[cfg(unix)]
fn path_filemtime(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.mtime())
}

#[cfg(not(unix))]
fn path_filemtime(bytes: &[u8]) -> Option<i64> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|path| std::fs::metadata(Path::new(path)).ok())
        .and_then(|metadata| metadata.modified().ok())
        .and_then(crate::time::system_time_unix_timestamp)
}

#[cfg(unix)]
fn path_fileinode(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .and_then(|metadata| i64::try_from(metadata.ino()).ok())
}

#[cfg(not(unix))]
fn path_fileinode(_bytes: &[u8]) -> Option<i64> {
    None
}

#[cfg(unix)]
fn path_fileowner(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.uid() as i64)
}

#[cfg(not(unix))]
fn path_fileowner(_bytes: &[u8]) -> Option<i64> {
    None
}

#[cfg(unix)]
fn path_filegroup(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.gid() as i64)
}

#[cfg(not(unix))]
fn path_filegroup(_bytes: &[u8]) -> Option<i64> {
    None
}

#[cfg(unix)]
fn path_fileperms(bytes: &[u8]) -> Option<i64> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()
        .map(|metadata| metadata.mode() as i64)
}

#[cfg(not(unix))]
fn path_fileperms(_bytes: &[u8]) -> Option<i64> {
    None
}

#[cfg(unix)]
fn path_filetype(bytes: &[u8]) -> Option<Vec<u8>> {
    use std::os::unix::fs::FileTypeExt;

    let file_type = std::fs::symlink_metadata(Path::new(OsStr::from_bytes(bytes)))
        .ok()?
        .file_type();
    let name = if file_type.is_symlink() {
        "link"
    } else if file_type.is_dir() {
        "dir"
    } else if file_type.is_file() {
        "file"
    } else if file_type.is_fifo() {
        "fifo"
    } else if file_type.is_char_device() {
        "char"
    } else if file_type.is_block_device() {
        "block"
    } else if file_type.is_socket() {
        "socket"
    } else {
        "unknown"
    };
    Some(name.as_bytes().to_vec())
}

#[cfg(not(unix))]
fn path_filetype(bytes: &[u8]) -> Option<Vec<u8>> {
    let path = std::str::from_utf8(bytes).ok()?;
    let file_type = std::fs::symlink_metadata(Path::new(path)).ok()?.file_type();
    let name = if file_type.is_symlink() {
        "link"
    } else if file_type.is_dir() {
        "dir"
    } else if file_type.is_file() {
        "file"
    } else {
        "unknown"
    };
    Some(name.as_bytes().to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_file_get_contents(
    filename: EchoValue,
    _use_include_path: EchoValue,
    _context: EchoValue,
    offset: EchoValue,
    length: EchoValue,
) -> EchoValue {
    let Some(filename) = filename.string_bytes() else {
        return EchoValue::error();
    };
    let offset = offset.php_int_value().unwrap_or(0);
    let length = if length.is_null() {
        None
    } else {
        match length.php_int_value() {
            Some(value) if value >= 0 => Some(value as usize),
            Some(_) => return EchoValue::bool(false),
            None => None,
        }
    };

    path_file_get_contents(&filename, offset, length)
        .map(echo_runtime_string)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_file_put_contents(
    filename: EchoValue,
    data: EchoValue,
    flags: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(filename) = filename.string_bytes() else {
        return EchoValue::error();
    };
    let Some(data) = data.string_bytes() else {
        return EchoValue::error();
    };
    let flags = flags.php_int_value().unwrap_or(0);

    path_file_put_contents(&filename, &data, flags)
        .and_then(|written| i64::try_from(written).ok())
        .map(EchoValue::int)
        .unwrap_or_else(|| EchoValue::bool(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_readfile(
    filename: EchoValue,
    _use_include_path: EchoValue,
    _context: EchoValue,
) -> EchoValue {
    let Some(filename) = filename.string_bytes() else {
        return EchoValue::error();
    };

    let Some(bytes) = path_file_get_contents(&filename, 0, None) else {
        return EchoValue::bool(false);
    };
    write_runtime_output(&bytes);

    i64::try_from(bytes.len())
        .map(EchoValue::int)
        .unwrap_or_else(|_| EchoValue::bool(false))
}

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

pub(crate) fn path_file_get_contents(
    bytes: &[u8],
    offset: i64,
    length: Option<usize>,
) -> Option<Vec<u8>> {
    let path = path_buf_from_bytes(bytes)?;
    let data = std::fs::read(path).ok()?;
    let start = if offset >= 0 {
        usize::try_from(offset).ok()?
    } else {
        let from_end = usize::try_from(offset.unsigned_abs()).ok()?;
        data.len().checked_sub(from_end)?
    };
    if start > data.len() {
        return None;
    }

    let end = length
        .and_then(|length| start.checked_add(length))
        .map(|end| end.min(data.len()))
        .unwrap_or(data.len());
    Some(data[start..end].to_vec())
}

pub(crate) const PHP_FILE_APPEND: i64 = 8;

fn path_file_put_contents(bytes: &[u8], data: &[u8], flags: i64) -> Option<usize> {
    let path = path_buf_from_bytes(bytes)?;
    let append = flags & PHP_FILE_APPEND != 0;
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let mut file = options.open(path).ok()?;
    file.write_all(data).ok()?;
    Some(data.len())
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
