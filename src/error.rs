//! Fehlertyp der Library.
//!
//! Alle Fehler stammen aus der Validierung der Parameter von
//! [`split`](crate::split) und [`reconstruct`](crate::reconstruct).

/// Fehler beim Aufteilen oder Rekonstruieren eines Geheimnisses.
///
/// Jede Variante trägt die beteiligten Werte mit, damit sich aus der
/// Meldung ablesen lässt, welche Bedingung verletzt wurde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// `threshold` war 0. Mindestens ein Share muss nötig sein.
    ThresholdZero,

    /// `threshold` war größer als die Anzahl der Shares. Das Geheimnis
    /// wäre dann nie rekonstruierbar.
    ThresholdLargerShares { threshold: u64, shares: u64 },

    /// Die Anzahl der Shares war nicht kleiner als `q`. Die x-Werte
    /// laufen von 1 bis zur Anzahl und müssen modulo `q` verschieden
    /// und ungleich 0 bleiben.
    SharesNotLessThanQ { shares: u64, q: u64 },

    /// Das Geheimnis war nicht kleiner als `q` und passt damit nicht
    /// in den Körper.
    SecretNotLessThanQ { secret: u64, q: u64 },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ThresholdZero => write!(f, "threshold darf nicht null sein"),
            Error::ThresholdLargerShares { threshold, shares } => {
                write!(
                    f,
                    "threshold {threshold} darf nicht größer als shares {shares} sein"
                )
            }
            Error::SharesNotLessThanQ { shares, q } => {
                write!(f, "shares {shares} muss kleiner als q {q} sein")
            }
            Error::SecretNotLessThanQ { secret, q } => {
                write!(f, "secret {secret} muss kleiner als q {q} sein")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Ergebnistyp der Library, mit [`Error`] als Fehlertyp.
pub type Result<T> = std::result::Result<T, Error>;
