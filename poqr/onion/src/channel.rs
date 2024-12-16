use crate::{Message, OnionPacket};
use ntru::ntru_key::NtruPublicKey;
use rsa_ext::{RsaPrivateKey, RsaPublicKey};
use std::io::{Read, Write};
use std::net::TcpStream;

/// A channel between two nodes in the network through which messages can be sent.
pub struct Channel {
    /// The ID of the relay node that the channel is connected to.
    pub id_key: NtruPublicKey,
    /// The public keys of the remote nodes used to encrypt messages sent forwards through the connection.
    pub forward_onion_keys: Vec<RsaPublicKey>,
    /// The public keys of the remote nodes used to encrypt messages sent backwards through the connection.
    pub backward_onion_keys: Vec<RsaPrivateKey>,
    /// A TCP connection to the remote node.
    pub connection: TcpStream,
}

impl Channel {
    pub fn send(&mut self, id: u32, msg: Message) {
        let packet = OnionPack
        let bytes = msg.to_be_bytes(self.id_key.clone(), self.forward_onion_keys.clone());
        self.connection.write(&bytes).unwrap();
    }

    pub fn recv(&mut self) -> Message {
        let mut buf: Vec<u8> = Vec::new();
        match self.connection.read(&mut buf) {
            Ok(_) => Message::from_be_bytes(buf),
            Err(_) => panic!("Failed to read from connection"),
        }
    }
}
