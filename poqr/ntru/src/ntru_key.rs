use crate::convolution_polynomial::{ternary_polynomial, ConvPoly};
use crate::params::*;
use crate::ntru_key::*;

/// An NTRU key pair
pub struct NtruKeyPair {
    /// The public key of the NTRU encryption scheme key pair
    pub public_key: NtruPublicKey,
    /// The private key of the NTRU encryption scheme key pair
    pub private_key: NtruPrivateKey,
}
impl NtruKeyPair {
    pub fn gen_keypair() -> NtruKeyPair {}
}

/// A public key used in the NTRU encryption scheme
pub struct NtruPublicKey {
    h: ConvPoly,
}
impl NtruPublicKey {}

/// A private key used in the NTRU encryption scheme
pub struct NtruPrivateKey {
    f: ConvPoly,
    g: ConvPoly,
}
impl NtruPrivateKey {
    fn new() -> NtruPrivateKey {
        let f: ConvPoly = {
             
        } 
    }
}
