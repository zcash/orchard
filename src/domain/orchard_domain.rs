//! The OrchardDomain trait represents the difference between the `OrchardVanilla` and the
//! `OrchardZSA` commitment, encryption and decryption procedures.

use core::fmt;

use blake2b_simd::{Hash as Blake2bHash, State};
use zcash_note_encryption_zsa::{note_bytes::NoteBytes, AEAD_TAG_SIZE};

use crate::{
    action::Action,
    bundle::{
        commitments::{
            hasher, ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
            ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
            ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        },
        Authorization, Authorized,
    },
    domain::{
        compact_action::CompactAction,
        zcash_note_encryption_domain::{Memo, MEMO_SIZE},
    },
    note::{AssetBase, Rho},
    Bundle, Note,
};

/// Represents the Orchard protocol domain specifics required for note encryption and decryption.
pub trait OrchardDomainCommon: fmt::Debug + Clone {
    /// The size of a compact note, specific to the Orchard protocol.
    const COMPACT_NOTE_SIZE: usize;

    /// The size of a note plaintext, including memo and other metadata.
    const NOTE_PLAINTEXT_SIZE: usize = Self::COMPACT_NOTE_SIZE + MEMO_SIZE;

    /// The size of an encrypted note ciphertext, accounting for additional AEAD tag space.
    const ENC_CIPHERTEXT_SIZE: usize = Self::NOTE_PLAINTEXT_SIZE + AEAD_TAG_SIZE;

    /// The raw bytes of a note plaintext.
    type NotePlaintextBytes: NoteBytes;
    /// The raw bytes of an encrypted note plaintext.
    type NoteCiphertextBytes: NoteBytes;
    /// The raw bytes of a compact note.
    type CompactNotePlaintextBytes: NoteBytes;
    /// The raw bytes of an encrypted compact note.
    type CompactNoteCiphertextBytes: NoteBytes;

    /// Builds NotePlaintextBytes from Note and Memo.
    fn build_note_plaintext_bytes(note: &Note, memo: &Memo) -> Self::NotePlaintextBytes;

    /// Extracts the asset from the note plaintext.
    fn extract_asset(plaintext: &Self::CompactNotePlaintextBytes) -> Option<AssetBase>;

    /// Evaluate `orchard_digest` for the bundle as defined in
    /// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
    /// for OrchardVanilla and as defined in
    /// [ZIP-226: Transfer and Burn of Zcash Shielded Assets][zip226]
    /// for OrchardZSA
    ///
    /// [zip244]: https://zips.z.cash/zip-0244
    /// [zip226]: https://zips.z.cash/zip-0226
    fn hash_bundle_txid_data<A: Authorization, V: Copy + Into<i64>>(
        bundle: &Bundle<A, V, Self>,
    ) -> Blake2bHash;

    /// Incorporates the hash of:
    /// orchard_actions_compact_digest,
    /// orchard_actions_memos_digest,
    /// orchard_actions_noncompact_digest
    /// into the hasher.
    ///
    /// More precisely, write disjoint parts of each Orchard shielded action as 3 separate hashes:
    /// * \[(nullifier, cmx, ephemeral_key, enc_ciphertext\[..52\])*\] personalized
    ///   with ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION
    /// * \[enc_ciphertext\[52..564\]*\] (memo ciphertexts) personalized
    ///   with ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION
    /// * \[(cv, rk, enc_ciphertext\[564..\], out_ciphertext)*\] personalized
    ///   with ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION
    /// as defined in [ZIP-244: Transaction Identifier Non-Malleability][zip244]
    ///
    /// [zip244]: https://zips.z.cash/zip-0244
    fn update_hash_with_actions<A: Authorization, V: Copy + Into<i64>>(
        main_hasher: &mut State,
        bundle: &Bundle<A, V, Self>,
    ) {
        let mut ch = hasher(ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION);
        let mut mh = hasher(ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION);
        let mut nh = hasher(ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION);

        for action in bundle.actions().iter() {
            ch.update(&action.nullifier().to_bytes());
            ch.update(&action.cmx().to_bytes());
            ch.update(&action.encrypted_note().epk_bytes);
            ch.update(&action.encrypted_note().enc_ciphertext.as_ref()[..Self::COMPACT_NOTE_SIZE]);

            mh.update(
                &action.encrypted_note().enc_ciphertext.as_ref()
                    [Self::COMPACT_NOTE_SIZE..Self::COMPACT_NOTE_SIZE + MEMO_SIZE],
            );

            nh.update(&action.cv_net().to_bytes());
            nh.update(&<[u8; 32]>::from(action.rk()));
            nh.update(
                &action.encrypted_note().enc_ciphertext.as_ref()
                    [Self::COMPACT_NOTE_SIZE + MEMO_SIZE..],
            );
            nh.update(&action.encrypted_note().out_ciphertext);
        }

        main_hasher.update(ch.finalize().as_bytes());
        main_hasher.update(mh.finalize().as_bytes());
        main_hasher.update(nh.finalize().as_bytes());
    }

    /// Evaluate `orchard_auth_digest` for the bundle as defined in
    /// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
    /// for OrchardVanilla and as defined in
    /// [ZIP-226: Transfer and Burn of Zcash Shielded Assets][zip226]
    /// for OrchardZSA
    ///
    /// [zip244]: https://zips.z.cash/zip-0244
    /// [zip226]: https://zips.z.cash/zip-0226
    fn hash_bundle_auth_data<V>(bundle: &Bundle<Authorized, V, Self>) -> Blake2bHash;
}

/// Orchard-specific note encryption logic.
#[derive(Debug, Clone)]
pub struct OrchardDomain<D: OrchardDomainCommon> {
    /// A parameter needed to generate the nullifier.
    pub rho: Rho,
    phantom: std::marker::PhantomData<D>,
}

impl<D: OrchardDomainCommon> OrchardDomain<D> {
    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_action<T>(act: &Action<T, D>) -> Self {
        Self {
            rho: act.rho(),
            phantom: Default::default(),
        }
    }

    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_compact_action(act: &CompactAction<D>) -> Self {
        Self {
            rho: act.rho(),
            phantom: Default::default(),
        }
    }

    /// Constructs a domain from a rho.
    #[cfg(test)]
    pub fn for_rho(rho: Rho) -> Self {
        Self {
            rho,
            phantom: Default::default(),
        }
    }
}
