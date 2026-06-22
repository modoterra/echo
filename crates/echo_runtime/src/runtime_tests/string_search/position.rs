use super::*;

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
