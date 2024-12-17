use crate::{Message, OnionHeader, OnionPacket};
use ntru::ntru_key::{NtruPrivateKey, NtruPublicKey};
use rsa_ext::{RsaPrivateKey, RsaPublicKey};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};

#[derive(Clone)]
/// A channel between two nodes in the network through which messages can be sent.
pub struct Channel {
    /// The identity key of the remote node.
    pub forward_id_key: Arc<NtruPublicKey>,
    /// Our own NTRU identity key
    pub backward_id_key: Arc<NtruPrivateKey>,
    /// The public keys of the remote nodes used to encrypt messages sent forwards through the connection.
    pub forward_onion_keys: Arc<Mutex<Vec<RsaPublicKey>>>,
    /// The public keys of the remote nodes used to encrypt messages sent backwards through the connection.
    pub backward_onion_keys: Arc<Vec<RsaPrivateKey>>,
    /// A TCP connection to the remote node.
    pub connection: Arc<Mutex<TcpStream>>,
    /// A channel to send packets to the this node's main listener thread.
    pub packet_sender: mpsc::Sender<OnionPacket>,
}

impl Channel {
    pub fn start_listener(&self) {
        let mut channel = self.clone();

        std::thread::spawn(move || loop {
            let packet = channel.recv();
            // Send the packet to the main listener thread
            channel.packet_sender.send(packet).unwrap();
        });
    }

    pub fn send(&mut self, id: u32, msg: Message) {
        let packet = Channel::build_packet(id, msg);
        let bytes = packet.to_be_bytes(
            (*self.forward_id_key).clone(),
            (*self.forward_onion_keys.lock().unwrap()).clone(),
        );

        let mut connection = self.connection.lock().unwrap();
        connection.write(&bytes).unwrap();
    }

    pub fn recv(&mut self) -> OnionPacket {
        let mut connection = self.connection.lock().unwrap();

        // Read the circuit ID
        let mut circ_id_buf = [0u8; 4];
        connection.read_exact(&mut circ_id_buf).unwrap();
        let circ_id: u32 = u32::from_be_bytes(circ_id_buf);

        // Read the message length
        let mut msg_len_buf = [0u8; 4];
        connection.read_exact(&mut msg_len_buf).unwrap();
        let msg_len = u32::from_be_bytes(msg_len_buf) as usize;

        // Read the message
        let mut msg_buf = vec![0u8; msg_len];
        connection.read_exact(&mut msg_buf).unwrap();
        let msg: Message = Message::from_be_bytes(
            msg_buf,
            (*self.backward_id_key).clone(),
            (*self.backward_onion_keys).clone(),
        );

        Channel::build_packet(circ_id, msg)
    }

    fn build_packet(id: u32, msg: Message) -> OnionPacket {
        let header = OnionHeader { circ_id: id };
        OnionPacket { header, msg }
    }
}
