use crate::convolution_polynomial::ConvPoly;

/// An NTRU key pair
pub struct NtruKeyPair {
    /// The public key of the NTRU encryption scheme key pair
    pub public_key: NtruPublicKey,
    /// The private key of the NTRU encryption scheme key pair
    pub private_key: NtruPrivateKey,
}

/// A public key used in the NTRU encryption scheme
pub struct NtruPublicKey {
    h: ConvPoly,
}

/// A private key used in the NTRU encryption scheme
pub struct NtruPrivateKey {
    f: ConvPoly,
    g: ConvPoly,
}
