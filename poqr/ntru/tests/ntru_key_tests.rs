#[cfg(test)]
mod ntru_key_tests {
    use ntru::{convolution_polynomial::ternary_polynomial, ntru_key::NtruKeyPair, ConvPoly};
    use rand::Rng; 

    #[test]
    fn test_bytecode() {
        let n = rand::thread_rng().gen_range(1..=15);
        let num_ones = rand::thread_rng().gen_range(0..=n);
        let num_neg_ones = rand::thread_rng().gen_range(0..=(n - num_ones));
        let poly = ternary_polynomial(n, num_ones, num_neg_ones);

        let enc_poly = poly.to_be_bytes();
        assert!(ConvPoly::from_be_bytes(&enc_poly) == poly, "Failed encoding polynomial")
    }

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

        // Test to bytes and out of bytes encrypt
        let keypair = NtruKeyPair::new();
        let msg = "helloworld".as_bytes().to_vec();
        println!("Message 3: {:?}", msg);
        let enc_msg = keypair.public.encrypt_bytes(msg.clone());
        let enc_msg_bytes = enc_msg.to_be_bytes();
        let enc_msg_debyted = ConvPoly::from_be_bytes(&enc_msg_bytes);
        let dec_msg = keypair.private.decrypt_to_bytes(enc_msg_debyted);
        assert_eq!(msg, dec_msg, "debyting message failed");


        // // Test random messages with new key pairs
        // let num_tests = 100;
        // let mut rng = rand::thread_rng();

        // for _ in 0..num_tests {
        //     let msg_len = rng.gen_range(0..100);
        //     let msg: Vec<u8> = (0..=msg_len).map(|_| rng.gen_range(1..=127)).collect();
        //     let keypair = NtruKeyPair::new();
        //     let enc_msg = keypair.public.encrypt_bytes(msg.clone());
        //     let dec_msg = keypair.private.decrypt_to_bytes(enc_msg);
        //     assert_eq!(msg, dec_msg, "Random message failed");
        // }
    }
}
