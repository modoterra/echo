use super::*;

#[test]
fn wordwrap_preserves_php_word_boundaries_and_cut_mode() {
    assert_eq!(
        echo_php_wordwrap(
            test_string_value(b"The quick brown fox jumps"),
            EchoValue::int(10),
            test_string_value(b"\n"),
            EchoValue::bool(false),
        )
        .string_bytes(),
        Some(b"The quick\nbrown fox\njumps".to_vec())
    );
    assert_eq!(
        echo_php_wordwrap(
            test_string_value(b"The quick brown fox"),
            EchoValue::int(12),
            test_string_value(b"|"),
            EchoValue::bool(false),
        )
        .string_bytes(),
        Some(b"The quick|brown fox".to_vec())
    );
    assert_eq!(
        echo_php_wordwrap(
            test_string_value(b"abcdefghij"),
            EchoValue::int(4),
            test_string_value(b"\n"),
            EchoValue::bool(false),
        )
        .string_bytes(),
        Some(b"abcdefghij".to_vec())
    );
    assert_eq!(
        echo_php_wordwrap(
            test_string_value(b"abcdefghij"),
            EchoValue::int(4),
            test_string_value(b"\n"),
            EchoValue::bool(true),
        )
        .string_bytes(),
        Some(b"abcd\nefgh\nij".to_vec())
    );
}

#[test]
fn str_repeat_preserves_php_byte_behavior() {
    let repeated = Box::into_raw(Box::new(EchoString {
        bytes: "xo".as_bytes().to_vec(),
    }));
    let empty_repeat = Box::into_raw(Box::new(EchoString {
        bytes: "x".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_str_repeat(EchoValue::string(repeated), EchoValue::int(3)).string_bytes(),
        Some("xoxoxo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_repeat(EchoValue::string(empty_repeat), EchoValue::int(0)).string_bytes(),
        Some(Vec::new())
    );

    unsafe {
        drop(Box::from_raw(repeated));
        drop(Box::from_raw(empty_repeat));
    }
}

#[test]
fn str_pad_preserves_php_byte_behavior() {
    let right = Box::into_raw(Box::new(EchoString {
        bytes: "ID".as_bytes().to_vec(),
    }));
    let left = Box::into_raw(Box::new(EchoString {
        bytes: "42".as_bytes().to_vec(),
    }));
    let both = Box::into_raw(Box::new(EchoString {
        bytes: "tag".as_bytes().to_vec(),
    }));
    let multi_left = Box::into_raw(Box::new(EchoString {
        bytes: "42".as_bytes().to_vec(),
    }));
    let multi_both = Box::into_raw(Box::new(EchoString {
        bytes: "go".as_bytes().to_vec(),
    }));
    let shorter = Box::into_raw(Box::new(EchoString {
        bytes: "already".as_bytes().to_vec(),
    }));
    let zero = Box::into_raw(Box::new(EchoString {
        bytes: "0".as_bytes().to_vec(),
    }));
    let dash = Box::into_raw(Box::new(EchoString {
        bytes: "-".as_bytes().to_vec(),
    }));
    let ab = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(right),
            EchoValue::int(6),
            EchoValue::string(zero),
            EchoValue::int(1),
        )
        .string_bytes(),
        Some("ID0000".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(left),
            EchoValue::int(5),
            EchoValue::string(zero),
            EchoValue::int(0),
        )
        .string_bytes(),
        Some("00042".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(both),
            EchoValue::int(8),
            EchoValue::string(dash),
            EchoValue::int(2),
        )
        .string_bytes(),
        Some("--tag---".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(multi_left),
            EchoValue::int(7),
            EchoValue::string(ab),
            EchoValue::int(0),
        )
        .string_bytes(),
        Some("ababa42".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(multi_both),
            EchoValue::int(7),
            EchoValue::string(ab),
            EchoValue::int(2),
        )
        .string_bytes(),
        Some("abgoaba".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(shorter),
            EchoValue::int(3),
            EchoValue::string(zero),
            EchoValue::int(0),
        )
        .string_bytes(),
        Some("already".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(right));
        drop(Box::from_raw(left));
        drop(Box::from_raw(both));
        drop(Box::from_raw(multi_left));
        drop(Box::from_raw(multi_both));
        drop(Box::from_raw(shorter));
        drop(Box::from_raw(zero));
        drop(Box::from_raw(dash));
        drop(Box::from_raw(ab));
    }
}

#[test]
fn string_chunk_builtins_preserve_php_byte_behavior() {
    let split = Box::into_raw(Box::new(EchoString {
        bytes: "abcde".as_bytes().to_vec(),
    }));
    let empty_split = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let chunk = Box::into_raw(Box::new(EchoString {
        bytes: "abcde".as_bytes().to_vec(),
    }));
    let empty_chunk = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let pipe = Box::into_raw(Box::new(EchoString {
        bytes: "|".as_bytes().to_vec(),
    }));
    let crlf = Box::into_raw(Box::new(EchoString {
        bytes: "\r\n".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_implode(
            EchoValue::string(pipe),
            echo_php_str_split(EchoValue::string(split), EchoValue::int(2)),
        )
        .string_bytes(),
        Some("ab|cd|e".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_implode(
            EchoValue::string(pipe),
            echo_php_str_split(EchoValue::string(empty_split), EchoValue::int(1)),
        )
        .string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_chunk_split(
            EchoValue::string(chunk),
            EchoValue::int(2),
            EchoValue::string(pipe)
        )
        .string_bytes(),
        Some("ab|cd|e|".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_chunk_split(
            EchoValue::string(empty_chunk),
            EchoValue::int(2),
            EchoValue::string(pipe),
        )
        .string_bytes(),
        Some("|".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_chunk_split(
            EchoValue::int(12345),
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: "2".as_bytes().to_vec(),
            }))),
            EchoValue::string(crlf),
        )
        .string_bytes(),
        Some("12\r\n34\r\n5\r\n".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(split));
        drop(Box::from_raw(empty_split));
        drop(Box::from_raw(chunk));
        drop(Box::from_raw(empty_chunk));
        drop(Box::from_raw(pipe));
        drop(Box::from_raw(crlf));
    }
}
