use crate::messages::*;
use crate::{Channel, ChannelTable, CircuitId, CircuitTable, Directory, RelayId, RelayInfo};
use ntru::ntru_key::NtruPublicKey;
use ntru::NtruKeyPair;
use rsa_ext::{RsaPrivateKey, RsaPublicKey};
use std::collections::HashSet;
use std::{
    net::TcpStream,
    sync::{Arc, RwLock},
};

const CIRCUIT_LENGTH: usize = 3;
const LOCALHOST: &str = "127.0.0.1";

pub struct Host {
    port: u16,
    channels: ChannelTable,
    circuit_table: CircuitTable,
    id_key: NtruKeyPair,
    pub directory: Arc<RwLock<Directory>>,
}

impl Host {
    pub fn new(port: u16, directory: Arc<RwLock<Directory>>) -> Host {
        Host {
            port,
            channels: ChannelTable::new(),
            circuit_table: CircuitTable::new(),
            id_key: NtruKeyPair::new(),
            directory,
        }
    }

    fn generate_onion_keys(&self, bits: usize) -> (Vec<RsaPublicKey>, Vec<RsaPrivateKey>) {
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
        let mut circuit_id = rand::random::<u32>();
        while self.circuit_table.used_circuit_ids.contains(&circuit_id) {
            circuit_id = rand::random::<u32>();
        }
        circuit_id
    }

    pub fn create_channel(
        &mut self,
        circuit_id: u32,
        port: u16,
        id_key: NtruPublicKey,
        onion_keys: Vec<RsaPrivateKey>,
    ) {
        let connection = TcpStream::connect(format!("{LOCALHOST}:{port}")).unwrap();
        // Instantiate channel
        let channel = Channel {
            forward_id_key: id_key,
            backward_id_key: self.id_key.private.clone(),
            forward_onion_keys: Vec::new(),
            backward_onion_keys: onion_keys,
            connection,
        };
        self.channels.insert(circuit_id, channel);
    }

    pub fn create_circuit(&mut self) -> CircuitId {
        // Generate ephemeral key pairs for backward communication from each relay
        let (public_keys, private_keys) = self.generate_onion_keys(1024);
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
        let channel = self.channels.get_mut(circuit_id).unwrap();

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
                channel.forward_onion_keys.push(payload.public_key);
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
                    channel.forward_onion_keys.push(payload.public_key);
                }
                _ => panic!("Unexpected message while extending circuit"),
            }
        }

        // At this point, the circuit is fully established
        circuit_id
    }
}
