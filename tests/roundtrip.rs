//! Integrationstest: benutzt die Library von außen, so wie ein fremdes
//! Programm es täte. Sichtbar ist hier nur die öffentliche API.

use shamir::{Error, Share, reconstruct, split};

/// 2^64 - 59: die größte Primzahl, die in u64 passt.
const Q: u64 = 18_446_744_073_709_551_557;

#[test]
fn recovers_secret_for_several_parameter_combinations() {
    let mut rng = rand::rng();
    let secret = 1_234_567_890;

    for (threshold, count) in [(1, 1), (2, 3), (3, 5), (5, 5), (2, 10)] {
        let shares = split(secret, threshold, count, Q, &mut rng).expect("gültige Parameter");
        assert_eq!(shares.len(), count);

        let recovered = reconstruct(&shares[..threshold], Q).expect("gültige Shares");
        assert_eq!(
            recovered, secret,
            "threshold = {threshold}, shares = {count}"
        );
    }
}

#[test]
fn recovers_secret_regardless_of_order_and_selection() {
    let mut rng = rand::rng();
    let shares = split(42, 3, 5, Q, &mut rng).expect("gültige Parameter");

    // Andere Auswahl, verdrehte Reihenfolge, überzählige Shares
    let selection = [shares[4], shares[0], shares[2]];
    assert_eq!(reconstruct(&selection, Q).expect("gültige Shares"), 42);
    assert_eq!(reconstruct(&shares, Q).expect("gültige Shares"), 42);
}

#[test]
fn recovers_boundary_secrets() {
    let mut rng = rand::rng();

    for secret in [0, 1, Q - 1] {
        let shares = split(secret, 3, 5, Q, &mut rng).expect("gültige Parameter");
        let recovered = reconstruct(&shares[..3], Q).expect("gültige Shares");
        assert_eq!(recovered, secret, "secret = {secret}");
    }
}

#[test]
fn numbers_shares_from_one() {
    let mut rng = rand::rng();
    let shares = split(42, 2, 3, Q, &mut rng).expect("gültige Parameter");

    // Die Felder sind von außen lesbar, etwa zum Speichern oder Versenden
    assert_eq!(shares[0].x, 1);
    assert_eq!(shares[2].x, 3);
}

#[test]
fn produces_different_shares_on_every_run() {
    // Echter Zufall: zweimal dasselbe Geheimnis, zweimal andere Punkte
    let mut rng = rand::rng();
    let first = split(42, 3, 5, Q, &mut rng).expect("gültige Parameter");
    let second = split(42, 3, 5, Q, &mut rng).expect("gültige Parameter");

    assert_ne!(first, second);
}

#[test]
fn reports_invalid_parameters() {
    let mut rng = rand::rng();

    assert_eq!(split(42, 0, 5, Q, &mut rng), Err(Error::ThresholdZero));
    assert_eq!(reconstruct(&[], Q), Err(Error::NoShares));
}

#[test]
fn reports_duplicate_shares() {
    let shares = [Share { x: 1, y: 10 }, Share { x: 1, y: 4 }];

    assert_eq!(
        reconstruct(&shares, 11),
        Err(Error::DuplicateShareX { x: 1 })
    );
}
