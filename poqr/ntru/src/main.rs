use ntru::ntru_key::NtruKeyPair;

fn main() {
    let keypair = NtruKeyPair::new();
    let msg = "minky".as_bytes().to_vec();
    let enc_msg = keypair.encrypt(msg.clone());
    let dec_msg = keypair.decrypt(enc_msg);
    assert_eq!(msg, dec_msg);

    println!("Done!");
}
