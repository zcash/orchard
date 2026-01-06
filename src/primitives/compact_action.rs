//! Defines actions for Orchard shielded outputs and compact action for light clients.

use core::fmt;

use zcash_note_encryption::{note_bytes::NoteBytes, EphemeralKeyBytes, ShieldedOutput};

use crate::{
    action::Action,
    flavor::OrchardVanilla,
    note::{ExtractedNoteCommitment, Nullifier, Rho},
};

use super::{orchard_domain::OrchardDomain, orchard_primitives::OrchardPrimitives};

impl<A, Pr: OrchardPrimitives> ShieldedOutput<OrchardDomain<Pr>> for Action<A, Pr> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.encrypted_note().epk_bytes)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        self.cmx()
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx().to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&Pr::NoteCiphertextBytes> {
        Some(&self.encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(&self) -> Pr::CompactNoteCiphertextBytes {
        Pr::CompactNoteCiphertextBytes::from_slice(
            &self.encrypted_note().enc_ciphertext.as_ref()[..Pr::COMPACT_NOTE_SIZE],
        )
        .expect("Pr::CompactNoteCiphertextBytes should have size Pr::COMPACT_NOTE_SIZE")
    }
}

impl ShieldedOutput<OrchardDomain<OrchardVanilla>> for crate::pczt::Action {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.output().encrypted_note().epk_bytes)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        self.output().cmx()
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.output().cmx().to_bytes()
    }

    fn enc_ciphertext(
        &self,
    ) -> Option<&<OrchardVanilla as OrchardPrimitives>::NoteCiphertextBytes> {
        Some(&self.output().encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(
        &self,
    ) -> <OrchardVanilla as OrchardPrimitives>::CompactNoteCiphertextBytes {
        <OrchardVanilla as OrchardPrimitives>::CompactNoteCiphertextBytes::from_slice(
            &self.output().encrypted_note().enc_ciphertext.as_ref()
                [..OrchardVanilla::COMPACT_NOTE_SIZE],
        )
        .expect("ciphertext must be at least COMPACT_NOTE_SIZE bytes")
    }
}

/// A compact Action for light clients.
#[derive(Clone)]
pub struct CompactAction<Pr: OrchardPrimitives> {
    nullifier: Nullifier,
    cmx: ExtractedNoteCommitment,
    ephemeral_key: EphemeralKeyBytes,
    enc_ciphertext: Pr::CompactNoteCiphertextBytes,
}

impl<Pr: OrchardPrimitives> fmt::Debug for CompactAction<Pr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactAction")
    }
}

impl<A, Pr: OrchardPrimitives> From<&Action<A, Pr>> for CompactAction<Pr>
where
    Action<A, Pr>: ShieldedOutput<OrchardDomain<Pr>>,
{
    fn from(action: &Action<A, Pr>) -> Self {
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: action.ephemeral_key(),
            enc_ciphertext: action.enc_ciphertext_compact(),
        }
    }
}

impl<Pr: OrchardPrimitives> ShieldedOutput<OrchardDomain<Pr>> for CompactAction<Pr> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.ephemeral_key.0)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx.to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&Pr::NoteCiphertextBytes> {
        None
    }

    fn enc_ciphertext_compact(&self) -> Pr::CompactNoteCiphertextBytes {
        self.enc_ciphertext
    }
}

impl<Pr: OrchardPrimitives> CompactAction<Pr> {
    /// Create a CompactAction from its constituent parts
    pub fn from_parts(
        nullifier: Nullifier,
        cmx: ExtractedNoteCommitment,
        ephemeral_key: EphemeralKeyBytes,
        enc_ciphertext: Pr::CompactNoteCiphertextBytes,
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
    pub fn fake_compact_action<R: RngCore, Pr: OrchardPrimitives>(
        rng: &mut R,
        nf_old: Nullifier,
        recipient: Address,
        value: NoteValue,
        ovk: Option<OutgoingViewingKey>,
    ) -> (CompactAction<Pr>, Note) {
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
        let encryptor = NoteEncryption::<OrchardDomain<Pr>>::new(ovk, note, [0u8; MEMO_SIZE]);
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let ephemeral_key = OrchardDomain::<Pr>::epk_bytes(encryptor.epk());
        let enc_ciphertext = encryptor.encrypt_note_plaintext();

        (
            CompactAction {
                nullifier: nf_old,
                cmx,
                ephemeral_key,
                enc_ciphertext: Pr::CompactNoteCiphertextBytes::from_slice(
                    &enc_ciphertext.as_ref()[..Pr::COMPACT_NOTE_SIZE],
                )
                .unwrap(),
            },
            note,
        )
    }
}
