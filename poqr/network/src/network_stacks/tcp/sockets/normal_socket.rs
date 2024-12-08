use super::super::TcpConnectionState;
use crate::network_stacks::tcp::{
    TcpConnection, TcpPacket, TcpPacketType, TcpReceiveBuffer, TcpRetransmissionQueue,
    TcpSendBuffer, TcpSocket, RECEIVE_WINDOW_SIZE,
};
use crate::network_stacks::TcpStack;
use std::cmp::min;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

// Must be less than MAX_PACKET_SIZE - 20 (IPv4 header) - 20 (TCP header)
// from crate::network_stacks::ip::MAX_PACKET_SIZE
const MAX_SEGMENT_SIZE: usize = 1024;

#[derive(Clone)]
/// A TCP normal socket that represents an ongoing TCP connection between two hosts.
pub struct TcpNormalSocket {
    /// The TCP stack that the socket is associated with.
    pub tcp_stack: Arc<dyn TcpStack>,
    /// The ID of the socket.
    pub id: u32,
    /// A buffer for sending data segments.
    pub send_buffer: Arc<Mutex<TcpSendBuffer>>,
    /// A buffer for receiving data segments.
    pub receive_buffer: Arc<Mutex<TcpReceiveBuffer>>,
    /// The retransmission queue for sent data segments that have not yet been acknowledged.
    pub retransmission_queue: Arc<Mutex<TcpRetransmissionQueue>>,
    /// The state of the socket connection (along with a condition variable for notifying threads waiting for state changes).
    pub state: Arc<(Mutex<TcpConnectionState>, Condvar)>,
    /// A condition variable for notifying threads waiting for the socket to be able to read more data from the receive buffer.
    pub can_read: Arc<(Mutex<bool>, Condvar)>,
    /// A condition variable for notifying threads waiting for the socket to be able to write more data to the send buffer.
    pub can_write: Arc<(Mutex<bool>, Condvar)>,
    /// A condition variable for notifying threads waiting for the socket to be able to send more data from the send buffer.
    pub can_send: Arc<(Mutex<bool>, Condvar)>,
    /// A condition variable for notifying threads waiting for the socket to close.
    pub closing: Arc<(Mutex<bool>, Condvar)>,
    /// A condition variable for notifying threads waiting for the socket to completely finish sending data.
    pub done_sending: Arc<(Mutex<bool>, Condvar)>,
    /// A condition variable for notifying threads waiting for the socket to completely finish reading data.u32
    pub done_reading: Arc<(Mutex<bool>, Condvar)>,
}

impl TcpNormalSocket {
    /// Construct a new `TcpNormalSocket`.
    pub fn new(
        tcp_stack: Arc<dyn TcpStack>,
        state: TcpConnectionState,
        remote_sequence_number: u32,
    ) -> TcpNormalSocket {
        TcpNormalSocket {
            tcp_stack: tcp_stack.clone(),
            id: tcp_stack.get_next_socket_id(),
            send_buffer: Arc::new(Mutex::new(TcpSendBuffer::new())),
            receive_buffer: Arc::new(Mutex::new(TcpReceiveBuffer::new(remote_sequence_number))),
            retransmission_queue: Arc::new(Mutex::new(TcpRetransmissionQueue::new())),
            state: Arc::new((Mutex::new(state), Condvar::new())),
            can_read: Arc::new((Mutex::new(false), Condvar::new())),
            can_write: Arc::new((Mutex::new(true), Condvar::new())),
            can_send: Arc::new((Mutex::new(false), Condvar::new())),
            closing: Arc::new((Mutex::new(false), Condvar::new())),
            done_sending: Arc::new((Mutex::new(true), Condvar::new())),
            done_reading: Arc::new((Mutex::new(true), Condvar::new())),
        }
    }

    /// Get a random high port number that is not currently in use.
    pub fn random_high_port() -> u16 {
        const MIN_PORT: u16 = 20000;
        const MAX_PORT: u16 = 65535;

        let range = (MAX_PORT - MIN_PORT) as u32;

        loop {
            let random_offset = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos())
                % range;

            let port = MIN_PORT + random_offset as u16;

            if let Ok(socket) = UdpSocket::bind(("0.0.0.0", port)) {
                // check if it is an unused port
                drop(socket);
                return port;
            }
        }
    }

    /// Get the state of the socket connection.
    pub fn state(&self) -> TcpConnectionState {
        let (state, _) = &*self.state;
        let state = state.lock().unwrap();
        state.clone()
    }

    /// Set the state of the socket connection. This will notify any threads waiting for the state to change.
    pub fn set_state(&self, state: TcpConnectionState) {
        let (current_state, condvar) = &*self.state;
        let mut current_state = current_state.lock().unwrap();
        *current_state = state;
        condvar.notify_all();
    }

    /// Get whether the socket can write more data to the send buffer (i.e. when the send buffer is not full).
    pub fn can_write(&self) -> bool {
        let (can_write, _) = &*self.can_write;
        let can_write = can_write.lock().unwrap();
        *can_write
    }

    /// Set whether the socket can write more data to the send buffer. This will notify any threads
    /// waiting for the socket to be able to write more data.
    pub fn set_can_write(&self, value: bool) {
        let (can_write, condvar) = &*self.can_write;
        let mut can_write = can_write.lock().unwrap();
        *can_write = value;
        condvar.notify_all();
    }

    /// Get whether the socket can send more data (i.e. when the send buffer is non-empty and the number
    /// of bytes "in flight" is less than the receiver's window size).
    pub fn can_send(&self) -> bool {
        let (can_send, _) = &*self.can_send;
        let can_send = can_send.lock().unwrap();
        *can_send
    }

    /// Set whether the socket can send more data. This will notify any threads waiting for the socket
    /// to be able to send more data.
    pub fn set_can_send(&self, value: bool) {
        let (can_send, condvar) = &*self.can_send;
        let mut can_send = can_send.lock().unwrap();
        *can_send = value;
        condvar.notify_all();
    }

    /// Get whether the socket can read more data (i.e. when the receive buffer is not empty).
    pub fn can_read(&self) -> bool {
        let (can_read, _) = &*self.can_read;
        let can_read = can_read.lock().unwrap();
        *can_read
    }

    /// Set whether the socket can read more data. This will notify any threads waiting for the socket to be able to read more data.
    pub fn set_can_read(&self, value: bool) {
        let (can_read, condvar) = &*self.can_read;
        let mut can_read = can_read.lock().unwrap();
        *can_read = value;
        condvar.notify_all();
    }

    /// Get whether the socket is closing.
    pub fn closing(&self) -> bool {
        let (closing, _) = &*self.closing;
        let closing = closing.lock().unwrap();
        *closing
    }

    /// Set whether the socket is closing. This will notify any threads waiting for the socket to close.
    pub fn set_closing(&self, value: bool) {
        let (closing, condvar) = &*self.closing;
        let mut closing = closing.lock().unwrap();
        *closing = value;
        condvar.notify_all();
    }

    /// Get whether the socket is done sending data (i.e. after the send buffer is empty).
    pub fn done_sending(&self) -> bool {
        let (done_sending, _) = &*self.done_sending;
        let done_sending = done_sending.lock().unwrap();
        *done_sending
    }

    /// Set whether the socket is done sending data. This will notify any threads waiting for the socket to finish sending.
    pub fn set_done_sending(&self, value: bool) {
        let (done_sending, condvar) = &*self.done_sending;
        let mut done_sending = done_sending.lock().unwrap();
        *done_sending = value;
        condvar.notify_all();
    }

    /// Get whether the socket is done reading data (i.e. after the receive buffer is empty).
    pub fn done_reading(&self) -> bool {
        let (done_reading, _) = &*self.done_reading;
        let done_reading = done_reading.lock().unwrap();
        *done_reading
    }

    /// Set whether the socket is done reading data. This will notify any threads waiting for the socket to finish reading.
    pub fn set_done_reading(&self, value: bool) {
        let (done_reading, condvar) = &*self.done_reading;
        let mut done_reading = done_reading.lock().unwrap();
        *done_reading = value;
        condvar.notify_all();
    }

    /// Wait until the buffer has some readable content (i.e. is not empty).
    pub fn wait_to_read(&self) {
        let (can_read, condvar) = &*self.can_read;
        let mut can_read = can_read.lock().unwrap();

        while !*can_read && !self.closing() {
            can_read = condvar.wait(can_read).unwrap();
        }
    }

    /// Wait for the socket to be able to write more data to the send buffer (i.e. after the send buffer is no longer full).
    pub fn wait_to_write(&self) {
        let (lock, cvar) = &*self.can_write;
        let mut can_write = lock.lock().unwrap();

        while !*can_write && !self.closing() {
            can_write = cvar.wait(can_write).unwrap();
        }
    }

    /// Wait for the socket to be able to send more data (i.e. after receiving an acknowledgment).
    pub fn wait_to_send(&self) {
        let (lock, cvar) = &*self.can_send;
        let mut can_send = lock.lock().unwrap();

        while !*can_send {
            can_send = cvar.wait(can_send).unwrap();
        }
    }

    /// Wait for the state of the socket connection to change to the given state.
    pub fn wait_for_state(&self, state: TcpConnectionState) {
        let (current_state, condvar) = &*self.state;
        let mut current_state = current_state.lock().unwrap();

        while *current_state != state {
            current_state = condvar.wait(current_state).unwrap();
        }
    }

    /// Wait for the socket to finish sending data (i.e. after the send buffer is empty).
    pub fn wait_to_finish_sending(&self) {
        let (done_sending, condvar) = &*self.done_sending;
        let mut done_sending = done_sending.lock().unwrap();

        while !*done_sending {
            done_sending = condvar.wait(done_sending).unwrap();
        }
    }

    /// Spawn a new thread that will constantly retransmit data from the retransmission queue when necessary.
    pub fn start_retransmitting(&self, connection: TcpConnection) {
        let socket = self.clone();

        std::thread::spawn(move || loop {
            let sleep_duration = {
                let mut retransmission_queue = socket.retransmission_queue.lock().unwrap();
                retransmission_queue.retransmit(&socket, connection.clone());
                retransmission_queue.sleep_duration()
            };

            std::thread::sleep(sleep_duration);
        });
    }

    /// Spawn a new thread that will constantly send data from the send buffer when possible.
    pub fn start_sending(&self, connection: TcpConnection) {
        let socket = self.clone();

        std::thread::spawn(move || loop {
            // Wait for the socket to be able to send more data before locking the buffers and queue
            socket.wait_to_send();

            // Send as much data as possible within the receiver's window size
            let receive_buffer = socket.receive_buffer.lock().unwrap();
            let mut send_buffer = socket.send_buffer.lock().unwrap();
            let mut retransmission_queue = socket.retransmission_queue.lock().unwrap();

            let is_zero_window_probe;
            let num_bytes_to_send = if send_buffer.window_size.unwrap() == 0 {
                is_zero_window_probe = true;
                1
            } else {
                is_zero_window_probe = false;
                send_buffer.unsent_window_bytes()
            };
            let mut num_bytes_sent = 0;

            // If there is no data to send, send a 'Pure ACK' to acknowledge any received data
            if num_bytes_to_send == 0 {
                socket.tcp_stack.construct_and_send_tcp_packet(
                    TcpPacketType::Ack,
                    connection.clone(),
                    send_buffer.next,
                    receive_buffer.next,
                    receive_buffer.window_size,
                    None,
                );
            }

            // Otherwise, repeatedly construct segments of up to MAX_SEGMENT_SIZE to send
            while num_bytes_sent < num_bytes_to_send {
                // Calculate the start and end indices for the next segment of data to send
                let start = send_buffer.next_index();
                let offset = min(num_bytes_to_send - num_bytes_sent, MAX_SEGMENT_SIZE);
                let end = (start + offset) % send_buffer.data.len();

                // Construct the segment
                let segment = if start < end {
                    // Normal case: no wraparound
                    send_buffer.data[start..end].to_vec()
                } else {
                    // Wraparound case: concatenate the two slices
                    let mut segment = send_buffer.data[start..].to_vec();
                    segment.extend_from_slice(&send_buffer.data[..end]);
                    segment
                };

                // Send the segment in an ACK packet
                let ack_packet = TcpPacket::new(
                    TcpPacketType::Ack,
                    connection.clone(),
                    send_buffer.next,
                    receive_buffer.next,
                    receive_buffer.window_size,
                    Some(segment.clone()),
                );
                socket
                    .tcp_stack
                    .send_tcp_packet(ack_packet.clone(), connection.clone());

                // Push the ACK packet onto the retransmission queue
                retransmission_queue.push(ack_packet, is_zero_window_probe);

                // Update relevant counters
                send_buffer.increment_next(segment.len() as u32);
                num_bytes_sent += segment.len();
            }

            // All data within the receiver's window has been sent; block the sender thread to wait for acknowledgment
            socket.set_can_send(false);
        });
    }

    /// Helper for `v_connect`. Create a new TCP connection to the given address and port.
    pub fn connect(
        tcp_stack: Arc<dyn TcpStack>,
        address: Ipv4Addr,
        port: u16,
    ) -> Result<TcpNormalSocket, String> {
        let remote_address = SocketAddrV4::new(address, port);

        // Check if we already have a connection to the given address and port
        if let Some(socket) = tcp_stack.get_socket_by_remote_address(remote_address.clone()) {
            return Ok(socket);
        }

        // Get the local IP address and a random unused high port for the connection
        let local_ip = tcp_stack.ip_address();
        let local_port = Self::random_high_port();

        let connection = TcpConnection {
            local_address: SocketAddrV4::new(local_ip, local_port),
            remote_address,
        };

        // Construct the normal socket and add it to the socket table
        let socket = TcpNormalSocket::new(tcp_stack.clone(), TcpConnectionState::SynSent, 0);
        tcp_stack.insert_socket_table_entry(connection.clone(), TcpSocket::Normal(socket.clone()));

        // Send a SYN packet
        {
            let receive_buffer = socket.receive_buffer.lock().unwrap();
            let mut send_buffer = socket.send_buffer.lock().unwrap();

            tcp_stack.construct_and_send_tcp_packet(
                TcpPacketType::Syn,
                connection.clone(),
                send_buffer.next,
                receive_buffer.next,
                receive_buffer.window_size,
                None,
            );

            // A SYN consumes one sequence number
            send_buffer.increment_next(1);
        }

        // Wait for a SYN-ACK packet
        socket.wait_for_state(TcpConnectionState::Established); // Once this returns, remote sequence number is set

        // Send an ACK packet
        {
            let receive_buffer = socket.receive_buffer.lock().unwrap();
            let send_buffer = socket.send_buffer.lock().unwrap();

            tcp_stack.construct_and_send_tcp_packet(
                TcpPacketType::Ack,
                connection.clone(),
                send_buffer.next,
                receive_buffer.next,
                receive_buffer.window_size,
                None,
            );
        }

        // Start the sender and retransmitter threads
        socket.start_sending(connection.clone());
        socket.start_retransmitting(connection);

        Ok(socket.clone())
    }

    /// Send data to the remote host. This function will block until all data has been written to the send buffer.
    pub fn v_write(&self, data: Vec<u8>) -> Result<usize, String> {
        let mut i = 0; // Number of bytes written to the send buffer

        while i < data.len() {
            // Wait for space to free up in the send buffer before locking it
            self.wait_to_write();

            if self.closing() {
                return Err("Socket is closing".to_string());
            }

            // Write as much data as possible to the send buffer
            let mut send_buffer = self.send_buffer.lock().unwrap();
            let bytes_written = send_buffer.push(&data[i..]);

            // If the send buffer is full, prevent further writing until some data is sent
            if send_buffer.available_bytes() == 0 {
                self.set_can_write(false);
            }

            // If we wrote some data, notify the sender thread to send it
            if bytes_written > 0 {
                if send_buffer.unsent_window_bytes() > 0 {
                    self.set_can_send(true);
                }
                self.set_done_sending(false);
            }

            i += bytes_written;
        }

        Ok(i)
    }

    /// Read data from the remote host. This function will block until there is some data to read.
    pub fn v_read(&self, buffer: &mut Vec<u8>) -> Result<usize, String> {
        // Wait until there is some data to read before locking the receive buffer
        self.wait_to_read();

        if self.done_reading() {
            return Err("Socket is closed. No more data can be received".to_string());
        }

        // Read as much data as possible from the receive buffer
        let mut receive_buffer = self.receive_buffer.lock().unwrap();
        let bytes_to_read = min(buffer.len(), receive_buffer.readable_bytes());

        // Copy the data into the buffer
        for i in 0..bytes_to_read {
            let read_index = (receive_buffer.start_index + i) % receive_buffer.data.len();
            buffer[i] = receive_buffer.data[read_index].unwrap();
            // Clear the data after reading it
            receive_buffer.data[read_index] = None;
        }

        // Update the `start_index` as we've read this data
        receive_buffer.increment_start_index(bytes_to_read);
        receive_buffer.window_size = min(
            RECEIVE_WINDOW_SIZE as u16,
            receive_buffer.window_size + bytes_to_read as u16,
        );

        // If we read all the contiguous data, prevent further reading until more data is received
        if receive_buffer.start_index == receive_buffer.next_index {
            self.set_can_read(false);

            if self.closing() {
                self.set_done_reading(true);
            }
        }

        // If we succeeded in reading some data, set the receive buffer to be non-full
        if bytes_to_read > 0 {
            receive_buffer.is_full = false;
        }

        // Return the number of bytes read
        Ok(bytes_to_read)
    }

    /// Close the socket connection. This function will block until the connection is closed.
    pub fn v_close(&self) {
        // Wait for all data to be sent
        self.wait_to_finish_sending();
        self.set_closing(true);

        println!("Socket {} is closing", self.id);

        // Lock the buffers to ensure that the data is consistent
        let receive_buffer = self.receive_buffer.lock().unwrap();
        let mut send_buffer = self.send_buffer.lock().unwrap();
        let mut retransmission_queue = self.retransmission_queue.lock().unwrap();
        let connection = self.tcp_stack.get_connection(self.id).unwrap();

        match self.state() {
            TcpConnectionState::SynSent => {
                // If in SYN_SENT, delete the connection and transition to CLOSED.
                self.set_state(TcpConnectionState::Closed);
                self.tcp_stack.remove_socket_table_entry(self.id);
            }
            TcpConnectionState::SynReceived => {
                // Send a FIN to initiate close.
                let fin_packet = TcpPacket::new(
                    TcpPacketType::Fin,
                    connection.clone(),
                    send_buffer.next,
                    receive_buffer.next,
                    receive_buffer.window_size,
                    None,
                );
                self.tcp_stack
                    .send_tcp_packet(fin_packet.clone(), connection.clone());

                // Push the FIN packet onto the retransmission queue
                retransmission_queue.push(fin_packet, false);

                // A FIN packet consumes one sequence number
                send_buffer.increment_next(1);

                // Transition to FIN-WAIT-1
                self.set_state(TcpConnectionState::FinWait1);
            }
            TcpConnectionState::Established => {
                // Send a FIN to initiate close.
                let fin_packet = TcpPacket::new(
                    TcpPacketType::Fin,
                    connection.clone(),
                    send_buffer.next,
                    receive_buffer.next,
                    receive_buffer.window_size,
                    None,
                );
                self.tcp_stack
                    .send_tcp_packet(fin_packet.clone(), connection.clone());

                // Push the FIN packet onto the retransmission queue
                retransmission_queue.push(fin_packet, false);

                // A FIN packet consumes one sequence number
                send_buffer.increment_next(1);

                // Transition to FIN-WAIT-1
                self.set_state(TcpConnectionState::FinWait1);
            }
            TcpConnectionState::CloseWait => {
                // We've already received a FIN from the remote host. We can send our own FIN now.
                let fin_packet = TcpPacket::new(
                    TcpPacketType::Fin,
                    connection.clone(),
                    send_buffer.next,
                    receive_buffer.next,
                    receive_buffer.window_size,
                    None,
                );
                self.tcp_stack
                    .send_tcp_packet(fin_packet.clone(), connection.clone());

                // Push the FIN packet onto the retransmission queue
                retransmission_queue.push(fin_packet, false);

                // A FIN packet consumes one sequence number
                send_buffer.increment_next(1);

                // Transition to LAST-ACK
                self.set_state(TcpConnectionState::LastAck);
            }
            _ => eprintln!("Invalid state to close the connection"),
        }
    }

    /// Handle a SYN-ACK packet received by a socket in the SYN_SENT state. Establishes the connection.
    pub fn handle_syn_ack_packet(&self, packet: TcpPacket) {
        // Lock both buffers beforehand to prevent any other threads from modifying them
        let mut receive_buffer = self.receive_buffer.lock().unwrap();
        let mut send_buffer = self.send_buffer.lock().unwrap();

        if packet.header.acknowledgment_number == send_buffer.next {
            match self.state() {
                TcpConnectionState::SynSent => {
                    // Initialize the receive buffer and set the next sequence number
                    receive_buffer.initial_sequence_number = packet.header.sequence_number;
                    receive_buffer.set_next(packet.header.sequence_number.wrapping_add(1));
                    // Update the state of the send buffer (the SYN packet we sent has been acknowledged)
                    send_buffer.increment_unacknowledged(1);
                    send_buffer.window_size = Some(packet.header.window_size);
                    // Move through the TCP state machine to the ESTABLISHED state
                    self.set_state(TcpConnectionState::Established);
                }
                _ => {}
            }
        }
    }

    // Handle an ACK packet received by the socket.
    pub fn handle_ack_packet(&self, packet: TcpPacket) {
        // Lock both buffers and the retransmission queue beforehand to prevent any other threads from modifying them
        let mut receive_buffer = self.receive_buffer.lock().unwrap();
        let mut send_buffer = self.send_buffer.lock().unwrap();
        let mut retransmission_queue = self.retransmission_queue.lock().unwrap();

        match self.state() {
            TcpConnectionState::SynReceived => {
                if packet.header.acknowledgment_number == send_buffer.next {
                    // Our SYN has been acknowledged, so we can move to the ESTABLISHED state
                    send_buffer.increment_unacknowledged(1);
                    send_buffer.window_size = Some(packet.header.window_size);
                    self.set_state(TcpConnectionState::Established);
                }
            }
            TcpConnectionState::Established => {
                self.handle_data_ack_packet(
                    &mut receive_buffer,
                    &mut send_buffer,
                    &mut retransmission_queue,
                    &packet,
                );
            }
            TcpConnectionState::LastAck => {
                if packet.header.acknowledgment_number == send_buffer.next {
                    // Our FIN has been acknowledged, so we can close the connection
                    send_buffer.increment_unacknowledged(1);
                    retransmission_queue.acknowledge(packet.header.acknowledgment_number);
                    self.set_state(TcpConnectionState::Closed);
                    self.tcp_stack.remove_socket_table_entry(self.id);
                } else {
                    self.handle_data_ack_packet(
                        &mut receive_buffer,
                        &mut send_buffer,
                        &mut retransmission_queue,
                        &packet,
                    );
                }
            }
            TcpConnectionState::FinWait1 => {
                if packet.header.acknowledgment_number == send_buffer.next {
                    // Our FIN has been acknowledged, so we can move to FIN_WAIT_2
                    send_buffer.increment_unacknowledged(1);
                    retransmission_queue.acknowledge(packet.header.acknowledgment_number);
                    self.set_state(TcpConnectionState::FinWait2);
                } else {
                    self.handle_data_ack_packet(
                        &mut receive_buffer,
                        &mut send_buffer,
                        &mut retransmission_queue,
                        &packet,
                    );
                }
            }
            TcpConnectionState::FinWait2 => {
                self.handle_data_ack_packet(
                    &mut receive_buffer,
                    &mut send_buffer,
                    &mut retransmission_queue,
                    &packet,
                );
            }
            TcpConnectionState::TimeWait => {
                self.handle_data_ack_packet(
                    &mut receive_buffer,
                    &mut send_buffer,
                    &mut retransmission_queue,
                    &packet,
                );
            }
            TcpConnectionState::CloseWait => {
                self.handle_data_ack_packet(
                    &mut receive_buffer,
                    &mut send_buffer,
                    &mut retransmission_queue,
                    &packet,
                );
            }
            _ => {}
        }
    }

    /// Handle an ACK packet received by the socket which contains data.
    pub fn handle_data_ack_packet(
        &self,
        receive_buffer: &mut TcpReceiveBuffer,
        send_buffer: &mut TcpSendBuffer,
        retransmission_queue: &mut TcpRetransmissionQueue,
        packet: &TcpPacket,
    ) {
        if receive_buffer.is_valid_segment(packet.header.sequence_number, &packet.data) {
            // Update the state of both the send and receive buffers
            let new_contiguous_bytes =
                receive_buffer.receive_segment(packet.header.sequence_number, &packet.data);
            let new_acknowledged_bytes = send_buffer.acknowledge(
                packet.header.acknowledgment_number,
                packet.header.window_size,
            );
            retransmission_queue.acknowledge(packet.header.acknowledgment_number);

            // Slide the receive window to the right by the number of contiguous bytes received
            if let Some(new_contiguous_bytes) = new_contiguous_bytes {
                receive_buffer.slide_window(new_contiguous_bytes);
                self.set_can_read(true);
                self.set_can_send(true);
                self.set_done_reading(false);
            }

            // Slide the send window to the right by the number of acknowledged bytes
            if let Some(new_acknowledged_bytes) = new_acknowledged_bytes {
                send_buffer.slide_window(new_acknowledged_bytes);
                self.set_can_write(true);

                // If the send buffer is empty, then we're done sending data entirely
                if send_buffer.occupied_bytes() == 0 {
                    self.set_done_sending(true);
                }

                // Enable the sender thread if:
                // - there is receivable, unsent data in the send buffer
                // - the receiver's window size is zero and we need to send a zero-window probe
                // - the socket is closing and we need to ACK a FIN
                if send_buffer.unsent_window_bytes() > 0
                    || send_buffer.window_size.unwrap() == 0
                    || self.closing()
                {
                    self.set_can_send(true);
                }
            }
        } else {
            // Edge case: we already have gotten the "last" segment needed to fill our receive_buffer window and
            // slide it, but the sender may not have received our ACK informing them of this. In this scenario,
            // the sender's window will be full and they will continue retransmitting the same data. To prevent
            // an abort due to this, we need to resend an ACK.
            self.set_can_send(true);
        }
    }

    /// Handle a FIN packet received by the socket.
    pub fn handle_fin_packet(&mut self, packet: TcpPacket, connection: TcpConnection) {
        // Lock both buffers beforehand to prevent any other threads from modifying them
        let mut receive_buffer = self.receive_buffer.lock().unwrap();
        let send_buffer = self.send_buffer.lock().unwrap();

        if packet.header.sequence_number == receive_buffer.next {
            match self.state() {
                TcpConnectionState::Established => {
                    // A FIN packet consumes one sequence number
                    receive_buffer.increment_next(1);

                    // Send an ACK for the received FIN packet
                    self.tcp_stack.construct_and_send_tcp_packet(
                        TcpPacketType::Ack,
                        connection.clone(),
                        send_buffer.next,
                        receive_buffer.next,
                        receive_buffer.window_size,
                        None,
                    );

                    // Move through the TCP state machine to the CLOSE_WAIT state
                    // NOTE: setting `closing` to `true` will make `wait_to_read` non-blocking when `can_read` is also
                    // set to `true` in the next line. This prevents the socket from being stuck in the `CLOSE_WAIT` state.
                    self.set_closing(true);
                    self.set_can_read(true);
                    self.set_state(TcpConnectionState::CloseWait);
                }
                TcpConnectionState::FinWait2 => {
                    // A FIN packet consumes one sequence number
                    receive_buffer.increment_next(1);

                    // Send an ACK for the received FIN packet
                    self.tcp_stack.construct_and_send_tcp_packet(
                        TcpPacketType::Ack,
                        connection.clone(),
                        send_buffer.next,
                        receive_buffer.next,
                        receive_buffer.window_size,
                        None,
                    );

                    // Wait for all sent data to be acknowledged before moving to the TIME_WAIT state
                    self.wait_to_finish_sending();

                    // Move through the TCP state machine to the TIME_WAIT state
                    self.set_state(TcpConnectionState::TimeWait);
                    self.wait_and_delete();
                }
                TcpConnectionState::CloseWait => {
                    // There's a chance our original ACK was lost when we first received the FIN in the
                    // ESTABLISHED state and responded. If we get a FIN after transitioning to CLOSE_WAIT,
                    // we need to re-ACK it.
                    self.tcp_stack.construct_and_send_tcp_packet(
                        TcpPacketType::Ack,
                        connection.clone(),
                        send_buffer.next,
                        receive_buffer.next,
                        receive_buffer.window_size,
                        None,
                    );
                }
                TcpConnectionState::LastAck => {
                    // There's a chance our original ACK was lost when we first received the FIN in the
                    // ESTABLISHED state and responded. If we called `v_close` after transitioning to CLOSE_WAIT
                    // and moved further through the TCP state machine to LAST_ACK, we may still need to re-ACK
                    // the original FIN.
                    self.tcp_stack.construct_and_send_tcp_packet(
                        TcpPacketType::Ack,
                        connection.clone(),
                        send_buffer.next,
                        receive_buffer.next,
                        receive_buffer.window_size,
                        None,
                    );
                }
                TcpConnectionState::TimeWait => {
                    // There's a chance our original ACK was lost when we first received a FIN in the
                    // FIN_WAIT_2 state and responded. If we get a FIN after transitioning to TIME_WAIT,
                    // we need to re-ACK it.
                    self.tcp_stack.construct_and_send_tcp_packet(
                        TcpPacketType::Ack,
                        connection.clone(),
                        send_buffer.next,
                        receive_buffer.next,
                        receive_buffer.window_size,
                        None,
                    );
                }
                _ => {}
            }
        }
    }

    /// Delete the socket after a timeout.
    pub fn wait_and_delete(&self) {
        let socket = self.clone();

        thread::spawn(move || {
            // Wait five seconds before deleting the socket
            thread::sleep(Duration::from_secs(120));
            println!("Socket {} has closed.", socket.id);
            socket.set_state(TcpConnectionState::Closed);
            socket.tcp_stack.remove_socket_table_entry(socket.id);
        });
    }
}
