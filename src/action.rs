use memuse::DynamicUsage;

use crate::{
    domain::OrchardDomainCommon,
    note::{ExtractedNoteCommitment, Nullifier, Rho, TransmittedNoteCiphertext},
    primitives::redpallas::{self, SpendAuth},
    value::ValueCommitment,
};

/// An action applied to the global ledger.
///
/// This both creates a note (adding a commitment to the global ledger), and consumes
/// some note created prior to this action (adding a nullifier to the global ledger).
#[derive(Debug, Clone)]
pub struct Action<A, D: OrchardDomainCommon> {
    /// The nullifier of the note being spent.
    nf: Nullifier,
    /// The randomized verification key for the note being spent.
    rk: redpallas::VerificationKey<SpendAuth>,
    /// A commitment to the new note being created.
    cmx: ExtractedNoteCommitment,
    /// The transmitted note ciphertext.
    encrypted_note: TransmittedNoteCiphertext<D>,
    /// A commitment to the net value created or consumed by this action.
    cv_net: ValueCommitment,
    /// The authorization for this action.
    authorization: A,
}

impl<A, D: OrchardDomainCommon> Action<A, D> {
    /// Constructs an `Action` from its constituent parts.
    pub fn from_parts(
        nf: Nullifier,
        rk: redpallas::VerificationKey<SpendAuth>,
        cmx: ExtractedNoteCommitment,
        encrypted_note: TransmittedNoteCiphertext<D>,
        cv_net: ValueCommitment,
        authorization: A,
    ) -> Self {
        Action {
            nf,
            rk,
            cmx,
            encrypted_note,
            cv_net,
            authorization,
        }
    }

    /// Returns the nullifier of the note being spent.
    pub fn nullifier(&self) -> &Nullifier {
        &self.nf
    }

    /// Returns the randomized verification key for the note being spent.
    pub fn rk(&self) -> &redpallas::VerificationKey<SpendAuth> {
        &self.rk
    }

    /// Returns the commitment to the new note being created.
    pub fn cmx(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    /// Returns the encrypted note ciphertext.
    pub fn encrypted_note(&self) -> &TransmittedNoteCiphertext<D> {
        &self.encrypted_note
    }

    /// Obtains the [`Rho`] value that was used to construct the new note being created.
    pub fn rho(&self) -> Rho {
        Rho::from_nf_old(self.nf)
    }

    /// Returns the commitment to the net value created or consumed by this action.
    pub fn cv_net(&self) -> &ValueCommitment {
        &self.cv_net
    }

    /// Returns the authorization for this action.
    pub fn authorization(&self) -> &A {
        &self.authorization
    }

    /// Transitions this action from one authorization state to another.
    pub fn map<U>(self, step: impl FnOnce(A) -> U) -> Action<U, D> {
        Action {
            nf: self.nf,
            rk: self.rk,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            cv_net: self.cv_net,
            authorization: step(self.authorization),
        }
    }

    /// Transitions this action from one authorization state to another.
    pub fn try_map<U, E>(self, step: impl FnOnce(A) -> Result<U, E>) -> Result<Action<U, D>, E> {
        Ok(Action {
            nf: self.nf,
            rk: self.rk,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            cv_net: self.cv_net,
            authorization: step(self.authorization)?,
        })
    }
}

impl<D: OrchardDomainCommon> DynamicUsage for Action<redpallas::Signature<SpendAuth>, D> {
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

    use zcash_note_encryption_zsa::NoteEncryption;

    use crate::{
        domain::{OrchardDomain, OrchardDomainCommon},
        note::{
            asset_base::testing::arb_asset_base, commitment::ExtractedNoteCommitment,
            nullifier::testing::arb_nullifier, testing::arb_note, Note, TransmittedNoteCiphertext,
        },
        primitives::redpallas::{
            self,
            testing::{arb_spendauth_signing_key, arb_spendauth_verification_key},
        },
        value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
    };

    use super::Action;

    /// `ActionArb` adapts `arb_...` functions for both Vanilla and ZSA Orchard protocol flavors
    /// in property-based testing, addressing proptest crate limitations.
    #[derive(Debug)]
    pub struct ActionArb<D: OrchardDomainCommon> {
        phantom: std::marker::PhantomData<D>,
    }

    impl<D: OrchardDomainCommon> ActionArb<D> {
        fn encrypt_note<R: RngCore>(
            note: Note,
            memo: Vec<u8>,
            cmx: &ExtractedNoteCommitment,
            cv_net: &ValueCommitment,
            rng: &mut R,
        ) -> TransmittedNoteCiphertext<D> {
            let encryptor =
                NoteEncryption::<OrchardDomain<D>>::new(None, note, memo.try_into().unwrap());

            TransmittedNoteCiphertext {
                epk_bytes: encryptor.epk().to_bytes().0,
                enc_ciphertext: encryptor.encrypt_note_plaintext(),
                out_ciphertext: encryptor.encrypt_outgoing_plaintext(cv_net, cmx, rng),
            }
        }

        prop_compose! {
            /// Generate an action without authorization data.
            pub fn arb_unauthorized_action(spend_value: NoteValue, output_value: NoteValue)(
                nf in arb_nullifier(),
                rk in arb_spendauth_verification_key(),
                note in arb_note(output_value),
                asset in arb_asset_base(),
                rng_seed in prop::array::uniform32(prop::num::u8::ANY),
                memo in prop::collection::vec(prop::num::u8::ANY, 512),
            ) -> Action<(), D> {
                let cmx = ExtractedNoteCommitment::from(note.commitment());
                let cv_net = ValueCommitment::derive(
                    spend_value - output_value,
                    ValueCommitTrapdoor::zero(),
                    asset
                );

                let mut rng = StdRng::from_seed(rng_seed);
                let encrypted_note = Self::encrypt_note(note, memo, &cmx, &cv_net, &mut rng);

                Action {
                    nf,
                    rk,
                    cmx,
                    encrypted_note,
                    cv_net,
                    authorization: ()
                }
            }
        }

        prop_compose! {
            /// Generate an action with invalid (random) authorization data.
            pub fn arb_action(spend_value: NoteValue, output_value: NoteValue)(
                nf in arb_nullifier(),
                sk in arb_spendauth_signing_key(),
                note in arb_note(output_value),
                rng_seed in prop::array::uniform32(prop::num::u8::ANY),
                fake_sighash in prop::array::uniform32(prop::num::u8::ANY),
                asset in arb_asset_base(),
                memo in prop::collection::vec(prop::num::u8::ANY, 512),
            ) -> Action<redpallas::Signature<SpendAuth>, D> {
                let cmx = ExtractedNoteCommitment::from(note.commitment());
                let cv_net = ValueCommitment::derive(
                    spend_value - output_value,
                    ValueCommitTrapdoor::zero(),
                    asset
                );

                let mut rng = StdRng::from_seed(rng_seed);

                let encrypted_note = Self::encrypt_note(note, memo, &cmx, &cv_net, &mut rng);

                Action {
                    nf,
                    rk: redpallas::VerificationKey::from(&sk),
                    cmx,
                    encrypted_note,
                    cv_net,
                    authorization: sk.sign(rng, &fake_sighash),
                }
            }
        }
    }
}
