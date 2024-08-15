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
