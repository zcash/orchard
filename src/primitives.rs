//! Primitives used in the Orchard protocol.
// TODO:
// - DH stuff
//     - EphemeralPublicKey
//     - EphemeralSecretKey

pub mod redpallas;

/// Bytes that are not a canonical encoding of a Pallas point.
///
/// Leaf error for [`ValueCommitmentBytes::decompress`](crate::value::ValueCommitmentBytes::decompress)
/// and [`VerificationKeyBytes::decompress`](redpallas::VerificationKeyBytes::decompress)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidPoint;

impl core::fmt::Display for InvalidPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("not a canonical encoding of a Pallas point")
    }
}

impl core::error::Error for InvalidPoint {}
