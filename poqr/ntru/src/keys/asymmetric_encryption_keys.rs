/// Trait for keys used in an asymmetric encryption scheme
pub trait KeyPair {
    /// Returns the public key of the key pair
    fn public_key(&self) -> dyn PublicKey;
    /// Returns the private key of the key pair
    fn private_key(&self) -> dyn PrivateKey;
}

/// A public key used in an asymmetric encryption scheme
pub trait PublicKey {}

/// A private key used in an asymmetric encryption scheme
pub trait PrivateKey {}
