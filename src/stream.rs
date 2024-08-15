use std::io;
use std::io::{IoSlice, IoSliceMut, Read, Write};
use std::net::Shutdown;
use std::time::Duration;
use crate::{Socket, SocketAddr};

#[derive(Debug)]
pub struct Stream(Socket);

impl Stream {
    pub(crate) const unsafe fn from_socket_unchecked(socket: Socket) -> Self {
        Self(socket)
    }
}

impl Stream {
    pub fn connect(addr: &SocketAddr) -> io::Result<Self> {
        let socket = Socket::new()?;
        socket.connect(addr)?;
        Ok(Self(socket))
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
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

impl Stream {
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
