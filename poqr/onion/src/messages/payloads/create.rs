use ntru::ntru_key::NtruPublicKey;

pub struct CreatePayload {
    /// A newly generated public onion key for the backwards direction of the circuit.
    pub public_key: NtruPublicKey,
}

impl CreatePayload {
    /// Serialize the CreatePayload to a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.public_key.to_be_bytes()
    }

    /// Deserialize the CreatePayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> CreatePayload {
        CreatePayload {
            public_key: NtruPublicKey::from_be_bytes(buf),
        }
    }
}
