use alloc::vec::Vec;
use core::fmt;

use group::Curve;
use halo2_proofs::plonk;
use pasta_curves::{pallas, vesta};
use rand::{CryptoRng, Rng};

use tracing::debug;

use super::{Authorized, Bundle};
use crate::{
    circuit::{Instance, VerifyingKey},
    primitives::redpallas::{self, Binding, SpendAuth},
    value::{ValueCommitment, ValueSum},
    Proof,
};

/// Rayon scheduling is worthwhile once at least four bundles share preparation.
#[cfg(any(feature = "multicore", test))]
const MIN_PARALLEL_BUNDLES: usize = 4;

/// Verification inputs retained in bundle order until batch preparation.
#[derive(Debug)]
struct QueuedBundle {
    instances: Vec<Instance>,
    spend_signatures: Vec<redpallas::Signature<SpendAuth>>,
    binding_signature: redpallas::Signature<Binding>,
    value_balance: i64,
    sighash: [u8; 32],
    proof: Proof,
}

impl QueuedBundle {
    fn binding_point(&self) -> pallas::Point {
        // https://p.z.cash/TCR:bad-txns-orchard-binding-signature-invalid?partial
        (self
            .instances
            .iter()
            .map(Instance::cv_net)
            .sum::<ValueCommitment>()
            - ValueCommitment::derive_public(ValueSum::from_raw(self.value_balance)))
        .point()
    }
}

/// Distributes expensive per-bundle work, retaining the original bundle order.
fn map_bundles<T: Send, U: Send>(bundles: Vec<T>, f: impl Fn(T) -> U + Send + Sync) -> Vec<U> {
    #[cfg(feature = "multicore")]
    if bundles.len() >= MIN_PARALLEL_BUNDLES {
        use rayon::prelude::*;
        return bundles.into_par_iter().map(f).collect();
    }
    bundles.into_iter().map(f).collect()
}

/// Error returned by [`BatchValidator::add_bundle`] when a bundle's flags require a circuit
/// capability the validator's verifying key does not provide.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BatchError {
    /// The bundle disables cross-address transfers, but the validator's verifying key's
    /// circuit version does not constrain the cross-address restriction (see
    /// [`OrchardCircuitVersion::supports_cross_address_restriction`]).
    ///
    /// [`OrchardCircuitVersion::supports_cross_address_restriction`]: crate::circuit::OrchardCircuitVersion::supports_cross_address_restriction
    RestrictionUnsupportedByKey,
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BatchError::RestrictionUnsupportedByKey => write!(
                f,
                "bundle disables cross-address transfers, but the verifying key's circuit \
                 version does not constrain the cross-address restriction",
            ),
        }
    }
}

impl core::error::Error for BatchError {}

/// Batch validation context for Orchard.
///
/// This batch-validates proofs and RedPallas signatures. The verifying key is bound at
/// construction: every bundle added to the batch is validated against it, and
/// [`Self::add_bundle`] rejects up front any bundle whose flags the key's circuit version
/// cannot enforce.
#[derive(Debug)]
pub struct BatchValidator<'a> {
    bundles: Vec<QueuedBundle>,
    /// The verifying key every queued bundle is validated against, and which
    /// [`Self::validate`] finalizes the proof batch with.
    vk: &'a VerifyingKey,
}

impl<'a> BatchValidator<'a> {
    /// Constructs a new batch validation context that validates against `vk`.
    pub fn new(vk: &'a VerifyingKey) -> Self {
        BatchValidator {
            bundles: Vec::new(),
            vk,
        }
    }

    /// Adds the proof and RedPallas signatures from the given bundle to the validator.
    ///
    /// Returns [`BatchError::RestrictionUnsupportedByKey`] if the bundle disables cross-address
    /// transfers but the validator's verifying key's circuit version does not support the
    /// cross-address restriction; in that case the bundle is not added to the batch.
    pub fn add_bundle<V: Copy + Into<i64>>(
        &mut self,
        bundle: &Bundle<Authorized, V>,
        sighash: [u8; 32],
    ) -> Result<(), BatchError> {
        if !bundle.flags().cross_address_enabled() && !self.vk.supports_cross_address_restriction()
        {
            return Err(BatchError::RestrictionUnsupportedByKey);
        }

        self.bundles.push(QueuedBundle {
            instances: bundle.to_instances(),
            spend_signatures: bundle
                .actions()
                .iter()
                .map(|action| action.authorization().clone())
                .collect(),
            binding_signature: bundle.authorization().binding_signature().clone(),
            value_balance: (*bundle.value_balance()).into(),
            sighash,
            proof: bundle.authorization().proof().clone(),
        });
        Ok(())
    }

    /// Batch-validates the accumulated bundles.
    ///
    /// Returns `true` if every proof and signature in every bundle added to the batch
    /// validator is valid. Returns `false` if one or more proofs or signatures are
    /// invalid. No attempt is made to figure out which of the accumulated bundles might
    /// be invalid; if that information is desired, construct separate [`BatchValidator`]s
    /// for sub-batches of the bundles.
    ///
    /// The cross-address-restriction capability is enforced when bundles are added (see
    /// [`Self::add_bundle`]), so it is already guaranteed here.
    pub fn validate<R: Rng + CryptoRng>(self, mut rng: R) -> bool {
        // https://p.z.cash/TCR:bad-txns-orchard-binding-signature-invalid?partial
        if self.bundles.is_empty() {
            return true;
        }

        // Compute the binding points independently, then normalize all public points together.
        let binding_points = map_bundles(self.bundles.iter().collect(), |bundle: &QueuedBundle| {
            bundle.binding_point()
        });
        let mut points = Vec::with_capacity(
            self.bundles
                .iter()
                .map(|bundle| 1 + 2 * bundle.instances.len())
                .sum(),
        );
        let mut starts = Vec::with_capacity(self.bundles.len());
        for (bundle, binding_point) in self.bundles.iter().zip(binding_points) {
            starts.push(points.len());
            points.push(binding_point);
            for instance in &bundle.instances {
                points.push(instance.cv_net().point());
                points.push(instance.rk().point());
            }
        }
        let mut affine = vec![pallas::Affine::default(); points.len()];
        pallas::Point::batch_normalize(&points, &mut affine);

        let prepared = map_bundles(
            self.bundles.into_iter().zip(starts).collect(),
            |(bundle, start): (QueuedBundle, usize)| {
                let mut signatures = Vec::with_capacity(bundle.instances.len() + 1);
                let binding_key = redpallas::VerificationKey::<Binding>::from_affine(affine[start]);
                let mut instances = Vec::with_capacity(bundle.instances.len());
                for (index, (instance, signature)) in bundle
                    .instances
                    .into_iter()
                    .zip(bundle.spend_signatures)
                    .enumerate()
                {
                    let cv = affine[start + 1 + 2 * index];
                    let rk = affine[start + 2 + 2 * index];
                    signatures.push(instance.rk().create_batch_item(signature, &bundle.sighash));
                    instances.push(
                        instance
                            .to_halo2_instance_with_points(cv, rk)
                            .into_iter()
                            .map(|column| column.to_vec())
                            .collect(),
                    );
                }
                signatures
                    .push(binding_key.create_batch_item(bundle.binding_signature, &bundle.sighash));
                (signatures, instances, bundle.proof)
            },
        );

        let mut validator = redpallas::batch::Verifier::new();
        let mut proofs = plonk::BatchVerifier::<vesta::Affine>::new();
        for (signatures, instances, proof) in prepared {
            for signature in signatures {
                validator.queue(signature);
            }
            proofs.add_proof(instances, proof.as_ref().to_vec());
        }

        match validator.verify(&mut rng) {
            Ok(()) => proofs.finalize(&self.vk.params, &self.vk.vk),
            Err(e) => {
                debug!("RedPallas batch validation failed: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod preparation_tests;

#[cfg(test)]
mod tests {
    use rand::{rand_core::UnwrapErr, rngs::SysRng};

    use super::{BatchError, BatchValidator};
    use crate::{
        bundle::tests::{sample_authorized_bundle, with_cross_address_disabled},
        circuit::OrchardCircuitVersion,
    };

    #[test]
    fn add_bundle_rejects_unsupported_cross_address_restriction() {
        let bundle = with_cross_address_disabled(sample_authorized_bundle(1))
            .try_map_value_balance(i64::try_from)
            .expect("generated bundle value balance fits in i64");

        // Keys whose circuit version cannot constrain the cross-address restriction reject
        // the bundle at insertion, so it never enters the batch.
        for circuit_version in [
            OrchardCircuitVersion::InsecurePreNu6_2,
            OrchardCircuitVersion::FixedPostNu6_2,
        ] {
            let vk = crate::cached_test_keys(circuit_version).verifying_key();
            let mut validator = BatchValidator::new(vk);
            assert_eq!(
                validator.add_bundle(&bundle, [0; 32]),
                Err(BatchError::RestrictionUnsupportedByKey)
            );
        }

        // The post-NU 6.3 key supports the restriction, so the bundle is accepted.
        let vk = crate::cached_test_keys(OrchardCircuitVersion::PostNu6_3).verifying_key();
        let mut validator = BatchValidator::new(vk);
        assert_eq!(validator.add_bundle(&bundle, [0; 32]), Ok(()));
    }

    #[test]
    fn empty_batch_validates() {
        for circuit_version in [
            OrchardCircuitVersion::InsecurePreNu6_2,
            OrchardCircuitVersion::FixedPostNu6_2,
            OrchardCircuitVersion::PostNu6_3,
        ] {
            let vk = crate::cached_test_keys(circuit_version).verifying_key();
            assert!(BatchValidator::new(vk).validate(UnwrapErr(SysRng)));
        }
    }
}
