pub struct CreatePayload {
    /// The ID of the circuit being created (only valid for one hop in the circuit).
    pub circuit_id: u32,
}

impl CreatePayload {
    /// Serialize the payload to a byte array.
    pub fn serialize(&self) -> Vec<u8> {
        self.circuit_id.to_be_bytes().to_vec()
    }

    /// Deserialize the payload from a byte array.
    pub fn deserialize(buf: &[u8]) -> CreatePayload {
        CreatePayload {
            circuit_id: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
        }
    }
}
