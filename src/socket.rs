use std::io;
use std::io::{IoSlice, IoSliceMut, Read, Write};
use std::net::Shutdown;
use std::time::Duration;
use socket2::Type;
use crate::SocketAddr;
pub use crate::sys::{DOMAIN, PROTOCOL};

pub const TYPE: Type = Type::STREAM;

#[derive(Debug)]
pub struct Socket(socket2::Socket);

/// Common functions
impl Socket {
    pub fn new() -> io::Result<Self> {
        Ok(Self(socket2::Socket::new(DOMAIN, TYPE, Some(PROTOCOL))?))
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr().map(|addr| unsafe { SocketAddr::from_raw_unchecked(addr) })
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

    pub fn listen(&self) -> io::Result<()> {
        self.0.listen(128)
    }
}

/// Listener functions
impl Socket {
    pub fn bind(&self, addr: &SocketAddr) -> io::Result<()> {
        self.0.bind(&addr.0)
    }

    pub fn accept(&self) -> io::Result<(Self, SocketAddr)> {
        self.0.accept()
            .map(|(sock, addr)| (Self(sock), unsafe { SocketAddr::from_raw_unchecked(addr) }))
    }
}

/// Stream functions
impl Socket {
    pub fn connect(&self, addr: &SocketAddr) -> io::Result<()> {
        self.0.connect(&addr.0)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr().map(|addr| unsafe { SocketAddr::from_raw_unchecked(addr) })
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.0.shutdown(how)
    }

    pub fn set_read_timeout(&self, duration: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(duration)
    }

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
