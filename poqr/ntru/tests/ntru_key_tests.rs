#[cfg(test)]
mod ntru_key_tests {
    use ntru::ntru_key::NtruKeyPair;

    #[test]
    fn test_ntru_encrypt_decrypt() {
        let keypair = NtruKeyPair::new();
        let msg = "hello world".as_bytes().to_vec();
        let enc_msg = keypair.encrypt(msg.clone());
        let dec_msg = keypair.decrypt(enc_msg);
        assert_eq!(msg, dec_msg);
    }
}
