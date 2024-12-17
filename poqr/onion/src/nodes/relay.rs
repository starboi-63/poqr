use crate::{ChannelTable, Directory, Message, OnionPacket, RelayPayload};
use ntru::NtruKeyPair;
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct Relay {
    /// The unique ID of the relay
    pub id: u32,
    /// The port the relay listens on
    pub port: u16,
    /// An mpsc channel for sending packets to the relay's main listener thread
    pub packet_sender: Arc<mpsc::Sender<OnionPacket>>,
    /// An mpsc channel for receiving packets from the relay's onion channels
    pub packet_receiver: Arc<Mutex<mpsc::Receiver<OnionPacket>>>,
    /// A table mapping circuit IDs to channels
    pub channels: Arc<Mutex<ChannelTable>>,
    /// The NTRU key pair used to verify the relay's identity
    pub id_key: Arc<NtruKeyPair>,
    /// The public directory of relays
    pub directory: Arc<RwLock<Directory>>,
}

impl Relay {
    pub fn new(id: u32, port: u16, directory: Arc<RwLock<Directory>>) -> Relay {
        let (sender, receiver) = mpsc::channel();

        Relay {
            id,
            port,
            packet_sender: Arc::new(sender),
            packet_receiver: Arc::new(Mutex::new(receiver)),
            channels: Arc::new(Mutex::new(ChannelTable::new())),
            id_key: Arc::new(NtruKeyPair::new()),
            directory,
        }
    }

    pub fn start_listener(&self) {
        let relay = self.clone();

        std::thread::spawn(move || {
            let port = relay.port;
            let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

            loop {
                match listener.accept() {
                    Ok((_socket, addr)) => println!("new client: {addr:?}"),
                    Err(e) => println!("couldn't get client: {e:?}"),
                }
            }
        });
    }

    pub fn start_packet_handler(&self) {
        let relay = self.clone();

        std::thread::spawn(move || {
            let receiver = relay.packet_receiver.lock().unwrap();

            loop {
                let packet = receiver.recv().unwrap();
                relay.handle_packet(packet)
            }
        });
    }

    fn handle_packet(&self, packet: OnionPacket) {
        match packet.msg {
            Message::Relay(payload) => match payload {
                RelayPayload::Data(data) => {
                    println!("Received data: {:?}", data);
                    // Forward the data to the next relay
                }
                RelayPayload::Extend(extend_payload) => {
                    println!("Received EXTEND request");
                }
                RelayPayload::Extended(_) => {
                    println!("Received EXTENDED confirmation");
                }
                RelayPayload::Begin(_) => {
                    println!("Got begin payload");
                }
            },
            _ => (),
        }
    }
}
