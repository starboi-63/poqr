use ntru::ntru_key::NtruPublicKey;

pub struct ExtendedPayload {
    pub public_key: NtruPublicKey,
}

impl ExtendedPayload {
    /// Serialize an ExtendedPayload into a byte buffer.
    pub fn serialize(&self) -> Vec<u8> {
        self.public_key.to_be_bytes()
    }

    /// Deserialize an ExtendedPayload from a byte buffer.
    pub fn deserialize(buf: &[u8]) -> ExtendedPayload {
        ExtendedPayload {
            public_key: NtruPublicKey::from_be_bytes(buf),
        }
    }
}
