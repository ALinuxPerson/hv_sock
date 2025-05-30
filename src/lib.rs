//! # hv_sock
//!
//! This crate provides a cross-platform API for Hyper-V sockets.
//!
//! Hyper-V sockets allow communication between a host machine and its virtual machines,
//! or between virtual machines, using a socket-based interface.
//!
//! ## Features
//! - `host-registry`: Enables functionality for interacting with the Hyper-V host registry (Windows-only).

mod sys;
mod socket;
mod listener;
mod stream;
mod addr;

#[cfg(all(feature = "host-registry", windows))]
pub mod host_registry;

pub use socket::{Socket, TYPE, DOMAIN, PROTOCOL};
pub use listener::{Listener, Incoming};
pub use stream::Stream;
pub use addr::SocketAddr;

#[cfg(all(feature = "host-registry", windows))]
pub use host_registry::{HostRegistry, Service, ServiceData, ServiceUuid};

pub type HyperVSocket = Socket;
pub type HyperVSocketListener = Listener;
pub type HyperVSocketStream = Stream;
pub type HyperVSocketAddr = SocketAddr;

#[cfg(not(any(target_os = "linux", windows)))]
compile_error!("hyper v sockets are only supported in linux and windows systems");

#[cfg(all(not(windows), feature = "host-registry"))]
compile_error!("the `host-registry` feature is only available on windows systems");
