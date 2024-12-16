use std::net::TcpStream;
use ntru::ntru_key::NtruPublicKey;
use crate::Message;
use std::io::{Read, Write};

/// A channel between two nodes in the network through which messages can be sent.
pub struct Channel {
    /// The public keys of the remote nodes used to encrypt messages sent through the connection.
    pub public_keys: Vec<NtruPublicKey>,
    /// A TCP connection to the remote node.
    pub connection: TcpStream,
}

impl Channel {
    pub fn send(&mut self, msg: Message) {
        let mut enc_msg: Vec<u8> = {        
 

            if !self.public_keys.is_empty() {
                let mut poly = self.public_keys[0].encrypt_bytes(msg.serialize());
                for key in &self.public_keys[1..] {
                    poly = key.encrypt_poly(poly);
                }
            } else {
                
            }


        };

        self.connection.write_all(&);
    }

    pub fn recv(&mut self) -> Message {
        let mut buf: Vec<u8> = Vec::new();
        match self.connection.read(&mut buf) {
            Ok(_) => Message::deserialize(buf),
            Err(_) => panic!("Failed to read from connection"),
        }
    }
} 
