use ntru::convolution_polynomial::ConvPoly;
use ntru::ntru_key::{NtruPrivateKey, NtruPublicKey};
use rsa_ext::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

use super::payloads::{
    BeginPayload, CreatePayload, CreatedPayload, DataPayload, ExtendPayload, ExtendedPayload,
};

/// A packet sent over the POQR network
pub struct OnionPacket {
    pub header: OnionHeader,
    pub msg: Message,
}

impl OnionPacket {
    /// Serialize an OnionPacket into a big-endian byte array.
    pub fn to_be_bytes(&self, id_key: NtruPublicKey, onion_keys: Vec<RsaPublicKey>) -> Vec<u8> {
        let mut buf = Vec::new();

        let msg_bytes = self.msg.to_be_bytes(id_key, onion_keys);
        let msg_len: u32 = msg_bytes.len() as u32;

        buf.extend_from_slice(&self.header.circ_id.to_be_bytes());
        buf.extend_from_slice(&msg_len.to_be_bytes());
        buf.extend_from_slice(&msg_bytes);
        buf
    }

    /// Deserialize an OnionPacket from a big-endian byte array.
    pub fn from_be_bytes(
        buf: &[u8],
        id_key: NtruPrivateKey,
        onion_keys: Vec<RsaPrivateKey>,
    ) -> OnionPacket {
        let header = OnionHeader {
            circ_id: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
        };
        let msg_len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
        let msg = Message::from_be_bytes(buf[8..8 + msg_len].to_vec(), id_key, onion_keys);
        OnionPacket { header, msg }
    }
}

/// Packet header, contains metadata about the packet
/// Unimplemented: Certificates are usually kept here, but left out for our implementation
pub struct OnionHeader {
    pub circ_id: u32,
}

const MESSAGE_CREATE: u8 = 0;
const MESSAGE_CREATED: u8 = 1;
const MESSAGE_RELAY: u8 = 2;

/// An enum representing the types of messages that can be sent on the POQR network
/// All messages except for Create/Created contain a Relay
pub enum Message {
    Create(CreatePayload),
    Created(CreatedPayload),
    Relay(RelayPayload),
}

const PAYLOAD_CREATE: u8 = 0;
const PAYLOAD_CREATED: u8 = 1;
const PAYLOAD_EXTEND: u8 = 2;
const PAYLOAD_EXTENDED: u8 = 3;
const PAYLOAD_BEGIN: u8 = 4;
const PAYLOAD_DATA: u8 = 5;
const PAYLOAD_END: u8 = 6;

/// This enum represents the different types of payloads that can be sent in a relay message,
/// and is encrypted onion-style.
pub enum RelayPayload {
    Extend(ExtendPayload),
    Extended(ExtendedPayload),
    Begin(BeginPayload),
    Data(DataPayload),
}

impl Message {
    /// Adds a layer of NTRU encryption to a Vec<u8> using a valid NTRU public key then serializes it to a new byte vector
    fn add_quantum_onion_skin(bytes: &[u8], id_key: NtruPublicKey) -> Vec<u8> {
        let poly = id_key.encrypt_bytes(bytes.to_vec());
        poly.to_be_bytes()
    }

    /// Deserializes a serialized NTRU encrypted message, unencrypts it, then reserializes it to a vector of bytes.
    fn remove_quantum_onion_skin(bytes: &[u8], id_key: NtruPrivateKey) -> Vec<u8> {
        let poly = ConvPoly::from_be_bytes(&bytes.to_vec());
        id_key.decrypt_to_bytes(poly)
    }

    fn add_onion_skin(bytes: &[u8], onion_keys: Vec<RsaPublicKey>) -> Vec<u8> {
        if onion_keys.is_empty() {
            // No onion keys, return the original bytes
            bytes.to_vec()
        } else {
            let padding = PaddingScheme::new_pkcs1v15_encrypt();
            let mut rng = rand::thread_rng();
            // Encrypt the message with the first onion key
            let mut enc = onion_keys[0].encrypt(&mut rng, padding, bytes).unwrap();
            // Encrypt the message with the rest of the onion keys
            for i in 1..onion_keys.len() {
                let padding = PaddingScheme::new_pkcs1v15_encrypt();
                enc = onion_keys[i].encrypt(&mut rng, padding, &enc).unwrap();
            }
            enc
        }
    }

    fn remove_onion_skin(bytes: &[u8], onion_keys: Vec<RsaPrivateKey>) -> Vec<u8> {
        if onion_keys.is_empty() {
            // No onion keys, return the original bytes
            bytes.to_vec()
        } else {
            let padding = PaddingScheme::new_pkcs1v15_encrypt();
            // Decrypt the message with the last onion key
            let mut dec = onion_keys[0].decrypt(padding, bytes).unwrap();
            // Decrypt the message with the rest of the onion keys
            for i in 1..onion_keys.len() {
                let padding = PaddingScheme::new_pkcs1v15_encrypt();
                dec = onion_keys[i].decrypt(padding, &dec).unwrap();
            }
            dec
        }
    }

    pub fn to_be_bytes(&self, id_key: NtruPublicKey, onion_keys: Vec<RsaPublicKey>) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            Message::Create(payload) => {
                buf.push(MESSAGE_CREATE);
                buf.extend_from_slice(&payload.to_be_bytes());
            }
            Message::Created(payload) => {
                buf.push(MESSAGE_CREATED);
                buf.extend_from_slice(&payload.to_be_bytes());
            }
            Message::Relay(payload) => {
                buf.push(MESSAGE_RELAY);

                match payload {
                    RelayPayload::Extend(payload) => {
                        buf.push(PAYLOAD_EXTEND);
                        let onion = Message::add_onion_skin(&payload.to_be_bytes(), onion_keys);
                        buf.extend_from_slice(&onion);
                    }
                    RelayPayload::Extended(payload) => {
                        buf.push(PAYLOAD_EXTENDED);
                        let onion = Message::add_onion_skin(&payload.to_be_bytes(), onion_keys);
                        buf.extend_from_slice(&onion);
                    }
                    RelayPayload::Begin(payload) => {
                        buf.push(PAYLOAD_BEGIN);
                        let onion = Message::add_onion_skin(&payload.to_be_bytes(), onion_keys);
                        buf.extend_from_slice(&onion);
                    }
                    RelayPayload::Data(payload) => {
                        buf.push(PAYLOAD_DATA);
                        let onion = Message::add_onion_skin(&payload.to_be_bytes(), onion_keys);
                        buf.extend_from_slice(&onion);
                    }
                }
            }
        }
        Message::add_quantum_onion_skin(&buf, id_key)
    }

    pub fn from_be_bytes(
        msg: Vec<u8>,
        id_key: NtruPrivateKey,
        onion_keys: Vec<RsaPrivateKey>,
    ) -> Message {
        let msg = Message::remove_quantum_onion_skin(&msg, id_key);

        match msg[0] {
            MESSAGE_CREATE => Message::Create(CreatePayload::from_be_bytes(&msg[1..])),
            MESSAGE_CREATED => Message::Created(CreatedPayload::from_be_bytes(&msg[1..])),
            MESSAGE_RELAY => match msg[1] {
                PAYLOAD_EXTEND => {
                    let payload_bytes = Message::remove_onion_skin(&msg[2..], onion_keys);
                    let payload = ExtendPayload::from_be_bytes(&payload_bytes);
                    Message::Relay(RelayPayload::Extend(payload))
                }
                PAYLOAD_EXTENDED => {
                    let payload_bytes = Message::remove_onion_skin(&msg[2..], onion_keys);
                    let payload = ExtendedPayload::from_be_bytes(&payload_bytes);
                    Message::Relay(RelayPayload::Extended(payload))
                }
                PAYLOAD_BEGIN => {
                    let payload_bytes = Message::remove_onion_skin(&msg[2..], onion_keys);
                    let payload = BeginPayload::from_be_bytes(&payload_bytes);
                    Message::Relay(RelayPayload::Begin(payload))
                }
                PAYLOAD_DATA => {
                    let payload_bytes = Message::remove_onion_skin(&msg[2..], onion_keys);
                    let payload = DataPayload::from_be_bytes(&payload_bytes);
                    Message::Relay(RelayPayload::Data(payload))
                }
                _ => panic!("Unknown payload type"),
            },
            _ => panic!("Unknown message type"),
        }
    }
}
