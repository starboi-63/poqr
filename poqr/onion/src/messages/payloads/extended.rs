use crate::{from_be_bytes, to_be_bytes};
use rsa_ext::RsaPublicKey;

pub struct ExtendedPayload {
    /// A newly generated public onion key for the forwards direction of the circuit.
    pub public_key: RsaPublicKey,
}

impl ExtendedPayload {
    /// Serialize an ExtendedPayload into a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        to_be_bytes(self.public_key.clone())
    }

    /// Deserialize an ExtendedPayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> ExtendedPayload {
        ExtendedPayload {
            public_key: from_be_bytes(buf),
        }
    }
}
