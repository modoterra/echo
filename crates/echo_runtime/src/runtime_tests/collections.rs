use super::*;
use crate::collections::EchoArrayKey;

mod keys;
mod search;
mod sequence;

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
fn array_merge_and_replace_builtins_preserve_php_key_behavior() {
    let mut base = echo_value_array_new();
    base = echo_value_array_set(base, test_string_value(b"sku"), test_string_value(b"A-42"));
    base = echo_value_array_set(base, EchoValue::int(7), test_string_value(b"old-bin"));
    base = echo_value_array_set(
        base,
        test_string_value(b"status"),
        test_string_value(b"draft"),
    );

    let mut override_row = echo_value_array_new();
    override_row = echo_value_array_set(
        override_row,
        test_string_value(b"status"),
        test_string_value(b"active"),
    );
    override_row = echo_value_array_set(
        override_row,
        EchoValue::int(4),
        test_string_value(b"new-bin"),
    );
    override_row = echo_value_array_set(
        override_row,
        test_string_value(b"owner"),
        test_string_value(b"maya"),
    );

    let mut extra = echo_value_array_new();
    extra = echo_value_array_set(extra, test_string_value(b"sku"), test_string_value(b"A-43"));
    extra = echo_value_array_set(extra, EchoValue::int(9), test_string_value(b"late"));

    let mut args = echo_value_array_new();
    args = echo_value_array_append(args, base);
    args = echo_value_array_append(args, override_row);
    args = echo_value_array_append(args, extra);

    let merged = echo_php_array_merge(args);
    let merged_ref = unsafe { (merged.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        merged_ref.keys,
        vec![
            EchoArrayKey::String(b"sku".to_vec()),
            EchoArrayKey::Int(0),
            EchoArrayKey::String(b"status".to_vec()),
            EchoArrayKey::Int(1),
            EchoArrayKey::String(b"owner".to_vec()),
            EchoArrayKey::Int(2),
        ]
    );
    assert_eq!(merged_ref.values[0].string_bytes(), Some(b"A-43".to_vec()));
    assert_eq!(
        merged_ref.values[1].string_bytes(),
        Some(b"old-bin".to_vec())
    );
    assert_eq!(
        merged_ref.values[2].string_bytes(),
        Some(b"active".to_vec())
    );
    assert_eq!(
        merged_ref.values[3].string_bytes(),
        Some(b"new-bin".to_vec())
    );
    assert_eq!(merged_ref.values[4].string_bytes(), Some(b"maya".to_vec()));
    assert_eq!(merged_ref.values[5].string_bytes(), Some(b"late".to_vec()));

    let replaced = echo_php_array_replace(args);
    let replaced_ref = unsafe { (replaced.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        replaced_ref.keys,
        vec![
            EchoArrayKey::String(b"sku".to_vec()),
            EchoArrayKey::Int(7),
            EchoArrayKey::String(b"status".to_vec()),
            EchoArrayKey::Int(4),
            EchoArrayKey::String(b"owner".to_vec()),
            EchoArrayKey::Int(9),
        ]
    );
    assert_eq!(
        replaced_ref.values[0].string_bytes(),
        Some(b"A-43".to_vec())
    );
    assert_eq!(
        replaced_ref.values[1].string_bytes(),
        Some(b"old-bin".to_vec())
    );
    assert_eq!(
        replaced_ref.values[2].string_bytes(),
        Some(b"active".to_vec())
    );

    let empty = echo_php_array_merge(echo_value_array_new());
    let empty_ref = unsafe { (empty.payload as *const EchoArray).as_ref() }.expect("array");
    assert!(empty_ref.keys.is_empty());
    assert_eq!(
        echo_php_array_replace(echo_value_array_new()).kind,
        ECHO_VALUE_ERROR
    );
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
