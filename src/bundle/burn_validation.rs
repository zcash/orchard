//! Validating burn operations on asset bundles.
//!
//! The module provides a function `validate_bundle_burn` that can be used to validate the burn values for the bundle.
//!
use core::fmt;

#[cfg(feature = "zsa-issuance")]
use alloc::collections::BTreeMap;

#[cfg(feature = "zsa-issuance")]
use crate::{issuance::AssetRecord, note::AssetBase, value::NoteValue};

/// Maximum burn value.
/// Burns must fit in both u64 and i64 for value balance calculations.
#[cfg(feature = "zsa-issuance")]
pub const MAX_BURN_VALUE: u64 = (1u64 << 63) - 1;

/// Possible errors that can occur during bundle burn validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BurnError {
    /// Encountered a duplicate asset to burn.
    DuplicateAsset,
    /// Cannot burn a zatoshi asset.
    ZatoshiAsset,
    /// Cannot burn an asset with a zero value.
    ZeroAmount,
    /// Burn amount does not fit in u63.
    InvalidAmount,
    /// Asset not found in global issuance state.
    AssetNotFoundInState,
    /// Insufficient supply for burn.
    InsufficientSupply,
}

impl fmt::Display for BurnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BurnError::DuplicateAsset => write!(f, "Encountered a duplicate asset to burn"),
            BurnError::ZatoshiAsset => write!(f, "Cannot burn a zatoshi asset"),
            BurnError::ZeroAmount => {
                write!(f, "Cannot burn an asset with a zero value")
            }
            BurnError::InvalidAmount => {
                write!(f, "Burn amount must fit in u63")
            }
            BurnError::AssetNotFoundInState => {
                write!(f, "Asset not found in global issuance state")
            }
            BurnError::InsufficientSupply => write!(f, "Insufficient supply for burn"),
        }
    }
}

/// Validates burn operations for a bundle and returns updated issuance records for the affected assets.
///
/// These issuance records correspond to entries in the “global issuance state” defined in ZIP-0227.
///
/// This function validates burn operations by:
/// - Ensuring each asset is unique, non-zatoshi, fits in u63, and has a non-zero burn value
/// - Verifying that each asset exists in the global issuance state
/// - Checking that there is sufficient supply to burn
/// - Computing the new asset records after burning
///
/// Each burn element is represented as a tuple of `AssetBase` and `NoteValue` (value for the burn).
///
/// # Arguments
///
/// * `burn` - An iterable of assets to burn, where each asset is represented as a tuple of `AssetBase` and `NoteValue`
/// * `get_current_record` - A closure that retrieves the current `AssetRecord` for a given `AssetBase`
///
/// # Returns
///
/// A `BTreeMap<AssetBase, AssetRecord>` containing updated records for affected assets only.
///
/// # Errors
///
/// Returns a `BurnError` if:
/// * Any asset in the `burn` vector is zatoshi (`BurnError::ZatoshiAsset`).
/// * Any asset in the `burn` vector has a zero value (`BurnError::ZeroAmount`).
/// * Any burn amount in the `burn` vector is out of the u63 range (`BurnError::InvalidAmount`).
/// * Any asset in the `burn` vector is not unique (`BurnError::DuplicateAsset`).
/// * Any asset is not found in the global issuance state (`BurnError::AssetNotFoundInState`).
/// * Any asset has insufficient supply for the burn amount (`BurnError::InsufficientSupply`).
#[cfg(feature = "zsa-issuance")]
pub fn validate_bundle_burn(
    burn: impl IntoIterator<Item = (AssetBase, NoteValue)>,
    mut get_current_record: impl FnMut(&AssetBase) -> Option<AssetRecord>,
) -> Result<BTreeMap<AssetBase, AssetRecord>, BurnError> {
    let mut new_records = BTreeMap::new();

    for (asset, amount) in burn {
        if asset.is_zatoshi().into() {
            return Err(BurnError::ZatoshiAsset);
        }

        let burn_amount_raw = amount.inner();
        if burn_amount_raw == 0 {
            return Err(BurnError::ZeroAmount);
        }
        if burn_amount_raw > MAX_BURN_VALUE {
            return Err(BurnError::InvalidAmount);
        }

        if new_records.contains_key(&asset) {
            return Err(BurnError::DuplicateAsset);
        }

        let current_record = get_current_record(&asset).ok_or(BurnError::AssetNotFoundInState)?;

        let current_amount_raw = current_record.amount.inner();
        if current_amount_raw < burn_amount_raw {
            return Err(BurnError::InsufficientSupply);
        }

        let new_record = AssetRecord {
            amount: NoteValue::from_raw(current_amount_raw - burn_amount_raw),
            is_finalized: current_record.is_finalized,
            reference_note: current_record.reference_note,
        };

        new_records.insert(asset, new_record);
    }

    Ok(new_records)
}

#[cfg(feature = "zsa-issuance")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{value::NoteValue, Note};

    use alloc::{collections::BTreeSet, vec::Vec};
    use rand_core::OsRng;

    /// Generates a vector of unique random assets.
    fn generate_unique_assets(count: usize) -> Vec<AssetBase> {
        let mut rng = OsRng;
        let mut used = BTreeSet::new();

        (0..count)
            .map(|_| loop {
                let asset = AssetBase::random(&mut rng);
                if used.insert(asset) {
                    break asset;
                }
            })
            .collect()
    }

    /// Test helper struct describing an issued supply for an asset.
    struct AssetSupply {
        asset: AssetBase,
        supply: u64,
    }

    impl AssetSupply {
        fn new(asset: AssetBase, supply: u64) -> Self {
            Self { asset, supply }
        }
    }

    /// Builds mock global issuance records used by burn validation tests.
    ///
    /// Each asset gets a finalized `AssetRecord` with a reference note and the given supply.
    fn mock_issuance_records(data: &[AssetSupply]) -> BTreeMap<AssetBase, AssetRecord> {
        use crate::constants::reference_keys::ReferenceKeys;

        let mut rng = OsRng;

        data.iter()
            .map(|AssetSupply { asset, supply }| {
                let reference_note = Note::new_issue_note(
                    ReferenceKeys::recipient(),
                    NoteValue::zero(),
                    *asset,
                    &mut rng,
                );

                let record = AssetRecord {
                    amount: NoteValue::from_raw(*supply),
                    is_finalized: true,
                    reference_note,
                };
                (*asset, record)
            })
            .collect()
    }

    /// Removes reference notes, keeping only amounts (reference notes contain
    /// randomness and can't be compared directly).
    fn remove_reference_notes(
        records: &BTreeMap<AssetBase, AssetRecord>,
    ) -> BTreeMap<AssetBase, NoteValue> {
        records
            .iter()
            .map(|(asset, record)| (*asset, record.amount))
            .collect()
    }

    #[test]
    fn validate_bundle_burn_success() {
        let assets = generate_unique_assets(3);

        // Create initial mock records (mock global issuance state)
        let mock_records = mock_issuance_records(&[
            AssetSupply::new(assets[0], 100),
            AssetSupply::new(assets[1], 50),
            AssetSupply::new(assets[2], 200),
        ]);

        let bundle_burn = vec![
            (assets[0], NoteValue::from_raw(10)),
            (assets[1], NoteValue::from_raw(20)),
            (assets[2], NoteValue::from_raw(10)),
        ];

        let result = validate_bundle_burn(bundle_burn, |asset| mock_records.get(asset).cloned());

        assert!(result.is_ok());

        let expected_records = mock_issuance_records(&[
            AssetSupply::new(assets[0], 90),
            AssetSupply::new(assets[1], 30),
            AssetSupply::new(assets[2], 190),
        ]);

        assert_eq!(
            remove_reference_notes(&result.unwrap()),
            remove_reference_notes(&expected_records)
        );
    }

    #[test]
    fn validate_bundle_burn_duplicate_asset() {
        let assets = generate_unique_assets(2);

        let mock_records = mock_issuance_records(&[
            AssetSupply::new(assets[0], 100),
            AssetSupply::new(assets[1], 200),
        ]);

        let bundle_burn = vec![
            (assets[0], NoteValue::from_raw(10)),
            (assets[0], NoteValue::from_raw(20)),
            (assets[1], NoteValue::from_raw(10)),
        ];

        let result = validate_bundle_burn(bundle_burn, |asset| mock_records.get(asset).cloned());

        assert_eq!(result, Err(BurnError::DuplicateAsset));
    }

    #[test]
    fn validate_bundle_burn_zatoshi_asset() {
        let assets = generate_unique_assets(2);

        let mock_records = mock_issuance_records(&[
            AssetSupply::new(assets[0], 100),
            AssetSupply::new(assets[1], 200),
        ]);

        let bundle_burn = vec![
            (assets[0], NoteValue::from_raw(10)),
            (AssetBase::zatoshi(), NoteValue::from_raw(20)),
            (assets[1], NoteValue::from_raw(10)),
        ];

        let result = validate_bundle_burn(bundle_burn, |asset| mock_records.get(asset).cloned());

        assert_eq!(result, Err(BurnError::ZatoshiAsset));
    }

    #[test]
    fn validate_bundle_burn_zero_value() {
        let assets = generate_unique_assets(3);

        let mock_records = mock_issuance_records(&[
            AssetSupply::new(assets[0], 100),
            AssetSupply::new(assets[1], 50),
            AssetSupply::new(assets[2], 200),
        ]);

        let bundle_burn = vec![
            (assets[0], NoteValue::from_raw(10)),
            (assets[1], NoteValue::from_raw(0)),
            (assets[2], NoteValue::from_raw(10)),
        ];

        let result = validate_bundle_burn(bundle_burn, |asset| mock_records.get(asset).cloned());

        assert_eq!(result, Err(BurnError::ZeroAmount));
    }

    #[test]
    fn validate_bundle_burn_invalid_amount() {
        let assets = generate_unique_assets(3);

        let mock_records = mock_issuance_records(&[
            AssetSupply::new(assets[0], u64::MAX),
            AssetSupply::new(assets[1], u64::MAX),
            AssetSupply::new(assets[2], u64::MAX),
        ]);

        let bundle_burn = vec![
            (assets[0], NoteValue::from_raw(10)),
            (assets[1], NoteValue::from_raw(MAX_BURN_VALUE + 1)),
            (assets[2], NoteValue::from_raw(10)),
        ];

        let result = validate_bundle_burn(bundle_burn, |asset| mock_records.get(asset).cloned());

        assert_eq!(result, Err(BurnError::InvalidAmount));
    }

    #[test]
    fn validate_bundle_burn_asset_not_found() {
        let assets = generate_unique_assets(3);

        // Only add first asset to the mock records (mock global issuance state)
        let mock_records = mock_issuance_records(&[AssetSupply::new(assets[0], 100)]);

        let bundle_burn = vec![
            (assets[0], NoteValue::from_raw(10)),
            (assets[1], NoteValue::from_raw(20)), // Not in the global issuance state
        ];

        let result = validate_bundle_burn(bundle_burn, |asset| mock_records.get(asset).cloned());

        assert_eq!(result, Err(BurnError::AssetNotFoundInState));
    }

    #[test]
    fn validate_bundle_burn_insufficient_supply() {
        let assets = generate_unique_assets(2);

        let mock_records = mock_issuance_records(&[
            AssetSupply::new(assets[0], 100),
            AssetSupply::new(assets[1], 50),
        ]);

        let bundle_burn = vec![
            (assets[0], NoteValue::from_raw(10)),
            (assets[1], NoteValue::from_raw(100)), // Only has 50
        ];

        let result = validate_bundle_burn(bundle_burn, |asset| mock_records.get(asset).cloned());

        assert_eq!(result, Err(BurnError::InsufficientSupply));
    }
}
