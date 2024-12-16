use crate::{from_be_bytes, to_be_bytes};
use rsa_ext::RsaPublicKey;

pub struct CreatedPayload {
    /// A newly generated public onion key of the node sending the CREATED message.
    public_key: RsaPublicKey,
}

impl CreatedPayload {
    /// Serialize a CreatedPayload into a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        to_be_bytes(self.public_key.clone())
    }

    /// Deserialize a CreatedPayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> CreatedPayload {
        CreatedPayload {
            public_key: from_be_bytes(buf),
        }
    }
}
