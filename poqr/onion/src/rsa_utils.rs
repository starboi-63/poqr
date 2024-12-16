use rsa_ext::{BigUint, PublicKeyParts, RsaPublicKey};

/// Serialize the RsaPublicKey to a big-endian byte array.
pub fn to_be_bytes(rsa_pub_key: RsaPublicKey) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&rsa_pub_key.n().to_bytes_be());
    buf.extend_from_slice(&rsa_pub_key.e().to_bytes_be());
    buf
}

/// Deserialize the RsaPublicKey from a big-endian byte array.
pub fn from_be_bytes(buf: &[u8]) -> RsaPublicKey {
    let n = BigUint::from_bytes_be(&buf[..128]);
    let e = BigUint::from_bytes_be(&buf[128..]);
    RsaPublicKey::new(n, e).unwrap()
}
