use super::*;

#[test]
fn string_case_builtins_convert_only_ascii_bytes() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: "Echo äÖ 123!".as_bytes().to_vec(),
    }));
    let value = EchoValue::string(string);

    assert_eq!(
        echo_php_strtoupper(value).string_bytes(),
        Some("ECHO äÖ 123!".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strtolower(value).string_bytes(),
        Some("echo äÖ 123!".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(string));
    }
}

#[test]
fn ucwords_preserves_php_default_separator_byte_behavior() {
    let words = Box::into_raw(Box::new(EchoString {
        bytes: "hello world".as_bytes().to_vec(),
    }));
    let tab = Box::into_raw(Box::new(EchoString {
        bytes: "hello\tworld".as_bytes().to_vec(),
    }));
    let hyphen = Box::into_raw(Box::new(EchoString {
        bytes: "hello-world".as_bytes().to_vec(),
    }));
    let mixed = Box::into_raw(Box::new(EchoString {
        bytes: "mIXed CASE".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ächo world".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_ucwords(EchoValue::string(words)).string_bytes(),
        Some("Hello World".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucwords(EchoValue::string(tab)).string_bytes(),
        Some("Hello\tWorld".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucwords(EchoValue::string(hyphen)).string_bytes(),
        Some("Hello-world".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucwords(EchoValue::string(mixed)).string_bytes(),
        Some("MIXed CASE".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucwords(EchoValue::string(non_ascii)).string_bytes(),
        Some("ächo World".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(words));
        drop(Box::from_raw(tab));
        drop(Box::from_raw(hyphen));
        drop(Box::from_raw(mixed));
        drop(Box::from_raw(non_ascii));
    }
}

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
fn float_scalar_math_builtins_preserve_php_scalar_behavior() {
    assert_float_value(echo_php_pi(), std::f64::consts::PI);
    assert_float_value(
        echo_php_fmod(EchoValue::float(5.7), EchoValue::float(1.3)),
        0.5,
    );
    assert_float_value(
        echo_php_fmod(EchoValue::float(-5.7), EchoValue::float(1.3)),
        -0.5,
    );
    assert!(f64::from_bits(echo_php_fmod(EchoValue::int(5), EchoValue::int(0)).payload).is_nan());
}

#[test]
fn string_unary_builtins_preserve_php_byte_behavior() {
    let reversed = Box::into_raw(Box::new(EchoString {
        bytes: "Echo ÄÖ 123!".as_bytes().to_vec(),
    }));
    let ucfirst = Box::into_raw(Box::new(EchoString {
        bytes: "echo".as_bytes().to_vec(),
    }));
    let lcfirst = Box::into_raw(Box::new(EchoString {
        bytes: "Echo".as_bytes().to_vec(),
    }));
    let non_ascii_first = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strrev(EchoValue::string(reversed)).string_bytes(),
        Some(vec![
            b'!', b'3', b'2', b'1', b' ', 0x96, 0xc3, 0x84, 0xc3, b' ', b'o', b'h', b'c', b'E'
        ])
    );
    assert_eq!(
        echo_php_ucfirst(EchoValue::string(ucfirst)).string_bytes(),
        Some("Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_lcfirst(EchoValue::string(lcfirst)).string_bytes(),
        Some("echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucfirst(EchoValue::string(non_ascii_first)).string_bytes(),
        Some("Ächo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_lcfirst(EchoValue::string(non_ascii_first)).string_bytes(),
        Some("Ächo".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(reversed));
        drop(Box::from_raw(ucfirst));
        drop(Box::from_raw(lcfirst));
        drop(Box::from_raw(non_ascii_first));
    }
}

#[test]
fn string_byte_builtins_preserve_php_byte_behavior() {
    let ascii = Box::into_raw(Box::new(EchoString {
        bytes: "A".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let rot13 = Box::into_raw(Box::new(EchoString {
        bytes: "Echo PHP 4.3.0 ÄÖ!".as_bytes().to_vec(),
    }));

    assert_eq!(echo_php_ord(EchoValue::string(ascii)), EchoValue::int(65));
    assert_eq!(
        echo_php_ord(EchoValue::string(non_ascii)),
        EchoValue::int(195)
    );
    assert_eq!(
        echo_php_str_rot13(EchoValue::string(rot13)).string_bytes(),
        Some("Rpub CUC 4.3.0 ÄÖ!".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(ascii));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(rot13));
    }
}

#[test]
fn base_string_conversion_builtins_preserve_php_byte_behavior() {
    assert_eq!(
        echo_php_chr(EchoValue::int(65)).string_bytes(),
        Some("A".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_chr(test_string_value(b"321")).string_bytes(),
        Some("A".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dechex(EchoValue::int(47)).string_bytes(),
        Some("2f".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dechex(EchoValue::int(-1)).string_bytes(),
        Some("ffffffffffffffff".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_decbin(EchoValue::int(26)).string_bytes(),
        Some("11010".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_decbin(EchoValue::int(-1)).string_bytes(),
        Some(
            "1111111111111111111111111111111111111111111111111111111111111111"
                .as_bytes()
                .to_vec()
        )
    );
    assert_eq!(
        echo_php_decoct(EchoValue::int(264)).string_bytes(),
        Some("410".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_decoct(EchoValue::int(-1)).string_bytes(),
        Some("1777777777777777777777".as_bytes().to_vec())
    );
}

#[test]
fn string_predicate_builtins_are_binary_safe_and_case_sensitive() {
    let haystack = Box::into_raw(Box::new(EchoString {
        bytes: "Echo PHP".as_bytes().to_vec(),
    }));
    let matching = Box::into_raw(Box::new(EchoString {
        bytes: "PHP".as_bytes().to_vec(),
    }));
    let mismatched_case = Box::into_raw(Box::new(EchoString {
        bytes: "php".as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let first_utf8_byte = Box::into_raw(Box::new(EchoString { bytes: vec![0xc3] }));

    assert_eq!(
        echo_php_str_contains(EchoValue::string(haystack), EchoValue::string(matching)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_str_contains(
            EchoValue::string(haystack),
            EchoValue::string(mismatched_case)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_str_contains(EchoValue::string(haystack), EchoValue::string(empty)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_str_starts_with(EchoValue::string(haystack), EchoValue::string(empty)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_str_ends_with(EchoValue::string(haystack), EchoValue::string(matching)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_str_contains(
            EchoValue::string(non_ascii),
            EchoValue::string(first_utf8_byte)
        ),
        EchoValue::bool(true)
    );

    unsafe {
        drop(Box::from_raw(haystack));
        drop(Box::from_raw(matching));
        drop(Box::from_raw(mismatched_case));
        drop(Box::from_raw(empty));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(first_utf8_byte));
    }
}
