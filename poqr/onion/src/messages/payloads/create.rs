use crate::{from_be_bytes, to_be_bytes};
use rsa_ext::RsaPublicKey;

pub struct CreatePayload {
    /// A newly generated public onion key for the backwards direction of the circuit.
    pub public_key: RsaPublicKey,
}

impl CreatePayload {
    /// Serialize the CreatePayload to a big-endian byte array.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        to_be_bytes(self.public_key.clone())
    }

    /// Deserialize the CreatePayload from a big-endian byte array.
    pub fn from_be_bytes(buf: &[u8]) -> CreatePayload {
        CreatePayload {
            public_key: from_be_bytes(buf),
        }
    }
}
