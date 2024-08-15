use std::io;
use crate::{Socket, SocketAddr, Stream};

#[derive(Debug)]
pub struct Listener(Socket);

impl Listener {
    pub fn bind(addr: &SocketAddr) -> io::Result<Self> {
        let socket = Socket::new()?;
        socket.bind(addr)?;
        socket.listen()?;
        Ok(Self(socket))
    }

    pub fn accept(&self) -> io::Result<(Stream, SocketAddr)> {
        self.0.accept()
            .map(|(sock, addr)| (unsafe { Stream::from_socket_unchecked(sock) }, addr))
    }
}

impl Listener {
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

#[derive(Debug)]
pub struct Incoming<'a>(&'a Listener);

impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<Stream>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.accept().map(|(stream, _)| stream))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

impl Listener {
    pub fn incoming(&self) -> Incoming {
        Incoming(self)
    }
}
