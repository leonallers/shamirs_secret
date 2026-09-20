//! Enthält die Aufteilung eines Geheimnisses in Shares.

use crate::error::{Error, Result};
use crate::polynomial::Polynomial;
use rand::Rng;

/// Ein Share ist ein Punkt auf dem geheimen Polynom.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Share {
    /// `x` ist die Nummer des Shares und ist niemals 0.
    /// `x == 0` wäre das Geheimnis selbst.
    pub x: u64,

    /// `y` ist der Wert des Polynoms an der Stelle `x`.
    pub y: u64,
}

/// Teilt ein Geheimnis in `shares` Shares auf, von denen `threshold` viele
/// zur Rekonstruktion genügen.
///
/// Das Geheimnis wird zum konstanten Term eines zufälligen Polynoms vom Grad
/// `threshold - 1`. Jeder Share ist ein Punkt auf diesem Polynom, ausgewertet
/// an den Stellen 1 bis `shares`. Das Polynom selbst wird nicht zurückgegeben.
///
/// Mit weniger als `threshold` Shares ist jeder mögliche Wert für das
/// Geheimnis gleich wahrscheinlich.
///
/// - `secret`: das Geheimnis, muss kleiner als `q` sein
/// - `threshold`: Anzahl der Shares, die zur Rekonstruktion nötig sind
/// - `shares`: Anzahl der erzeugten Shares, muss kleiner als `q` sein
/// - `q`: der Modulus. Muss eine Primzahl sein, sonst schlägt die
///   Rekonstruktion fehl. Das wird hier nicht geprüft.
/// - `rng`: die Quelle der Zufallszahlen. Für den produktiven Einsatz muss
///   sie kryptografisch sicher sein, etwa `rand::rng()`.
///
/// # Errors
///
/// - [`Error::ThresholdZero`], wenn `threshold` 0 ist
/// - [`Error::ThresholdLargerShares`], wenn `threshold` größer als `shares` ist
/// - [`Error::SharesNotLessThanQ`], wenn `shares` nicht kleiner als `q` ist
/// - [`Error::SecretNotLessThanQ`], wenn `secret` nicht kleiner als `q` ist
///
/// # Examples
///
/// ```
/// let mut rng = rand::rng();
/// let shares = shamir::split(42, 2, 5, 97, &mut rng)?;
///
/// assert_eq!(shares.len(), 5);
/// assert_eq!(shares[0].x, 1);
/// # Ok::<(), shamir::Error>(())
/// ```
pub fn split<R: Rng + ?Sized>(
    secret: u64,
    threshold: usize,
    shares: usize,
    q: u64,
    rng: &mut R,
) -> Result<Vec<Share>> {
    if threshold == 0 {
        return Err(Error::ThresholdZero);
    }

    let threshold_u64 = u64::try_from(threshold).expect("usize passt auf 64-Bit-Systemen in u64");
    let shares_u64 = u64::try_from(shares).expect("usize passt auf 64-Bit-Systemen in u64");

    if threshold > shares {
        return Err(Error::ThresholdLargerShares {
            threshold: threshold_u64,
            shares: shares_u64,
        });
    }
    if shares_u64 >= q {
        return Err(Error::SharesNotLessThanQ {
            shares: shares_u64,
            q,
        });
    }
    if secret >= q {
        return Err(Error::SecretNotLessThanQ { secret, q });
    }

    let polynomial = Polynomial::new_random(secret, threshold, q, rng).ok_or(Error::Internal)?;
    let mut result: Vec<Share> = Vec::with_capacity(shares);
    for x in 1..=shares_u64 {
        result.push(Share {
            x,
            y: polynomial.evaluate(x, q),
        })
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    mod split {
        use super::*;

        #[test]
        fn returns_as_many_shares_as_requested() {
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 2, 5, 97, &mut rng).expect("gültige Parameter");
            assert_eq!(shares.len(), 5);
        }

        #[test]
        fn numbers_shares_from_one() {
            // x ist die Nummer des Shares, also 1, 2, 3, ... und nie 0,
            // denn an der Stelle 0 steht das Geheimnis selbst.
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 2, 5, 97, &mut rng).expect("gültige Parameter");
            for (index, share) in shares.iter().enumerate() {
                let expected = u64::try_from(index + 1).expect("Index passt in u64");
                assert_eq!(share.x, expected);
            }
        }

        #[test]
        fn returns_secret_as_y_for_threshold_one() {
            // Bei threshold 1 hat das Polynom keinen Term mit x,
            // der Wert ist also an jeder Stelle das Geheimnis.
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 1, 3, 97, &mut rng).expect("gültige Parameter");
            for share in &shares {
                assert_eq!(share.y, 42, "x = {}", share.x);
            }
        }

        #[test]
        fn returns_points_of_the_generated_polynomial() {
            // Gleicher Startwert heißt gleiche Koeffizienten: Das Polynom hier ist
            // dasselbe, das split intern erzeugt. Die y-Werte aus split müssen
            // deshalb mit den eigenen Auswertungen übereinstimmen.
            let mut rng_split = StdRng::seed_from_u64(42);
            let mut rng_polynomial = StdRng::seed_from_u64(42);

            let shares = split(42, 2, 3, 97, &mut rng_split).expect("gültige Parameter");
            let polynomial = Polynomial::new_random(42, 2, 97, &mut rng_polynomial)
                .expect("threshold > 0 und secret < q");

            for share in &shares {
                assert_eq!(share.y, polynomial.evaluate(share.x, 97), "x = {}", share.x);
            }
        }

        #[test]
        fn returns_different_y_values_for_higher_threshold() {
            // Ab threshold 2 hängt y von x ab, die Werte dürfen nicht alle gleich sein
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 3, 5, 97, &mut rng).expect("gültige Parameter");
            let first = shares[0].y;
            assert!(shares.iter().any(|share| share.y != first));
        }

        #[test]
        fn returns_error_for_threshold_zero() {
            let mut rng = StdRng::seed_from_u64(42);
            let result = split(42, 0, 5, 97, &mut rng);
            assert_eq!(result, Err(Error::ThresholdZero));
        }

        #[test]
        fn returns_error_when_threshold_is_larger_than_shares() {
            let mut rng = StdRng::seed_from_u64(42);
            let result = split(42, 10, 5, 97, &mut rng);
            assert_eq!(
                result,
                Err(Error::ThresholdLargerShares {
                    threshold: 10,
                    shares: 5
                })
            );
        }

        #[test]
        fn returns_error_when_shares_are_not_less_than_q() {
            // secret und threshold sind gültig, damit nur diese Prüfung greift
            let mut rng = StdRng::seed_from_u64(42);
            let result = split(1, 2, 5, 4, &mut rng);
            assert_eq!(result, Err(Error::SharesNotLessThanQ { shares: 5, q: 4 }));
        }

        #[test]
        fn returns_error_when_secret_is_not_less_than_q() {
            // shares 3 ist kleiner als q 4, damit nur das Geheimnis die Prüfung auslöst
            let mut rng = StdRng::seed_from_u64(42);
            let result = split(4, 2, 3, 4, &mut rng);
            assert_eq!(result, Err(Error::SecretNotLessThanQ { secret: 4, q: 4 }));
        }

        #[test]
        fn accepts_threshold_equal_to_shares() {
            // Grenzfall: Dann werden alle Shares zur Rekonstruktion gebraucht
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 3, 3, 97, &mut rng).expect("gültige Parameter");
            assert_eq!(shares.len(), 3);
        }
    }
}
