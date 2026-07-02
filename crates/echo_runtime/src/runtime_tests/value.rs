use super::*;

mod arithmetic;
mod coercion;
mod inspect;
mod reflection;
mod types;

#[test]
fn lists_are_distinct_from_php_arrays() {
    let list = Box::into_raw(Box::new(EchoList {
        values: vec![EchoValue::int(1)],
    }));
    let value = EchoValue::list(list);

    assert_eq!(value.string_bytes(), Some(b"List".to_vec()));
    assert_eq!(echo_php_is_array(value), EchoValue::bool(false));
    assert_eq!(echo_php_is_countable(value), EchoValue::bool(true));
    assert_eq!(echo_php_is_iterable(value), EchoValue::bool(true));

    unsafe {
        drop(Box::from_raw(list));
    }
}

#[test]
fn arrays_are_distinct_from_lists() {
    let array = Box::into_raw(Box::new(EchoArray::from_values(vec![
        EchoValue::int(1),
        EchoValue::int(2),
    ])));
    let value = EchoValue::array(array);

    assert_eq!(value.string_bytes(), Some(b"Array".to_vec()));
    assert_eq!(
        echo_std_reflect_type_of(value).string_bytes(),
        Some(b"array".to_vec())
    );
    assert_eq!(
        echo_php_gettype(value).string_bytes(),
        Some(b"array".to_vec())
    );
    assert_eq!(echo_php_count(value), EchoValue::int(2));
    assert_eq!(echo_php_is_array(value), EchoValue::bool(true));
    assert_eq!(echo_php_is_countable(value), EchoValue::bool(true));
    assert_eq!(echo_php_is_iterable(value), EchoValue::bool(true));

    unsafe {
        drop(Box::from_raw(array));
    }
}

#[test]
fn serialize_preserves_scalar_and_array_shapes() {
    assert_eq!(
        echo_php_serialize(EchoValue::null()).string_bytes(),
        Some(b"N;".to_vec())
    );
    assert_eq!(
        echo_php_serialize(EchoValue::bool(true)).string_bytes(),
        Some(b"b:1;".to_vec())
    );
    assert_eq!(
        echo_php_serialize(EchoValue::bool(false)).string_bytes(),
        Some(b"b:0;".to_vec())
    );
    assert_eq!(
        echo_php_serialize(EchoValue::int(42)).string_bytes(),
        Some(b"i:42;".to_vec())
    );
    assert_eq!(
        echo_php_serialize(test_string_value(b"Echo")).string_bytes(),
        Some(b"s:4:\"Echo\";".to_vec())
    );

    let mut array = echo_value_array_new();
    array = echo_value_array_set(array, test_string_value(b"id"), EchoValue::int(42));
    array = echo_value_array_set(array, test_string_value(b"name"), test_string_value(b"Ada"));

    assert_eq!(
        echo_php_serialize(array).string_bytes(),
        Some(b"a:2:{s:2:\"id\";i:42;s:4:\"name\";s:3:\"Ada\";}".to_vec())
    );
}

#[test]
fn assert_intrinsics_report_success() {
    let left = Box::into_raw(Box::new(EchoString {
        bytes: b"same".to_vec(),
    }));
    let right = Box::into_raw(Box::new(EchoString {
        bytes: b"same".to_vec(),
    }));

    assert_eq!(
        echo_std_assert_ok(EchoValue::bool(true)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_std_assert_equals(EchoValue::int(42), EchoValue::int(42)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_std_assert_equals(EchoValue::string(left), EchoValue::string(right)),
        EchoValue::bool(true)
    );

    unsafe {
        drop(Box::from_raw(left));
        drop(Box::from_raw(right));
    }
}
