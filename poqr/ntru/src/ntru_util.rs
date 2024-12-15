use crate::convolution_polynomial::*;
use crate::params::*;
use std::collections::VecDeque;

/// Takes in a plain message encoded in ASCII and returns a convolution polynomial with coefficients representing that message
pub fn serialize(plain_msg: Vec<u8>) -> ConvPoly {
    assert!(
        plain_msg.len() * 5 <= N,
        "serialize: Message cannot exceed N - 1 in length"
    );
    // Convert the message to a vector of ternary digits
    let mut digit_vec = Vec::new();
    for c in plain_msg {
        digit_vec.extend(ternary(c.into()));
    }

    ConvPoly { coeffs: digit_vec }
}

/// Converts a 32 bit integer to a balanced ternary representation in the form of a 5-integer array
/// Max value is 242
fn ternary(mut c: i32) -> Vec<i32> {
    // Sanity checking
    assert!(
        c < 242 && c >= 0,
        "5-index ternary can encode at max a positive value less than 242"
    );
    // Base case
    if c == 0 {
        return vec![0, 0, 0, 0, 0];
    }
    let mut digits: VecDeque<i32> = VecDeque::new();
    // Ternary conversion ; due to sanity checks should not exceed 5 digits
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
    // NOTE : Make this nicer
    //
    // Pad with zeros on less than 5 digit cases
    while digits.len() < 5 {
        digits.push_front(0);
    }
    digits.into_iter().collect()
}

/// Deserializes a convolution polynomial into the message it represents as a vector
/// of u8s
pub fn deserialize(ser_msg: ConvPoly) -> Vec<u8> {
    let coeffs = ser_msg.coeffs; 
    let mut ret: Vec<u8> = Vec::new();
    for chunk in coeffs.chunks(5) {
        match out_of_ternary(chunk) {
            Some(c) => {
                ret.push(c);
            }
            None => (),
        }
    }
    ret
}

/// Takes a balanced ternary number in the form of an array and converts it to
/// a decimal u8 (aka a char)
/// Returns None if given a non valid char encoding
fn out_of_ternary(ser_ch: &[i32]) -> Option<u8> {
    if ser_ch == [0; 5] {
        return None;
    }
    let mut ser_ch = Vec::from(ser_ch);
    if ser_ch.len() % 5 != 0 || ser_ch.len() > 5 {
        // Pad the array with zeros until it reaches length 5
        while ser_ch.len() < 5 {
            ser_ch.push(0);
        }
        // Truncate to 5 elements if it's longer
        ser_ch.truncate(5);
    }
    let mut ans = 0;
    // We know every balanced ternary number will be 5 indices, so here's a fun little
    // constant time deserialization :)
    ans += bal_tern_esc(ser_ch[4], 0);
    ans += bal_tern_esc(ser_ch[3], 1);
    ans += bal_tern_esc(ser_ch[2], 2);
    ans += bal_tern_esc(ser_ch[1], 3);
    ans += bal_tern_esc(ser_ch[0], 4);
    // If value is for some reason not a u8, returns None
    match u8::try_from(ans) {
        Ok(a) => Some(a),
        Err(_) => {
            eprintln!(
                "out_of_ternary: Shouldn't try to deserialize a non u8 representable character"
            );
            None
        }
    }
}

/// Takes a value from an array representing a balanced ternary number
/// and converts it to a decimal component of the decimal conversion,
/// dependent on its position in the array and index value.
fn bal_tern_esc(n: i32, i: u32) -> i32 {
    if n == -1 {
        2 * 3_i32.pow(i)
    } else {
        n * 3_i32.pow(i)
    }
}
