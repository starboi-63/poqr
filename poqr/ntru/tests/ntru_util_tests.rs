#[cfg(test)]
mod ntru_util_tests {
    use ntru::ntru_util::{deserialize, serialize};

    #[test]
    fn test_serialize() {
        let msg_test = String::from("hello");
        let msg_test_bytes = msg_test.as_bytes().to_vec();
        println!(
            "coeffs for test bytes: {:?}",
            serialize(msg_test_bytes.clone()).coeffs
        );
        assert_eq!(
            serialize(msg_test_bytes).coeffs,
            vec![1, 0, -1, 1, -1, 1, 0, -1, 0, -1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0]
        );
    }

    #[test]
    fn test_ser_deserialize() {
        let msg = "hello guys this is alex";
        let ser_msg = {
            let msg_bytes = String::from(msg).as_bytes().to_vec();
            serialize(msg_bytes)
        };
        println!("Coeffs: {:?}", ser_msg.coeffs);
        let deser = deserialize(ser_msg);
        println!("deser: {}", String::from_utf8_lossy(&deser));
        assert_eq!(msg.as_bytes().to_vec(), deser);
        println!("characters in message: {}", msg.len());
    }
}
