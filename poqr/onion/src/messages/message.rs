use super::payloads::{CreatedPayload, ExtendPayload, ExtendedPayload};

const MESSAGE_CREATE: u8 = 0;
const MESSAGE_CREATED: u8 = 1;
const MESSAGE_RELAY: u8 = 2;

pub enum Message {
    Create,
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
            Message::Create => {
                buf.push(MESSAGE_CREATE);
            }
            Message::Created(payload) => {
                buf.push(MESSAGE_CREATED);
                buf.extend_from_slice(&payload.serialize());
            }
            Message::Relay(payload) => match payload {
                RelayPayload::Extend(payload) => {
                    buf.push(MESSAGE_RELAY);
                    buf.push(PAYLOAD_EXTEND);
                    buf.extend_from_slice(&payload.serialize());
                }
                RelayPayload::Extended(payload) => {
                    buf.push(MESSAGE_RELAY);
                    buf.push(PAYLOAD_EXTENDED);
                    buf.extend_from_slice(&payload.serialize());
                } // RelayPayload::Begin(payload) => {
                  //     buf.push(MESSAGE_RELAY);
                  //     buf.push(PAYLOAD_BEGIN);
                  //     buf.extend_from_slice(&payload.serialize());
                  // }
                  // RelayPayload::Data(payload) => {
                  //     buf.push(MESSAGE_RELAY);
                  //     buf.push(PAYLOAD_DATA);
                  //     buf.extend_from_slice(&payload.serialize());
                  // }
            },
        }

        buf
    }

    pub fn deserialize(msg: Vec<u8>) -> Message {
        match msg[0] {
            MESSAGE_CREATE => Message::Create,
            MESSAGE_CREATED => {
                let payload = CreatedPayload::deserialize(&msg[1..]);
                Message::Created(payload)
            }
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
