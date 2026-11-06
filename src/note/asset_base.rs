use core::cmp::Ordering;
use group::{Group, GroupEncoding};
use pasta_curves::pallas;
use subtle::{Choice, ConstantTimeEq, CtOption};

use crate::constants::zatoshi_asset_base::zatoshi_asset_base;

#[cfg(test)]
use rand_core::CryptoRngCore;

/// Note type identifier.
#[derive(Clone, Copy, Debug, Eq)]
pub struct AssetBase(pallas::Point);

// AssetBase must implement PartialOrd and Ord to be used as a key in BTreeMap.
impl PartialOrd for AssetBase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AssetBase {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_bytes().cmp(&other.0.to_bytes())
    }
}

impl AssetBase {
    /// Deserialize the AssetBase from a byte array.
    ///
    /// Returns `None` if the byte encoding is invalid or if it corresponds
    /// to the identity point.
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Point::from_bytes(bytes)
            .and_then(|asset| CtOption::new(AssetBase(asset), !asset.is_identity()))
    }

    /// Serialize the AssetBase to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Note type for zatoshi, maintains backward compatibility with Orchard untyped notes.
    pub fn zatoshi() -> Self {
        AssetBase(pallas::Point::from(zatoshi_asset_base()))
    }

    /// The base point used in value commitments.
    pub fn cv_base(&self) -> pallas::Point {
        self.0
    }

    /// Whether this note represents zatoshi or ZSA asset.
    pub fn is_zatoshi(&self) -> Choice {
        self.0.ct_eq(&Self::zatoshi().0)
    }

    /// Generates a ZSA random asset from a random non-identity Pallas point.
    ///
    /// Normally, an `AssetBase` is derived from an issuance validating key. For testing purposes,
    /// it is sufficient to use a random non-identity Pallas point. This allows generating a random
    /// `AssetBase` even when `zsa-issuance` feature is disabled.
    ///
    /// This is only used in tests.
    #[cfg(test)]
    pub(crate) fn random(rng: &mut impl CryptoRngCore) -> Self {
        loop {
            let random_point = pallas::Point::random(&mut *rng);
            // Extremely unlikely, but we explicitly reject the identity point.
            if bool::from(random_point.is_identity()) {
                continue;
            }
            return Self(random_point);
        }
    }
}

impl PartialEq for AssetBase {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }
}
