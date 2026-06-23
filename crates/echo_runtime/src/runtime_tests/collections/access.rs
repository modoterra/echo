use super::*;

#[test]
fn php_array_add_uses_union_semantics_for_numeric_keys() {
    let left = EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(vec![
        EchoValue::int(1),
        EchoValue::int(2),
    ]))));
    let right = EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(vec![
        EchoValue::int(3),
        EchoValue::int(4),
        EchoValue::int(5),
    ]))));

    let result = echo_value_add(left, right);
    let array = unsafe { (result.payload as *const EchoArray).as_ref() }.expect("array");

    assert_eq!(
        array.values,
        vec![EchoValue::int(1), EchoValue::int(2), EchoValue::int(5)]
    );
}

#[test]
fn php_array_add_uses_union_semantics_for_string_keys() {
    let left_key = test_string_value(b"a");
    let duplicate_key = test_string_value(b"a");
    let new_key = test_string_value(b"b");
    let left = echo_value_array_set(echo_value_array_new(), left_key, EchoValue::int(1));
    let right = echo_value_array_set(echo_value_array_new(), duplicate_key, EchoValue::int(2));
    let right = echo_value_array_set(right, new_key, EchoValue::int(3));

    let result = echo_value_add(left, right);
    let array = unsafe { (result.payload as *const EchoArray).as_ref() }.expect("array");

    assert_eq!(array.keys.len(), 2);
    assert_eq!(array.values, vec![EchoValue::int(1), EchoValue::int(3)]);
    assert_eq!(echo_php_array_is_list(result), EchoValue::bool(false));
}

#[test]
fn index_get_reads_array_and_list_values() {
    let array = echo_value_array_append(echo_value_array_new(), EchoValue::int(4));
    let key = test_string_value(b"name");
    let array = echo_value_array_set(array, key, test_string_value(b"Echo"));

    assert_eq!(
        echo_value_index_get(array, EchoValue::int(0)),
        EchoValue::int(4)
    );
    assert_eq!(
        echo_value_index_get(array, test_string_value(b"name")).string_bytes(),
        Some(b"Echo".to_vec())
    );

    let list = echo_value_list_append(echo_value_list_new(), EchoValue::int(7));
    assert_eq!(
        echo_value_index_get(list, EchoValue::int(0)),
        EchoValue::int(7)
    );
}

#[test]
fn index_get_returns_null_for_missing_values() {
    assert_eq!(
        echo_value_index_get(echo_value_array_new(), EchoValue::int(0)),
        EchoValue::null()
    );
    assert_eq!(
        echo_value_index_get(echo_value_list_new(), EchoValue::int(0)),
        EchoValue::null()
    );
}
