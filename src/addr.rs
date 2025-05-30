/// Platform-specific implementations for socket addresses.
mod sys {
    #[cfg(target_os = "linux")]
    /// Linux-specific implementation for VSOCK addresses.
    mod linux {
        use std::ptr;
        use std::mem::MaybeUninit;
        use libc::{AF_VSOCK, sockaddr_vm, VMADDR_CID_HOST};
        use socket2::SockAddr;
        use crate::SocketAddr;

        pub(crate) type BackingType = sockaddr_vm;

        impl SocketAddr {
            /// Creates a new `SocketAddr` for Linux with the given port.
            ///
            /// The CID is set to `VMADDR_CID_HOST`.
            pub fn new(port: u32) -> Self {
                let mut storage = MaybeUninit::zeroed();

                unsafe {
                    let storage = storage.as_mut_ptr() as *mut sockaddr_vm;
                    ptr::addr_of_mut!((*storage).svm_family).write(AF_VSOCK as _);
                    ptr::addr_of_mut!((*storage).svm_port).write(port);
                    ptr::addr_of_mut!((*storage).svm_cid).write(VMADDR_CID_HOST);
                }

                unsafe { Self::from_raw_unchecked(SockAddr::new(storage.assume_init(), size_of::<sockaddr_vm>() as _)) }
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub(super) use linux::BackingType;

    #[cfg(windows)]
    /// Windows-specific implementation for Hyper-V socket addresses.
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

        /// Converts a `Uuid` to a `GUID`.
        /// This is necessary for interacting with Windows APIs that expect `GUID`.
        fn uuid_to_guid(uuid: Uuid) -> GUID {
            let (data1, data2, data3, data4) = uuid.as_fields();
            GUID { data1, data2, data3, data4: *data4 }
        }

        impl SocketAddr {
            /// Creates a new `SocketAddr` for Windows with the given VM ID and service ID.
            pub fn new(vm_id: Uuid, service_id: Uuid) -> Self {
                let mut storage = MaybeUninit::zeroed();

                unsafe {
                    let storage = storage.as_mut_ptr() as *mut SOCKADDR_HV;
                    ptr::addr_of_mut!((*storage).Family).write(ADDRESS_FAMILY(AF_HYPERV));
                    ptr::addr_of_mut!((*storage).VmId).write(uuid_to_guid(vm_id));
                    ptr::addr_of_mut!((*storage).ServiceId).write(uuid_to_guid(service_id));
                }

                unsafe { Self::from_raw_unchecked(SockAddr::new(storage.assume_init(), size_of::<SOCKADDR_HV>() as _)) }
            }
        }
    }

    #[cfg(windows)]
    pub(super) use windows::BackingType;
}

use sys::BackingType;
use socket2::SockAddr;

/// Represents a socket address for Hyper-V sockets (hv_sock).
///
/// This struct is a wrapper around the platform-specific socket address type.
/// On Linux, it wraps `sockaddr_vm` for VSOCK.
/// On Windows, it wraps `SOCKADDR_HV` for Hyper-V sockets.
#[derive(Debug)]
pub struct SocketAddr(pub(crate) SockAddr);

impl SocketAddr {
    /// Creates a `SocketAddr` from a raw `socket2::SockAddr`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided `value` is a valid socket address
    /// for the current platform and that its length matches the size of the
    /// platform-specific backing type (`sockaddr_vm` on Linux, `SOCKADDR_HV` on Windows).
    /// Failure to meet these conditions can lead to undefined behavior.
    pub(crate) const unsafe fn from_raw_unchecked(value: SockAddr) -> Self {
        // we don't use `debug_assert_eq` here because we're in a `const fn`
        // The assertion checks that the length of the provided SockAddr matches
        // the expected size of the platform-specific backing type.
        debug_assert!(value.len() as usize == size_of::<BackingType>());
        Self(value)
    }
}
