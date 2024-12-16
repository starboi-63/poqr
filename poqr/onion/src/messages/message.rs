use super::payloads::{CreatePayload, CreatedPayload, ExtendPayload, ExtendedPayload};

const MESSAGE_CREATE: u8 = 0;
const MESSAGE_CREATED: u8 = 1;
const MESSAGE_RELAY: u8 = 2;

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
    // Begin(BeginPayload),
    // Data(DataPayload),
}

impl Message {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            Message::Create(payload) => {
                buf.push(MESSAGE_CREATE);
                buf.extend_from_slice(&payload.serialize());
            }
            Message::Created(payload) => {
                buf.push(MESSAGE_CREATED);
                buf.extend_from_slice(&payload.serialize());
            }
            Message::Relay(payload) => {
                buf.push(MESSAGE_RELAY);

                match payload {
                    RelayPayload::Extend(payload) => {
                        buf.push(PAYLOAD_EXTEND);
                        buf.extend_from_slice(&payload.serialize());
                    }
                    RelayPayload::Extended(payload) => {
                        buf.push(PAYLOAD_EXTENDED);
                        buf.extend_from_slice(&payload.serialize());
                    } // RelayPayload::Begin(payload) => {
                      //     buf.push(PAYLOAD_BEGIN);
                      //     buf.extend_from_slice(&payload.serialize());
                      // }
                      // RelayPayload::Data(payload) => {
                      //     buf.push(PAYLOAD_DATA);
                      //     buf.extend_from_slice(&payload.serialize());
                      // }
                }
            }
        }

        buf
    }

    pub fn deserialize(msg: Vec<u8>) -> Message {
        match msg[0] {
            MESSAGE_CREATE => Message::Create(CreatePayload::deserialize(&msg[1..])),
            MESSAGE_CREATED => Message::Created(CreatedPayload::deserialize(&msg[1..])),
            MESSAGE_RELAY => {
                match msg[1] {
                    PAYLOAD_EXTEND => {
                        let payload = ExtendPayload::deserialize(&msg[2..]);
                        Message::Relay(RelayPayload::Extend(payload))
                    }
                    PAYLOAD_EXTENDED => {
                        let payload = ExtendedPayload::deserialize(&msg[2..]);
                        Message::Relay(RelayPayload::Extended(payload))
                    }
                    // PAYLOAD_BEGIN => {
                    //     // deserialize BeginPayload
                    // }
                    // PAYLOAD_DATA => {
                    //     // deserialize DataPayload
                    // }
                    _ => panic!("Unknown payload type"),
                }
            }
            _ => panic!("Unknown message type"),
        }
    }
}
