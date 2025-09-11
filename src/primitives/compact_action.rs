//! Defines actions for Orchard shielded outputs and compact action for light clients.

// Review hint: this file is largely derived from src/note_encryption.rs

use core::fmt;

use zcash_note_encryption::{note_bytes::NoteBytes, EphemeralKeyBytes, ShieldedOutput};

use crate::{
    action::Action,
    note::{ExtractedNoteCommitment, Nullifier, Rho},
};

use super::{orchard_domain::OrchardDomain, orchard_primitives::OrchardPrimitives};

impl<A, P: OrchardPrimitives> ShieldedOutput<OrchardDomain<P>> for Action<A, P> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.encrypted_note().epk_bytes)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        self.cmx()
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx().to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&P::NoteCiphertextBytes> {
        Some(&self.encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(&self) -> P::CompactNoteCiphertextBytes {
        P::CompactNoteCiphertextBytes::from_slice(
            &self.encrypted_note().enc_ciphertext.as_ref()[..P::COMPACT_NOTE_SIZE],
        )
        .unwrap()
    }
}

/// A compact Action for light clients.
#[derive(Clone)]
pub struct CompactAction<P: OrchardPrimitives> {
    nullifier: Nullifier,
    cmx: ExtractedNoteCommitment,
    ephemeral_key: EphemeralKeyBytes,
    enc_ciphertext: P::CompactNoteCiphertextBytes,
}

impl<P: OrchardPrimitives> fmt::Debug for CompactAction<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactAction")
    }
}

impl<A, P: OrchardPrimitives> From<&Action<A, P>> for CompactAction<P>
where
    Action<A, P>: ShieldedOutput<OrchardDomain<P>>,
{
    fn from(action: &Action<A, P>) -> Self {
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: action.ephemeral_key(),
            enc_ciphertext: action.enc_ciphertext_compact(),
        }
    }
}

impl<P: OrchardPrimitives> ShieldedOutput<OrchardDomain<P>> for CompactAction<P> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.ephemeral_key.0)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx.to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&P::NoteCiphertextBytes> {
        None
    }

    fn enc_ciphertext_compact(&self) -> P::CompactNoteCiphertextBytes {
        P::CompactNoteCiphertextBytes::from_slice(self.enc_ciphertext.as_ref()).unwrap()
    }
}

impl<P: OrchardPrimitives> CompactAction<P> {
    /// Create a CompactAction from its constituent parts
    pub fn from_parts(
        nullifier: Nullifier,
        cmx: ExtractedNoteCommitment,
        ephemeral_key: EphemeralKeyBytes,
        enc_ciphertext: P::CompactNoteCiphertextBytes,
    ) -> Self {
        Self {
            nullifier,
            cmx,
            ephemeral_key,
            enc_ciphertext,
        }
    }

    /// Returns the nullifier of the note being spent.
    pub fn nullifier(&self) -> Nullifier {
        self.nullifier
    }

    /// Returns the commitment to the new note being created.
    pub fn cmx(&self) -> ExtractedNoteCommitment {
        self.cmx
    }

    /// Obtains the [`Rho`] value that was used to construct the new note being created.
    pub fn rho(&self) -> Rho {
        Rho::from_nf_old(self.nullifier)
    }
}

/// Utilities for constructing test data.
#[cfg(feature = "test-dependencies")]
pub mod testing {
    use rand::RngCore;

    use zcash_note_encryption::{note_bytes::NoteBytes, Domain, NoteEncryption};

    use crate::{
        address::Address,
        keys::OutgoingViewingKey,
        note::{AssetBase, ExtractedNoteCommitment, Note, Nullifier, RandomSeed, Rho},
        primitives::zcash_note_encryption_domain::MEMO_SIZE,
        value::NoteValue,
    };

    use super::{CompactAction, OrchardDomain, OrchardPrimitives};

    /// Creates a fake `CompactAction` paying the given recipient the specified value.
    ///
    /// Returns the `CompactAction` and the new note.
    pub fn fake_compact_action<R: RngCore, P: OrchardPrimitives>(
        rng: &mut R,
        nf_old: Nullifier,
        recipient: Address,
        value: NoteValue,
        ovk: Option<OutgoingViewingKey>,
    ) -> (CompactAction<P>, Note) {
        let rho = Rho::from_nf_old(nf_old);
        let rseed = {
            loop {
                let mut bytes = [0; 32];
                rng.fill_bytes(&mut bytes);
                let rseed = RandomSeed::from_bytes(bytes, &rho);
                if rseed.is_some().into() {
                    break rseed.unwrap();
                }
            }
        };
        let note = Note::from_parts(recipient, value, AssetBase::native(), rho, rseed).unwrap();
        let encryptor = NoteEncryption::<OrchardDomain<P>>::new(ovk, note, [0u8; MEMO_SIZE]);
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let ephemeral_key = OrchardDomain::<P>::epk_bytes(encryptor.epk());
        let enc_ciphertext = encryptor.encrypt_note_plaintext();

        (
            CompactAction {
                nullifier: nf_old,
                cmx,
                ephemeral_key,
                enc_ciphertext: P::CompactNoteCiphertextBytes::from_slice(
                    &enc_ciphertext.as_ref()[..P::COMPACT_NOTE_SIZE],
                )
                .unwrap(),
            },
            note,
        )
    }
}
