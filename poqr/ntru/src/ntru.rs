use crate::convolution_polynomial::*;
use std::collections::VecDeque;

// NTRU Parameters, derived by Tanish and Alex
const N: usize = 503;
const P: u32 = 3;
const Q: u32 = 419;
const D: u32 = 23;

pub fn encrypt(msg: Vec<u8>, k_pub: ConvolutionPolynomial) {}

pub fn decrypt(enc_msg: ConvolutionPolynomial, k_priv: ConvolutionPolynomial) {}

/// Takes in a plain message encoded in ASCII and returns a convolution polynomial with coefficients reperesenting that message
fn serialize(plain_msg: Vec<u8>) -> ConvolutionPolynomial {
    assert!(
        plain_msg.len() * 5 <= N,
        "serialize: Message cannot exceed N - 1 in length"
    );
    let digit_vec = {
        let mut temp: Vec<i32> = Vec::new();
        for c in plain_msg {
            temp.extend(ternary(c.into()));
        }
        temp
    };
    ConvolutionPolynomial { coeffs: digit_vec }
}

#[test]
fn test_ser() {
    let msg_test = String::from("hello");
    let msg_test_bytes = msg_test.as_bytes().to_vec();
    println!("coeffs for test bytes: {:?}", serialize(msg_test_bytes.clone()).coeffs);
    assert_eq!(serialize(msg_test_bytes).coeffs, vec![1, 0, -1, 1, -1, 1, 0, -1, 0, -1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0]);
}

/// Converts a 32 bit integer to a balanced ternary representation in the form of a 5-integer array
/// Max value is 242
fn ternary(mut c: i32) -> Vec<i32> {
    assert!(c < 242, "5-index ternary can encode at max a value of 242");
    if c == 0 {
        return vec![0, 0, 0, 0, 0]
    }
    let mut digits: VecDeque<i32> = VecDeque::new();
    while c > 0 {
        let rem: i32 = {
            let rem_temp = c % 3;
            c /= 3;
            if rem_temp == 2 {
                -1
            } else {
                rem_temp
            }
        };
        digits.push_front(rem);
    }
    //NOTE: Might wanna find a nicer way of doing this
    while digits.len() < 5 {
        digits.push_front(0);
    }
    digits.into_iter().collect()
}

/// Deserializes a convolution polynomial into the message it represents as a vector
/// of u8s
fn deserialize(ser_msg: ConvolutionPolynomial) -> Vec<u8> {
    let coeffs = ser_msg.coeffs;
    let mut ret: Vec<u8> = Vec::new();
    for chunk in coeffs.chunks(5) {
        match out_of_ternary(chunk) {
            Some(c) => ret.push(c),
            None => ()
        }
    }
    ret
}

/// Takes a balanced ternary number in the form of an array and converts it to
/// a decimal u8 (aka a char)
/// Returns None if given a non valid char encoding
fn out_of_ternary(ser_ch: &[i32]) -> Option<u8> {
    println!("Running out of ternary on: {:?}", ser_ch);
    if ser_ch.len() != 5 {
        return None
    }
    let mut ans = 0;
    // We know every balanced ternary number will be 5 indices, so here's a fun little
    // constant time deserialization :)
    ans += bal_tern_esc(ser_ch[4], 0);
    ans += bal_tern_esc(ser_ch[3], 1);
    ans += bal_tern_esc(ser_ch[2], 2);
    ans += bal_tern_esc(ser_ch[1], 3);
    ans += bal_tern_esc(ser_ch[0], 4);
    match u8::try_from(ans) {
        Ok(a) => Some(a),
        Err(_) => None
    }
} 

/// Takes an index from an array representing a balanced ternary number
/// and converts it to a decimal component of the decimal conversion, 
/// dependent on its position in the array and index value.
fn bal_tern_esc(n: i32, i: u32) -> i32 {
    if n == -1 {
        2 * 3_i32.pow(i)
    } else {
        n * 3_i32.pow(i)
    }
}

#[test]
fn test_ser_deser() {
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