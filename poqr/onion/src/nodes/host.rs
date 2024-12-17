use crate::messages::*;
use crate::{Channel, ChannelTable, CircuitId, CircuitTable, Directory};
use ntru::ntru_key::NtruPublicKey;
use ntru::NtruKeyPair;
use rsa_ext::{RsaPrivateKey, RsaPublicKey};
use std::collections::HashSet;
use std::{
    net::TcpStream,
    sync::{mpsc, Arc, Mutex, RwLock},
};

const CIRCUIT_LENGTH: usize = 3;
const LOCALHOST: &str = "127.0.0.1";

pub struct Host {
    /// The port on which the host listens for incoming connections
    pub port: u16,
    /// An mpsc channel for sending packets to the relay's main listener thread
    pub packet_sender: Arc<mpsc::Sender<OnionPacket>>,
    /// An mpsc channel for receiving packets from the relay's onion channels
    pub packet_receiver: Arc<Mutex<mpsc::Receiver<OnionPacket>>>,
    /// A table mapping circuit IDs to channels
    pub channels: Arc<Mutex<ChannelTable>>,
    /// A table mapping destination ports to circuit IDs
    pub circuit_table: Arc<Mutex<CircuitTable>>,
    /// The NTRU key pair used to verify the host's identity
    pub id_key: Arc<NtruKeyPair>,
    /// The public directory of relays
    pub directory: Arc<RwLock<Directory>>,
}

impl Host {
    pub fn new(port: u16, directory: Arc<RwLock<Directory>>) -> Host {
        let (sender, receiver) = mpsc::channel();

        Host {
            port,
            packet_sender: Arc::new(sender),
            packet_receiver: Arc::new(Mutex::new(receiver)),
            channels: Arc::new(Mutex::new(ChannelTable::new())),
            circuit_table: Arc::new(Mutex::new(CircuitTable::new())),
            id_key: Arc::new(NtruKeyPair::new()),
            directory,
        }
    }

    fn generate_onion_keys(bits: usize) -> (Vec<RsaPublicKey>, Vec<RsaPrivateKey>) {
        let mut rng = rand::thread_rng();
        let (mut public_keys, mut private_keys) = (Vec::new(), Vec::new());

        for _ in 0..CIRCUIT_LENGTH {
            let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
            let public_key = RsaPublicKey::from(&private_key);
            public_keys.push(public_key);
            private_keys.push(private_key);
        }

        (public_keys, private_keys)
    }

    fn generate_new_circuit_id(&self) -> CircuitId {
        let circuits = self.circuit_table.lock().unwrap();

        let mut circuit_id = rand::random::<u32>();
        while circuits.used_circuit_ids.contains(&circuit_id) {
            circuit_id = rand::random::<u32>();
        }

        circuit_id
    }

    pub fn create_channel(
        &self,
        circuit_id: u32,
        port: u16,
        id_key: NtruPublicKey,
        onion_keys: Vec<RsaPrivateKey>,
    ) {
        let mut channels = self.channels.lock().unwrap();
        let connection = TcpStream::connect(format!("{LOCALHOST}:{port}")).unwrap();
        // Instantiate channel
        let channel = Channel {
            forward_id_key: Arc::new(id_key),
            backward_id_key: Arc::new(self.id_key.private.clone()),
            forward_onion_keys: Arc::new(Mutex::new(Vec::new())),
            backward_onion_keys: Arc::new(onion_keys),
            connection: Arc::new(Mutex::new(connection)),
            packet_sender: (*self.packet_sender).clone(),
        };
        channels.insert(circuit_id, channel);
    }

    pub fn create_circuit(&mut self, destination: u16) -> CircuitId {
        // Lock the tables
        let mut circuits = self.circuit_table.lock().unwrap();
        let mut channels = self.channels.lock().unwrap();
        // Generate
        // Generate ephemeral key pairs for backward communication from each relay
        let (public_keys, private_keys) = Host::generate_onion_keys(1024);
        // Exclude list to avoid using the same relay twice
        let mut exclude_list: HashSet<u32> = HashSet::new();

        // Initialize a new circuit id and choose the first relay
        let circuit_id = self.generate_new_circuit_id();
        let relay = {
            let dir = self.directory.read().unwrap();
            dir.get_random_relay(exclude_list.clone()).unwrap().clone()
        };
        exclude_list.insert(relay.id);

        // Establish connection with the first relay
        self.create_channel(
            circuit_id,
            relay.port,
            relay.id_key_pub,
            private_keys.clone(),
        );
        let channel = channels.get_mut(circuit_id).unwrap();

        // Send the CREATE message to the first relay
        let create_payload = CreatePayload {
            public_key: public_keys[0].clone(), // The public onion key for this relay to encrypt backward messages
        };
        let create_message = Message::Create(create_payload);
        channel.send(circuit_id, create_message);

        // Wait for the CREATED message
        let response = channel.recv();
        match response.msg {
            Message::Created(payload) => {
                let mut forward_onion_keys = channel.forward_onion_keys.lock().unwrap();
                forward_onion_keys.push(payload.public_key);
            }
            _ => panic!("Unexpected message while creating circuit"),
        }

        // Extend the circuit to additional relays
        for i in 1..CIRCUIT_LENGTH {
            // Select the next relay, avoiding duplicates
            let relay = {
                let dir = self.directory.read().unwrap();
                dir.get_random_relay(exclude_list.clone()).unwrap().clone()
            };
            exclude_list.insert(relay.id);

            // Send EXTEND message
            let extend_payload = ExtendPayload {
                public_key: public_keys[i].clone(),
            };
            let extend_message = Message::Relay(RelayPayload::Extend(extend_payload));
            channel.send(circuit_id, extend_message);

            // Wait for EXTENDED message
            let response = channel.recv();
            match response.msg {
                Message::Relay(RelayPayload::Extended(payload)) => {
                    // Successfully extended to the next relay
                    let mut forward_onion_keys = channel.forward_onion_keys.lock().unwrap();
                    forward_onion_keys.push(payload.public_key);
                }
                _ => panic!("Unexpected message while extending circuit"),
            }
        }

        // At this point, the circuit is fully established
        circuits.insert(destination, circuit_id);
        circuit_id
    }
}
