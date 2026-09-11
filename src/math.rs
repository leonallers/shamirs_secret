//! Modulare Arithmetik über dem Primkörper `F_q`.
//!
//! Das Fundament für Shamir's Secret Sharing: Addition, Subtraktion,
//! Multiplikation, Potenz und Inverses modulo einer Primzahl `q`.
//!
//! Alle Funktionen akzeptieren beliebige `u64`-Eingaben und reduzieren sie
//! selbst modulo `q`. Ergebnisse liegen immer im Bereich `0..q`.
//!
//! **Lernprojekt:** Die Implementierung ist auf `u64` beschränkt und nicht
//! constant-time. Für produktive Kryptografie gehören geprüfte Bibliotheken
//! mit Big-Integer-Typen verwendet.

/// Berechnet `(a + b) mod q`.
///
/// Rechnet intern in `u128`, weil `a + b` bei großem `q` über `u64`
/// hinausgehen kann.
///
/// # Panics
///
/// Panics, wenn `q == 0`.
pub(crate) fn mod_add(a: u64, b: u64, q: u64) -> u64 {
    let sum = u128::from(a) + u128::from(b);
    // Kann nicht abschneiden: Das Ergebnis ist kleiner als q, und q passt in u64.
    (sum % u128::from(q)) as u64
}

/// Berechnet `(a - b) mod q`.
///
/// Kommt ohne `u128` aus: Nach der Reduktion sind `a` und `b` kleiner als `q`,
/// und kein Zwischenergebnis wird größer als `q`.
///
/// # Panics
///
/// Panics, wenn `q == 0`.
pub(crate) fn mod_sub(a: u64, b: u64, q: u64) -> u64 {
    let a = a % q;
    let b = b % q;
    if a >= b { a - b } else { q - (b - a) }
}

/// Berechnet `(a * b) mod q`.
///
/// Rechnet intern in `u128`: Das Produkt zweier `u64` ist kleiner als 2^128
/// und passt immer.
///
/// # Panics
///
/// Panics, wenn `q == 0`.
pub(crate) fn mod_mul(a: u64, b: u64, q: u64) -> u64 {
    let product = u128::from(a) * u128::from(b);
    // Kann nicht abschneiden: Das Ergebnis ist kleiner als q, und q passt in u64.
    (product % u128::from(q)) as u64
}

/// Berechnet `base^exponent mod q`.
///
/// Wendet zwei Regeln an, bis der Exponent 0 ist:
///
/// - Exponent ungerade: Ergebnis mit der Basis multiplizieren, Exponent − 1
/// - Exponent gerade: Basis quadrieren, Exponent halbieren
///
/// Dadurch reichen O(log exponent) Multiplikationen statt `exponent` vieler.
/// Nach Konvention gilt `0^0 = 1`.
///
/// # Panics
///
/// Panics, wenn `q == 0`.
pub(crate) fn mod_pow(mut base: u64, mut exponent: u64, q: u64) -> u64 {
    // 1 % q statt 1: Bei q = 1 muss auch base^0 den Wert 0 ergeben, und
    // bei q = 0 panict die Funktion so unabhängig vom Exponenten.
    let mut result = 1 % q;
    while exponent != 0 {
        if !exponent.is_multiple_of(2) {
            result = mod_mul(result, base, q);
            exponent -= 1;
        } else {
            base = mod_mul(base, base, q);
            exponent /= 2;
        }
    }
    result
}

/// Berechnet das multiplikative Inverse von `a` modulo `q`.
///
/// Gesucht ist das `x` mit `a * x ≡ 1 (mod q)`. Nach dem kleinen Satz von
/// Fermat ist das `a^(q-2) mod q`.
///
/// Gibt `None` zurück, wenn `a ≡ 0 (mod q)`, denn die Null hat kein Inverses.
///
/// # Vorbedingung
///
/// `q` muss eine Primzahl sein. Das wird aus Kostengründen nicht geprüft.
/// Bei zusammengesetztem `q` liefert Fermat im Release-Build still ein
/// falsches Ergebnis; in Debug-Builds fängt eine Prüfung das ab.
///
/// # Panics
///
/// Panics, wenn `q == 0`. In Debug-Builds zusätzlich, wenn das berechnete
/// Ergebnis kein Inverses ist, `q` also keine Primzahl sein kann.
pub(crate) fn mod_inv(a: u64, q: u64) -> Option<u64> {
    let a = a % q;
    // Vor q - 2 prüfen: Bei q = 1 ist a hier immer 0, q - 2 läuft also nie unter null.
    if a == 0 {
        return None;
    }
    let inverse = mod_pow(a, q - 2, q);
    debug_assert_eq!(
        mod_mul(a, inverse, q),
        1,
        "kein gültiges Inverses, q ist vermutlich keine Primzahl"
    );
    Some(inverse)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2^64 - 59: die größte Primzahl, die in u64 passt.
    const LARGE_PRIME: u64 = 18_446_744_073_709_551_557;

    mod mod_add {
        use super::*;

        #[test]
        fn adds_small_numbers() {
            // 5 + 4 = 9, 9 mod 7 = 2
            assert_eq!(mod_add(5, 4, 7), 2);
        }

        #[test]
        fn reduces_inputs_larger_than_modulus() {
            // 10 + 12 = 22, 22 mod 7 = 1
            assert_eq!(mod_add(10, 12, 7), 1);
        }

        #[test]
        fn wraps_around_at_modulus() {
            // (q - 1) + 1 = q ≡ 0
            assert_eq!(mod_add(LARGE_PRIME - 1, 1, LARGE_PRIME), 0);
        }

        #[test]
        fn handles_max_values_without_overflow() {
            // u64::MAX = 2^64 - 1 = q + 58, also u64::MAX ≡ 58 und 58 + 58 = 116
            assert_eq!(mod_add(u64::MAX, u64::MAX, LARGE_PRIME), 116);
        }
    }

    mod mod_sub {
        use super::*;

        #[test]
        fn subtracts_small_numbers() {
            // 5 - 3 = 2
            assert_eq!(mod_sub(5, 3, 7), 2);
        }

        #[test]
        fn wraps_around_when_b_is_larger() {
            // 3 - 5 = -2 ≡ 7 - 2 = 5
            assert_eq!(mod_sub(3, 5, 7), 5);
        }

        #[test]
        fn returns_zero_for_equal_inputs() {
            // Regressionstest: Eine frühere Version lieferte hier q statt 0.
            assert_eq!(mod_sub(4, 4, 7), 0);
        }

        #[test]
        fn reduces_inputs_larger_than_modulus() {
            // 12 ≡ 5, 5 - 3 = 2
            assert_eq!(mod_sub(12, 3, 7), 2);
        }

        #[test]
        fn wraps_around_at_large_modulus() {
            // 0 - 1 = -1 ≡ q - 1
            assert_eq!(mod_sub(0, 1, LARGE_PRIME), LARGE_PRIME - 1);
        }

        #[test]
        fn undoes_addition() {
            // (a + b) - b muss wieder a ergeben, für a < q
            let cases = [
                (0, 0),
                (5, 3),
                (LARGE_PRIME - 1, LARGE_PRIME - 1),
                (123_456_789, LARGE_PRIME - 2),
            ];
            for (a, b) in cases {
                let sum = mod_add(a, b, LARGE_PRIME);
                assert_eq!(mod_sub(sum, b, LARGE_PRIME), a, "a = {a}, b = {b}");
            }
        }
    }

    mod mod_mul {
        use super::*;

        #[test]
        fn multiplies_small_numbers() {
            // 3 * 4 = 12, 12 mod 7 = 5
            assert_eq!(mod_mul(3, 4, 7), 5);
        }

        #[test]
        fn reduces_inputs_larger_than_modulus() {
            // 10 * 10 = 100, 100 mod 7 = 2
            assert_eq!(mod_mul(10, 10, 7), 2);
        }

        #[test]
        fn returns_zero_when_a_factor_is_zero() {
            assert_eq!(mod_mul(0, 5, 7), 0);
        }

        #[test]
        fn handles_max_values_without_overflow() {
            // 2^3 ≡ 1 mod 7, also 2^64 ≡ 2 und u64::MAX = 2^64 - 1 ≡ 1
            assert_eq!(mod_mul(u64::MAX, u64::MAX, 7), 1);
        }

        #[test]
        fn handles_product_beyond_u64() {
            // 2 * (q - 1) = 2q - 2 ≡ q - 2; das Zwischenprodukt passt nicht in u64
            assert_eq!(mod_mul(LARGE_PRIME - 1, 2, LARGE_PRIME), LARGE_PRIME - 2);
        }

        #[test]
        fn squares_minus_one_to_one() {
            // q - 1 ≡ -1, also (q - 1)² ≡ 1
            assert_eq!(mod_mul(LARGE_PRIME - 1, LARGE_PRIME - 1, LARGE_PRIME), 1);
        }
    }

    mod mod_pow {
        use super::*;

        #[test]
        fn computes_small_example() {
            // Fermat: 3^6 ≡ 1 mod 7, also 3^12 ≡ 1 und 3^13 ≡ 3
            assert_eq!(mod_pow(3, 13, 7), 3);
        }

        #[test]
        fn computes_exact_power_when_modulus_is_larger() {
            // 3^13 = 1_594_323 < q, das Modulo reduziert also nichts
            assert_eq!(mod_pow(3, 13, LARGE_PRIME), 1_594_323);
        }

        #[test]
        fn returns_one_for_exponent_zero() {
            assert_eq!(mod_pow(5, 0, 7), 1);
        }

        #[test]
        fn returns_reduced_base_for_exponent_one() {
            // 10 mod 7 = 3
            assert_eq!(mod_pow(10, 1, 7), 3);
        }

        #[test]
        fn returns_zero_for_base_zero() {
            assert_eq!(mod_pow(0, 5, 7), 0);
        }

        #[test]
        fn treats_zero_to_the_zero_as_one() {
            // Konvention, siehe Doku-Kommentar
            assert_eq!(mod_pow(0, 0, 7), 1);
        }

        #[test]
        fn returns_zero_for_modulus_one() {
            // Modulo 1 ist alles 0, auch base^0
            assert_eq!(mod_pow(5, 0, 1), 0);
            assert_eq!(mod_pow(5, 3, 1), 0);
        }

        #[test]
        fn satisfies_fermat_for_large_prime() {
            // Kleiner Satz von Fermat: a^(p - 1) ≡ 1 mod p, wenn p prim ist und a nicht teilt.
            // Anders als bei Basis 1 oder q - 1 entstehen hier über viele Runden
            // "krumme" Zwischenwerte nahe am Modulus.
            for base in [2, 3, 123_456_789, LARGE_PRIME - 2] {
                assert_eq!(
                    mod_pow(base, LARGE_PRIME - 1, LARGE_PRIME),
                    1,
                    "base = {base}"
                );
            }
        }

        #[test]
        #[should_panic(expected = "divisor of zero")]
        fn panics_for_zero_modulus_even_with_exponent_zero() {
            // Durch 1 % q panict die Funktion bei q = 0 immer, nicht nur, wenn die Schleife läuft.
            mod_pow(5, 0, 0);
        }
    }

    mod mod_inv {
        use super::*;

        #[test]
        fn finds_inverse_for_small_prime() {
            // 2 * 6 = 12 ≡ 1 mod 11
            assert_eq!(mod_inv(2, 11), Some(6));
        }

        #[test]
        fn returns_one_for_one() {
            assert_eq!(mod_inv(1, LARGE_PRIME), Some(1));
        }

        #[test]
        fn returns_minus_one_for_minus_one() {
            // (q - 1)² ≡ (-1)² = 1
            assert_eq!(mod_inv(LARGE_PRIME - 1, LARGE_PRIME), Some(LARGE_PRIME - 1));
        }

        #[test]
        fn finds_inverse_of_two_for_large_prime() {
            // 2 * (q + 1) / 2 = q + 1 ≡ 1; q ist ungerade, die Division geht also glatt auf
            assert_eq!(mod_inv(2, LARGE_PRIME), Some((LARGE_PRIME + 1) / 2));
        }

        #[test]
        fn product_with_inverse_is_one() {
            for a in [2, 3, 12_345, LARGE_PRIME - 2, u64::MAX] {
                let inverse = mod_inv(a, LARGE_PRIME).expect("a ist nicht durch q teilbar");
                assert_eq!(mod_mul(a, inverse, LARGE_PRIME), 1, "a = {a}");
            }
        }

        #[test]
        fn returns_none_for_zero() {
            assert_eq!(mod_inv(0, LARGE_PRIME), None);
        }

        #[test]
        fn returns_none_for_multiple_of_modulus() {
            // q ≡ 0
            assert_eq!(mod_inv(LARGE_PRIME, LARGE_PRIME), None);
        }

        #[test]
        fn returns_none_for_modulus_one() {
            // Modulo 1 ist jedes a ≡ 0; prüft außerdem, dass q - 2 nicht unterläuft
            assert_eq!(mod_inv(100, 1), None);
        }

        #[test]
        #[cfg(debug_assertions)]
        #[should_panic(expected = "keine Primzahl")]
        fn detects_non_prime_modulus_in_debug_builds() {
            // Modulo 8 ist 3 zu sich selbst invers (3 * 3 = 9 ≡ 1),
            // Fermat liefert aber 3^6 ≡ 1, und 3 * 1 ≢ 1.
            mod_inv(3, 8);
        }
    }
}
