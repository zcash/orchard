//! Primitives used in the Orchard protocol.
// TODO:
// - DH stuff
//     - EphemeralPublicKey
//     - EphemeralSecretKey

pub mod redpallas;

/// Bytes that are not a canonical encoding of the point they should hold.
///
/// Leaf error for the `decompress` methods on the compressed types, which do not know which
/// field they were read into; a description names the field in its own error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidPoint;

impl core::fmt::Display for InvalidPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("not a canonical encoding of a Pallas point")
    }
}

impl core::error::Error for InvalidPoint {}
