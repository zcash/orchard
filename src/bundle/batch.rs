use alloc::vec::Vec;

use core::fmt;

use halo2_proofs::plonk;
use pasta_curves::vesta;
use rand::{CryptoRng, RngCore};
use tracing::debug;

use super::{Authorized, Bundle};
use crate::{
    circuit::VerifyingKey,
    primitives::redpallas::{self, Binding, SpendAuth},
};

/// A signature within an authorized Orchard bundle.
#[derive(Debug)]
struct BundleSignature {
    /// The signature item for validation.
    signature: redpallas::batch::Item<SpendAuth, Binding>,
}

/// Batch validation context for Orchard.
///
/// This batch-validates proofs and RedPallas signatures.
#[derive(Debug, Default)]
pub struct BatchValidator {
    proofs: plonk::BatchVerifier<vesta::Affine>,
    signatures: Vec<BundleSignature>,
}

/// Errors that can occur when adding a bundle to a [`BatchValidator`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BatchValidatorError {
    /// An action has an identity `rk`, which is forbidden by the consensus rule
    /// introduced in zcashd v6.12.1 and Zebra 4.3.1.
    IdentityRk,
}

impl fmt::Display for BatchValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchValidatorError::IdentityRk => {
                write!(f, "an Orchard action with identity `rk` is not valid")
            }
        }
    }
}

impl core::error::Error for BatchValidatorError {}

impl BatchValidator {
    /// Constructs a new batch validation context.
    pub fn new() -> Self {
        BatchValidator {
            proofs: plonk::BatchVerifier::new(),
            signatures: vec![],
        }
    }

    /// Adds the proof and RedPallas signatures from the given bundle to the validator.
    ///
    /// Returns [`BatchValidatorError::IdentityRk`] without modifying the validator if
    /// any action in the bundle has an identity `rk`.
    pub fn add_bundle<V: Copy + Into<i64>>(
        &mut self,
        bundle: &Bundle<Authorized, V>,
        sighash: [u8; 32],
    ) -> Result<(), BatchValidatorError> {
        let instances = bundle
            .try_to_instances()
            .ok_or(BatchValidatorError::IdentityRk)?;

        for action in bundle.actions().iter() {
            self.signatures.push(BundleSignature {
                signature: action
                    .rk()
                    .create_batch_item(action.authorization().clone(), &sighash),
            });
        }

        self.signatures.push(BundleSignature {
            signature: bundle
                .binding_validating_key()
                .create_batch_item(bundle.authorization().binding_signature().clone(), &sighash),
        });

        bundle
            .authorization()
            .proof()
            .add_to_batch(&mut self.proofs, instances);

        Ok(())
    }

    /// Batch-validates the accumulated bundles.
    ///
    /// Returns `true` if every proof and signature in every bundle added to the batch
    /// validator is valid, or `false` if one or more are invalid. No attempt is made to
    /// figure out which of the accumulated bundles might be invalid; if that information
    /// is desired, construct separate [`BatchValidator`]s for sub-batches of the bundles.
    pub fn validate<R: RngCore + CryptoRng>(self, vk: &VerifyingKey, rng: R) -> bool {
        // https://p.z.cash/TCR:bad-txns-orchard-binding-signature-invalid?partial

        if self.signatures.is_empty() {
            // An empty batch is always valid, but is not free to run; skip it.
            // Note that a transaction has at least a binding signature, so if
            // there are no signatures, there are also no proofs.
            return true;
        }

        let mut validator = redpallas::batch::Verifier::new();
        for sig in self.signatures.iter() {
            validator.queue(sig.signature.clone());
        }

        match validator.verify(rng) {
            // If signatures are valid, check the proofs.
            Ok(()) => self.proofs.finalize(&vk.params, &vk.vk),
            Err(e) => {
                debug!("RedPallas batch validation failed: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use nonempty::NonEmpty;
    use proptest::prelude::*;

    use super::{BatchValidator, BatchValidatorError};
    use crate::{
        action::testing::clone_with_rk_for_test,
        bundle::{Authorized, Bundle, ProofSizeEnforcement},
        primitives::redpallas::{self, SpendAuth},
    };

    fn identity_rk() -> redpallas::VerificationKey<SpendAuth> {
        redpallas::VerificationKey::<SpendAuth>::try_from([0u8; 32])
            .expect("plain redpallas accepts the identity encoding")
    }

    proptest! {
        // The property is deterministic given the bundle.
        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn add_bundle_accepts_valid_bundle(
            bundle in crate::bundle::testing::arb_bundle(2),
            sighash in prop::array::uniform32(prop::num::u8::ANY),
        ) {
            let bundle = bundle
                .try_map_value_balance(|_| Ok::<i64, ()>(0))
                .expect("mapping to i64 cannot fail");
            let mut validator = BatchValidator::new();
            prop_assert_eq!(validator.add_bundle(&bundle, sighash), Ok(()));
        }

        #[test]
        fn add_bundle_rejects_identity_rk(
            bundle in crate::bundle::testing::arb_bundle(1),
            sighash in prop::array::uniform32(prop::num::u8::ANY),
        ) {
            let bundle = bundle
                .try_map_value_balance(|_| Ok::<i64, ()>(0))
                .expect("mapping to i64 cannot fail");
            let action = clone_with_rk_for_test(bundle.actions().first(), identity_rk());
            let invalid_bundle = Bundle::try_from_parts(
                NonEmpty::from_vec(vec![action]).expect("one action"),
                *bundle.flags(),
                *bundle.value_balance(),
                *bundle.anchor(),
                Authorized::from_parts(
                    bundle.authorization().proof().clone(),
                    bundle.authorization().binding_signature().clone(),
                ),
                ProofSizeEnforcement::Strict,
            )
            .expect("proof size is unchanged");

            let mut validator = BatchValidator::new();
            prop_assert_eq!(
                validator.add_bundle(&invalid_bundle, sighash),
                Err(BatchValidatorError::IdentityRk)
            );
        }
    }
}
