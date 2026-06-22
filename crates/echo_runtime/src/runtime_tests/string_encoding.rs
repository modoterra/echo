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
