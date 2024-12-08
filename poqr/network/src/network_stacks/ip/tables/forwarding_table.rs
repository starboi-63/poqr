use super::super::{Interface, IpNetwork, IpTable};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
/// A forwarding table that maps IP subnets to interfaces
pub struct ForwardingTable {
    pub interfaces: HashMap<IpNetwork, Arc<Mutex<Interface>>>,
}

// Implement the Table trait for ForwardingTable
impl IpTable<Arc<Mutex<Interface>>> for ForwardingTable {
    /// Get the interface with the longest prefix match for the given destination IP
    fn longest_prefix_match(&self, destination: Ipv4Addr) -> (Option<Arc<Mutex<Interface>>>, u8) {
        let mut best_interface: Option<Arc<Mutex<Interface>>> = None;
        let mut longest_prefix_length: u8 = 0;

        for (network_ip, interface) in self.interfaces.iter() {
            if self.matches_prefix(destination, network_ip.clone()) {
                if network_ip.mask_length >= longest_prefix_length {
                    longest_prefix_length = network_ip.mask_length;
                    best_interface = Some(interface.clone());
                }
            }
        }

        (best_interface, longest_prefix_length)
    }

    /// Add a new entry to the forwarding table
    fn insert(&mut self, subnet: IpNetwork, interface: Arc<Mutex<Interface>>) {
        self.interfaces.insert(subnet, interface);
    }

    /// Remove an entry from the forwarding table
    fn remove(&mut self, subnet: IpNetwork) {
        self.interfaces.remove(&subnet);
    }

    /// Search for an entry in the forwarding table
    fn get(&self, subnet: IpNetwork) -> Option<Arc<Mutex<Interface>>> {
        self.interfaces.get(&subnet).cloned()
    }
}

// Implement methods specific to ForwardingTable
impl ForwardingTable {
    /// Create a new forwarding table
    pub fn new() -> Self {
        Self {
            interfaces: HashMap::new(),
        }
    }
}
