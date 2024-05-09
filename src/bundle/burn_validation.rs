//! Validating burn operations on asset bundles.
//!
//! The module provides a function `validate_bundle_burn` that can be used to validate the burn values for the bundle.
//!
use std::fmt;

use crate::note::AssetBase;

/// Possible errors that can occur during bundle burn validation.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum BurnError {
    /// Encountered a duplicate asset to burn.
    DuplicateAsset,
    /// Cannot burn a native asset.
    NativeAsset,
    /// Cannot burn an asset with a non-positive value.
    NonPositiveAmount,
}

/// Validates burn for a bundle by ensuring each asset is unique, non-native, and has a positive value.
///
/// Each burn element is represented as a tuple of `AssetBase` and `i64` (value for the burn).
///
/// # Arguments
///
/// * `burn` - A vector of assets, where each asset is represented as a tuple of `AssetBase` and `i64` (value the burn).
///
/// # Errors
///
/// Returns a `BurnError` if:
/// * Any asset in the `burn` vector is not unique (`BurnError::DuplicateAsset`).
/// * Any asset in the `burn` vector is native (`BurnError::NativeAsset`).
/// * Any asset in the `burn` vector has a non-positive value (`BurnError::NonPositiveAmount`).
pub fn validate_bundle_burn(bundle_burn: &Vec<(AssetBase, i64)>) -> Result<(), BurnError> {
    let mut asset_set = std::collections::HashSet::<&AssetBase>::new();

    for (asset, value) in bundle_burn {
        if !asset_set.insert(asset) {
            return Err(BurnError::DuplicateAsset);
        }
        if asset.is_native().into() {
            return Err(BurnError::NativeAsset);
        }
        if *value <= 0 {
            return Err(BurnError::NonPositiveAmount);
        }
    }

    Ok(())
}

impl fmt::Display for BurnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BurnError::DuplicateAsset => write!(f, "Encountered a duplicate asset to burn."),
            BurnError::NativeAsset => write!(f, "Cannot burn a native asset."),
            BurnError::NonPositiveAmount => {
                write!(f, "Cannot burn an asset with a non-positive value.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates an item of bundle burn list for a given asset description and value.
    ///
    /// This function is deterministic and guarantees that each call with the same parameters
    /// will return the same result. It achieves determinism by using a static `IssuanceAuthorizingKey`.
    ///
    /// # Arguments
    ///
    /// * `asset_desc` - The asset description string.
    /// * `value` - The value for the burn.
    ///
    /// # Returns
    ///
    /// A tuple `(AssetBase, Amount)` representing the burn list item.
    ///
    pub fn get_burn_tuple(asset_desc: &str, value: i64) -> (AssetBase, i64) {
        use crate::keys::{IssuanceAuthorizingKey, IssuanceValidatingKey};

        let isk = IssuanceAuthorizingKey::from_bytes([1u8; 32]).unwrap();

        (
            AssetBase::derive(&IssuanceValidatingKey::from(&isk), asset_desc),
            value,
        )
    }

    #[test]
    fn validate_bundle_burn_success() {
        let bundle_burn = vec![
            get_burn_tuple("Asset 1", 10),
            get_burn_tuple("Asset 2", 20),
            get_burn_tuple("Asset 3", 10),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert!(result.is_ok());
    }

    #[test]
    fn validate_bundle_burn_duplicate_asset() {
        let bundle_burn = vec![
            get_burn_tuple("Asset 1", 10),
            get_burn_tuple("Asset 1", 20),
            get_burn_tuple("Asset 3", 10),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::DuplicateAsset));
    }

    #[test]
    fn validate_bundle_burn_native_asset() {
        let bundle_burn = vec![
            get_burn_tuple("Asset 1", 10),
            (AssetBase::native(), 20),
            get_burn_tuple("Asset 3", 10),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::NativeAsset));
    }

    #[test]
    fn validate_bundle_burn_zero_value() {
        let bundle_burn = vec![
            get_burn_tuple("Asset 1", 10),
            get_burn_tuple("Asset 2", 0),
            get_burn_tuple("Asset 3", 10),
        ];

        let result = validate_bundle_burn(&bundle_burn);

        assert_eq!(result, Err(BurnError::NonPositiveAmount));
    }
}
