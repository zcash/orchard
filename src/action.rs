use memuse::DynamicUsage;

use crate::{
    note::{ExtractedNoteCommitment, Nullifier, Rho, TransmittedNoteCiphertext},
    primitives::redpallas::{self, SpendAuth},
    value::ValueCommitment,
};

/// An action applied to the global ledger.
///
/// This both creates a note (adding a commitment to the global ledger), and consumes
/// some note created prior to this action (adding a nullifier to the global ledger).
///
/// # Invariants
///
/// Every `Action` has a non-identity `rk`.
#[derive(Debug, Clone)]
pub struct Action<A> {
    /// The nullifier of the note being spent.
    nf: Nullifier,
    /// The randomized verification key for the note being spent.
    rk: redpallas::VerificationKey<SpendAuth>,
    /// A commitment to the new note being created.
    cmx: ExtractedNoteCommitment,
    /// The transmitted note ciphertext.
    encrypted_note: TransmittedNoteCiphertext,
    /// A commitment to the net value created or consumed by this action.
    cv_net: ValueCommitment,
    /// The authorization for this action.
    authorization: A,
}

impl<T> Action<T> {
    /// Constructs an `Action` from its constituent parts.
    ///
    /// Returns `None` if `rk` is the identity [`pasta_curves::pallas::Point`].
    /// zcashd v6.12.1 and Zebra 4.3.1 both added a consensus rule rejecting
    /// transactions whose Orchard actions have an identity `rk`; the Zcash
    /// protocol specification will be updated to match, and this crate aligns
    /// with that rule.
    ///
    /// See:
    /// - <https://zodl.com/zcashd-zebra-april-2026-disclosure/>
    /// - <https://zfnd.org/zebra-4-3-1-critical-security-fixes-dockerized-mining-and-ci-hardening/>
    pub fn from_parts(
        nf: Nullifier,
        rk: redpallas::VerificationKey<SpendAuth>,
        cmx: ExtractedNoteCommitment,
        encrypted_note: TransmittedNoteCiphertext,
        cv_net: ValueCommitment,
        authorization: T,
    ) -> Option<Self> {
        (!rk.is_identity()).then_some(Action {
            nf,
            rk,
            cmx,
            encrypted_note,
            cv_net,
            authorization,
        })
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
    pub fn encrypted_note(&self) -> &TransmittedNoteCiphertext {
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
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Transitions this action from one authorization state to another.
    pub fn map<U>(self, step: impl FnOnce(T) -> U) -> Action<U> {
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
    pub fn try_map<U, E>(self, step: impl FnOnce(T) -> Result<U, E>) -> Result<Action<U>, E> {
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

impl DynamicUsage for Action<redpallas::Signature<SpendAuth>> {
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
            commitment::ExtractedNoteCommitment, nullifier::testing::arb_nullifier,
            testing::arb_note, TransmittedNoteCiphertext,
        },
        primitives::redpallas::{self, testing::arb_valid_spendauth_keypair},
        value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
    };

    use super::Action;

    prop_compose! {
        /// Generate an action without authorization data.
        pub fn arb_unauthorized_action(spend_value: NoteValue, output_value: NoteValue)(
            nf in arb_nullifier(),
            (_, rk) in arb_valid_spendauth_keypair(),
            note in arb_note(output_value),
        ) -> Action<()> {
            let cmx = ExtractedNoteCommitment::from(note.commitment());
            let cv_net = ValueCommitment::derive(
                spend_value - output_value,
                ValueCommitTrapdoor::zero()
            );
            // FIXME: make a real one from the note.
            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: [0u8; 32],
                enc_ciphertext: [0u8; 580],
                out_ciphertext: [0u8; 80]
            };
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
            (rsk, rk) in arb_valid_spendauth_keypair(),
            note in arb_note(output_value),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY),
            fake_sighash in prop::array::uniform32(prop::num::u8::ANY),
        ) -> Action<redpallas::Signature<SpendAuth>> {
            let cmx = ExtractedNoteCommitment::from(note.commitment());
            let cv_net = ValueCommitment::derive(
                spend_value - output_value,
                ValueCommitTrapdoor::zero()
            );

            // FIXME: make a real one from the note.
            let encrypted_note = TransmittedNoteCiphertext {
                epk_bytes: [0u8; 32],
                enc_ciphertext: [0u8; 580],
                out_ciphertext: [0u8; 80]
            };

            let rng = StdRng::from_seed(rng_seed);

            Action {
                nf,
                rk,
                cmx,
                encrypted_note,
                cv_net,
                authorization: rsk.sign(rng, &fake_sighash),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use group::ff::{Field as _, PrimeField as _};
    use pasta_curves::pallas;

    use super::Action;
    use crate::{
        note::{ExtractedNoteCommitment, Nullifier, TransmittedNoteCiphertext},
        primitives::redpallas::{self, SpendAuth},
        value::{ValueCommitTrapdoor, ValueCommitment, ValueSum},
    };

    /// The canonical Pallas encoding of the identity is [0u8; 32]; plain
    /// redpallas accepts it as a `VerificationKey<SpendAuth>`.
    fn identity_rk() -> redpallas::VerificationKey<SpendAuth> {
        redpallas::VerificationKey::<SpendAuth>::try_from([0u8; 32])
            .expect("plain redpallas accepts the identity encoding")
    }

    /// The verification key derived from a signing key with scalar 1 is the
    /// SpendAuthSig basepoint G, which is not the identity.
    fn non_identity_rk() -> redpallas::VerificationKey<SpendAuth> {
        let ask_bytes: [u8; 32] = pallas::Scalar::ONE.to_repr().into();
        let ask =
            redpallas::SigningKey::<SpendAuth>::try_from(ask_bytes).expect("1 is a valid scalar");
        (&ask).into()
    }

    /// Arbitrary non-zero values for the non-`rk` fields of an `Action`.
    /// Distinct non-zero patterns (rather than all-zeros) avoid accidental
    /// overlap with sentinel values.
    ///
    /// `cv_net` (a Pallas point encoding per the Orchard Spend statement)
    /// is checked at deserialization by `ValueCommitment::from_bytes`
    /// (e.g. `src/pczt/parse.rs`). `epk_bytes` is stored as raw bytes by
    /// this crate; its type in an Action description (§4.6, §5.4.5.5)
    /// is KA^{Orchard}.Public = ℙ*, but the consensus-level type check
    /// lives at the transaction deserializer in a consumer crate such
    /// as `zcash_primitives`. Neither check runs in `Action::from_parts`,
    /// so the `epk_bytes` here is just a byte pattern and need not decode
    /// to a curve point for this test.
    fn dummy_other_fields() -> (
        Nullifier,
        ExtractedNoteCommitment,
        TransmittedNoteCiphertext,
        ValueCommitment,
    ) {
        let nf = Nullifier::from_bytes(&[1u8; 32]).unwrap();
        let cmx = ExtractedNoteCommitment::from_bytes(&[2u8; 32]).unwrap();
        let encrypted_note = TransmittedNoteCiphertext {
            epk_bytes: [3u8; 32],
            enc_ciphertext: [4u8; 580],
            out_ciphertext: [5u8; 80],
        };
        let cv_net = ValueCommitment::derive(ValueSum::from_raw(42), ValueCommitTrapdoor::zero());
        (nf, cmx, encrypted_note, cv_net)
    }

    #[test]
    fn is_identity_detects_identity() {
        assert!(identity_rk().is_identity());
    }

    #[test]
    fn is_identity_rejects_non_identity() {
        assert!(!non_identity_rk().is_identity());
    }

    #[test]
    fn from_parts_rejects_identity_rk() {
        let (nf, cmx, encrypted_note, cv_net) = dummy_other_fields();
        let result = Action::from_parts(nf, identity_rk(), cmx, encrypted_note, cv_net, ());
        assert!(result.is_none());
    }

    #[test]
    fn from_parts_accepts_non_identity_rk() {
        let (nf, cmx, encrypted_note, cv_net) = dummy_other_fields();
        let rk = non_identity_rk();
        let action = Action::from_parts(nf, rk.clone(), cmx, encrypted_note, cv_net, ())
            .expect("non-identity rk must be accepted");
        assert_eq!(action.rk, rk);
    }
}
