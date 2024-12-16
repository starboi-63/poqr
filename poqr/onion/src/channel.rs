use ntru::ntru_key::NtruPublicKey;
use std::net::TcpStream;

/// A channel between two nodes in the network through which messages can be sent.
pub struct Channel {
    /// A TCP connection to the remote node.
    pub connection: TcpStream,
    /// The public key of the remote node.
    pub encryption_key: NtruPublicKey,
}
