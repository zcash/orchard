//! Utility functions for computing bundle commitments

use alloc::{collections::BTreeMap, vec::Vec};
use blake2b_simd::{Hash as Blake2bHash, Params, State};

use crate::{
    bundle::{Authorization, Authorized, Bundle},
    issuance::{
        sighash_versioning::IssueSighashVersion,
        {IssueAuth, IssueBundle, Signed},
    },
    primitives::OrchardPrimitives,
    sighash_versioning::OrchardSighashVersion,
};

pub(crate) const ZCASH_ORCHARD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";
pub(crate) const ZCASH_ORCHARD_ACTION_GROUPS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActGHash";
pub(crate) const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] =
    b"ZTxIdOrcActCHash";
pub(crate) const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION_V6: &[u8; 16] =
    b"ZTxId6OActC_Hash";
pub(crate) const ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActMHash";
pub(crate) const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] =
    b"ZTxIdOrcActNHash";
pub(crate) const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION_V6: &[u8; 16] =
    b"ZTxId6OActN_Hash";
pub(crate) const ZCASH_ORCHARD_ZSA_BURN_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcBurnHash";
pub(crate) const ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaHash";
pub(crate) const ZCASH_ORCHARD_ACTION_GROUPS_SIGS_HASH_PERSONALIZATION: &[u8; 16] =
    b"ZTxAuthOrcAGHash";
pub(crate) const ZCASH_ORCHARD_SPEND_AUTH_SIGS_HASH_PERSONALIZATION: &[u8; 16] =
    b"ZTxAuthOrSASHash";

const ZCASH_ORCHARD_ZSA_ISSUE_PERSONALIZATION: &[u8; 16] = b"ZTxIdSAIssueHash";
const ZCASH_ORCHARD_ZSA_ISSUE_ACTION_PERSONALIZATION: &[u8; 16] = b"ZTxIdIssuActHash";
const ZCASH_ORCHARD_ZSA_ISSUE_NOTE_PERSONALIZATION: &[u8; 16] = b"ZTxIdIAcNoteHash";
const ZCASH_ORCHARD_ZSA_ISSUE_SIG_PERSONALIZATION: &[u8; 16] = b"ZTxAuthZSAOrHash";

pub(crate) fn hasher(personal: &[u8; 16]) -> State {
    Params::new().hash_length(32).personal(personal).to_state()
}

/// Evaluate `orchard_digest` for the bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
/// for OrchardVanilla and as defined in
/// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
/// for OrchardZSA
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip246]: https://zips.z.cash/zip-0246
pub(crate) fn hash_bundle_txid_data<
    A: Authorization,
    V: Copy + Into<i64>,
    Pr: OrchardPrimitives,
>(
    bundle: &Bundle<A, V, Pr>,
) -> Blake2bHash {
    Pr::hash_bundle_txid_data(bundle)
}

/// Construct the commitment for the absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_txid_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_HASH_PERSONALIZATION).finalize()
}

/// Construct the `orchard_auth_digest` commitment to the authorizing data of an
/// authorized bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
/// for OrchardVanilla and as defined in
/// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
/// for OrchardZSA
///
/// The `sighash_version_map` provides the mapping from each
/// `OrchardSighashVersion` to the corresponding `SighashInfo`
/// encoding.
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip246]: https://zips.z.cash/zip-0246
pub(crate) fn hash_bundle_auth_data<V, Pr: OrchardPrimitives>(
    bundle: &Bundle<Authorized, V, Pr>,
    sighash_version_map: &BTreeMap<OrchardSighashVersion, Vec<u8>>,
) -> Blake2bHash {
    Pr::hash_bundle_auth_data(bundle, sighash_version_map)
}

/// Construct the `orchard_auth_digest` commitment for an absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_auth_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION).finalize()
}

/// Construct the `issuance_digest` commitment for an absent issue bundle as defined in
/// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
///
/// [zip246]: https://zips.z.cash/zip-0246
pub fn hash_issue_bundle_txid_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_ZSA_ISSUE_PERSONALIZATION).finalize()
}

/// Construct the `issuance_digest` commitment for the issue bundle as defined in
/// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
///
/// [zip246]: https://zips.z.cash/zip-0246
pub(crate) fn hash_issue_bundle_txid_data<A: IssueAuth>(bundle: &IssueBundle<A>) -> Blake2bHash {
    let mut h = hasher(ZCASH_ORCHARD_ZSA_ISSUE_PERSONALIZATION);

    let ik_encoding = bundle.ik().encode();
    h.update(&get_compact_size(ik_encoding.len()));
    h.update(&ik_encoding);

    let mut ia = hasher(ZCASH_ORCHARD_ZSA_ISSUE_ACTION_PERSONALIZATION);
    for action in bundle.actions() {
        ia.update(action.asset_desc_hash());

        let mut ind = hasher(ZCASH_ORCHARD_ZSA_ISSUE_NOTE_PERSONALIZATION);
        for note in action.notes().iter() {
            ind.update(&note.recipient().to_raw_address_bytes());
            ind.update(&note.value().to_bytes());
            ind.update(&note.rho().to_bytes());
            ind.update(note.rseed().as_bytes());
        }
        ia.update(ind.finalize().as_bytes());
        ia.update(&[action.flags().to_byte()]);
    }
    h.update(ia.finalize().as_bytes());
    h.finalize()
}

/// Construct the `issuance_auth_digest` commitment for an absent issue bundle as defined in
/// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
///
/// [zip246]: https://zips.z.cash/zip-0246
pub fn hash_issue_bundle_auth_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_ZSA_ISSUE_SIG_PERSONALIZATION).finalize()
}

/// Construct the `issuance_auth_digest` commitment to the authorizing data of an
/// authorized issue bundle as defined in
/// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
///
/// The `sighash_version_map` provides the mapping from each
/// `IssueSighashVersion` to the corresponding `SighashInfo`
/// encoding.
///
/// [zip246]: https://zips.z.cash/zip-0246
pub(crate) fn hash_issue_bundle_auth_data(
    bundle: &IssueBundle<Signed>,
    sighash_version_map: &BTreeMap<IssueSighashVersion, Vec<u8>>,
) -> Blake2bHash {
    let mut h = hasher(ZCASH_ORCHARD_ZSA_ISSUE_SIG_PERSONALIZATION);
    let version_bytes = sighash_version_map
        .get(bundle.authorization().signature().version())
        .expect("Unknown issue sighash version.");
    h.update(&get_compact_size(version_bytes.len()));
    h.update(version_bytes);

    let sig_enc = bundle.authorization().signature().sig().encode();
    assert_eq!(sig_enc.len(), 65);
    assert_eq!(sig_enc[0], 0x00); // ZIP-230: algorithm byte must be 0x00
    h.update(&get_compact_size(sig_enc.len()));
    h.update(&sig_enc);
    h.finalize()
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

#[cfg(test)]
mod tests {
    use crate::{
        builder::{Builder, BundleType, UnauthorizedBundle},
        bundle::{
            commitments::{
                get_compact_size, hash_bundle_auth_data, hash_bundle_txid_data,
                hash_issue_bundle_auth_data, hash_issue_bundle_txid_data,
            },
            Authorized, Bundle,
        },
        circuit::ProvingKey,
        flavor::{OrchardFlavor, OrchardVanilla, OrchardZSA},
        issuance::{
            auth::{IssueAuthKey, IssueValidatingKey, ZSASchnorr},
            sighash_versioning::IssueSighashVersion,
            {compute_asset_desc_hash, AwaitingSighash, IssueBundle, IssueInfo},
        },
        keys::{FullViewingKey, Scope, SpendingKey},
        note::{AssetBase, Nullifier},
        sighash_versioning::OrchardSighashVersion,
        value::NoteValue,
        Anchor,
    };
    use alloc::collections::BTreeMap;
    use nonempty::NonEmpty;
    use rand::{rngs::StdRng, SeedableRng};

    fn generate_bundle<FL: OrchardFlavor>(bundle_type: BundleType) -> UnauthorizedBundle<i64, FL> {
        let rng = StdRng::seed_from_u64(5);

        let sk = SpendingKey::from_bytes([7; 32]).unwrap();
        let recipient = FullViewingKey::from(&sk).address_at(0u32, Scope::External);

        let mut builder = Builder::new(bundle_type, Anchor::from_bytes([0; 32]).unwrap());
        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(10),
                AssetBase::native(),
                [0u8; 512],
            )
            .unwrap();

        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(20),
                AssetBase::native(),
                [0u8; 512],
            )
            .unwrap();

        builder.build::<i64, FL>(rng).unwrap().0
    }

    /// Verify that the hash for an Orchard Vanilla bundle matches a fixed reference value
    /// to ensure consistency.
    #[test]
    fn test_hash_bundle_txid_data_for_orchard_vanilla() {
        let bundle = generate_bundle::<OrchardVanilla>(BundleType::DEFAULT);
        let sighash = hash_bundle_txid_data(&bundle);
        assert_eq!(
            sighash.to_hex().as_str(),
            "f3ea89ea2b1e17b3313a6f2f9e4e47c21eec1574902f5ea6961227e1eaed2327"
        );
    }

    /// Verify that the hash for an OrchardZSA bundle matches a fixed reference value
    /// to ensure consistency.
    #[test]
    fn test_hash_bundle_txid_data_for_orchard_zsa() {
        let bundle = generate_bundle::<OrchardZSA>(BundleType::DEFAULT_ZSA);
        let sighash = hash_bundle_txid_data(&bundle);
        assert_eq!(
            sighash.to_hex().as_str(),
            "a0d843b7278788e3b47dc9fe1e1da227a94898b7111d76514a87df486d32773c"
        );
    }

    fn generate_auth_bundle<FL: OrchardFlavor>(
        bundle_type: BundleType,
    ) -> Bundle<Authorized, i64, FL> {
        let mut rng = StdRng::seed_from_u64(6);
        let pk = ProvingKey::build::<FL>();
        let bundle = generate_bundle(bundle_type)
            .create_proof(&pk, &mut rng)
            .unwrap();
        let sighash = bundle.commitment().into();
        bundle.prepare(rng, sighash).finalize().unwrap()
    }

    /// Verify that the authorizing data commitment for an Orchard Vanilla bundle matches a fixed
    /// reference value to ensure consistency.
    #[test]
    fn test_hash_bundle_auth_data_for_orchard_vanilla() {
        let bundle = generate_auth_bundle::<OrchardVanilla>(BundleType::DEFAULT);
        let orchard_auth_digest = hash_bundle_auth_data(&bundle, &BTreeMap::new());
        assert_eq!(
            orchard_auth_digest.to_hex().as_str(),
            "c99aa5a33fd4e7b78de0ee846397e2eb0da3a5d176e6df57d0401c49f51d7295"
        );
    }

    /// Verify that the authorizing data commitment for an OrchardZSA bundle matches a fixed
    /// reference value to ensure consistency.
    #[test]
    fn test_hash_bundle_auth_data_for_orchard_zsa() {
        let mut sighash_version_map = BTreeMap::new();
        sighash_version_map.insert(OrchardSighashVersion::V0, vec![0]);

        let bundle = generate_auth_bundle::<OrchardZSA>(BundleType::DEFAULT_ZSA);
        let orchard_auth_digest = hash_bundle_auth_data(&bundle, &sighash_version_map);
        assert_eq!(
            orchard_auth_digest.to_hex().as_str(),
            "9d47819082f2323b30ceabe0fea993b39541cc0e62a8be6e1bc2a19840b0d9ab"
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

    fn generate_issue_bundle() -> (IssueBundle<AwaitingSighash>, IssueAuthKey<ZSASchnorr>) {
        let mut rng = StdRng::seed_from_u64(5);

        let isk = IssueAuthKey::random(&mut rng);
        let ik = IssueValidatingKey::from(&isk);
        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let recipient = fvk.address_at(0u32, Scope::External);
        let first_nullifier = Nullifier::dummy(&mut rng);

        let asset_desc_hash_1 =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"first asset").unwrap());
        let asset_desc_hash_2 =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"second asset").unwrap());

        let (mut bundle, asset) = IssueBundle::new(
            ik,
            asset_desc_hash_1,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            &mut rng,
        );

        let another_asset = bundle
            .add_recipient(
                asset_desc_hash_1,
                recipient,
                NoteValue::from_raw(10),
                false,
                &mut rng,
            )
            .unwrap();
        assert_eq!(asset, another_asset);

        let third_asset = bundle
            .add_recipient(
                asset_desc_hash_2,
                recipient,
                NoteValue::from_raw(15),
                true,
                &mut rng,
            )
            .unwrap();
        assert_ne!(asset, third_asset);

        (bundle.update_rho(&first_nullifier), isk)
    }

    /// Verify that the `issuance_digest` of an IssueBundle matches a fixed reference value
    /// to ensure consistency.
    #[test]
    fn test_hash_issue_bundle_txid_data() {
        let (bundle, _) = generate_issue_bundle();
        let issuance_digest = hash_issue_bundle_txid_data(&bundle);
        assert_eq!(
            issuance_digest.to_hex().as_str(),
            "7d7e9b66cee8896453aa7dffdbe885b880b700a49cfff947ab1503a2407b5e1b"
        );
    }

    /// Verify that the `issuance_auth_digest` of an IssueBundle matches a fixed reference value
    /// to ensure consistency.
    #[test]
    fn test_hash_issue_bundle_auth_data() {
        let (bundle, isk) = generate_issue_bundle();
        let issuance_digest = bundle.commitment().into();
        let signed_bundle = bundle.prepare(issuance_digest).sign(&isk).unwrap();

        let mut sighash_version_map = BTreeMap::new();
        sighash_version_map.insert(IssueSighashVersion::V0, vec![0]);

        let issuance_auth_digest =
            hash_issue_bundle_auth_data(&signed_bundle, &sighash_version_map);
        assert_eq!(
            issuance_auth_digest.to_hex().as_str(),
            "b0e465381e86b4462403723283e75b5b1928110cf2a45a0602d5a5037f07c9ad"
        );
    }
}
