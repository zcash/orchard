//! Utility functions for computing bundle commitments

use blake2b_simd::{Hash as Blake2bHash, Params, State};

use crate::{
    bundle::{Authorization, Authorized, Bundle},
    domain::OrchardDomainCommon,
    issuance::{IssueAuth, IssueBundle, Signed},
};

pub(crate) const ZCASH_ORCHARD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";
pub(crate) const ZCASH_ORCHARD_ACTION_GROUPS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActGHash";
pub(crate) const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] =
    b"ZTxIdOrcActCHash";
pub(crate) const ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActMHash";
pub(crate) const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] =
    b"ZTxIdOrcActNHash";
pub(crate) const ZCASH_ORCHARD_ZSA_BURN_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcBurnHash";
const ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaHash";
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
/// [ZIP-226: Transfer and Burn of Zcash Shielded Assets][zip226]
/// for OrchardZSA
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip226]: https://zips.z.cash/zip-0226
pub(crate) fn hash_bundle_txid_data<
    A: Authorization,
    V: Copy + Into<i64>,
    D: OrchardDomainCommon,
>(
    bundle: &Bundle<A, V, D>,
) -> Blake2bHash {
    D::hash_bundle_txid_data(bundle)
}

/// Construct the commitment for the absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_txid_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_HASH_PERSONALIZATION).finalize()
}

/// Construct the commitment to the authorizing data of an
/// authorized bundle as defined in [ZIP-244: Transaction
/// Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub(crate) fn hash_bundle_auth_data<V, D: OrchardDomainCommon>(
    bundle: &Bundle<Authorized, V, D>,
) -> Blake2bHash {
    let mut h = hasher(ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION);
    h.update(bundle.authorization().proof().as_ref());
    for action in bundle.actions().iter() {
        h.update(&<[u8; 64]>::from(action.authorization()));
    }
    h.update(&<[u8; 64]>::from(
        bundle.authorization().binding_signature(),
    ));
    h.finalize()
}

/// Construct the commitment for an absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_auth_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION).finalize()
}

/// Construct the commitment for an absent issue bundle as defined in
/// [ZIP-227: Issuance of Zcash Shielded Assets][zip227]
///
/// [zip227]: https://zips.z.cash/zip-0227
pub fn hash_issue_bundle_auth_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_ZSA_ISSUE_SIG_PERSONALIZATION).finalize()
}

/// Construct the commitment for an absent issue bundle as defined in
/// [ZIP-227: Issuance of Zcash Shielded Assets][zip227]
///
/// [zip227]: https://zips.z.cash/zip-0227
pub fn hash_issue_bundle_txid_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_ZSA_ISSUE_PERSONALIZATION).finalize()
}

/// Construct the commitment for the issue bundle
pub(crate) fn hash_issue_bundle_txid_data<A: IssueAuth>(bundle: &IssueBundle<A>) -> Blake2bHash {
    let mut h = hasher(ZCASH_ORCHARD_ZSA_ISSUE_PERSONALIZATION);
    let mut ia = hasher(ZCASH_ORCHARD_ZSA_ISSUE_ACTION_PERSONALIZATION);

    for action in bundle.actions() {
        let mut ind = hasher(ZCASH_ORCHARD_ZSA_ISSUE_NOTE_PERSONALIZATION);
        for note in action.notes().iter() {
            ind.update(&note.recipient().to_raw_address_bytes());
            ind.update(&note.value().to_bytes());
            ind.update(&note.asset().to_bytes());
            ind.update(&note.rho().to_bytes());
            ind.update(note.rseed().as_bytes());
        }
        ia.update(ind.finalize().as_bytes());
        ia.update(action.asset_desc());
        ia.update(&[u8::from(action.is_finalized())]);
    }
    h.update(ia.finalize().as_bytes());
    h.update(&bundle.ik().to_bytes());
    h.finalize()
}

/// Construct the commitment to the authorizing data of an
/// authorized issue bundle
pub(crate) fn hash_issue_bundle_auth_data(bundle: &IssueBundle<Signed>) -> Blake2bHash {
    let mut h = hasher(ZCASH_ORCHARD_ZSA_ISSUE_SIG_PERSONALIZATION);
    h.update(&<[u8; 64]>::from(bundle.authorization().signature()));
    h.finalize()
}

#[cfg(test)]
mod tests {
    use crate::{
        builder::{Builder, BundleType, UnauthorizedBundle},
        bundle::commitments::hash_bundle_txid_data,
        keys::{FullViewingKey, Scope, SpendingKey},
        note::AssetBase,
        orchard_flavor::{OrchardFlavor, OrchardVanilla, OrchardZSA},
        value::NoteValue,
        Anchor,
    };
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
                None,
            )
            .unwrap();

        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(20),
                AssetBase::native(),
                None,
            )
            .unwrap();

        builder.build::<i64, FL>(rng).unwrap().0
    }

    /// Verify that the hash for an Orchard Vanilla bundle matches a fixed reference value
    /// to ensure consistency.
    #[test]
    fn test_hash_bundle_txid_data_for_orchard_vanilla() {
        let bundle = generate_bundle::<OrchardVanilla>(BundleType::DEFAULT_VANILLA);
        let sighash = hash_bundle_txid_data(&bundle);
        assert_eq!(
            sighash.to_hex().as_str(),
            // Bundle hash for Orchard (vanilla) generated using
            // Zcash/Orchard commit: 23a167e3972632586dc628ddbdd69d156dfd607b
            "cd6f8156a54473d411c738e781b4d601363990688a926a3335145575003bf4b8"
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
            "43cfaab1ffcd8d4752e5e7479fd619c769e3ab459b6f10bbba80533608f546b0"
        );
    }
}
