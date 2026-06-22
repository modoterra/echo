use super::*;

#[test]
fn escapeshellarg_preserves_php_unix_shell_argument_quoting() {
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"Echo")).string_bytes(),
        Some("'Echo'".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"it's ready")).string_bytes(),
        Some("'it'\\''s ready'".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"")).string_bytes(),
        Some("''".as_bytes().to_vec())
    );
}

#[test]
fn escapeshellcmd_preserves_php_unix_shell_meta_escaping() {
    fn string_value(bytes: &[u8]) -> *mut EchoString {
        Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        }))
    }

    let semicolon = string_value(b"path; rm -rf /");
    let paired_double = string_value(b"echo \"ok\"");
    let unpaired_double = string_value(b"echo \"unterminated");
    let paired_single = string_value(b"echo 'ok'");
    let unpaired_single = string_value(b"echo 'unterminated");
    let newline = string_value(b"line\nbreak");
    let slash = string_value(b"a\\b");
    let dollar = string_value(b"a$b");

    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(semicolon)).string_bytes(),
        Some(b"path\\; rm -rf /".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(paired_double)).string_bytes(),
        Some(b"echo \"ok\"".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(unpaired_double)).string_bytes(),
        Some(b"echo \\\"unterminated".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(paired_single)).string_bytes(),
        Some(b"echo 'ok'".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(unpaired_single)).string_bytes(),
        Some(b"echo \\'unterminated".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(newline)).string_bytes(),
        Some(b"line\\\nbreak".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(slash)).string_bytes(),
        Some(b"a\\\\b".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(dollar)).string_bytes(),
        Some(b"a\\$b".to_vec())
    );

    unsafe {
        drop(Box::from_raw(semicolon));
        drop(Box::from_raw(paired_double));
        drop(Box::from_raw(unpaired_double));
        drop(Box::from_raw(paired_single));
        drop(Box::from_raw(unpaired_single));
        drop(Box::from_raw(newline));
        drop(Box::from_raw(slash));
        drop(Box::from_raw(dollar));
    }
}
