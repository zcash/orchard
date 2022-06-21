use rand::{CryptoRng, RngCore};
use tracing::debug;

use super::{Authorized, Bundle};
use crate::primitives::redpallas::{self, Binding, SpendAuth};

/// A signature within an authorized Orchard bundle.
#[derive(Debug)]
struct BundleSignature {
    /// The signature item for validation.
    signature: redpallas::batch::Item<SpendAuth, Binding>,
}

/// Batch validation context for Orchard.
///
/// This batch-validates RedPallas signatures.
#[derive(Debug, Default)]
pub struct BatchValidator {
    signatures: Vec<BundleSignature>,
}

impl BatchValidator {
    /// Constructs a new batch validation context.
    pub fn new() -> Self {
        BatchValidator { signatures: vec![] }
    }

    /// Adds the RedPallas signatures from the given bundle to the validator.
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
    }

    /// Batch-validates the accumulated bundles.
    ///
    /// Returns `true` if every signature in every bundle added to the batch validator is
    /// valid, or `false` if one or more are invalid. No attempt is made to figure out
    /// which of the accumulated bundles might be invalid; if that information is desired,
    /// construct separate [`BatchValidator`]s for sub-batches of the bundles.
    pub fn validate<R: RngCore + CryptoRng>(&self, rng: R) -> bool {
        if self.signatures.is_empty() {
            // An empty batch is always valid, but is not free to run; skip it.
            return true;
        }

        let mut validator = redpallas::batch::Verifier::new();
        for sig in self.signatures.iter() {
            validator.queue(sig.signature.clone());
        }

        match validator.verify(rng) {
            Ok(()) => true,
            Err(e) => {
                debug!("RedPallas batch validation failed: {}", e);
                false
            }
        }
    }
}
