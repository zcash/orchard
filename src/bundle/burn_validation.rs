//! Validating burn operations on asset bundles.
//!
//! The module provides a function `validate_bundle_burn` that can be used to validate the burn values for the bundle.
//!
use alloc::collections::BTreeSet;
use core::fmt;

use crate::{note::AssetBase, value::NoteValue};

/// Maximum burn value.
/// Burns must fit in both u64 and i64 for value balance calculations.
pub const MAX_BURN_VALUE: u64 = (1u64 << 63) - 1;

/// Possible errors that can occur during bundle burn validation.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum BurnError {
    /// Encountered a duplicate asset to burn.
    DuplicateAsset,
    /// Cannot burn a zatoshi asset.
    ZatoshiAsset,
    /// Cannot burn an asset with a zero value.
    ZeroAmount,
    /// Burn amount does not fit in u63.
    InvalidAmount,
}

/// Validates burn for a bundle by ensuring each asset is unique, non-zatoshi, fit in u63 and has a
/// non-zero value.
///
/// Each burn element is represented as a tuple of `AssetBase` and `NoteValue` (value for the burn).
///
/// # Arguments
///
/// * `burn` - A vector of assets, where each asset is represented as a tuple of `AssetBase` and `NoteValue` (value the burn).
///
/// # Errors
///
/// Returns a `BurnError` if:
/// * Any asset in the `burn` vector is zatoshi (`BurnError::ZatoshiAsset`).
/// * Any asset in the `burn` vector has a zero value (`BurnError::ZeroAmount`).
/// * Any burn amount in the `burn` vector is out of the u63 range (`BurnError::InvalidAmount`).
/// * Any asset in the `burn` vector is not unique (`BurnError::DuplicateAsset`).
pub fn validate_bundle_burn(burn: &[(AssetBase, NoteValue)]) -> Result<(), BurnError> {
    let mut burn_set = BTreeSet::new();

    for (asset, value) in burn {
        if asset.is_zatoshi().into() {
            return Err(BurnError::ZatoshiAsset);
        }
        if value.inner() == 0 {
            return Err(BurnError::ZeroAmount);
        }
        if value.inner() > MAX_BURN_VALUE {
            return Err(BurnError::InvalidAmount);
        }
        if !burn_set.insert(*asset) {
            return Err(BurnError::DuplicateAsset);
        }
    }

    Ok(())
}

impl fmt::Display for BurnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BurnError::DuplicateAsset => write!(f, "Encountered a duplicate asset to burn."),
            BurnError::ZatoshiAsset => write!(f, "Cannot burn a zatoshi asset."),
            BurnError::ZeroAmount => {
                write!(f, "Cannot burn an asset with a zero value.")
            }
            BurnError::InvalidAmount => {
                write!(f, "Burn amount must fit in u63.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::NoteValue;

    use alloc::collections::BTreeSet;
    use rand_core::{CryptoRngCore, OsRng};

    fn burn_tuple_unique(
        rng: &mut impl CryptoRngCore,
        used: &mut BTreeSet<AssetBase>,
        value: u64,
    ) -> (AssetBase, NoteValue) {
        loop {
            let asset = AssetBase::random(rng);
            if used.insert(asset) {
                return (asset, NoteValue::from_raw(value));
            }
        }
    }

    #[test]
    fn validate_bundle_burn_success() {
        let mut rng = OsRng;
        let mut used = BTreeSet::new();

        let bundle_burn = vec![
            burn_tuple_unique(&mut rng, &mut used, 10),
            burn_tuple_unique(&mut rng, &mut used, 20),
            burn_tuple_unique(&mut rng, &mut used, 10),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert!(result.is_ok());
    }

    #[test]
    fn validate_bundle_burn_duplicate_asset() {
        let mut rng = OsRng;

        let asset = AssetBase::random(&mut rng);

        let bundle_burn = vec![
            (asset, NoteValue::from_raw(10)),
            (asset, NoteValue::from_raw(20)),
            (AssetBase::random(&mut rng), NoteValue::from_raw(10)),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::DuplicateAsset));
    }

    #[test]
    fn validate_bundle_burn_zatoshi_asset() {
        let mut rng = OsRng;
        let mut used = BTreeSet::new();

        let bundle_burn = vec![
            burn_tuple_unique(&mut rng, &mut used, 10),
            (AssetBase::zatoshi(), NoteValue::from_raw(20)),
            burn_tuple_unique(&mut rng, &mut used, 10),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::ZatoshiAsset));
    }

    #[test]
    fn validate_bundle_burn_zero_value() {
        let mut rng = OsRng;
        let mut used = BTreeSet::new();

        let bundle_burn = vec![
            burn_tuple_unique(&mut rng, &mut used, 10),
            burn_tuple_unique(&mut rng, &mut used, 0),
            burn_tuple_unique(&mut rng, &mut used, 10),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::ZeroAmount));
    }

    #[test]
    fn validate_bundle_burn_invalid_amount() {
        let mut rng = OsRng;
        let mut used = BTreeSet::new();

        let bundle_burn = vec![
            burn_tuple_unique(&mut rng, &mut used, 10),
            burn_tuple_unique(&mut rng, &mut used, MAX_BURN_VALUE + 1),
            burn_tuple_unique(&mut rng, &mut used, 10),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::InvalidAmount));
    }
}
