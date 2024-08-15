mod sys;
mod socket;
mod listener;
mod stream;
mod addr;

pub use socket::{Socket, TYPE, DOMAIN, PROTOCOL};
pub use listener::{Listener, Incoming};
pub use stream::Stream;
pub use addr::SocketAddr;

pub type HyperVSocket = Socket;
pub type HyperVSocketListener = Listener;
pub type HyperVSocketStream = Stream;
pub type HyperVSocketAddr = SocketAddr;

#[cfg(not(any(target_os = "linux", windows)))]
compile_error!("hyper v sockets are only supported in linux and windows systems");
