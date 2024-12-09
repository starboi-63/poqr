use ark_bn254::Fr;
use ark_poly::polynomial::univariate::SparsePolynomial;
use rand::prelude::*;

//NOTE: Functionality implemented tonight (12/7) will likely be sectionalized into an ntru-utils
//file but i encourage not expanding into complex file structure until needed. For now, just
//keeping this in the base file :)

/// Generates a random polynomial given a degree bound
pub fn random_poly(deg_b: usize) -> SparsePolynomial<Fr> {
    let mut rng = rand::thread_rng();
    let coeffs: Vec<(usize, Fr)> = (0..deg_b + 1)
        .map(|i| (i, Fr::from(rng.gen_range(-1..=1))))
        .collect();
    SparsePolynomial::from_coefficients_vec(coeffs)
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
