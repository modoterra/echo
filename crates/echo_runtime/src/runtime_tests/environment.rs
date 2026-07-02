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
        echo_php_phpcredits(EchoValue::int(0)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_zend_version().string_bytes(),
        Some(b"8.2.0".to_vec())
    );
    assert_eq!(
        echo_php_dl(test_string_value(b"missing_echo_extension.so")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_extension_loaded(test_string_value(b"json")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_extension_loaded(test_string_value(b"JSON")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_count(echo_php_get_loaded_extensions(EchoValue::bool(false))),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_count(echo_php_get_loaded_extensions(EchoValue::bool(true))),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_get_extension_funcs(test_string_value(b"json")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_get_extension_funcs(test_string_value(b"JSON")),
        EchoValue::bool(false)
    );
    assert!(echo_php_cli_get_process_title().is_null());
    assert_eq!(
        echo_php_cli_set_process_title(test_string_value(b"echo worker")),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_cli_get_process_title().string_bytes(),
        Some(b"echo worker".to_vec())
    );
    assert_eq!(
        echo_php_get_cfg_var(test_string_value(b"include_path")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_get_cfg_var(test_string_value(b"memory_limit")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_ini_get(test_string_value(b"include_path")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_ini_get(test_string_value(b"memory_limit")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_count(echo_php_ini_get_all(
            EchoValue::null(),
            EchoValue::bool(true)
        )),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_count(echo_php_ini_get_all(
            test_string_value(b""),
            EchoValue::bool(false)
        )),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_ini_get_all(test_string_value(b"json"), EchoValue::bool(true)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_ini_parse_quantity(test_string_value(b"256M")),
        EchoValue::int(268_435_456)
    );
    assert_eq!(
        echo_php_ini_parse_quantity(test_string_value(b"4G")),
        EchoValue::int(4_294_967_296)
    );
    assert_eq!(
        echo_php_ini_parse_quantity(test_string_value(b"0x10")),
        EchoValue::int(16)
    );
    assert_eq!(
        echo_php_ini_parse_quantity(test_string_value(b"0b1010")),
        EchoValue::int(10)
    );
    assert_eq!(
        echo_php_ini_parse_quantity(test_string_value(b"010")),
        EchoValue::int(8)
    );
    assert_eq!(
        echo_php_ini_parse_quantity(test_string_value(b"10F")),
        EchoValue::int(10)
    );
    assert_eq!(
        echo_php_ini_parse_quantity(test_string_value(b"foobar")),
        EchoValue::int(0)
    );
    assert_eq!(echo_php_get_include_path(), EchoValue::bool(false));
    assert_eq!(
        echo_php_set_include_path(test_string_value(b".:/app/lib")),
        EchoValue::bool(false)
    );
    assert_eq!(echo_php_connection_aborted(), EchoValue::int(0));
    assert_eq!(echo_php_connection_status(), EchoValue::int(0));
    assert_eq!(
        echo_php_ignore_user_abort(EchoValue::null()),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_ignore_user_abort(EchoValue::bool(true)),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_ignore_user_abort(EchoValue::null()),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_ignore_user_abort(EchoValue::bool(false)),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_ignore_user_abort(EchoValue::null()),
        EchoValue::int(0)
    );
    assert_eq!(echo_php_count(echo_php_headers_list()), EchoValue::int(0));
    assert_eq!(echo_php_headers_sent(), EchoValue::bool(false));
    echo_php_header(
        test_string_value(b"X-Test: one"),
        EchoValue::bool(true),
        EchoValue::int(0),
    );
    echo_php_header(
        test_string_value(b"HTTP/1.1 404 Not Found"),
        EchoValue::bool(true),
        EchoValue::int(404),
    );
    echo_php_header_remove(EchoValue::null());
    echo_php_header_remove(test_string_value(b"X-Test"));
    assert_eq!(
        echo_php_http_response_code(EchoValue::null()),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_http_response_code(EchoValue::int(201)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_http_response_code(EchoValue::null()),
        EchoValue::int(201)
    );
    assert_eq!(
        echo_php_http_response_code(EchoValue::int(404)),
        EchoValue::int(201)
    );
    assert_eq!(
        echo_php_http_response_code(EchoValue::null()),
        EchoValue::int(404)
    );
    assert_eq!(
        echo_php_ini_set(
            test_string_value(b"memory_limit"),
            test_string_value(b"128M")
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_ini_set(
            test_string_value(b"include_path"),
            test_string_value(b".:/app")
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_ini_alter(
            test_string_value(b"memory_limit"),
            test_string_value(b"128M")
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_ini_alter(
            test_string_value(b"include_path"),
            test_string_value(b".:/app")
        ),
        EchoValue::bool(false)
    );
    echo_php_ini_restore(test_string_value(b"memory_limit"));
    echo_php_ini_restore(test_string_value(b"include_path"));
    assert_eq!(echo_php_php_ini_loaded_file(), EchoValue::bool(false));
    assert_eq!(echo_php_php_ini_scanned_files(), EchoValue::bool(false));
}

#[test]
fn syslog_builtins_return_true_for_cli_baseline() {
    assert_eq!(
        echo_php_openlog(
            test_string_value(b"echo-test"),
            EchoValue::int(0),
            EchoValue::int(8)
        ),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_syslog(
            EchoValue::int(6),
            test_string_value(b"compatibility message")
        ),
        EchoValue::bool(true)
    );
    assert_eq!(echo_php_closelog(), EchoValue::bool(true));
}

#[test]
fn image_type_helpers_map_documented_constants() {
    assert_eq!(
        echo_php_image_type_to_extension(EchoValue::int(1), EchoValue::bool(true)).string_bytes(),
        Some(b".gif".to_vec())
    );
    assert_eq!(
        echo_php_image_type_to_extension(EchoValue::int(2), EchoValue::bool(false)).string_bytes(),
        Some(b"jpeg".to_vec())
    );
    assert_eq!(
        echo_php_image_type_to_mime_type(EchoValue::int(3)).string_bytes(),
        Some(b"image/png".to_vec())
    );
    assert_eq!(
        echo_php_image_type_to_mime_type(EchoValue::int(18)).string_bytes(),
        Some(b"image/webp".to_vec())
    );
    assert_eq!(
        echo_php_image_type_to_extension(EchoValue::int(999), EchoValue::bool(true)),
        EchoValue::bool(false)
    );
}

#[test]
fn http_last_response_headers_start_empty_and_clear_is_noop() {
    assert_eq!(echo_php_http_get_last_response_headers(), EchoValue::null());
    echo_php_http_clear_last_response_headers();
    assert_eq!(echo_php_http_get_last_response_headers(), EchoValue::null());
}

#[test]
fn gethostbyname_returns_ipv4_or_original_hostname() {
    assert_eq!(
        echo_php_gethostbyname(test_string_value(b"localhost")).string_bytes(),
        Some(b"127.0.0.1".to_vec())
    );
    assert_eq!(
        echo_php_gethostbyname(test_string_value(b"echo.invalid")).string_bytes(),
        Some(b"echo.invalid".to_vec())
    );
}

#[test]
fn gethostbynamel_returns_ipv4_array_or_false() {
    let local = echo_php_gethostbynamel(test_string_value(b"localhost"));
    assert!(local.is_array());
    assert_eq!(
        echo_php_in_array(
            test_string_value(b"127.0.0.1"),
            local,
            EchoValue::bool(true)
        ),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_gethostbynamel(test_string_value(b"echo.invalid")),
        EchoValue::bool(false)
    );
}

#[test]
fn gethostbyaddr_returns_hosts_entry_original_ip_or_false() {
    assert_eq!(
        echo_php_gethostbyaddr(test_string_value(b"127.0.0.1")).string_bytes(),
        Some(b"localhost".to_vec())
    );
    assert_eq!(
        echo_php_gethostbyaddr(test_string_value(b"192.0.2.1")).string_bytes(),
        Some(b"192.0.2.1".to_vec())
    );
    assert_eq!(
        echo_php_gethostbyaddr(test_string_value(b"not-an-ip")),
        EchoValue::bool(false)
    );
}

#[test]
fn inet_packed_address_builtins_round_trip_ipv4_and_ipv6() {
    assert_eq!(
        echo_php_bin2hex(echo_php_inet_pton(test_string_value(b"127.0.0.1"))).string_bytes(),
        Some(b"7f000001".to_vec())
    );
    assert_eq!(
        echo_php_bin2hex(echo_php_inet_pton(test_string_value(b"::1"))).string_bytes(),
        Some(b"00000000000000000000000000000001".to_vec())
    );
    assert_eq!(
        echo_php_inet_ntop(echo_php_inet_pton(test_string_value(b"2001:db8::1"))).string_bytes(),
        Some(b"2001:db8::1".to_vec())
    );
    assert_eq!(
        echo_php_inet_pton(test_string_value(b"172.27.1.04")),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_inet_ntop(test_string_value(b"bad")),
        EchoValue::bool(false)
    );
}
