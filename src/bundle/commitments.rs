//! Utility functions for computing bundle commitments

use blake2b_simd::{Hash as Blake2bHash, Params, State};

use crate::{
    bundle::{Authorization, Authorized, Bundle},
    issuance::{IssueAuth, IssueBundle, Signed},
    note::AssetBase,
    note_encryption::{OrchardDomainCommon, MEMO_SIZE},
    orchard_flavor::{OrchardVanilla, OrchardZSA},
    value::NoteValue,
};

const ZCASH_ORCHARD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";
const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActCHash";
const ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActMHash";
const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActNHash";
const ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaHash";
const ZCASH_ORCHARD_ZSA_BURN_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcBurnHash";
const ZCASH_ORCHARD_ZSA_ISSUE_PERSONALIZATION: &[u8; 16] = b"ZTxIdSAIssueHash";
const ZCASH_ORCHARD_ZSA_ISSUE_ACTION_PERSONALIZATION: &[u8; 16] = b"ZTxIdIssuActHash";
const ZCASH_ORCHARD_ZSA_ISSUE_NOTE_PERSONALIZATION: &[u8; 16] = b"ZTxIdIAcNoteHash";
const ZCASH_ORCHARD_ZSA_ISSUE_SIG_PERSONALIZATION: &[u8; 16] = b"ZTxAuthZSAOrHash";

fn hasher(personal: &[u8; 16]) -> State {
    Params::new().hash_length(32).personal(personal).to_state()
}

/// Manages the hashing of ZSA burn-related data in transactions.
pub trait OrchardHash {
    /// Incorporates the hash of burn items into the main transaction hash.
    fn update_hash_with_burn(main_hasher: &mut State, burn_items: &[(AssetBase, NoteValue)]);
}

impl OrchardHash for OrchardVanilla {
    fn update_hash_with_burn(_main_hasher: &mut State, _burn_items: &[(AssetBase, NoteValue)]) {}
}

impl OrchardHash for OrchardZSA {
    fn update_hash_with_burn(main_hasher: &mut State, burn_items: &[(AssetBase, NoteValue)]) {
        let mut burn_hasher = hasher(ZCASH_ORCHARD_ZSA_BURN_HASH_PERSONALIZATION);
        for burn_item in burn_items {
            burn_hasher.update(&burn_item.0.to_bytes());
            burn_hasher.update(&burn_item.1.to_bytes());
        }
        main_hasher.update(burn_hasher.finalize().as_bytes());
    }
}

/// Write disjoint parts of each Orchard shielded action as 3 separate hashes:
/// * \[(nullifier, cmx, ephemeral_key, enc_ciphertext\[..52\])*\] personalized
///   with ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION
/// * \[enc_ciphertext\[52..564\]*\] (memo ciphertexts) personalized
///   with ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION
/// * \[(cv, rk, enc_ciphertext\[564..\], out_ciphertext)*\] personalized
///   with ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION
/// as defined in [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// Then, hash these together along with (flags, value_balance_orchard, anchor_orchard),
/// and potentially the burn fields, if it is an OrchardZSA action.
///
/// The final hash is personalized with ZCASH_ORCHARD_HASH_PERSONALIZATION.
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip226]: https://zips.z.cash/zip-0226 (for ZSA burn field hashing)
pub(crate) fn hash_bundle_txid_data<
    A: Authorization,
    V: Copy + Into<i64>,
    D: OrchardDomainCommon + OrchardHash,
>(
    bundle: &Bundle<A, V, D>,
) -> Blake2bHash {
    let mut h = hasher(ZCASH_ORCHARD_HASH_PERSONALIZATION);
    let mut ch = hasher(ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION);
    let mut mh = hasher(ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION);
    let mut nh = hasher(ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION);

    for action in bundle.actions().iter() {
        ch.update(&action.nullifier().to_bytes());
        ch.update(&action.cmx().to_bytes());
        ch.update(&action.encrypted_note().epk_bytes);
        ch.update(&action.encrypted_note().enc_ciphertext.as_ref()[..D::COMPACT_NOTE_SIZE]);

        mh.update(
            &action.encrypted_note().enc_ciphertext.as_ref()
                [D::COMPACT_NOTE_SIZE..D::COMPACT_NOTE_SIZE + MEMO_SIZE],
        );

        nh.update(&action.cv_net().to_bytes());
        nh.update(&<[u8; 32]>::from(action.rk()));
        nh.update(
            &action.encrypted_note().enc_ciphertext.as_ref()[D::COMPACT_NOTE_SIZE + MEMO_SIZE..],
        );
        nh.update(&action.encrypted_note().out_ciphertext);
    }

    h.update(ch.finalize().as_bytes());
    h.update(mh.finalize().as_bytes());
    h.update(nh.finalize().as_bytes());

    D::update_hash_with_burn(&mut h, &bundle.burn);

    h.update(&[bundle.flags().to_byte()]);
    h.update(&(*bundle.value_balance()).into().to_le_bytes());
    h.update(&bundle.anchor().to_bytes());
    h.finalize()
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
/// [zip227]: https://qed-it.github.io/zips/zip-0227
pub fn hash_issue_bundle_auth_empty() -> Blake2bHash {
    hasher(ZCASH_ORCHARD_ZSA_ISSUE_SIG_PERSONALIZATION).finalize()
}

/// Construct the commitment for an absent issue bundle as defined in
/// [ZIP-227: Issuance of Zcash Shielded Assets][zip227]
///
/// [zip227]: https://qed-it.github.io/zips/zip-0227
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
