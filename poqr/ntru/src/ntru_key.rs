use crate::convolution_polynomial::{ternary_polynomial, ConvPoly};
use crate::ntru_util::{deserialize, serialize};
use crate::params::*;

#[derive(Clone)]
/// An NTRU key pair
pub struct NtruKeyPair {
    /// The public key of the NTRU encryption scheme key pair
    pub public: NtruPublicKey,
    /// The private key of the NTRU encryption scheme key pair
    pub private: NtruPrivateKey,
}

impl NtruKeyPair {
    /// Generates a new public/private NTRU key pair
    pub fn new() -> NtruKeyPair {
        let k_priv = NtruPrivateKey::new();
        let k_pub = NtruPublicKey::new(&k_priv);
        NtruKeyPair {
            public: k_pub,
            private: k_priv,
        }
    }
}

#[derive(Clone)]
/// A public key used in the NTRU encryption scheme
pub struct NtruPublicKey {
    h: ConvPoly,
}

impl NtruPublicKey {
    /// Generates a public key given a corresponding private key
    fn new(k_priv: &NtruPrivateKey) -> NtruPublicKey {
        // Generate f inverse over Q
        let f_inv = &k_priv.f_q;
        // Public key generated as f inverse Q * g
        let h = f_inv.mul(&k_priv.g, N);
        NtruPublicKey { h }
    }

    /// Encrypts a convolution polynomial represented message using the NTRU encryption scheme.
    /// Used for successive layers of encryption after a message has already been serialized.
    pub fn encrypt_poly(&self, msg: ConvPoly) -> ConvPoly {
        // Compute r(x) as a random perturbation in T(d, d)
        let rand = ternary_polynomial(N, D, D);
        // Compute the encrypted message e(x) ≡ m(x) + p*r(x)*h(x)  (mod q)
        let p = ConvPoly::constant(P);
        let enc_msg = msg.add(&p.mul(&rand.mul(&self.h, N), N)).modulo(Q);
        enc_msg
    }

    /// Encrypts an ASCII byte vector of a message using the NTRU encryption scheme
    /// Should be used as a first layer of encryption since it serializes the message.
    pub fn encrypt_bytes(&self, msg: Vec<u8>) -> ConvPoly {
        self.encrypt_poly(serialize(msg))
    }

    /// Serializes the public key into a byte vector
    pub fn to_be_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.h.coeffs.len() * size_of::<i32>());
        for coeff in &self.h.coeffs {
            buf.extend_from_slice(&coeff.to_be_bytes());
        }
        buf
    }

    /// Deserializes a byte vector into an NTRU public key
    pub fn from_be_bytes(buf: &[u8]) -> NtruPublicKey {
        let mut coeffs = Vec::with_capacity(buf.len() / size_of::<i32>());
        for chunk in buf.chunks(size_of::<i32>()) {
            let mut bytes = [0; size_of::<i32>()];
            bytes.copy_from_slice(chunk);
            coeffs.push(i32::from_be_bytes(bytes));
        }
        NtruPublicKey {
            h: ConvPoly { coeffs },
        }
    }
}

#[derive(Clone)]
/// A private key used in the NTRU encryption scheme
pub struct NtruPrivateKey {
    /// A random polynomial generated over T(D+1, D)
    f: ConvPoly,
    /// The inverse of f(x) modulo P within the ring (Z/PZ)\[x\]/(x^N - 1)
    f_p: ConvPoly,
    /// The inverse of f(x) modulo Q within the ring (Z/QZ)\[x\]/(x^N - 1)
    f_q: ConvPoly,
    /// A random polynomial generated over T(D, D)
    g: ConvPoly,
}

impl NtruPrivateKey {
    /// Generates a new random NTRU private key
    fn new() -> NtruPrivateKey {
        loop {
            let f = ternary_polynomial(N, D + 1, D);
            let f_p = f.inverse(P, N);
            let f_q = f.inverse(Q, N);
            match (f_p, f_q) {
                (Ok(f_p), Ok(f_q)) => {
                    let g = ternary_polynomial(N, D, D);
                    return NtruPrivateKey { f, f_p, f_q, g };
                }
                _ => continue,
            }
        }
    }

    /// Decrypts a polynomial-encoded message using the NTRU encryption scheme into a byte vector
    /// ONLY FUNCTIONAL ON SINGLE LAYER ENCRYPTION ; MULTIPLE LAYERS WILL BREAK!
    pub fn decrypt_to_bytes(&self, enc_msg: ConvPoly) -> Vec<u8> {
        deserialize(self.decrypt_to_poly(enc_msg))
    }

    /// Decrypts a polynomial-encoded message using the NTRU encryption scheme into another polynomial
    /// ONLY FUNCTIONAL ON MULTI-LAYERED ENCRYPTION : FINAL LAYER WILL BREAK!
    pub fn decrypt_to_poly(&self, enc_msg: ConvPoly) -> ConvPoly {
        // a(x) ≡ e(x) * f(x) (mod q)
        let a = enc_msg.mul(&self.f, N).center_lift(Q);
        // m(x) ≡ a(x) * Fp(x) (mod p)
        let msg_poly = a.mul(&self.f_p, N).modulo(P);
        msg_poly
    }
}
