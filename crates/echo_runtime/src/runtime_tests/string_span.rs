use super::*;

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
