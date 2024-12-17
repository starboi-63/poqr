use ntru::convolution_polynomial::ConvPoly;
use ntru::ntru_key::{NtruKeyPair, NtruPublicKey};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::Arc;

fn run_server() {
    let keypair = Arc::new(NtruKeyPair::new());
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind to port 7878");

    println!("Server listening on 127.0.0.1:7878");

    for stream in listener.incoming() {
        let keypair = Arc::clone(&keypair);
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream, keypair);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

fn handle_client(mut stream: TcpStream, keypair: Arc<NtruKeyPair>) {
    println!("New client connected. Initiating handshake...");

    // Send the public key to the client
    let public_key_bytes = keypair.public.to_be_bytes();
    stream
        .write_all(&public_key_bytes)
        .expect("Failed to send public key to client");

    println!("Public key sent to client.");

    // Now, receive and decrypt messages from the client
    let mut buffer = vec![0; 4096];
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected.");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from client: {}", e);
                break;
            }
        };

        let enc_poly = ConvPoly::from_be_bytes(&buffer[..bytes_read].to_vec());
        let decrypted = keypair.private.decrypt_to_bytes(enc_poly);
        let message = String::from_utf8_lossy(&decrypted);

        println!("Received and decrypted message: {}", message);
    }
}

fn run_client() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Failed to connect to server");

    // Receive the public key from the server
    let mut buffer = vec![0; 4096];
    let bytes_read = stream
        .read(&mut buffer)
        .expect("Failed to read public key from server");

    let public_key = NtruPublicKey::from_be_bytes(&buffer[..bytes_read].to_vec());
    println!("Received public key from server. You can now send messages.");

    // Loop to send messages
    loop {
        print!("Enter message: ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let message = input.trim();
        if message == "exit" {
            println!("Exiting client.");
            break;
        }

        if message.len() > 100 {
            println!("Message too long. Please keep it under 100 bytes.");
            continue;
        }

        let message_bytes = message.as_bytes().to_vec();
        let enc_poly = public_key.encrypt_bytes(message_bytes);
        let enc_bytes = enc_poly.to_be_bytes();

        stream
            .write_all(&enc_bytes)
            .expect("Failed to send message");

        println!("Sent encrypted message.");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [server|client]", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "server" => run_server(),
        "client" => run_client(),
        _ => {
            eprintln!("Invalid argument. Use 'server' or 'client'.");
            std::process::exit(1);
        }
    }
}
