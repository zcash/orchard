//! Defines actions for Orchard shielded outputs and compact action for light clients.

use std::fmt;

use zcash_note_encryption_zsa::{note_bytes::NoteBytes, EphemeralKeyBytes, ShieldedOutput};

use crate::{
    action::Action,
    note::{ExtractedNoteCommitment, Nullifier, Rho},
};

use super::orchard_domain::{OrchardDomain, OrchardDomainCommon};

impl<A, D: OrchardDomainCommon> ShieldedOutput<OrchardDomain<D>> for Action<A, D> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.encrypted_note().epk_bytes)
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx().to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&D::NoteCiphertextBytes> {
        Some(&self.encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(&self) -> D::CompactNoteCiphertextBytes {
        D::CompactNoteCiphertextBytes::from_slice(
            &self.encrypted_note().enc_ciphertext.as_ref()[..D::COMPACT_NOTE_SIZE],
        )
        .unwrap()
    }
}

/// A compact Action for light clients.
#[derive(Clone)]
pub struct CompactAction<D: OrchardDomainCommon> {
    nullifier: Nullifier,
    cmx: ExtractedNoteCommitment,
    ephemeral_key: EphemeralKeyBytes,
    enc_ciphertext: D::CompactNoteCiphertextBytes,
}

impl<D: OrchardDomainCommon> fmt::Debug for CompactAction<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactAction")
    }
}

impl<A, D: OrchardDomainCommon> From<&Action<A, D>> for CompactAction<D>
where
    Action<A, D>: ShieldedOutput<OrchardDomain<D>>,
{
    fn from(action: &Action<A, D>) -> Self {
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: action.ephemeral_key(),
            enc_ciphertext: action.enc_ciphertext_compact(),
        }
    }
}

impl<D: OrchardDomainCommon> ShieldedOutput<OrchardDomain<D>> for CompactAction<D> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.ephemeral_key.0)
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx.to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&D::NoteCiphertextBytes> {
        None
    }

    fn enc_ciphertext_compact(&self) -> D::CompactNoteCiphertextBytes {
        D::CompactNoteCiphertextBytes::from_slice(self.enc_ciphertext.as_ref()).unwrap()
    }
}

impl<D: OrchardDomainCommon> CompactAction<D> {
    /// Create a CompactAction from its constituent parts
    pub fn from_parts(
        nullifier: Nullifier,
        cmx: ExtractedNoteCommitment,
        ephemeral_key: EphemeralKeyBytes,
        enc_ciphertext: D::CompactNoteCiphertextBytes,
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

    use zcash_note_encryption_zsa::{note_bytes::NoteBytes, Domain, NoteEncryption};

    use crate::{
        address::Address,
        domain::zcash_note_encryption_domain::MEMO_SIZE,
        keys::OutgoingViewingKey,
        note::{AssetBase, ExtractedNoteCommitment, Note, Nullifier, RandomSeed, Rho},
        value::NoteValue,
    };

    use super::{CompactAction, OrchardDomain, OrchardDomainCommon};

    /// Creates a fake `CompactAction` paying the given recipient the specified value.
    ///
    /// Returns the `CompactAction` and the new note.
    pub fn fake_compact_action<R: RngCore, D: OrchardDomainCommon>(
        rng: &mut R,
        nf_old: Nullifier,
        recipient: Address,
        value: NoteValue,
        ovk: Option<OutgoingViewingKey>,
    ) -> (CompactAction<D>, Note) {
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
        let encryptor = NoteEncryption::<OrchardDomain<D>>::new(ovk, note, [0u8; MEMO_SIZE]);
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let ephemeral_key = OrchardDomain::<D>::epk_bytes(encryptor.epk());
        let enc_ciphertext = encryptor.encrypt_note_plaintext();

        (
            CompactAction {
                nullifier: nf_old,
                cmx,
                ephemeral_key,
                enc_ciphertext: D::CompactNoteCiphertextBytes::from_slice(
                    &enc_ciphertext.as_ref()[..D::COMPACT_NOTE_SIZE],
                )
                .unwrap(),
            },
            note,
        )
    }
}
