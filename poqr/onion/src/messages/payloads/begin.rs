pub struct BeginPayload {
    status: bool,
}

impl BeginPayload {
    /// Serialize a CreatedPayload into a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        vec![self.status as u8]
    }

    /// Deserialize a CreatedPayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> BeginPayload {
        BeginPayload {
            status: buf[0] != 0,
        }
    }
}
