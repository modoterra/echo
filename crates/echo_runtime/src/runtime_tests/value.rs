use super::*;
use crate::reflection::REFLECTION_SOURCE_PHP_BUILTIN;

#[test]
fn abs_preserves_php_integer_absolute_value_behavior() {
    assert_eq!(echo_php_abs(EchoValue::int(42)), EchoValue::int(42));
    assert_eq!(echo_php_abs(EchoValue::int(-42)), EchoValue::int(42));
    assert_eq!(echo_php_abs(EchoValue::int(0)), EchoValue::int(0));
    assert_eq!(echo_php_abs(EchoValue::int(i64::MIN)), EchoValue::error());
    assert_eq!(echo_php_abs(EchoValue::bool(true)), EchoValue::error());
}

#[test]
fn is_numeric_preserves_php_numeric_string_rules() {
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: b" 1337e0 ".to_vec(),
    }));
    let decimal = Box::into_raw(Box::new(EchoString {
        bytes: b"4.2".to_vec(),
    }));
    let hex_prefixed = Box::into_raw(Box::new(EchoString {
        bytes: b"0x539".to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_is_numeric(EchoValue::int(42)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::string(numeric)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::string(decimal)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::string(hex_prefixed)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::string(empty)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::bool(true)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_numeric(EchoValue::null()),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(decimal));
        drop(Box::from_raw(hex_prefixed));
        drop(Box::from_raw(empty));
    }
}

#[test]
fn is_float_is_false_for_current_non_float_values() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: b"4.2".to_vec(),
    }));
    let array = Box::into_raw(Box::new(EchoArray::from_values(vec![EchoValue::int(1)])));

    assert_eq!(
        echo_php_is_float(EchoValue::int(42)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_float(EchoValue::string(string)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_float(EchoValue::array(array)),
        EchoValue::bool(false)
    );
    assert_eq!(echo_php_is_float(EchoValue::null()), EchoValue::bool(false));

    unsafe {
        drop(Box::from_raw(string));
        drop(Box::from_raw(array));
    }
}

#[test]
fn is_finite_preserves_php_float_coercion_for_current_values() {
    let finite_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b" 4.2 ".to_vec(),
    }));
    let infinite_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b"1e9999".to_vec(),
    }));
    let non_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b"not numeric".to_vec(),
    }));
    let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

    assert_eq!(
        echo_php_is_finite(EchoValue::int(42)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_finite(EchoValue::bool(false)),
        EchoValue::bool(true)
    );
    assert_eq!(echo_php_is_finite(EchoValue::null()), EchoValue::bool(true));
    assert_eq!(
        echo_php_is_finite(EchoValue::string(finite_numeric)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_finite(EchoValue::string(infinite_numeric)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_finite(EchoValue::string(non_numeric)),
        EchoValue::error()
    );
    assert_eq!(
        echo_php_is_finite(EchoValue::array(array)),
        EchoValue::error()
    );

    unsafe {
        drop(Box::from_raw(finite_numeric));
        drop(Box::from_raw(infinite_numeric));
        drop(Box::from_raw(non_numeric));
        drop(Box::from_raw(array));
    }
}

#[test]
fn is_infinite_preserves_php_float_coercion_for_current_values() {
    let finite_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b" 4.2 ".to_vec(),
    }));
    let infinite_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b"1e9999".to_vec(),
    }));
    let negative_infinite_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b"-1e9999".to_vec(),
    }));
    let non_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b"not numeric".to_vec(),
    }));
    let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

    assert_eq!(
        echo_php_is_infinite(EchoValue::int(42)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_infinite(EchoValue::bool(true)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_infinite(EchoValue::null()),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_infinite(EchoValue::string(finite_numeric)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_infinite(EchoValue::string(infinite_numeric)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_infinite(EchoValue::string(negative_infinite_numeric)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_infinite(EchoValue::string(non_numeric)),
        EchoValue::error()
    );
    assert_eq!(
        echo_php_is_infinite(EchoValue::array(array)),
        EchoValue::error()
    );

    unsafe {
        drop(Box::from_raw(finite_numeric));
        drop(Box::from_raw(infinite_numeric));
        drop(Box::from_raw(negative_infinite_numeric));
        drop(Box::from_raw(non_numeric));
        drop(Box::from_raw(array));
    }
}

#[test]
fn is_nan_preserves_php_float_coercion_for_current_values() {
    let finite_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b" 4.2 ".to_vec(),
    }));
    let infinite_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b"1e9999".to_vec(),
    }));
    let non_numeric = Box::into_raw(Box::new(EchoString {
        bytes: b"not numeric".to_vec(),
    }));
    let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

    assert_eq!(echo_php_is_nan(EchoValue::int(42)), EchoValue::bool(false));
    assert_eq!(
        echo_php_is_nan(EchoValue::bool(true)),
        EchoValue::bool(false)
    );
    assert_eq!(echo_php_is_nan(EchoValue::null()), EchoValue::bool(false));
    assert_eq!(
        echo_php_is_nan(EchoValue::string(finite_numeric)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_nan(EchoValue::string(infinite_numeric)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_nan(EchoValue::string(non_numeric)),
        EchoValue::error()
    );
    assert_eq!(echo_php_is_nan(EchoValue::array(array)), EchoValue::error());

    unsafe {
        drop(Box::from_raw(finite_numeric));
        drop(Box::from_raw(infinite_numeric));
        drop(Box::from_raw(non_numeric));
        drop(Box::from_raw(array));
    }
}

#[test]
fn is_object_reports_only_object_values() {
    let object = Box::into_raw(Box::new(EchoObject { fields: Vec::new() }));
    let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));
    let list = Box::into_raw(Box::new(EchoList { values: Vec::new() }));
    let string = Box::into_raw(Box::new(EchoString {
        bytes: b"value".to_vec(),
    }));

    assert_eq!(
        echo_php_is_object(EchoValue::object(object)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_object(EchoValue::array(array)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_object(EchoValue::list(list)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_object(EchoValue::string(string)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_object(EchoValue::int(42)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_object(EchoValue::bool(true)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_object(EchoValue::null()),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(object));
        drop(Box::from_raw(array));
        drop(Box::from_raw(list));
        drop(Box::from_raw(string));
    }
}

#[test]
fn is_resource_reports_runtime_resource_handles() {
    let listener = Box::into_raw(Box::new(net::listen("127.0.0.1:0").expect("listen")));
    let object = Box::into_raw(Box::new(EchoObject { fields: Vec::new() }));
    let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

    assert_eq!(
        echo_php_is_resource(EchoValue::tcp_listener(listener)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_resource(EchoValue::object(object)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_resource(EchoValue::array(array)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_resource(EchoValue::int(42)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_resource(EchoValue::null()),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(listener));
        drop(Box::from_raw(object));
        drop(Box::from_raw(array));
    }
}

#[test]
fn gettype_returns_php_type_names_for_current_values() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: b"abc".to_vec(),
    }));
    let list = Box::into_raw(Box::new(EchoList {
        values: vec![EchoValue::int(1)],
    }));

    assert_eq!(
        echo_php_gettype(EchoValue::null()).string_bytes(),
        Some(b"NULL".to_vec())
    );
    assert_eq!(
        echo_php_gettype(EchoValue::bool(false)).string_bytes(),
        Some(b"boolean".to_vec())
    );
    assert_eq!(
        echo_php_gettype(EchoValue::int(42)).string_bytes(),
        Some(b"integer".to_vec())
    );
    assert_eq!(
        echo_php_gettype(EchoValue::string(string)).string_bytes(),
        Some(b"string".to_vec())
    );
    assert_eq!(
        echo_php_gettype(EchoValue::list(list)).string_bytes(),
        Some(b"list".to_vec())
    );

    unsafe {
        drop(Box::from_raw(string));
        drop(Box::from_raw(list));
    }
}

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
fn function_exists_reports_supported_internal_builtins() {
    unsafe {
        register_reflection_for_test(
            "strlen",
            "string $string",
            "int",
            REFLECTION_SOURCE_PHP_BUILTIN,
        );
        register_reflection_for_test(
            "sizeof",
            "Countable|array $value",
            "int",
            REFLECTION_SOURCE_PHP_BUILTIN,
        );
    }

    let strlen = Box::into_raw(Box::new(EchoString {
        bytes: b"strlen".to_vec(),
    }));
    let uppercase = Box::into_raw(Box::new(EchoString {
        bytes: b"STRLEN".to_vec(),
    }));
    let alias = Box::into_raw(Box::new(EchoString {
        bytes: b"sizeof".to_vec(),
    }));
    let construct = Box::into_raw(Box::new(EchoString {
        bytes: b"echo".to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: b"definitely_missing_echo_builtin".to_vec(),
    }));

    assert_eq!(
        echo_php_function_exists(EchoValue::string(strlen)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_function_exists(EchoValue::string(uppercase)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_function_exists(EchoValue::string(alias)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_function_exists(EchoValue::string(construct)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_function_exists(EchoValue::string(missing)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(strlen));
        drop(Box::from_raw(uppercase));
        drop(Box::from_raw(alias));
        drop(Box::from_raw(construct));
        drop(Box::from_raw(missing));
    }
}

#[test]
fn is_callable_reports_registered_function_names() {
    unsafe {
        register_reflection_for_test(
            "fixture_callable_builtin",
            "",
            "",
            REFLECTION_SOURCE_PHP_BUILTIN,
        );
        register_reflection_for_test("fixture_callable_userland", "", "", 0);
    }

    let builtin = Box::into_raw(Box::new(EchoString {
        bytes: b"fixture_callable_builtin".to_vec(),
    }));
    let uppercase = Box::into_raw(Box::new(EchoString {
        bytes: b"FIXTURE_CALLABLE_BUILTIN".to_vec(),
    }));
    let userland = Box::into_raw(Box::new(EchoString {
        bytes: b"fixture_callable_userland".to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: b"definitely_missing_callable".to_vec(),
    }));
    let non_utf8 = Box::into_raw(Box::new(EchoString { bytes: vec![0xff] }));
    let array = Box::into_raw(Box::new(EchoArray::from_values(Vec::new())));

    assert_eq!(
        echo_php_is_callable(EchoValue::string(builtin)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_callable(EchoValue::string(uppercase)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_callable(EchoValue::string(userland)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_is_callable(EchoValue::string(missing)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_callable(EchoValue::string(non_utf8)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_callable(EchoValue::array(array)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_is_callable(EchoValue::null()),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(builtin));
        drop(Box::from_raw(uppercase));
        drop(Box::from_raw(userland));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(non_utf8));
        drop(Box::from_raw(array));
    }
}

unsafe fn register_reflection_for_test(
    name: &str,
    params: &str,
    return_type: &str,
    source_kind: i32,
) {
    unsafe {
        echo_reflection_register_function(
            name.as_ptr(),
            name.len(),
            params.as_ptr(),
            params.len(),
            return_type.as_ptr(),
            return_type.len(),
            source_kind,
        );
    }
}

#[test]
fn reflect_type_of_reports_runtime_value_categories() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: b"text".to_vec(),
    }));
    let list = Box::into_raw(Box::new(EchoList { values: Vec::new() }));

    assert_eq!(
        echo_std_reflect_type_of(EchoValue::null()).string_bytes(),
        Some(b"null".to_vec())
    );
    assert_eq!(
        echo_std_reflect_type_of(EchoValue::bool(true)).string_bytes(),
        Some(b"bool".to_vec())
    );
    assert_eq!(
        echo_std_reflect_type_of(EchoValue::int(42)).string_bytes(),
        Some(b"int".to_vec())
    );
    assert_eq!(
        echo_std_reflect_type_of(EchoValue::string(string)).string_bytes(),
        Some(b"string".to_vec())
    );
    assert_eq!(
        echo_std_reflect_type_of(EchoValue::list(list)).string_bytes(),
        Some(b"list".to_vec())
    );

    unsafe {
        drop(Box::from_raw(string));
        drop(Box::from_raw(list));
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
