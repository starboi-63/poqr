// Module: sockets
mod listener_socket;
mod normal_socket;
// Exported from sockets module
pub use listener_socket::TcpListenerSocket;
pub use normal_socket::TcpNormalSocket;
