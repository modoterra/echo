use crate::collections::EchoArrayKey;
use crate::{EchoArray, EchoValue, echo_runtime_string};
use std::env;
#[cfg(unix)]
use std::ffi::OsStr;
use std::fs::OpenOptions;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::time::unix_duration_now_or_zero;

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

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_lstat(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_lstat(&bytes)
            .map(stat_array)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_stat(filename: EchoValue) -> EchoValue {
    match filename.string_bytes() {
        Some(bytes) => path_stat(&bytes)
            .map(stat_array)
            .unwrap_or_else(|| EchoValue::bool(false)),
        None => EchoValue::error(),
    }
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
pub(crate) fn path_exists(bytes: &[u8]) -> bool {
    Path::new(OsStr::from_bytes(bytes)).exists()
}

#[cfg(unix)]
pub(crate) fn path_chdir(bytes: &[u8]) -> bool {
    env::set_current_dir(Path::new(OsStr::from_bytes(bytes))).is_ok()
}

#[cfg(unix)]
pub(crate) fn path_getcwd() -> Option<Vec<u8>> {
    env::current_dir()
        .ok()
        .map(|path| path.into_os_string().as_bytes().to_vec())
}

#[cfg(not(unix))]
pub(crate) fn path_exists(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes)
        .map(|path| Path::new(path).exists())
        .unwrap_or(false)
}

#[cfg(not(unix))]
pub(crate) fn path_chdir(bytes: &[u8]) -> bool {
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

#[derive(Clone, Copy)]
pub(crate) struct PhpStat {
    dev: i64,
    ino: i64,
    mode: i64,
    nlink: i64,
    uid: i64,
    gid: i64,
    rdev: i64,
    size: i64,
    atime: i64,
    mtime: i64,
    ctime: i64,
    blksize: i64,
    blocks: i64,
}

pub(crate) fn stat_array(stat: PhpStat) -> EchoValue {
    let fields = [
        ("dev", stat.dev),
        ("ino", stat.ino),
        ("mode", stat.mode),
        ("nlink", stat.nlink),
        ("uid", stat.uid),
        ("gid", stat.gid),
        ("rdev", stat.rdev),
        ("size", stat.size),
        ("atime", stat.atime),
        ("mtime", stat.mtime),
        ("ctime", stat.ctime),
        ("blksize", stat.blksize),
        ("blocks", stat.blocks),
    ];
    let mut keys = Vec::with_capacity(fields.len() * 2);
    let mut values = Vec::with_capacity(fields.len() * 2);
    for (index, (_, value)) in fields.iter().enumerate() {
        keys.push(EchoArrayKey::Int(index as i64));
        values.push(EchoValue::int(*value));
    }
    for (name, value) in fields {
        keys.push(EchoArrayKey::String(name.as_bytes().to_vec()));
        values.push(EchoValue::int(value));
    }

    EchoValue::array(Box::into_raw(Box::new(EchoArray { keys, values })))
}

#[cfg(unix)]
fn path_lstat(bytes: &[u8]) -> Option<PhpStat> {
    let metadata = std::fs::symlink_metadata(Path::new(OsStr::from_bytes(bytes))).ok()?;
    php_stat_from_metadata(metadata)
}

#[cfg(unix)]
fn path_stat(bytes: &[u8]) -> Option<PhpStat> {
    let metadata = std::fs::metadata(Path::new(OsStr::from_bytes(bytes))).ok()?;
    php_stat_from_metadata(metadata)
}

#[cfg(unix)]
pub(crate) fn php_stat_from_metadata(metadata: std::fs::Metadata) -> Option<PhpStat> {
    use std::os::unix::fs::MetadataExt;

    Some(PhpStat {
        dev: metadata.dev() as i64,
        ino: i64::try_from(metadata.ino()).ok()?,
        mode: metadata.mode() as i64,
        nlink: metadata.nlink() as i64,
        uid: metadata.uid() as i64,
        gid: metadata.gid() as i64,
        rdev: metadata.rdev() as i64,
        size: i64::try_from(metadata.size()).ok()?,
        atime: metadata.atime(),
        mtime: metadata.mtime(),
        ctime: metadata.ctime(),
        blksize: metadata.blksize() as i64,
        blocks: metadata.blocks() as i64,
    })
}

#[cfg(not(unix))]
fn path_lstat(bytes: &[u8]) -> Option<PhpStat> {
    let path = std::str::from_utf8(bytes).ok()?;
    let metadata = std::fs::symlink_metadata(Path::new(path)).ok()?;
    php_stat_from_metadata(metadata)
}

#[cfg(not(unix))]
fn path_stat(bytes: &[u8]) -> Option<PhpStat> {
    let path = std::str::from_utf8(bytes).ok()?;
    let metadata = std::fs::metadata(Path::new(path)).ok()?;
    php_stat_from_metadata(metadata)
}

#[cfg(not(unix))]
pub(crate) fn php_stat_from_metadata(metadata: std::fs::Metadata) -> Option<PhpStat> {
    Some(PhpStat {
        dev: 0,
        ino: 0,
        mode: 0,
        nlink: 0,
        uid: 0,
        gid: 0,
        rdev: 0,
        size: i64::try_from(metadata.len()).ok()?,
        atime: metadata
            .accessed()
            .ok()
            .and_then(crate::time::system_time_unix_timestamp)
            .unwrap_or(0),
        mtime: metadata
            .modified()
            .ok()
            .and_then(crate::time::system_time_unix_timestamp)
            .unwrap_or(0),
        ctime: metadata
            .created()
            .ok()
            .and_then(crate::time::system_time_unix_timestamp)
            .unwrap_or(0),
        blksize: 0,
        blocks: 0,
    })
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
