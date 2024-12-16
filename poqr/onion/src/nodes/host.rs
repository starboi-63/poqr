use crate::{Channel, ChannelTable, Circuit, CircuitTable, Directory, RelayInfo};
use ntru::{ntru_key::NtruPublicKey, NtruKeyPair};
use std::{
    net::TcpStream,
    sync::{Arc, RwLock},
};

const CIRCUIT_LENGTH: usize = 3;

pub struct Host {
    port: u16,
    channel_table: ChannelTable,
    circuit_table: CircuitTable,
    onion_key: NtruKeyPair,
    directory: Arc<RwLock<Directory>>,
}

impl Host {
    pub fn new(port: u16, directory: Arc<RwLock<Directory>>) -> Host {
        Host {
            port,
            channel_table: ChannelTable::new(),
            circuit_table: CircuitTable::new(),
            onion_key: NtruKeyPair::new(),
            directory,
        }
    }

    pub fn create_channel(&mut self, relay_info: RelayInfo, circuit_id: u32) {
        let (encryption_key, port) = (relay_info.public_key, relay_info.port);
        let connection = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        let channel = Channel {
            connection,
            encryption_key,
        };
        self.channel_table.insert(circuit_id, channel);
    }

    pub fn create_circuit(&mut self) -> Circuit {
        // Send a CREATE message to the first relay in the circuit
        // NOTE: need to prevent the random relay from being the same as the host (our ourselves)
        let relay_info = {
            let dir = self.directory.read().unwrap();
            dir.get_random_relay().unwrap().clone()
        };
        let circuit_id = {
            let mut circuit_id = rand::random::<u32>();
            while self.circuit_table.get(circuit_id).is_some() {
                circuit_id = rand::random::<u32>();
            }
            circuit_id
        };

        // Wait for the CREATED message

        for _ in 0..CIRCUIT_LENGTH {
            // Send an EXTEND message to the first relay

            // Wait for an EXTENDED message
        }

        todo!()
    }
}
