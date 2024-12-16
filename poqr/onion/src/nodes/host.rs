use crate::{Channel, ChannelTable, Circuit, CircuitTable, Directory};
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
    identity_key: NtruKeyPair,
    pub directory: Arc<RwLock<Directory>>,
}

impl Host {
    pub fn new(port: u16, directory: Arc<RwLock<Directory>>) -> Host {
        Host {
            port,
            channels: ChannelTable::new(),
            circuit_table: CircuitTable::new(),
            identity_key: NtruKeyPair::new(),
            directory,
        }
    }

    pub fn generate_onion_keys(&self, bits: usize) -> (Vec<RsaPublicKey>, Vec<RsaPrivateKey>) {
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
            id_key: id_key.clone(),
            forward_onion_keys: Vec::new(),
            backward_onion_keys: onion_keys,
            connection,
        };
        self.channels.insert(circuit_id, channel);
    }

    pub fn create_circuit(&mut self) -> Circuit {
        // Generate ephemeral key pairs for backwards communication from each relay
        let (public_keys, private_keys) = self.generate_onion_keys(1024);

        // Find a random relay to start the circuit
        let exclude_list: HashSet<u32> = HashSet::new();
        let relay = {
            let dir = self.directory.read().unwrap();
            dir.get_random_relay(exclude_list).unwrap().clone()
        };
        let circuit_id = {
            let mut circuit_id = rand::random::<u32>();
            while self.circuit_table.get(circuit_id).is_some() {
                circuit_id = rand::random::<u32>();
            }
            circuit_id
        };

        // Establish connection with first relay and send create message
        self.create_channel(circuit_id, relay.port, relay.id_key_pub, private_keys);
        let channel = self.channels.get(circuit_id).unwrap();

        channel.send();

        // Wait for the CREATED message

        for _ in 0..CIRCUIT_LENGTH {
            // Send an EXTEND message to the first relay

            // Wait for an EXTENDED message
        }

        todo!()
    }
}
