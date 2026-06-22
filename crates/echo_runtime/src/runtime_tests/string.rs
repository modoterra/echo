use super::*;

#[test]
fn hex2bin_preserves_php_byte_behavior() {
    let hex = Box::into_raw(Box::new(EchoString {
        bytes: "c384".as_bytes().to_vec(),
    }));
    let upper_hex = Box::into_raw(Box::new(EchoString {
        bytes: "4563686F".as_bytes().to_vec(),
    }));
    let invalid_hex = Box::into_raw(Box::new(EchoString {
        bytes: "f".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_hex2bin(EchoValue::string(hex)).string_bytes(),
        Some("Ä".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_hex2bin(EchoValue::string(upper_hex)).string_bytes(),
        Some("Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_hex2bin(EchoValue::string(invalid_hex)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(hex));
        drop(Box::from_raw(upper_hex));
        drop(Box::from_raw(invalid_hex));
    }
}

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
