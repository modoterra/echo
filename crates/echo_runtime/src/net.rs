use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

use bytes::Bytes;
use socket2::{Domain, Protocol, Socket, Type};

use crate::{EchoString, EchoValue, echo_value_object_new, echo_value_object_set};

#[derive(Debug)]
pub struct EchoTcpListener {
    inner: TcpListener,
}

#[derive(Debug)]
pub struct EchoTcpConnection {
    inner: Option<TcpStream>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoNetBuffer {
    bytes: Bytes,
}

impl EchoTcpListener {
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }
}

impl EchoTcpConnection {
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream()?.peer_addr()
    }

    fn stream(&self) -> io::Result<&TcpStream> {
        self.inner
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "connection is closed"))
    }

    fn stream_mut(&mut self) -> io::Result<&mut TcpStream> {
        self.inner
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "connection is closed"))
    }
}

impl EchoNetBuffer {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }
}

pub fn listen<A>(addr: A) -> io::Result<EchoTcpListener>
where
    A: ToSocketAddrs,
{
    let addr = addr.to_socket_addrs()?.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "address resolved to nothing")
    })?;

    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;

    Ok(EchoTcpListener {
        inner: socket.into(),
    })
}

pub fn accept(listener: &EchoTcpListener) -> io::Result<EchoTcpConnection> {
    let (stream, _) = listener.inner.accept()?;

    Ok(EchoTcpConnection {
        inner: Some(stream),
    })
}

pub fn connect<A>(addr: A) -> io::Result<EchoTcpConnection>
where
    A: ToSocketAddrs,
{
    TcpStream::connect(addr).map(|stream| EchoTcpConnection {
        inner: Some(stream),
    })
}

pub fn read(connection: &mut EchoTcpConnection, max_bytes: usize) -> io::Result<EchoNetBuffer> {
    let mut bytes = vec![0; max_bytes];
    let read = connection.stream_mut()?.read(&mut bytes)?;
    bytes.truncate(read);

    Ok(EchoNetBuffer {
        bytes: Bytes::from(bytes),
    })
}

pub fn write(connection: &mut EchoTcpConnection, bytes: &[u8]) -> io::Result<usize> {
    connection.stream_mut()?.write(bytes)
}

pub fn close(connection: &mut EchoTcpConnection) -> io::Result<()> {
    let Some(stream) = connection.inner.take() else {
        return Ok(());
    };

    stream.shutdown(std::net::Shutdown::Both)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_listen(address: EchoValue) -> EchoValue {
    let Some(bytes) = address.string_bytes() else {
        return EchoValue::error();
    };
    let Ok(address) = String::from_utf8(bytes) else {
        return EchoValue::error();
    };

    match listen(address) {
        Ok(listener) => EchoValue::tcp_listener(Box::into_raw(Box::new(listener))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_connect(address: EchoValue) -> EchoValue {
    let Some(bytes) = address.string_bytes() else {
        return EchoValue::error();
    };
    let Ok(address) = String::from_utf8(bytes) else {
        return EchoValue::error();
    };

    match connect(address) {
        Ok(connection) => EchoValue::tcp_connection(Box::into_raw(Box::new(connection))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_accept(listener: EchoValue) -> EchoValue {
    let Some(listener) = listener.as_tcp_listener_ref() else {
        return EchoValue::error();
    };

    match accept(listener) {
        Ok(connection) => EchoValue::tcp_connection(Box::into_raw(Box::new(connection))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_read(connection: EchoValue, max_bytes: EchoValue) -> EchoValue {
    let Some(connection) = connection.as_tcp_connection_mut() else {
        return EchoValue::error();
    };
    if !max_bytes.is_int() {
        return EchoValue::error();
    }

    match read(connection, max_bytes.payload as usize) {
        Ok(bytes) => EchoValue::string(Box::into_raw(Box::new(EchoString::new(
            bytes.into_bytes().to_vec(),
        )))),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_write(connection: EchoValue, data: EchoValue) -> EchoValue {
    let Some(connection) = connection.as_tcp_connection_mut() else {
        return EchoValue::error();
    };
    let Some(bytes) = data.string_bytes() else {
        return EchoValue::error();
    };

    match write(connection, &bytes) {
        Ok(written) => EchoValue::int(written as i64),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_net_close(connection: EchoValue) -> EchoValue {
    let Some(connection) = connection.as_tcp_connection_mut() else {
        return EchoValue::error();
    };

    match close(connection) {
        Ok(()) => EchoValue::null(),
        Err(_) => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_http_response_text(body: EchoValue) -> EchoValue {
    let Some(body) = body.string_bytes() else {
        return EchoValue::error();
    };

    let Ok(response) = http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "text/plain")
        .header(http::header::CONTENT_LENGTH, body.len().to_string())
        .header(http::header::CONNECTION, "close")
        .body(body)
    else {
        return EchoValue::error();
    };

    let (parts, body) = response.into_parts();
    let reason = parts.status.canonical_reason().unwrap_or("OK");
    let mut bytes = format!("HTTP/1.1 {} {reason}\r\n", parts.status.as_u16()).into_bytes();

    for (name, value) in &parts.headers {
        let Ok(value) = value.to_str() else {
            return EchoValue::error();
        };
        bytes.extend_from_slice(name.as_str().as_bytes());
        bytes.extend_from_slice(b": ");
        bytes.extend_from_slice(value.as_bytes());
        bytes.extend_from_slice(b"\r\n");
    }

    bytes.extend_from_slice(b"\r\n");
    bytes.extend_from_slice(&body);

    EchoValue::string(Box::into_raw(Box::new(EchoString::new(bytes))))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_http_read_request(connection: EchoValue) -> EchoValue {
    let Some(connection) = connection.as_tcp_connection_mut() else {
        return EchoValue::error();
    };
    let Ok(buffer) = read(connection, 4096) else {
        return EchoValue::error();
    };
    let Ok(request) = std::str::from_utf8(buffer.as_bytes()) else {
        return EchoValue::error();
    };
    let Some(request_line) = request.lines().next() else {
        return EchoValue::error();
    };
    let mut parts = request_line.split_whitespace();
    let (Some(_method), Some(path), Some(_version)) = (parts.next(), parts.next(), parts.next())
    else {
        return EchoValue::error();
    };

    let object = echo_value_object_new();
    let path = EchoValue::string(Box::into_raw(Box::new(EchoString::new(
        path.as_bytes().to_vec(),
    ))));
    unsafe { echo_value_object_set(object, b"path".as_ptr(), 4, path) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::thread;

    #[test]
    fn accepts_reads_writes_and_closes_loopback_connection() {
        let listener = listen("127.0.0.1:0").expect("listen");
        let addr = listener.local_addr().expect("local addr");

        let client = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).expect("connect");
            stream.write_all(b"ping").expect("client write");

            let mut reply = [0; 4];
            stream.read_exact(&mut reply).expect("client read");
            reply
        });

        let mut connection = accept(&listener).expect("accept");
        let request = read(&mut connection, 1024).expect("read");
        assert_eq!(request.as_bytes(), b"ping");

        let written = write(&mut connection, b"pong").expect("write");
        assert_eq!(written, 4);

        close(&mut connection).expect("close");
        assert!(write(&mut connection, b"again").is_err());

        assert_eq!(client.join().expect("client thread"), *b"pong");
    }
}
