use super::*;
use crate::collections::EchoArrayKey;

#[test]
fn url_encoding_builtins_preserve_php_byte_behavior() {
    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        })))
    }

    assert_eq!(
        echo_php_rawurlencode(string_value(b"sales and marketing/Miami~")).string_bytes(),
        Some(b"sales%20and%20marketing%2FMiami~".to_vec())
    );
    assert_eq!(
        echo_php_urlencode(string_value(b"Data123!@-_ +~")).string_bytes(),
        Some(b"Data123%21%40-_+%2B%7E".to_vec())
    );
    assert_eq!(
        echo_php_rawurldecode(string_value(b"foo%20bar%40baz+plus%ZZ")).string_bytes(),
        Some(b"foo bar@baz+plus%ZZ".to_vec())
    );
    assert_eq!(
        echo_php_urldecode(string_value(b"green+and+red%2Bblue%ZZ")).string_bytes(),
        Some(b"green and red+blue%ZZ".to_vec())
    );
    assert_eq!(
        echo_php_rawurldecode(echo_php_rawurlencode(string_value(b"a/b c+~"))).string_bytes(),
        Some(b"a/b c+~".to_vec())
    );
    assert_eq!(
        echo_php_rawurlencode(string_value(&[0xc3, 0x84])).string_bytes(),
        Some(b"%C3%84".to_vec())
    );
}

#[test]
fn parse_url_returns_common_full_url_parts() {
    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        })))
    }

    let parts = echo_php_parse_url(string_value(
        b"http://username:password@hostname:9090/path?arg=value#anchor",
    ));

    assert!(parts.is_array());
    assert_eq!(crate::echo_value_array_len(parts), 8);
    assert_eq!(
        crate::echo_value_array_key_at(parts, 0).string_bytes(),
        Some(b"scheme".to_vec())
    );
    assert_eq!(
        crate::echo_value_array_value_at(parts, 0).string_bytes(),
        Some(b"http".to_vec())
    );
    assert_eq!(
        crate::echo_value_array_key_at(parts, 2).string_bytes(),
        Some(b"port".to_vec())
    );
    assert_eq!(
        crate::echo_value_array_value_at(parts, 2),
        EchoValue::int(9090)
    );
}

#[test]
fn http_build_query_encodes_arrays_and_nested_arrays() {
    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        })))
    }

    let nested = EchoValue::array(Box::into_raw(Box::new(EchoArray {
        keys: vec![EchoArrayKey::Int(0), EchoArrayKey::Int(1)],
        values: vec![string_value(b"red apple"), string_value(b"blue")],
    })));
    let data = EchoValue::array(Box::into_raw(Box::new(EchoArray {
        keys: vec![
            EchoArrayKey::String(b"foo".to_vec()),
            EchoArrayKey::String(b"null".to_vec()),
            EchoArrayKey::String(b"items".to_vec()),
            EchoArrayKey::Int(0),
        ],
        values: vec![
            string_value(b"bar baz"),
            EchoValue::null(),
            nested,
            string_value(b"lead"),
        ],
    })));

    assert_eq!(
        echo_php_http_build_query(
            data,
            string_value(b"n_"),
            string_value(b"&"),
            EchoValue::int(1)
        )
        .string_bytes(),
        Some(b"foo=bar+baz&items%5B0%5D=red+apple&items%5B1%5D=blue&n_0=lead".to_vec())
    );
    assert_eq!(
        echo_php_http_build_query(
            data,
            string_value(b"n_"),
            string_value(b"&amp;"),
            EchoValue::int(1)
        )
        .string_bytes(),
        Some(b"foo=bar+baz&amp;items%5B0%5D=red+apple&amp;items%5B1%5D=blue&amp;n_0=lead".to_vec())
    );

    let rfc3986 = EchoValue::array(Box::into_raw(Box::new(EchoArray {
        keys: vec![EchoArrayKey::String(b"space".to_vec())],
        values: vec![string_value(b"a b")],
    })));
    assert_eq!(
        echo_php_http_build_query(
            rfc3986,
            string_value(b""),
            string_value(b"&"),
            EchoValue::int(2)
        )
        .string_bytes(),
        Some(b"space=a%20b".to_vec())
    );
}
