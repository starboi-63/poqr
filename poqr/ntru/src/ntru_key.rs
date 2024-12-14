use crate::convolution_polynomial::{ternary_polynomial, ConvPoly};
use crate::ntru_key::*;
use crate::params::*;

/// An NTRU key pair
pub struct NtruKeyPair {
    /// The public key of the NTRU encryption scheme key pair
    pub public_key: NtruPublicKey,
    /// The private key of the NTRU encryption scheme key pair
    pub private_key: NtruPrivateKey,
}
impl NtruKeyPair {
    pub fn new() -> NtruKeyPair {
        let k_priv = NtruPrivateKey::new();
        let k_pub = NtruPublickKey::new(k_priv);
        NtruKeyPair {
            public_key: k_pub,
            private_key: k_priv,
        }
    }
}

/// A public key used in the NTRU encryption scheme
pub struct NtruPublicKey {
    h: ConvPoly,
}
impl NtruPublicKey {
    fn new(k_priv: NtruPrivateKey) -> NtruPublicKey {
        let f_inv = k_priv.f.inverse(Q).unwrap();
        let h = f_inv.mul(&k_priv.g);
        NtruPublicKey { h }
    }
}

/// A private key used in the NTRU encryption scheme
pub struct NtruPrivateKey {
    f: ConvPoly,
    g: ConvPoly,
}
impl NtruPrivateKey {
    fn new() -> NtruPrivateKey {
        let f: ConvPoly = {
            let mut f: ConvPoly = ternary_polynomial(N, D + 1, D);
            let mut valid_poly: bool = false;
            while !valid_poly {
                if f.inverse(Q).is_ok() && f.inverse(P).is_ok() {
                    valid_poly = true;
                }
                f = ternary_polynomial(N, D + 1, D)
            }
            f
        };
        let g: ConvPoly = ternary_polynomial(N, D, D);
        NtruPrivateKey { f, g }
    }
}
