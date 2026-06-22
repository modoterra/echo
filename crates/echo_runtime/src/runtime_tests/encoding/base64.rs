use super::*;

#[test]
fn base64_encode_preserves_php_byte_behavior() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let one_byte = Box::into_raw(Box::new(EchoString {
        bytes: "f".as_bytes().to_vec(),
    }));
    let two_bytes = Box::into_raw(Box::new(EchoString {
        bytes: "fo".as_bytes().to_vec(),
    }));
    let three_bytes = Box::into_raw(Box::new(EchoString {
        bytes: "foo".as_bytes().to_vec(),
    }));
    let text = Box::into_raw(Box::new(EchoString {
        bytes: "hello world".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_base64_encode(EchoValue::string(empty)).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(one_byte)).string_bytes(),
        Some("Zg==".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(two_bytes)).string_bytes(),
        Some("Zm8=".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(three_bytes)).string_bytes(),
        Some("Zm9v".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(text)).string_bytes(),
        Some("aGVsbG8gd29ybGQ=".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(non_ascii)).string_bytes(),
        Some("w4RjaG8=".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::int(123)).string_bytes(),
        Some("MTIz".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(one_byte));
        drop(Box::from_raw(two_bytes));
        drop(Box::from_raw(three_bytes));
        drop(Box::from_raw(text));
        drop(Box::from_raw(non_ascii));
    }
}

#[test]
fn base64_decode_preserves_php_default_non_strict_byte_behavior() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let one_byte = Box::into_raw(Box::new(EchoString {
        bytes: "Zg==".as_bytes().to_vec(),
    }));
    let two_bytes = Box::into_raw(Box::new(EchoString {
        bytes: "Zm8=".as_bytes().to_vec(),
    }));
    let three_bytes = Box::into_raw(Box::new(EchoString {
        bytes: "Zm9v".as_bytes().to_vec(),
    }));
    let text = Box::into_raw(Box::new(EchoString {
        bytes: "aGVsbG8gd29ybGQ=".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "w4RjaG8=".as_bytes().to_vec(),
    }));
    let ignored = Box::into_raw(Box::new(EchoString {
        bytes: "Zm 9v".as_bytes().to_vec(),
    }));
    let invalid = Box::into_raw(Box::new(EchoString {
        bytes: "!!!!".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_base64_decode(EchoValue::string(empty)).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(one_byte)).string_bytes(),
        Some("f".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(two_bytes)).string_bytes(),
        Some("fo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(three_bytes)).string_bytes(),
        Some("foo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(text)).string_bytes(),
        Some("hello world".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(non_ascii)).string_bytes(),
        Some("Ächo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(ignored)).string_bytes(),
        Some("foo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(invalid)).string_bytes(),
        Some(Vec::new())
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(one_byte));
        drop(Box::from_raw(two_bytes));
        drop(Box::from_raw(three_bytes));
        drop(Box::from_raw(text));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(ignored));
        drop(Box::from_raw(invalid));
    }
}
