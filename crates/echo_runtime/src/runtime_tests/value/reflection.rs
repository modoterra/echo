use super::*;
use crate::reflection::REFLECTION_SOURCE_PHP_BUILTIN;

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
