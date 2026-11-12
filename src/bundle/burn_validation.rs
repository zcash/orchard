//! Validating burn operations on asset bundles.

use alloc::collections::BTreeSet;
use core::fmt;

#[cfg(feature = "zsa-issuance")]
use alloc::{collections::BTreeMap, vec::Vec};

#[cfg(feature = "zsa-issuance")]
use crate::issuance::AssetRecord;

use crate::{
    bundle::{BundleVersion, Flags},
    note::AssetBase,
    value::NoteValue,
};

/// Maximum burn value.
/// Burns must fit in both u64 and i64 for value balance calculations.
pub const MAX_BURN_VALUE: u64 = (1u64 << 63) - 1;

/// Possible errors that can occur when validating a burn.
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
    /// A non-empty burn was provided for a bundle whose version does not permit ZSA, or
    /// whose flags do not enable ZSA.
    BurnNotPermitted,
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
            BurnError::BurnNotPermitted => write!(
                f,
                "a non-empty burn was provided for a bundle whose version does not permit ZSA, \
                 or whose flags do not enable ZSA"
            ),
        }
    }
}

/// Whether a bundle of this version, with these flags, may carry a burn at all.
///
/// Burn instructions are only meaningful for the ZSA protocol, so both conditions are
/// required. The version alone is not enough: a ZSA bundle may still carry `zsa_enabled`
/// cleared, and the circuit then forces every asset to zatoshi, so no burn can be balanced.
pub(crate) fn burn_permitted(flags: &Flags, bundle_version: BundleVersion) -> bool {
    bundle_version.permits_zsa() && flags.zsa_enabled()
}

/// Checks a bundle's whole `burn` set, without consulting the global issuance state.
///
/// This covers the three properties that can be checked without that state:
///
/// - a non-empty burn is only permitted when [`burn_permitted`] holds;
/// - each entry is valid on its own, per [`validate_burn_entry`];
/// - no asset appears twice.
///
/// # Errors
///
/// * [`BurnError::BurnNotPermitted`] if `burn` is non-empty but `bundle_version` does not
///   permit ZSA, or `flags` do not enable ZSA;
/// * [`BurnError::ZatoshiAsset`] if an entry burns the zatoshi asset;
/// * [`BurnError::ZeroAmount`] if an entry burns a zero amount;
/// * [`BurnError::InvalidAmount`] if an entry burns more than [`MAX_BURN_VALUE`];
/// * [`BurnError::DuplicateAsset`] if the same asset appears in more than one entry.
///
/// Entries are checked in order, and the first failure is returned.
pub(crate) fn validate_burn(
    burn: &[(AssetBase, NoteValue)],
    flags: &Flags,
    bundle_version: BundleVersion,
) -> Result<(), BurnError> {
    // The permission applies to a non-empty burn: an empty one is fine under any version
    // and flags.
    if !burn.is_empty() && !burn_permitted(flags, bundle_version) {
        return Err(BurnError::BurnNotPermitted);
    }

    let mut seen = BTreeSet::new();
    for &(asset, value) in burn {
        validate_burn_entry(asset, value)?;
        if !seen.insert(asset) {
            return Err(BurnError::DuplicateAsset);
        }
    }

    Ok(())
}

/// Checks one burn entry: the asset must not be the native one, and the amount must be
/// non-zero and at most [`MAX_BURN_VALUE`].
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
/// - Ensuring a non-empty burn is permitted by the bundle version and flags
/// - Ensuring each asset is unique, non-zatoshi, fits in u63, and has a non-zero burn value
/// - Verifying that each asset exists in the global issuance state
/// - Checking that there is sufficient supply to burn
/// - Computing the new asset records after burning
///
/// Per ZIP 227, burns are applied before issuance: the state that
/// [`verify_issue_bundle`](crate::issuance::verify_issue_bundle) reads for the same transaction
/// must include the returned records.
///
/// Each burn element is represented as a tuple of `AssetBase` and `NoteValue` (value for the burn).
///
/// # Arguments
///
/// * `burn` - An iterable of assets to burn, where each asset is represented as a tuple of `AssetBase` and `NoteValue`
/// * `flags` - The bundle's [`Flags`], which must enable ZSA for a non-empty burn
/// * `bundle_version` - The bundle's [`BundleVersion`], which must permit ZSA for a non-empty burn
/// * `get_current_record` - A closure that retrieves the current `AssetRecord` for a given `AssetBase`
///
/// # Returns
///
/// A `BTreeMap<AssetBase, AssetRecord>` containing updated records for affected assets only.
///
/// # Errors
///
/// Returns a `BurnError` if:
/// * The burn is non-empty but `bundle_version` does not permit ZSA, or `flags` do not enable
///   ZSA (`BurnError::BurnNotPermitted`).
/// * Any asset in the `burn` vector is zatoshi (`BurnError::ZatoshiAsset`).
/// * Any asset in the `burn` vector has a zero value (`BurnError::ZeroAmount`).
/// * Any burn amount in the `burn` vector is out of the u63 range (`BurnError::InvalidAmount`).
/// * Any asset in the `burn` vector is not unique (`BurnError::DuplicateAsset`).
/// * Any asset is not found in the global issuance state (`BurnError::AssetNotFoundInState`).
/// * Any asset has insufficient supply for the burn amount (`BurnError::InsufficientSupply`).
#[cfg(feature = "zsa-issuance")]
pub fn validate_bundle_burn(
    burn: impl IntoIterator<Item = (AssetBase, NoteValue)>,
    flags: &Flags,
    bundle_version: BundleVersion,
    mut get_current_record: impl FnMut(&AssetBase) -> Option<AssetRecord>,
) -> Result<BTreeMap<AssetBase, AssetRecord>, BurnError> {
    // State-independent checks.
    let burn: Vec<(AssetBase, NoteValue)> = burn.into_iter().collect();
    validate_burn(&burn, flags, bundle_version)?;

    // Fill new_records.
    let mut new_records = BTreeMap::new();

    for (asset, amount) in burn {
        let burn_amount_raw = amount.inner();

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
    use super::{validate_burn, BurnError};
    use crate::{
        bundle::{BundleVersion, Flags},
        note::AssetBase,
        value::NoteValue,
    };
    use rand_core::OsRng;

    #[test]
    fn burn_needs_both_a_zsa_version_and_the_zsa_flag() {
        let mut rng = OsRng;
        let burn = [(AssetBase::random(&mut rng), NoteValue::from_raw(1))];

        // Both conditions hold: permitted.
        assert!(validate_burn(&burn, &Flags::ENABLED_WITH_ZSA, BundleVersion::zsa()).is_ok());

        // The ZSA version alone is not enough; `zsa_enabled` is a separate bit.
        assert!(matches!(
            validate_burn(&burn, &Flags::ENABLED, BundleVersion::zsa()),
            Err(BurnError::BurnNotPermitted)
        ));

        // Nor is the flag alone, on a version that cannot encode a burn.
        assert!(matches!(
            validate_burn(
                &burn,
                &Flags::ENABLED_WITH_ZSA,
                BundleVersion::ironwood_v3()
            ),
            Err(BurnError::BurnNotPermitted)
        ));

        // An empty burn is always fine, whatever the version and flags.
        let empty: [(AssetBase, NoteValue); 0] = [];
        assert!(validate_burn(&empty, &Flags::ENABLED, BundleVersion::ironwood_v3()).is_ok());
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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED_WITH_ZSA,
            BundleVersion::zsa(),
            |asset| mock_records.get(asset).cloned(),
        );

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
    fn validate_bundle_burn_empty_burn() {
        let assets = generate_unique_assets(2);

        let mock_records = mock_issuance_records(&[
            AssetSupply::new(assets[0], 100),
            AssetSupply::new(assets[1], 200),
        ]);

        let result = validate_bundle_burn(
            vec![],                       // burn
            &Flags::ENABLED,              // enable_zsa = false
            BundleVersion::ironwood_v3(), // ZSA not permitted
            |asset| mock_records.get(asset).cloned(),
        );

        assert!(result.is_ok());
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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED_WITH_ZSA,
            BundleVersion::zsa(),
            |asset| mock_records.get(asset).cloned(),
        );

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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED_WITH_ZSA,
            BundleVersion::zsa(),
            |asset| mock_records.get(asset).cloned(),
        );

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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED_WITH_ZSA,
            BundleVersion::zsa(),
            |asset| mock_records.get(asset).cloned(),
        );

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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED_WITH_ZSA,
            BundleVersion::zsa(),
            |asset| mock_records.get(asset).cloned(),
        );

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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED_WITH_ZSA,
            BundleVersion::zsa(),
            |asset| mock_records.get(asset).cloned(),
        );

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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED_WITH_ZSA,
            BundleVersion::zsa(),
            |asset| mock_records.get(asset).cloned(),
        );

        assert_eq!(result, Err(BurnError::InsufficientSupply));
    }

    #[test]
    fn validate_bundle_burn_wrong_flags() {
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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED,
            BundleVersion::zsa(),
            |asset| mock_records.get(asset).cloned(),
        );

        assert_eq!(result, Err(BurnError::BurnNotPermitted));
    }

    #[test]
    fn validate_bundle_burn_wrong_bundle_version() {
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

        let result = validate_bundle_burn(
            bundle_burn,
            &Flags::ENABLED_WITH_ZSA,
            BundleVersion::ironwood_v3(),
            |asset| mock_records.get(asset).cloned(),
        );

        assert_eq!(result, Err(BurnError::BurnNotPermitted));
    }
}
