use super::*;
use crate::collections::EchoArrayKey;

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
fn array_slice_and_chunk_builtins_preserve_php_key_behavior() {
    let mut row = echo_value_array_new();
    row = echo_value_array_set(row, test_string_value(b"id"), EchoValue::int(101));
    row = echo_value_array_set(row, test_string_value(b"sku"), test_string_value(b"A-42"));
    row = echo_value_array_set(row, EchoValue::int(7), test_string_value(b"warehouse"));
    row = echo_value_array_set(
        row,
        test_string_value(b"status"),
        test_string_value(b"active"),
    );
    row = echo_value_array_set(row, EchoValue::int(8), test_string_value(b"late"));
    row = echo_value_array_set(row, test_string_value(b"owner"), test_string_value(b"maya"));

    let slice = echo_php_array_slice(
        row,
        EchoValue::int(1),
        EchoValue::int(-1),
        EchoValue::bool(false),
    );
    let slice_ref = unsafe { (slice.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        slice_ref.keys,
        vec![
            EchoArrayKey::String(b"sku".to_vec()),
            EchoArrayKey::Int(0),
            EchoArrayKey::String(b"status".to_vec()),
            EchoArrayKey::Int(1),
        ]
    );
    assert_eq!(slice_ref.values[0].string_bytes(), Some(b"A-42".to_vec()));
    assert_eq!(
        slice_ref.values[1].string_bytes(),
        Some(b"warehouse".to_vec())
    );
    assert_eq!(slice_ref.values[2].string_bytes(), Some(b"active".to_vec()));
    assert_eq!(slice_ref.values[3].string_bytes(), Some(b"late".to_vec()));

    let preserved = echo_php_array_slice(
        row,
        EchoValue::int(-4),
        EchoValue::int(3),
        EchoValue::bool(true),
    );
    let preserved_ref = unsafe { (preserved.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        preserved_ref.keys,
        vec![
            EchoArrayKey::Int(7),
            EchoArrayKey::String(b"status".to_vec()),
            EchoArrayKey::Int(8),
        ]
    );

    let chunks = echo_php_array_chunk(row, EchoValue::int(2), EchoValue::bool(false));
    let chunks_ref = unsafe { (chunks.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        chunks_ref.keys,
        vec![
            EchoArrayKey::Int(0),
            EchoArrayKey::Int(1),
            EchoArrayKey::Int(2)
        ]
    );
    let chunk_0 =
        unsafe { (chunks_ref.values[0].payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        chunk_0.keys,
        vec![EchoArrayKey::Int(0), EchoArrayKey::Int(1)]
    );
    assert_eq!(chunk_0.values[0], EchoValue::int(101));
    assert_eq!(chunk_0.values[1].string_bytes(), Some(b"A-42".to_vec()));
    let chunk_1 =
        unsafe { (chunks_ref.values[1].payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        chunk_1.keys,
        vec![EchoArrayKey::Int(0), EchoArrayKey::Int(1)]
    );
    assert_eq!(
        chunk_1.values[0].string_bytes(),
        Some(b"warehouse".to_vec())
    );
    assert_eq!(chunk_1.values[1].string_bytes(), Some(b"active".to_vec()));

    let preserved_chunks = echo_php_array_chunk(row, EchoValue::int(2), EchoValue::bool(true));
    let preserved_chunks_ref =
        unsafe { (preserved_chunks.payload as *const EchoArray).as_ref() }.expect("array");
    let preserved_chunk_1 =
        unsafe { (preserved_chunks_ref.values[1].payload as *const EchoArray).as_ref() }
            .expect("array");
    assert_eq!(
        preserved_chunk_1.keys,
        vec![
            EchoArrayKey::Int(7),
            EchoArrayKey::String(b"status".to_vec())
        ]
    );
    let preserved_chunk_2 =
        unsafe { (preserved_chunks_ref.values[2].payload as *const EchoArray).as_ref() }
            .expect("array");
    assert_eq!(
        preserved_chunk_2.values[0].string_bytes(),
        Some(b"late".to_vec())
    );
    assert_eq!(
        preserved_chunk_2.values[1].string_bytes(),
        Some(b"maya".to_vec())
    );

    assert_eq!(
        echo_php_array_chunk(row, EchoValue::int(0), EchoValue::bool(false)).kind,
        ECHO_VALUE_ERROR
    );
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
fn array_order_builtins_preserve_php_key_behavior() {
    let sku = test_string_value(b"sku");
    let qty = test_string_value(b"qty");
    let mut row = echo_value_array_new();
    row = echo_value_array_set(row, sku, test_string_value(b"A-42"));
    row = echo_value_array_set(row, EchoValue::int(7), test_string_value(b"seven"));
    row = echo_value_array_set(row, qty, test_string_value(b"2"));
    row = echo_value_array_set(row, EchoValue::int(10), test_string_value(b"ten"));

    let reversed = echo_php_array_reverse(row, EchoValue::bool(false));
    let reversed_ref = unsafe { (reversed.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        reversed_ref.keys,
        vec![
            EchoArrayKey::Int(0),
            EchoArrayKey::String(b"qty".to_vec()),
            EchoArrayKey::Int(1),
            EchoArrayKey::String(b"sku".to_vec())
        ]
    );
    assert_eq!(
        reversed_ref
            .values
            .iter()
            .map(|value| value.string_bytes().unwrap())
            .collect::<Vec<_>>(),
        vec![
            b"ten".to_vec(),
            b"2".to_vec(),
            b"seven".to_vec(),
            b"A-42".to_vec()
        ]
    );

    let preserved = echo_php_array_reverse(row, EchoValue::bool(true));
    let preserved_ref = unsafe { (preserved.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        preserved_ref.keys,
        vec![
            EchoArrayKey::Int(10),
            EchoArrayKey::String(b"qty".to_vec()),
            EchoArrayKey::Int(7),
            EchoArrayKey::String(b"sku".to_vec())
        ]
    );

    let mut map = echo_value_array_new();
    map = echo_value_array_set(map, test_string_value(b"first"), test_string_value(b"id"));
    map = echo_value_array_set(map, test_string_value(b"second"), test_string_value(b"qty"));
    map = echo_value_array_set(map, test_string_value(b"third"), test_string_value(b"id"));
    map = echo_value_array_set(map, test_string_value(b"num"), test_string_value(b"2"));
    map = echo_value_array_set(map, test_string_value(b"int"), EchoValue::int(5));
    map = echo_value_array_set(map, test_string_value(b"skip"), EchoValue::bool(true));

    let flipped = echo_php_array_flip(map);
    let flipped_ref = unsafe { (flipped.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        flipped_ref.keys,
        vec![
            EchoArrayKey::String(b"id".to_vec()),
            EchoArrayKey::String(b"qty".to_vec()),
            EchoArrayKey::Int(2),
            EchoArrayKey::Int(5)
        ]
    );
    assert_eq!(
        flipped_ref.values[0].string_bytes(),
        Some(b"third".to_vec())
    );
    assert_eq!(
        flipped_ref.values[1].string_bytes(),
        Some(b"second".to_vec())
    );
    assert_eq!(flipped_ref.values[2].string_bytes(), Some(b"num".to_vec()));
    assert_eq!(flipped_ref.values[3].string_bytes(), Some(b"int".to_vec()));
}

#[test]
fn array_search_and_count_values_preserve_php_value_behavior() {
    let mut row = echo_value_array_new();
    row = echo_value_array_set(row, test_string_value(b"sku"), test_string_value(b"A-42"));
    row = echo_value_array_set(row, EchoValue::int(7), test_string_value(b"A-42"));
    row = echo_value_array_set(row, test_string_value(b"qty"), EchoValue::int(4));
    row = echo_value_array_set(row, test_string_value(b"flag"), EchoValue::bool(true));
    row = echo_value_array_set(row, test_string_value(b"code"), test_string_value(b"4"));

    assert_eq!(
        echo_php_array_search(EchoValue::int(4), row, EchoValue::bool(false)).string_bytes(),
        Some(b"qty".to_vec())
    );
    assert_eq!(
        echo_php_array_search(EchoValue::int(4), row, EchoValue::bool(true)).string_bytes(),
        Some(b"qty".to_vec())
    );
    assert_eq!(
        echo_php_array_search(test_string_value(b"A-42"), row, EchoValue::bool(true))
            .string_bytes(),
        Some(b"sku".to_vec())
    );
    assert_eq!(
        echo_php_array_search(test_string_value(b"missing"), row, EchoValue::bool(true)),
        EchoValue::bool(false)
    );

    let mut values = echo_value_array_new();
    values = echo_value_array_append(values, test_string_value(b"new"));
    values = echo_value_array_append(values, test_string_value(b"new"));
    values = echo_value_array_append(values, test_string_value(b"done"));
    values = echo_value_array_append(values, EchoValue::int(2));
    values = echo_value_array_append(values, test_string_value(b"2"));
    values = echo_value_array_append(values, EchoValue::int(3));
    values = echo_value_array_append(values, EchoValue::bool(true));

    let counts = echo_php_array_count_values(values);
    let counts_ref = unsafe { (counts.payload as *const EchoArray).as_ref() }.expect("array");
    assert_eq!(
        counts_ref.keys,
        vec![
            EchoArrayKey::String(b"new".to_vec()),
            EchoArrayKey::String(b"done".to_vec()),
            EchoArrayKey::Int(2),
            EchoArrayKey::Int(3),
        ]
    );
    assert_eq!(
        counts_ref.values,
        vec![
            EchoValue::int(2),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::int(1)
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
    assert_eq!(
        echo_php_array_key_first(echo_value_array_new()),
        EchoValue::null()
    );
    assert_eq!(
        echo_php_array_key_last(echo_value_array_new()),
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

#[test]
fn repl_inspector_displays_array_values() {
    let key = test_string_value(b"name");
    let array = echo_value_array_set(echo_value_array_new(), key, test_string_value(b"Echo"));
    let array = echo_value_array_append(array, EchoValue::int(4));

    assert_eq!(
        array.inspect_bytes(),
        Some(br#"Array ["name" => "Echo", 0 => 4]"#.to_vec())
    );
}

#[test]
fn repl_inspector_truncates_large_arrays() {
    let mut array = echo_value_array_new();
    for value in 0..10 {
        array = echo_value_array_append(array, EchoValue::int(value));
    }

    assert_eq!(
        array.inspect_bytes(),
        Some(
            b"Array [0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7, ... 2 more]"
                .to_vec()
        )
    );
}

#[test]
fn runtime_capture_stdout_enables_repl_inspection_without_process_env() {
    let array = echo_value_array_append(echo_value_array_new(), EchoValue::int(2));

    let ((), stdout) = capture_stdout(true, || unsafe {
        echo_write_value(array);
    });

    assert_eq!(stdout, b"Array [0 => 2]");
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
