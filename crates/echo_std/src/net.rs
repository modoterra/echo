use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

use bytes::Bytes;
use echo_runtime::net::{self, EchoTcpConnection, EchoTcpListener};

#[derive(Debug)]
pub struct TcpServer {
    listener: EchoTcpListener,
}

#[derive(Debug)]
pub struct TcpConnection {
    connection: EchoTcpConnection,
}

impl TcpServer {
    pub fn listen<A>(addr: A) -> io::Result<Self>
    where
        A: ToSocketAddrs,
    {
        net::listen(addr).map(|listener| Self { listener })
    }

    pub fn accept(&self) -> io::Result<TcpConnection> {
        net::accept(&self.listener).map(|connection| TcpConnection { connection })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }
}

impl TcpConnection {
    pub fn read(&mut self, max_bytes: usize) -> io::Result<Bytes> {
        net::read(&mut self.connection, max_bytes).map(|buffer| buffer.into_bytes())
    }

    pub fn write(&mut self, bytes: impl AsRef<[u8]>) -> io::Result<usize> {
        net::write(&mut self.connection, bytes.as_ref())
    }

    pub fn close(&mut self) -> io::Result<()> {
        net::close(&mut self.connection)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.connection.peer_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;

    #[test]
    fn tcp_server_wraps_runtime_net_primitives() {
        let server = TcpServer::listen("127.0.0.1:0").expect("listen");
        let addr = server.local_addr().expect("local addr");

        let client = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).expect("connect");
            stream.write_all(b"hello").expect("write request");

            let mut reply = [0; 2];
            stream.read_exact(&mut reply).expect("read reply");
            reply
        });

        let mut connection = server.accept().expect("accept");
        assert_eq!(
            connection.read(16).expect("read"),
            Bytes::from_static(b"hello")
        );
        assert_eq!(connection.write("ok").expect("write"), 2);
        connection.close().expect("close");

        assert_eq!(client.join().expect("client thread"), *b"ok");
    }
}
