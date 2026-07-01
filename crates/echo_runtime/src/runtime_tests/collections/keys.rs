use super::*;
use crate::collections::EchoArrayKey;

#[test]
fn array_key_value_and_aggregate_builtins_preserve_php_array_behavior() {
    let id = test_string_value(b"id");
    let qty = test_string_value(b"qty");
    let string_two = test_string_value(b"2");
    let mut array = echo_value_array_new();
    array = echo_value_array_set(array, id, EchoValue::int(10));
    array = echo_value_array_set(array, qty, string_two);
    array = echo_value_array_set(array, EchoValue::int(5), EchoValue::int(2));

    let values = echo_php_array_values(array);
    let values_ref = unsafe { (values.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        values_ref.keys,
        vec![
            EchoArrayKey::Int(0),
            EchoArrayKey::Int(1),
            EchoArrayKey::Int(2)
        ]
    );
    assert_eq!(
        values_ref.values,
        vec![EchoValue::int(10), string_two, EchoValue::int(2)]
    );

    let keys = echo_php_array_keys(array, EchoValue::pending(), EchoValue::bool(false));
    let keys_ref = unsafe { (keys.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(keys_ref.values[0].string_bytes(), Some(b"id".to_vec()));
    assert_eq!(keys_ref.values[1].string_bytes(), Some(b"qty".to_vec()));
    assert_eq!(keys_ref.values[2], EchoValue::int(5));

    let loose = echo_php_array_keys(array, EchoValue::int(2), EchoValue::bool(false));
    let loose_ref = unsafe { (loose.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(loose_ref.values.len(), 2);
    assert_eq!(loose_ref.values[0].string_bytes(), Some(b"qty".to_vec()));
    assert_eq!(loose_ref.values[1], EchoValue::int(5));

    let strict = echo_php_array_keys(array, EchoValue::int(2), EchoValue::bool(true));
    let strict_ref = unsafe { (strict.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(strict_ref.values, vec![EchoValue::int(5)]);

    assert_eq!(echo_php_array_sum(array), EchoValue::int(14));
    assert_eq!(echo_php_array_product(array), EchoValue::int(40));
    assert_eq!(
        echo_php_array_sum(echo_value_array_new()),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_array_product(echo_value_array_new()),
        EchoValue::int(1)
    );
}

#[test]
fn array_fill_builtins_preserve_php_key_construction_behavior() {
    let fill = echo_php_array_fill(
        EchoValue::int(-2),
        EchoValue::int(4),
        test_string_value(b"pear"),
    );
    let fill_ref = unsafe { (fill.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        fill_ref.keys,
        vec![
            EchoArrayKey::Int(-2),
            EchoArrayKey::Int(-1),
            EchoArrayKey::Int(0),
            EchoArrayKey::Int(1)
        ]
    );
    assert!(
        fill_ref
            .values
            .iter()
            .all(|value| value.string_bytes() == Some(b"pear".to_vec()))
    );
    assert_eq!(
        echo_php_array_fill(EchoValue::int(0), EchoValue::int(-1), EchoValue::null()).kind,
        ECHO_VALUE_ERROR
    );

    let mut keys = echo_value_array_new();
    keys = echo_value_array_append(keys, test_string_value(b"sku"));
    keys = echo_value_array_append(keys, test_string_value(b"2"));
    keys = echo_value_array_append(keys, EchoValue::int(5));
    keys = echo_value_array_append(keys, EchoValue::bool(true));
    keys = echo_value_array_append(keys, EchoValue::null());
    keys = echo_value_array_append(keys, test_string_value(b"sku"));
    let keyed = echo_php_array_fill_keys(keys, test_string_value(b"todo"));
    let keyed_ref = unsafe { (keyed.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        keyed_ref.keys,
        vec![
            EchoArrayKey::String(b"sku".to_vec()),
            EchoArrayKey::Int(2),
            EchoArrayKey::Int(5),
            EchoArrayKey::Int(1),
            EchoArrayKey::String(Vec::new())
        ]
    );
    assert!(
        keyed_ref
            .values
            .iter()
            .all(|value| value.string_bytes() == Some(b"todo".to_vec()))
    );
}

#[test]
fn array_combine_and_pad_builtins_preserve_php_key_behavior() {
    let mut keys = echo_value_array_new();
    keys = echo_value_array_append(keys, test_string_value(b"sku"));
    keys = echo_value_array_append(keys, test_string_value(b"qty"));
    keys = echo_value_array_append(keys, test_string_value(b"qty"));
    keys = echo_value_array_append(keys, test_string_value(b"2"));

    let mut values = echo_value_array_new();
    values = echo_value_array_append(values, test_string_value(b"A-42"));
    values = echo_value_array_append(values, EchoValue::int(3));
    values = echo_value_array_append(values, EchoValue::int(4));
    values = echo_value_array_append(values, test_string_value(b"numeric"));

    let combined = echo_php_array_combine(keys, values);
    let combined_ref = unsafe { (combined.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        combined_ref.keys,
        vec![
            EchoArrayKey::String(b"sku".to_vec()),
            EchoArrayKey::String(b"qty".to_vec()),
            EchoArrayKey::Int(2),
        ]
    );
    assert_eq!(
        combined_ref.values[0].string_bytes(),
        Some(b"A-42".to_vec())
    );
    assert_eq!(combined_ref.values[1], EchoValue::int(4));
    assert_eq!(
        combined_ref.values[2].string_bytes(),
        Some(b"numeric".to_vec())
    );

    let mut row = echo_value_array_new();
    row = echo_value_array_set(row, test_string_value(b"sku"), test_string_value(b"A-42"));
    row = echo_value_array_set(row, EchoValue::int(7), test_string_value(b"seven"));
    row = echo_value_array_set(row, test_string_value(b"qty"), EchoValue::int(4));

    let right = echo_php_array_pad(row, EchoValue::int(5), test_string_value(b"missing"));
    let right_ref = unsafe { (right.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        right_ref.keys,
        vec![
            EchoArrayKey::String(b"sku".to_vec()),
            EchoArrayKey::Int(0),
            EchoArrayKey::String(b"qty".to_vec()),
            EchoArrayKey::Int(1),
            EchoArrayKey::Int(2),
        ]
    );

    let left = echo_php_array_pad(row, EchoValue::int(-5), test_string_value(b"missing"));
    let left_ref = unsafe { (left.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        left_ref.keys,
        vec![
            EchoArrayKey::Int(0),
            EchoArrayKey::Int(1),
            EchoArrayKey::String(b"sku".to_vec()),
            EchoArrayKey::Int(2),
            EchoArrayKey::String(b"qty".to_vec()),
        ]
    );

    let unchanged = echo_php_array_pad(row, EchoValue::int(2), test_string_value(b"noop"));
    let unchanged_ref = unsafe { (unchanged.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        unchanged_ref.keys,
        vec![
            EchoArrayKey::String(b"sku".to_vec()),
            EchoArrayKey::Int(7),
            EchoArrayKey::String(b"qty".to_vec()),
        ]
    );
}

#[test]
fn array_lookup_builtins_preserve_php_key_and_value_behavior() {
    let id = test_string_value(b"id");
    let qty = test_string_value(b"qty");
    let string_two = test_string_value(b"2");
    let zero = test_string_value(b"0");
    let mut array = echo_value_array_new();
    array = echo_value_array_set(array, id, EchoValue::int(10));
    array = echo_value_array_set(array, qty, string_two);
    array = echo_value_array_set(array, EchoValue::int(5), EchoValue::null());
    array = echo_value_array_set(array, zero, test_string_value(b"zero"));

    assert_eq!(
        echo_php_array_key_exists(test_string_value(b"id"), array),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_array_key_exists(EchoValue::int(5), array),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_array_key_exists(test_string_value(b"missing"), array),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_array_key_exists(EchoValue::bool(false), array),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_array_key_first(array).string_bytes(),
        Some(b"id".to_vec())
    );
    assert_eq!(echo_php_array_key_last(array), EchoValue::int(0));
    assert_eq!(echo_php_array_first(array), EchoValue::int(10));
    assert_eq!(
        echo_php_array_last(array).string_bytes(),
        Some(b"zero".to_vec())
    );
    assert_eq!(
        echo_php_array_key_first(echo_value_array_new()),
        EchoValue::null()
    );
    assert_eq!(
        echo_php_array_key_last(echo_value_array_new()),
        EchoValue::null()
    );
    assert_eq!(
        echo_php_array_first(echo_value_array_new()),
        EchoValue::null()
    );
    assert_eq!(
        echo_php_array_last(echo_value_array_new()),
        EchoValue::null()
    );
    assert_eq!(
        echo_php_in_array(EchoValue::int(2), array, EchoValue::bool(false)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_in_array(EchoValue::int(2), array, EchoValue::bool(true)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_in_array(string_two, array, EchoValue::bool(true)),
        EchoValue::bool(true)
    );
}
