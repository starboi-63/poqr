use network::network_stacks::IpStack;

pub trait Relay: IpStack {
    /// Returns the identity key of the relay (in TOR protocol, this is an Ed25519 key).
    /// This key is kept offline
    fn identity_key(&self) -> KeyPair;
    /// Returns the signing key of the relay (in TOR protocol, this is an Ed25519 key).
    /// This key is kept online and is signed by the identity key
    fn signing_key(&self) -> KeyPair;
    /// Returns the circuit extension key (aka "onion key") of the relay (in TOR protocol, this is a curve25519 key).
    fn onion_key(&self) -> KeyPair;
}
