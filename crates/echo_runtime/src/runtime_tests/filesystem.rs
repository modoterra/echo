use super::*;
use crate::filesystem::{
    PHP_FILE_APPEND, path_buf_from_bytes, path_getcwd, path_is_dir, path_is_file, path_realpath,
};
use std::env;
use std::path::Path;

mod content;
mod links;

#[test]
fn file_exists_reports_existing_files_and_directories() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let existing_file = manifest_dir.join("Cargo.toml");
    let existing_dir = manifest_dir.join("src");
    let missing_path = manifest_dir.join("definitely_missing_echo_file");
    let cargo_toml = Box::into_raw(Box::new(EchoString {
        bytes: existing_file.to_string_lossy().as_bytes().to_vec(),
    }));
    let src_dir = Box::into_raw(Box::new(EchoString {
        bytes: existing_dir.to_string_lossy().as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: missing_path.to_string_lossy().as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_file_exists(EchoValue::string(cargo_toml)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_file_exists(EchoValue::string(src_dir)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_file_exists(EchoValue::string(missing)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_file_exists(EchoValue::string(empty)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(cargo_toml));
        drop(Box::from_raw(src_dir));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty));
    }
}

#[test]
fn chdir_and_getcwd_preserve_php_working_directory_behavior() {
    let original = env::current_dir().expect("current dir");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let missing = manifest_dir.join("definitely_missing_echo_directory");
    let original_bytes = original.to_string_lossy().as_bytes().to_vec();
    let missing_bytes = missing.to_string_lossy().as_bytes().to_vec();

    assert_eq!(
        echo_php_chdir(test_string_value(&original_bytes)),
        EchoValue::bool(true)
    );
    assert_eq!(echo_php_getcwd().string_bytes(), path_getcwd());
    assert_eq!(
        echo_php_chdir(test_string_value(&missing_bytes)),
        EchoValue::bool(false)
    );
}

#[test]
fn is_dir_reports_only_existing_directories() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let existing_file = manifest_dir.join("Cargo.toml");
    let existing_dir = manifest_dir.join("src");
    let missing_path = manifest_dir.join("definitely_missing_echo_directory");
    let cargo_toml = Box::into_raw(Box::new(EchoString {
        bytes: existing_file.to_string_lossy().as_bytes().to_vec(),
    }));
    let src_dir = Box::into_raw(Box::new(EchoString {
        bytes: existing_dir.to_string_lossy().as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: missing_path.to_string_lossy().as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_is_dir(EchoValue::string(cargo_toml)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_dir(EchoValue::string(src_dir)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_dir(EchoValue::string(missing)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_dir(EchoValue::string(empty)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(cargo_toml));
        drop(Box::from_raw(src_dir));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty));
    }
}

#[test]
fn is_file_reports_only_existing_regular_files() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let existing_file = manifest_dir.join("Cargo.toml");
    let existing_dir = manifest_dir.join("src");
    let missing_path = manifest_dir.join("definitely_missing_echo_file");
    let cargo_toml = Box::into_raw(Box::new(EchoString {
        bytes: existing_file.to_string_lossy().as_bytes().to_vec(),
    }));
    let src_dir = Box::into_raw(Box::new(EchoString {
        bytes: existing_dir.to_string_lossy().as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: missing_path.to_string_lossy().as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_is_file(EchoValue::string(cargo_toml)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_file(EchoValue::string(src_dir)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_file(EchoValue::string(missing)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_file(EchoValue::string(empty)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(cargo_toml));
        drop(Box::from_raw(src_dir));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty));
    }
}

#[cfg(unix)]
#[test]
fn is_link_reports_only_existing_symbolic_links() {
    let temp_dir =
        std::env::temp_dir().join(format!("echo-runtime-is-link-{}", std::process::id()));
    let target_path = temp_dir.join("target.txt");
    let link_path = temp_dir.join("linked-target.txt");
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::create_dir_all(&temp_dir).expect("create temp test directory");
    std::fs::write(&target_path, b"target").expect("write symlink target");
    std::os::unix::fs::symlink(&target_path, &link_path).expect("create symlink");

    let target = Box::into_raw(Box::new(EchoString {
        bytes: target_path.to_string_lossy().as_bytes().to_vec(),
    }));
    let link = Box::into_raw(Box::new(EchoString {
        bytes: link_path.to_string_lossy().as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: temp_dir
            .join("definitely_missing_echo_link")
            .to_string_lossy()
            .as_bytes()
            .to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_is_link(EchoValue::string(target)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_link(EchoValue::string(link)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_link(EchoValue::string(missing)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_link(EchoValue::string(empty)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(target));
        drop(Box::from_raw(link));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty));
    }
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn temporary_name_builtins_create_files_and_identifiers() {
    let temp_dir = std::env::temp_dir().join(format!(
        "echo-runtime-temporary-names-{}",
        std::process::id()
    ));
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::create_dir_all(&temp_dir).expect("create temp test directory");

    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        })))
    }

    let sys_temp = echo_php_sys_get_temp_dir();
    let sys_temp_bytes = sys_temp.string_bytes().expect("temp dir string");
    assert!(path_is_dir(&sys_temp_bytes));

    let temp_file = echo_php_tempnam(
        string_value(temp_dir.to_string_lossy().as_bytes()),
        string_value(b"exo"),
    );
    let temp_file_bytes = temp_file.string_bytes().expect("tempnam string");
    assert!(path_is_file(&temp_file_bytes));
    assert!(
        path_buf_from_bytes(&temp_file_bytes)
            .and_then(|path| path.file_name().map(|name| name.to_owned()))
            .and_then(|name| name.into_string().ok())
            .is_some_and(|name| name.starts_with("exo"))
    );

    let plain = echo_php_uniqid(EchoValue::null(), EchoValue::bool(false))
        .string_bytes()
        .expect("uniqid string");
    let prefixed = echo_php_uniqid(string_value(b"job_"), EchoValue::bool(true))
        .string_bytes()
        .expect("prefixed uniqid string");
    assert_eq!(plain.len(), 13);
    assert_eq!(prefixed.len(), 27);
    assert!(prefixed.starts_with(b"job_"));

    std::fs::remove_file(path_buf_from_bytes(&temp_file_bytes).expect("tempnam path")).ok();
    std::fs::remove_dir_all(&temp_dir).ok();
}

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

#[test]
fn filesystem_mutation_builtins_create_move_and_remove_paths() {
    let temp_dir = std::env::temp_dir().join(format!(
        "echo-runtime-filesystem-mutation-{}",
        std::process::id()
    ));
    let nested_dir = temp_dir.join("cache").join("daily");
    let marker_path = nested_dir.join("marker.txt");
    let copied_path = nested_dir.join("marker-copy.txt");
    let renamed_path = nested_dir.join("marker-final.txt");
    let missing_path = nested_dir.join("missing.txt");
    std::fs::remove_dir_all(&temp_dir).ok();

    fn path_value(path: &Path) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: path.to_string_lossy().as_bytes().to_vec(),
        })))
    }

    let nested = path_value(&nested_dir);
    let marker = path_value(&marker_path);
    let copied = path_value(&copied_path);
    let renamed = path_value(&renamed_path);
    let missing = path_value(&missing_path);

    assert_eq!(
        echo_php_mkdir(
            nested,
            EchoValue::int(0o755),
            EchoValue::bool(true),
            EchoValue::null()
        ),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_mkdir(
            nested,
            EchoValue::int(0o755),
            EchoValue::bool(true),
            EchoValue::null()
        ),
        EchoValue::bool(false)
    );
    assert!(nested_dir.is_dir());

    assert_eq!(
        echo_php_touch(marker, EchoValue::int(1_700_000_000), EchoValue::null()),
        EchoValue::bool(true)
    );
    assert_eq!(echo_php_filemtime(marker), EchoValue::int(1_700_000_000));
    assert!(marker_path.is_file());

    assert_eq!(
        echo_php_copy(marker, copied, EchoValue::null()),
        EchoValue::bool(true)
    );
    assert!(copied_path.is_file());

    assert_eq!(
        echo_php_rename(copied, renamed, EchoValue::null()),
        EchoValue::bool(true)
    );
    assert!(!copied_path.exists());
    assert!(renamed_path.is_file());

    assert_eq!(
        echo_php_unlink(renamed, EchoValue::null()),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_unlink(missing, EchoValue::null()),
        EchoValue::bool(false)
    );
    assert!(!renamed_path.exists());

    assert_eq!(
        echo_php_unlink(marker, EchoValue::null()),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_rmdir(nested, EchoValue::null()),
        EchoValue::bool(true)
    );
    assert!(!nested_dir.exists());

    std::fs::remove_dir_all(&temp_dir).ok();
}
