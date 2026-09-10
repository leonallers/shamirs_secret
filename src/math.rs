fn mod_mul(a: u64, b: u64, q: u64) -> u64 {
    // eigentlich num-bigint nutzen für echte krypto
    let a = u128::from(a);
    let b = u128::from(b);
    let q = u128::from(q);
    ((a * b) % q) as u64
}

fn mod_add(a: u64, b: u64, q: u64) -> u64 {
    let a = u128::from(a);
    let b = u128::from(b);
    let q = u128::from(q);
    ((a + b) % q) as u64
}

fn mod_sub(a: u64, b: u64, q: u64) -> u64 {
    let a = a % q;
    let b = b % q;
    if a >= b { a - b } else { q - (b - a) }
}

fn mod_pow(base: u64, exponent: u64, q: u64) -> u64 {
    12
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2^64 - 59: die größte Primzahl, die in u64 passt
    const LARGE_PRIME: u64 = 18_446_744_073_709_551_557;

    // -- Tests für mod_mul
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
        fn returns_zero_when_factor_is_zero() {
            assert_eq!(mod_mul(0, 5, 7), 0);
        }

        #[test]
        fn handles_max_values_without_overflow() {
            // 2^3 ≡ 1 mod 7, also 2^64 ≡ 2 und u64::MAX = 2^64 - 1 ≡ 1
            assert_eq!(mod_mul(u64::MAX, u64::MAX, 7), 1);
        }

        #[test]
        fn handles_large_result_near_modulus() {
            // 2 * (q - 1) = 2q - 2 ≡ q - 2; das Zwischenprodukt passt nicht in u64
            assert_eq!(mod_mul(LARGE_PRIME - 1, 2, LARGE_PRIME), LARGE_PRIME - 2);
        }

        #[test]
        fn squares_minus_one_to_one() {
            // q - 1 ≡ -1, also (q - 1)² ≡ 1 mod q
            assert_eq!(mod_mul(LARGE_PRIME - 1, LARGE_PRIME - 1, LARGE_PRIME), 1);
        }
    }

    // -- Tests für mod_add
    mod mod_add {
        use super::*;

        #[test]
        fn adds_small_numbers() {
            assert_eq!(mod_add(5, 4, 7), 2);
        }

        #[test]
        fn adds_large_b_to_small_a() {
            assert_eq!(mod_add(u64::MAX, u64::MAX, LARGE_PRIME), 116);
        }

        #[test]
        fn adds_q_minus_one_to_one() {
            assert_eq!(mod_add(LARGE_PRIME - 1, 1, LARGE_PRIME), 0)
        }
    }

    // -- Tests für mod_sub
    mod mod_sub {
        use super::*;

        #[test]
        fn sub_small_numbers() {
            assert_eq!(mod_sub(5, 3, 7), 2);
        }

        #[test]
        fn sub_a_with_larger_b() {
            assert_eq!(mod_sub(3, 5, 7), 5);
        }
    }

    // -- Tests für mod_pow
    mod mod_pow {}
}
