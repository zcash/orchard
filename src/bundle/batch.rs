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
    /// Whether any queued instance disables cross-address transfers. Such statements can
    /// only be validated with a verifying key whose circuit version constrains the
    /// `disableCrossAddress` public input; the key is not known until [`Self::validate`].
    restricted: bool,
}

impl BatchValidator {
    /// Constructs a new batch validation context.
    pub fn new() -> Self {
        BatchValidator {
            proofs: plonk::BatchVerifier::new(),
            signatures: vec![],
            restricted: false,
        }
    }

    /// Adds the proof and RedPallas signatures from the given bundle to the validator.
    ///
    /// If the bundle sets `disableCrossAddress`, this records that [`Self::validate`]
    /// must be called with a verifying key whose circuit version supports the
    /// cross-address restriction.
    pub fn add_bundle<V: Copy + Into<i64>>(
        &mut self,
        bundle: &Bundle<Authorized, V>,
        sighash: [u8; 32],
    ) {
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

        let instances = bundle.to_instances();
        self.restricted |= instances.iter().any(|i| i.disable_cross_address());
        bundle
            .authorization()
            .proof()
            .add_to_batch(&mut self.proofs, instances);
    }

    /// Batch-validates the accumulated bundles.
    ///
    /// Returns `true` if every proof and signature in every bundle added to the batch
    /// validator is valid, and any `disableCrossAddress` instances are supported by
    /// `vk`'s circuit version. Returns `false` if one or more proofs or signatures are
    /// invalid, or if the batch contains a restricted instance and `vk` does not support
    /// it. No attempt is made to figure out which of the accumulated bundles might be
    /// invalid; if that information is desired, construct separate [`BatchValidator`]s
    /// for sub-batches of the bundles.
    pub fn validate<R: RngCore + CryptoRng>(self, vk: &VerifyingKey, rng: R) -> bool {
        // https://p.z.cash/TCR:bad-txns-orchard-binding-signature-invalid?partial

        if self.restricted && !vk.supports_cross_address_restriction() {
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
        circuit::{OrchardCircuitVersion, VerifyingKey},
    };

    #[test]
    fn add_bundle_records_restricted_instances() {
        let bundle = with_disable_cross_address(sample_authorized_bundle(1))
            .try_map_value_balance(i64::try_from)
            .expect("generated bundle value balance fits in i64");

        let mut validator = BatchValidator::new();
        assert!(!validator.restricted);

        validator.add_bundle(&bundle, [0; 32]);
        assert!(validator.restricted);
    }

    // A bundle with fake authorizing data fails `validate` whether or not it sets
    // `disableCrossAddress`, so instead check the key-capability short-circuit against an
    // otherwise-empty batch, which is trivially valid.
    #[test]
    fn validate_requires_key_support_for_disable_cross_address() {
        for circuit_version in [
            OrchardCircuitVersion::InsecurePreNu6_2,
            OrchardCircuitVersion::FixedPostNu6_2,
        ] {
            let vk = VerifyingKey::build(circuit_version);

            assert!(BatchValidator::new().validate(&vk, OsRng));

            let mut validator = BatchValidator::new();
            validator.restricted = true;
            assert!(!validator.validate(&vk, OsRng));
        }

        let vk = VerifyingKey::build(OrchardCircuitVersion::Ironwood);
        let mut validator = BatchValidator::new();
        validator.restricted = true;
        assert!(validator.validate(&vk, OsRng));
    }
}
