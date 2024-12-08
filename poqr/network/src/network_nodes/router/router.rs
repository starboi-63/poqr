// use super::super::{ForwardingTable, Interface, IpStack, Neighbor, Route, RoutingTable, Table};
use crate::network_stacks::ip::{
    ForwardingTable, Interface, IpHandler, IpNetwork, IpPacket, IpRepl, IpStack, IpTable, Neighbor,
    Rip, Route, RoutingTable, RIP_PROTOCOL, TCP_PROTOCOL, TEST_PROTOCOL,
};
use crate::network_stacks::tcp::TcpStack;
use crate::parser::{IPConfig, RoutingType};
use crate::repl::{Repl, ReplHandler};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
/// A network router (a device on the network that forwards packets between its many interfaces)
pub struct Router {
    /// The interfaces belonging to this router
    pub interfaces: Arc<Mutex<HashMap<String, Arc<Mutex<Interface>>>>>,
    /// The forwarding table for this router
    pub forwarding_table: Arc<Mutex<ForwardingTable>>,
    /// The routing table for this router
    pub routing_table: Arc<Mutex<RoutingTable>>,
    /// The channel for receiving packets from the router's interfaces
    pub packet_receiver: Arc<Mutex<mpsc::Receiver<IpPacket>>>,
    /// The protocol handlers for this router (functions to handle packets of a specific type)
    pub protocol_handlers: Arc<Mutex<HashMap<u8, IpHandler>>>,
    /// The REPL handlers for this router (functions to handle REPL commands)
    pub repl_handlers: Arc<Mutex<HashMap<String, ReplHandler>>>,
    /// Whether RIP is enabled on this router
    pub rip_enabled: bool,
}

impl Router {
    /// Create a new router
    pub fn new(ip_config: IPConfig) -> Self {
        // Create a channel for sending packets from each interface thread to the main router thread
        let (packet_sender, packet_receiver) = mpsc::channel();

        // Create empty struct fields
        let mut interfaces = HashMap::new();
        let mut forwarding_table = ForwardingTable::new();
        let mut routing_table = RoutingTable::new();
        let rip_enabled = match ip_config.routing_mode {
            RoutingType::Rip => true,
            _ => false,
        };

        // Create the interfaces belonging to this node
        for interface_config in ip_config.interfaces {
            // Calculate the subnet this interface belongs to
            let subnet = IpNetwork::new(
                interface_config.assigned_ip,
                interface_config.assigned_prefix.prefix_len(),
            );

            // Create a new interface
            let interface = Interface::new(
                interface_config.name.clone(),
                interface_config.assigned_ip,
                interface_config.udp_port,
                subnet,
                packet_sender.clone(),
                HashMap::new(),
            );

            let interface = Arc::new(Mutex::new(interface));
            interfaces.insert(interface_config.name, interface.clone());

            // Add an entry to the forwarding table for the interface
            forwarding_table.insert(subnet, interface.clone());
        }

        // Create a hash set of RIP enabled neighbors
        let mut rip_neighbors_set = HashSet::new();

        if let Some(rip_neighbors) = ip_config.rip_neighbors.clone() {
            for ip_address in rip_neighbors {
                rip_neighbors_set.insert(ip_address);
            }
        }

        // Initialize the interface's {ip -> udp_port} tables based on the neighbor information in the configuration
        for neighbor_config in ip_config.neighbors {
            // Add the neighbor to the appropriate interface's neighbor table
            if let Some(interface) = interfaces.get(&neighbor_config.interface_name) {
                let mut interface = interface.lock().unwrap();

                let neighbor = Neighbor {
                    udp_port: neighbor_config.udp_port,
                    rip_enabled: rip_neighbors_set.contains(&neighbor_config.dest_addr),
                };

                interface
                    .neighbors
                    .insert(neighbor_config.dest_addr, neighbor);
            }
        }

        // Add all static routes to the routing table
        for (prefix, ip_address) in ip_config.static_routes {
            let network_ip = IpNetwork::new(prefix.addr(), prefix.prefix_len());

            let route = Route {
                next_hop: ip_address,
                cost: 1,
                creation_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
                is_static: true,
            };

            routing_table.insert(network_ip, route);
        }

        // Return the final router struct
        Self {
            interfaces: Arc::new(Mutex::new(interfaces)),
            forwarding_table: Arc::new(Mutex::new(forwarding_table)),
            routing_table: Arc::new(Mutex::new(routing_table)),
            packet_receiver: Arc::new(Mutex::new(packet_receiver)),
            protocol_handlers: Arc::new(Mutex::new(HashMap::new())),
            repl_handlers: Arc::new(Mutex::new(HashMap::new())),
            rip_enabled,
        }
    }

    /// Main entry point for the router
    pub fn start(&self) {
        // Register all IP handlers
        if self.rip_enabled {
            self.register_protocol_handler(RIP_PROTOCOL, <dyn Rip>::handle_rip_packet);
        }
        self.register_protocol_handler(TEST_PROTOCOL, <dyn IpStack>::handle_test_packet);
        self.register_protocol_handler(TCP_PROTOCOL, <dyn TcpStack>::handle_tcp_packet);
        // Register all REPL handlers
        self.register_ip_repl_handlers();

        // Start the main packet handling loop in a new thread
        self.receive_packets();

        // Start up each interface on the router
        {
            let interfaces = self.interfaces.lock().unwrap();
            for (_, interface) in interfaces.iter() {
                let interface = interface.lock().unwrap();
                interface.listen_for_packets();
            }
        }

        // Start the REPL in a new thread
        self.start_repl("router");

        if self.rip_enabled {
            // Start the RIP route reaper in a new thread
            self.start_route_reaper();
            // Request routing information from all neighbors
            self.send_rip_message_to_all(1, None);

            // Constantly send update messages to other routers using rip if enabled
            loop {
                thread::sleep(Duration::from_secs(5));
                self.send_rip_message_to_all(2, None);
            }
        } else {
            // Wait forever
            thread::park();
        }
    }
}
