use crate::network_stacks::ip::{
    IpNetwork, IpPacket, IpStack, PacketData, RIPEntry, RipMessage, Route, RIP_PROTOCOL,
    RIP_REQUEST, RIP_RESPONSE,
};
use std::cmp::min;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Routing Information Protocol (RIP) implementation for routers
pub trait Rip: IpStack {
    /// Wrap `self` in an `Arc` for use in threads
    fn to_arc(&self) -> Arc<dyn Rip>;

    /// Send an RIP request message to a neighboring router
    fn send_rip_request(&self, destination: Ipv4Addr, source: Ipv4Addr) {
        let rip_message = RipMessage {
            command: RIP_REQUEST,
            num_entries: 0,
            entries: Vec::new(),
        };
        let packet = IpPacket::new(
            source,
            destination,
            RIP_PROTOCOL,
            PacketData::RipMessage(rip_message),
        );
        self.send_packet(packet);
    }

    /// Send an RIP response message to a neighboring router
    ///
    /// If updates is `Some`, the response will contain only the given updates (triggered updates).
    /// Otherwise, if updates is `None`, the response will contain the entire forwarding and routing tables (periodic updates / responses)
    fn send_rip_response(
        &self,
        updates: Option<Vec<(IpNetwork, Route)>>,
        destination: Ipv4Addr,
        source: Ipv4Addr,
    ) {
        // Populate the RIP entries based on whether updates are provided (for triggered updates) or not (for periodic updates / responses)
        let mut entries = Vec::new();

        match updates {
            Some(updates) => {
                // Convert only the given routing table routes to RIP entries
                for (subnet, route) in updates {
                    let mut cost = route.cost;
                    if destination == route.next_hop {
                        cost = 16; // poison reverse
                    }
                    let entry = RIPEntry {
                        cost,
                        address: u32::from(subnet.address),
                        mask: subnet.bit_mask(),
                    };
                    entries.push(entry);
                }
            }
            None => {
                // Convert the entire forwarding + routing tables to RIP entries
                let forwarding_table = self.forwarding_table();
                let forwarding_table = forwarding_table.lock().unwrap();

                for (network_ip, _) in &forwarding_table.interfaces {
                    // Send the route for each network in the routing table
                    let entry = RIPEntry {
                        cost: 0,
                        address: u32::from(network_ip.address),
                        mask: network_ip.bit_mask(),
                    };
                    entries.push(entry);
                }

                let routing_table = self.routing_table();
                let routing_table = routing_table.lock().unwrap();

                for (network_ip, route) in &routing_table.routes {
                    // Send the route for each network in the routing table
                    let mut cost = route.cost;
                    if destination == route.next_hop {
                        cost = 16; // poison reverse
                    }
                    let entry = RIPEntry {
                        cost,
                        address: u32::from(network_ip.address),
                        mask: network_ip.bit_mask(),
                    };
                    entries.push(entry);
                }
            }
        }

        // Construct the RIP message and send it
        let rip_message = RipMessage {
            command: RIP_RESPONSE,
            num_entries: entries.len() as u16,
            entries,
        };
        let packet = IpPacket::new(
            source,
            destination,
            RIP_PROTOCOL,
            PacketData::RipMessage(rip_message),
        );
        self.send_packet(packet);
    }

    /// Determine the source and destination IP addresses for packets sent to RIP neighbors
    fn neighbor_source_dest_pairs(&self) -> Vec<(Ipv4Addr, Ipv4Addr)> {
        let mut packet_info = Vec::new();

        let interfaces = self.interfaces();
        let interfaces = interfaces.lock().unwrap();

        for (_, interface) in interfaces.iter() {
            let interface = interface.lock().unwrap();
            let neighbors = interface.neighbors.clone();
            let source_ip = interface.ip_address;

            for dest_ip in neighbors.keys() {
                if neighbors[dest_ip].rip_enabled {
                    packet_info.push((*dest_ip, source_ip));
                }
            }
        }

        packet_info
    }

    /// Send an RIP message to all neighboring routers
    fn send_rip_message_to_all(&self, command: u16, updates: Option<Vec<(IpNetwork, Route)>>) {
        match command {
            1 => {
                // Request command: send the request to all RIP neighbors
                for (dest_ip, source_ip) in self.neighbor_source_dest_pairs() {
                    self.send_rip_request(dest_ip, source_ip);
                }
            }
            2 => {
                // Response command: send the response to all RIP neighbors
                for (dest_ip, source_ip) in self.neighbor_source_dest_pairs() {
                    self.send_rip_response(updates.clone(), dest_ip, source_ip);
                }
            }
            _ => eprintln!("router: Invalid RIP message command."),
        }
    }

    /// Start the RIP reaper thread, which will remove expired routes from the routing table
    fn start_route_reaper(&self) {
        let rip_node = Rip::to_arc(self);

        thread::spawn(move || loop {
            let mut expired_routes = Vec::new();

            {
                let routing_table = rip_node.routing_table();
                let routing_table = routing_table.lock().unwrap();
                let time_now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                for (subnet, route) in routing_table.routes.iter() {
                    if time_now - route.creation_time >= 12 && !route.is_static {
                        // The route has expired since 12 seconds have elapsed
                        println!("Route expired: {:?}", subnet);
                        expired_routes.push((*subnet, *route));
                    }
                }
            }

            for (subnet, route) in expired_routes {
                // Set the route's cost to 16
                let mut new_route = route.clone();
                new_route.cost = 16;
                // Send the updated routing table to all neighbors
                let updates = vec![(subnet, new_route)];
                rip_node.send_rip_message_to_all(2, Some(updates));
                // Delete the routing table entry
                rip_node.remove_routing_table_entry(subnet);
                // println!("Removed expired route: {:?}", subnet);
            }

            // Wait for 1 second before repeating
            thread::sleep(std::time::Duration::from_millis(1000));
        });
    }
}

impl dyn Rip {
    /// Handle an RIP message received by the router
    pub fn handle_rip_packet(ip_stack: &Arc<dyn IpStack>, packet: IpPacket) {
        if let Some(router) = ip_stack.as_router() {
            let rip_message = match &packet.data {
                PacketData::RipMessage(rip_message) => rip_message,
                _ => panic!("router: Received non-RIP message in RIP handler."),
            };
            let source = packet.source();
            let destination = packet.destination();

            match rip_message.command {
                1 => {
                    // Message is a request for this router to send its routing information
                    router.send_rip_response(None, source, destination);
                }
                2 => {
                    // Message is a response giving this router new information that it requested
                    for entry in &rip_message.entries {
                        let cost = min(entry.cost + 1, 16);
                        let time_now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_secs();
                        let subnet = IpNetwork::new(entry.address(), entry.mask_length());

                        let new_route = Route {
                            next_hop: source,
                            cost,
                            creation_time: time_now,
                            is_static: false,
                        };

                        match router.get_routing_table_entry(subnet) {
                            Some(existing_route) => {
                                if cost < existing_route.cost {
                                    // If the new cost is lower, update the entry
                                    router.insert_routing_table_entry(subnet, new_route);
                                    // Send the updated routing table to all neighbors
                                    let updates = vec![(subnet, new_route)];
                                    router.send_rip_message_to_all(2, Some(updates));
                                } else if cost > existing_route.cost {
                                    // If the new cost is higher, only update if the next hop is the same
                                    if new_route.next_hop == existing_route.next_hop {
                                        // Same next hop, update to higher cost
                                        router.insert_routing_table_entry(subnet, new_route);
                                        // Send the updated routing table to all neighbors
                                        let updates = vec![(subnet, new_route)];
                                        router.send_rip_message_to_all(2, Some(updates));
                                    }
                                } else if new_route.next_hop == existing_route.next_hop {
                                    // If the costs are equal and the next hops are the same, refresh the creation time
                                    let mut refreshed_route = existing_route.clone();
                                    refreshed_route.creation_time = time_now;
                                    router.insert_routing_table_entry(subnet, refreshed_route);
                                }
                            }
                            None => {
                                // There is no entry in the routing table mapping the subnet, so check the forwarding
                                // table. If the destination is part of a new subnet, add it to the routing table
                                if router.get_forwarding_table_entry(subnet).is_none() {
                                    router.insert_routing_table_entry(subnet, new_route);
                                    // Send the updated routing table to all neighbors
                                    let updates = vec![(subnet, new_route)];
                                    router.send_rip_message_to_all(2, Some(updates));
                                }
                            }
                        }
                    }
                }
                _ => eprintln!("router: Invalid RIP message command."),
            }
        }
    }
}
