use super::*;

#[test]
fn strval_preserves_php_scalar_string_coercion() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: "hello".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strval(EchoValue::string(string)).string_bytes(),
        Some("hello".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strval(EchoValue::int(42)).string_bytes(),
        Some("42".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strval(EchoValue::bool(true)).string_bytes(),
        Some("1".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strval(EchoValue::bool(false)).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_strval(EchoValue::null()).string_bytes(),
        Some(Vec::new())
    );

    unsafe {
        drop(Box::from_raw(string));
    }
}

#[test]
fn boolval_preserves_php_scalar_truthiness() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let zero = Box::into_raw(Box::new(EchoString {
        bytes: "0".as_bytes().to_vec(),
    }));
    let false_text = Box::into_raw(Box::new(EchoString {
        bytes: "false".as_bytes().to_vec(),
    }));

    assert_eq!(echo_php_boolval(EchoValue::null()), EchoValue::bool(false));
    assert_eq!(
        echo_php_boolval(EchoValue::bool(false)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_boolval(EchoValue::bool(true)),
        EchoValue::bool(true)
    );
    assert_eq!(echo_php_boolval(EchoValue::int(0)), EchoValue::bool(false));
    assert_eq!(echo_php_boolval(EchoValue::int(42)), EchoValue::bool(true));
    assert_eq!(
        echo_php_boolval(EchoValue::string(empty)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_boolval(EchoValue::string(zero)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_boolval(EchoValue::string(false_text)),
        EchoValue::bool(true)
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(zero));
        drop(Box::from_raw(false_text));
    }
}

#[test]
fn logical_not_uses_php_scalar_truthiness() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let zero = Box::into_raw(Box::new(EchoString {
        bytes: "0".as_bytes().to_vec(),
    }));
    let false_text = Box::into_raw(Box::new(EchoString {
        bytes: "false".as_bytes().to_vec(),
    }));

    assert_eq!(echo_value_not(EchoValue::null()), EchoValue::bool(true));
    assert_eq!(
        echo_value_not(EchoValue::bool(false)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_value_not(EchoValue::bool(true)),
        EchoValue::bool(false)
    );
    assert_eq!(echo_value_not(EchoValue::int(0)), EchoValue::bool(true));
    assert_eq!(echo_value_not(EchoValue::int(42)), EchoValue::bool(false));
    assert_eq!(
        echo_value_not(EchoValue::string(empty)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_value_not(EchoValue::string(zero)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_value_not(EchoValue::string(false_text)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(zero));
        drop(Box::from_raw(false_text));
    }
}

#[test]
fn intval_preserves_php_default_base_scalar_coercion() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let prefixed = Box::into_raw(Box::new(EchoString {
        bytes: "42abc".as_bytes().to_vec(),
    }));
    let spaced = Box::into_raw(Box::new(EchoString {
        bytes: "  15".as_bytes().to_vec(),
    }));
    let negative = Box::into_raw(Box::new(EchoString {
        bytes: "-7".as_bytes().to_vec(),
    }));
    let non_numeric = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));

    assert_eq!(echo_php_intval(EchoValue::null()), EchoValue::int(0));
    assert_eq!(echo_php_intval(EchoValue::bool(false)), EchoValue::int(0));
    assert_eq!(echo_php_intval(EchoValue::bool(true)), EchoValue::int(1));
    assert_eq!(echo_php_intval(EchoValue::int(42)), EchoValue::int(42));
    assert_eq!(echo_php_intval(EchoValue::string(empty)), EchoValue::int(0));
    assert_eq!(
        echo_php_intval(EchoValue::string(prefixed)),
        EchoValue::int(42)
    );
    assert_eq!(
        echo_php_intval(EchoValue::string(spaced)),
        EchoValue::int(15)
    );
    assert_eq!(
        echo_php_intval(EchoValue::string(negative)),
        EchoValue::int(-7)
    );
    assert_eq!(
        echo_php_intval(EchoValue::string(non_numeric)),
        EchoValue::int(0)
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(prefixed));
        drop(Box::from_raw(spaced));
        drop(Box::from_raw(negative));
        drop(Box::from_raw(non_numeric));
    }
}

#[test]
fn floatval_preserves_php_scalar_float_coercion() {
    assert_float_value(echo_php_floatval(EchoValue::null()), 0.0);
    assert_float_value(echo_php_floatval(EchoValue::bool(true)), 1.0);
    assert_float_value(echo_php_floatval(EchoValue::int(42)), 42.0);

    let prefixed = Box::into_raw(Box::new(EchoString {
        bytes: b"122.34343The".to_vec(),
    }));
    let invalid = Box::into_raw(Box::new(EchoString {
        bytes: b"The122.34343".to_vec(),
    }));
    let offset = Box::into_raw(Box::new(EchoString {
        bytes: b"  -12.5px".to_vec(),
    }));
    let exponent = Box::into_raw(Box::new(EchoString {
        bytes: b"1e2x".to_vec(),
    }));

    assert_float_value(echo_php_floatval(EchoValue::string(prefixed)), 122.34343);
    assert_float_value(echo_php_floatval(EchoValue::string(invalid)), 0.0);
    assert_float_value(echo_php_floatval(EchoValue::string(offset)), -12.5);
    assert_float_value(echo_php_floatval(EchoValue::string(exponent)), 100.0);

    unsafe {
        drop(Box::from_raw(prefixed));
        drop(Box::from_raw(invalid));
        drop(Box::from_raw(offset));
        drop(Box::from_raw(exponent));
    }
}

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
