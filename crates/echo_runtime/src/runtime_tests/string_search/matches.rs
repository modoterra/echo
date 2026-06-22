use super::*;

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
