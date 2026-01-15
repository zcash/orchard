//! Issuance-related commitment functions (ZSA feature)

use alloc::{collections::BTreeMap, vec::Vec};
use blake2b_simd::Hash as Blake2bHash;

use crate::{
    bundle::commitments::{get_compact_size, hasher},
    issuance::{sighash_versioning::IssueSighashVersion, IssueAuth, IssueBundle, Signed},
};

const ZCASH_ORCHARD_ZSA_ISSUE_PERSONALIZATION: &[u8; 16] = b"ZTxIdSAIssueHash";
const ZCASH_ORCHARD_ZSA_ISSUE_ACTION_PERSONALIZATION: &[u8; 16] = b"ZTxIdIssuActHash";
const ZCASH_ORCHARD_ZSA_ISSUE_NOTE_PERSONALIZATION: &[u8; 16] = b"ZTxIdIAcNoteHash";
const ZCASH_ORCHARD_ZSA_ISSUE_SIG_PERSONALIZATION: &[u8; 16] = b"ZTxAuthZSAOrHash";

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
        ia.update(&[u8::from(action.is_finalized())]);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        issuance::{
            auth::{IssueAuthKey, IssueValidatingKey, ZSASchnorr},
            compute_asset_desc_hash, AwaitingSighash, IssueInfo,
        },
        keys::{FullViewingKey, Scope, SpendingKey},
        note::Nullifier,
        value::NoteValue,
    };
    use nonempty::NonEmpty;
    use rand::{rngs::StdRng, SeedableRng};

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

        (bundle.update_rho(&first_nullifier, rng), isk)
    }

    /// Verifies that the `issuance_digest` of an IssueBundle matches a fixed reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_issue_bundle_txid_data() {
        let (bundle, _) = generate_issue_bundle();
        let issuance_digest = hash_issue_bundle_txid_data(&bundle);
        assert_eq!(
            issuance_digest.to_hex().as_str(),
            "ee70e3b61674fd0428ac0020cc4fc5819386e39c4eb3c63357c84c998195bcdb"
        );
    }

    /// Verifies that the `issuance_auth_digest` of an IssueBundle matches a fixed reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
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
            "6df77af7b5323d99376336b770e4c5b06ffc195de81ac7692d1b08b6eb19534d"
        );
    }
}
