use memuse::DynamicUsage;

use crate::{
    note::{ExtractedNoteCommitment, TransmittedNoteCiphertext},
    primitives::redpallas::{self, SpendAuth},
};

/// An issue action applied to the global ledger.
///
/// Externally, this creates a zsa note (adding a commitment to the global ledger).
#[derive(Debug, Clone)]
pub struct IssueAction<A> {
    /// The issuer key for the note being created.
    ik: redpallas::VerificationKey<SpendAuth>,
    /// A commitment to the new note being created.
    cmx: ExtractedNoteCommitment,
    /// The transmitted note ciphertext.
    encrypted_note: TransmittedNoteCiphertext,
    /// The authorization for this action.
    authorization: A,
}

impl<T> IssueAction<T> {
    /// Constructs an `IssueAction` from its constituent parts.
    pub fn from_parts(
        ik: redpallas::VerificationKey<SpendAuth>,
        cmx: ExtractedNoteCommitment,
        encrypted_note: TransmittedNoteCiphertext,
        authorization: T,
    ) -> Self {
        IssueAction {
            ik,
            cmx,
            encrypted_note,
            authorization,
        }
    }

    /// Returns the issuer verification key for the note being created.
    pub fn ik(&self) -> &redpallas::VerificationKey<SpendAuth> {
        &self.ik
    }

    /// Returns the commitment to the new note being created.
    pub fn cmx(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    /// Returns the encrypted note ciphertext.
    pub fn encrypted_note(&self) -> &TransmittedNoteCiphertext {
        &self.encrypted_note
    }

    /// Returns the authorization for this action.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Transitions this issue action from one authorization state to another.
    pub fn map<U>(self, step: impl FnOnce(T) -> U) -> IssueAction<U> {
        IssueAction {
            ik: self.ik,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            authorization: step(self.authorization),
        }
    }

    /// Transitions this issue action from one authorization state to another.
    pub fn try_map<U, E>(self, step: impl FnOnce(T) -> Result<U, E>) -> Result<IssueAction<U>, E> {
        Ok(IssueAction {
            ik: self.ik,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            authorization: step(self.authorization)?,
        })
    }
}

impl DynamicUsage for IssueAction<redpallas::Signature<SpendAuth>> {
    #[inline(always)]
    fn dynamic_usage(&self) -> usize {
        0
    }

    #[inline(always)]
    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub(crate) mod testing {
    use rand::{rngs::StdRng, SeedableRng};
    use reddsa::orchard::SpendAuth;

    use proptest::prelude::*;

    use crate::{
        note::{
            commitment::ExtractedNoteCommitment,
            testing::arb_note, TransmittedNoteCiphertext,
        },
        primitives::redpallas::{
            self,
            testing::{arb_spendauth_signing_key, arb_spendauth_verification_key},
        },
        value::{NoteValue},
    };

    use super::IssueAction;

    prop_compose! {
        /// Generate an issue action without authorization data.
        pub fn arb_unauthorized_issue_action(output_value: NoteValue)(
            ik in arb_spendauth_verification_key(),
            note in arb_note(output_value),
        ) -> IssueAction<()> {
            let cmx = ExtractedNoteCommitment::from(note.commitment());

            // FIXME: make a real one from the note.
            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: [0u8; 32],
                enc_ciphertext: [0u8; 580],
                out_ciphertext: [0u8; 80]
            };
            IssueAction {
                ik,
                cmx,
                encrypted_note,
                authorization: ()
            }
        }
    }

    prop_compose! {
        /// Generate an issue action with invalid (random) authorization data.
        pub fn arb_issue_action(output_value: NoteValue)(
            sk in arb_spendauth_signing_key(),
            note in arb_note(output_value),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY),
            fake_sighash in prop::array::uniform32(prop::num::u8::ANY),
        ) -> IssueAction<redpallas::Signature<SpendAuth>> {
            let cmx = ExtractedNoteCommitment::from(note.commitment());

            // FIXME: make a real one from the note.
            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: [0u8; 32],
                enc_ciphertext: [0u8; 580],
                out_ciphertext: [0u8; 80]
            };

            let rng = StdRng::from_seed(rng_seed);

            IssueAction {
                ik: redpallas::VerificationKey::from(&sk),
                cmx,
                encrypted_note,
                authorization: sk.sign(rng, &fake_sighash),
            }
        }
    }
}
