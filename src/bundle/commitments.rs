//! Utility functions for computing bundle commitments

use alloc::vec::Vec;
use blake2b_simd::{Hash as Blake2bHash, Params, State};

use crate::{
    bundle::{Authorization, Authorized, Bundle, CommitmentError, TxVersion},
    primitives::OrchardPrimitives,
    sighash_kind::OrchardSighashKind,
    ValuePool,
};

#[cfg(feature = "zsa-issuance")]
mod issuance;

#[cfg(feature = "zsa-issuance")]
pub(crate) use issuance::{hash_issue_bundle_auth_data, hash_issue_bundle_txid_data};

#[cfg(feature = "zsa-issuance")]
pub use issuance::{hash_issue_bundle_auth_empty, hash_issue_bundle_txid_empty};

const ZCASH_ORCHARD_V5_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";
const ZCASH_ORCHARD_V6_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardH_v6";
const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActCHash";
const ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActMHash";
const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActNHash";
const ZCASH_ORCHARD_V5_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaHash";
const ZCASH_ORCHARD_V6_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaH_v6";
pub(crate) const ZCASH_IRONWOOD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIronwd_H_v6";
const ZCASH_IRONWOOD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActCH_v6";
pub(crate) const ZCASH_IRONWOOD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActMH_v6";
const ZCASH_IRONWOOD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActNH_v6";
pub(crate) const ZCASH_IRONWOOD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthIrnwdH_v6";

pub(crate) const ZCASH_ZSA_ACTION_GROUPS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActGHash";
pub(crate) const ZCASH_ZSA_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxId6OActC_Hash";
pub(crate) const ZCASH_ZSA_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxId6OActN_Hash";
pub(crate) const ZCASH_ZSA_BURN_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcBurnHash";
pub(crate) const ZCASH_ZSA_ACTION_GROUPS_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrcAGHash";
pub(crate) const ZCASH_ZSA_SPEND_AUTH_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrSASHash";

#[derive(Clone, Copy, Debug)]
pub(crate) struct BundleCommitmentPersonalizations {
    pub(crate) bundle: &'static [u8; 16],
    pub(crate) actions_compact: &'static [u8; 16],
    pub(crate) actions_memos: &'static [u8; 16],
    pub(crate) actions_noncompact: &'static [u8; 16],
    pub(crate) auth: &'static [u8; 16],
    pub(crate) zsa: Option<ZSAPersonalizations>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ZSAPersonalizations {
    pub(crate) action_groups: &'static [u8; 16],
    pub(crate) ironwood_burn: &'static [u8; 16],
    pub(crate) action_groups_auth: &'static [u8; 16],
    pub(crate) zsa_spend_auth: &'static [u8; 16],
}

const ORCHARD_V5_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_ORCHARD_V5_HASH_PERSONALIZATION,
        actions_compact: ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_ORCHARD_V5_SIGS_HASH_PERSONALIZATION,
        zsa: None,
    };

// Orchard v6 deliberately reuses the v5 action-level personalizations
// (compact/memos/noncompact); only the top-level `bundle` and `auth` strings gain `_v6`.
// Ironwood instead uses fresh `_v6` strings throughout. Either way the top-level digest is
// domain-separated by its `bundle`/`auth` personalization, so reusing the action-level ones
// cannot collide across formats.
const ORCHARD_V6_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_ORCHARD_V6_HASH_PERSONALIZATION,
        actions_compact: ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_ORCHARD_V6_SIGS_HASH_PERSONALIZATION,
        zsa: None,
    };

const IRONWOOD_V6_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_IRONWOOD_HASH_PERSONALIZATION,
        actions_compact: ZCASH_IRONWOOD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_IRONWOOD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_IRONWOOD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_IRONWOOD_SIGS_HASH_PERSONALIZATION,
        zsa: None,
    };

const ZSA_PERSONALIZATIONS: BundleCommitmentPersonalizations = BundleCommitmentPersonalizations {
    bundle: ZCASH_IRONWOOD_HASH_PERSONALIZATION,
    actions_compact: ZCASH_ZSA_ACTIONS_COMPACT_HASH_PERSONALIZATION,
    actions_memos: ZCASH_IRONWOOD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
    actions_noncompact: ZCASH_ZSA_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
    auth: ZCASH_IRONWOOD_SIGS_HASH_PERSONALIZATION,
    zsa: Some(ZSAPersonalizations {
        action_groups: ZCASH_ZSA_ACTION_GROUPS_HASH_PERSONALIZATION,
        ironwood_burn: ZCASH_ZSA_BURN_HASH_PERSONALIZATION,
        action_groups_auth: ZCASH_ZSA_ACTION_GROUPS_SIGS_HASH_PERSONALIZATION,
        zsa_spend_auth: ZCASH_ZSA_SPEND_AUTH_SIGS_HASH_PERSONALIZATION,
    }),
};

/// The hash format used to compute a bundle's transaction-ID and authorizing digests,
/// selected from the bundle's pool and the version of the transaction it is encoded in.
/// Orchard bundles use the v5 or v6 format according to the transaction; Ironwood bundles
/// exist only in v6 transactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum BundleCommitmentFormat {
    OrchardV5,
    OrchardV6,
    IronwoodV6,
    ZSA,
}

impl ValuePool {
    pub(crate) fn commitment_format(
        self,
        tx_version: TxVersion,
    ) -> Result<BundleCommitmentFormat, CommitmentError> {
        match (self, tx_version) {
            (ValuePool::Orchard, TxVersion::V5) => Ok(BundleCommitmentFormat::OrchardV5),
            (ValuePool::Orchard, TxVersion::V6) => Ok(BundleCommitmentFormat::OrchardV6),
            (ValuePool::Orchard, TxVersion::ZSA) => Err(CommitmentError::InvalidTransactionVersion),
            (ValuePool::Ironwood, TxVersion::V5) => Err(CommitmentError::InvalidTransactionVersion),
            (ValuePool::Ironwood, TxVersion::V6) => Ok(BundleCommitmentFormat::IronwoodV6),
            (ValuePool::Ironwood, TxVersion::ZSA) => Ok(BundleCommitmentFormat::ZSA),
        }
    }
}

impl BundleCommitmentFormat {
    pub(crate) fn personalizations(self) -> BundleCommitmentPersonalizations {
        match self {
            BundleCommitmentFormat::OrchardV5 => ORCHARD_V5_PERSONALIZATIONS,
            BundleCommitmentFormat::OrchardV6 => ORCHARD_V6_PERSONALIZATIONS,
            BundleCommitmentFormat::IronwoodV6 => IRONWOOD_V6_PERSONALIZATIONS,
            BundleCommitmentFormat::ZSA => ZSA_PERSONALIZATIONS,
        }
    }

    pub(crate) fn includes_anchor_in_txid_digest(self) -> bool {
        matches!(self, BundleCommitmentFormat::OrchardV5)
    }

    pub(crate) fn includes_anchor_in_authorizing_digest(self) -> bool {
        matches!(
            self,
            BundleCommitmentFormat::OrchardV6 | BundleCommitmentFormat::IronwoodV6
        )
    }
}

pub(crate) fn hasher(personal: &[u8; 16]) -> State {
    Params::new().hash_length(32).personal(personal).to_state()
}

/// Evaluate `orchard_digest` for the bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
/// or in [ZIP-229: Version 6 Transaction Format][zip229]
/// for OrchardVanilla and as defined in
/// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
/// for OrchardZSA
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip229]: https://zips.z.cash/zip-0229
/// [zip246]: https://zips.z.cash/zip-0246
pub(crate) fn hash_bundle_txid_data<
    A: Authorization,
    V: Copy + Into<i64>,
    Pr: OrchardPrimitives,
>(
    bundle: &Bundle<A, V, Pr>,
    tx_version: TxVersion,
) -> Result<Blake2bHash, CommitmentError> {
    Pr::hash_bundle_txid_data(bundle, tx_version)
}

/// Construct the commitment for the absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_txid_empty(
    value_pool: ValuePool,
    tx_version: TxVersion,
) -> Result<Blake2bHash, CommitmentError> {
    Ok(hasher(
        value_pool
            .commitment_format(tx_version)?
            .personalizations()
            .bundle,
    )
    .finalize())
}

/// Evaluate `orchard_auth_digest` for the bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
/// or in [ZIP-229: Version 6 Transaction Format][zip229]
/// for OrchardVanilla and as defined in
/// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
/// for OrchardZSA
///
/// The `sighash_info_for_kind` closure returns the `SighashInfo` encoding
/// for a given [`OrchardSighashKind`].
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip229]: https://zips.z.cash/zip-0229
/// [zip246]: https://zips.z.cash/zip-0246
pub(crate) fn hash_bundle_auth_data<V, Pr: OrchardPrimitives>(
    bundle: &Bundle<Authorized, V, Pr>,
    tx_version: TxVersion,
    sighash_info_for_kind: impl Fn(&OrchardSighashKind) -> Vec<u8>,
) -> Result<Blake2bHash, CommitmentError> {
    Pr::hash_bundle_auth_data(bundle, tx_version, sighash_info_for_kind)
}

/// Construct the commitment for an absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_auth_empty(
    value_pool: ValuePool,
    tx_version: TxVersion,
) -> Result<Blake2bHash, CommitmentError> {
    Ok(hasher(
        value_pool
            .commitment_format(tx_version)?
            .personalizations()
            .auth,
    )
    .finalize())
}

/// Encodes a size in the CompactSize format.
///
/// This implementation is inspired from `zcash_encoding::CompactSize::write` [code]
/// We cannot use zcash_encoding crate to avoid circular dependency.
///
/// [code]: https://github.com/zcash/librustzcash/blob/8be259c579762f1b0f569453a20c0d0dbeae6c07/components/zcash_encoding/src/lib.rs#L93
pub fn get_compact_size(size: usize) -> Vec<u8> {
    match size {
        s if s < 253 => vec![s as u8],
        s if s <= 0xFFFF => [&[253_u8], &(s as u16).to_le_bytes()[..]].concat(),
        s if s <= 0xFFFFFFFF => [&[254_u8], &(s as u32).to_le_bytes()[..]].concat(),
        s => [&[255_u8], &(s as u64).to_le_bytes()[..]].concat(),
    }
}

#[cfg(all(test, feature = "circuit"))]
mod tests {
    use crate::{
        builder::{Builder, BundleType, UnauthorizedBundle},
        bundle::{
            commitments::{get_compact_size, hash_bundle_auth_data, hash_bundle_txid_data},
            Authorized, Bundle, BundleVersion, TxVersion,
        },
        circuit::ProvingKey,
        flavor::{OrchardFlavor, OrchardVanilla, OrchardZSA},
        keys::{FullViewingKey, Scope, SpendingKey},
        note::AssetBase,
        sighash_kind::test_sighash_info_for_kind,
        value::NoteValue,
        Anchor,
    };
    use rand::{rngs::StdRng, SeedableRng};

    fn generate_bundle<FL: OrchardFlavor>(
        bundle_version: BundleVersion,
    ) -> UnauthorizedBundle<i64, FL> {
        let rng = StdRng::seed_from_u64(5);

        let sk = SpendingKey::from_bytes([7; 32]).unwrap();
        let recipient = FullViewingKey::from(&sk).address_at(0u32, Scope::External);

        let mut builder = Builder::new(
            BundleType::DEFAULT,
            bundle_version,
            bundle_version.default_flags(),
            Anchor::from_bytes([0; 32]).unwrap(),
        )
        .unwrap();
        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(10),
                AssetBase::zatoshi(),
                [0u8; 512],
            )
            .unwrap();

        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(20),
                AssetBase::zatoshi(),
                [0u8; 512],
            )
            .unwrap();

        builder.build::<i64, FL>(rng).unwrap().unwrap().0
    }

    /// Verifies that the hash for an Orchard Vanilla bundle matches a fixed reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_txid_data_for_orchard_vanilla() {
        let bundle = generate_bundle::<OrchardVanilla>(BundleVersion::orchard_v2());
        let sighash = hash_bundle_txid_data(&bundle, TxVersion::V5).unwrap();
        assert_eq!(
            sighash.to_hex().as_str(),
            // Bundle hash for Orchard (vanilla) generated using
            // Zcash/Orchard commit: 9d89b504
            "0ac1e319f6761a8561b7bd3fc0907a5c73ed5590a6c210c4d39ffae1d5741875"
        );
    }

    // TODO Constance: add test for (Orchard, V3) and (Ironwood, V3)

    /// Verifies that the hash for an OrchardZSA bundle matches a fixed reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_txid_data_for_orchard_zsa() {
        let bundle = generate_bundle::<OrchardZSA>(BundleVersion::zsa());
        let sighash = hash_bundle_txid_data(&bundle, TxVersion::ZSA).unwrap();
        assert_eq!(
            sighash.to_hex().as_str(),
            "5c2d17a3466f7f90f1765241d9cea75d822966cd7adc105f36ab8862da6e2db2"
        );
    }

    fn generate_auth_bundle<FL: OrchardFlavor>(
        bundle_version: BundleVersion,
        tx_version: TxVersion,
    ) -> Bundle<Authorized, i64, FL> {
        let mut rng = StdRng::seed_from_u64(6);
        let pk = ProvingKey::build::<FL>(bundle_version.circuit_version());
        let bundle = generate_bundle(bundle_version)
            .create_proof(&pk, &mut rng)
            .unwrap();
        let sighash = bundle.commitment(tx_version).unwrap().into();
        bundle.prepare(rng, sighash).finalize().unwrap()
    }

    /// Verifies that the authorizing data commitment for an Orchard Vanilla bundle matches a fixed
    /// reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_auth_data_for_orchard_vanilla() {
        let bundle =
            generate_auth_bundle::<OrchardVanilla>(BundleVersion::orchard_v2(), TxVersion::V5);
        let orchard_auth_digest =
            hash_bundle_auth_data(&bundle, TxVersion::V5, test_sighash_info_for_kind).unwrap();
        assert_eq!(
            orchard_auth_digest.to_hex().as_str(),
            // Bundle hash for Orchard (vanilla) generated using
            // Zcash/Orchard commit: 82e0739
            "37d6c29faa98c2cb54420f3f7cac0477fdb105df1cdfde7adb7fbf68a24e3085"
        );
    }

    // TODO Constance: add test for (Orchard, V3) and (Ironwood, V3)

    /// Verifies that the authorizing data commitment for an OrchardZSA bundle matches a fixed
    /// reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_auth_data_for_orchard_zsa() {
        let bundle = generate_auth_bundle::<OrchardZSA>(BundleVersion::zsa(), TxVersion::ZSA);
        let orchard_auth_digest =
            hash_bundle_auth_data(&bundle, TxVersion::ZSA, test_sighash_info_for_kind).unwrap();
        assert_eq!(
            orchard_auth_digest.to_hex().as_str(),
            "31888a8b09c7b4e1362c69bb3de2f3bf8c2bf9cbc57c94ebc6c4e6a6bdf519ee"
        );
    }

    #[test]
    fn test_compact_size() {
        assert_eq!(get_compact_size(0), vec![0]);
        assert_eq!(get_compact_size(1), vec![1]);
        assert_eq!(get_compact_size(252), vec![252]);
        assert_eq!(get_compact_size(253), vec![253, 253, 0]);
        assert_eq!(get_compact_size(254), vec![253, 254, 0]);
        assert_eq!(get_compact_size(255), vec![253, 255, 0]);
        assert_eq!(get_compact_size(65535), vec![253, 255, 255]);
        assert_eq!(get_compact_size(65536), vec![254, 0, 0, 1, 0]);
        assert_eq!(get_compact_size(65537), vec![254, 1, 0, 1, 0]);
        assert_eq!(get_compact_size(33554432), vec![254, 0, 0, 0, 2]);
    }
}
