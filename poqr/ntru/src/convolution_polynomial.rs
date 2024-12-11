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
    ConvolutionPolynomial { coeffs: (0..n).map(|_| rng.gen_range(-1..=1)).collect() }
}

/// A polynomial in the ring of convolution polynomials Z[x]/(x^n - 1).
#[derive(Debug, Clone)]
pub struct ConvolutionPolynomial {
    pub coeffs: Vec<i32>, // Coefficients of the polynomial such that coeffs[i] is the coefficient of x^i
}

impl ConvolutionPolynomial {
    /// Returns the degree of the polynomial (i.e. the highest power of x with a non-zero coefficient)
    pub fn degree(&self) -> usize {
        self.coeffs.iter().rposition(|&x| x != 0).unwrap_or(0)
    }

    /// Returns whether the polynomial is the zero polynomial (i.e. all coefficients are 0)
    pub fn is_zero(&self) -> bool {
        self.coeffs.iter().all(|&x| x == 0)
    }

    /// Returns the leading coefficient of the polynomial (i.e. the coefficient of x^degree)
    pub fn lc(&self) -> i32 {
        self.coeffs[self.degree()]
    }

    /// Adds another polynomial to this one by adding the corresponding coefficients. If `m` is
    /// provided, the addition is performed modulo `m` (i.e. in the ring (Z/mZ)[x]/(x^n - 1) instead
    /// of Z[x]/(x^n - 1)).
    pub fn add(self, other: ConvolutionPolynomial, m: Option<i32>) -> ConvolutionPolynomial {
        assert!(self.coeffs.len() == other.coeffs.len(), "Polynomials should be part of the same ring"); // Sanity check

        let n = self.coeffs.len();
        let mut result = ConvolutionPolynomial { coeffs: vec![0;n] };

        for i in 0..n {
            result.coeffs[i] = self.coeffs[i] + other.coeffs[i];
            if let Some(m) = m {
                result.coeffs[i] = result.coeffs[i].rem_euclid(m);
            }
        }

        result
    }

    /// Subtracts another polynomial from this one by subtracting the corresponding coefficients. If `m` is
    /// provided, the subtraction is performed modulo `m` (i.e. in the ring (Z/mZ)[x]/(x^n - 1) instead
    /// of Z[x]/(x^n - 1)).
    pub fn sub(self, other: ConvolutionPolynomial, m: Option<i32>) -> ConvolutionPolynomial {
        assert!(self.coeffs.len() == other.coeffs.len(), "Polynomials should be part of the same ring"); // Sanity check

        let n = self.coeffs.len();
        let mut result = ConvolutionPolynomial { coeffs: vec![0;n] };

        for i in 0..n {
            result.coeffs[i] = self.coeffs[i] - other.coeffs[i];
            if let Some(m) = m {
                result.coeffs[i] = result.coeffs[i].rem_euclid(m);
            }
        }

        result
    }

    /// Multiplies this polynomial by another using the convolution operation in the ring. If `m` is
    /// provided, the multiplication is performed modulo `m` (i.e. in the ring (Z/mZ)[x]/(x^n - 1) instead
    /// of Z[x]/(x^n - 1)).
    pub fn mul(self, other: ConvolutionPolynomial, m: Option<i32>) -> ConvolutionPolynomial {
        assert!(self.coeffs.len() == other.coeffs.len(), "Polynomials should be part of the same ring"); // Sanity check
        
        let n = self.coeffs.len();
        let mut result = ConvolutionPolynomial { coeffs: vec![0; n] };

        // Perform the convolution operation on self and other and store in result
        for i in 0..n {
            for j in 0..n {
                result.coeffs[(i + j) % n] += self.coeffs[i] * other.coeffs[j];
            }
        }
      
        // If `m` is provided, then ensure that each coefficient lies in Z/mZ (which is the range [0,m])
        if let Some(m) = m {
            for i in 0..n {
                result.coeffs[i] = result.coeffs[i].rem_euclid(m);
            }
        }

        result
    }

    // Computes the inverse of the given polynomial within the ring (Z/mZ)[x]/(x^n - 1) using the
    // Extended Euclidean Algorithm. Returns None if the polynomial is not invertible.
    fn inverse(&self, m: i32) -> Option<ConvolutionPolynomial> {
        todo!()
    }

    /// Divides the polynomial by another polynomial and returns the quotient and remainder.
    fn divmod(
        &self,
        divisor: &ConvolutionPolynomial,
        m: i32,
    ) -> Result<(ConvolutionPolynomial, ConvolutionPolynomial), String> {
        let n = self.coeffs.len();
        
        // Sanity checks
        assert!(n == divisor.coeffs.len(), "divmod: Polynomials must be in the same ring");
        assert!(!divisor.is_zero(), "divmod: Division by zero polynomial not permitted");
        
        let mut remainder = self.clone();
        let mut quotient = ConvolutionPolynomial {
            coeffs: vec![0;n],
        };

        // Check whether the given divisor is valid by attempting to compute the multiplicative inverse of its leading coefficient
        let inverse_divisor_lc = if let Ok(inverse) = inverse(divisor.lc(), m) {
            inverse
        } else {
            return Err("Invalid divisor polynomial; no multiplicative inverse for its leading coefficient (mod m)".to_string());
        };

        while remainder.degree() >= divisor.degree() {
            // Construct the term c * x^d
            let d = remainder.degree() - divisor.degree();
            let c = remainder.lc() * inverse_divisor_lc;
            let term = ConvolutionPolynomial { coeffs: (0..n).map(|i| if i == d { c } else { 0 }).collect() };

            // Add the term to the quotient
            quotient = quotient.add(term.clone(), Some(m));
            // Subtract the term * divisor from the dividend
            remainder = remainder.sub(divisor.clone().mul(term, Some(m)), Some(m));
        }
      
        Ok((quotient, remainder))
    }
}

/// The Euclidean Algorithm. Return the greatest common divisor and a and b.
fn gcd(a: i32, b: i32) -> i32 {
    let (mut a, mut b) = (a.clone(), b.clone());
    
    while b != 0 {
        (a, b) = (b, a % b);
    }

    a
}

/// The Extended Euclidean Algorithm. Returns (x, y) such that a*x + b*y = gcd(a, b).
fn extended_euclidean_algorithm(a: i32, b: i32) -> (i32, i32) {
    let (mut a, mut b) = (a.clone(), b.clone());
    let (mut x, mut y, mut z, mut w) = (1, 0, 0, 1);

    while b != 0 {
        (x, y, z, w) = (z, w, x - (a / b) * z, y - (a / b) * w);
        (a, b) = (b, a % b);
    }

    (x, y)
}

/// Returns the multiplicative inverse of `a` within the unit group (Z/mZ)^*.
fn inverse(a: i32, m: i32) -> Result<i32, String> {
    if gcd(a, m) != 1 {
        return Err("`a` only has an inverse (mod m) if it is relatively prime to m.".to_string());
    }

    let (x, _) = extended_euclidean_algorithm(a, m);
    Ok(x.rem_euclid(m))
}