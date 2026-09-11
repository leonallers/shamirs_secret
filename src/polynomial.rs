//! Polynome über dem Primkörper `F_q`.
//!
//! Beim Secret Sharing ist der konstante Term das Geheimnis, und jeder Share
//! ist die Auswertung des Polynoms an einer Stelle `x != 0`.

use crate::math::{mod_add, mod_mul};

/// Ein Polynom, gespeichert als Liste seiner Koeffizienten.
///
/// Der Koeffizient an Position `i` gehört zu `x^i`, Position 0 ist also der
/// konstante Term. Der Grad ist die Anzahl der Koeffizienten minus eins.
///
/// ```text
/// Koeffizienten [7, 3]    entsprechen 7 + 3x
/// Koeffizienten [7, 3, 2] entsprechen 7 + 3x + 2x²
/// ```
#[derive(Debug, PartialEq)]
pub(crate) struct Polynomial {
    coefficients: Vec<u64>,
}

impl Polynomial {
    /// Erzeugt ein Polynom aus seinen Koeffizienten.
    ///
    /// Gibt `None` zurück, wenn `coefficients` leer ist.
    pub(crate) fn new(coefficients: Vec<u64>) -> Option<Self> {
        if coefficients.is_empty() {
            None
        } else {
            Some(Self { coefficients })
        }
    }

    /// Wertet das Polynom an der Stelle `x` modulo `q` aus.
    ///
    /// Nutzt das Horner-Schema und kommt dadurch ohne Potenzen aus.
    ///
    /// # Panics
    ///
    /// Panics, wenn `q == 0`.
    pub(crate) fn evaluate(&self, x: u64, q: u64) -> u64 {
        let mut accu = 0;
        for coefficient in self.coefficients.iter().rev() {
            accu = mod_mul(accu, x, q);
            accu = mod_add(accu, *coefficient, q);
        }
        accu
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2^64 - 59: die größte Primzahl, die in u64 passt.
    const LARGE_PRIME: u64 = 18_446_744_073_709_551_557;

    mod new {
        use super::*;

        #[test]
        fn returns_none_for_empty_coefficients() {
            assert!(Polynomial::new(vec![]).is_none());
        }

        #[test]
        fn returns_some_for_non_empty_coefficients() {
            assert!(Polynomial::new(vec![7, 3]).is_some());
        }
    }

    mod evaluate {
        use super::*;

        #[test]
        fn evaluates_linear_polynomial() {
            // 7 + 3x mod 11: x = 1 → 10, x = 2 → 13 ≡ 2, x = 3 → 16 ≡ 5
            let polynomial = Polynomial::new(vec![7, 3]).expect("Koeffizienten sind nicht leer");
            for (x, expected) in [(1, 10), (2, 2), (3, 5)] {
                assert_eq!(polynomial.evaluate(x, 11), expected, "x = {x}");
            }
        }

        #[test]
        fn evaluates_quadratic_polynomial() {
            // 7 + 3 * 2 + 2 * 2² = 21 ≡ 10 mod 11
            let polynomial = Polynomial::new(vec![7, 3, 2]).expect("Koeffizienten sind nicht leer");
            assert_eq!(polynomial.evaluate(2, 11), 10);
        }

        #[test]
        fn returns_constant_term_at_zero_for_linear_polynomial() {
            // Bei x = 0 fallen alle Terme mit x weg
            let polynomial = Polynomial::new(vec![7, 3]).expect("Koeffizienten sind nicht leer");
            assert_eq!(polynomial.evaluate(0, 11), 7);
        }

        #[test]
        fn returns_constant_term_at_zero_for_quadratic_polynomial() {
            // Bei x = 0 fallen alle Terme mit x weg
            let polynomial = Polynomial::new(vec![7, 3, 2]).expect("Koeffizienten sind nicht leer");
            assert_eq!(polynomial.evaluate(0, 11), 7);
        }

        #[test]
        fn returns_same_value_for_constant_polynomial() {
            // Ohne Terme mit x ist das Ergebnis für jedes x gleich
            let polynomial = Polynomial::new(vec![42]).expect("Koeffizienten sind nicht leer");
            for x in [0, 1, 5, 96, 1_000] {
                assert_eq!(polynomial.evaluate(x, 97), 42, "x = {x}");
            }
        }

        #[test]
        fn sums_coefficients_at_x_equals_one() {
            // Bei x = 1 werden alle Koeffizienten addiert: 3 * (q - 1) = 3q - 3 ≡ q - 3
            let polynomial =
                Polynomial::new(vec![LARGE_PRIME - 1, LARGE_PRIME - 1, LARGE_PRIME - 1])
                    .expect("Koeffizienten sind nicht leer");
            assert_eq!(polynomial.evaluate(1, LARGE_PRIME), LARGE_PRIME - 3);
        }

        #[test]
        fn alternates_signs_at_x_equals_minus_one() {
            // x = q - 1 ≡ -1: (-1) + (-1)(-1) + (-1)(-1)² = -1 + 1 - 1 = -1 ≡ q - 1
            let polynomial =
                Polynomial::new(vec![LARGE_PRIME - 1, LARGE_PRIME - 1, LARGE_PRIME - 1])
                    .expect("Koeffizienten sind nicht leer");
            assert_eq!(
                polynomial.evaluate(LARGE_PRIME - 1, LARGE_PRIME),
                LARGE_PRIME - 1
            );
        }
    }
}
