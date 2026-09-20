//! Aufteilung eines Geheimnisses in Shares und Rekonstruktion daraus.

use crate::error::{Error, Result};
use crate::math::{mod_add, mod_inv, mod_mul, mod_sub};
use crate::polynomial::Polynomial;
use rand::Rng;
use std::collections::HashSet;

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

/// Rekonstruiert das Geheimnis aus einer Menge von Shares.
///
/// Erwartet werden mindestens so viele Shares, wie beim Aufteilen als
/// `threshold` angegeben wurden. Welche es sind, spielt keine Rolle, und
/// überzählige Shares ändern das Ergebnis nicht.
///
/// `q` muss derselbe Modulus sein wie beim Aufteilen.
///
/// # Warnung
///
/// Mit **zu wenigen** Shares schlägt die Funktion nicht fehl, sondern
/// liefert einen anderen, praktisch zufälligen Wert. Sie kennt `threshold`
/// nicht und kann den Fall deshalb nicht erkennen. Das ist keine Schwäche,
/// sondern die Eigenschaft, auf der das Verfahren beruht: Wer zu wenige
/// Shares hat, erfährt nichts über das Geheimnis.
///
/// # Errors
///
/// - [`Error::NoShares`], wenn `shares` leer ist
/// - [`Error::InvalidShare`], wenn ein `x` gleich 0 ist oder ein `x` oder `y`
///   nicht kleiner als `q` ist
/// - [`Error::DuplicateShareX`], wenn zwei Shares dasselbe `x` haben
///
/// # Examples
///
/// ```
/// use shamir::{reconstruct, Share};
///
/// // Punkte des Polynoms 7 + 3x modulo 11
/// let shares = [Share { x: 1, y: 10 }, Share { x: 3, y: 5 }];
/// assert_eq!(reconstruct(&shares, 11)?, 7);
/// # Ok::<(), shamir::Error>(())
/// ```
pub fn reconstruct(shares: &[Share], q: u64) -> Result<u64> {
    if shares.is_empty() {
        return Err(Error::NoShares);
    }
    for share in shares {
        if share.x == 0 || share.x >= q || share.y >= q {
            return Err(Error::InvalidShare {
                x: share.x,
                y: share.y,
            });
        }
    }
    let mut seen: HashSet<u64> = HashSet::new();
    for share in shares {
        if !seen.insert(share.x) {
            return Err(Error::DuplicateShareX { x: share.x });
        }
    }

    let mut secret: u64 = 0;
    for current in shares {
        let mut weight: u64 = 1;
        for other in shares {
            if current.x == other.x {
                continue;
            }
            let denominator = mod_sub(other.x, current.x, q);
            let inverse =
                mod_inv(denominator, q).ok_or(Error::DuplicateShareX { x: denominator })?;
            weight = mod_mul(weight, other.x, q);
            weight = mod_mul(weight, inverse, q);
        }
        secret = mod_add(secret, mod_mul(weight, current.y, q), q);
    }
    Ok(secret)
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

    mod reconstruct {
        use super::*;

        /// 2^64 - 59: die größte Primzahl, die in u64 passt.
        const LARGE_PRIME: u64 = 18_446_744_073_709_551_557;

        // Handrechnung: Das Polynom 7 + 3x modulo 11 hat die Punkte
        // (1, 10), (2, 2) und (3, 5). Das Geheimnis ist 7.

        #[test]
        fn reconstructs_secret_from_first_and_third_share() {
            let shares = [Share { x: 1, y: 10 }, Share { x: 3, y: 5 }];
            assert_eq!(reconstruct(&shares, 11).expect("gültige Shares"), 7);
        }

        #[test]
        fn reconstructs_secret_from_second_and_third_share() {
            let shares = [Share { x: 2, y: 2 }, Share { x: 3, y: 5 }];
            assert_eq!(reconstruct(&shares, 11).expect("gültige Shares"), 7);
        }

        #[test]
        fn reconstructs_secret_from_more_shares_than_needed() {
            // Überzählige Shares liegen auf derselben Kurve und ändern nichts
            let shares = [
                Share { x: 1, y: 10 },
                Share { x: 2, y: 2 },
                Share { x: 3, y: 5 },
            ];
            assert_eq!(reconstruct(&shares, 11).expect("gültige Shares"), 7);
        }

        #[test]
        fn returns_y_for_a_single_share() {
            // Ohne weitere Shares bleibt das Gewicht 1, das Ergebnis ist y selbst.
            // Genau das ist der Fall threshold = 1.
            let shares = [Share { x: 1, y: 7 }];
            assert_eq!(reconstruct(&shares, 11).expect("gültige Shares"), 7);
        }

        #[test]
        fn roundtrip_with_the_first_threshold_shares() {
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 2, 5, 97, &mut rng).expect("gültige Parameter");
            assert_eq!(reconstruct(&shares[..2], 97).expect("gültige Shares"), 42);
        }

        #[test]
        fn roundtrip_with_any_selection_of_shares() {
            // Welche Shares genommen werden, darf keine Rolle spielen
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 3, 5, 97, &mut rng).expect("gültige Parameter");

            let selection = [shares[0], shares[2], shares[4]];
            assert_eq!(reconstruct(&selection, 97).expect("gültige Shares"), 42);

            let selection = [shares[1], shares[3], shares[4]];
            assert_eq!(reconstruct(&selection, 97).expect("gültige Shares"), 42);
        }

        #[test]
        fn roundtrip_with_all_shares() {
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 3, 5, 97, &mut rng).expect("gültige Parameter");
            assert_eq!(reconstruct(&shares, 97).expect("gültige Shares"), 42);
        }

        #[test]
        fn does_not_reconstruct_secret_with_too_few_shares() {
            // Die Sicherheitseigenschaft: Unterhalb von threshold kommt irgendein
            // Wert heraus, kein Fehler. Großes q, damit nicht zufällig das
            // Geheimnis getroffen wird.
            let mut rng = StdRng::seed_from_u64(42);
            let shares = split(42, 3, 5, LARGE_PRIME, &mut rng).expect("gültige Parameter");
            let result = reconstruct(&shares[..2], LARGE_PRIME).expect("gültige Shares");
            assert_ne!(result, 42);
        }

        #[test]
        fn returns_error_for_empty_shares() {
            assert_eq!(reconstruct(&[], 11), Err(Error::NoShares));
        }

        #[test]
        fn returns_error_for_share_with_x_zero() {
            // An der Stelle 0 steht das Geheimnis, ein solcher Share ist ungültig
            let shares = [Share { x: 0, y: 5 }];
            assert_eq!(
                reconstruct(&shares, 11),
                Err(Error::InvalidShare { x: 0, y: 5 })
            );
        }

        #[test]
        fn returns_error_for_share_with_x_not_less_than_q() {
            let shares = [Share { x: 11, y: 5 }];
            assert_eq!(
                reconstruct(&shares, 11),
                Err(Error::InvalidShare { x: 11, y: 5 })
            );
        }

        #[test]
        fn returns_error_for_share_with_y_not_less_than_q() {
            let shares = [Share { x: 1, y: 11 }];
            assert_eq!(
                reconstruct(&shares, 11),
                Err(Error::InvalidShare { x: 1, y: 11 })
            );
        }

        #[test]
        fn returns_error_for_duplicate_x() {
            // Zwei Shares mit demselben x: Der Nenner wäre 0, es gäbe kein Inverses
            let shares = [Share { x: 1, y: 10 }, Share { x: 1, y: 4 }];
            assert_eq!(
                reconstruct(&shares, 11),
                Err(Error::DuplicateShareX { x: 1 })
            );
        }
    }
}
