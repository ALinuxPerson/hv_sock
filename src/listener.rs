use std::io;
use crate::{Socket, SocketAddr, Stream};

/// A socket server, listening for connections.
///
/// You can accept connections by calling `accept` or by iterating
/// over the `Incoming` iterator.
#[derive(Debug)]
pub struct Listener(Socket);

impl Listener {
    /// Creates a new `Listener` bound to the specified address.
    ///
    /// This will create a new socket, bind it to the given `SocketAddr`,
    /// and then listen for incoming connections.
    pub fn bind(addr: &SocketAddr) -> io::Result<Self> {
        let socket = Socket::new()?;
        socket.bind(addr)?;
        socket.listen()?;
        Ok(Self(socket))
    }

    /// Accepts a new incoming connection from this listener.
    ///
    /// This function will block the calling thread until a new connection is established.
    /// When a connection is established, the corresponding `Stream` and the remote
    /// peer's `SocketAddr` will be returned.
    pub fn accept(&self) -> io::Result<(Stream, SocketAddr)> {
        self.0.accept()
            .map(|(sock, addr)| (unsafe { Stream::from_socket_unchecked(sock) }, addr))
    }
}

impl Listener {
    /// Returns the local socket address of this listener.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned `Listener` is a reference to the same socket that this
    /// object references. Both handles will read and write the same socket,
    /// and options set on one listener will affect the other.
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

    /// Returns the value of the `SO_ERROR` option.
    ///
    /// This is useful for checking for asynchronous errors.
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Sets the non-blocking mode of the listener.
    ///
    /// If `nonblocking` is `true`, then `accept` will return an error with
    /// kind `io::ErrorKind::WouldBlock` if there are no pending connections.
    /// Otherwise, `accept` will block until a new connection is available.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

/// An iterator over the connections being received on a `Listener`.
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
    /// Returns an iterator over the connections being received on this listener.
    ///
    /// The iterator will yield instances of `io::Result<Stream>`.
    pub fn incoming(&self) -> Incoming {
        Incoming(self)
    }
}
