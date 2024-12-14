use rand::prelude::*;
use std::fmt;

// TERNARY POLYNOMIALS

/// Generates a random ternary convolution polynomial of degree less than `n` with `num_ones` 1s and `num_neg_ones`
/// -1s. The remaining coefficients are 0. The polynomial can be viewed as an element of the ring Z\[x\]/(x^n - 1).
pub fn ternary_polynomial(n: usize, num_ones: usize, num_neg_ones: usize) -> ConvPoly {
    // Sanity checks
    assert!(
        num_ones + num_neg_ones <= n,
        "Number of 1s and -1s should be <= n (the number of terms in the polynomial)"
    );
    assert!(n > 0, "Polynomial degree should be greater than 0");

    let mut poly = ConvPoly { coeffs: vec![0; n] };
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

// CONVOLUTION POLYNOMIALS

/// A polynomial in the ring of convolution polynomials Z\[x\]/(x^N - 1). Here, N is the modulus of the polynomial
/// degree, and equals the length of the `coeffs` vector.
#[derive(Debug, Clone, PartialEq)]
pub struct ConvPoly {
    pub coeffs: Vec<i32>, // Coefficients of the polynomial such that coeffs[i] is the coefficient of x^i
}

/// Display implementation for convolution polynomials. The polynomial is displayed in the form
/// c0 + c1x + c2x^2 + ... + cnx^n where c0, c1, ..., cn are the coefficients of the polynomial.
impl fmt::Display for ConvPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut terms = Vec::new();
        for (i, &coeff) in self.coeffs.iter().enumerate() {
            if coeff != 0 {
                let coeff_str = if coeff == 1 {
                    "".to_string()
                } else if coeff == -1 {
                    "-".to_string()
                } else {
                    coeff.to_string()
                };
                let term = match i {
                    0 => format!("{}", coeff),
                    1 => format!("{}x", coeff_str),
                    _ => format!("{}x^{}", coeff_str, i),
                };
                terms.push(term);
            }
        }

        if terms.is_empty() {
            write!(f, "0")
        } else {
            write!(f, "{}", terms.join(" + ").replace("+ -", "- "))
        }
    }
}

impl ConvPoly {
    /// Constructs a constant polynomial f(x) = c in the ring Z\[x\]/(x^n - 1).
    pub fn constant(c: i32, n: usize) -> ConvPoly {
        ConvPoly {
            coeffs: (0..n).map(|i| if i == 0 { c } else { 0 }).collect(),
        }
    }

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

    /// Applies the modulus operation to each coefficient of the polynomial and returns the result,
    /// which lies in the ring (Z/mZ)\[x\]/(x^N - 1). The modulus `m` must be a positive integer.
    pub fn modulo(&self, m: i32) -> ConvPoly {
        assert!(m > 0, "Modulus `m` must be a positive integer");

        ConvPoly {
            coeffs: self.coeffs.iter().map(|x| x.rem_euclid(m)).collect(),
        }
    }

    /// Adds another polynomial to this one by adding the corresponding coefficients.
    pub fn add(&self, other: &ConvPoly) -> ConvPoly {
        assert!(
            self.coeffs.len() == other.coeffs.len(),
            "Polynomials should be part of the same ring"
        ); // Sanity check

        let n = self.coeffs.len();
        ConvPoly {
            coeffs: (0..n).map(|i| self.coeffs[i] + other.coeffs[i]).collect(),
        }
    }

    /// Subtracts another polynomial from this one by subtracting the corresponding coefficients.
    pub fn sub(&self, other: &ConvPoly) -> ConvPoly {
        assert!(
            self.coeffs.len() == other.coeffs.len(),
            "Polynomials should be part of the same ring"
        ); // Sanity check

        let n = self.coeffs.len();
        ConvPoly {
            coeffs: (0..n).map(|i| self.coeffs[i] - other.coeffs[i]).collect(),
        }
    }

    /// Multiplies this polynomial by another using the convolution operation in the ring.
    pub fn mul(&self, other: &ConvPoly) -> ConvPoly {
        assert!(
            self.coeffs.len() == other.coeffs.len(),
            "Polynomials should be part of the same ring"
        ); // Sanity check

        let n = self.coeffs.len();
        let mut result = ConvPoly { coeffs: vec![0; n] };

        for i in 0..n {
            for j in 0..n {
                result.coeffs[(i + j) % n] += self.coeffs[i] * other.coeffs[j];
            }
        }

        result
    }

    /// Divides the polynomial by another polynomial and returns the quotient and remainder. The division is
    /// treated as though it is happening within the polynomial ring (Z/mZ)\[x\]/(x^N-1). If `m` is not a unit in
    /// the ring (Z/mZ), then the division is not possible and an error is returned.
    pub fn div_mod(&self, divisor: &ConvPoly, m: i32) -> Result<(ConvPoly, ConvPoly), String> {
        let n = self.coeffs.len();

        // Sanity checks
        assert!(
            n == divisor.coeffs.len(),
            "Polynomials must be in the same ring"
        );
        assert!(
            !divisor.is_zero(),
            "Division by zero polynomial not permitted"
        );

        let mut remainder = self.clone();
        let mut quotient = ConvPoly { coeffs: vec![0; n] };

        // Check whether the given divisor is valid by attempting to compute the multiplicative inverse of its leading coefficient
        let inverse_divisor_lc = if let Ok(inverse) = inverse(divisor.lc(), m) {
            inverse
        } else {
            return Err("Invalid divisor polynomial; no multiplicative inverse for its leading coefficient (mod m)".to_string());
        };

        while remainder.deg() >= divisor.deg() && !remainder.is_zero() {
            // Construct the term c * x^d
            let d = remainder.deg() - divisor.deg();
            println!("d: {}", d);
            let c = (remainder.lc() * inverse_divisor_lc).rem_euclid(m);
            println!("c: {}", c);
            let term = ConvPoly {
                coeffs: (0..n).map(|i| if i == d { c } else { 0 }).collect(),
            };
            println!("term: {}", term);
            // Add the term to the quotient
            quotient = quotient.add(&term).modulo(m);
            // Subtract the term * divisor from the dividend
            remainder = remainder.sub(&divisor.clone().mul(&term)).modulo(m);
            println!("quotient (updated): {}", quotient);
            println!("remainder (updated): {}", remainder);
        }

        Ok((quotient, remainder))
    }

    /// The Extended Euclidean Algorithm for polynomials. Returns (gcd, s(x), t(x)) such that
    /// a(x)s(x) + b(x)t(x) = gcd(a(x), b(x)) within the ring (Z/mZ)\[x\]/(x^N - 1). Returns an error
    /// if division fails at any point (which occurs when the leading coefficient of the divisor isn't
    /// a unit in the ring Z/mZ).
    pub fn extended_gcd(
        a: &ConvPoly,
        b: &ConvPoly,
        m: i32,
    ) -> Result<(ConvPoly, ConvPoly, ConvPoly), String> {
        // Sanity checks
        assert!(
            a.coeffs.len() == b.coeffs.len(),
            "Polynomials should be part of the same ring"
        );
        assert!(
            !a.is_zero() || !b.is_zero(),
            "At least one of the polynomials must be non-zero"
        );
        assert!(m > 0, "Modulus `m` must be a positive integer");
        // Initial state
        // a(x) = 1a(x) + 0b(x)  -->  old_r(x) = a(x)old_s(x) + b(x)old_t(x)  so old_r(x) is a linear combination of a(x),b(x)
        // b(x) = 0a(x) + 1b(x)  -->  r(x) = a(x)s(x) + b(x)t(x)              so r(x) is also a linear combination of a(x),b(x)
        //
        // Update step
        // Let old_r(x) = r(x)q(x) + new_r(x) (polynomial division algo). Then because of the above, new_r(x) = old_r(x) - r(x)q(x)
        // is still a linear combination of a(x),b(x) with new_s(x) = old_s(x) - s(x)q(x) and new_t(x) = old_t(x) - t(x)q(x). By
        // induction, we can continue assigning new_r(x) to r(x) like this until r(x) = 0 (which we know will happen by the standard
        // Euclidean Algorithm) and be left with Bézout polynomial coefficients.
        let n = a.coeffs.len();
        let (mut old_r, mut old_s, mut old_t) = (
            a.clone(),
            ConvPoly::constant(1, n),
            ConvPoly::constant(0, n),
        );
        let (mut r, mut s, mut t) = (
            b.clone(),
            ConvPoly::constant(0, n),
            ConvPoly::constant(1, n),
        );

        while !r.is_zero() {
            println!();
            println!("old_r: {}, r: {}", old_r, r);
            let (q, new_r) = old_r.div_mod(&r, m)?;
            println!("q: {}, new_r: {}", q, new_r);
            (old_r, r) = (r, new_r);
            (old_s, s) = (s.clone(), old_s.sub(&s.mul(&q)).modulo(m));
            (old_t, t) = (t.clone(), old_t.sub(&t.mul(&q)).modulo(m));
            println!("old_s: {}, s: {}", old_s, s);
            println!("old_t: {}, t: {}", old_t, t);
        }

        Ok((old_r, old_s, old_t))
    }

    /// Computes the inverse of the given polynomial within the ring (Z/mZ)\[x\]/(x^N - 1)
    /// using the Extended Euclidean Algorithm. Returns an error if the polynomial is not invertible.
    pub fn inverse(&self, m: i32) -> Result<ConvPoly, String> {
        if self.is_zero() {
            return Err("The inverse of the zero polynomial does not exist.".to_string());
        }

        // Create the modulus polynomial x^n - 1
        let n = self.coeffs.len();
        let modulus = ConvPoly {
            coeffs: {
                let mut coeffs = vec![0; n + 1];
                (coeffs[0], coeffs[n]) = (-1, 1);
                coeffs
            },
        };
        // Extend `self` to a polynomial in the ring Z[x]/(x^{n+1} - 1) to match the length of the modulus
        let self_ext = ConvPoly {
            coeffs: {
                let mut coeffs = self.coeffs.clone();
                coeffs.push(0);
                coeffs
            },
        };

        // After computing gcd, remove the extra coefficient from the polynomials to bring them back to the original ring
        let (mut gcd, mut s, _) = ConvPoly::extended_gcd(&self_ext, &modulus, m)?;
        gcd.coeffs.pop();
        s.coeffs.pop();

        if gcd != ConvPoly::constant(1, n) {
            return Err("The polynomial is not invertible in the given ring.".to_string());
        }

        Ok(s)
    }
}

// INTEGER ARITHMETIC

/// The Euclidean Algorithm. Return the greatest common divisor and a and b.
pub fn gcd(a: i32, b: i32) -> i32 {
    assert!(a != 0 || b != 0, "At least one of a and b must be non-zero");
    // Let a = bq + r (division algo). This algorithm works because gcd(a, b) = gcd(b, r) since
    // if a number divides a and b, then it divides a - bq = r. We can therefore
    // keep taking the remainder and shift until r is 0 (which is guaranteed to happen)
    let (mut old_r, mut r) = (a.clone().abs(), b.clone().abs());

    while r != 0 {
        (old_r, r) = (r, old_r % r);
    }

    old_r
}

/// The Extended Euclidean Algorithm. Returns (gcd, x, y) such that ax + by = gcd(a, b).
/// If negative inputs are provided, the algorithm will use their absolute values.
pub fn extended_gcd(a: i32, b: i32) -> (i32, i32, i32) {
    assert!(a != 0 || b != 0, "At least one of a and b must be non-zero");
    // Initial state
    // a = 1a + 0b  -->  old_r = a(old_x) + b(old_y)  so `old_r` is a linear combination of a,b
    // b = 0a + 1b  -->  r = ax + by                  so `r` is also a linear combination of a,b
    //
    // Update step
    // Let old_r = rq + new_r (division algo). Then because of the above, new_r = old_r - rq is still
    // a linear combination of a,b with new_x = old_x - xq and new_y = old_y - yq. By induction, we can
    // continue assigning new_r to r like this until r = 0 (which we know will happen by the standard
    // Euclidean Algorithm) and be left with Bézout coefficients.
    let (mut old_r, mut old_x, mut old_y) = (a.clone().abs(), 1, 0);
    let (mut r, mut x, mut y) = (b.clone().abs(), 0, 1);

    while r != 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r % r);
        (old_x, x) = (x, old_x - x * q);
        (old_y, y) = (y, old_y - y * q);
    }

    (old_r, old_x, old_y)
}

/// Returns the multiplicative inverse of `a` within the unit group (Z/mZ)*. Returns an error if no
/// such inverse exists (i.e. if `a` is not relatively prime to `m`, and therefore not a member of the group).
pub fn inverse(a: i32, m: i32) -> Result<i32, String> {
    assert!(m > 0, "Modulus `m` must be a positive integer");

    if a == 0 {
        return Err("The multiplicative inverse of 0 does not exist.".to_string());
    }

    let (gcd, x, _) = extended_gcd(a.rem_euclid(m), m);

    if gcd != 1 {
        return Err(
            "`a` only has a multiplicative inverse (mod m) if it is relatively prime to m."
                .to_string(),
        );
    }

    Ok(x.rem_euclid(m))
}
