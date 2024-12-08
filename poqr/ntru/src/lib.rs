use ark_bn254::Fr;
use ark_poly::{polynomial::univariate::DensePolynomial, DenseUVPolynomial};
use rand::prelude::*;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

/// Generates a random polynomial given a degree bound
pub fn random_poly(deg_b: usize) -> DensePolynomial<Fr> {
    let mut rng = rand::thread_rng();
    let coeffs: Vec<Fr> = (0..deg_b + 1).map(|_| Fr::from(rng.gen::<i64>())).collect();
    DensePolynomial::from_coefficients_vec(coeffs)
}


#[cfg(test)]
mod tests {
    use ark_poly::Polynomial;

    use super::*;

    #[test]
    fn test_poly_gen() {
        let poly = random_poly(10);
        assert_eq!(poly.degree(), 10);
    }
}