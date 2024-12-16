use crate::Message;
use ntru::ntru_key::NtruPublicKey;
use std::io::{Read, Write};
use std::net::TcpStream;

/// A channel between two nodes in the network through which messages can be sent.
pub struct Channel {
    /// The public keys of the remote nodes used to encrypt messages sent through the connection.
    pub public_keys: Vec<NtruPublicKey>,
    /// A TCP connection to the remote node.
    pub connection: TcpStream,
}

impl Channel {
    pub fn send(&mut self, msg: Message) {
        let enc_msg: Vec<u8> = {
            if !self.public_keys.is_empty() {
                let mut poly = self.public_keys[0].encrypt_bytes(msg.to_be_bytes());
                for key in &self.public_keys[1..] {
                    poly = key.encrypt_poly(poly);
                }
                poly.to_be_bytes()
            } else {
                msg.to_be_bytes()
            }
        };

        self.connection.write_all(&enc_msg).unwrap();
    }

    pub fn recv(&mut self) -> Message {
        let mut buf: Vec<u8> = Vec::new();
        match self.connection.read(&mut buf) {
            Ok(_) => Message::from_be_bytes(buf),
            Err(_) => panic!("Failed to read from connection"),
        }
    }
}
