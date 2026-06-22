use super::*;
use crate::collections::EchoArrayKey;
use crate::value::{ECHO_VALUE_TCP_CONNECTION, ECHO_VALUE_TCP_LISTENER};
use std::thread;
use std::time::{Duration, Instant};

fn test_string_value(bytes: &[u8]) -> EchoValue {
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes.to_vec()))))
}

fn assert_float_value(value: EchoValue, expected: f64) {
    assert_eq!(value.kind, ECHO_VALUE_FLOAT);
    assert!((f64::from_bits(value.payload) - expected).abs() < 0.000000000001);
}

mod collections;
mod filesystem;

#[test]
fn task_defer_returns_task_value() {
    unsafe extern "C" fn callback() -> EchoValue {
        EchoValue::int(1)
    }

    let value = echo_task_defer(Some(callback));

    assert!(value.is_task());
    assert_ne!(value.payload, 0);
}

#[test]
fn task_run_starts_callback_before_join_collects_it() {
    static CALLBACK_RUNS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn callback() -> EchoValue {
        CALLBACK_RUNS.fetch_add(1, Ordering::Relaxed);
        EchoValue::int(42)
    }

    CALLBACK_RUNS.store(0, Ordering::Relaxed);
    let task = echo_task_defer(Some(callback));
    let task = echo_task_run(task);

    let deadline = Instant::now() + Duration::from_secs(1);
    while CALLBACK_RUNS.load(Ordering::Relaxed) == 0 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
    }

    assert_eq!(CALLBACK_RUNS.load(Ordering::Relaxed), 1);

    let result = echo_task_join(task);

    assert_eq!(CALLBACK_RUNS.load(Ordering::Relaxed), 1);
    assert_eq!(result, EchoValue::int(42));
}

#[test]
fn integer_arithmetic_core_abi_adds_and_subtracts() {
    assert_eq!(
        echo_value_add(EchoValue::int(3), EchoValue::int(5)),
        EchoValue::int(8)
    );
    assert_eq!(
        echo_value_sub(EchoValue::int(3), EchoValue::int(5)),
        EchoValue::int(-2)
    );
    assert_eq!(
        echo_value_sub(EchoValue::int(3), test_string_value(b"not numeric")),
        EchoValue::error()
    );
}

#[test]
fn php_numeric_arithmetic_coerces_strings_bools_and_null() {
    assert_float_value(
        echo_value_add(test_string_value(b"3.2"), test_string_value(b"3.4")),
        6.6,
    );
    assert_eq!(
        echo_value_add(EchoValue::null(), EchoValue::int(5)),
        EchoValue::int(5)
    );
    assert_eq!(
        echo_value_add(EchoValue::bool(true), EchoValue::int(2)),
        EchoValue::int(3)
    );
}

#[test]
fn php_arithmetic_core_abi_handles_remaining_operators() {
    assert_eq!(
        echo_value_mul(EchoValue::int(3), EchoValue::int(5)),
        EchoValue::int(15)
    );
    assert_eq!(
        echo_value_div(EchoValue::int(5), EchoValue::int(2)),
        EchoValue::float(2.5)
    );
    assert_eq!(
        echo_value_div(EchoValue::int(6), EchoValue::int(3)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_value_mod(EchoValue::int(-5), EchoValue::int(3)),
        EchoValue::int(-2)
    );
    assert_eq!(
        echo_value_pow(EchoValue::int(2), EchoValue::int(3)),
        EchoValue::int(8)
    );
    assert_eq!(
        echo_value_unary_minus(EchoValue::float(2.5)),
        EchoValue::float(-2.5)
    );
}

#[test]
fn std_net_abi_exchanges_loopback_bytes() {
    let address = unsafe { echo_value_string(c"127.0.0.1:0".as_ptr().cast(), 11) };
    let server = echo_std_net_listen(address);
    assert_eq!(server.kind, ECHO_VALUE_TCP_LISTENER);

    let listener = server.as_tcp_listener_ref().expect("listener");
    let address = listener.local_addr().expect("local addr").to_string();

    let client = thread::spawn(move || {
        let address = unsafe { echo_value_string(address.as_ptr(), address.len()) };
        let connection = echo_std_net_connect(address);
        assert_eq!(connection.kind, ECHO_VALUE_TCP_CONNECTION);

        let request = unsafe { echo_value_string(c"ping".as_ptr().cast(), 4) };
        assert_eq!(echo_std_net_write(connection, request), EchoValue::int(4));
        let response = echo_std_net_read(connection, EchoValue::int(4));
        assert_eq!(response.string_bytes().expect("response"), b"pong");
        assert_eq!(echo_std_net_close(connection), EchoValue::null());
    });

    let connection = echo_std_net_accept(server);
    assert_eq!(connection.kind, ECHO_VALUE_TCP_CONNECTION);
    let request = echo_std_net_read(connection, EchoValue::int(4));
    assert_eq!(request.string_bytes().expect("request"), b"ping");
    let response = unsafe { echo_value_string(c"pong".as_ptr().cast(), 4) };
    assert_eq!(echo_std_net_write(connection, response), EchoValue::int(4));
    assert_eq!(echo_std_net_close(connection), EchoValue::null());

    client.join().expect("client");
}

#[test]
fn std_http_response_text_formats_http_response() {
    let body = unsafe { echo_value_string(c"hello".as_ptr().cast(), 5) };
    let response = echo_std_http_response_text(body);

    assert_eq!(
        response.string_bytes().expect("response"),
        b"HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\ncontent-length: 5\r\nconnection: close\r\n\r\nhello"
    );
}

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

#[test]
fn string_case_builtins_convert_only_ascii_bytes() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: "Echo äÖ 123!".as_bytes().to_vec(),
    }));
    let value = EchoValue::string(string);

    assert_eq!(
        echo_php_strtoupper(value).string_bytes(),
        Some("ECHO äÖ 123!".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strtolower(value).string_bytes(),
        Some("echo äÖ 123!".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(string));
    }
}

#[test]
fn ucwords_preserves_php_default_separator_byte_behavior() {
    let words = Box::into_raw(Box::new(EchoString {
        bytes: "hello world".as_bytes().to_vec(),
    }));
    let tab = Box::into_raw(Box::new(EchoString {
        bytes: "hello\tworld".as_bytes().to_vec(),
    }));
    let hyphen = Box::into_raw(Box::new(EchoString {
        bytes: "hello-world".as_bytes().to_vec(),
    }));
    let mixed = Box::into_raw(Box::new(EchoString {
        bytes: "mIXed CASE".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ächo world".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_ucwords(EchoValue::string(words)).string_bytes(),
        Some("Hello World".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucwords(EchoValue::string(tab)).string_bytes(),
        Some("Hello\tWorld".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucwords(EchoValue::string(hyphen)).string_bytes(),
        Some("Hello-world".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucwords(EchoValue::string(mixed)).string_bytes(),
        Some("MIXed CASE".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucwords(EchoValue::string(non_ascii)).string_bytes(),
        Some("ächo World".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(words));
        drop(Box::from_raw(tab));
        drop(Box::from_raw(hyphen));
        drop(Box::from_raw(mixed));
        drop(Box::from_raw(non_ascii));
    }
}

#[test]
fn strval_preserves_php_scalar_string_coercion() {
    let string = Box::into_raw(Box::new(EchoString {
        bytes: "hello".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strval(EchoValue::string(string)).string_bytes(),
        Some("hello".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strval(EchoValue::int(42)).string_bytes(),
        Some("42".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strval(EchoValue::bool(true)).string_bytes(),
        Some("1".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strval(EchoValue::bool(false)).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_strval(EchoValue::null()).string_bytes(),
        Some(Vec::new())
    );

    unsafe {
        drop(Box::from_raw(string));
    }
}

#[test]
fn boolval_preserves_php_scalar_truthiness() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let zero = Box::into_raw(Box::new(EchoString {
        bytes: "0".as_bytes().to_vec(),
    }));
    let false_text = Box::into_raw(Box::new(EchoString {
        bytes: "false".as_bytes().to_vec(),
    }));

    assert_eq!(echo_php_boolval(EchoValue::null()), EchoValue::bool(false));
    assert_eq!(
        echo_php_boolval(EchoValue::bool(false)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_boolval(EchoValue::bool(true)),
        EchoValue::bool(true)
    );
    assert_eq!(echo_php_boolval(EchoValue::int(0)), EchoValue::bool(false));
    assert_eq!(echo_php_boolval(EchoValue::int(42)), EchoValue::bool(true));
    assert_eq!(
        echo_php_boolval(EchoValue::string(empty)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_boolval(EchoValue::string(zero)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_boolval(EchoValue::string(false_text)),
        EchoValue::bool(true)
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(zero));
        drop(Box::from_raw(false_text));
    }
}

#[test]
fn intval_preserves_php_default_base_scalar_coercion() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let prefixed = Box::into_raw(Box::new(EchoString {
        bytes: "42abc".as_bytes().to_vec(),
    }));
    let spaced = Box::into_raw(Box::new(EchoString {
        bytes: "  15".as_bytes().to_vec(),
    }));
    let negative = Box::into_raw(Box::new(EchoString {
        bytes: "-7".as_bytes().to_vec(),
    }));
    let non_numeric = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));

    assert_eq!(echo_php_intval(EchoValue::null()), EchoValue::int(0));
    assert_eq!(echo_php_intval(EchoValue::bool(false)), EchoValue::int(0));
    assert_eq!(echo_php_intval(EchoValue::bool(true)), EchoValue::int(1));
    assert_eq!(echo_php_intval(EchoValue::int(42)), EchoValue::int(42));
    assert_eq!(echo_php_intval(EchoValue::string(empty)), EchoValue::int(0));
    assert_eq!(
        echo_php_intval(EchoValue::string(prefixed)),
        EchoValue::int(42)
    );
    assert_eq!(
        echo_php_intval(EchoValue::string(spaced)),
        EchoValue::int(15)
    );
    assert_eq!(
        echo_php_intval(EchoValue::string(negative)),
        EchoValue::int(-7)
    );
    assert_eq!(
        echo_php_intval(EchoValue::string(non_numeric)),
        EchoValue::int(0)
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(prefixed));
        drop(Box::from_raw(spaced));
        drop(Box::from_raw(negative));
        drop(Box::from_raw(non_numeric));
    }
}

#[test]
fn floatval_preserves_php_scalar_float_coercion() {
    assert_float_value(echo_php_floatval(EchoValue::null()), 0.0);
    assert_float_value(echo_php_floatval(EchoValue::bool(true)), 1.0);
    assert_float_value(echo_php_floatval(EchoValue::int(42)), 42.0);

    let prefixed = Box::into_raw(Box::new(EchoString {
        bytes: b"122.34343The".to_vec(),
    }));
    let invalid = Box::into_raw(Box::new(EchoString {
        bytes: b"The122.34343".to_vec(),
    }));
    let offset = Box::into_raw(Box::new(EchoString {
        bytes: b"  -12.5px".to_vec(),
    }));
    let exponent = Box::into_raw(Box::new(EchoString {
        bytes: b"1e2x".to_vec(),
    }));

    assert_float_value(echo_php_floatval(EchoValue::string(prefixed)), 122.34343);
    assert_float_value(echo_php_floatval(EchoValue::string(invalid)), 0.0);
    assert_float_value(echo_php_floatval(EchoValue::string(offset)), -12.5);
    assert_float_value(echo_php_floatval(EchoValue::string(exponent)), 100.0);

    unsafe {
        drop(Box::from_raw(prefixed));
        drop(Box::from_raw(invalid));
        drop(Box::from_raw(offset));
        drop(Box::from_raw(exponent));
    }
}

#[test]
fn float_scalar_math_builtins_preserve_php_scalar_behavior() {
    assert_float_value(echo_php_pi(), std::f64::consts::PI);
    assert_float_value(
        echo_php_fmod(EchoValue::float(5.7), EchoValue::float(1.3)),
        0.5,
    );
    assert_float_value(
        echo_php_fmod(EchoValue::float(-5.7), EchoValue::float(1.3)),
        -0.5,
    );
    assert!(f64::from_bits(echo_php_fmod(EchoValue::int(5), EchoValue::int(0)).payload).is_nan());
}

#[test]
fn string_unary_builtins_preserve_php_byte_behavior() {
    let reversed = Box::into_raw(Box::new(EchoString {
        bytes: "Echo ÄÖ 123!".as_bytes().to_vec(),
    }));
    let ucfirst = Box::into_raw(Box::new(EchoString {
        bytes: "echo".as_bytes().to_vec(),
    }));
    let lcfirst = Box::into_raw(Box::new(EchoString {
        bytes: "Echo".as_bytes().to_vec(),
    }));
    let non_ascii_first = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strrev(EchoValue::string(reversed)).string_bytes(),
        Some(vec![
            b'!', b'3', b'2', b'1', b' ', 0x96, 0xc3, 0x84, 0xc3, b' ', b'o', b'h', b'c', b'E'
        ])
    );
    assert_eq!(
        echo_php_ucfirst(EchoValue::string(ucfirst)).string_bytes(),
        Some("Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_lcfirst(EchoValue::string(lcfirst)).string_bytes(),
        Some("echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ucfirst(EchoValue::string(non_ascii_first)).string_bytes(),
        Some("Ächo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_lcfirst(EchoValue::string(non_ascii_first)).string_bytes(),
        Some("Ächo".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(reversed));
        drop(Box::from_raw(ucfirst));
        drop(Box::from_raw(lcfirst));
        drop(Box::from_raw(non_ascii_first));
    }
}

#[test]
fn string_byte_builtins_preserve_php_byte_behavior() {
    let ascii = Box::into_raw(Box::new(EchoString {
        bytes: "A".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let rot13 = Box::into_raw(Box::new(EchoString {
        bytes: "Echo PHP 4.3.0 ÄÖ!".as_bytes().to_vec(),
    }));

    assert_eq!(echo_php_ord(EchoValue::string(ascii)), EchoValue::int(65));
    assert_eq!(
        echo_php_ord(EchoValue::string(non_ascii)),
        EchoValue::int(195)
    );
    assert_eq!(
        echo_php_str_rot13(EchoValue::string(rot13)).string_bytes(),
        Some("Rpub CUC 4.3.0 ÄÖ!".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(ascii));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(rot13));
    }
}

#[test]
fn base_string_conversion_builtins_preserve_php_byte_behavior() {
    assert_eq!(
        echo_php_chr(EchoValue::int(65)).string_bytes(),
        Some("A".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_chr(test_string_value(b"321")).string_bytes(),
        Some("A".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dechex(EchoValue::int(47)).string_bytes(),
        Some("2f".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dechex(EchoValue::int(-1)).string_bytes(),
        Some("ffffffffffffffff".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_decbin(EchoValue::int(26)).string_bytes(),
        Some("11010".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_decbin(EchoValue::int(-1)).string_bytes(),
        Some(
            "1111111111111111111111111111111111111111111111111111111111111111"
                .as_bytes()
                .to_vec()
        )
    );
    assert_eq!(
        echo_php_decoct(EchoValue::int(264)).string_bytes(),
        Some("410".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_decoct(EchoValue::int(-1)).string_bytes(),
        Some("1777777777777777777777".as_bytes().to_vec())
    );
}

#[test]
fn bin2hex_preserves_php_byte_behavior() {
    assert_eq!(
        echo_php_bin2hex(test_string_value(b"Echo")).string_bytes(),
        Some("4563686f".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_bin2hex(test_string_value("Ä".as_bytes())).string_bytes(),
        Some("c384".as_bytes().to_vec())
    );
}

#[test]
fn checksum_builtins_preserve_php_byte_behavior() {
    assert_eq!(
        echo_php_crc32(test_string_value(b"Echo\nPHP")),
        EchoValue::int(286159390)
    );
    assert_eq!(
        echo_php_md5(test_string_value(b"Echo\nPHP"), EchoValue::bool(false)).string_bytes(),
        Some("d4f2cb8de8248adb1e54f021bcd5e8c2".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_md5(test_string_value(b"Echo\nPHP"), EchoValue::bool(true)).string_bytes(),
        Some(vec![
            0xd4, 0xf2, 0xcb, 0x8d, 0xe8, 0x24, 0x8a, 0xdb, 0x1e, 0x54, 0xf0, 0x21, 0xbc, 0xd5,
            0xe8, 0xc2,
        ])
    );
    assert_eq!(
        echo_php_sha1(test_string_value(b"Echo\nPHP"), EchoValue::bool(false)).string_bytes(),
        Some(
            "2ac003b31b44befef7f0c8b7e0154e3118689876"
                .as_bytes()
                .to_vec()
        )
    );
    assert_eq!(
        echo_php_sha1(test_string_value(b"Echo\nPHP"), EchoValue::bool(true)).string_bytes(),
        Some(vec![
            0x2a, 0xc0, 0x03, 0xb3, 0x1b, 0x44, 0xbe, 0xfe, 0xf7, 0xf0, 0xc8, 0xb7, 0xe0, 0x15,
            0x4e, 0x31, 0x18, 0x68, 0x98, 0x76,
        ])
    );
}

#[test]
fn base_to_decimal_builtins_preserve_php_unsigned_string_behavior() {
    assert_eq!(
        echo_php_bindec(test_string_value(b"1010")),
        EchoValue::int(10)
    );
    assert_eq!(
        echo_php_bindec(test_string_value(b"0b10xx11")),
        EchoValue::int(11)
    );
    assert_eq!(
        echo_php_hexdec(test_string_value(b"0xff")),
        EchoValue::int(255)
    );
    assert_eq!(
        echo_php_hexdec(test_string_value(b"ffzz10")),
        EchoValue::int(65296)
    );
    assert_eq!(
        echo_php_octdec(test_string_value(b"0789")),
        EchoValue::int(7)
    );
    assert_eq!(echo_php_bindec(EchoValue::int(10)), EchoValue::int(2));
    assert_eq!(echo_php_hexdec(EchoValue::float(10.7)), EchoValue::int(263));
    assert_eq!(echo_php_octdec(EchoValue::null()), EchoValue::int(0));
    assert_float_value(
        echo_php_hexdec(test_string_value(b"FFFFFFFFFFFFFFFF")),
        u64::MAX as f64,
    );
    assert_eq!(
        echo_php_base_convert(
            test_string_value(b"a37334"),
            EchoValue::int(16),
            EchoValue::int(2)
        )
        .string_bytes(),
        Some("101000110111001100110100".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base_convert(
            test_string_value(b"ffzz10"),
            EchoValue::int(16),
            EchoValue::int(10)
        )
        .string_bytes(),
        Some("65296".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base_convert(
            EchoValue::float(3.14),
            EchoValue::int(10),
            EchoValue::int(10)
        )
        .string_bytes(),
        Some("314".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base_convert(
            test_string_value(b"10"),
            EchoValue::int(1),
            EchoValue::int(10)
        ),
        EchoValue::error()
    );
}

#[test]
fn escapeshellarg_preserves_php_unix_shell_argument_quoting() {
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"Echo")).string_bytes(),
        Some("'Echo'".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"it's ready")).string_bytes(),
        Some("'it'\\''s ready'".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_escapeshellarg(test_string_value(b"")).string_bytes(),
        Some("''".as_bytes().to_vec())
    );
}

#[test]
fn escapeshellcmd_preserves_php_unix_shell_meta_escaping() {
    fn string_value(bytes: &[u8]) -> *mut EchoString {
        Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        }))
    }

    let semicolon = string_value(b"path; rm -rf /");
    let paired_double = string_value(b"echo \"ok\"");
    let unpaired_double = string_value(b"echo \"unterminated");
    let paired_single = string_value(b"echo 'ok'");
    let unpaired_single = string_value(b"echo 'unterminated");
    let newline = string_value(b"line\nbreak");
    let slash = string_value(b"a\\b");
    let dollar = string_value(b"a$b");

    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(semicolon)).string_bytes(),
        Some(b"path\\; rm -rf /".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(paired_double)).string_bytes(),
        Some(b"echo \"ok\"".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(unpaired_double)).string_bytes(),
        Some(b"echo \\\"unterminated".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(paired_single)).string_bytes(),
        Some(b"echo 'ok'".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(unpaired_single)).string_bytes(),
        Some(b"echo \\'unterminated".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(newline)).string_bytes(),
        Some(b"line\\\nbreak".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(slash)).string_bytes(),
        Some(b"a\\\\b".to_vec())
    );
    assert_eq!(
        echo_php_escapeshellcmd(EchoValue::string(dollar)).string_bytes(),
        Some(b"a\\$b".to_vec())
    );

    unsafe {
        drop(Box::from_raw(semicolon));
        drop(Box::from_raw(paired_double));
        drop(Box::from_raw(unpaired_double));
        drop(Box::from_raw(paired_single));
        drop(Box::from_raw(unpaired_single));
        drop(Box::from_raw(newline));
        drop(Box::from_raw(slash));
        drop(Box::from_raw(dollar));
    }
}

#[test]
fn explode_preserves_php_array_count_and_limit_behavior() {
    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        })))
    }

    fn array_string_values(value: EchoValue) -> Vec<Vec<u8>> {
        let array = unsafe { (value.payload as *const EchoArray).as_ref() }.expect("array");
        array
            .values
            .iter()
            .map(|value| value.string_bytes().expect("string value"))
            .collect()
    }

    let all = echo_php_explode(
        string_value(b","),
        string_value(b"a,b,c"),
        EchoValue::int(i64::MAX),
    );
    let positive_limit = echo_php_explode(
        string_value(b","),
        string_value(b"a,b,c"),
        EchoValue::int(2),
    );
    let zero_limit = echo_php_explode(
        string_value(b","),
        string_value(b"a,b,c"),
        EchoValue::int(0),
    );
    let negative_limit = echo_php_explode(
        string_value(b","),
        string_value(b"a,b,c"),
        EchoValue::int(-1),
    );
    let missing_negative =
        echo_php_explode(string_value(b","), string_value(b"abc"), EchoValue::int(-1));
    let edge_empty = echo_php_explode(
        string_value(b","),
        string_value(b",a,"),
        EchoValue::int(i64::MAX),
    );

    assert_eq!(echo_php_count(all), EchoValue::int(3));
    assert_eq!(echo_php_array_is_list(all), EchoValue::bool(true));
    assert_eq!(all.string_bytes(), Some(b"Array".to_vec()));
    assert_eq!(
        array_string_values(all),
        vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]
    );
    assert_eq!(
        array_string_values(positive_limit),
        vec![b"a".to_vec(), b"b,c".to_vec()]
    );
    assert_eq!(array_string_values(zero_limit), vec![b"a,b,c".to_vec()]);
    assert_eq!(
        array_string_values(negative_limit),
        vec![b"a".to_vec(), b"b".to_vec()]
    );
    assert_eq!(array_string_values(missing_negative), Vec::<Vec<u8>>::new());
    assert_eq!(
        array_string_values(edge_empty),
        vec![Vec::new(), b"a".to_vec(), Vec::new()]
    );
    assert_eq!(
        echo_php_explode(
            string_value(b""),
            string_value(b"a,b"),
            EchoValue::int(i64::MAX)
        ),
        EchoValue::error()
    );
}

#[test]
fn implode_joins_array_values_with_php_string_coercion() {
    let array = EchoValue::array(Box::into_raw(Box::new(EchoArray {
        keys: vec![
            EchoArrayKey::String(b"first".to_vec()),
            EchoArrayKey::Int(0),
            EchoArrayKey::String(b"third".to_vec()),
            EchoArrayKey::Int(1),
            EchoArrayKey::Int(2),
        ],
        values: vec![
            test_string_value(b"one"),
            EchoValue::int(2),
            EchoValue::bool(true),
            EchoValue::bool(false),
            EchoValue::null(),
        ],
    })));
    let empty = EchoValue::array(Box::into_raw(Box::new(EchoArray::from_values(Vec::new()))));

    assert_eq!(
        echo_php_implode(test_string_value(b"|"), array).string_bytes(),
        Some(b"one|2|1||".to_vec())
    );
    assert_eq!(
        echo_php_implode(test_string_value(b"hello"), empty).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_implode(test_string_value(b","), test_string_value(b"not-array")),
        EchoValue::error()
    );
}

#[test]
fn base64_encode_preserves_php_byte_behavior() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let one_byte = Box::into_raw(Box::new(EchoString {
        bytes: "f".as_bytes().to_vec(),
    }));
    let two_bytes = Box::into_raw(Box::new(EchoString {
        bytes: "fo".as_bytes().to_vec(),
    }));
    let three_bytes = Box::into_raw(Box::new(EchoString {
        bytes: "foo".as_bytes().to_vec(),
    }));
    let text = Box::into_raw(Box::new(EchoString {
        bytes: "hello world".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_base64_encode(EchoValue::string(empty)).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(one_byte)).string_bytes(),
        Some("Zg==".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(two_bytes)).string_bytes(),
        Some("Zm8=".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(three_bytes)).string_bytes(),
        Some("Zm9v".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(text)).string_bytes(),
        Some("aGVsbG8gd29ybGQ=".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::string(non_ascii)).string_bytes(),
        Some("w4RjaG8=".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_encode(EchoValue::int(123)).string_bytes(),
        Some("MTIz".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(one_byte));
        drop(Box::from_raw(two_bytes));
        drop(Box::from_raw(three_bytes));
        drop(Box::from_raw(text));
        drop(Box::from_raw(non_ascii));
    }
}

#[test]
fn base64_decode_preserves_php_default_non_strict_byte_behavior() {
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let one_byte = Box::into_raw(Box::new(EchoString {
        bytes: "Zg==".as_bytes().to_vec(),
    }));
    let two_bytes = Box::into_raw(Box::new(EchoString {
        bytes: "Zm8=".as_bytes().to_vec(),
    }));
    let three_bytes = Box::into_raw(Box::new(EchoString {
        bytes: "Zm9v".as_bytes().to_vec(),
    }));
    let text = Box::into_raw(Box::new(EchoString {
        bytes: "aGVsbG8gd29ybGQ=".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "w4RjaG8=".as_bytes().to_vec(),
    }));
    let ignored = Box::into_raw(Box::new(EchoString {
        bytes: "Zm 9v".as_bytes().to_vec(),
    }));
    let invalid = Box::into_raw(Box::new(EchoString {
        bytes: "!!!!".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_base64_decode(EchoValue::string(empty)).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(one_byte)).string_bytes(),
        Some("f".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(two_bytes)).string_bytes(),
        Some("fo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(three_bytes)).string_bytes(),
        Some("foo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(text)).string_bytes(),
        Some("hello world".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(non_ascii)).string_bytes(),
        Some("Ächo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(ignored)).string_bytes(),
        Some("foo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_base64_decode(EchoValue::string(invalid)).string_bytes(),
        Some(Vec::new())
    );

    unsafe {
        drop(Box::from_raw(empty));
        drop(Box::from_raw(one_byte));
        drop(Box::from_raw(two_bytes));
        drop(Box::from_raw(three_bytes));
        drop(Box::from_raw(text));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(ignored));
        drop(Box::from_raw(invalid));
    }
}

#[test]
fn url_encoding_builtins_preserve_php_byte_behavior() {
    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        })))
    }

    assert_eq!(
        echo_php_rawurlencode(string_value(b"sales and marketing/Miami~")).string_bytes(),
        Some(b"sales%20and%20marketing%2FMiami~".to_vec())
    );
    assert_eq!(
        echo_php_urlencode(string_value(b"Data123!@-_ +~")).string_bytes(),
        Some(b"Data123%21%40-_+%2B%7E".to_vec())
    );
    assert_eq!(
        echo_php_rawurldecode(string_value(b"foo%20bar%40baz+plus%ZZ")).string_bytes(),
        Some(b"foo bar@baz+plus%ZZ".to_vec())
    );
    assert_eq!(
        echo_php_urldecode(string_value(b"green+and+red%2Bblue%ZZ")).string_bytes(),
        Some(b"green and red+blue%ZZ".to_vec())
    );
    assert_eq!(
        echo_php_rawurldecode(echo_php_rawurlencode(string_value(b"a/b c+~"))).string_bytes(),
        Some(b"a/b c+~".to_vec())
    );
    assert_eq!(
        echo_php_rawurlencode(string_value(&[0xc3, 0x84])).string_bytes(),
        Some(b"%C3%84".to_vec())
    );
}

#[test]
fn angle_conversion_builtins_preserve_php_float_coercion() {
    assert_float_value(echo_php_deg2rad(EchoValue::int(180)), std::f64::consts::PI);
    assert_float_value(
        echo_php_rad2deg(EchoValue::float(std::f64::consts::PI)),
        180.0,
    );
    assert_float_value(
        echo_php_deg2rad(test_string_value(b"-90")),
        -std::f64::consts::FRAC_PI_2,
    );
    assert_float_value(echo_php_rad2deg(EchoValue::bool(true)), 57.29577951308232);
    assert_float_value(echo_php_deg2rad(EchoValue::null()), 0.0);
    assert_eq!(
        echo_php_deg2rad(test_string_value(b"not numeric")),
        EchoValue::error()
    );
}

#[test]
fn trigonometric_builtins_preserve_php_float_coercion() {
    assert_float_value(
        echo_php_sin(EchoValue::float(std::f64::consts::FRAC_PI_6)),
        0.5,
    );
    assert_float_value(
        echo_php_cos(EchoValue::float(std::f64::consts::FRAC_PI_3)),
        0.5,
    );
    assert_float_value(
        echo_php_tan(EchoValue::float(std::f64::consts::FRAC_PI_4)),
        1.0,
    );
    assert_float_value(
        echo_php_asin(EchoValue::float(0.5)),
        std::f64::consts::FRAC_PI_6,
    );
    assert_float_value(
        echo_php_acos(EchoValue::float(0.5)),
        std::f64::consts::FRAC_PI_3,
    );
    assert_float_value(
        echo_php_atan(EchoValue::float(1.0)),
        std::f64::consts::FRAC_PI_4,
    );
    assert_float_value(
        echo_php_atan2(EchoValue::float(3.0), EchoValue::float(-3.0)),
        2.356194490192345,
    );
    assert_float_value(echo_php_sin(test_string_value(b"0.5")), 0.479425538604203);
    assert_float_value(echo_php_cos(EchoValue::bool(true)), 0.5403023058681398);
    assert!(f64::from_bits(echo_php_acos(EchoValue::int(2)).payload).is_nan());
}

#[test]
fn hyperbolic_builtins_preserve_php_float_behavior() {
    assert_float_value(echo_php_sinh(EchoValue::int(0)), 0.0);
    assert_float_value(echo_php_sinh(EchoValue::int(1)), 1.1752011936438014);
    assert_float_value(echo_php_cosh(EchoValue::int(0)), 1.0);
    assert_float_value(echo_php_cosh(EchoValue::int(1)), 1.5430806348152437);
    assert_float_value(echo_php_tanh(EchoValue::int(1)), 0.7615941559557649);
    assert_float_value(echo_php_asinh(EchoValue::int(1)), 0.881373587019543);
    assert_float_value(echo_php_acosh(EchoValue::int(1)), 0.0);
    assert_float_value(echo_php_atanh(EchoValue::float(0.5)), 0.5493061443340548);
    assert_float_value(echo_php_cosh(test_string_value(b"2.5")), 6.132289479663687);
    assert!(f64::from_bits(echo_php_acosh(EchoValue::int(0)).payload).is_nan());
    assert!(f64::from_bits(echo_php_atanh(EchoValue::int(2)).payload).is_nan());
}

#[test]
fn rounding_and_magnitude_builtins_preserve_php_float_behavior() {
    assert_float_value(echo_php_ceil(EchoValue::float(4.3)), 5.0);
    assert_float_value(echo_php_floor(EchoValue::float(9.999)), 9.0);
    assert_float_value(echo_php_floor(EchoValue::float(-3.14)), -4.0);
    assert_eq!(
        f64::from_bits(echo_php_ceil(EchoValue::float(-0.1)).payload).to_bits(),
        (-0.0f64).to_bits()
    );
    assert_float_value(echo_php_ceil(test_string_value(b"12.2")), 13.0);
    assert_float_value(echo_php_floor(EchoValue::bool(true)), 1.0);
    assert_float_value(echo_php_sqrt(EchoValue::int(9)), 3.0);
    assert_float_value(echo_php_sqrt(EchoValue::float(10.0)), 3.162277660168379);
    assert!(f64::from_bits(echo_php_sqrt(EchoValue::int(-1)).payload).is_nan());
    assert_float_value(echo_php_hypot(EchoValue::int(3), EchoValue::int(4)), 5.0);
    assert_float_value(
        echo_php_hypot(test_string_value(b"5"), test_string_value(b"12")),
        13.0,
    );
}

#[test]
fn exponential_and_logarithm_builtins_preserve_php_float_behavior() {
    assert_float_value(echo_php_exp(EchoValue::int(0)), 1.0);
    assert_float_value(echo_php_expm1(EchoValue::int(0)), 0.0);
    assert_float_value(echo_php_log(EchoValue::int(8), EchoValue::int(2)), 3.0);
    assert_float_value(echo_php_log10(EchoValue::int(1000)), 3.0);
    assert_float_value(echo_php_log1p(EchoValue::int(0)), 0.0);
    assert_eq!(
        f64::from_bits(
            echo_php_log(EchoValue::int(0), EchoValue::float(std::f64::consts::E)).payload
        ),
        f64::NEG_INFINITY
    );
    assert!(
        f64::from_bits(
            echo_php_log(EchoValue::int(-1), EchoValue::float(std::f64::consts::E)).payload
        )
        .is_nan()
    );
    assert!(f64::from_bits(echo_php_log1p(EchoValue::int(-2)).payload).is_nan());
    assert_eq!(
        echo_php_log(EchoValue::int(8), EchoValue::int(0)),
        EchoValue::error()
    );
    assert_eq!(
        echo_php_pow(EchoValue::int(2), EchoValue::int(8)),
        EchoValue::int(256)
    );
    assert_float_value(echo_php_pow(EchoValue::int(10), EchoValue::int(-1)), 0.1);
    assert_float_value(
        echo_php_fdiv(EchoValue::int(125), EchoValue::int(100)),
        1.25,
    );
    assert_eq!(
        f64::from_bits(echo_php_fdiv(EchoValue::int(1), EchoValue::int(0)).payload),
        f64::INFINITY
    );
    assert_float_value(
        echo_php_fpow(EchoValue::float(1.05), EchoValue::int(2)),
        1.1025,
    );
    assert_eq!(
        f64::from_bits(echo_php_fpow(EchoValue::int(0), EchoValue::int(-2)).payload),
        f64::INFINITY
    );
    assert!(
        f64::from_bits(echo_php_fpow(EchoValue::int(-1), EchoValue::float(5.5)).payload).is_nan()
    );
    assert!(
        f64::from_bits(echo_php_pow(EchoValue::int(-1), EchoValue::float(5.5)).payload).is_nan()
    );
    assert_eq!(
        f64::from_bits(echo_php_pow(EchoValue::int(0), EchoValue::int(-1)).payload),
        f64::INFINITY
    );
}

#[test]
fn basename_preserves_php_unix_path_string_behavior() {
    let path = Box::into_raw(Box::new(EchoString {
        bytes: "/etc/sudoers.d".as_bytes().to_vec(),
    }));
    let suffix = Box::into_raw(Box::new(EchoString {
        bytes: ".d".as_bytes().to_vec(),
    }));
    let trailing = Box::into_raw(Box::new(EchoString {
        bytes: "/etc/".as_bytes().to_vec(),
    }));
    let root = Box::into_raw(Box::new(EchoString {
        bytes: "/".as_bytes().to_vec(),
    }));
    let dot = Box::into_raw(Box::new(EchoString {
        bytes: ".".as_bytes().to_vec(),
    }));
    let empty_suffix = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let missing_suffix = Box::into_raw(Box::new(EchoString {
        bytes: ".txt".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_basename(EchoValue::string(path), EchoValue::string(suffix)).string_bytes(),
        Some("sudoers".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_basename(EchoValue::string(trailing), EchoValue::string(empty_suffix))
            .string_bytes(),
        Some("etc".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_basename(EchoValue::string(root), EchoValue::string(missing_suffix))
            .string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_basename(EchoValue::string(dot), EchoValue::string(empty_suffix)).string_bytes(),
        Some(".".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(path));
        drop(Box::from_raw(suffix));
        drop(Box::from_raw(trailing));
        drop(Box::from_raw(root));
        drop(Box::from_raw(dot));
        drop(Box::from_raw(empty_suffix));
        drop(Box::from_raw(missing_suffix));
    }
}

#[test]
fn dirname_preserves_php_unix_path_string_behavior() {
    let file = Box::into_raw(Box::new(EchoString {
        bytes: "/etc/passwd".as_bytes().to_vec(),
    }));
    let trailing = Box::into_raw(Box::new(EchoString {
        bytes: "/etc/".as_bytes().to_vec(),
    }));
    let root = Box::into_raw(Box::new(EchoString {
        bytes: "/".as_bytes().to_vec(),
    }));
    let dot = Box::into_raw(Box::new(EchoString {
        bytes: ".".as_bytes().to_vec(),
    }));
    let relative = Box::into_raw(Box::new(EchoString {
        bytes: "foo/bar/baz".as_bytes().to_vec(),
    }));
    let repeated = Box::into_raw(Box::new(EchoString {
        bytes: "foo//bar".as_bytes().to_vec(),
    }));
    let nested = Box::into_raw(Box::new(EchoString {
        bytes: "/usr/local/lib".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_dirname(EchoValue::string(file), EchoValue::int(1)).string_bytes(),
        Some("/etc".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(trailing), EchoValue::int(1)).string_bytes(),
        Some("/".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(root), EchoValue::int(1)).string_bytes(),
        Some("/".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(dot), EchoValue::int(1)).string_bytes(),
        Some(".".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(relative), EchoValue::int(1)).string_bytes(),
        Some("foo/bar".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(repeated), EchoValue::int(1)).string_bytes(),
        Some("foo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(nested), EchoValue::int(2)).string_bytes(),
        Some("/usr".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(file));
        drop(Box::from_raw(trailing));
        drop(Box::from_raw(root));
        drop(Box::from_raw(dot));
        drop(Box::from_raw(relative));
        drop(Box::from_raw(repeated));
        drop(Box::from_raw(nested));
    }
}

#[test]
fn hex2bin_preserves_php_byte_behavior() {
    let hex = Box::into_raw(Box::new(EchoString {
        bytes: "c384".as_bytes().to_vec(),
    }));
    let upper_hex = Box::into_raw(Box::new(EchoString {
        bytes: "4563686F".as_bytes().to_vec(),
    }));
    let invalid_hex = Box::into_raw(Box::new(EchoString {
        bytes: "f".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_hex2bin(EchoValue::string(hex)).string_bytes(),
        Some("Ä".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_hex2bin(EchoValue::string(upper_hex)).string_bytes(),
        Some("Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_hex2bin(EchoValue::string(invalid_hex)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(hex));
        drop(Box::from_raw(upper_hex));
        drop(Box::from_raw(invalid_hex));
    }
}

#[test]
fn str_repeat_preserves_php_byte_behavior() {
    let repeated = Box::into_raw(Box::new(EchoString {
        bytes: "xo".as_bytes().to_vec(),
    }));
    let empty_repeat = Box::into_raw(Box::new(EchoString {
        bytes: "x".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_str_repeat(EchoValue::string(repeated), EchoValue::int(3)).string_bytes(),
        Some("xoxoxo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_repeat(EchoValue::string(empty_repeat), EchoValue::int(0)).string_bytes(),
        Some(Vec::new())
    );

    unsafe {
        drop(Box::from_raw(repeated));
        drop(Box::from_raw(empty_repeat));
    }
}

#[test]
fn str_pad_preserves_php_byte_behavior() {
    let right = Box::into_raw(Box::new(EchoString {
        bytes: "ID".as_bytes().to_vec(),
    }));
    let left = Box::into_raw(Box::new(EchoString {
        bytes: "42".as_bytes().to_vec(),
    }));
    let both = Box::into_raw(Box::new(EchoString {
        bytes: "tag".as_bytes().to_vec(),
    }));
    let multi_left = Box::into_raw(Box::new(EchoString {
        bytes: "42".as_bytes().to_vec(),
    }));
    let multi_both = Box::into_raw(Box::new(EchoString {
        bytes: "go".as_bytes().to_vec(),
    }));
    let shorter = Box::into_raw(Box::new(EchoString {
        bytes: "already".as_bytes().to_vec(),
    }));
    let zero = Box::into_raw(Box::new(EchoString {
        bytes: "0".as_bytes().to_vec(),
    }));
    let dash = Box::into_raw(Box::new(EchoString {
        bytes: "-".as_bytes().to_vec(),
    }));
    let ab = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(right),
            EchoValue::int(6),
            EchoValue::string(zero),
            EchoValue::int(1),
        )
        .string_bytes(),
        Some("ID0000".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(left),
            EchoValue::int(5),
            EchoValue::string(zero),
            EchoValue::int(0),
        )
        .string_bytes(),
        Some("00042".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(both),
            EchoValue::int(8),
            EchoValue::string(dash),
            EchoValue::int(2),
        )
        .string_bytes(),
        Some("--tag---".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(multi_left),
            EchoValue::int(7),
            EchoValue::string(ab),
            EchoValue::int(0),
        )
        .string_bytes(),
        Some("ababa42".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(multi_both),
            EchoValue::int(7),
            EchoValue::string(ab),
            EchoValue::int(2),
        )
        .string_bytes(),
        Some("abgoaba".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_str_pad(
            EchoValue::string(shorter),
            EchoValue::int(3),
            EchoValue::string(zero),
            EchoValue::int(0),
        )
        .string_bytes(),
        Some("already".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(right));
        drop(Box::from_raw(left));
        drop(Box::from_raw(both));
        drop(Box::from_raw(multi_left));
        drop(Box::from_raw(multi_both));
        drop(Box::from_raw(shorter));
        drop(Box::from_raw(zero));
        drop(Box::from_raw(dash));
        drop(Box::from_raw(ab));
    }
}

#[test]
fn string_chunk_builtins_preserve_php_byte_behavior() {
    let split = Box::into_raw(Box::new(EchoString {
        bytes: "abcde".as_bytes().to_vec(),
    }));
    let empty_split = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let chunk = Box::into_raw(Box::new(EchoString {
        bytes: "abcde".as_bytes().to_vec(),
    }));
    let empty_chunk = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let pipe = Box::into_raw(Box::new(EchoString {
        bytes: "|".as_bytes().to_vec(),
    }));
    let crlf = Box::into_raw(Box::new(EchoString {
        bytes: "\r\n".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_implode(
            EchoValue::string(pipe),
            echo_php_str_split(EchoValue::string(split), EchoValue::int(2)),
        )
        .string_bytes(),
        Some("ab|cd|e".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_implode(
            EchoValue::string(pipe),
            echo_php_str_split(EchoValue::string(empty_split), EchoValue::int(1)),
        )
        .string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_chunk_split(
            EchoValue::string(chunk),
            EchoValue::int(2),
            EchoValue::string(pipe)
        )
        .string_bytes(),
        Some("ab|cd|e|".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_chunk_split(
            EchoValue::string(empty_chunk),
            EchoValue::int(2),
            EchoValue::string(pipe),
        )
        .string_bytes(),
        Some("|".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_chunk_split(
            EchoValue::int(12345),
            EchoValue::string(Box::into_raw(Box::new(EchoString {
                bytes: "2".as_bytes().to_vec(),
            }))),
            EchoValue::string(crlf),
        )
        .string_bytes(),
        Some("12\r\n34\r\n5\r\n".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(split));
        drop(Box::from_raw(empty_split));
        drop(Box::from_raw(chunk));
        drop(Box::from_raw(empty_chunk));
        drop(Box::from_raw(pipe));
        drop(Box::from_raw(crlf));
    }
}

#[test]
fn substr_preserves_php_byte_behavior() {
    let positive = Box::into_raw(Box::new(EchoString {
        bytes: "Echo PHP".as_bytes().to_vec(),
    }));
    let out_of_range = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_offset = Box::into_raw(Box::new(EchoString {
        bytes: "1".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let negative = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_substr(EchoValue::string(positive), EchoValue::int(5)).string_bytes(),
        Some("PHP".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_substr(EchoValue::string(out_of_range), EchoValue::int(99)).string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_substr(EchoValue::string(negative), EchoValue::int(-2)).string_bytes(),
        Some("ef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_substr(
            EchoValue::string(non_ascii),
            EchoValue::string(numeric_offset)
        )
        .string_bytes(),
        Some(vec![0x84, b'c', b'h', b'o'])
    );

    unsafe {
        drop(Box::from_raw(positive));
        drop(Box::from_raw(out_of_range));
        drop(Box::from_raw(numeric_offset));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(negative));
    }
}

#[test]
fn strpos_preserves_php_byte_behavior() {
    let found_at_zero = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let found_later = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_needle = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));
    let needle_later = Box::into_raw(Box::new(EchoString {
        bytes: "cd".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "c".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strpos(
            EchoValue::string(found_at_zero),
            EchoValue::string(needle_start)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strpos(
            EchoValue::string(found_later),
            EchoValue::string(needle_later)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strpos(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strpos(EchoValue::string(numeric_needle), EchoValue::int(34)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strpos(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::int(2)
    );

    unsafe {
        drop(Box::from_raw(found_at_zero));
        drop(Box::from_raw(found_later));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_later));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn stripos_preserves_php_ascii_case_insensitive_byte_behavior() {
    let found_at_zero = Box::into_raw(Box::new(EchoString {
        bytes: "ABC".as_bytes().to_vec(),
    }));
    let found_later = Box::into_raw(Box::new(EchoString {
        bytes: "xxEcho".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_needle = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let needle_later = Box::into_raw(Box::new(EchoString {
        bytes: "ECHO".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "XY".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ä".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_stripos(
            EchoValue::string(found_at_zero),
            EchoValue::string(needle_start)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_stripos(
            EchoValue::string(found_later),
            EchoValue::string(needle_later)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_stripos(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_stripos(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_stripos(EchoValue::string(numeric_needle), EchoValue::int(34)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_stripos(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(found_at_zero));
        drop(Box::from_raw(found_later));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(numeric_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_later));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn strrpos_preserves_php_byte_behavior() {
    let repeated_start = Box::into_raw(Box::new(EchoString {
        bytes: "abcabc".as_bytes().to_vec(),
    }));
    let repeated_end = Box::into_raw(Box::new(EchoString {
        bytes: "abcabc".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_needle = Box::into_raw(Box::new(EchoString {
        bytes: "1234545".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächocho".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));
    let needle_end = Box::into_raw(Box::new(EchoString {
        bytes: "bc".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "c".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(repeated_start),
            EchoValue::string(needle_start)
        ),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(repeated_end),
            EchoValue::string(needle_end)
        ),
        EchoValue::int(4)
    );
    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::int(6)
    );
    assert_eq!(
        echo_php_strrpos(EchoValue::string(numeric_needle), EchoValue::int(45)),
        EchoValue::int(5)
    );
    assert_eq!(
        echo_php_strrpos(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::int(5)
    );

    unsafe {
        drop(Box::from_raw(repeated_start));
        drop(Box::from_raw(repeated_end));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(numeric_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_end));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn strripos_preserves_php_ascii_case_insensitive_byte_behavior() {
    let repeated_start = Box::into_raw(Box::new(EchoString {
        bytes: "abABcd".as_bytes().to_vec(),
    }));
    let repeated_end = Box::into_raw(Box::new(EchoString {
        bytes: "abcABC".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric_needle = Box::into_raw(Box::new(EchoString {
        bytes: "1234545".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "aB".as_bytes().to_vec(),
    }));
    let needle_end = Box::into_raw(Box::new(EchoString {
        bytes: "BC".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "XY".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ä".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strripos(
            EchoValue::string(repeated_start),
            EchoValue::string(needle_start)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strripos(
            EchoValue::string(repeated_end),
            EchoValue::string(needle_end)
        ),
        EchoValue::int(4)
    );
    assert_eq!(
        echo_php_strripos(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strripos(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::int(6)
    );
    assert_eq!(
        echo_php_strripos(EchoValue::string(numeric_needle), EchoValue::int(45)),
        EchoValue::int(5)
    );
    assert_eq!(
        echo_php_strripos(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(repeated_start));
        drop(Box::from_raw(repeated_end));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(numeric_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_end));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn strstr_preserves_php_byte_behavior() {
    let email = Box::into_raw(Box::new(EchoString {
        bytes: "name@example.com".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let at_start = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_at = Box::into_raw(Box::new(EchoString {
        bytes: "@".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "c".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strstr(EchoValue::string(email), EchoValue::string(needle_at)).string_bytes(),
        Some("@example.com".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strstr(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strstr(EchoValue::string(at_start), EchoValue::string(needle_start))
            .string_bytes(),
        Some("abcdef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strstr(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
        Some("345".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strstr(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        )
        .string_bytes(),
        Some("abcdef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strstr(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        )
        .string_bytes(),
        Some("cho".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(email));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(at_start));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_at));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn stristr_preserves_php_ascii_case_insensitive_byte_behavior() {
    let email = Box::into_raw(Box::new(EchoString {
        bytes: "USER@EXAMPLE.com".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let at_start = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let needle_email = Box::into_raw(Box::new(EchoString {
        bytes: "e".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "XY".as_bytes().to_vec(),
    }));
    let needle_start = Box::into_raw(Box::new(EchoString {
        bytes: "AB".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ä".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_stristr(EchoValue::string(email), EchoValue::string(needle_email)).string_bytes(),
        Some("ER@EXAMPLE.com".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_stristr(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_stristr(EchoValue::string(at_start), EchoValue::string(needle_start))
            .string_bytes(),
        Some("abcdef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_stristr(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
        Some("345".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_stristr(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        )
        .string_bytes(),
        Some("abcdef".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_stristr(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(email));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(at_start));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(needle_email));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_start));
        drop(Box::from_raw(needle_empty));
        drop(Box::from_raw(needle_non_ascii));
    }
}

#[test]
fn strrchr_preserves_php_byte_behavior() {
    let email = Box::into_raw(Box::new(EchoString {
        bytes: "name@example.com".as_bytes().to_vec(),
    }));
    let repeated = Box::into_raw(Box::new(EchoString {
        bytes: "abcabc".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "1234545".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ÄchoÄ".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let needle_at = Box::into_raw(Box::new(EchoString {
        bytes: "@".as_bytes().to_vec(),
    }));
    let needle_repeated = Box::into_raw(Box::new(EchoString {
        bytes: "bc".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_strrchr(EchoValue::string(email), EchoValue::string(needle_at)).string_bytes(),
        Some("@example.com".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strrchr(
            EchoValue::string(repeated),
            EchoValue::string(needle_repeated)
        )
        .string_bytes(),
        Some("bc".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strrchr(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strrchr(EchoValue::string(numeric), EchoValue::int(45)).string_bytes(),
        Some("45".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strrchr(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        )
        .string_bytes(),
        Some("Ä".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strrchr(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(email));
        drop(Box::from_raw(repeated));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(needle_at));
        drop(Box::from_raw(needle_repeated));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_non_ascii));
        drop(Box::from_raw(needle_empty));
    }
}

#[test]
fn strpbrk_preserves_php_byte_mask_behavior() {
    let text = Box::into_raw(Box::new(EchoString {
        bytes: "This is a Simple text.".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let empty_mask = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let mask_text = Box::into_raw(Box::new(EchoString {
        bytes: "mi".as_bytes().to_vec(),
    }));
    let mask_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let mask_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_strpbrk(EchoValue::string(text), EchoValue::string(mask_text)).string_bytes(),
        Some("is is a Simple text.".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strpbrk(EchoValue::string(missing), EchoValue::string(mask_missing)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_strpbrk(EchoValue::string(numeric), EchoValue::int(34)).string_bytes(),
        Some("345".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strpbrk(
            EchoValue::string(non_ascii),
            EchoValue::string(mask_non_ascii)
        )
        .string_bytes(),
        Some("Ächo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_strpbrk(EchoValue::string(empty_mask), EchoValue::string(mask_empty)),
        EchoValue::error()
    );

    unsafe {
        drop(Box::from_raw(text));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty_mask));
        drop(Box::from_raw(mask_text));
        drop(Box::from_raw(mask_missing));
        drop(Box::from_raw(mask_non_ascii));
        drop(Box::from_raw(mask_empty));
    }
}

#[test]
fn strspn_preserves_php_byte_mask_behavior() {
    let digits = Box::into_raw(Box::new(EchoString {
        bytes: "42 is the answer".as_bytes().to_vec(),
    }));
    let prefix = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let mask_digits = Box::into_raw(Box::new(EchoString {
        bytes: "0123456789".as_bytes().to_vec(),
    }));
    let mask_prefix = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let mask_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xyz".as_bytes().to_vec(),
    }));
    let mask_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Äc".as_bytes().to_vec(),
    }));
    let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_strspn(EchoValue::string(digits), EchoValue::string(mask_digits)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strspn(EchoValue::string(prefix), EchoValue::string(mask_prefix)),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_strspn(EchoValue::string(missing), EchoValue::string(mask_missing)),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strspn(EchoValue::string(numeric), EchoValue::int(12)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strspn(
            EchoValue::string(non_ascii),
            EchoValue::string(mask_non_ascii)
        ),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_strspn(EchoValue::string(empty), EchoValue::string(mask_empty)),
        EchoValue::int(0)
    );

    unsafe {
        drop(Box::from_raw(digits));
        drop(Box::from_raw(prefix));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty));
        drop(Box::from_raw(mask_digits));
        drop(Box::from_raw(mask_prefix));
        drop(Box::from_raw(mask_missing));
        drop(Box::from_raw(mask_non_ascii));
        drop(Box::from_raw(mask_empty));
    }
}

#[test]
fn strcspn_preserves_php_byte_mask_behavior() {
    let no_match = Box::into_raw(Box::new(EchoString {
        bytes: "abcd".as_bytes().to_vec(),
    }));
    let end_match = Box::into_raw(Box::new(EchoString {
        bytes: "abcd".as_bytes().to_vec(),
    }));
    let middle_match = Box::into_raw(Box::new(EchoString {
        bytes: "abcd".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "12345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ächo".as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let mask_no_match = Box::into_raw(Box::new(EchoString {
        bytes: "x".as_bytes().to_vec(),
    }));
    let mask_end_match = Box::into_raw(Box::new(EchoString {
        bytes: "d".as_bytes().to_vec(),
    }));
    let mask_middle_match = Box::into_raw(Box::new(EchoString {
        bytes: "bd".as_bytes().to_vec(),
    }));
    let mask_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "c".as_bytes().to_vec(),
    }));
    let mask_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_strcspn(
            EchoValue::string(no_match),
            EchoValue::string(mask_no_match)
        ),
        EchoValue::int(4)
    );
    assert_eq!(
        echo_php_strcspn(
            EchoValue::string(end_match),
            EchoValue::string(mask_end_match)
        ),
        EchoValue::int(3)
    );
    assert_eq!(
        echo_php_strcspn(
            EchoValue::string(middle_match),
            EchoValue::string(mask_middle_match)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcspn(EchoValue::string(numeric), EchoValue::int(34)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strcspn(
            EchoValue::string(non_ascii),
            EchoValue::string(mask_non_ascii)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_strcspn(EchoValue::string(empty), EchoValue::string(mask_empty)),
        EchoValue::int(3)
    );

    unsafe {
        drop(Box::from_raw(no_match));
        drop(Box::from_raw(end_match));
        drop(Box::from_raw(middle_match));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty));
        drop(Box::from_raw(mask_no_match));
        drop(Box::from_raw(mask_end_match));
        drop(Box::from_raw(mask_middle_match));
        drop(Box::from_raw(mask_non_ascii));
        drop(Box::from_raw(mask_empty));
    }
}

#[test]
fn substr_count_preserves_php_non_overlapping_byte_behavior() {
    let words = Box::into_raw(Box::new(EchoString {
        bytes: "This is a test".as_bytes().to_vec(),
    }));
    let repeated = Box::into_raw(Box::new(EchoString {
        bytes: "aaaa".as_bytes().to_vec(),
    }));
    let missing = Box::into_raw(Box::new(EchoString {
        bytes: "abcdef".as_bytes().to_vec(),
    }));
    let numeric = Box::into_raw(Box::new(EchoString {
        bytes: "1234512345".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "ÄchoÄ".as_bytes().to_vec(),
    }));
    let empty_needle = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let needle_words = Box::into_raw(Box::new(EchoString {
        bytes: "is".as_bytes().to_vec(),
    }));
    let needle_repeated = Box::into_raw(Box::new(EchoString {
        bytes: "aa".as_bytes().to_vec(),
    }));
    let needle_missing = Box::into_raw(Box::new(EchoString {
        bytes: "xy".as_bytes().to_vec(),
    }));
    let needle_non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let needle_empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_substr_count(EchoValue::string(words), EchoValue::string(needle_words)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_substr_count(
            EchoValue::string(repeated),
            EchoValue::string(needle_repeated)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_substr_count(
            EchoValue::string(missing),
            EchoValue::string(needle_missing)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_substr_count(EchoValue::string(numeric), EchoValue::int(45)),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_substr_count(
            EchoValue::string(non_ascii),
            EchoValue::string(needle_non_ascii)
        ),
        EchoValue::int(2)
    );
    assert_eq!(
        echo_php_substr_count(
            EchoValue::string(empty_needle),
            EchoValue::string(needle_empty)
        ),
        EchoValue::error()
    );

    unsafe {
        drop(Box::from_raw(words));
        drop(Box::from_raw(repeated));
        drop(Box::from_raw(missing));
        drop(Box::from_raw(numeric));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(empty_needle));
        drop(Box::from_raw(needle_words));
        drop(Box::from_raw(needle_repeated));
        drop(Box::from_raw(needle_missing));
        drop(Box::from_raw(needle_non_ascii));
        drop(Box::from_raw(needle_empty));
    }
}

#[test]
fn strcmp_preserves_php_byte_sign_behavior() {
    let less_left = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let less_right = Box::into_raw(Box::new(EchoString {
        bytes: "b".as_bytes().to_vec(),
    }));
    let greater_left = Box::into_raw(Box::new(EchoString {
        bytes: "b".as_bytes().to_vec(),
    }));
    let greater_right = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let equal_left = Box::into_raw(Box::new(EchoString {
        bytes: "same".as_bytes().to_vec(),
    }));
    let equal_right = Box::into_raw(Box::new(EchoString {
        bytes: "same".as_bytes().to_vec(),
    }));
    let prefix_left = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let prefix_right = Box::into_raw(Box::new(EchoString {
        bytes: "ab".as_bytes().to_vec(),
    }));
    let numeric_left = Box::into_raw(Box::new(EchoString {
        bytes: "123".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strcmp(EchoValue::string(less_left), EchoValue::string(less_right)),
        EchoValue::int(-1)
    );
    assert_eq!(
        echo_php_strcmp(
            EchoValue::string(greater_left),
            EchoValue::string(greater_right)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcmp(
            EchoValue::string(equal_left),
            EchoValue::string(equal_right)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strcmp(
            EchoValue::string(prefix_left),
            EchoValue::string(prefix_right)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcmp(EchoValue::string(numeric_left), EchoValue::int(123)),
        EchoValue::int(0)
    );

    unsafe {
        drop(Box::from_raw(less_left));
        drop(Box::from_raw(less_right));
        drop(Box::from_raw(greater_left));
        drop(Box::from_raw(greater_right));
        drop(Box::from_raw(equal_left));
        drop(Box::from_raw(equal_right));
        drop(Box::from_raw(prefix_left));
        drop(Box::from_raw(prefix_right));
        drop(Box::from_raw(numeric_left));
    }
}

#[test]
fn strcasecmp_preserves_php_ascii_case_insensitive_behavior() {
    let equal_left = Box::into_raw(Box::new(EchoString {
        bytes: "Echo".as_bytes().to_vec(),
    }));
    let equal_right = Box::into_raw(Box::new(EchoString {
        bytes: "echo".as_bytes().to_vec(),
    }));
    let less_left = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let less_right = Box::into_raw(Box::new(EchoString {
        bytes: "B".as_bytes().to_vec(),
    }));
    let greater_left = Box::into_raw(Box::new(EchoString {
        bytes: "B".as_bytes().to_vec(),
    }));
    let greater_right = Box::into_raw(Box::new(EchoString {
        bytes: "a".as_bytes().to_vec(),
    }));
    let prefix_left = Box::into_raw(Box::new(EchoString {
        bytes: "abc".as_bytes().to_vec(),
    }));
    let prefix_right = Box::into_raw(Box::new(EchoString {
        bytes: "AB".as_bytes().to_vec(),
    }));
    let numeric_left = Box::into_raw(Box::new(EchoString {
        bytes: "123".as_bytes().to_vec(),
    }));
    let non_ascii_left = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let non_ascii_right = Box::into_raw(Box::new(EchoString {
        bytes: "ä".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_strcasecmp(
            EchoValue::string(equal_left),
            EchoValue::string(equal_right)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strcasecmp(EchoValue::string(less_left), EchoValue::string(less_right)),
        EchoValue::int(-1)
    );
    assert_eq!(
        echo_php_strcasecmp(
            EchoValue::string(greater_left),
            EchoValue::string(greater_right)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcasecmp(
            EchoValue::string(prefix_left),
            EchoValue::string(prefix_right)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strcasecmp(EchoValue::string(numeric_left), EchoValue::int(123)),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strcasecmp(
            EchoValue::string(non_ascii_left),
            EchoValue::string(non_ascii_right)
        ),
        EchoValue::int(-32)
    );

    unsafe {
        drop(Box::from_raw(equal_left));
        drop(Box::from_raw(equal_right));
        drop(Box::from_raw(less_left));
        drop(Box::from_raw(less_right));
        drop(Box::from_raw(greater_left));
        drop(Box::from_raw(greater_right));
        drop(Box::from_raw(prefix_left));
        drop(Box::from_raw(prefix_right));
        drop(Box::from_raw(numeric_left));
        drop(Box::from_raw(non_ascii_left));
        drop(Box::from_raw(non_ascii_right));
    }
}

#[test]
fn strncmp_builtins_preserve_php_prefix_behavior() {
    let abc = Box::into_raw(Box::new(EchoString {
        bytes: b"abc".to_vec(),
    }));
    let abd = Box::into_raw(Box::new(EchoString {
        bytes: b"abd".to_vec(),
    }));
    let ab = Box::into_raw(Box::new(EchoString {
        bytes: b"ab".to_vec(),
    }));
    let upper_abd = Box::into_raw(Box::new(EchoString {
        bytes: b"ABD".to_vec(),
    }));

    assert_eq!(
        echo_php_strncmp(
            EchoValue::string(abc),
            EchoValue::string(abd),
            EchoValue::int(2)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strncmp(
            EchoValue::string(abc),
            EchoValue::string(abd),
            EchoValue::int(3)
        ),
        EchoValue::int(-1)
    );
    assert_eq!(
        echo_php_strncmp(
            EchoValue::string(abc),
            EchoValue::string(ab),
            EchoValue::int(3)
        ),
        EchoValue::int(1)
    );
    assert_eq!(
        echo_php_strncasecmp(
            EchoValue::string(abc),
            EchoValue::string(upper_abd),
            EchoValue::int(2)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_strncasecmp(
            EchoValue::string(abc),
            EchoValue::string(upper_abd),
            EchoValue::int(3)
        ),
        EchoValue::int(-1)
    );

    unsafe {
        drop(Box::from_raw(abc));
        drop(Box::from_raw(abd));
        drop(Box::from_raw(ab));
        drop(Box::from_raw(upper_abd));
    }
}

#[test]
fn substr_compare_preserves_php_offset_length_and_case_behavior() {
    let haystack = Box::into_raw(Box::new(EchoString {
        bytes: b"abcde".to_vec(),
    }));
    let needle_bc = Box::into_raw(Box::new(EchoString {
        bytes: b"bc".to_vec(),
    }));
    let needle_bcg = Box::into_raw(Box::new(EchoString {
        bytes: b"bcg".to_vec(),
    }));
    let needle_upper_bc = Box::into_raw(Box::new(EchoString {
        bytes: b"BC".to_vec(),
    }));
    let needle_cd = Box::into_raw(Box::new(EchoString {
        bytes: b"cd".to_vec(),
    }));

    assert_eq!(
        echo_php_substr_compare(
            EchoValue::string(haystack),
            EchoValue::string(needle_bc),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::bool(false)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_substr_compare(
            EchoValue::string(haystack),
            EchoValue::string(needle_bcg),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::bool(false)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_substr_compare(
            EchoValue::string(haystack),
            EchoValue::string(needle_upper_bc),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::bool(true)
        ),
        EchoValue::int(0)
    );
    assert_eq!(
        echo_php_substr_compare(
            EchoValue::string(haystack),
            EchoValue::string(needle_cd),
            EchoValue::int(1),
            EchoValue::int(2),
            EchoValue::bool(false)
        ),
        EchoValue::int(-1)
    );

    unsafe {
        drop(Box::from_raw(haystack));
        drop(Box::from_raw(needle_bc));
        drop(Box::from_raw(needle_bcg));
        drop(Box::from_raw(needle_upper_bc));
        drop(Box::from_raw(needle_cd));
    }
}

#[test]
fn trim_builtins_strip_default_php_ascii_whitespace() {
    let trim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let ltrim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let rtrim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: " Ä ".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_trim(EchoValue::string(trim)).string_bytes(),
        Some("Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ltrim(EchoValue::string(ltrim)).string_bytes(),
        Some("Echo \n".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_rtrim(EchoValue::string(rtrim)).string_bytes(),
        Some("\t Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_trim(EchoValue::string(non_ascii)).string_bytes(),
        Some("Ä".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(trim));
        drop(Box::from_raw(ltrim));
        drop(Box::from_raw(rtrim));
        drop(Box::from_raw(non_ascii));
    }
}

#[test]
fn string_rewrite_builtins_preserve_php_byte_behavior() {
    let chop = Box::into_raw(Box::new(EchoString {
        bytes: b"invoice:1001\n".to_vec(),
    }));
    let quoted = Box::into_raw(Box::new(EchoString {
        bytes: b"a=b\nnext".to_vec(),
    }));
    let quoted_decode = Box::into_raw(Box::new(EchoString {
        bytes: b"a=3Db=0Anext".to_vec(),
    }));
    let nl2br = Box::into_raw(Box::new(EchoString {
        bytes: b"line1\nline2".to_vec(),
    }));
    let search = Box::into_raw(Box::new(EchoString {
        bytes: b"{{name}}".to_vec(),
    }));
    let replace = Box::into_raw(Box::new(EchoString {
        bytes: b"Ada".to_vec(),
    }));
    let subject = Box::into_raw(Box::new(EchoString {
        bytes: b"Hello {{name}}".to_vec(),
    }));
    let isearch = Box::into_raw(Box::new(EchoString {
        bytes: b"TOKEN".to_vec(),
    }));
    let ireplace = Box::into_raw(Box::new(EchoString {
        bytes: b"redacted".to_vec(),
    }));
    let isubject = Box::into_raw(Box::new(EchoString {
        bytes: b"token TOKEN".to_vec(),
    }));
    let tr_value = Box::into_raw(Box::new(EchoString {
        bytes: b"abc-123".to_vec(),
    }));
    let tr_from = Box::into_raw(Box::new(EchoString {
        bytes: b"abc123".to_vec(),
    }));
    let tr_to = Box::into_raw(Box::new(EchoString {
        bytes: b"xyz789".to_vec(),
    }));

    assert_eq!(
        echo_php_rtrim(EchoValue::string(chop)).string_bytes(),
        Some(b"invoice:1001".to_vec())
    );
    assert_eq!(
        echo_php_quoted_printable_encode(EchoValue::string(quoted)).string_bytes(),
        Some(b"a=3Db=0Anext".to_vec())
    );
    assert_eq!(
        echo_php_quoted_printable_decode(EchoValue::string(quoted_decode)).string_bytes(),
        Some(b"a=b\nnext".to_vec())
    );
    assert_eq!(
        echo_php_nl2br(EchoValue::string(nl2br), EchoValue::bool(false)).string_bytes(),
        Some(b"line1<br>\nline2".to_vec())
    );
    assert_eq!(
        echo_php_str_replace(
            EchoValue::string(search),
            EchoValue::string(replace),
            EchoValue::string(subject),
        )
        .string_bytes(),
        Some(b"Hello Ada".to_vec())
    );
    assert_eq!(
        echo_php_str_ireplace(
            EchoValue::string(isearch),
            EchoValue::string(ireplace),
            EchoValue::string(isubject),
        )
        .string_bytes(),
        Some(b"redacted redacted".to_vec())
    );
    assert_eq!(
        echo_php_strtr(
            EchoValue::string(tr_value),
            EchoValue::string(tr_from),
            EchoValue::string(tr_to),
        )
        .string_bytes(),
        Some(b"xyz-789".to_vec())
    );

    unsafe {
        drop(Box::from_raw(chop));
        drop(Box::from_raw(quoted));
        drop(Box::from_raw(quoted_decode));
        drop(Box::from_raw(nl2br));
        drop(Box::from_raw(search));
        drop(Box::from_raw(replace));
        drop(Box::from_raw(subject));
        drop(Box::from_raw(isearch));
        drop(Box::from_raw(ireplace));
        drop(Box::from_raw(isubject));
        drop(Box::from_raw(tr_value));
        drop(Box::from_raw(tr_from));
        drop(Box::from_raw(tr_to));
    }
}

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

#[test]
fn string_predicate_builtins_are_binary_safe_and_case_sensitive() {
    let haystack = Box::into_raw(Box::new(EchoString {
        bytes: "Echo PHP".as_bytes().to_vec(),
    }));
    let matching = Box::into_raw(Box::new(EchoString {
        bytes: "PHP".as_bytes().to_vec(),
    }));
    let mismatched_case = Box::into_raw(Box::new(EchoString {
        bytes: "php".as_bytes().to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: "Ä".as_bytes().to_vec(),
    }));
    let first_utf8_byte = Box::into_raw(Box::new(EchoString { bytes: vec![0xc3] }));

    assert_eq!(
        echo_php_str_contains(EchoValue::string(haystack), EchoValue::string(matching)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_str_contains(
            EchoValue::string(haystack),
            EchoValue::string(mismatched_case)
        ),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_str_contains(EchoValue::string(haystack), EchoValue::string(empty)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_str_starts_with(EchoValue::string(haystack), EchoValue::string(empty)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_str_ends_with(EchoValue::string(haystack), EchoValue::string(matching)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_str_contains(
            EchoValue::string(non_ascii),
            EchoValue::string(first_utf8_byte)
        ),
        EchoValue::bool(true)
    );

    unsafe {
        drop(Box::from_raw(haystack));
        drop(Box::from_raw(matching));
        drop(Box::from_raw(mismatched_case));
        drop(Box::from_raw(empty));
        drop(Box::from_raw(non_ascii));
        drop(Box::from_raw(first_utf8_byte));
    }
}

#[test]
fn string_escape_builtins_preserve_php_byte_behavior() {
    let quoted = Box::into_raw(Box::new(EchoString {
        bytes: vec![b'A', b'\'', b'"', b'\\', b'B'],
    }));
    let slashed_zero = Box::into_raw(Box::new(EchoString {
        bytes: b"\\0".to_vec(),
    }));
    let meta = Box::into_raw(Box::new(EchoString {
        bytes: b".\\+*?[^]($)".to_vec(),
    }));
    let empty = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));

    assert_eq!(
        echo_php_addslashes(EchoValue::string(quoted)).string_bytes(),
        Some(b"A\\'\\\"\\\\B".to_vec())
    );
    assert_eq!(
        echo_php_stripslashes(EchoValue::string(slashed_zero)).string_bytes(),
        Some(vec![0])
    );
    assert_eq!(
        echo_php_quotemeta(EchoValue::string(meta)).string_bytes(),
        Some(b"\\.\\\\\\+\\*\\?\\[\\^\\]\\(\\$\\)".to_vec())
    );
    assert_eq!(
        echo_php_quotemeta(EchoValue::string(empty)),
        EchoValue::bool(false)
    );

    unsafe {
        drop(Box::from_raw(quoted));
        drop(Box::from_raw(slashed_zero));
        drop(Box::from_raw(meta));
        drop(Box::from_raw(empty));
    }
}

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
}
