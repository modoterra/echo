use super::*;

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

#[test]
fn upload_file_builtins_reject_ordinary_local_paths() {
    let temp_dir = std::env::temp_dir().join(format!(
        "echo-runtime-upload-file-guards-{}",
        std::process::id()
    ));
    let source_path = temp_dir.join("candidate.txt");
    let target_path = temp_dir.join("moved.txt");
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::create_dir_all(&temp_dir).expect("create upload guard fixture");
    std::fs::write(&source_path, b"payload\n").expect("write upload guard fixture");

    fn path_value(path: &Path) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: path.to_string_lossy().as_bytes().to_vec(),
        })))
    }

    let source = path_value(&source_path);
    let target = path_value(&target_path);

    assert_eq!(echo_php_is_uploaded_file(source), EchoValue::bool(false));
    assert_eq!(
        echo_php_move_uploaded_file(source, target),
        EchoValue::bool(false)
    );
    assert!(source_path.is_file());
    assert!(!target_path.exists());

    std::fs::remove_dir_all(&temp_dir).ok();
}
