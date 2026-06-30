use super::*;

#[test]
fn environment_process_builtins_follow_php_shapes() {
    let key = format!("ECHO_RUNTIME_ENV_TEST_{}", std::process::id());
    let set_assignment = test_string_value(format!("{key}=staging").as_bytes());
    let empty_assignment = test_string_value(format!("{key}=").as_bytes());
    let unset_assignment = test_string_value(key.as_bytes());
    let key_value = test_string_value(key.as_bytes());

    assert_eq!(echo_php_putenv(set_assignment), EchoValue::bool(true));
    assert_eq!(
        echo_php_getenv(key_value, EchoValue::bool(false)).string_bytes(),
        Some(b"staging".to_vec())
    );

    assert_eq!(echo_php_putenv(empty_assignment), EchoValue::bool(true));
    assert_eq!(
        echo_php_getenv(key_value, EchoValue::bool(false)).string_bytes(),
        Some(Vec::new())
    );

    assert_eq!(echo_php_putenv(unset_assignment), EchoValue::bool(true));
    assert_eq!(
        echo_php_getenv(key_value, EchoValue::bool(false)),
        EchoValue::bool(false)
    );
    assert!(echo_php_getenv(EchoValue::null(), EchoValue::bool(false)).is_array());
    assert!(echo_php_gethostname().is_string() || echo_php_gethostname() == EchoValue::bool(false));
    assert_eq!(echo_php_is_int(echo_php_getmypid()), EchoValue::bool(true));
    assert_eq!(
        echo_php_phpversion(EchoValue::null()).string_bytes(),
        Some(b"8.2.0".to_vec())
    );
    assert_eq!(
        echo_php_phpversion(test_string_value(b"json")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_php_sapi_name().string_bytes(),
        Some(b"cli".to_vec())
    );
    assert_eq!(
        echo_php_zend_version().string_bytes(),
        Some(b"8.2.0".to_vec())
    );
}
