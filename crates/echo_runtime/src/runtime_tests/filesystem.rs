use super::*;
use crate::collections::EchoArrayKey;
use crate::filesystem::path_getcwd;
use std::env;
use std::path::Path;

mod content;
mod links;
mod metadata;
mod mutation;
mod stream;
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
fn disk_space_reports_float_counts_for_existing_directories() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let missing_path = manifest_dir.join("definitely_missing_echo_directory");
    let existing = Box::into_raw(Box::new(EchoString {
        bytes: manifest_dir.to_string_lossy().as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: missing_path.to_string_lossy().as_bytes().to_vec(),
    }));

    let free = echo_php_disk_free_space(EchoValue::string(existing));
    let total = echo_php_disk_total_space(EchoValue::string(existing));

    assert!(free.is_float());
    assert!(total.is_float());
    assert!(f64::from_bits(free.payload) <= f64::from_bits(total.payload));
    assert_eq!(
        echo_php_disk_free_space(EchoValue::string(missing)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(existing));
        drop(Box::from_raw(missing));
    }
}

#[test]
fn glob_returns_sorted_matches_for_local_directory_patterns() {
    let fixture_dir =
        std::env::temp_dir().join(format!("echo-runtime-glob-tests-{}", std::process::id()));
    std::fs::remove_dir_all(&fixture_dir).ok();
    std::fs::create_dir_all(&fixture_dir).expect("create glob fixture");
    std::fs::write(fixture_dir.join("b.txt"), b"b").expect("write glob fixture");
    std::fs::write(fixture_dir.join("a.txt"), b"a").expect("write glob fixture");
    std::fs::write(fixture_dir.join("ignore.log"), b"log").expect("write glob fixture");

    let pattern = Box::into_raw(Box::new(EchoString {
        bytes: fixture_dir
            .join("*.txt")
            .to_string_lossy()
            .as_bytes()
            .to_vec(),
    }));
    let result = echo_php_glob(EchoValue::string(pattern), EchoValue::int(0));
    let array = unsafe { (result.payload as *const EchoArray).as_ref() }.expect("glob array");

    assert_eq!(array.keys, vec![EchoArrayKey::Int(0), EchoArrayKey::Int(1)]);
    assert_eq!(
        array.values[0].string_bytes(),
        Some(
            fixture_dir
                .join("a.txt")
                .to_string_lossy()
                .as_bytes()
                .to_vec()
        )
    );
    assert_eq!(
        array.values[1].string_bytes(),
        Some(
            fixture_dir
                .join("b.txt")
                .to_string_lossy()
                .as_bytes()
                .to_vec()
        )
    );

    let missing_pattern = Box::into_raw(Box::new(EchoString {
        bytes: fixture_dir
            .join("*.missing")
            .to_string_lossy()
            .as_bytes()
            .to_vec(),
    }));
    let missing = echo_php_glob(EchoValue::string(missing_pattern), EchoValue::int(0));
    let missing_array =
        unsafe { (missing.payload as *const EchoArray).as_ref() }.expect("empty glob array");
    assert!(missing_array.values.is_empty());
    assert_eq!(
        echo_php_glob(EchoValue::string(pattern), EchoValue::int(1)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(pattern));
        drop(Box::from_raw(missing_pattern));
    }
    std::fs::remove_dir_all(fixture_dir).ok();
}

#[test]
fn scandir_returns_sorted_directory_entries_with_dots() {
    let fixture_dir =
        std::env::temp_dir().join(format!("echo-runtime-scandir-tests-{}", std::process::id()));
    std::fs::remove_dir_all(&fixture_dir).ok();
    std::fs::create_dir_all(&fixture_dir).expect("create scandir fixture");
    std::fs::write(fixture_dir.join("b.txt"), b"b").expect("write scandir fixture");
    std::fs::write(fixture_dir.join("a.txt"), b"a").expect("write scandir fixture");

    let path = Box::into_raw(Box::new(EchoString {
        bytes: fixture_dir.to_string_lossy().as_bytes().to_vec(),
    }));
    let result = echo_php_scandir(EchoValue::string(path));
    let array = unsafe { (result.payload as *const EchoArray).as_ref() }.expect("scandir array");

    assert_eq!(
        array.keys,
        vec![
            EchoArrayKey::Int(0),
            EchoArrayKey::Int(1),
            EchoArrayKey::Int(2),
            EchoArrayKey::Int(3)
        ]
    );
    assert_eq!(array.values[0].string_bytes(), Some(b".".to_vec()));
    assert_eq!(array.values[1].string_bytes(), Some(b"..".to_vec()));
    assert_eq!(array.values[2].string_bytes(), Some(b"a.txt".to_vec()));
    assert_eq!(array.values[3].string_bytes(), Some(b"b.txt".to_vec()));
    assert_eq!(
        echo_php_scandir(test_string_value(b"/definitely/missing/echo/scandir")),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(path));
    }
    std::fs::remove_dir_all(fixture_dir).ok();
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
fn clearstatcache_accepts_default_and_explicit_arguments() {
    echo_php_clearstatcache(EchoValue::bool(false), EchoValue::null());
    echo_php_clearstatcache(EchoValue::bool(true), test_string_value(b"Cargo.toml"));
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
