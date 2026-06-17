use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};

use bytes::Bytes;
use socket2::{Domain, Protocol, Socket, Type};

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
