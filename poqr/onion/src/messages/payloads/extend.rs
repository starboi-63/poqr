pub struct ExtendPayload {
    pub next_hop: u16,
    pub encrypted_keypair: Vec<u8>,
}

impl ExtendPayload {
    /// Serialize an ExtendPayload into a byte buffer.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.next_hop.to_be_bytes());
        buf.extend_from_slice(&self.encrypted_keypair);
        buf
    }

    /// Deserialize an ExtendPayload from a byte buffer.
    pub fn deserialize(buf: &[u8]) -> ExtendPayload {
        let next_hop = u16::from_be_bytes([buf[0], buf[1]]);
        let encrypted_keypair = buf[2..].to_vec();

        ExtendPayload {
            next_hop,
            encrypted_keypair,
        }
    }
}
