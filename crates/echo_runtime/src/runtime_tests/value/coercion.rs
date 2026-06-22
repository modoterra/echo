use super::*;

#[test]
fn is_numeric_preserves_php_numeric_string_rules() {
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: b" 1337e0 ".to_vec(),
    }));
    let decimal = Box::into_raw(Box::new(EchoString {
        bytes: b"4.2".to_vec(),
    }));
    let hex_prefixed = Box::into_raw(Box::new(EchoString {
        bytes: b"0x539".to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_is_numeric(EchoValue::int(42)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::string(numeric)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::string(decimal)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::string(hex_prefixed)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::string(empty)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::bool(true)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::null()),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(decimal));
        drop(Box::from_raw(hex_prefixed));
        drop(Box::from_raw(empty));
    }
}
