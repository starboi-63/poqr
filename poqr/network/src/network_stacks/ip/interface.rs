use super::{IpNetwork, IpPacket};
use std::collections::HashMap;
use std::net::{Ipv4Addr, UdpSocket};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub const MAX_PACKET_SIZE: usize = 1500;

#[derive(Debug)]
/// A network interface (a connection to the network that can send and receive packets)
pub struct Interface {
    /// The name of the interface (e.g. "if0")
    pub name: String,
    /// The IP address of the interface (e.g. 10.0.0.2)
    pub ip_address: Ipv4Addr,
    /// The UDP port (i.e. virtual MAC Address) of the interface (e.g. 5512)
    pub subnet: IpNetwork,
    /// The UDP socket used to send and receive packets
    pub udp_socket: UdpSocket,
    /// The channel for sending packets to the main thread of the node this interface belongs to
    pub packet_sender: mpsc::Sender<IpPacket>,
    /// Whether the interface is enabled or disabled
    pub state: Arc<AtomicBool>,
    /// The neighbors of this interface (IP address -> Neighbor)
    pub neighbors: HashMap<Ipv4Addr, Neighbor>,
}

#[derive(Debug, Clone)]
/// A neighbor of an interface (a device connected to the interface)
pub struct Neighbor {
    /// The UDP port (i.e. virtual MAC Address) of the neighbor
    pub udp_port: u16,
    /// Whether RIP is enabled for this neighbor
    pub rip_enabled: bool,
}

impl Interface {
    /// Create a new interface
    pub fn new(
        name: String,
        ip_address: Ipv4Addr,
        udp_port: u16,
        subnet: IpNetwork,
        packet_sender: mpsc::Sender<IpPacket>,
        neighbors: HashMap<Ipv4Addr, Neighbor>,
    ) -> Self {
        let udp_socket =
            UdpSocket::bind(format!("127.0.0.1:{udp_port}")).expect("Could not bind UDP socket");

        Self {
            name,
            ip_address,
            subnet,
            udp_socket,
            packet_sender,
            state: Arc::new(AtomicBool::new(true)),
            neighbors,
        }
    }

    /// Spawn a new thread to listen for packets on the interface
    pub fn listen_for_packets(&self) {
        let mut buf = [0; MAX_PACKET_SIZE];

        // Clone the UDP socket and packet sender so that the memory can be moved to the new thread
        let udp_socket = self
            .udp_socket
            .try_clone()
            .expect("Could not clone UDP socket");
        let sender = self.packet_sender.clone();
        let state = self.state.clone();

        thread::spawn(move || {
            udp_socket
                .set_read_timeout(Some(Duration::from_millis(100)))
                .expect("interface: Could not set read timeout on UDP socket");

            loop {
                // If the interface is disabled, stop listening for packets
                if !state.load(Ordering::Relaxed) {
                    return;
                }

                // Try to receive a packet as a byte stream from the UDP socket
                // NOTE: we set a read timeout, so this will not block indefinitely
                if let Ok((num_bytes, _)) = udp_socket.recv_from(&mut buf) {
                    // Deserialize the packet
                    let packet = IpPacket::deserialize(&buf[..num_bytes]);

                    // Validate the checksum of the packet
                    if !packet.validate_checksum() {
                        eprintln!("interface: Invalid checksum, dropping packet");
                        continue;
                    }

                    // Send the packet to the router's main thread
                    sender
                        .send(packet)
                        .expect("interface: Could not send packet to router's main thread");
                }
            }
        });
    }

    /// Send a packet out of the interface (or to the OS if the destination is local)
    pub fn send_packet(&self, packet: IpPacket, next_hop: Option<Ipv4Addr>) {
        // If the interface is disabled, do not send the packet
        if !self.state.load(Ordering::Relaxed) {
            eprintln!("interface: Interface is disabled, refusing to send packet");
            return;
        }

        // TTL checks happen within the router's main thread, so we don't need to worry about them here
        match next_hop {
            Some(next_hop) => {
                // The destination is not local, so send the packet to the next hop
                let buf = packet.serialize();
                let udp_port = self.neighbors.get(&next_hop).unwrap().udp_port;

                self.udp_socket
                    .send_to(&buf, format!("127.0.0.1:{udp_port}"))
                    .expect("interface: Could not send data to UDP socket");
            }
            None => packet.print(), // The destination is local, so send the packet to the "OS"
        }
    }

    /// Enable the interface. This will restart the packet listener thread if it was previously disabled
    pub fn enable(&self) {
        if !self.state.load(Ordering::Relaxed) {
            self.state.store(true, Ordering::Relaxed);
            self.listen_for_packets();
        }
    }

    /// Disable the interface, preventing it from sending or receiving packets
    pub fn disable(&self) {
        self.state.store(false, Ordering::Relaxed);
    }
}
