use super::*;

mod base64;
mod shell;
mod url;

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
fn utf8_encode_converts_latin1_bytes_to_utf8() {
    assert_eq!(
        echo_php_bin2hex(echo_php_utf8_encode(test_string_value(&[b'Z', b'o', 0xeb])))
            .string_bytes(),
        Some("5a6fc3ab".as_bytes().to_vec())
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
fn soundex_preserves_php_phonetic_key_examples() {
    for (word, key) in [
        (b"Euler".as_slice(), b"E460".as_slice()),
        (b"Ellery".as_slice(), b"E460".as_slice()),
        (b"Gauss".as_slice(), b"G200".as_slice()),
        (b"Ghosh".as_slice(), b"G200".as_slice()),
        (b"Hilbert".as_slice(), b"H416".as_slice()),
        (b"Heilbronn".as_slice(), b"H416".as_slice()),
        (b"Knuth".as_slice(), b"K530".as_slice()),
        (b"Kant".as_slice(), b"K530".as_slice()),
        (b"Lloyd".as_slice(), b"L300".as_slice()),
        (b"Ladd".as_slice(), b"L300".as_slice()),
        (b"Ashcraft".as_slice(), b"A261".as_slice()),
        (b"1Robert".as_slice(), b"R163".as_slice()),
        (b"1234".as_slice(), b"0000".as_slice()),
    ] {
        assert_eq!(
            echo_php_soundex(test_string_value(word)).string_bytes(),
            Some(key.to_vec())
        );
    }
}

#[test]
fn string_escape_builtins_preserve_php_byte_behavior() {
    let quoted = Box::into_raw(Box::new(EchoString {
        bytes: vec![b'A', b'\'', b'"', b'\\', b'B'],
    }));
    let slashed_zero = Box::into_raw(Box::new(EchoString {
        bytes: b"\\0".to_vec(),
    }));
    let c_escaped = Box::into_raw(Box::new(EchoString {
        bytes: b"\\n\\t\\x41\\101".to_vec(),
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
        echo_php_stripcslashes(EchoValue::string(c_escaped)).string_bytes(),
        Some(b"\n\tAA".to_vec())
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
        drop(Box::from_raw(c_escaped));
        drop(Box::from_raw(meta));
        drop(Box::from_raw(empty));
    }
}
