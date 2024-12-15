use crate::convolution_polynomial::{ternary_polynomial, ConvPoly};
use crate::ntru_util::{deserialize, serialize};
use crate::params::*;

/// An NTRU key pair
pub struct NtruKeyPair {
    /// The public key of the NTRU encryption scheme key pair
    public_key: NtruPublicKey,
    /// The private key of the NTRU encryption scheme key pair
    private_key: NtruPrivateKey,
}
impl NtruKeyPair {
    /// Generates a new public/private NTRU key pair
    pub fn new() -> NtruKeyPair {
        let k_priv = NtruPrivateKey::new();
        let k_pub = NtruPublicKey::new(&k_priv);
        NtruKeyPair {
            public_key: k_pub,
            private_key: k_priv,
        }
    }

    /// Encrypts a message using the NTRU encryption scheme
    pub fn encrypt(&self, msg: Vec<u8>) -> ConvPoly {
        assert!(msg.len() * 5 <= N, "encrypt: message too long");
        // ASCII message serialized as a balanced ternary polynomial
        let ser_msg = serialize(msg);
        // Compute r(x) as a random perturbation in T(d, d)
        let rand = ternary_polynomial(N, D, D);
        // Compute the encrypted message e(x) ≡ m(x) + p*r(x)*h(x)  (mod q)
        let p = ConvPoly::constant(P, N);
        let enc_msg = ser_msg.add(&p.mul(&rand.mul(&self.public_key.h))).modulo(Q);
        enc_msg
    }

    /// Decrypts a message using the NTRU encryption scheme
    pub fn decrypt(&self, enc_msg: ConvPoly) -> Vec<u8> {
        // a(x) ≡ e(x) * f(x) (mod q)
        let a = enc_msg.mul(&self.private_key.f).center_lift(Q);
        // m(x) ≡ a(x) * Fp(x) (mod p)
        let msg_poly = a.mul(&self.private_key.f_p).modulo(P);
        let msg = deserialize(msg_poly);
        msg
    }
}

/// A public key used in the NTRU encryption scheme
struct NtruPublicKey {
    h: ConvPoly,
}
impl NtruPublicKey {
    /// Generates a public key given a corresponding private key
    fn new(k_priv: &NtruPrivateKey) -> NtruPublicKey {
        // Generate f inverse over Q
        let f_inv = &k_priv.f_q;
        // Public key generated as f inverse Q * g
        let h = f_inv.mul(&k_priv.g);
        NtruPublicKey { h }
    }
}

/// A private key used in the NTRU encryption scheme
struct NtruPrivateKey {
    f: ConvPoly,
    f_p: ConvPoly,
    f_q: ConvPoly,
    g: ConvPoly,
}
impl NtruPrivateKey {
    /// Generates a new random NTRU private key
    fn new() -> NtruPrivateKey {
        loop {
            let f = ternary_polynomial(N, D + 1, D);
            let f_q = f.inverse(Q);
            let f_p = f.inverse(P);
            match (f_p, f_q) {
                (Ok(f_p), Ok(f_q)) => {
                    let g = ternary_polynomial(N, D, D);
                    return NtruPrivateKey {f, f_p, f_q, g}
                } 
                _ => continue
            }
        }
    }
}
