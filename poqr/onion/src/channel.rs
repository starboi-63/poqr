use crate::{RelayPayload, Message, RelayId};
use rsa_ext::{RsaPublicKey, PaddingScheme, PublicKey};
use std::io::{Read, Write};
use std::net::TcpStream;
use rand::Rng;

/// A channel between two nodes in the network through which messages can be sent.
pub struct Channel {
    /// The ID of the relay node that the channel is connected to.
    pub relay_id: RelayId,
    /// The public keys of the remote nodes used to encrypt messages sent through the connection.
    pub rsa_public_keys: Vec<RsaPublicKey>,
    /// A TCP connection to the remote node.
    pub connection: TcpStream,
    /// The directory of the onion-routing network.
    directory: Arc<Mutex<Directory>>,
}

impl Channel {
    pub fn onion_skin(&mut self, bytes: &Vec<u8>) -> Vec<u8> {
        let ret: Vec<u8> = Vec::new();
        for key in &self.rsa_public_keys {
            let padding = PaddingScheme::new_pkcs1v15_encrypt();
            ret = key.encrypt(&mut rng, padding, bytes).expect("failed to encrypt");
        }
        ret
    }
    
    pub fn send(&mut self, msg: Message) {
        let mut rng = rand::thread_rng();

        // Apply onion skins on top of messages
        let onion = match msg {
            Message::Relay(payload) => match payload {
                RelayPayload::Extend(extend_payload) => self.onion_skin(&extend_payload.to_be_bytes()),
                RelayPayload::Extended(extended_payload) => self.onion_skin(&extended_payload.to_be_bytes()),
            },
            _ => msg
        };

        // Apply NTRU encryption on top of onion-skinned RSA encrypted message
        let dir = self.directory.read().unwrap();
        let ntru_key: NtruPublicKey = dir.get_relay_info(self.relay_id).unwrap().id_key_pub;
        let enc_msg_bytes = ntru_key.encrypt_bytes(onion);
        
        todo!()
    }

    pub fn recv(&mut self) -> Message {
        let mut buf: Vec<u8> = Vec::new();
        match self.connection.read(&mut buf) {
            Ok(_) => Message::from_be_bytes(buf),
            Err(_) => panic!("Failed to read from connection"),
        }
    }
}

}