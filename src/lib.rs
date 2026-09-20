//! Shamir's Secret Sharing über einem Primkörper.
//!
//! Ein Geheimnis wird in `n` Shares aufgeteilt, von denen `t` beliebige
//! genügen, um es zurückzugewinnen. Mit weniger als `t` Shares ist jeder
//! mögliche Wert gleich wahrscheinlich, sie verraten also nichts.
//!
//! Das Geheimnis ist der konstante Term eines zufälligen Polynoms vom Grad
//! `t - 1`, jeder Share ein Punkt darauf. Die Rekonstruktion wertet dieses
//! Polynom über Lagrange-Interpolation an der Stelle 0 aus.
//!
//! # Examples
//!
//! ```
//! use shamir::{reconstruct, split};
//!
//! const Q: u64 = 65_537;
//!
//! let mut rng = rand::rng();
//! let shares = split(1234, 3, 5, Q, &mut rng)?;
//!
//! // Drei beliebige Shares genügen
//! let secret = reconstruct(&shares[..3], Q)?;
//! assert_eq!(secret, 1234);
//! # Ok::<(), shamir::Error>(())
//! ```
//!
//! # Einschränkungen
//!
//! Dies ist ein Lernprojekt und nicht für produktive Kryptografie gedacht.
//! Geheimnisse sind auf `u64` beschränkt, die Implementierung ist nicht
//! constant-time, und der Modulus `q` wird nicht auf Primalität geprüft.

mod error;
mod math;
mod polynomial;
mod sharing;

pub use crate::error::{Error, Result};
pub use crate::sharing::{Share, reconstruct, split};
