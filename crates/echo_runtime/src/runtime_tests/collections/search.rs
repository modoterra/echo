use super::*;
use crate::collections::EchoArrayKey;

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
