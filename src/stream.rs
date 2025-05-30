use std::io;
use std::io::{IoSlice, IoSliceMut, Read, Write};
use std::net::Shutdown;
use std::time::Duration;
use crate::{Socket, SocketAddr};

/// A stream socket that connects to a Hyper-V socket.
#[derive(Debug)]
pub struct Stream(Socket);

impl Stream {
    pub(crate) const unsafe fn from_socket_unchecked(socket: Socket) -> Self {
        Self(socket)
    }
}

impl Stream {
    /// Establishes a connection to the specified Hyper-V socket address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address of the peer to connect to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).expect("Couldn\\'t connect to the server...");
    /// ```
    pub fn connect(addr: &SocketAddr) -> io::Result<Self> {
        let socket = Socket::new()?;
        socket.connect(addr)?;
        Ok(Self(socket))
    }

    /// Returns the socket address of the remote peer of this connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).unwrap();
    /// let peer_addr = stream.peer_addr().unwrap();
    /// ```
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O calls on the
    /// specified portions to immediately return with an appropriate error.
    ///
    /// # Arguments
    ///
    /// * `how` - The direction to shut down.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::net::Shutdown;
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).unwrap();
    /// stream.shutdown(Shutdown::Both).expect("shutdown call failed");
    /// ```
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.0.shutdown(how)
    }

    /// Sets the read timeout for the socket.
    ///
    /// If the provided value is `None`, then `read` calls will block indefinitely.
    /// An `Err` is returned if the zero `Duration` is passed to this method.
    ///
    /// # Arguments
    ///
    /// * `duration` - The read timeout.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::time::Duration;
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).unwrap();
    /// stream.set_read_timeout(Some(Duration::new(5, 0))).expect("set_read_timeout call failed");
    /// ```
    pub fn set_read_timeout(&self, duration: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(duration)
    }

    /// Sets the write timeout for the socket.
    ///
    /// If the provided value is `None`, then `write` calls will block indefinitely.
    /// An `Err` is returned if the zero `Duration` is passed to this method.
    ///
    /// # Arguments
    ///
    /// * `duration` - The write timeout.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::time::Duration;
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).unwrap();
    /// stream.set_write_timeout(Some(Duration::new(5, 0))).expect("set_write_timeout call failed");
    /// ```
    pub fn set_write_timeout(&self, duration: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(duration)
    }
}

impl Stream {
    /// Returns the local socket address of this connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).unwrap();
    /// let local_addr = stream.local_addr().unwrap();
    /// ```
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }

    /// Creates a new independently owned handle to the underlying socket.
    ///
    /// The returned `Stream` is a reference to the same stream that this
    /// object references. Both handles will read and write the same stream of
    /// data, and options set on one stream will affect the other.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).unwrap();
    /// let cloned_stream = stream.try_clone().unwrap();
    /// ```
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

    /// Gets the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, if one
    /// exists. An `Ok(None)` is returned if there is no error saved.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).unwrap();
    /// let error = stream.take_error().unwrap();
    /// // error will be None if the connection was successful.
    /// ```
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    /// Sets the non-blocking mode of this stream.
    ///
    /// If `nonblocking` is `true`, then `read` and `write` operations will
    /// return an error with kind `io::ErrorKind::WouldBlock` if the operation
    /// would block. If `nonblocking` is `false`, then `read` and `write`
    /// operations will block until completed or an error occurs.
    ///
    /// # Arguments
    ///
    /// * `nonblocking` - Whether to set the stream to non-blocking mode.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hv_sock::{SocketAddr, Stream};
    /// use std::str::FromStr;
    ///
    /// let addr = SocketAddr::from_str("00000000-0000-0000-0000-000000000000:123").unwrap();
    /// let stream = Stream::connect(&addr).unwrap();
    /// stream.set_nonblocking(true).expect("set_nonblocking call failed");
    /// ```
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }
}

impl<'a> Read for &'a Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.0).read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (&self.0).read_vectored(bufs)
    }
}

impl Write for Stream {
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

impl<'a> Write for &'a Stream {
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
