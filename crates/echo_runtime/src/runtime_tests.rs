use super::*;
use crate::collections::EchoArrayKey;
use crate::value::{ECHO_VALUE_TCP_CONNECTION, ECHO_VALUE_TCP_LISTENER};
use std::thread;

fn test_string_value(bytes: &[u8]) -> EchoValue {
    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes.to_vec()))))
}

fn assert_float_value(value: EchoValue, expected: f64) {
    assert_eq!(value.kind, ECHO_VALUE_FLOAT);
    assert!((f64::from_bits(value.payload) - expected).abs() < 0.000000000001);
}

mod arithmetic;
mod callable;
mod collections;
mod encoding;
mod filesystem;
mod math;
mod scalar;
mod string;
mod task;
mod value;

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
