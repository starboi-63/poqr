use super::sockets::{TcpListenerSocket, TcpNormalSocket};
use super::TcpConnectionState;

#[derive(Clone)]
pub enum TcpSocket {
    Listener(TcpListenerSocket),
    Normal(TcpNormalSocket),
}

impl TcpSocket {
    /// Get the ID of the socket.
    pub fn id(&self) -> u32 {
        match self {
            TcpSocket::Listener(listener_socket) => listener_socket.id,
            TcpSocket::Normal(normal_socket) => normal_socket.id,
        }
    }

    /// Get the state of the socket.
    pub fn state(&self) -> TcpConnectionState {
        match self {
            TcpSocket::Listener(_) => TcpConnectionState::Listen,
            TcpSocket::Normal(normal_socket) => normal_socket.state(),
        }
    }

    /// Downcast the socket to a listener socket if it is one. Otherwise, return `None`.
    pub fn as_listener(&self) -> Option<TcpListenerSocket> {
        match self {
            TcpSocket::Listener(listener_socket) => Some(listener_socket.clone()),
            _ => None,
        }
    }

    /// Downcast the socket to a normal socket if it is one. Otherwise, return `None`.
    pub fn as_normal(&self) -> Option<TcpNormalSocket> {
        match self {
            TcpSocket::Normal(normal_socket) => Some(normal_socket.clone()),
            _ => None,
        }
    }
}
