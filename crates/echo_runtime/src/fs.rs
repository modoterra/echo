//! Filesystem: paths, files, directories (`std/fs`).
//!
//! Path arguments accept **string** or **locator** handles. File payloads are
//! **bytes** (string inputs to write are taken as UTF-8 bytes).

use std::fs;
use std::path::PathBuf;

use crate::{
    bytes_data, bytes_to_handle, echo_runtime_list_new, echo_runtime_list_push, locator_data,
    string_data, string_to_handle,
};

fn path_from_handle(v: i64) -> Option<PathBuf> {
    if let Some(s) = string_data(v) {
        return Some(PathBuf::from(s));
    }
    if let Some(s) = locator_data(v) {
        return Some(PathBuf::from(s));
    }
    None
}

fn payload_bytes(v: i64) -> Option<Vec<u8>> {
    if let Some(b) = bytes_data(v) {
        return Some(b);
    }
    if let Some(s) = string_data(v) {
        return Some(s.into_bytes());
    }
    None
}

/// 1 if path exists, else 0.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_exists(path: i64) -> i64 {
    match path_from_handle(path) {
        Some(p) if p.exists() => 1,
        _ => 0,
    }
}

/// 1 if path is a regular file, else 0.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_is_file(path: i64) -> i64 {
    match path_from_handle(path) {
        Some(p) if p.is_file() => 1,
        _ => 0,
    }
}

/// 1 if path is a directory, else 0.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_is_dir(path: i64) -> i64 {
    match path_from_handle(path) {
        Some(p) if p.is_dir() => 1,
        _ => 0,
    }
}

/// Join two path components → string handle. Empty string on bad handles.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_join(base: i64, rel: i64) -> i64 {
    let Some(b) = path_from_handle(base) else {
        return string_to_handle(String::new());
    };
    let Some(r) = path_from_handle(rel) else {
        return string_to_handle(String::new());
    };
    let joined = b.join(r);
    string_to_handle(joined.to_string_lossy().into_owned())
}

/// Read file as bytes handle. **0** on failure (missing path, I/O error).
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_read(path: i64) -> i64 {
    let Some(p) = path_from_handle(path) else {
        return 0;
    };
    match fs::read(&p) {
        Ok(data) => bytes_to_handle(data),
        Err(_) => 0,
    }
}

/// Write bytes/string payload to path (create/truncate). **0** ok, **-1** fail.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_write(path: i64, data: i64) -> i64 {
    let Some(p) = path_from_handle(path) else {
        return -1;
    };
    let Some(bytes) = payload_bytes(data) else {
        return -1;
    };
    match fs::write(&p, bytes) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Remove a file. **0** ok, **-1** fail.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_remove(path: i64) -> i64 {
    let Some(p) = path_from_handle(path) else {
        return -1;
    };
    match fs::remove_file(&p) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Create a single directory (parent must exist). **0** ok, **-1** fail.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_create_dir(path: i64) -> i64 {
    let Some(p) = path_from_handle(path) else {
        return -1;
    };
    match fs::create_dir(&p) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Create directory and parents. **0** ok, **-1** fail.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_create_dir_all(path: i64) -> i64 {
    let Some(p) = path_from_handle(path) else {
        return -1;
    };
    match fs::create_dir_all(&p) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// List directory entry names (not full paths) as list of strings.
/// **0** on failure. Does not include `.` / `..`.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_read_dir(path: i64) -> i64 {
    let Some(p) = path_from_handle(path) else {
        return 0;
    };
    let rd = match fs::read_dir(&p) {
        Ok(rd) => rd,
        Err(_) => return 0,
    };
    let list = echo_runtime_list_new();
    for ent in rd.flatten() {
        let name = ent.file_name();
        let s = name.to_string_lossy().into_owned();
        let h = string_to_handle(s);
        unsafe {
            echo_runtime_list_push(list, h);
        }
    }
    list
}

/// Remove an empty directory. **0** ok, **-1** fail.
#[unsafe(no_mangle)]
pub extern "C" fn echo_runtime_fs_remove_dir(path: i64) -> i64 {
    let Some(p) = path_from_handle(path) else {
        return -1;
    };
    match fs::remove_dir(&p) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::echo_runtime_string_from_utf8;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn s(text: &str) -> i64 {
        unsafe { echo_runtime_string_from_utf8(text.as_ptr(), text.len()) }
    }

    fn tmp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("echo_fs_test_{nanos}"))
    }

    #[test]
    fn file_write_read_remove() {
        let root = tmp_root();
        fs::create_dir_all(&root).unwrap();
        let file = root.join("a.txt");
        let path = s(file.to_str().unwrap());

        assert_eq!(echo_runtime_fs_exists(path), 0);
        assert_eq!(echo_runtime_fs_write(path, s("hi")), 0);
        assert_eq!(echo_runtime_fs_exists(path), 1);
        assert_eq!(echo_runtime_fs_is_file(path), 1);
        assert_eq!(echo_runtime_fs_is_dir(path), 0);

        let data = echo_runtime_fs_read(path);
        assert_ne!(data, 0);
        assert_eq!(bytes_data(data).as_deref(), Some(b"hi".as_slice()));

        assert_eq!(echo_runtime_fs_remove(path), 0);
        assert_eq!(echo_runtime_fs_exists(path), 0);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn dir_create_list_remove() {
        let root = tmp_root();
        let root_s = s(root.to_str().unwrap());
        assert_eq!(echo_runtime_fs_create_dir_all(root_s), 0);
        assert_eq!(echo_runtime_fs_is_dir(root_s), 1);

        let child = root.join("child");
        let child_s = s(child.to_str().unwrap());
        assert_eq!(echo_runtime_fs_create_dir(child_s), 0);

        let names = echo_runtime_fs_read_dir(root_s);
        assert_ne!(names, 0);
        let n = unsafe { crate::echo_runtime_list_len(names) };
        assert!(n >= 1);

        assert_eq!(echo_runtime_fs_remove_dir(child_s), 0);
        assert_eq!(echo_runtime_fs_remove_dir(root_s), 0);
    }

    #[test]
    fn join_and_missing() {
        let joined = echo_runtime_fs_join(s("/tmp"), s("x"));
        let text = string_data(joined).unwrap();
        assert!(text.contains("x"), "{text}");
        assert_eq!(echo_runtime_fs_read(s("/__echo_fs_missing_xyz__")), 0);
        assert_eq!(echo_runtime_fs_write(0, s("x")), -1);
    }

}
