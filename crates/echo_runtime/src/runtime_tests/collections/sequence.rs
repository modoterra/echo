use super::*;
use crate::collections::EchoArrayKey;

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
