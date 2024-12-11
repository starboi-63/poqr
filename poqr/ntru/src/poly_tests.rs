
#[cfg(test)]
mod poly_tests { 
    use crate::convolution_polynomial::{ConvolutionPolynomial, ternary_polynomial};

    /// Helper function to create a `ConvolutionPolynomial` from a vector of coefficients.
    fn poly_from_vec(coeffs: Vec<i32>) -> ConvolutionPolynomial {
        ConvolutionPolynomial { coeffs }
    } 

    #[test]
    fn test_degree() {
        let poly = poly_from_vec(vec![0, 0, 3, 0, 1]); // Highest non-zero coefficient is at index 4
        assert_eq!(poly.degree(), 4);

        let zero_poly = poly_from_vec(vec![0, 0, 0, 0]);
        assert_eq!(zero_poly.degree(), 0);
    }

    #[test]
    fn test_is_zero() {
        let zero_poly = poly_from_vec(vec![0, 0, 0, 0]);
        assert!(zero_poly.is_zero(), "Expected the polynomial to be zero");

        let non_zero_poly = poly_from_vec(vec![0, 1, 0]);
        assert!(!non_zero_poly.is_zero(), "Expected the polynomial to be non-zero");
    }

    #[test]
    fn test_leading_coefficient() {
        let poly = poly_from_vec(vec![0, 0, 3, 0, 5]); // Degree 4, leading coefficient is 5
        assert_eq!(poly.lc(), 5);

        let zero_poly = poly_from_vec(vec![0, 0, 0, 0]);
        assert_eq!(zero_poly.lc(), 0);
    }

    #[test]
    fn test_sub() {
        let poly1 = poly_from_vec(vec![3, 4, 5]);
        let poly2 = poly_from_vec(vec![1, 2, 3]);

        // Without modulo
        let result = poly1.clone().sub(poly2.clone(), None);
        assert_eq!(result.coeffs, vec![2, 2, 2], "Subtraction without modulo failed");

        // With modulo 5
        let result_mod = poly1.sub(poly2, Some(5));
        assert_eq!(result_mod.coeffs, vec![2, 2, 2], "Subtraction with modulo 5 failed");
    }

    #[test]
    fn test_mul() {
        let poly1 = poly_from_vec(vec![3, 2, 1]);
        let poly2 = poly_from_vec(vec![6, 4, 5]);

        let result = poly1.clone().mul(poly2.clone(), Some(7));
        assert_eq!(result.coeffs, vec![4, 1, 1]);
    }

}
