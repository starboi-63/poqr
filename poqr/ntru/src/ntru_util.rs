use crate::convolution_polynomial::*;
use crate::params::*;

/// Takes in a plain message encoded in ASCII and returns a convolution polynomial with coefficients representing that message
pub fn serialize(plain_msg: Vec<u8>) -> ConvPoly {
    assert!(
        plain_msg.len() * 5 <= N,
        "serialize: Message cannot exceed N - 1 in length"
    );
    // Convert the message to a vector of ternary digits
    let mut digit_vec = Vec::with_capacity(plain_msg.len() * 5);
    for c in plain_msg {
        let arr = ternary(c.into());
        digit_vec.extend_from_slice(&arr);
    }

    ConvPoly { coeffs: digit_vec }
}

/// Converts a 32 bit integer to a balanced ternary representation in the form of a 5-integer array
/// Max value is 242
fn ternary(mut c: i32) -> [i32; 5] {
    assert!(c < 242 && c >= 0);
    if c == 0 {
        return [0; 5];
    }

    let mut digits = [0; 5];
    for i in (0..5).rev() {
        if c == 0 {
            break;
        }
        let rem_temp = c % 3;
        c /= 3;
        digits[i] = if rem_temp == 2 { -1 } else { rem_temp };
    }
    digits
}

/// Deserializes a convolution polynomial into the message it represents as a vector
/// of u8s
pub fn deserialize(ser_msg: ConvPoly) -> Vec<u8> {
    let coeffs = ser_msg.coeffs;
    let mut ret: Vec<u8> = Vec::new();
    for chunk in coeffs.chunks(5) {
        let mut padded = [0; 5];
        padded[..chunk.len()].copy_from_slice(chunk);
        match out_of_ternary(&padded) {
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

    const POWERS: [i32; 5] = [1, 3, 9, 27, 81];

    let mut ans = 0;
    // We know every balanced ternary number will be 5 indices, so here's a fun little
    // constant time deserialization :)
    ans += bal_tern_esc(ser_ch[4], POWERS[0]);
    ans += bal_tern_esc(ser_ch[3], POWERS[1]);
    ans += bal_tern_esc(ser_ch[2], POWERS[2]);
    ans += bal_tern_esc(ser_ch[1], POWERS[3]);
    ans += bal_tern_esc(ser_ch[0], POWERS[4]);
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
fn bal_tern_esc(n: i32, exp: i32) -> i32 {
    if n == -1 {
        2 * exp
    } else {
        n * exp
    }
}
