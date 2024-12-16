#[cfg(test)]
mod ntru_key_tests {
    use ntru::ntru_key::NtruKeyPair;
    use rand::Rng;

    #[test]
    fn test_ntru_encrypt_decrypt() {
        // "Hello World" message
        let keypair = NtruKeyPair::new();
        let msg = "Hello World".as_bytes().to_vec();
        println!("Message: {:?}", msg);
        let enc_msg = keypair.public.encrypt_bytes(msg.clone());
        let dec_msg = keypair.private.decrypt_to_bytes(enc_msg);
        println!("Decrypted message: {:?}", dec_msg);
        assert_eq!(msg, dec_msg, "Hello World failed");

        // Test empty message
        let keypair = NtruKeyPair::new();
        let msg = vec![];
        let enc_msg = keypair.public.encrypt_bytes(msg.clone());
        let dec_msg = keypair.private.decrypt_to_bytes(enc_msg);
        assert_eq!(msg, dec_msg, "Empty message failed");

        // Test random messages with new key pairs
        let num_tests = 100;
        let mut rng = rand::thread_rng();

        for _ in 0..num_tests {
            let msg_len = rng.gen_range(0..100);
            let msg: Vec<u8> = (0..=msg_len).map(|_| rng.gen_range(1..=127)).collect();
            let keypair = NtruKeyPair::new();
            let enc_msg = keypair.public.encrypt_bytes(msg.clone());
            let dec_msg = keypair.private.decrypt_to_bytes(enc_msg);
            assert_eq!(msg, dec_msg, "Random message failed");
        }
    }
}
