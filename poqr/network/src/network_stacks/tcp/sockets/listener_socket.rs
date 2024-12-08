use super::super::{TcpConnection, TcpConnectionState, TcpNormalSocket};
use crate::network_stacks::tcp::tcp_socket::TcpSocket;
use crate::network_stacks::tcp::TcpPacketType;
use crate::network_stacks::TcpStack;
use std::net::SocketAddrV4;
use std::sync::{mpsc, Arc, Mutex};

#[derive(Clone)]
/// A TCP listener socket that listens for incoming connections on a given port.
pub struct TcpListenerSocket {
    /// The TCP stack that the socket is associated with.
    pub tcp_stack: Arc<dyn TcpStack>,
    /// The ID of the socket.
    pub id: u32,
    /// The IP address and port that the socket is bound to.
    pub local_address: SocketAddrV4,
    /// The sender half of the channel for sending new connections to the listener socket.
    pub connection_sender: mpsc::Sender<TcpConnection>,
    /// The receiver half of the channel for receiving new connections from the listener socket.
    pub connection_receiver: Arc<Mutex<mpsc::Receiver<TcpConnection>>>,
}

impl TcpListenerSocket {
    /// Create a new TCP listener socket with the given TCP stack and local IP address.
    pub fn new(tcp_stack: Arc<dyn TcpStack>, port: u16) -> TcpListenerSocket {
        let local_address = SocketAddrV4::new(tcp_stack.ip_address(), port);
        let (connection_sender, connection_receiver) = mpsc::channel();

        TcpListenerSocket {
            tcp_stack: tcp_stack.clone(),
            id: tcp_stack.get_next_socket_id(),
            local_address,
            connection_sender,
            connection_receiver: Arc::new(Mutex::new(connection_receiver)),
        }
    }

    /// Helper for `v_listen`. Open a new TCP listener socket on the given port.
    pub fn listen(tcp_stack: Arc<dyn TcpStack>, port: u16) -> TcpListenerSocket {
        // Create a new listener socket
        let socket = TcpListenerSocket::new(tcp_stack.clone(), port);

        // Add the listener to the socket table
        let listener_key = TcpConnection {
            local_address: SocketAddrV4::new([0, 0, 0, 0].into(), port),
            remote_address: SocketAddrV4::new([0, 0, 0, 0].into(), 0),
        };
        tcp_stack.insert_socket_table_entry(listener_key, TcpSocket::Listener(socket.clone()));

        socket
    }

    /// Upon receiving a SYN packet, register a new connection with the given remote address and sequence number.
    /// This creates a new normal socket with status `SynReceived`.
    pub fn register_connection(&self, remote_address: SocketAddrV4, remote_sequence_number: u32) {
        // Create a new connection
        let connection = TcpConnection {
            local_address: self.local_address,
            remote_address,
        };

        // Create a new normal socket for the connection
        let socket = TcpNormalSocket::new(
            self.tcp_stack.clone(),
            TcpConnectionState::SynReceived,
            remote_sequence_number,
        );

        // The SYN packet we received to initiate the connection consumes one sequence number
        let mut receive_buffer = socket.receive_buffer.lock().unwrap();
        receive_buffer.increment_next(1);

        // Add the connection to the socket table
        self.tcp_stack
            .insert_socket_table_entry(connection.clone(), TcpSocket::Normal(socket.clone()));

        // Send the connection through the listener socket's channel for further handling if/when v_accept is called
        self.connection_sender.send(connection).unwrap();
    }

    /// Accept a new connection. This function blocks until a new connection is available.
    pub fn v_accept(&self) -> Result<TcpNormalSocket, String> {
        let connection_receiver = self.connection_receiver.lock().unwrap();

        match connection_receiver.recv() {
            Ok(connection) => {
                // Get the normal socket for the connection
                let socket = self
                    .tcp_stack
                    .get_normal_socket(connection.clone())
                    .unwrap();

                // Send a SYN-ACK packet
                {
                    let receive_buffer = socket.receive_buffer.lock().unwrap();
                    let mut send_buffer = socket.send_buffer.lock().unwrap();

                    self.tcp_stack.construct_and_send_tcp_packet(
                        TcpPacketType::SynAck,
                        connection.clone(),
                        send_buffer.next,
                        receive_buffer.next,
                        receive_buffer.window_size,
                        None,
                    );

                    // A SYN-ACK packet consumes one sequence number
                    send_buffer.increment_next(1);
                }

                // Wait for ACK packet
                socket.wait_for_state(TcpConnectionState::Established);

                // Start the sender and retransmitter threads
                socket.start_sending(connection.clone());
                socket.start_retransmitting(connection);

                Ok(socket.clone())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// close the connection
    pub fn v_close(&self) {
        self.tcp_stack.remove_socket_table_entry(self.id); // remove the entry from the table
    }
}
