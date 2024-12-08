use super::super::{IpNetwork, IpTable};
use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Clone, Copy, Debug)]
/// Represents a route in the routing table
pub struct Route {
    /// The next hop address for the route
    pub next_hop: Ipv4Addr,
    /// The cost to reach the destination
    pub cost: u32,
    /// The time when the route was created (in seconds since the Unix epoch)
    pub creation_time: u64,
    /// Whether the route is static (manually configured) or dynamic (learned from RIP)
    pub is_static: bool,
}

#[derive(Debug)]
/// A routing table that maps IP subnets to routes
pub struct RoutingTable {
    pub routes: HashMap<IpNetwork, Route>,
}

// Implement the Table trait for RoutingTable
impl IpTable<Route> for RoutingTable {
    /// Get the next hop address with the longest prefix match for the given destination IP
    fn longest_prefix_match(&self, destination: Ipv4Addr) -> (Option<Route>, u8) {
        let mut best_route = None;
        let mut longest_prefix_length: u8 = 0;

        for (network_ip, route) in self.routes.iter() {
            if self.matches_prefix(destination, network_ip.clone()) {
                if network_ip.mask_length >= longest_prefix_length {
                    longest_prefix_length = network_ip.mask_length;

                    if route.cost < 16 {
                        best_route = Some(route);
                    }
                }
            }
        }

        (best_route.copied(), longest_prefix_length)
    }

    /// Add a new entry to the routing table
    fn insert(&mut self, subnet: IpNetwork, route: Route) {
        self.routes.insert(subnet, route);
    }

    /// Remove an entry from the routing table
    fn remove(&mut self, subnet: IpNetwork) {
        self.routes.remove(&subnet);
    }

    /// Search for an entry in the routing table
    fn get(&self, subnet: IpNetwork) -> Option<Route> {
        self.routes.get(&subnet).cloned()
    }
}

// Implement methods specific to RoutingTable
impl RoutingTable {
    /// Create a new routing table
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }
}
