use crate::network_stacks::tcp::tcp_socket::TcpSocket;
use crate::network_stacks::TcpStack;
use crate::repl::Repl;
use std::fs;
use std::io::Write;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread;

/// TCP-specific REPL commands
pub trait TcpRepl: TcpStack + Repl {
    /// Register all TCP-specific command handlers with the REPL
    fn register_tcp_repl_handlers(&self) {
        /// Handle connection (c) - connects to a given IP and port
        fn connect_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 2 {
                eprintln!("Usage: c <vip> <port>");
                return;
            }

            let ip = args[0].parse().unwrap();
            let port = args[1].parse().unwrap();

            if let Some(tcp_repl) = repl.as_tcp_repl() {
                tcp_repl.connect(ip, port);
            }
        }

        /// Handle listen + accept (a) - listens on a given port and accepts connections
        fn listen_accept_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 1 {
                eprintln!("Usage: a <port>");
                return;
            }

            let port = args[0].parse().unwrap();

            if let Some(tcp_repl) = repl.as_tcp_repl() {
                tcp_repl.listen_accept(port);
            }
        }

        /// Handle list sockets (ls) - lists all open sockets
        fn list_sockets_handler(repl: Arc<dyn Repl>, _args: Vec<&str>) {
            if let Some(tcp_repl) = repl.as_tcp_repl() {
                tcp_repl.list_sockets();
            }
        }

        /// Handle send (s) - sends bytes to a socket
        fn send_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 2 {
                eprintln!("Usage: s <socket ID> <bytes>");
                return;
            }

            let socket_id = args[0].parse().unwrap();
            let data = args[1].as_bytes();

            if let Some(tcp_repl) = repl.as_tcp_repl() {
                tcp_repl.send(socket_id, data);
            }
        }

        /// Handle receive (r) - reads bytes from a socket
        fn receive_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 2 {
                eprintln!("Usage: r <socket ID> <num bytes>");
                return;
            }

            let socket_id: u32 = args[0].parse().unwrap();
            let num_bytes: usize = args[1].parse().unwrap();

            if let Some(tcp_repl) = repl.as_tcp_repl() {
                tcp_repl.receive(socket_id, num_bytes);
            }
        }

        /// Handle send file (sf) - sends a file to a given IP and port
        fn send_file_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 3 {
                eprintln!("Usage: sf <file path> <addr> <port>");
                return;
            }

            let file_path = args[0].to_string();
            let ip = args[1].parse().unwrap();
            let port = args[2].parse().unwrap();

            if let Some(tcp_repl) = repl.as_tcp_repl() {
                tcp_repl.send_file(file_path, ip, port);
            }
        }

        /// Handle receive file (rf) - receives a file from a given port and saves it to a destination file
        fn receive_file_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 2 {
                eprintln!("Usage: rf <dest file> <port>");
                return;
            }

            let dest_file_path = args[0].to_string();
            let port = args[1].parse().unwrap();

            if let Some(tcp_repl) = repl.as_tcp_repl() {
                tcp_repl.receive_file(dest_file_path, port);
            }
        }

        /// Handle close socket (cl) - initiates connection teardown by calling v_close
        fn close_socket_handler(repl: Arc<dyn Repl>, args: Vec<&str>) {
            if args.len() != 1 {
                eprintln!("Usage: cl <socket ID>");
                return;
            }

            let socket_id = args[0].parse().unwrap();

            if let Some(tcp_repl) = repl.as_tcp_repl() {
                tcp_repl.close_socket(socket_id);
            }
        }

        // Register the TCP REPL command handlers
        self.register_repl_handler("c", connect_handler);
        self.register_repl_handler("a", listen_accept_handler);
        self.register_repl_handler("ls", list_sockets_handler);
        self.register_repl_handler("cl", close_socket_handler);
        self.register_repl_handler("s", send_handler);
        self.register_repl_handler("r", receive_handler);
        self.register_repl_handler("sf", send_file_handler);
        self.register_repl_handler("rf", receive_file_handler);
    }

    /// Connect to a remote host with the given IP and port
    fn connect(&self, ip: Ipv4Addr, port: u16) {
        match self.v_connect(ip, port) {
            Ok(socket) => {
                println!(
                    "Connected to {} on port {}. Socket ID: {}",
                    ip, port, socket.id
                );
            }
            Err(e) => {
                eprintln!("Error connecting to remote host: {}", e);
            }
        }
    }

    /// Listens on a given port and accepts incoming connections
    fn listen_accept(&self, port: u16) {
        let listener_socket = self.v_listen(port);
        println!("Listening on port {}", port);

        // Loop to accept incoming connections
        let tcp_repl = TcpStack::to_arc(self);

        thread::spawn(move || loop {
            match listener_socket.v_accept() {
                Ok(socket) => {
                    let connection = tcp_repl.get_connection(socket.id).unwrap();
                    println!(
                        "Accepted connection from {} on port {}. Socket ID: {}",
                        connection.remote_address.ip(),
                        connection.remote_address.port(),
                        socket.id
                    );
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            };
        });
    }

    /// Lists all open sockets on the TCP stack
    fn list_sockets(&self) {
        let socket_table = self.socket_table();
        let socket_table = socket_table.lock().unwrap();

        println!("SID      LAddr LPort      RAddr RPort       Status");
        for (connection, socket) in socket_table.sockets.iter() {
            match &**socket {
                TcpSocket::Listener(socket) => {
                    println!(
                        "{}    {}      {}        -          -        {}",
                        socket.id,
                        socket.local_address.ip(),
                        socket.local_address.port(),
                        "LISTEN"
                    );
                }
                TcpSocket::Normal(socket) => {
                    println!(
                        "{}    {}      {}        {}        {}        {:?}",
                        socket.id,
                        connection.local_address.ip(),
                        connection.local_address.port(),
                        connection.remote_address.ip(),
                        connection.remote_address.port(),
                        socket.state()
                    );
                }
            }
        }
    }

    /// Closes a socket with the given ID
    fn close_socket(&self, socket_id: u32) {
        match self.get_socket_by_id(socket_id) {
            Some(socket) => match &*socket {
                TcpSocket::Listener(socket) => {
                    socket.v_close();
                }
                TcpSocket::Normal(socket) => {
                    socket.v_close();
                }
            },
            None => {
                eprintln!("Socket ID not found");
            }
        }
    }

    /// Sends bytes to a socket with the given ID
    fn send(&self, socket_id: u32, data: &[u8]) {
        match self.get_socket_by_id(socket_id) {
            Some(socket) => match &*socket {
                TcpSocket::Listener(_) => {
                    eprintln!("Cannot send with a socket_id that corresponds to a Listen Socket");
                }
                TcpSocket::Normal(socket) => match socket.v_write(data.to_vec()) {
                    Ok(bytes_sent) => {
                        println!("Sent {:?} bytes", bytes_sent);
                    }
                    Err(e) => {
                        eprintln!("Error writing data to send buffer: {}", e);
                    }
                },
            },
            None => {
                eprintln!("Socket ID not found");
            }
        }
    }

    /// Receives bytes from a socket with the given ID
    fn receive(&self, socket_id: u32, num_bytes: usize) {
        match self.get_socket_by_id(socket_id) {
            Some(socket) => match &*socket {
                TcpSocket::Listener(_) => {
                    eprintln!("Cannot read with a socket_id that corresponds to a Listen Socket");
                }
                TcpSocket::Normal(socket) => {
                    let mut read_buffer: Vec<u8> = vec![0; num_bytes];

                    match socket.v_read(&mut read_buffer) {
                        Ok(bytes_read) => {
                            println!(
                                "Read {} bytes: {}",
                                bytes_read,
                                String::from_utf8_lossy(&read_buffer)
                            );
                        }
                        Err(e) => {
                            eprintln!("Error reading data from receive buffer: {}", e);
                        }
                    }
                }
            },
            None => {
                eprintln!("Socket ID not found");
            }
        }
    }

    /// Sends a file to a given IP and port
    fn send_file(&self, file_path: String, ip: Ipv4Addr, port: u16) {
        let tcp_repl = TcpStack::to_arc(self);

        thread::spawn(move || {
            // Read the entire file into memory
            match fs::read(&file_path) {
                Ok(file_content) => {
                    // Create a connection to the remote IP and port
                    match tcp_repl.v_connect(ip, port) {
                        Ok(socket) => {
                            let total_bytes = file_content.len();

                            // Write the file content to the send buffer
                            match socket.v_write(file_content) {
                                Ok(bytes_sent) => {
                                    println!("Sent {}/{} bytes", bytes_sent, total_bytes);
                                }
                                Err(e) => {
                                    eprintln!("Error writing data to send buffer: {}", e);
                                }
                            }

                            // Close the socket after all data has been written
                            socket.v_close();
                        }
                        Err(e) => {
                            eprintln!("Error connecting to remote host: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                }
            }
        });
    }

    /// Receives a file from a given port and saves it to a destination file
    fn receive_file(&self, dest_file_path: String, port: u16) {
        let tcp_repl = TcpStack::to_arc(self);

        thread::spawn(move || {
            let listener_socket = tcp_repl.v_listen(port);
            println!("Listening on port {}", port);

            match listener_socket.v_accept() {
                Ok(socket) => {
                    let mut total_bytes_received = 0;
                    let file = fs::File::create(dest_file_path);

                    match file {
                        Ok(mut file) => {
                            loop {
                                let mut buffer = vec![0; 1024];

                                match socket.v_read(&mut buffer) {
                                    Ok(bytes_read) => match file.write_all(&buffer[..bytes_read]) {
                                        Ok(_) => total_bytes_received += bytes_read,
                                        Err(e) => {
                                            eprintln!("Error writing to file: {}", e);
                                            return;
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Error reading from socket: {}", e);
                                        break;
                                    }
                                }
                            }

                            socket.v_close();
                            println!("Received {} total bytes", total_bytes_received);
                        }
                        Err(e) => {
                            eprintln!("Error opening destination file {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        });
    }
}
