use crate::{Channel, ChannelTable, Circuit, CircuitTable, Directory, RelayInfo};
use ntru::NtruKeyPair;
use std::{
    net::TcpStream,
    sync::{Arc, RwLock},
};
use std::collections::HashSet;

const CIRCUIT_LENGTH: usize = 3;
const LOCALHOST: &str = "127.0.0.1";

pub struct Host {
    port: u16,
    channel_table: ChannelTable,
    circuit_table: CircuitTable,
    identity_key: NtruKeyPair,
    pub directory: Arc<RwLock<Directory>>,
}

impl Host {
    pub fn new(port: u16, directory: Arc<RwLock<Directory>>) -> Host {
        Host {
            port,
            channel_table: ChannelTable::new(),
            circuit_table: CircuitTable::new(),
            identity_key: NtruKeyPair::new(),
            directory,
        }
    }

    pub fn create_channel(&mut self, port: u16, relay_id: u32, circuit_id: u32, encryption_key: Option<NtruPublicKey>) {
        let connection = TcpStream::connect(format!("{LOCALHOST}:{port}")).unwrap();
        // If a key is given, instantiate public keys vec
        let k_pub_vec = {
            match encryption_key {
                Some(key) => vec![key],
                None => Vec::new()
            }
        };
        // Instantiate channel
        let channel = Channel {
            relay_id,
            rsa_public_keys: k_pub_vec,
            connection,
            directory: self.directory.clone(),
        };
        self.channel_table.insert(circuit_id, channel);
    }

    pub fn create_circuit(&mut self) -> Circuit {
        // Generate ephemeral key pairs for backwards communication from each relay
        let keypairs: Vec<NtruKeyPair> = (0..CIRCUIT_LENGTH).map(|_| NtruKeyPair::new()).collect();

        // Instantiate exclude list for connection
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
        self.create_channel(relay.port, relay_id, circuit_id, None); 
        let channel = self.channel_table.get(circuit_id).unwrap();
        // Wait for the CREATED message

        for _ in 0..CIRCUIT_LENGTH {
            // Send an EXTEND message to the first relay

            // Wait for an EXTENDED message
        }

        todo!()
    }
}
