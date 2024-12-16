use ntru::ntru_key::NtruPublicKey;

pub struct CreatedPayload {
    /// A newly generated public onion key of the node sending the CREATED message.
    public_key: NtruPublicKey,
}

impl CreatedPayload {
    /// Serialize a CreatedPayload into a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.public_key.to_be_bytes()
    }

    /// Deserialize a CreatedPayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> CreatedPayload {
        CreatedPayload {
            public_key: NtruPublicKey::from_be_bytes(buf),
        }
    }
}
