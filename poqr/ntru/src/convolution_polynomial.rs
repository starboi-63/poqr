use rand::prelude::*;

/// Generates a random ternary polynomial of degree less than `n` with `num_ones` 1s and `num_neg_ones` -1s.
/// The remaining coefficients are 0. The polynomial can be viewed as an element of the ring Z[x]/(x^n - 1).
///
/// ### Arguments
/// * `n` - Modulus of the polynomial degree (i.e. x^n = 1)
/// * `num_ones` - Number of coefficients equal to 1 in the polynomial
/// * `num_neg_ones` - Number of coefficients equal to -1 in the polynomial
pub fn ternary_polynomial(n: usize, num_ones: usize, num_neg_ones: usize) -> ConvolutionPolynomial {
    assert!(num_ones + num_neg_ones <= n); // Sanity check
    let mut rng = rand::thread_rng();

    ConvolutionPolynomial {
        coeffs: (0..n).map(|_| rng.gen_range(-1..=1)).collect(),
        n,
    }
}

/// A polynomial in the ring of convolution polynomials Z[x]/(x^n - 1).
pub struct ConvolutionPolynomial {
    pub coeffs: Vec<i32>, // Coefficients of the polynomial such that coeffs[i] is the coefficient of x^i
    pub n: usize, // Modulus of the polynomial degree (i.e. the polynomial is in the ring Z[x]/(x^n - 1))
}

impl ConvolutionPolynomial {
    /// Returns the degree of the polynomial (i.e. the highest power of x with a non-zero coefficient)
    fn degree(&self) -> usize {
        self.coeffs.iter().rposition(|&x| x != 0).unwrap_or(0)
    }

    /// Returns whether the polynomial is the zero polynomial (i.e. all coefficients are 0)
    fn is_zero(&self) -> bool {
        self.coeffs.iter().all(|&x| x == 0)
    }

    /// Returns the leading coefficient of the polynomial (i.e. the coefficient of x^degree)
    fn leading_coefficient(&self) -> i32 {
        self.coeffs[self.degree()]
    }

    /// Subtracts another polynomial from this one by subtracting the corresponding coefficients. If `m` is
    /// provided, the subtraction is performed modulo `m` (i.e. in the ring (Z/mZ)[x]/(x^n - 1) instead
    /// of Z[x]/(x^n - 1)).
    fn sub(self, other: ConvolutionPolynomial, m: Option<i32>) -> ConvolutionPolynomial {
        assert!(self.n == other.n); // Sanity check

        ConvolutionPolynomial {
            coeffs: self
                .coeffs
                .iter()
                .zip(other.coeffs.iter())
                .map(|(&a, &b)| {
                    if let Some(m) = m {
                        (a - b).rem_euclid(m)
                    } else {
                        a - b
                    }
                })
                .collect(),
            n: self.n,
        }
    }

    /// Multiplies this polynomial by another using the convolution operation in the ring. If `m` is
    /// provided, the multiplication is performed modulo `m` (i.e. in the ring (Z/mZ)[x]/(x^n - 1) instead
    /// of Z[x]/(x^n - 1)).
    fn mul(self, other: ConvolutionPolynomial, m: Option<i32>) -> ConvolutionPolynomial {
        let mut result_coeffs = vec![0; self.n];
        for i in 0..self.n {
            for j in 0..self.n {
                result_coeffs[(i + j) % self.n] += self.coeffs[i] * other.coeffs[j];
            }
        }

        ConvolutionPolynomial {
            coeffs: result_coeffs
                .iter()
                .map(|&x| if let Some(m) = m { x.rem_euclid(m) } else { x })
                .collect(),
            n: self.n,
        }
    }

    // Computes the inverse of the given polynomial within the ring (Z/mZ)[x]/(x^n - 1) using the
    // Extended Euclidean Algorithm. Returns None if the polynomial is not invertible.
    fn inverse(&self, m: i32) -> Option<ConvolutionPolynomial> {
        todo!()
    }

    /// Divides the polynomial by another polynomial and returns the quotient and remainder.
    fn divmod(
        &self,
        other: &ConvolutionPolynomial,
        m: i32,
    ) -> (ConvolutionPolynomial, ConvolutionPolynomial) {
        // Sanity checks
        assert!(self.n == other.n, "Polynomials must be in the same ring");
        assert!(!other.is_zero(), "Division by zero polynomial");

        // THE FOLLOWING CODE IS LIKELY TO BE INCORRECT (TARNISH)

        // let mut dividend = self.clone();
        // let mut quotient = ConvolutionPolynomial {
        //     coeffs: vec![0; self.n],
        //     n: self.n,
        // };

        // while dividend.degree() >= other.degree() {
        //     let degree_diff = dividend.degree() - other.degree();

        //     // Leading coefficient of the quotient term
        //     let lead_coeff =
        //         (dividend.coeffs[dividend.degree()] * other.coeffs[other.degree()]).rem_euclid(m);

        //     // Create the term to subtract
        //     let mut term = vec![0; self.n];
        //     term[degree_diff] = lead_coeff;

        //     let term_poly = ConvolutionPolynomial {
        //         coeffs: term,
        //         n: self.n,
        //     };

        //     quotient = quotient.sub(term_poly.clone(), Some(m));

        //     let subtracted = other.mul(term_poly, Some(m));
        //     dividend = dividend.sub(subtracted, Some(m));
        // }

        // (quotient, dividend)
    }
}
