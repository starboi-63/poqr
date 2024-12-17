#[derive(Debug)]
pub struct DataPayload {
    /// A newly generated public onion key of the node sending the CREATED message.
    data: Vec<u8>,
}

impl DataPayload {
    /// Serialize a CreatedPayload into a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Deserialize a CreatedPayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> DataPayload {
        DataPayload { data: buf.to_vec() }
    }
}
