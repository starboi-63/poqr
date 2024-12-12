#[cfg(test)]
mod tests {
    use crate::convolution_polynomial::{
        extended_gcd, gcd, inverse, ternary_polynomial, ConvolutionPolynomial,
    };
    use rand::Rng;

    #[test]
    fn test_ternary_polynomial() {
        let num_tests = 100;

        for _ in 0..num_tests {
            let n = rand::thread_rng().gen_range(1..=15);
            let num_ones = rand::thread_rng().gen_range(0..=n);
            let num_neg_ones = rand::thread_rng().gen_range(0..=(n - num_ones));
            let poly = ternary_polynomial(n, num_ones, num_neg_ones);

            assert_eq!(poly.coeffs.len(), n, "Polynomial length should be n");
            assert_eq!(
                poly.coeffs.iter().filter(|&&c| c == 1).count(),
                num_ones,
                "Number of 1 coefficients should match. N: {}, Poly: {:?}",
                n,
                poly
            );
            assert_eq!(
                poly.coeffs.iter().filter(|&&c| c == -1).count(),
                num_neg_ones,
                "Number of -1 coefficients should match"
            );
            assert!(
                poly.coeffs.iter().all(|&c| c >= -1 && c <= 1),
                "Coefficients should be ternary (-1, 0, 1)"
            );
        }
    }

    #[test]
    fn test_convolution_polynomial_deg() {
        // Normal polynomial
        let poly = ConvolutionPolynomial {
            coeffs: vec![0, 0, 3, 0, 0], // 3x^2
        };
        assert_eq!(poly.deg(), 2, "Degree should be 2");

        // Zero polynomial
        let zero_poly = ConvolutionPolynomial { coeffs: vec![0; 5] };
        assert_eq!(zero_poly.deg(), 0, "Degree of zero polynomial should be 0");

        // Completely non-zero polynomial
        let poly = ConvolutionPolynomial {
            coeffs: vec![1, 2, 3, 4, 5], // 5x^4 + 4x^3 + 3x^2 + 2x + 1
        };
        assert_eq!(poly.deg(), 4, "Degree should be 4");
    }

    #[test]
    fn test_convolution_polynomial_is_zero() {
        // Zero polynomial
        let zero_poly = ConvolutionPolynomial { coeffs: vec![0; 5] };
        assert!(zero_poly.is_zero(), "Zero polynomial should return true");

        // Non-zero polynomial
        let non_zero_poly = ConvolutionPolynomial {
            coeffs: vec![0, 1, 0], // x
        };
        assert!(
            !non_zero_poly.is_zero(),
            "Non-zero polynomial should return false"
        );
    }

    #[test]
    fn test_convolution_polynomial_lc() {
        // Leading coefficient at the end
        let poly = ConvolutionPolynomial {
            coeffs: vec![1, 2, 3, 4, 5], // 5x^4 + 4x^3 + 3x^2 + 2x + 1
        };
        assert_eq!(poly.lc(), 5, "Leading coefficient should be 5");

        // Leading coefficient with zeros at the end
        let poly = ConvolutionPolynomial {
            coeffs: vec![1, 2, 3, 4, 0], // 4x^3 + 3x^2 + 2x + 1
        };
        assert_eq!(poly.lc(), 4, "Leading coefficient should be 4");

        // Leading coefficient with zeros at the beginning
        let poly = ConvolutionPolynomial {
            coeffs: vec![0, 0, 3, 4, 5], // 5x^4 + 4x^3 + 3x^2
        };
        assert_eq!(poly.lc(), 5, "Leading coefficient should be 5");

        // Leading coefficient surrounded by zeros
        let poly = ConvolutionPolynomial {
            coeffs: vec![0, 0, 3, 0, 0], // 3x^2
        };
        assert_eq!(poly.lc(), 3, "Leading coefficient should be 3");
    }

    #[test]
    fn test_convolution_polynomial_add() {
        // Addition without modulo
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
        };
        let expected_sum = ConvolutionPolynomial {
            coeffs: vec![6, 5, 4], // 4x^2 + 5x + 6
        };
        let sum = poly1.add(&poly2, None);
        assert_eq!(
            expected_sum.coeffs, sum.coeffs,
            "Addition without modulo failed"
        );

        // Addition without modulo with negative coefficients
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![4, -3, 2], // 2x^2 - 3x + 4
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
        };
        let expected_sum = ConvolutionPolynomial {
            coeffs: vec![6, -1, 4], // 4x^2 - x + 6
        };
        let sum = poly1.add(&poly2, None);
        assert_eq!(
            expected_sum.coeffs, sum.coeffs,
            "Addition without modulo failed"
        );

        // Modulo 5 addition without wraparound
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![1, 2, 3], // 3x^2 + 2x + 1
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![3, 2, 1], // x^2 + 2x + 3
        };
        let expected_sum = ConvolutionPolynomial {
            coeffs: vec![4, 4, 4], // 4x^2 + 4x + 4
        };
        let sum = poly1.add(&poly2, Some(5));
        assert_eq!(expected_sum.coeffs, sum.coeffs, "Addition modulo 5 failed");

        // Modulo 5 addition with wraparound
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![2, -16, 10], // 10x^2 - 16x + 2
        };
        let expected_sum = ConvolutionPolynomial {
            coeffs: vec![1, 2, 2], // 2x^2 + 2x + 1
        };
        let sum = poly1.add(&poly2, Some(5));
        assert_eq!(expected_sum.coeffs, sum.coeffs, "Addition modulo 5 failed");
    }

    #[test]
    fn test_convolution_polynomial_sub() {
        // Subtraction without modulo
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
        };
        let expected_diff = ConvolutionPolynomial {
            coeffs: vec![2, 1, 0], // x + 2
        };
        let diff = poly1.sub(&poly2, None);
        assert_eq!(
            expected_diff.coeffs, diff.coeffs,
            "Subtraction without modulo failed"
        );

        // Subtraction without modulo with negative coefficients
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![4, -3, 2], // 2x^2 - 3x + 4
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![2, 2, 3], // 3x^2 + 2x + 2
        };
        let expected_diff = ConvolutionPolynomial {
            coeffs: vec![2, -5, -1], // -x^2 - 5x + 2
        };
        let diff = poly1.sub(&poly2, None);
        assert_eq!(
            expected_diff.coeffs, diff.coeffs,
            "Subtraction without modulo failed"
        );

        // Modulo 5 subtraction without wraparound
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![4, 3, 2], // 2x^2 + 3x + 4
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
        };
        let expected_diff = ConvolutionPolynomial {
            coeffs: vec![2, 1, 0], // x + 2
        };
        let diff = poly1.sub(&poly2, Some(5));
        assert_eq!(
            expected_diff.coeffs, diff.coeffs,
            "Subtraction modulo 5 failed"
        );

        // Modulo 5 subtraction with wraparound
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![2, 3, 2], // 2x^2 + 3x + 2
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![-5, 16, 10], // 10x^2 + 16x - 5
        };
        let expected_diff = ConvolutionPolynomial {
            coeffs: vec![2, 2, 2], // 2x^2 + 2x + 2
        };
        let diff = poly1.sub(&poly2, Some(5));
        assert_eq!(
            expected_diff.coeffs, diff.coeffs,
            "Subtraction modulo 5 failed"
        );
    }

    #[test]
    fn test_convolution_polynomial_mul() {
        // Example in the ring Z[x]/(x^5 - 1)
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![1, -2, 0, 4, -1], // -x^4 + 4x^3 - 2x + 1
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![3, 4, -2, 5, 2], // 2x^4 + 5x^3 - 2x^2 + 4x + 3
        };
        let expected_product = ConvolutionPolynomial {
            coeffs: vec![-13, 20, -7, 19, 5], // 5x^4 + 19x^3 - 7x^2 + 20x - 13
        };
        let product = poly1.mul(&poly2, None);
        assert_eq!(
            expected_product.coeffs, product.coeffs,
            "Multiplication failed"
        );

        // Example in the ring (Z/11Z)[x]/(x^5 - 1)
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![1, -2, 0, 4, -1], // -x^4 + 4x^3 - 2x + 1
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![3, 4, -2, 5, 2], // 2x^4 + 5x^3 - 2x^2 + 4x + 3
        };
        let expected_product = ConvolutionPolynomial {
            coeffs: vec![9, 9, 4, 8, 5], // 5x^4 + 8x^3 + 4x^2 + 9x + 10
        };
        let product = poly1.mul(&poly2, Some(11));
        assert_eq!(
            expected_product.coeffs, product.coeffs,
            "Multiplication modulo 11 failed"
        );
    }

    #[test]
    fn test_convolution_polynomial_div() {
        // Example in the ring (Z/2Z)[x]/(x^6 - 1)
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![1, 0, 0, 0, 0, 1], // x^5 + 1
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![1, 1, 0, 0, 1, 0], // x^4 + x + 1
        };
        let expected_quotient = ConvolutionPolynomial {
            coeffs: vec![0, 1, 0, 0, 0, 0], // x
        };
        let expected_remainder = ConvolutionPolynomial {
            coeffs: vec![1, 1, 1, 0, 0, 0], // x^2 + x + 1
        };
        let (quotient, remainder) = poly1.div(&poly2, 2).unwrap();
        assert_eq!(
            expected_quotient.coeffs, quotient.coeffs,
            "Division quotient failed"
        );
        assert_eq!(
            expected_remainder.coeffs, remainder.coeffs,
            "Division remainder failed"
        );

        // Another example in the ring (Z/2Z)[x]/(x^6 - 1)
        let poly1 = ConvolutionPolynomial {
            coeffs: vec![1, 1, 0, 0, 1, 0], // x^4 + x + 1
        };
        let poly2 = ConvolutionPolynomial {
            coeffs: vec![1, 1, 1, 0, 0, 0], // x^2 + x + 1
        };
        let expected_quotient = ConvolutionPolynomial {
            coeffs: vec![0, 1, 1, 0, 0, 0], // x^2 + x
        };
        let expected_remainder = ConvolutionPolynomial {
            coeffs: vec![1, 0, 0, 0, 0, 0], // 1
        };
        let (quotient, remainder) = poly1.div(&poly2, 2).unwrap();
        assert_eq!(
            expected_quotient.coeffs, quotient.coeffs,
            "Division quotient failed"
        );
        assert_eq!(
            expected_remainder.coeffs, remainder.coeffs,
            "Division remainder failed"
        );
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6, "GCD of 48 and 18 should be 6");
        assert_eq!(gcd(101, 103), 1, "GCD of two primes should be 1");
        assert_eq!(gcd(2, 4), 2, "GCD of 2 and 4 should be 2");
        assert_eq!(gcd(72, 36), 36, "GCD of 72 and 36 should be 36");
        assert_eq!(gcd(72, 54), 18, "GCD of 72 and 54 should be 18");
    }

    #[test]
    fn test_extended_gcd() {
        let mut rng = rand::thread_rng();
        let num_tests = 100;

        for _ in 0..num_tests {
            let a = rng.gen_range(1..=1000);
            let b = rng.gen_range(1..=1000);
            let (d, x, y) = extended_gcd(a, b);

            assert_eq!(d, gcd(a, b), "GCD calculation failed");
            assert_eq!(d, a * x + b * y, "Extended GCD calculation failed");
        }
    }

    #[test]
    fn test_inverse_modulo() {
        // Test with prime modulus (so that all elements have inverses)
        let inv = inverse(1, 7).expect("Inverse should exist");
        assert_eq!(inv, 1, "Inverse modulo calculation failed");

        let inv = inverse(2, 7).expect("Inverse should exist");
        assert_eq!(inv, 4, "Inverse modulo calculation failed");

        let inv = inverse(3, 7).expect("Inverse should exist");
        assert_eq!(inv, 5, "Inverse modulo calculation failed");

        // No inverse exists
        assert!(
            inverse(6, 9).is_err(),
            "Inverse should not exist for non-coprime"
        );
    }
}
