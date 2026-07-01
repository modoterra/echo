use super::*;
use crate::value::{ECHO_VALUE_TCP_CONNECTION, ECHO_VALUE_TCP_LISTENER};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn loopback_exchange_available() -> bool {
    let Ok(listener) = TcpListener::bind("127.0.0.1:0") else {
        return false;
    };
    let Ok(address) = listener.local_addr() else {
        return false;
    };

    let client = thread::spawn(move || TcpStream::connect(address).is_ok());
    let accepted = listener.accept().is_ok();
    accepted && client.join().unwrap_or(false)
}

#[test]
fn std_net_abi_exchanges_loopback_bytes() {
    if !loopback_exchange_available() {
        eprintln!("skipping std.net loopback test because local TCP loopback is unavailable");
        return;
    }

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
