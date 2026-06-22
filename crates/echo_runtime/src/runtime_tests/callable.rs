use super::*;

#[test]
fn null_normalizes_to_no_callable() {
    assert_eq!(echo_normalize_callable(EchoValue::null()), Ok(None));
    assert!(!echo_is_callable(EchoValue::null()));
}

#[test]
fn invalid_value_does_not_normalize_to_callable() {
    let value = EchoValue {
        kind: 999,
        payload: 0,
    };

    assert_eq!(
        echo_normalize_callable(value),
        Err(EchoError::InvalidCallable)
    );
    assert!(!echo_is_callable(value));
}

#[test]
fn string_value_normalizes_to_function_callable() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: b"filter".to_vec(),
    }));
    let value = EchoValue::string(string);

    assert_eq!(
        echo_normalize_callable(value),
        Ok(Some(EchoCallable::Function(EchoSymbol::new("filter"))))
    );
    assert!(echo_is_callable(value));

    unsafe {
        drop(Box::from_raw(string));
    }
}

#[test]
fn non_utf8_string_value_is_not_callable() {
    let string = Box::into_raw(Box::new(EchoString { bytes: vec![0xff] }));
    let value = EchoValue::string(string);

    assert_eq!(
        echo_normalize_callable(value),
        Err(EchoError::InvalidCallable)
    );

    unsafe {
        drop(Box::from_raw(string));
    }
}

#[test]
fn function_callable_call_fails_until_registry_exists() {
    let callable = EchoCallable::Function(EchoSymbol::new("filter"));

    assert_eq!(
        echo_call(&callable, &[]),
        Err(EchoError::UndefinedFunction(EchoSymbol::new("filter")))
    );
}
