use super::*;
use crate::filesystem::path_realpath;

#[test]
fn filesystem_metadata_builtins_report_paths_and_false_failures() {
    let temp_dir = std::env::temp_dir().join(format!(
        "echo-runtime-filesystem-metadata-{}",
        std::process::id()
    ));
    let file_path = temp_dir.join("sample.txt");
    let script_path = temp_dir.join("run.sh");
    let missing_path = temp_dir.join("missing.txt");
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::create_dir_all(&temp_dir).expect("create temp test directory");
    std::fs::write(&file_path, b"Echo file\n").expect("write sample file");
    std::fs::write(&script_path, b"#!/bin/sh\nexit 0\n").expect("write script file");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&script_path)
            .expect("stat script")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script_path, permissions).expect("chmod script");
    }

    fn path_value(path: &Path) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: path.to_string_lossy().as_bytes().to_vec(),
        })))
    }

    let file = path_value(&file_path);
    let script = path_value(&script_path);
    let dir = path_value(&temp_dir);
    let missing = path_value(&missing_path);
    let parent_lookup = EchoValue::string(Box::into_raw(Box::new(EchoString {
        bytes: temp_dir
            .join("..")
            .join(temp_dir.file_name().expect("temp dir name"))
            .join("sample.txt")
            .to_string_lossy()
            .as_bytes()
            .to_vec(),
    })));

    assert_eq!(echo_php_is_readable(file), EchoValue::bool(true));
    assert_eq!(echo_php_is_writable(file), EchoValue::bool(true));
    assert_eq!(echo_php_is_executable(file), EchoValue::bool(false));
    assert_eq!(echo_php_is_executable(script), EchoValue::bool(cfg!(unix)));
    assert_eq!(echo_php_is_readable(missing), EchoValue::bool(false));
    assert_eq!(echo_php_is_writable(missing), EchoValue::bool(false));
    assert_eq!(echo_php_filesize(file), EchoValue::int(10));
    assert_eq!(echo_php_filesize(missing), EchoValue::bool(false));
    assert_eq!(
        echo_php_filetype(file).string_bytes(),
        Some(b"file".to_vec())
    );
    assert_eq!(echo_php_filetype(dir).string_bytes(), Some(b"dir".to_vec()));
    assert!(echo_php_fileatime(file).is_int());
    assert!(echo_php_filectime(file).is_int());
    assert!(echo_php_filemtime(file).is_int());
    assert_eq!(echo_php_fileatime(missing), EchoValue::bool(false));
    assert_eq!(echo_php_filetype(missing), EchoValue::bool(false));
    #[cfg(unix)]
    {
        assert!(echo_php_fileinode(file).is_int());
        assert!(echo_php_fileowner(file).is_int());
        assert!(echo_php_filegroup(file).is_int());
        assert!(echo_php_fileperms(file).is_int());
    }
    assert_eq!(echo_php_realpath(missing), EchoValue::bool(false));
    assert_eq!(
        echo_php_realpath(parent_lookup).string_bytes(),
        path_realpath(file_path.to_string_lossy().as_bytes())
    );

    std::fs::remove_dir_all(&temp_dir).ok();
}
