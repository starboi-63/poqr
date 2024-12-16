use ntru::ntru_key::NtruPublicKey;

pub struct CreatedPayload {
    public_key: NtruPublicKey,
}

impl CreatedPayload {
    /// Serialize a CreatedPayload into a byte buffer.
    pub fn serialize(&self) -> Vec<u8> {
        self.public_key.serialize()
    }

    /// Deserialize a CreatedPayload from a byte buffer.
    pub fn deserialize(buf: &[u8]) -> CreatedPayload {
        CreatedPayload {
            public_key: NtruPublicKey::deserialize(buf),
        }
    }
}
