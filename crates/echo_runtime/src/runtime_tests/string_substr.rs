use super::*;

#[test]
fn substr_preserves_php_byte_behavior() {
    let positive = Box::into_raw(Box::new(EchoString {
        bytes: "Echo PHP".as_bytes().to_vec(),
    }));
    let out_of_range = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_offset = Box::into_raw(Box::new(EchoString {
        bytes: "1".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let negative = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_substr(EchoValue::string(positive), EchoValue::int(5)).string_bytes(),
        Some("PHP".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_substr(EchoValue::string(out_of_range), EchoValue::int(99)).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_substr(EchoValue::string(negative), EchoValue::int(-2)).string_bytes(),
        Some("ef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_substr(
            EchoValue::string(non_ascii),
            EchoValue::string(numeric_offset)
        )
        .string_bytes(),
        Some(vec![0x84, b'c', b'h', b'o'])
    );

    unsafe {
        drop(Box::from_raw(positive));
        drop(Box::from_raw(out_of_range));
        drop(Box::from_raw(numeric_offset));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(negative));
    }
}

#[test]
fn substr_replace_preserves_php_scalar_offset_and_length_behavior() {
    assert_eq!(
        echo_php_substr_replace(
            test_string_value(b"Bearer abc123"),
            test_string_value(b"redacted"),
            EchoValue::int(7),
            EchoValue::null(),
        )
        .string_bytes(),
        Some(b"Bearer redacted".to_vec())
    );
    assert_eq!(
        echo_php_substr_replace(
            test_string_value(b"invoice-2026-draft.txt"),
            test_string_value(b"-final"),
            EchoValue::int(-4),
            EchoValue::int(0),
        )
        .string_bytes(),
        Some(b"invoice-2026-draft-final.txt".to_vec())
    );
    assert_eq!(
        echo_php_substr_replace(
            test_string_value(b"abcdef"),
            test_string_value(b"XX"),
            EchoValue::int(2),
            EchoValue::int(3),
        )
        .string_bytes(),
        Some(b"abXXf".to_vec())
    );
    assert_eq!(
        echo_php_substr_replace(
            test_string_value(b"abcdef"),
            test_string_value(b"YY"),
            EchoValue::int(2),
            EchoValue::int(-1),
        )
        .string_bytes(),
        Some(b"abYYf".to_vec())
    );
    assert_eq!(
        echo_php_substr_replace(
            test_string_value(b"abc"),
            test_string_value(b"!"),
            EchoValue::int(99),
            EchoValue::null(),
        )
        .string_bytes(),
        Some(b"abc!".to_vec())
    );
}
