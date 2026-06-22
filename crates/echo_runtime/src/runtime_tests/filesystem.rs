use super::*;
use crate::filesystem::path_getcwd;
use std::env;
use std::path::Path;

mod content;
mod links;
mod metadata;
mod mutation;
mod temporary;

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
