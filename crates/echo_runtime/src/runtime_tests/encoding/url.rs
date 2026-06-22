use super::*;

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
