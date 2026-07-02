use super::*;

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
fn get_debug_type_returns_declaration_style_type_names() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: b"abc".to_vec(),
    }));
    let list = Box::into_raw(Box::new(EchoList {
        values: vec![EchoValue::int(1)],
    }));
    let fixture_path =
        std::env::temp_dir().join(format!("echo-runtime-debug-type-{}", std::process::id()));
    std::fs::write(&fixture_path, b"abc").expect("write debug type stream fixture");
    let path = Box::into_raw(Box::new(EchoString {
        bytes: fixture_path.to_string_lossy().as_bytes().to_vec(),
    }));
    let stream = echo_php_fopen(
        EchoValue::string(path),
        test_string_value(b"r"),
        EchoValue::bool(false),
        EchoValue::null(),
    );

    assert_eq!(
        echo_php_get_debug_type(EchoValue::null()).string_bytes(),
        Some(b"null".to_vec())
    );
    assert_eq!(
        echo_php_get_debug_type(EchoValue::bool(false)).string_bytes(),
        Some(b"bool".to_vec())
    );
    assert_eq!(
        echo_php_get_debug_type(EchoValue::int(42)).string_bytes(),
        Some(b"int".to_vec())
    );
    assert_eq!(
        echo_php_get_debug_type(EchoValue::string(string)).string_bytes(),
        Some(b"string".to_vec())
    );
    assert_eq!(
        echo_php_get_debug_type(EchoValue::list(list)).string_bytes(),
        Some(b"array".to_vec())
    );
    assert_eq!(
        echo_php_get_debug_type(stream).string_bytes(),
        Some(b"resource (stream)".to_vec())
    );
    assert_eq!(echo_php_fclose(stream), EchoValue::bool(true));
    assert_eq!(
        echo_php_get_debug_type(stream).string_bytes(),
        Some(b"resource (closed)".to_vec())
    );

    unsafe {
        drop(Box::from_raw(string));
        drop(Box::from_raw(list));
        drop(Box::from_raw(path));
    }
    std::fs::remove_file(fixture_path).ok();
}
