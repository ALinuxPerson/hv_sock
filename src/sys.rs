#[cfg(target_os = "linux")]
mod linux {
    use std::mem;
    use socket2::{Domain, Protocol};

    /// The socket domain for VSOCK sockets on Linux.
    pub const DOMAIN: Domain = unsafe { mem::transmute(libc::AF_VSOCK) };
    /// The socket protocol for VSOCK sockets on Linux.
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

    /// The socket domain for AF_HYPERV sockets on Windows.
    pub const DOMAIN: Domain = unsafe { mem::transmute(AF_HYPERV as c_int) };
    /// The socket protocol for AF_HYPERV sockets on Windows.
    pub const PROTOCOL: Protocol = unsafe { mem::transmute(HV_PROTOCOL_RAW) };
}

#[cfg(windows)]
pub use windows::{DOMAIN, PROTOCOL};
