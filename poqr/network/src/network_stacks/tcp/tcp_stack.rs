use super::{
    tcp_socket::TcpSocket, TcpConnection, TcpListenerSocket, TcpNormalSocket, TcpPacket,
    TcpPacketType, TcpSocketTable,
};
use crate::network_stacks::ip::{IpPacket, IpStack, PacketData, TCP_PROTOCOL};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::{Arc, Mutex},
};

/// A trait for a TCP stack that provides the necessary functionality for handling TCP packets.
pub trait TcpStack: IpStack {
    /// Wrap the `TcpStack` in an `Arc` for use in other threads.
    fn to_arc(&self) -> Arc<dyn TcpStack>;

    /// Get the IPv4 address of the node. This function assumes that the node has only one interface.
    fn ip_address(&self) -> Ipv4Addr {
        let interfaces = self.interfaces();
        let interfaces = interfaces.lock().unwrap();
        let interface = interfaces.values().next().unwrap();
        let interface = interface.lock().unwrap();
        interface.ip_address
    }

    /// Get the socket table of the node.
    fn socket_table(&self) -> Arc<Mutex<TcpSocketTable>>;

    /// Get the next ID to assign to a new socket.
    fn get_next_socket_id(&self) -> u32 {
        let socket_table = self.socket_table();
        let mut socket_table = socket_table.lock().unwrap();
        socket_table.get_next_id()
    }

    /// Insert a new entry into the socket table.
    fn insert_socket_table_entry(&self, connection: TcpConnection, socket: TcpSocket) {
        let socket_table = self.socket_table();
        let mut socket_table = socket_table.lock().unwrap();

        socket_table.insert(connection, socket);
    }

    /// Remove an entry from the socket table
    fn remove_socket_table_entry(&self, id: u32) {
        let socket_table = self.socket_table();
        let mut socket_table = socket_table.lock().unwrap();

        socket_table.remove_socket(id);
    }

    /// Get the listener socket for the given port. If the port is closed, return `None`.
    fn get_listener_socket(&self, port: u16) -> Option<TcpListenerSocket> {
        let socket_table = self.socket_table();
        let socket_table = socket_table.lock().unwrap();

        // All zeros except for the local port
        let listener_key = TcpConnection {
            local_address: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port),
            remote_address: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0),
        };

        match socket_table.get(&listener_key) {
            Some(socket) => socket.as_listener(),
            None => None,
        }
    }

    /// Get the normal socket for the given connection. If the connection hasn't been registered, return `None`.
    fn get_normal_socket(&self, connection: TcpConnection) -> Option<TcpNormalSocket> {
        let socket_table = self.socket_table();
        let socket_table = socket_table.lock().unwrap();

        match socket_table.get(&connection) {
            Some(socket) => socket.as_normal(),
            None => None,
        }
    }

    /// Iterate through the Socket Table and get the socket with the corresponding ID
    fn get_socket_by_id(&self, id: u32) -> Option<Arc<TcpSocket>> {
        let socket_table = self.socket_table();
        let socket_table = socket_table.lock().unwrap();

        for (_, socket) in socket_table.sockets.iter() {
            if socket.id() == id {
                return Some(socket.clone());
            }
        }

        None
    }

    /// Iterate through the Socket Table and get the normal socket with the corresponding remote address
    fn get_socket_by_remote_address(
        &self,
        remote_address: SocketAddrV4,
    ) -> Option<TcpNormalSocket> {
        let socket_table = self.socket_table();
        let socket_table = socket_table.lock().unwrap();

        for (connection, socket) in socket_table.sockets.iter() {
            if connection.remote_address == remote_address {
                return socket.as_normal();
            }
        }

        None
    }

    /// Takes in a socket id and returns the corresponding TCP connection
    fn get_connection(&self, socket_id: u32) -> Option<TcpConnection> {
        let socket_table = self.socket_table();
        let socket_table = socket_table.lock().unwrap();

        for (connection, socket) in socket_table.sockets.iter() {
            if socket.id() == socket_id {
                return Some(connection.clone());
            }
        }

        None
    }

    /// Send a TCP packet to the destination address given in the connection. Calls `send_packet` from the `IpStack` trait.
    fn construct_and_send_tcp_packet(
        &self,
        packet_type: TcpPacketType,
        connection: TcpConnection,
        sequence_number: u32,
        acknowledgment_number: u32,
        window_size: u16,
        data: Option<Vec<u8>>,
    ) {
        // Create the TCP packet and wrap it in an IP packet
        let tcp_packet = TcpPacket::new(
            packet_type,
            connection.clone(),
            sequence_number,
            acknowledgment_number,
            window_size,
            data,
        );
        let ip_packet = IpPacket::new(
            *connection.local_address.ip(),
            *connection.remote_address.ip(),
            TCP_PROTOCOL,
            PacketData::TcpMessage(tcp_packet),
        );

        // Send the IP packet
        self.send_packet(ip_packet);
    }

    /// Send a TCP packet to the destination address given in the connection. Calls `send_packet` from the `IpStack` trait.
    fn send_tcp_packet(&self, packet: TcpPacket, connection: TcpConnection) {
        // Create the TCP packet and wrap it in an IP packet
        let ip_packet = IpPacket::new(
            *connection.local_address.ip(),
            *connection.remote_address.ip(),
            TCP_PROTOCOL,
            PacketData::TcpMessage(packet),
        );

        // Send the IP packet
        self.send_packet(ip_packet);
    }

    /// Handle a SYN packet by registering a new connection with the appropriate listener socket (if it exists).
    fn handle_syn_packet(&self, packet: TcpPacket, connection: TcpConnection) {
        if let None = self.get_normal_socket(connection.clone()) {
            // No existing connection, so check if there is a listener socket
            if let Some(socket) = self.get_listener_socket(packet.header.destination_port) {
                socket
                    .register_connection(connection.remote_address, packet.header.sequence_number);
            }
        }
    }

    /// Handle a SYN-ACK packet by sending it to the corresponding normal socket (if it exists).
    fn handle_syn_ack_packet(&self, packet: TcpPacket, connection: TcpConnection) {
        if let Some(socket) = self.get_normal_socket(connection) {
            socket.handle_syn_ack_packet(packet);
        }
    }

    /// Handle an incoming ACK packet by sending it to the corresponding normal socket (if it exists).
    fn handle_ack_packet(&self, packet: TcpPacket, connection: TcpConnection) {
        if let Some(socket) = self.get_normal_socket(connection.clone()) {
            socket.handle_ack_packet(packet);
        }
    }

    /// Handle an incoming FIN packet by sending it to the corresponding normal socket (if it exists).
    fn handle_fin_packet(&self, packet: TcpPacket, connection: &TcpConnection) {
        let normal_socket = self.get_normal_socket(connection.clone());

        if let Some(mut socket) = normal_socket {
            socket.handle_fin_packet(packet, connection.clone());
        }
    }

    /// Open a new TCP listener socket on the given port.
    fn v_listen(&self, port: u16) -> TcpListenerSocket {
        let tcp_stack = TcpStack::to_arc(self);
        TcpListenerSocket::listen(tcp_stack, port)
    }

    /// Create a new TCP connection to the given address and port.
    fn v_connect(&self, address: Ipv4Addr, port: u16) -> Result<TcpNormalSocket, String> {
        let tcp_stack = TcpStack::to_arc(self);
        TcpNormalSocket::connect(tcp_stack, address, port)
    }
}

impl dyn TcpStack {
    /// Handle a TCP packet by calling the appropriate handler based on the packet type.
    pub fn handle_tcp_packet(ip_stack: &Arc<dyn IpStack>, packet: IpPacket) {
        if let PacketData::TcpMessage(tcp_packet) = &packet.data {
            if let Some(host) = ip_stack.as_host() {
                // Handle the packet as a host
                let tcp_header = &tcp_packet.header;
                let connection = TcpConnection {
                    local_address: SocketAddrV4::new(
                        packet.destination(),
                        tcp_header.destination_port,
                    ),
                    remote_address: SocketAddrV4::new(packet.source(), tcp_header.source_port),
                };

                // Validate the checksum
                match tcp_packet.clone().validate_checksum(connection.clone()) {
                    Ok(valid) => {
                        if !valid {
                            // Drop the packet if the checksum is invalid
                            eprintln!("Invalid TCP checksum");
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error validating TCP checksum: {}", e);
                        return;
                    }
                }

                // Call the appropriate handler based on the TCP packet type
                if let Ok(packet_type) =
                    TcpPacketType::from_flags(tcp_header.syn, tcp_header.ack, tcp_header.fin)
                {
                    match packet_type {
                        TcpPacketType::Syn => {
                            host.handle_syn_packet(tcp_packet.clone(), connection);
                        }
                        TcpPacketType::SynAck => {
                            host.handle_syn_ack_packet(tcp_packet.clone(), connection);
                        }
                        TcpPacketType::Ack => {
                            host.handle_ack_packet(tcp_packet.clone(), connection);
                        }
                        TcpPacketType::Fin => {
                            host.handle_fin_packet(tcp_packet.clone(), &connection);
                        }
                        _ => {}
                    }
                }
            } else if let Some(router) = ip_stack.as_router() {
                // Handle the packet as a router
                router.send_packet(packet);
            }
        }
    }
}
