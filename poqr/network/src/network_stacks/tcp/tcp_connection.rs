use std::net::SocketAddrV4;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
/// Represents a TCP connection between two sockets on different hosts.
pub struct TcpConnection {
    pub local_address: SocketAddrV4,
    pub remote_address: SocketAddrV4,
}

#[derive(Debug, Clone, PartialEq)]
/// The state of a TCP connection.
pub enum TcpConnectionState {
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
    Closed,
}
