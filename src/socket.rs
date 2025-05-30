use std::io;
use std::io::{IoSlice, IoSliceMut, Read, Write};
use std::net::Shutdown;
use std::time::Duration;
use socket2::Type;
use crate::SocketAddr;
pub use crate::sys::{DOMAIN, PROTOCOL};

/// The type of the socket, which is `Type::STREAM`.
pub const TYPE: Type = Type::STREAM;

/// Represents a Hyper-V socket.
#[derive(Debug)]
pub struct Socket(socket2::Socket);

/// Common functions
impl Socket {
    /// Creates a new Hyper-V socket.
    pub fn new() -> io::Result<Self> {
        Ok(Self(socket2::Socket::new(DOMAIN, TYPE, Some(PROTOCOL))?))
    }

    /// Returns the local socket address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr().map(|addr| unsafe { SocketAddr::from_raw_unchecked(addr) })
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The new handle refers to the same socket as `self`.
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

    /// Gets the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing the
    /// error only if it is successfully retrieved.
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Sets the non-blocking mode of the socket.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }

    /// Marks the socket as a listening socket.
    ///
    /// This is a wrapper around `listen(2)`.
    pub fn listen(&self) -> io::Result<()> {
        self.0.listen(128)
    }
}

/// Listener functions
impl Socket {
    /// Binds the socket to the given address.
    pub fn bind(&self, addr: &SocketAddr) -> io::Result<()> {
        self.0.bind(&addr.0)
    }

    /// Accepts a new incoming connection from this listener.
    ///
    /// This function will block the calling thread until a new connection is established.
    /// When established, the corresponding `Socket` and the remote peer's address will be returned.
    pub fn accept(&self) -> io::Result<(Self, SocketAddr)> {
        self.0.accept()
            .map(|(sock, addr)| (Self(sock), unsafe { SocketAddr::from_raw_unchecked(addr) }))
    }
}

/// Stream functions
impl Socket {
    /// Establishes a connection to the given socket address.
    pub fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        self.0.connect(&addr.0)
    }

    /// Returns the socket address of the remote peer.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr().map(|addr| unsafe { SocketAddr::from_raw_unchecked(addr) })
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified portions to return
    /// immediately with an appropriate value (see the documentation of `Shutdown`).
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.0.shutdown(how)
    }

    /// Sets the read timeout for the socket.
    ///
    /// If the provided value is `None`, then `read` calls will block indefinitely.
    /// An `Err` is returned if the zero `Duration` is passed to this method.
    pub fn set_read_timeout(&self, duration: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(duration)
    }

    /// Sets the write timeout for the socket.
    ///
    /// If the provided value is `None`, then `write` calls will block indefinitely.
    /// An `Err` is returned if the zero `Duration` is passed to this method.
    pub fn set_write_timeout(&self, duration: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(duration)
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }
}

impl<'a> Read for &'a Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.0).read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (&self.0).read_vectored(bufs)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<'a> Write for &'a Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.0).write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        (&self.0).write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.0).flush()
    }
}
