use super::*;
use crate::filesystem::PHP_FILE_APPEND;

#[test]
fn filesystem_content_builtins_read_write_append_and_stream_output() {
    let temp_dir = std::env::temp_dir().join(format!(
        "echo-runtime-filesystem-content-{}",
        std::process::id()
    ));
    let file_path = temp_dir.join("report.txt");
    let missing_path = temp_dir.join("missing.txt");
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::create_dir_all(&temp_dir).expect("create temp test directory");

    fn path_value(path: &Path) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: path.to_string_lossy().as_bytes().to_vec(),
        })))
    }

    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        })))
    }

    let file = path_value(&file_path);
    let missing = path_value(&missing_path);

    assert_eq!(
        echo_php_file_put_contents(
            file,
            string_value(b"alpha\nbeta\ngamma\n"),
            EchoValue::int(0),
            EchoValue::null()
        ),
        EchoValue::int(17)
    );
    assert_eq!(
        echo_php_file_put_contents(
            file,
            string_value(b"delta\n"),
            EchoValue::int(PHP_FILE_APPEND),
            EchoValue::null()
        ),
        EchoValue::int(6)
    );
    assert_eq!(
        echo_php_file_get_contents(
            file,
            EchoValue::bool(false),
            EchoValue::null(),
            EchoValue::int(6),
            EchoValue::int(4)
        )
        .string_bytes(),
        Some(b"beta".to_vec())
    );
    assert_eq!(
        echo_php_file_get_contents(
            file,
            EchoValue::bool(false),
            EchoValue::null(),
            EchoValue::int(-6),
            EchoValue::null()
        )
        .string_bytes(),
        Some(b"delta\n".to_vec())
    );
    assert_eq!(
        echo_php_file_get_contents(
            missing,
            EchoValue::bool(false),
            EchoValue::null(),
            EchoValue::int(0),
            EchoValue::null()
        ),
        EchoValue::bool(false)
    );

    let (bytes_read, stdout) = capture_stdout(false, || {
        echo_php_readfile(file, EchoValue::bool(false), EchoValue::null())
    });
    assert_eq!(bytes_read, EchoValue::int(23));
    assert_eq!(stdout, b"alpha\nbeta\ngamma\ndelta\n");

    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn php_strip_whitespace_removes_comments_and_collapses_space() {
    let temp_dir = std::env::temp_dir().join(format!(
        "echo-runtime-strip-whitespace-{}",
        std::process::id()
    ));
    let source_path = temp_dir.join("source.php");
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::create_dir_all(&temp_dir).expect("create temp test directory");
    std::fs::write(
        &source_path,
        b"<?php\n// leading comment\n$name = \"Ada // not a comment\"; /* inline */\necho  $name  .  \"\\n\";\n# tail\n",
    )
    .expect("write strip fixture");

    let path = EchoValue::string(Box::into_raw(Box::new(EchoString {
        bytes: source_path.to_string_lossy().as_bytes().to_vec(),
    })));
    assert_eq!(
        echo_php_php_strip_whitespace(path).string_bytes(),
        Some(b"<?php\n $name = \"Ada // not a comment\"; echo $name . \"\\n\"; ".to_vec())
    );

    std::fs::remove_dir_all(&temp_dir).ok();
}
