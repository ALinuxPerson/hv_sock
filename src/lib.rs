mod sys {
    #[cfg(target_os = "linux")]
    mod linux {
        use std::mem;
        use socket2::{Domain, Protocol};

        pub const DOMAIN: Domain = unsafe { mem::transmute(libc::AF_VSOCK) };
        pub const PROTOCOL: Protocol = unsafe { mem::transmute(0) };
    }

    #[cfg(target_os = "linux")]
    pub use linux::{DOMAIN, PROTOCOL};

    #[cfg(windows)]
    mod windows {
        use std::ffi::c_int;
        use std::mem;
        use socket2::{Domain, Protocol};
        use windows::Win32::Networking::WinSock::AF_HYPERV;
        use windows::Win32::System::Hypervisor::HV_PROTOCOL_RAW;

        pub const DOMAIN: Domain = unsafe { mem::transmute(AF_HYPERV as c_int) };
        pub const PROTOCOL: Protocol = unsafe { mem::transmute(HV_PROTOCOL_RAW) };
    }

    #[cfg(windows)]
    pub use windows::{DOMAIN, PROTOCOL};
}
pub mod socket {
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
}
mod listener {
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
}
pub mod stream {
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
}
pub mod addr {
    mod sys {
        #[cfg(target_os = "linux")]
        mod linux {
            use std::ptr;
            use std::mem::MaybeUninit;
            use libc::{AF_VSOCK, sockaddr_vm, VMADDR_CID_HOST};
            use socket2::SockAddr;
            use crate::SocketAddr;

            pub(crate) type BackingType = sockaddr_vm;
            
            impl SocketAddr {
                pub fn new(port: u32) -> Self {
                    let sockaddr_vm = sockaddr_vm {
                        svm_family: AF_VSOCK as _,
                        svm_reserved1: 0,
                        svm_port: port,
                        svm_cid: VMADDR_CID_HOST,
                        svm_zero: [0; 4],
                    };
                    let len = size_of::<sockaddr_vm>();
                    let mut storage = MaybeUninit::uninit();
                    unsafe { ptr::copy_nonoverlapping(&sockaddr_vm, storage.as_mut_ptr() as *mut _, len) };
                
                    unsafe { Self::from_raw_unchecked(SockAddr::new(storage.assume_init(), len as _)) }
                }
            }
        }

        #[cfg(target_os = "linux")]
        pub(super) use linux::BackingType;

        #[cfg(windows)]
        mod windows {
            use std::ptr;
            use std::mem::MaybeUninit;
            use socket2::SockAddr;
            use uuid::Uuid;
            use windows::Win32::System::Hypervisor::SOCKADDR_HV;
            use windows::core::GUID;
            use windows::Win32::Networking::WinSock::{ADDRESS_FAMILY, AF_HYPERV};
            use crate::addr::SocketAddr;

            pub(crate) type BackingType = SOCKADDR_HV;

            fn uuid_to_guid(uuid: Uuid) -> GUID {
                let (data1, data2, data3, data4) = uuid.as_fields();
                GUID { data1, data2, data3, data4: *data4 }
            }

            impl SocketAddr {
                pub fn new(vm_id: Uuid, service_id: Uuid) -> Self {
                    let sockaddr_hv = SOCKADDR_HV {
                        Family: ADDRESS_FAMILY(AF_HYPERV),
                        Reserved: 0,
                        VmId: uuid_to_guid(vm_id),
                        ServiceId: uuid_to_guid(service_id),
                    };
                    let len = size_of::<SOCKADDR_HV>();
                    let mut storage = MaybeUninit::uninit();
                    unsafe { ptr::copy_nonoverlapping(&sockaddr_hv, storage.as_mut_ptr() as *mut _, len) }

                    unsafe { Self::from_raw_unchecked(SockAddr::new(storage.assume_init(), len as _)) }
                }
            }
        }

        #[cfg(windows)]
        pub(super) use windows::BackingType;
    }

    use sys::BackingType;
    use socket2::SockAddr;

    #[derive(Debug)]
    pub struct SocketAddr(pub(crate) SockAddr);

    impl SocketAddr {
        pub(crate) const unsafe fn from_raw_unchecked(value: SockAddr) -> Self {
            // we don't use `debug_assert_eq` here because we're in a `const fn`
            debug_assert!(value.len() as usize == size_of::<BackingType>());
            Self(value)
        }
    }
}

pub use socket::Socket;
pub use listener::{Listener, Incoming};
pub use stream::Stream;
pub use addr::SocketAddr;

pub type HyperVSocket = Socket;
pub type HyperVSocketListener = Listener;
pub type HyperVSocketStream = Stream;
pub type HyperVSocketAddr = SocketAddr;

#[cfg(not(any(target_os = "linux", windows)))]
compile_error!("hyper v sockets are only supported in linux and windows systems");
