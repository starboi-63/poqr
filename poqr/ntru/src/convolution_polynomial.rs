use rand::prelude::*;

/// Generates a random ternary polynomial of degree less than `n` with `num_ones` 1s and `num_neg_ones` -1s.
/// The remaining coefficients are 0. The polynomial can be viewed as an element of the ring Z\[x\]/(x^n - 1).
///
/// ### Arguments
/// * `n` - Modulus of the polynomial degree (i.e. x^n = 1)
/// * `num_ones` - Number of coefficients equal to 1 in the polynomial
/// * `num_neg_ones` - Number of coefficients equal to -1 in the polynomial
pub fn ternary_polynomial(n: usize, num_ones: usize, num_neg_ones: usize) -> ConvolutionPolynomial {
    // Sanity checks
    assert!(
        num_ones + num_neg_ones <= n,
        "Number of 1s and -1s should be <= n (the number of terms in the polynomial)"
    );
    assert!(n > 0, "Polynomial degree should be greater than 0");

    let mut poly = ConvolutionPolynomial { coeffs: vec![0; n] };
    let mut rng = rand::thread_rng();
    let mut rand_indices: Vec<usize> = (0..n).collect();
    rand_indices.shuffle(&mut rng);

    // Set the first `num_ones` random indices to 1
    for i in 0..num_ones {
        poly.coeffs[rand_indices[i]] = 1;
    }

    // Set the next `num_neg_ones` random indices to -1
    for i in num_ones..num_ones + num_neg_ones {
        poly.coeffs[rand_indices[i]] = -1;
    }

    poly
}

/// A polynomial in the ring of convolution polynomials Z\[x\]/(x^N - 1). Here, N is the modulus of the polynomial
/// degree, and equals the length of the `coeffs` vector.
#[derive(Debug, Clone)]
pub struct ConvolutionPolynomial {
    pub coeffs: Vec<i32>, // Coefficients of the polynomial such that coeffs[i] is the coefficient of x^i
}

impl ConvolutionPolynomial {
    /// Returns the degree of the polynomial (i.e. the highest power of x with a non-zero coefficient)
    pub fn deg(&self) -> usize {
        self.coeffs.iter().rposition(|&x| x != 0).unwrap_or(0)
    }

    /// Returns whether the polynomial is the zero polynomial (i.e. all coefficients are 0)
    pub fn is_zero(&self) -> bool {
        self.coeffs.iter().all(|&x| x == 0)
    }

    /// Returns the leading coefficient of the polynomial (i.e. the coefficient of x^degree)
    pub fn lc(&self) -> i32 {
        self.coeffs[self.deg()]
    }

    /// Adds another polynomial to this one by adding the corresponding coefficients. If `m` is
    /// provided, the addition is performed modulo `m` (i.e. in the ring (Z/mZ)\[x\]/(x^N - 1) instead
    /// of Z\[x\]/(x^N - 1)).
    pub fn add(self, other: &ConvolutionPolynomial, m: Option<i32>) -> ConvolutionPolynomial {
        assert!(
            self.coeffs.len() == other.coeffs.len(),
            "Polynomials should be part of the same ring"
        ); // Sanity check

        let n = self.coeffs.len();
        let mut result = ConvolutionPolynomial { coeffs: vec![0; n] };

        for i in 0..n {
            result.coeffs[i] = self.coeffs[i] + other.coeffs[i];
            if let Some(m) = m {
                result.coeffs[i] = result.coeffs[i].rem_euclid(m);
            }
        }

        result
    }

    /// Subtracts another polynomial from this one by subtracting the corresponding coefficients. If `m` is
    /// provided, the subtraction is performed modulo `m` (i.e. in the ring (Z/mZ)\[x\]/(x^N - 1) instead
    /// of Z\[x\]/(x^N - 1)).
    pub fn sub(self, other: &ConvolutionPolynomial, m: Option<i32>) -> ConvolutionPolynomial {
        assert!(
            self.coeffs.len() == other.coeffs.len(),
            "Polynomials should be part of the same ring"
        ); // Sanity check

        let n = self.coeffs.len();
        let mut result = ConvolutionPolynomial { coeffs: vec![0; n] };

        for i in 0..n {
            result.coeffs[i] = self.coeffs[i] - other.coeffs[i];
            if let Some(m) = m {
                result.coeffs[i] = result.coeffs[i].rem_euclid(m);
            }
        }

        result
    }

    /// Multiplies this polynomial by another using the convolution operation in the ring. If `m` is
    /// provided, the multiplication is performed modulo `m` (i.e. in the ring (Z/mZ)\[x\]/(x^N - 1) instead
    /// of Z\[x\]/(x^N - 1)).
    pub fn mul(self, other: &ConvolutionPolynomial, m: Option<i32>) -> ConvolutionPolynomial {
        assert!(
            self.coeffs.len() == other.coeffs.len(),
            "Polynomials should be part of the same ring"
        ); // Sanity check

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

    /// Divides the polynomial by another polynomial and returns the quotient and remainder. The division is
    /// treated as though it is happening within the polynomial ring (Z/mZ)[x]/(x^N-1). If `m` is not a unit in
    /// the ring (Z/mZ), then the division is not possible and an error is returned.
    pub fn div(
        &self,
        divisor: &ConvolutionPolynomial,
        m: i32,
    ) -> Result<(ConvolutionPolynomial, ConvolutionPolynomial), String> {
        let n = self.coeffs.len();

        // Sanity checks
        assert!(
            n == divisor.coeffs.len(),
            "divmod: Polynomials must be in the same ring"
        );
        assert!(
            !divisor.is_zero(),
            "divmod: Division by zero polynomial not permitted"
        );

        let mut remainder = self.clone();
        let mut quotient = ConvolutionPolynomial { coeffs: vec![0; n] };

        // Check whether the given divisor is valid by attempting to compute the multiplicative inverse of its leading coefficient
        let inverse_divisor_lc = if let Ok(inverse) = inverse(divisor.lc(), m) {
            inverse
        } else {
            return Err("Invalid divisor polynomial; no multiplicative inverse for its leading coefficient (mod m)".to_string());
        };

        while remainder.deg() >= divisor.deg() {
            // Construct the term c * x^d
            let d = remainder.deg() - divisor.deg();
            let c = (remainder.lc() * inverse_divisor_lc).rem_euclid(m);
            let term = ConvolutionPolynomial {
                coeffs: (0..n).map(|i| if i == d { c } else { 0 }).collect(),
            };

            // Add the term to the quotient
            quotient = quotient.add(&term, Some(m));
            // Subtract the term * divisor from the dividend
            remainder = remainder.sub(&divisor.clone().mul(&term, Some(m)), Some(m));
        }

        Ok((quotient, remainder))
    }

    // Computes the inverse of the given polynomial within the ring (Z/mZ)\[x\]/(x^N - 1) using the
    // Extended Euclidean Algorithm. Returns `None`` if the polynomial is not invertible.
    // pub fn inverse(&self, m: i32) -> Option<ConvolutionPolynomial> {
    //     todo!()
    // }
}

/// The Euclidean Algorithm. Return the greatest common divisor and a and b.
pub fn gcd(a: i32, b: i32) -> i32 {
    assert!(a != 0 || b != 0, "At least one of a and b must be non-zero");
    let (mut a, mut b) = (a.clone(), b.clone());

    while b != 0 {
        (a, b) = (b, a % b);
    }

    a
}

/// The Extended Euclidean Algorithm. Returns (gcd, x, y) such that a*x + b*y = gcd(a, b).
pub fn extended_gcd(a: i32, b: i32) -> (i32, i32, i32) {
    assert!(a != 0 || b != 0, "At least one of a and b must be non-zero");
    let (mut a, mut b) = (a.clone(), b.clone());
    let (mut x, mut y, mut z, mut w) = (1, 0, 0, 1);

    while b != 0 {
        (x, y, z, w) = (z, w, x - (a / b) * z, y - (a / b) * w);
        (a, b) = (b, a % b);
    }

    (a, x, y)
}

/// Returns the multiplicative inverse of `a` within the unit group (Z/mZ)*. Returns an error if no
/// such inverse exists (i.e. if `a` is not relatively prime to `m`, and therefore not a member of the group).
pub fn inverse(a: i32, m: i32) -> Result<i32, String> {
    if a == 0 {
        return Err("The multiplicative inverse of 0 does not exist.".to_string());
    }

    if gcd(a, m) != 1 {
        return Err("`a` only has an inverse (mod m) if it is relatively prime to m.".to_string());
    }

    let (_, x, _) = extended_gcd(a, m);
    Ok(x.rem_euclid(m))
}
