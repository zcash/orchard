//! The OrchardDomain trait represents the difference between the `OrchardVanilla` and the `OrchardZSA`
//! encryption and decryption procedures.

use core::fmt;

use zcash_note_encryption_zsa::{note_bytes::NoteBytes, AEAD_TAG_SIZE, MEMO_SIZE};

use crate::{
    action::Action,
    note::{AssetBase, Rho},
    Note,
};

use super::{compact_action::CompactAction, domain::Memo};

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
