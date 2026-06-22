use super::*;

#[test]
fn bin2hex_preserves_php_byte_behavior() {
    assert_eq!(
        echo_php_bin2hex(test_string_value(b"Echo")).string_bytes(),
        Some("4563686f".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_bin2hex(test_string_value("Ä".as_bytes())).string_bytes(),
        Some("c384".as_bytes().to_vec())
    );
}

#[test]
fn checksum_builtins_preserve_php_byte_behavior() {
    assert_eq!(
        echo_php_crc32(test_string_value(b"Echo\nPHP")),
        EchoValue::int(286159390)
    );
    assert_eq!(
        echo_php_md5(test_string_value(b"Echo\nPHP"), EchoValue::bool(false)).string_bytes(),
        Some("d4f2cb8de8248adb1e54f021bcd5e8c2".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_md5(test_string_value(b"Echo\nPHP"), EchoValue::bool(true)).string_bytes(),
        Some(vec![
            0xd4, 0xf2, 0xcb, 0x8d, 0xe8, 0x24, 0x8a, 0xdb, 0x1e, 0x54, 0xf0, 0x21, 0xbc, 0xd5,
            0xe8, 0xc2,
        ])
    );
    assert_eq!(
        echo_php_sha1(test_string_value(b"Echo\nPHP"), EchoValue::bool(false)).string_bytes(),
        Some(
            "2ac003b31b44befef7f0c8b7e0154e3118689876"
                .as_bytes()
                .to_vec()
        )
    );
    assert_eq!(
        echo_php_sha1(test_string_value(b"Echo\nPHP"), EchoValue::bool(true)).string_bytes(),
        Some(vec![
            0x2a, 0xc0, 0x03, 0xb3, 0x1b, 0x44, 0xbe, 0xfe, 0xf7, 0xf0, 0xc8, 0xb7, 0xe0, 0x15,
            0x4e, 0x31, 0x18, 0x68, 0x98, 0x76,
        ])
    );
}

#[test]
fn base_to_decimal_builtins_preserve_php_unsigned_string_behavior() {
    assert_eq!(
        echo_php_bindec(test_string_value(b"1010")),
        EchoValue::int(10)
    );
    assert_eq!(
        echo_php_bindec(test_string_value(b"0b10xx11")),
        EchoValue::int(11)
    );
    assert_eq!(
        echo_php_hexdec(test_string_value(b"0xff")),
        EchoValue::int(255)
    );
    assert_eq!(
        echo_php_hexdec(test_string_value(b"ffzz10")),
        EchoValue::int(65296)
    );
    assert_eq!(
        echo_php_octdec(test_string_value(b"0789")),
        EchoValue::int(7)
    );
    assert_eq!(echo_php_bindec(EchoValue::int(10)), EchoValue::int(2));
    assert_eq!(echo_php_hexdec(EchoValue::float(10.7)), EchoValue::int(263));
    assert_eq!(echo_php_octdec(EchoValue::null()), EchoValue::int(0));
    assert_float_value(
        echo_php_hexdec(test_string_value(b"FFFFFFFFFFFFFFFF")),
        u64::MAX as f64,
    );
    assert_eq!(
        echo_php_base_convert(
            test_string_value(b"a37334"),
            EchoValue::int(16),
            EchoValue::int(2)
        )
        .string_bytes(),
        Some("101000110111001100110100".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base_convert(
            test_string_value(b"ffzz10"),
            EchoValue::int(16),
            EchoValue::int(10)
        )
        .string_bytes(),
        Some("65296".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base_convert(
            EchoValue::float(3.14),
            EchoValue::int(10),
            EchoValue::int(10)
        )
        .string_bytes(),
        Some("314".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base_convert(
            test_string_value(b"10"),
            EchoValue::int(1),
            EchoValue::int(10)
        ),
        EchoValue::error()
    );
}

#[test]
fn escapeshellarg_preserves_php_unix_shell_argument_quoting() {
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"Echo")).string_bytes(),
        Some("'Echo'".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"it's ready")).string_bytes(),
        Some("'it'\\''s ready'".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"")).string_bytes(),
        Some("''".as_bytes().to_vec())
    );
}

#[test]
fn escapeshellcmd_preserves_php_unix_shell_meta_escaping() {
    fn string_value(bytes: &[u8]) -> *mut EchoString {
        Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        }))
    }

    let semicolon = string_value(b"path; rm -rf /");
    let paired_double = string_value(b"echo \"ok\"");
    let unpaired_double = string_value(b"echo \"unterminated");
    let paired_single = string_value(b"echo 'ok'");
    let unpaired_single = string_value(b"echo 'unterminated");
    let newline = string_value(b"line\nbreak");
    let slash = string_value(b"a\\b");
    let dollar = string_value(b"a$b");

    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(semicolon)).string_bytes(),
        Some(b"path\\; rm -rf /".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(paired_double)).string_bytes(),
        Some(b"echo \"ok\"".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(unpaired_double)).string_bytes(),
        Some(b"echo \\\"unterminated".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(paired_single)).string_bytes(),
        Some(b"echo 'ok'".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(unpaired_single)).string_bytes(),
        Some(b"echo \\'unterminated".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(newline)).string_bytes(),
        Some(b"line\\\nbreak".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(slash)).string_bytes(),
        Some(b"a\\\\b".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(dollar)).string_bytes(),
        Some(b"a\\$b".to_vec())
    );

    unsafe {
        drop(Box::from_raw(semicolon));
        drop(Box::from_raw(paired_double));
        drop(Box::from_raw(unpaired_double));
        drop(Box::from_raw(paired_single));
        drop(Box::from_raw(unpaired_single));
        drop(Box::from_raw(newline));
        drop(Box::from_raw(slash));
        drop(Box::from_raw(dollar));
    }
}

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
fn string_escape_builtins_preserve_php_byte_behavior() {
    let quoted = Box::into_raw(Box::new(EchoString {
        bytes: vec![b'A', b'\'', b'"', b'\\', b'B'],
    }));
    let slashed_zero = Box::into_raw(Box::new(EchoString {
        bytes: b"\\0".to_vec(),
    }));
    let meta = Box::into_raw(Box::new(EchoString {
        bytes: b".\\+*?[^]($)".to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_addslashes(EchoValue::string(quoted)).string_bytes(),
        Some(b"A\\'\\\"\\\\B".to_vec())
    );
    assert_eq!(
        echo_php_stripslashes(EchoValue::string(slashed_zero)).string_bytes(),
        Some(vec![0])
    );
    assert_eq!(
        echo_php_quotemeta(EchoValue::string(meta)).string_bytes(),
        Some(b"\\.\\\\\\+\\*\\?\\[\\^\\]\\(\\$\\)".to_vec())
    );
    assert_eq!(
        echo_php_quotemeta(EchoValue::string(empty)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(quoted));
        drop(Box::from_raw(slashed_zero));
        drop(Box::from_raw(meta));
        drop(Box::from_raw(empty));
    }
}
