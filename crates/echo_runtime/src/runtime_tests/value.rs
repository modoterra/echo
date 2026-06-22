use super::*;

mod arithmetic;
mod coercion;
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
