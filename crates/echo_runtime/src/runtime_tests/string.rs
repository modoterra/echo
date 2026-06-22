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

#[test]
fn strpos_preserves_php_byte_behavior() {
    let found_at_zero = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let found_later = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_needle = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));
    let needle_later = Box::into_raw(Box::new(EchoString {
        bytes: "cd".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "c".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strpos(
            EchoValue::string(found_at_zero),
            EchoValue::string(needle_start)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strpos(
            EchoValue::string(found_later),
            EchoValue::string(needle_later)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strpos(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strpos(EchoValue::string(numeric_needle), EchoValue::int(34)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strpos(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::int(2)
    );

    unsafe {
        drop(Box::from_raw(found_at_zero));
        drop(Box::from_raw(found_later));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_later));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn stripos_preserves_php_ascii_case_insensitive_byte_behavior() {
    let found_at_zero = Box::into_raw(Box::new(EchoString {
        bytes: "ABC".as_bytes().to_vec(),
    }));
    let found_later = Box::into_raw(Box::new(EchoString {
        bytes: "xxEcho".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_needle = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let needle_later = Box::into_raw(Box::new(EchoString {
        bytes: "ECHO".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "XY".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ä".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_stripos(
            EchoValue::string(found_at_zero),
            EchoValue::string(needle_start)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_stripos(
            EchoValue::string(found_later),
            EchoValue::string(needle_later)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_stripos(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_stripos(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_stripos(EchoValue::string(numeric_needle), EchoValue::int(34)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_stripos(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(found_at_zero));
        drop(Box::from_raw(found_later));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(numeric_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_later));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn strrpos_preserves_php_byte_behavior() {
    let repeated_start = Box::into_raw(Box::new(EchoString {
        bytes: "abcabc".as_bytes().to_vec(),
    }));
    let repeated_end = Box::into_raw(Box::new(EchoString {
        bytes: "abcabc".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_needle = Box::into_raw(Box::new(EchoString {
        bytes: "1234545".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächocho".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));
    let needle_end = Box::into_raw(Box::new(EchoString {
        bytes: "bc".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "c".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(repeated_start),
            EchoValue::string(needle_start)
        ),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(repeated_end),
            EchoValue::string(needle_end)
        ),
        EchoValue::int(4)
    );
    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::int(6)
    );
    assert_eq!(
        echo_php_strrpos(EchoValue::string(numeric_needle), EchoValue::int(45)),
        EchoValue::int(5)
    );
    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::int(5)
    );

    unsafe {
        drop(Box::from_raw(repeated_start));
        drop(Box::from_raw(repeated_end));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(numeric_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_end));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn strripos_preserves_php_ascii_case_insensitive_byte_behavior() {
    let repeated_start = Box::into_raw(Box::new(EchoString {
        bytes: "abABcd".as_bytes().to_vec(),
    }));
    let repeated_end = Box::into_raw(Box::new(EchoString {
        bytes: "abcABC".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_needle = Box::into_raw(Box::new(EchoString {
        bytes: "1234545".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "aB".as_bytes().to_vec(),
    }));
    let needle_end = Box::into_raw(Box::new(EchoString {
        bytes: "BC".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "XY".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ä".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strripos(
            EchoValue::string(repeated_start),
            EchoValue::string(needle_start)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strripos(
            EchoValue::string(repeated_end),
            EchoValue::string(needle_end)
        ),
        EchoValue::int(4)
    );
    assert_eq!(
        echo_php_strripos(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strripos(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::int(6)
    );
    assert_eq!(
        echo_php_strripos(EchoValue::string(numeric_needle), EchoValue::int(45)),
        EchoValue::int(5)
    );
    assert_eq!(
        echo_php_strripos(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(repeated_start));
        drop(Box::from_raw(repeated_end));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(numeric_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_end));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn strstr_preserves_php_byte_behavior() {
    let email = Box::into_raw(Box::new(EchoString {
        bytes: "name@example.com".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let at_start = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_at = Box::into_raw(Box::new(EchoString {
        bytes: "@".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "c".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strstr(EchoValue::string(email), EchoValue::string(needle_at)).string_bytes(),
        Some("@example.com".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strstr(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strstr(EchoValue::string(at_start), EchoValue::string(needle_start))
            .string_bytes(),
        Some("abcdef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strstr(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
        Some("345".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strstr(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        )
        .string_bytes(),
        Some("abcdef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strstr(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        )
        .string_bytes(),
        Some("cho".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(email));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(at_start));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_at));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn stristr_preserves_php_ascii_case_insensitive_byte_behavior() {
    let email = Box::into_raw(Box::new(EchoString {
        bytes: "USER@EXAMPLE.com".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let at_start = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_email = Box::into_raw(Box::new(EchoString {
        bytes: "e".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "XY".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "AB".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ä".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_stristr(EchoValue::string(email), EchoValue::string(needle_email)).string_bytes(),
        Some("ER@EXAMPLE.com".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_stristr(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_stristr(EchoValue::string(at_start), EchoValue::string(needle_start))
            .string_bytes(),
        Some("abcdef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_stristr(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
        Some("345".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_stristr(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        )
        .string_bytes(),
        Some("abcdef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_stristr(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(email));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(at_start));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_email));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn strrchr_preserves_php_byte_behavior() {
    let email = Box::into_raw(Box::new(EchoString {
        bytes: "name@example.com".as_bytes().to_vec(),
    }));
    let repeated = Box::into_raw(Box::new(EchoString {
        bytes: "abcabc".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "1234545".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ÄchoÄ".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let needle_at = Box::into_raw(Box::new(EchoString {
        bytes: "@".as_bytes().to_vec(),
    }));
    let needle_repeated = Box::into_raw(Box::new(EchoString {
        bytes: "bc".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_strrchr(EchoValue::string(email), EchoValue::string(needle_at)).string_bytes(),
        Some("@example.com".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strrchr(
            EchoValue::string(repeated),
            EchoValue::string(needle_repeated)
        )
        .string_bytes(),
        Some("bc".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strrchr(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strrchr(EchoValue::string(numeric), EchoValue::int(45)).string_bytes(),
        Some("45".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strrchr(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        )
        .string_bytes(),
        Some("Ä".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strrchr(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(email));
        drop(Box::from_raw(repeated));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(needle_at));
        drop(Box::from_raw(needle_repeated));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_non_ascii));
        drop(Box::from_raw(needle_empty));
    }
}

#[test]
fn strpbrk_preserves_php_byte_mask_behavior() {
    let text = Box::into_raw(Box::new(EchoString {
        bytes: "This is a Simple text.".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let empty_mask = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let mask_text = Box::into_raw(Box::new(EchoString {
        bytes: "mi".as_bytes().to_vec(),
    }));
    let mask_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let mask_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_strpbrk(EchoValue::string(text), EchoValue::string(mask_text)).string_bytes(),
        Some("is is a Simple text.".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strpbrk(EchoValue::string(missing), EchoValue::string(mask_missing)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strpbrk(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
        Some("345".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strpbrk(
            EchoValue::string(non_ascii),
            EchoValue::string(mask_non_ascii)
        )
        .string_bytes(),
        Some("Ächo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strpbrk(EchoValue::string(empty_mask), EchoValue::string(mask_empty)),
        EchoValue::error()
    );

    unsafe {
        drop(Box::from_raw(text));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty_mask));
        drop(Box::from_raw(mask_text));
        drop(Box::from_raw(mask_missing));
        drop(Box::from_raw(mask_non_ascii));
        drop(Box::from_raw(mask_empty));
    }
}

#[test]
fn strspn_preserves_php_byte_mask_behavior() {
    let digits = Box::into_raw(Box::new(EchoString {
        bytes: "42 is the answer".as_bytes().to_vec(),
    }));
    let prefix = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let mask_digits = Box::into_raw(Box::new(EchoString {
        bytes: "0123456789".as_bytes().to_vec(),
    }));
    let mask_prefix = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let mask_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xyz".as_bytes().to_vec(),
    }));
    let mask_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Äc".as_bytes().to_vec(),
    }));
    let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_strspn(EchoValue::string(digits), EchoValue::string(mask_digits)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strspn(EchoValue::string(prefix), EchoValue::string(mask_prefix)),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_strspn(EchoValue::string(missing), EchoValue::string(mask_missing)),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strspn(EchoValue::string(numeric), EchoValue::int(12)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strspn(
            EchoValue::string(non_ascii),
            EchoValue::string(mask_non_ascii)
        ),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_strspn(EchoValue::string(empty), EchoValue::string(mask_empty)),
        EchoValue::int(0)
    );

    unsafe {
        drop(Box::from_raw(digits));
        drop(Box::from_raw(prefix));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty));
        drop(Box::from_raw(mask_digits));
        drop(Box::from_raw(mask_prefix));
        drop(Box::from_raw(mask_missing));
        drop(Box::from_raw(mask_non_ascii));
        drop(Box::from_raw(mask_empty));
    }
}

#[test]
fn strcspn_preserves_php_byte_mask_behavior() {
    let no_match = Box::into_raw(Box::new(EchoString {
        bytes: "abcd".as_bytes().to_vec(),
    }));
    let end_match = Box::into_raw(Box::new(EchoString {
        bytes: "abcd".as_bytes().to_vec(),
    }));
    let middle_match = Box::into_raw(Box::new(EchoString {
        bytes: "abcd".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let mask_no_match = Box::into_raw(Box::new(EchoString {
        bytes: "x".as_bytes().to_vec(),
    }));
    let mask_end_match = Box::into_raw(Box::new(EchoString {
        bytes: "d".as_bytes().to_vec(),
    }));
    let mask_middle_match = Box::into_raw(Box::new(EchoString {
        bytes: "bd".as_bytes().to_vec(),
    }));
    let mask_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "c".as_bytes().to_vec(),
    }));
    let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_strcspn(
            EchoValue::string(no_match),
            EchoValue::string(mask_no_match)
        ),
        EchoValue::int(4)
    );
    assert_eq!(
        echo_php_strcspn(
            EchoValue::string(end_match),
            EchoValue::string(mask_end_match)
        ),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_strcspn(
            EchoValue::string(middle_match),
            EchoValue::string(mask_middle_match)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcspn(EchoValue::string(numeric), EchoValue::int(34)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strcspn(
            EchoValue::string(non_ascii),
            EchoValue::string(mask_non_ascii)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strcspn(EchoValue::string(empty), EchoValue::string(mask_empty)),
        EchoValue::int(3)
    );

    unsafe {
        drop(Box::from_raw(no_match));
        drop(Box::from_raw(end_match));
        drop(Box::from_raw(middle_match));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty));
        drop(Box::from_raw(mask_no_match));
        drop(Box::from_raw(mask_end_match));
        drop(Box::from_raw(mask_middle_match));
        drop(Box::from_raw(mask_non_ascii));
        drop(Box::from_raw(mask_empty));
    }
}

#[test]
fn substr_count_preserves_php_non_overlapping_byte_behavior() {
    let words = Box::into_raw(Box::new(EchoString {
        bytes: "This is a test".as_bytes().to_vec(),
    }));
    let repeated = Box::into_raw(Box::new(EchoString {
        bytes: "aaaa".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "1234512345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ÄchoÄ".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let needle_words = Box::into_raw(Box::new(EchoString {
        bytes: "is".as_bytes().to_vec(),
    }));
    let needle_repeated = Box::into_raw(Box::new(EchoString {
        bytes: "aa".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_substr_count(EchoValue::string(words), EchoValue::string(needle_words)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_substr_count(
            EchoValue::string(repeated),
            EchoValue::string(needle_repeated)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_substr_count(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_substr_count(EchoValue::string(numeric), EchoValue::int(45)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_substr_count(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_substr_count(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::error()
    );

    unsafe {
        drop(Box::from_raw(words));
        drop(Box::from_raw(repeated));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(needle_words));
        drop(Box::from_raw(needle_repeated));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_non_ascii));
        drop(Box::from_raw(needle_empty));
    }
}

#[test]
fn strcmp_preserves_php_byte_sign_behavior() {
    let less_left = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let less_right = Box::into_raw(Box::new(EchoString {
        bytes: "b".as_bytes().to_vec(),
    }));
    let greater_left = Box::into_raw(Box::new(EchoString {
        bytes: "b".as_bytes().to_vec(),
    }));
    let greater_right = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let equal_left = Box::into_raw(Box::new(EchoString {
        bytes: "same".as_bytes().to_vec(),
    }));
    let equal_right = Box::into_raw(Box::new(EchoString {
        bytes: "same".as_bytes().to_vec(),
    }));
    let prefix_left = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let prefix_right = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));
    let numeric_left = Box::into_raw(Box::new(EchoString {
        bytes: "123".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strcmp(EchoValue::string(less_left), EchoValue::string(less_right)),
        EchoValue::int(-1)
    );
    assert_eq!(
        echo_php_strcmp(
            EchoValue::string(greater_left),
            EchoValue::string(greater_right)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcmp(
            EchoValue::string(equal_left),
            EchoValue::string(equal_right)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strcmp(
            EchoValue::string(prefix_left),
            EchoValue::string(prefix_right)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcmp(EchoValue::string(numeric_left), EchoValue::int(123)),
        EchoValue::int(0)
    );

    unsafe {
        drop(Box::from_raw(less_left));
        drop(Box::from_raw(less_right));
        drop(Box::from_raw(greater_left));
        drop(Box::from_raw(greater_right));
        drop(Box::from_raw(equal_left));
        drop(Box::from_raw(equal_right));
        drop(Box::from_raw(prefix_left));
        drop(Box::from_raw(prefix_right));
        drop(Box::from_raw(numeric_left));
    }
}

#[test]
fn strcasecmp_preserves_php_ascii_case_insensitive_behavior() {
    let equal_left = Box::into_raw(Box::new(EchoString {
        bytes: "Echo".as_bytes().to_vec(),
    }));
    let equal_right = Box::into_raw(Box::new(EchoString {
        bytes: "echo".as_bytes().to_vec(),
    }));
    let less_left = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let less_right = Box::into_raw(Box::new(EchoString {
        bytes: "B".as_bytes().to_vec(),
    }));
    let greater_left = Box::into_raw(Box::new(EchoString {
        bytes: "B".as_bytes().to_vec(),
    }));
    let greater_right = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let prefix_left = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let prefix_right = Box::into_raw(Box::new(EchoString {
        bytes: "AB".as_bytes().to_vec(),
    }));
    let numeric_left = Box::into_raw(Box::new(EchoString {
        bytes: "123".as_bytes().to_vec(),
    }));
    let non_ascii_left = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let non_ascii_right = Box::into_raw(Box::new(EchoString {
        bytes: "ä".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strcasecmp(
            EchoValue::string(equal_left),
            EchoValue::string(equal_right)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strcasecmp(EchoValue::string(less_left), EchoValue::string(less_right)),
        EchoValue::int(-1)
    );
    assert_eq!(
        echo_php_strcasecmp(
            EchoValue::string(greater_left),
            EchoValue::string(greater_right)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcasecmp(
            EchoValue::string(prefix_left),
            EchoValue::string(prefix_right)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcasecmp(EchoValue::string(numeric_left), EchoValue::int(123)),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strcasecmp(
            EchoValue::string(non_ascii_left),
            EchoValue::string(non_ascii_right)
        ),
        EchoValue::int(-32)
    );

    unsafe {
        drop(Box::from_raw(equal_left));
        drop(Box::from_raw(equal_right));
        drop(Box::from_raw(less_left));
        drop(Box::from_raw(less_right));
        drop(Box::from_raw(greater_left));
        drop(Box::from_raw(greater_right));
        drop(Box::from_raw(prefix_left));
        drop(Box::from_raw(prefix_right));
        drop(Box::from_raw(numeric_left));
        drop(Box::from_raw(non_ascii_left));
        drop(Box::from_raw(non_ascii_right));
    }
}

#[test]
fn strncmp_builtins_preserve_php_prefix_behavior() {
    let abc = Box::into_raw(Box::new(EchoString {
        bytes: b"abc".to_vec(),
    }));
    let abd = Box::into_raw(Box::new(EchoString {
        bytes: b"abd".to_vec(),
    }));
    let ab = Box::into_raw(Box::new(EchoString {
        bytes: b"ab".to_vec(),
    }));
    let upper_abd = Box::into_raw(Box::new(EchoString {
        bytes: b"ABD".to_vec(),
    }));

    assert_eq!(
        echo_php_strncmp(
            EchoValue::string(abc),
            EchoValue::string(abd),
            EchoValue::int(2)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strncmp(
            EchoValue::string(abc),
            EchoValue::string(abd),
            EchoValue::int(3)
        ),
        EchoValue::int(-1)
    );
    assert_eq!(
        echo_php_strncmp(
            EchoValue::string(abc),
            EchoValue::string(ab),
            EchoValue::int(3)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strncasecmp(
            EchoValue::string(abc),
            EchoValue::string(upper_abd),
            EchoValue::int(2)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strncasecmp(
            EchoValue::string(abc),
            EchoValue::string(upper_abd),
            EchoValue::int(3)
        ),
        EchoValue::int(-1)
    );

    unsafe {
        drop(Box::from_raw(abc));
        drop(Box::from_raw(abd));
        drop(Box::from_raw(ab));
        drop(Box::from_raw(upper_abd));
    }
}

#[test]
fn substr_compare_preserves_php_offset_length_and_case_behavior() {
    let haystack = Box::into_raw(Box::new(EchoString {
        bytes: b"abcde".to_vec(),
    }));
    let needle_bc = Box::into_raw(Box::new(EchoString {
        bytes: b"bc".to_vec(),
    }));
    let needle_bcg = Box::into_raw(Box::new(EchoString {
        bytes: b"bcg".to_vec(),
    }));
    let needle_upper_bc = Box::into_raw(Box::new(EchoString {
        bytes: b"BC".to_vec(),
    }));
    let needle_cd = Box::into_raw(Box::new(EchoString {
        bytes: b"cd".to_vec(),
    }));

    assert_eq!(
        echo_php_substr_compare(
            EchoValue::string(haystack),
            EchoValue::string(needle_bc),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::bool(false)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_substr_compare(
            EchoValue::string(haystack),
            EchoValue::string(needle_bcg),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::bool(false)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_substr_compare(
            EchoValue::string(haystack),
            EchoValue::string(needle_upper_bc),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::bool(true)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_substr_compare(
            EchoValue::string(haystack),
            EchoValue::string(needle_cd),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::bool(false)
        ),
        EchoValue::int(-1)
    );

    unsafe {
        drop(Box::from_raw(haystack));
        drop(Box::from_raw(needle_bc));
        drop(Box::from_raw(needle_bcg));
        drop(Box::from_raw(needle_upper_bc));
        drop(Box::from_raw(needle_cd));
    }
}

#[test]
fn trim_builtins_strip_default_php_ascii_whitespace() {
    let trim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let ltrim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let rtrim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: " Ä ".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_trim(EchoValue::string(trim)).string_bytes(),
        Some("Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ltrim(EchoValue::string(ltrim)).string_bytes(),
        Some("Echo \n".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_rtrim(EchoValue::string(rtrim)).string_bytes(),
        Some("\t Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_trim(EchoValue::string(non_ascii)).string_bytes(),
        Some("Ä".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(trim));
        drop(Box::from_raw(ltrim));
        drop(Box::from_raw(rtrim));
        drop(Box::from_raw(non_ascii));
    }
}

#[test]
fn string_rewrite_builtins_preserve_php_byte_behavior() {
    let chop = Box::into_raw(Box::new(EchoString {
        bytes: b"invoice:1001\n".to_vec(),
    }));
    let quoted = Box::into_raw(Box::new(EchoString {
        bytes: b"a=b\nnext".to_vec(),
    }));
    let quoted_decode = Box::into_raw(Box::new(EchoString {
        bytes: b"a=3Db=0Anext".to_vec(),
    }));
    let nl2br = Box::into_raw(Box::new(EchoString {
        bytes: b"line1\nline2".to_vec(),
    }));
    let search = Box::into_raw(Box::new(EchoString {
        bytes: b"{{name}}".to_vec(),
    }));
    let replace = Box::into_raw(Box::new(EchoString {
        bytes: b"Ada".to_vec(),
    }));
    let subject = Box::into_raw(Box::new(EchoString {
        bytes: b"Hello {{name}}".to_vec(),
    }));
    let isearch = Box::into_raw(Box::new(EchoString {
        bytes: b"TOKEN".to_vec(),
    }));
    let ireplace = Box::into_raw(Box::new(EchoString {
        bytes: b"redacted".to_vec(),
    }));
    let isubject = Box::into_raw(Box::new(EchoString {
        bytes: b"token TOKEN".to_vec(),
    }));
    let tr_value = Box::into_raw(Box::new(EchoString {
        bytes: b"abc-123".to_vec(),
    }));
    let tr_from = Box::into_raw(Box::new(EchoString {
        bytes: b"abc123".to_vec(),
    }));
    let tr_to = Box::into_raw(Box::new(EchoString {
        bytes: b"xyz789".to_vec(),
    }));

    assert_eq!(
        echo_php_rtrim(EchoValue::string(chop)).string_bytes(),
        Some(b"invoice:1001".to_vec())
    );
    assert_eq!(
        echo_php_quoted_printable_encode(EchoValue::string(quoted)).string_bytes(),
        Some(b"a=3Db=0Anext".to_vec())
    );
    assert_eq!(
        echo_php_quoted_printable_decode(EchoValue::string(quoted_decode)).string_bytes(),
        Some(b"a=b\nnext".to_vec())
    );
    assert_eq!(
        echo_php_nl2br(EchoValue::string(nl2br), EchoValue::bool(false)).string_bytes(),
        Some(b"line1<br>\nline2".to_vec())
    );
    assert_eq!(
        echo_php_str_replace(
            EchoValue::string(search),
            EchoValue::string(replace),
            EchoValue::string(subject),
        )
        .string_bytes(),
        Some(b"Hello Ada".to_vec())
    );
    assert_eq!(
        echo_php_str_ireplace(
            EchoValue::string(isearch),
            EchoValue::string(ireplace),
            EchoValue::string(isubject),
        )
        .string_bytes(),
        Some(b"redacted redacted".to_vec())
    );
    assert_eq!(
        echo_php_strtr(
            EchoValue::string(tr_value),
            EchoValue::string(tr_from),
            EchoValue::string(tr_to),
        )
        .string_bytes(),
        Some(b"xyz-789".to_vec())
    );

    unsafe {
        drop(Box::from_raw(chop));
        drop(Box::from_raw(quoted));
        drop(Box::from_raw(quoted_decode));
        drop(Box::from_raw(nl2br));
        drop(Box::from_raw(search));
        drop(Box::from_raw(replace));
        drop(Box::from_raw(subject));
        drop(Box::from_raw(isearch));
        drop(Box::from_raw(ireplace));
        drop(Box::from_raw(isubject));
        drop(Box::from_raw(tr_value));
        drop(Box::from_raw(tr_from));
        drop(Box::from_raw(tr_to));
    }
}
