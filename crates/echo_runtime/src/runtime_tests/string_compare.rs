use super::*;

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
    assert_eq!(
        echo_php_strcoll(EchoValue::string(less_left), EchoValue::string(less_right)),
        EchoValue::int(-1)
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
fn natural_compare_builtins_preserve_php_number_aware_ordering() {
    assert_eq!(
        echo_php_strnatcmp(test_string_value(b"img2"), test_string_value(b"img10")),
        EchoValue::int(-1)
    );
    assert_eq!(
        echo_php_strnatcmp(test_string_value(b"img10"), test_string_value(b"img2")),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strnatcmp(test_string_value(b"img10"), test_string_value(b"img10")),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strnatcmp(test_string_value(b"Image2"), test_string_value(b"image2")),
        EchoValue::int(-1)
    );
    assert_eq!(
        echo_php_strnatcasecmp(test_string_value(b"Image2"), test_string_value(b"image2")),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strnatcasecmp(test_string_value(b"file9"), test_string_value(b"file10")),
        EchoValue::int(-1)
    );
}

#[test]
fn levenshtein_preserves_php_byte_distance_and_costs() {
    assert_eq!(
        echo_php_levenshtein(
            test_string_value(b"kitten"),
            test_string_value(b"sitting"),
            EchoValue::int(1),
            EchoValue::int(1),
            EchoValue::int(1),
        ),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_levenshtein(
            test_string_value(b"abc"),
            test_string_value(b"adc"),
            EchoValue::int(1),
            EchoValue::int(5),
            EchoValue::int(1),
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_levenshtein(
            test_string_value(b""),
            test_string_value(b"echo"),
            EchoValue::int(1),
            EchoValue::int(1),
            EchoValue::int(1),
        ),
        EchoValue::int(4)
    );
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
