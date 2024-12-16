use crate::nodes::Relay;
use ntru::ntru_key::NtruPublicKey;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::net::UdpSocket;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct RelayInfo {
    pub port: u16,
    pub id_key_pub: NtruPublicKey,
}

type RelayId = u32;

/// Directory of relays and their public info.
pub struct Directory {
    /// Map from relay ID to public relay info
    relays: HashMap<RelayId, RelayInfo>,
    /// Set of used ports
    used_ports: HashSet<u16>,
    /// Next relay ID to assign
    next_relay_id: u32,
}

impl Directory {
    pub fn new() -> Directory {
        Directory {
            relays: HashMap::new(),
            used_ports: HashSet::new(),
            next_relay_id: 0,
        }
    }

    /// Get a random high port number that is not currently in use.
    pub fn random_high_port() -> u16 {
        const MIN_PORT: u16 = 20000;
        const MAX_PORT: u16 = 65535;

        let range = (MAX_PORT - MIN_PORT) as u32;

        loop {
            let random_offset = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos())
                % range;

            let port = MIN_PORT + random_offset as u16;

            // Check if it is an unused port
            if let Ok(socket) = UdpSocket::bind(("0.0.0.0", port)) {
                drop(socket);
                return port;
            }
        }
    }

    /// Generate a new relay and return its ID.
    pub fn generate_relay(directory: Arc<RwLock<Directory>>) -> RelayId {
        let mut dir = directory.write().unwrap();

        // Find an unused port and relay ID
        let (mut port, id) = (Self::random_high_port(), dir.next_relay_id);
        while dir.used_ports.contains(&port) {
            port = Self::random_high_port();
        }
        dir.used_ports.insert(port);

        // Construct a new relay and add it to the directory
        let relay = Relay::new(id, port, directory.clone());
        let relay_info = RelayInfo {
            port,
            id_key_pub: relay.identity_key.public,
        };
        dir.relays.insert(id, relay_info);

        // Increment the next relay ID
        dir.next_relay_id += 1;

        id
    }

    /// Get the public info for a relay.
    pub fn get_relay_info(&self, id: RelayId) -> Option<&RelayInfo> {
        self.relays.get(&id)
    }

    /// Get a random relay from the directory.
    pub fn get_random_relay(&self, exclude_list: HashSet<RelayId>) -> Option<&RelayInfo> {
        if self.relays.is_empty() {
            return None;
        }

        let keys: Vec<&RelayId> = self.relays.keys().collect();
        let mut rng = rand::thread_rng();

        let random_key: &RelayId = {
            let mut key = keys[rng.gen_range(0..keys.len())];
            while exclude_list.contains(key) {
                key = keys[rng.gen_range(0..keys.len())];
            }
            key
        };

        self.relays.get(random_key)
    }
}
