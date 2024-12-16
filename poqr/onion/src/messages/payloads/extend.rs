use ntru::ntru_key::NtruPublicKey;

pub struct ExtendPayload {
    /// A newly generated public onion key for the backwards direction of the circuit.
    pub public_key: NtruPublicKey,
}

impl ExtendPayload {
    /// Serialize an ExtendPayload into a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.public_key.to_be_bytes()
    }

    /// Deserialize an ExtendPayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> ExtendPayload {
        ExtendPayload {
            public_key: NtruPublicKey::from_be_bytes(buf),
        }
    }
}
