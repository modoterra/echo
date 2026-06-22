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

mod metadata;

pub use metadata::{
    echo_php_fileatime, echo_php_filectime, echo_php_filegroup, echo_php_fileinode,
    echo_php_filemtime, echo_php_fileowner, echo_php_fileperms, echo_php_filesize,
    echo_php_filetype, echo_php_is_dir, echo_php_is_executable, echo_php_is_file, echo_php_is_link,
    echo_php_is_readable, echo_php_is_writable,
};
pub(crate) use metadata::{path_chdir, path_exists, path_getcwd};
#[cfg(test)]
pub(crate) use metadata::{path_is_dir, path_is_file};

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
