//! Validating burn operations on asset bundles.
//!
//! The module provides a function `validate_bundle_burn` that can be used to validate the burn values for the bundle.
//!
use alloc::collections::BTreeSet;
use core::fmt;

use crate::{note::AssetBase, value::NoteValue};

/// Possible errors that can occur during bundle burn validation.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum BurnError {
    /// Encountered a duplicate asset to burn.
    DuplicateAsset,
    /// Cannot burn a native asset.
    NativeAsset,
    /// Cannot burn an asset with a zero value.
    ZeroAmount,
}

/// Validates burn for a bundle by ensuring each asset is unique, non-native, and has a non-zero value.
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
/// * Any asset in the `burn` vector is native (`BurnError::NativeAsset`).
/// * Any asset in the `burn` vector has a zero value (`BurnError::ZeroAmount`).
/// * Any asset in the `burn` vector is not unique (`BurnError::DuplicateAsset`).
pub fn validate_bundle_burn(burn: &[(AssetBase, NoteValue)]) -> Result<(), BurnError> {
    let mut burn_set = BTreeSet::new();

    for (asset, value) in burn {
        if asset.is_native().into() {
            return Err(BurnError::NativeAsset);
        }
        if value.inner() == 0 {
            return Err(BurnError::ZeroAmount);
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
            BurnError::NativeAsset => write!(f, "Cannot burn a native asset."),
            BurnError::ZeroAmount => {
                write!(f, "Cannot burn an asset with a zero value.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{issuance::compute_asset_desc_hash, issuance_auth::ZSASchnorr, value::NoteValue};
    use nonempty::NonEmpty;

    /// Creates an item of bundle burn list for a given asset description hash and value.
    ///
    /// This function is deterministic and guarantees that each call with the same parameters
    /// will return the same result. It achieves determinism by using a static `IssueAuthKey`.
    ///
    /// # Arguments
    ///
    /// * `asset_desc_hash` - The asset description hash.
    /// * `value` - The value for the burn.
    ///
    /// # Returns
    ///
    /// A tuple `(AssetBase, Amount)` representing the burn list item.
    ///
    fn get_burn_tuple(asset_desc_hash: &[u8; 32], value: u64) -> (AssetBase, NoteValue) {
        use crate::issuance_auth::{IssueAuthKey, IssueValidatingKey};

        let isk = IssueAuthKey::<ZSASchnorr>::from_bytes(&[1u8; 32]).unwrap();

        (
            AssetBase::derive(&IssueValidatingKey::from(&isk), asset_desc_hash),
            NoteValue::from_raw(value),
        )
    }

    #[test]
    fn validate_bundle_burn_success() {
        let bundle_burn = vec![
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 1").unwrap()),
                10,
            ),
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 2").unwrap()),
                20,
            ),
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 3").unwrap()),
                10,
            ),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert!(result.is_ok());
    }

    #[test]
    fn validate_bundle_burn_duplicate_asset() {
        let bundle_burn = vec![
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 1").unwrap()),
                10,
            ),
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 1").unwrap()),
                20,
            ),
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 3").unwrap()),
                10,
            ),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::DuplicateAsset));
    }

    #[test]
    fn validate_bundle_burn_native_asset() {
        let bundle_burn = vec![
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 1").unwrap()),
                10,
            ),
            (AssetBase::native(), NoteValue::from_raw(20)),
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 3").unwrap()),
                10,
            ),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::NativeAsset));
    }

    #[test]
    fn validate_bundle_burn_zero_value() {
        let bundle_burn = vec![
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 1").unwrap()),
                10,
            ),
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 2").unwrap()),
                0,
            ),
            get_burn_tuple(
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset 3").unwrap()),
                10,
            ),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::ZeroAmount));
    }
}
