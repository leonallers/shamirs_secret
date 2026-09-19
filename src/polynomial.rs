//! Polynome über dem Primkörper `F_q`.
//!
//! Beim Secret Sharing ist der konstante Term das Geheimnis, und jeder Share
//! ist die Auswertung des Polynoms an einer Stelle `x != 0`.

use crate::math::{mod_add, mod_mul};
use rand::{Rng, RngExt};

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

    /// Erzeugt ein zufälliges Polynom mit `secret` als konstantem Term.
    ///
    /// Beim Secret Sharing ist das der Kern des Verfahrens: Das Geheimnis
    /// steht an Position 0, alle weiteren Koeffizienten sind Zufall.
    ///
    /// - `secret`: das Geheimnis, wird zum konstanten Term
    /// - `threshold`: Anzahl der Koeffizienten, also Grad + 1. Entspricht
    ///   der Zahl an Shares, die später zur Rekonstruktion nötig sind.
    /// - `q`: der Modulus. Alle Koeffizienten werden gleichverteilt aus
    ///   `0..q` gezogen.
    /// - `rng`: die Quelle der Zufallszahlen. Für den produktiven Einsatz
    ///   muss sie kryptografisch sicher sein, etwa `rand::rng()`. In Tests
    ///   lässt sich ein Generator mit festem Startwert übergeben.
    ///
    /// Gibt `None` zurück, wenn `threshold == 0` oder `secret >= q` ist.
    /// Der Fall `q == 0` ist damit mit abgedeckt, die Funktion panict nie.
    pub(crate) fn new_random<R: Rng + ?Sized>(
        secret: u64,
        threshold: usize,
        q: u64,
        rng: &mut R,
    ) -> Option<Self> {
        if threshold == 0 || secret >= q {
            return None;
        }
        let mut coefficients = Vec::with_capacity(threshold);
        coefficients.push(secret);
        for _ in 1..threshold {
            coefficients.push(rng.random_range(0..q));
        }
        Self::new(coefficients)
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
    use rand::{SeedableRng, rngs::StdRng};

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

    mod new_random {
        use super::*;

        #[test]
        fn returns_none_for_threshold_zero() {
            let mut rng = StdRng::seed_from_u64(42);
            assert!(Polynomial::new_random(7, 0, 11, &mut rng).is_none());
        }

        #[test]
        fn returns_none_when_secret_equals_q() {
            let mut rng = StdRng::seed_from_u64(42);
            assert!(Polynomial::new_random(11, 3, 11, &mut rng).is_none());
        }

        #[test]
        fn returns_none_when_secret_is_larger_than_q() {
            let mut rng = StdRng::seed_from_u64(42);
            assert!(Polynomial::new_random(12, 3, 11, &mut rng).is_none());
        }

        #[test]
        fn returns_constant_polynomial_for_threshold_one() {
            // Ohne Terme mit x ist das Ergebnis für jedes x das Geheimnis
            let mut rng = StdRng::seed_from_u64(42);
            let polynomial =
                Polynomial::new_random(42, 1, 97, &mut rng).expect("threshold > 0 und secret < q");
            for x in [0, 1, 5, 96] {
                assert_eq!(polynomial.evaluate(x, 97), 42, "x = {x}");
            }
        }

        #[test]
        fn keeps_secret_as_constant_term() {
            // Die Auswertung an der Stelle 0 liefert immer den konstanten Term
            let mut rng = StdRng::seed_from_u64(42);
            let polynomial =
                Polynomial::new_random(42, 3, 97, &mut rng).expect("threshold > 0 und secret < q");
            assert_eq!(polynomial.evaluate(0, 97), 42);
        }

        #[test]
        fn creates_as_many_coefficients_as_threshold() {
            let mut rng = StdRng::seed_from_u64(42);
            let polynomial =
                Polynomial::new_random(42, 3, 97, &mut rng).expect("threshold > 0 und secret < q");
            assert_eq!(polynomial.coefficients.len(), 3);
        }

        #[test]
        fn creates_coefficients_smaller_than_q() {
            // Kleines q, viele Koeffizienten: Eine falsche Range fällt sofort auf
            let q = 7;
            let mut rng = StdRng::seed_from_u64(42);
            let polynomial =
                Polynomial::new_random(3, 10, q, &mut rng).expect("threshold > 0 und secret < q");
            for coefficient in &polynomial.coefficients {
                assert!(*coefficient < q, "Koeffizient = {coefficient}");
            }
        }

        #[test]
        fn creates_same_polynomial_for_same_seed() {
            let mut rng1 = StdRng::seed_from_u64(42);
            let mut rng2 = StdRng::seed_from_u64(42);
            let polynomial1 = Polynomial::new_random(22, 3, 97, &mut rng1);
            let polynomial2 = Polynomial::new_random(22, 3, 97, &mut rng2);
            assert_eq!(polynomial1, polynomial2);
        }

        #[test]
        fn creates_different_polynomials_for_different_seeds() {
            let mut rng1 = StdRng::seed_from_u64(42);
            let mut rng2 = StdRng::seed_from_u64(200);
            let polynomial1 = Polynomial::new_random(22, 3, LARGE_PRIME, &mut rng1);
            let polynomial2 = Polynomial::new_random(22, 3, LARGE_PRIME, &mut rng2);
            assert_ne!(polynomial1, polynomial2);
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
