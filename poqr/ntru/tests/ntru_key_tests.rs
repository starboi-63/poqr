#[cfg(test)]
mod ntru_key_tests {
    use std::time::Instant;

    use ntru::ntru_key::NtruKeyPair;

    #[test]
    fn test_ntru_encrypt_decrypt() {
        // "Hello World" message
        let keypair = NtruKeyPair::new();
        let msg = "Hello World".as_bytes().to_vec();
        println!("Message: {:?}", msg);
        let enc_msg = keypair.encrypt(msg.clone());
        let dec_msg = keypair.decrypt(enc_msg);
        println!("Decrypted message: {:?}", dec_msg);
        assert_eq!(msg, dec_msg);

        // Test empty message

        // Test random messages
        // let num_tests = 1;
        // let mut rng = rand::thread_rng();

        // for _ in 0..num_tests {
        //     let msg_len = rng.gen_range(0..=90);
        //     let keypair = NtruKeyPair::new();
        //     let msg: Vec<u8> = (0..=msg_len).map(|_| rng.gen_range(1..=127)).collect();
        //     let enc_msg = keypair.encrypt(msg.clone());
        //     let dec_msg = keypair.decrypt(enc_msg);
        //     assert_eq!(msg, dec_msg);
        // }
    }
}
