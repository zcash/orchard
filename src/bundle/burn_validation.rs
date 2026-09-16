//! Validating burn operations on asset bundles.

use core::fmt;

#[cfg(feature = "zsa-issuance")]
use alloc::collections::BTreeMap;

#[cfg(feature = "zsa-issuance")]
use crate::issuance::AssetRecord;

use crate::{
    bundle::{BundleError, BundleVersion, Flags},
    note::AssetBase,
    value::NoteValue,
};

/// Maximum burn value.
/// Burns must fit in both u64 and i64 for value balance calculations.
pub const MAX_BURN_VALUE: u64 = (1u64 << 63) - 1;

/// Possible errors that can occur during bundle burn validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

/// Checks that a non-empty `burn` is only present when `bundle_version` permits ZSA and
/// `flags` enable ZSA.
///
/// Burn instructions are only meaningful for the ZSA protocol, so both conditions are
/// required. The version alone is not enough: a ZSA bundle may still be built with
/// `zsa_enabled` cleared, and the circuit then forces every asset to zatoshi, so no burn
/// can be balanced.
///
/// Returns [`BundleError::BurnNotPermitted`] otherwise.
pub(crate) fn validate_burn(
    burn: &[(AssetBase, NoteValue)],
    flags: &Flags,
    bundle_version: BundleVersion,
) -> Result<(), BundleError> {
    if burn.is_empty() || (bundle_version.permits_zsa() && flags.zsa_enabled()) {
        Ok(())
    } else {
        Err(BundleError::BurnNotPermitted)
    }
}

/// Checks one burn entry: the asset must not be the native one, and the amount must be
/// non-zero and at most [`MAX_BURN_VALUE`].
///
/// Uniqueness is a property of the whole set, so it is left to the caller collecting one.
pub(crate) fn validate_burn_entry(asset: AssetBase, value: NoteValue) -> Result<(), BurnError> {
    if asset.is_zatoshi().into() {
        Err(BurnError::ZatoshiAsset)
    } else if value.inner() == 0 {
        Err(BurnError::ZeroAmount)
    } else if value.inner() > MAX_BURN_VALUE {
        Err(BurnError::InvalidAmount)
    } else {
        Ok(())
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
        validate_burn_entry(asset, amount)?;

        let burn_amount_raw = amount.inner();

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

#[cfg(test)]
mod burn_permission_tests {
    use super::validate_burn;
    use crate::{
        bundle::{BundleError, BundleVersion, Flags},
        note::AssetBase,
        value::NoteValue,
    };
    use alloc::vec;
    use rand_core::OsRng;

    #[test]
    fn burn_needs_both_a_zsa_version_and_the_zsa_flag() {
        let mut rng = OsRng;
        let burn = vec![(AssetBase::random(&mut rng), NoteValue::from_raw(1))];

        // Both conditions hold: permitted.
        assert!(validate_burn(&burn, &Flags::ENABLED_WITH_ZSA, BundleVersion::zsa()).is_ok());

        // The ZSA version alone is not enough; `zsa_enabled` is a separate bit.
        assert!(matches!(
            validate_burn(&burn, &Flags::ENABLED, BundleVersion::zsa()),
            Err(BundleError::BurnNotPermitted)
        ));

        // Nor is the flag alone, on a version that cannot encode a burn.
        assert!(matches!(
            validate_burn(
                &burn,
                &Flags::ENABLED_WITH_ZSA,
                BundleVersion::ironwood_v3()
            ),
            Err(BundleError::BurnNotPermitted)
        ));

        // An empty burn is always fine, whatever the version and flags.
        assert!(validate_burn(&[], &Flags::ENABLED, BundleVersion::ironwood_v3()).is_ok());
    }
}

#[cfg(feature = "zsa-issuance")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{note::NoteVersion, value::NoteValue, Note};

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
                    NoteValue::ZERO,
                    *asset,
                    NoteVersion::ZSA,
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
