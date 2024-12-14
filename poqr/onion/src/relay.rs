use ntru::convolution_polynomial::ternary_polynomial;
use ntru::ntru::*;
use ntru::ntru_key::*;

pub struct Relay {
    /// Returns the signing key of the relay (in TOR protocol, this is an Ed25519 key).
    /// This key is kept online and is signed by the identity key
    signing_key: NtruKeyPair,
    /// Returns the circuit extension key (aka "onion key") of the relay (in TOR protocol, this is a curve25519 key).
    onion_key: NtruKeyPair,
}

impl Relay {
    pub fn new() {}
}
