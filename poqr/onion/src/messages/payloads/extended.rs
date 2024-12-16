use ntru::ntru_key::NtruPublicKey;

pub struct ExtendedPayload {
    /// A newly generated public onion key for the forwards direction of the circuit.
    pub public_key: NtruPublicKey,
}

impl ExtendedPayload {
    /// Serialize an ExtendedPayload into a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.public_key.to_be_bytes()
    }

    /// Deserialize an ExtendedPayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> ExtendedPayload {
        ExtendedPayload {
            public_key: NtruPublicKey::from_be_bytes(buf),
        }
    }
}
