use super::{TcpConnection, TcpSocket};
use std::collections::HashMap;
use std::sync::Arc;

/// A table mapping TCP connections to sockets
pub struct TcpSocketTable {
    pub sockets: HashMap<TcpConnection, Arc<TcpSocket>>,
    /// The next ID to assign to a new socket
    pub next_id: u32,
}

impl TcpSocketTable {
    /// Create a new TCP socket table
    pub fn new() -> TcpSocketTable {
        TcpSocketTable {
            sockets: HashMap::new(),
            next_id: 0,
        }
    }

    /// Get the next ID to assign to a new socket.
    pub fn get_next_id(&mut self) -> u32 {
        let next_id = self.next_id;
        self.next_id += 1;
        next_id
    }

    /// Get the TCP socket which corresponds to the given connection
    pub fn get(&self, connection: &TcpConnection) -> Option<Arc<TcpSocket>> {
        self.sockets.get(connection).cloned()
    }

    /// Insert a new socket into the socket table
    pub fn insert(&mut self, connection: TcpConnection, socket: TcpSocket) {
        self.sockets.insert(connection, Arc::new(socket));
    }

    /// Remove a TCP connection from the socket table
    pub fn remove_connection(&mut self, connection: TcpConnection) {
        self.sockets.remove(&connection);
    }

    /// Remove a TCP connection from the socket table using the given socket ID
    pub fn remove_socket(&mut self, id: u32) {
        let connections_to_remove: Vec<TcpConnection> = self
            .sockets
            .iter()
            .filter(|(_, socket)| socket.id() == id) // Check if the socket's id matches
            .map(|(conn, _)| conn.clone())
            .collect();

        for conn in connections_to_remove {
            self.sockets.remove(&conn);
        }
    }
}
