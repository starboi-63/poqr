use ntru::convolution_polynomial::ConvPoly;

fn main() {
    // Example in the ring (Z/2Z)[x]/(x^5 - 1)
    let poly = ConvPoly {
        coeffs: vec![1, 1, 0, 0, 1], // x^4 + x + 1
    };
    let expected_inverse = ConvPoly {
        coeffs: vec![1, 0, 1, 1], // x^3 + x^2 + 1
    };
    let inverse = poly.inverse(2, 5).unwrap();
    assert_eq!(
        expected_inverse.coeffs, inverse.coeffs,
        "Inverse modulo 2 failed"
    );

    println!("Done!");
}
