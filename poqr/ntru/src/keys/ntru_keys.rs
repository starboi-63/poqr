use super::{KeyPair, PrivateKey, PublicKey};

/// An NTRU key pair
struct NtruKeyPair {
    /// The public key of the NTRU encryption scheme key pair
    pub public_key: NtruPublicKey,
    /// The private key of the NTRU encryption scheme key pair
    pub private_key: NtruPrivateKey,
}

/// A public key used in the NTRU encryption scheme
struct NtruPublicKey {
    h: Vec<u8>,
    p: Vec<u8>,
    q: Vec<u8>,
}

/// A private key used in the NTRU encryption scheme
struct NtruPrivateKey {
    f: Vec<u8>,
    g: Vec<u8>,
    inv_f: Vec<u8>,
}

impl PublicKey for NtruPublicKey {}
impl PrivateKey for NtruPrivateKey {}

impl KeyPair for NtruKeyPair {
    fn public_key(&self) -> PublicKey {
        self.public_key
    }

    fn private_key(&self) -> PrivateKey {
        self.private_key
    }
}
