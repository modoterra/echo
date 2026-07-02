use super::*;
use std::io::Write;
use std::path::Path;

#[test]
fn fopen_and_stream_reading_updates_pointer_state() {
    let fixture_dir =
        std::env::temp_dir().join(format!("echo-runtime-stream-tests-{}", std::process::id()));
    std::fs::remove_dir_all(&fixture_dir).ok();
    std::fs::create_dir_all(&fixture_dir).expect("create stream test directory");
    let path = fixture_dir.join("stream-reader.txt");
    {
        let mut file = std::fs::File::create(&path).expect("create stream fixture");
        file.write_all(b"hello world")
            .expect("write stream fixture");
    }

    let path = Box::into_raw(Box::new(EchoString {
        bytes: path.to_string_lossy().as_bytes().to_vec(),
    }));
    let stream = echo_php_fopen(
        EchoValue::string(path),
        test_string_value(b"r"),
        EchoValue::bool(false),
        EchoValue::null(),
    );
    assert!(echo_php_is_resource(stream).is_true_bool());

    assert_eq!(
        echo_php_fread(stream, test_string_value(b"5")).string_bytes(),
        Some(b"hello".to_vec())
    );
    assert_eq!(echo_php_ftell(stream), EchoValue::int(5));
    assert_eq!(
        echo_php_stream_get_contents(stream, EchoValue::null(), EchoValue::int(-1)).string_bytes(),
        Some(b" world".to_vec())
    );
    assert_eq!(echo_php_ftell(stream), EchoValue::int(11));
    assert_eq!(echo_php_fseek(stream, EchoValue::int(6)), EchoValue::int(0));
    assert_eq!(echo_php_ftell(stream), EchoValue::int(6));
    assert_eq!(echo_php_fclose(stream), EchoValue::bool(true));
    assert_eq!(echo_php_ftell(stream), EchoValue::bool(false));
    assert_eq!(
        echo_php_fseek(stream, EchoValue::int(0)),
        EchoValue::int(-1)
    );
    assert_eq!(echo_php_fclose(stream), EchoValue::bool(false));
    assert_eq!(
        echo_php_fread(stream, test_string_value(b"3")),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(path));
    }
    assert_eq!(
        echo_php_is_resource(echo_php_tmpfile()).is_true_bool(),
        true
    );
    std::fs::remove_dir_all(fixture_dir).ok();
}

#[test]
fn fopen_rejects_unsupported_mode() {
    let temp_file = std::env::temp_dir().join(format!(
        "echo-runtime-stream-mode-{}.txt",
        std::process::id()
    ));
    std::fs::write(&temp_file, b"content").expect("write mode test fixture");

    let filename = Box::into_raw(Box::new(EchoString {
        bytes: temp_file.to_string_lossy().as_bytes().to_vec(),
    }));
    assert_eq!(
        echo_php_fopen(
            EchoValue::string(filename),
            test_string_value(b"q"),
            EchoValue::bool(false),
            EchoValue::null(),
        ),
        EchoValue::bool(false)
    );
    std::fs::remove_file(&temp_file).ok();

    unsafe {
        drop(Box::from_raw(filename));
    }
}

#[test]
fn stream_get_contents_fails_for_closed_stream() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file = manifest_dir.join("Cargo.toml");
    let filename = Box::into_raw(Box::new(EchoString {
        bytes: file.to_string_lossy().as_bytes().to_vec(),
    }));
    let stream = echo_php_fopen(
        EchoValue::string(filename),
        test_string_value(b"r"),
        EchoValue::bool(false),
        EchoValue::null(),
    );
    assert_eq!(echo_php_fclose(stream), EchoValue::bool(true));
    assert_eq!(
        echo_php_stream_get_contents(stream, EchoValue::null(), EchoValue::int(-1)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(filename));
    }
}
