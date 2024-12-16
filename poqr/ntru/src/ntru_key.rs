use crate::convolution_polynomial::{ternary_polynomial, ConvPoly};
use crate::ntru_util::{deserialize, serialize};
use crate::params::*;

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

    /// Encrypts a message using the NTRU encryption scheme
    pub fn encrypt(&self, msg: Vec<u8>) -> ConvPoly {
        assert!(msg.len() * 5 <= N, "encrypt: message too long");
        // ASCII message serialized as a balanced ternary polynomial
        let ser_msg = serialize(msg);
        // Compute r(x) as a random perturbation in T(d, d)
        let rand = ternary_polynomial(N, D, D);
        // Compute the encrypted message e(x) ≡ m(x) + p*r(x)*h(x)  (mod q)
        let p = ConvPoly::constant(P);
        let enc_msg = ser_msg.add(&p.mul(&rand.mul(&self.h, N), N)).modulo(Q);
        enc_msg
    }

    /// Serializes the public key into a byte vector
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.h.coeffs.len() * size_of::<i32>());
        for coeff in &self.h.coeffs {
            buf.extend_from_slice(&coeff.to_be_bytes());
        }
        buf
    }

    /// Deserializes a byte vector into an NTRU public key
    pub fn deserialize(buf: &[u8]) -> NtruPublicKey {
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

    /// Decrypts a message using the NTRU encryption scheme
    pub fn decrypt(&self, enc_msg: ConvPoly) -> Vec<u8> {
        // a(x) ≡ e(x) * f(x) (mod q)
        let a = enc_msg.mul(&self.f, N).center_lift(Q);
        // m(x) ≡ a(x) * Fp(x) (mod p)
        let msg_poly = a.mul(&self.f_p, N).modulo(P);
        let msg = deserialize(msg_poly);
        msg
    }

    /// Serializes the private key into a byte vector
    pub fn serialize(&self) -> Vec<u8> {
        // Allocate space for the four polynomials in the private key (and their lengths)
        let total_coeffs = self.f.coeffs.len()
            + self.f_p.coeffs.len()
            + self.f_q.coeffs.len()
            + self.g.coeffs.len();
        let mut buf =
            Vec::with_capacity((4 * size_of::<u32>()) + (total_coeffs * size_of::<i32>()));
        // Serialize the four polynomials in the private key
        let poly_list = [&self.f, &self.f_p, &self.f_q, &self.g];
        for poly in poly_list {
            buf.extend_from_slice(&(poly.coeffs.len() as u32).to_be_bytes());
            for coeff in &poly.coeffs {
                buf.extend_from_slice(&coeff.to_be_bytes());
            }
        }
        buf
    }

    /// Deserializes a byte vector into an NTRU private key
    pub fn deserialize(buf: &[u8]) -> NtruPrivateKey {
        let mut polys: Vec<ConvPoly> = Vec::new();
        let mut i = 0;
        let num_polys = 4;

        for _ in 0..num_polys {
            let len = u32::from_be_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]) as usize;
            let coeff_bytes = buf[i..(i + len * size_of::<i32>())].to_vec();
            polys.push(ConvPoly::deserialize(&coeff_bytes));
            i += size_of::<u32>() + len * size_of::<i32>();
        }

        let (f, f_p, f_q, g) = (
            polys[0].clone(),
            polys[1].clone(),
            polys[2].clone(),
            polys[3].clone(),
        );
        NtruPrivateKey { f, f_p, f_q, g }
    }
}
