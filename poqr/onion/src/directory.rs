use crate::nodes::Relay;
use ntru::ntru_key::NtruPublicKey;
use std::collections::{HashMap, HashSet};
use std::net::UdpSocket;

struct RelayInfo {
    port: u16,
    public_key: NtruPublicKey,
}

pub struct Directory {
    /// Map from relay ID to public relay info
    relays: HashMap<u32, RelayInfo>,
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
    pub fn generate_relay(&mut self) -> u32 {
        // Find an unused port and relay ID
        let (mut port, id) = (Self::random_high_port(), self.next_relay_id);
        while self.used_ports.contains(&port) {
            port = Self::random_high_port();
        }
        self.used_ports.insert(port);

        // Construct a new relay and add it to the directory
        let relay = Relay::new(id, port);
        let relay_info = RelayInfo {
            port,
            public_key: relay.onion_key.public,
        };
        self.relays.insert(id, relay_info);

        // Increment the next relay ID
        self.next_relay_id += 1;

        id
    }
}
