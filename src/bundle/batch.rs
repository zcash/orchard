use alloc::vec::Vec;

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
    // TODO(ebfull): Once a circuit version supports `disableCrossAddress`, store whether
    // any queued instance sets the flag and compare that with the verifying key's
    // circuit-version support in `validate`.
    unsupported_flags: bool,
}

impl BatchValidator {
    /// Constructs a new batch validation context.
    pub fn new() -> Self {
        BatchValidator {
            proofs: plonk::BatchVerifier::new(),
            signatures: vec![],
            unsupported_flags: false,
        }
    }

    /// Adds the proof and RedPallas signatures from the given bundle to the validator.
    ///
    /// If the bundle sets the `disableCrossAddress` flag, which no circuit version in
    /// this crate supports, [`Self::validate`] will return `false` for the entire batch.
    pub fn add_bundle<V: Copy + Into<i64>>(
        &mut self,
        bundle: &Bundle<Authorized, V>,
        sighash: [u8; 32],
    ) {
        self.unsupported_flags |= bundle.flags().disable_cross_address();

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
            .add_to_batch(&mut self.proofs, bundle.to_instances());
    }

    /// Batch-validates the accumulated bundles.
    ///
    /// Returns `true` if every proof and signature in every bundle added to the batch
    /// validator is valid, or `false` if one or more are invalid, or if any added bundle
    /// set a flag that no circuit version in this crate supports (such as
    /// `disableCrossAddress`). No attempt is made to figure out which of the accumulated
    /// bundles might be invalid; if that information is desired, construct separate
    /// [`BatchValidator`]s for sub-batches of the bundles.
    pub fn validate<R: RngCore + CryptoRng>(self, vk: &VerifyingKey, rng: R) -> bool {
        // https://p.z.cash/TCR:bad-txns-orchard-binding-signature-invalid?partial

        if self.unsupported_flags {
            return false;
        }

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
    use rand::rngs::OsRng;

    use super::BatchValidator;
    use crate::{
        bundle::tests::{sample_authorized_bundle, with_disable_cross_address},
        circuit::VerifyingKey,
    };

    #[test]
    fn add_bundle_records_unsupported_flags() {
        let bundle = with_disable_cross_address(sample_authorized_bundle(1))
            .try_map_value_balance(i64::try_from)
            .expect("generated bundle value balance fits in i64");

        let mut validator = BatchValidator::new();
        assert!(!validator.unsupported_flags);

        validator.add_bundle(&bundle, [0; 32]);
        assert!(validator.unsupported_flags);
    }

    // A bundle with fake authorizing data fails `validate` whether or not it sets
    // unsupported flags, so instead check the short-circuit against an otherwise-empty
    // batch, which is trivially valid.
    #[test]
    fn validate_rejects_unsupported_flags() {
        let vk = VerifyingKey::build();

        assert!(BatchValidator::new().validate(&vk, OsRng));

        let mut validator = BatchValidator::new();
        validator.unsupported_flags = true;
        assert!(!validator.validate(&vk, OsRng));
    }
}
