use super::{IpPacket, PacketData, TestMessage, TEST_COMMAND, TEST_PROTOCOL};
use crate::network_stacks::IpStack;
use crate::repl::Repl;
use std::net::Ipv4Addr;
use std::sync::Arc;

/// IP-specific REPL commands
pub trait IpRepl: IpStack + Repl {
    /// Register all IP-specific command handlers with the REPL
    fn register_ip_repl_handlers(&self) {
        // Define handlers that can take in a generic REPL and downcast it to an IP REPL
        fn list_interfaces_handler(repl: Arc<dyn Repl>, _args: Vec<&str>) {
            if let Some(ip_repl) = repl.as_ip_repl() {
                ip_repl.list_interfaces();
            }
        }

        fn list_neighbors_handler(repl: Arc<dyn Repl>, _args: Vec<&str>) {
            if let Some(ip_repl) = repl.as_ip_repl() {
                ip_repl.list_neighbors();
            }
        }

        fn list_routes_handler(repl: Arc<dyn Repl>, _args: Vec<&str>) {
            if let Some(ip_repl) = repl.as_ip_repl() {
                ip_repl.list_routes();
            }
        }

        fn enable_interface_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 1 {
                eprintln!("Usage: up <ifname>");
                return;
            }

            if let Some(ip_repl) = repl.as_ip_repl() {
                ip_repl.enable_interface(args[0]);
            }
        }

        fn disable_interface_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 1 {
                eprintln!("Usage: down <ifname>");
                return;
            }

            if let Some(ip_repl) = repl.as_ip_repl() {
                ip_repl.disable_interface(args[0]);
            }
        }

        fn send_test_packet_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 2 {
                eprintln!("Usage: send <ip-addr> <message>");
                return;
            }

            if let Some(ip_repl) = repl.as_ip_repl() {
                if let Ok(destination) = args[0].parse::<Ipv4Addr>() {
                    let message = args[1..].join(" ");
                    ip_repl.send_test_packet(destination, message);
                } else {
                    eprintln!("Invalid IP address format: {}", args[1]);
                }
            }
        }

        // Register handlers for IP-related REPL commands
        self.register_repl_handler("li", list_interfaces_handler);
        self.register_repl_handler("ln", list_neighbors_handler);
        self.register_repl_handler("lr", list_routes_handler);
        self.register_repl_handler("up", enable_interface_handler);
        self.register_repl_handler("down", disable_interface_handler);
        self.register_repl_handler("send", send_test_packet_handler);
    }

    /// List the node's interfaces
    fn list_interfaces(&self) {
        let interfaces = self.interfaces();
        let interfaces = interfaces.lock().unwrap();

        println!("Name  Addr/Prefix State");
        for (_, interface) in interfaces.iter() {
            let interface = interface.lock().unwrap();
            println!(
                " {:<3}  {:<11}/{:<2} {}",
                interface.name,
                interface.ip_address,
                interface.subnet.mask_length,
                if interface.state.load(std::sync::atomic::Ordering::Relaxed) {
                    "up"
                } else {
                    "down"
                }
            );
        }
    }

    /// List the node's neighbors
    fn list_neighbors(&self) {
        let interfaces = self.interfaces();
        let interfaces = interfaces.lock().unwrap();

        println!("Iface          VIP          UDPAddr");
        for (_, interface) in interfaces.iter() {
            let interface = interface.lock().unwrap();
            if interface.state.load(std::sync::atomic::Ordering::Relaxed) {
                // Iterate through neighbors
                for (neighbor_ip, neighbor) in interface.neighbors.iter() {
                    println!(
                        "{:<10} {:<12} {}:{}",
                        interface.name, interface.ip_address, neighbor_ip, neighbor.udp_port
                    );
                }
            }
        }
    }

    /// List the node's routing and forwarding tables
    fn list_routes(&self) {
        // Lock forwarding table before routing table to avoid deadlock
        let forwarding_table = self.forwarding_table();
        let forwarding_table = forwarding_table.lock().unwrap();
        let routing_table = self.routing_table();
        let routing_table = routing_table.lock().unwrap();

        println!("T       Prefix       Next hop       Cost");

        // List entries in the routing table (denoted as 'S' for static and 'R' for RIP)
        for (network_ip, route) in routing_table.routes.iter() {
            let route_type = if route.is_static { "S" } else { "R" };

            println!(
                "{}  {}/{}   {}      {}",
                route_type, network_ip.address, network_ip.mask_length, route.next_hop, route.cost
            );
        }

        // List entries in the forwarding table (denoted as 'L')
        for (network_ip, interface) in forwarding_table.interfaces.iter() {
            // Lock the interface
            let interface = interface.lock().unwrap();

            println!(
                "L  {}/{}   LOCAL:{}      0",
                network_ip.address, network_ip.mask_length, interface.name
            );
        }
    }

    /// Enable an interface on the node
    fn enable_interface(&self, interface_name: &str) {
        if let Some(interface) = self.get_interface(&interface_name.to_string()) {
            let interface = interface.lock().unwrap();
            interface.enable();
            println!("Interface {} is now up.", interface_name);
        } else {
            println!("Interface {} not found.", interface_name);
        }
    }

    /// Disable an interface on the node
    fn disable_interface(&self, interface_name: &str) {
        if let Some(interface) = self.get_interface(&interface_name.to_string()) {
            let interface = interface.lock().unwrap();
            interface.disable();
            println!("Interface {} is now down.", interface_name);
        } else {
            println!("Interface {} not found.", interface_name);
        }
    }

    /// Send a test packet to a destination IP address on the virtual network
    fn send_test_packet(&self, destination: Ipv4Addr, message: String) {
        let (interface, _) = self.search_tables(destination);

        match interface {
            Some(interface) => {
                let source_ip = {
                    let interface = interface.lock().unwrap();
                    interface.ip_address
                };
                let test_message = TestMessage {
                    command: TEST_COMMAND,
                    data: message.into(),
                };
                let packet = IpPacket::new(
                    source_ip,
                    destination,
                    TEST_PROTOCOL,
                    PacketData::TestMessage(test_message),
                );
                self.send_packet(packet);
            }
            None => {
                eprintln!("No route to target.");
            }
        }
    }
}
