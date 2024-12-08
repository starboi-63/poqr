use super::{
    ForwardingTable, Interface, IpHandler, IpNetwork, IpPacket, IpTable, Route, RoutingTable,
};
use crate::network_nodes::{Host, Router};
use std::any::Any;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// Shared internet protocol behavior for different networking nodes (hosts and routers).
pub trait IpStack: Send + Sync + Any {
    /// Wrap `self` with an `Arc` for use in threads
    fn to_arc(&self) -> Arc<dyn IpStack>;

    /// Cast the node to an `Any` type for future downcasting
    fn as_any(&self) -> &dyn Any;

    /// Downcast the IP Stack to a router object
    fn as_router(&self) -> Option<Router> {
        self.as_any().downcast_ref::<Router>().cloned()
    }

    /// Downcast the IP Stack to a host object
    fn as_host(&self) -> Option<Host> {
        self.as_any().downcast_ref::<Host>().cloned()
    }

    /// Get the node's forwarding table
    fn forwarding_table(&self) -> Arc<Mutex<ForwardingTable>>;

    /// Get the node's routing table
    fn routing_table(&self) -> Arc<Mutex<RoutingTable>>;

    /// Search for a hit in the node's forwarding table
    fn get_forwarding_table_entry(&self, subnet: IpNetwork) -> Option<Arc<Mutex<Interface>>> {
        let forwarding_table = self.forwarding_table();
        let forwarding_table = forwarding_table.lock().unwrap();
        forwarding_table.get(subnet)
    }

    /// Search for a hit in the node's routing table
    fn get_routing_table_entry(&self, subnet: IpNetwork) -> Option<Route> {
        let routing_table = self.routing_table();
        let routing_table = routing_table.lock().unwrap();
        routing_table.get(subnet)
    }

    /// Add an entry to the node's routing table
    fn insert_routing_table_entry(&self, subnet: IpNetwork, route: Route) {
        let routing_table = self.routing_table();
        let mut routing_table = routing_table.lock().unwrap();
        routing_table.insert(subnet, route);
    }

    /// Remove an entry from the node's routing table
    fn remove_routing_table_entry(&self, subnet: IpNetwork) {
        let routing_table = self.routing_table();
        let mut routing_table = routing_table.lock().unwrap();
        routing_table.remove(subnet);
    }

    /// Get the node's interfaces
    fn interfaces(&self) -> Arc<Mutex<HashMap<String, Arc<Mutex<Interface>>>>>;

    /// Given a name, get a particular interface from the node
    fn get_interface(&self, interface_name: &str) -> Option<Arc<Mutex<Interface>>> {
        let interfaces = self.interfaces();
        let interfaces = interfaces.lock().unwrap();
        interfaces.get(interface_name).cloned()
    }

    /// Get the node's packet receiver
    fn packet_receiver(&self) -> Arc<Mutex<mpsc::Receiver<IpPacket>>>;

    /// Get the node's protocol handlers
    fn protocol_handlers(&self) -> Arc<Mutex<HashMap<u8, IpHandler>>>;

    /// Register a new handler function for a specific type of packet
    fn register_protocol_handler(&self, protocol: u8, handler: IpHandler) {
        let protocol_handlers = self.protocol_handlers();
        let mut protocol_handlers = protocol_handlers.lock().unwrap();
        protocol_handlers.insert(protocol, handler);
    }

    /// Look up the inputted destination IP in the node's tables and return an interface and next hop address
    ///
    /// Returns:
    /// - The interface to process the packet (None if the packet should be dropped)
    /// - The next hop address (None if the packet has reached its local destination)
    fn search_tables(
        &self,
        destination: Ipv4Addr,
    ) -> (Option<Arc<Mutex<Interface>>>, Option<Ipv4Addr>) {
        // Lock the forwarding table before the routing table to avoid deadlock
        let forwarding_table = self.forwarding_table();
        let forwarding_table = forwarding_table.lock().unwrap();
        let routing_table = self.routing_table();
        let routing_table = routing_table.lock().unwrap();

        // Find the longest prefix match in each table
        let (mut interface, forwarding_match_length) =
            forwarding_table.longest_prefix_match(destination);
        let (route, routing_match_length) = routing_table.longest_prefix_match(destination);

        if forwarding_match_length > routing_match_length {
            // If the forwarding table has a longer prefix match, the packet has reached its local destination
            (interface, None)
        } else {
            // If the routing table has a longer prefix match, the packet needs to be forwarded to the next hop
            match route {
                Some(_) => {
                    let next_hop = route.unwrap().next_hop;
                    (interface, _) = forwarding_table.longest_prefix_match(next_hop);
                    (interface, Some(next_hop))
                }
                None => (None, None),
            }
        }
    }

    /// Send an IP packet out of the node
    fn send_packet(&self, mut packet: IpPacket) {
        // Check if the destination is an immediate neighbor
        let mut interface_to_neighbor = None;
        let destination = packet.destination();

        {
            let interfaces = self.interfaces();
            let interfaces = interfaces.lock().unwrap();

            for (_, interface) in interfaces.iter() {
                let interface = interface.lock().unwrap();

                if let Some(_) = interface.neighbors.get(&destination) {
                    interface_to_neighbor = Some(interface.name.clone());
                }
            }
        }

        if let Some(interface_name) = interface_to_neighbor {
            // The destination is an immediate neighbor, so send the packet directly
            let interface = self.get_interface(&interface_name).unwrap();
            let interface = interface.lock().unwrap();
            // Decrement the TTL, then send the packet
            packet.decrement_ttl();
            interface.send_packet(packet, Some(destination));
            return;
        }

        // Packet is not going to a neighbor; find the interface to send the packet from
        let (interface, next_hop) = self.search_tables(destination);

        match interface {
            Some(interface) => {
                // Decrement the TTL, then send the packet
                let interface = interface.lock().unwrap();
                packet.decrement_ttl();
                interface.send_packet(packet, next_hop);
            }
            None => {} // If the destination is not in the routing table, drop the packet
        }
    }

    /// Receive and handle packets from the node's interfaces
    fn receive_packets(&self) {
        let node = self.to_arc();

        thread::spawn(move || {
            loop {
                let receiver = node.packet_receiver();
                let receiver = receiver.lock().unwrap();

                match receiver.recv() {
                    Ok(packet) => {
                        // Check whether the packet has expired
                        if packet.time_to_live() > 0 {
                            // Handle the packet based on its protocol
                            let protocol_handlers = node.protocol_handlers();
                            let protocol_handlers = protocol_handlers.lock().unwrap();
                            let handler = protocol_handlers.get(&packet.protocol());

                            if let Some(handler) = handler {
                                handler(&node, packet);
                            }
                        } else {
                            eprintln!("router: Packet dropped due to TTL expiration.");
                        }
                    }
                    Err(e) => eprintln!("Error receiving packet: {}", e),
                }
            }
        });
    }
}

impl dyn IpStack {
    /// Handle a test message received by the node
    pub fn handle_test_packet(ip_stack: &Arc<dyn IpStack>, packet: IpPacket) {
        // Print the packet's contents
        packet.print();

        // Forward the packet only if its destination is not the node itself
        {
            for interface in ip_stack.interfaces().lock().unwrap().values() {
                let interface = interface.lock().unwrap();
                if interface.ip_address == packet.destination() {
                    return; // Send the packet to the "OS" if the destination is local
                }
            }
        }

        ip_stack.send_packet(packet);
    }
}
