use crate::{
    BeginPayload, ChannelTable, CreatePayload, CreatedPayload, Directory, ExtendPayload,
    ExtendedPayload, Message, OnionPacket, RelayPayload,
};
use ntru::NtruKeyPair;
use std::net::TcpListener;
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
            Message::Create(create_payload) => {
                println!("Received CREATE request");
                self.handle_create(create_payload);
            }
            Message::Created(created_payload) => {
                println!("Received CREATED confirmation");
                self.handle_created(created_payload);
            }
            Message::Relay(payload) => match payload {
                RelayPayload::Data(data) => {
                    println!("Received data: {:?}", data);
                    self.handle_data(data);
                }
                RelayPayload::Extend(extend_payload) => {
                    println!("Received EXTEND request");
                    self.handle_extend(extend_payload);
                }
                RelayPayload::Extended(extended_payload) => {
                    println!("Received EXTENDED confirmation");
                    self.handle_extended(extended_payload);
                }
                RelayPayload::Begin(begin_payload) => {
                    println!("Got begin payload");
                    self.handle_begin(begin_payload);
                }
            },
            _ => (),
        }
    }

    fn handle_create(&self, payload: CreatePayload) {
        // Get the channel for the circuit
        let mut channels = self.channels.lock().unwrap();
        let channel = channels.get_mut(&payload.circuit_id).unwrap();
        // Add the backward onion key to the channel
        let mut backward_onion_keys = channel.backward_onion_keys.lock().unwrap();
        backward_onion_keys.push(payload.public_key);
    }

    fn handle_created(&self, payload: CreatedPayload) {
        // Get the channel for the circuit
        let mut channels = self.channels.lock().unwrap();
        let channel = channels.get_mut(&payload.circuit_id).unwrap();
        // Add the forward onion key to the channel
        let mut forward_onion_keys = channel.forward_onion_keys.lock().unwrap();
        forward_onion_keys.push(payload.public_key);
    }

    //TODO: IMPLEMENT HANDLING EXTENDS AND SENDING BACK EXTENDED
    fn handle_extend(&self, payload: ExtendPayload) {
        eprintln!("This would be implemented if we had more time!");
        todo!();
    }
    //TODO: SCRAPPED DUE TO TIMEFRAME
    fn handle_extended(&self, payload: ExtendedPayload) {
        eprintln!("This would be implemented if we had more time!");
        todo!()
    }
    //TODO: SCRAPPED DUE TO TIMEFRAME
    fn handle_begin(&self, payload: BeginPayload) {
        eprintln!("This would be implemented if we had more time!");
        todo!()
    }
    //TODO: SCRAPPED DUE TO TIMEFRAME
    fn handle_data(&self, data: Vec<u8>) {
        eprintln!("This would be implemented if we had more time!");
        todo!()
    }
}
