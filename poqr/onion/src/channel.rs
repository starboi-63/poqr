use std::net::TcpStream;

use ntru::ntru_key::NtruPublicKey;

/// A channel between two nodes in the network through which messages can be sent.
pub struct Channel {
    /// The public key of the remote node used to encrypt messages sent through the connection.
    pub public_key: NtruPublicKey,
    /// A TCP connection to the remote node.
    pub connection: TcpStream,
}
