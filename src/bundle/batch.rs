use alloc::vec::Vec;
use core::fmt;

use group::Curve;
use halo2_proofs::plonk;
use pasta_curves::pallas;
use rand::{CryptoRng, Rng};

use tracing::debug;

use super::{Authorized, Bundle};
use crate::{
    circuit::{Instance, VerifyingKey},
    primitives::redpallas::{self, Binding, SpendAuth},
    value::{ValueCommitTrapdoor, ValueCommitment, ValueSum},
    Proof,
};

/// Batches of fewer bundles than this are prepared on the calling thread, where
/// the work is too small to pay for distributing it across the pool.
#[cfg(feature = "multicore")]
const MIN_PARALLEL_BUNDLES: usize = 4;

/// An authorized bundle queued for batch validation.
///
/// [`BatchValidator::add_bundle`] records only what validation consumes. The
/// point arithmetic (binding validating keys, affine instance points) and the
/// signature challenges are deferred to [`BatchValidator::validate`], which runs
/// them across the thread pool and normalizes every bundle's points to affine
/// form with one shared inversion.
#[derive(Debug)]
struct QueuedBundle {
    /// One circuit instance per Action, in Action order.
    instances: Vec<Instance>,
    /// One spend authorization signature per Action, in Action order.
    spend_auth_signatures: Vec<redpallas::Signature<SpendAuth>>,
    binding_signature: redpallas::Signature<Binding>,
    value_balance: i64,
    sighash: [u8; 32],
    proof: Proof,
}

impl QueuedBundle {
    /// The binding validating key's point; equal to the point of
    /// [`Bundle::binding_validating_key`].
    fn binding_validating_key_point(&self) -> pallas::Point {
        // https://p.z.cash/TCR:bad-txns-orchard-binding-signature-invalid?partial
        (self
            .instances
            .iter()
            .map(Instance::cv_net)
            .sum::<ValueCommitment>()
            - ValueCommitment::derive(
                ValueSum::from_raw(self.value_balance),
                ValueCommitTrapdoor::zero(),
            ))
        .point()
    }

    /// The number of points this bundle contributes to the shared normalization:
    /// its binding validating key and, for Halo 2 instances, each Action's `cv_net`
    /// and `rk`.
    fn normalized_points(&self) -> usize {
        1 + 2 * self.instances.len()
    }
}

/// Maps `f` over `items`, across the thread pool when the batch is large enough.
fn map_bundles<T: Send, U: Send>(items: Vec<T>, f: impl Fn(T) -> U + Send + Sync) -> Vec<U> {
    #[cfg(feature = "multicore")]
    if items.len() >= MIN_PARALLEL_BUNDLES {
        use rayon::prelude::*;
        return items.into_par_iter().map(f).collect();
    }
    items.into_iter().map(f).collect()
}

/// Normalizes `points` to affine form, splitting them into one contiguous chunk
/// (and one inversion) per worker.
fn normalize(points: &[pallas::Point], bundles: usize) -> Vec<pallas::Affine> {
    let mut affine = alloc::vec![pallas::Affine::default(); points.len()];
    #[cfg(feature = "multicore")]
    if bundles >= MIN_PARALLEL_BUNDLES {
        use rayon::prelude::*;
        let chunk = points.len().div_ceil(rayon::current_num_threads()).max(1);
        points
            .par_chunks(chunk)
            .zip(affine.par_chunks_mut(chunk))
            .for_each(|(points, affine)| pallas::Point::batch_normalize(points, affine));
        return affine;
    }
    let _ = bundles;
    pallas::Point::batch_normalize(points, &mut affine);
    affine
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
            spend_auth_signatures: bundle
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
            // An empty batch is always valid, but is not free to run; skip it.
            // Note that a transaction has at least a binding signature, so if
            // there are no signatures, there are also no proofs.
            return true;
        }
        let bundles = self.bundles;
        let count = bundles.len();

        // Every bundle's binding validating key, then one shared normalization of
        // those keys and (for Halo 2) each Action's `cv_net` and `rk`. Each bundle
        // owns a contiguous run: its key, then its `cv_net`s, then its `rk`s.
        let bvks = map_bundles(
            bundles.iter().collect(),
            QueuedBundle::binding_validating_key_point,
        );
        let mut points =
            Vec::with_capacity(bundles.iter().map(QueuedBundle::normalized_points).sum());
        let mut starts = Vec::with_capacity(count);
        for (bundle, bvk) in bundles.iter().zip(bvks) {
            starts.push(points.len());
            points.push(bvk);
            points.extend(bundle.instances.iter().map(|i| i.cv_net().point()));
            points.extend(bundle.instances.iter().map(|i| i.rk().point()));
        }
        let affine = normalize(&points, count);
        drop(points);

        let prepared = map_bundles(
            bundles.into_iter().zip(starts).collect(),
            |(bundle, start)| {
                let QueuedBundle {
                    instances,
                    spend_auth_signatures,
                    binding_signature,
                    sighash,
                    proof,
                    ..
                } = bundle;
                let mut signatures = Vec::with_capacity(instances.len() + 1);
                signatures.extend(
                    instances
                        .iter()
                        .zip(spend_auth_signatures)
                        .map(|(instance, sig)| instance.rk().create_batch_item(sig, &sighash)),
                );
                signatures.push(
                    redpallas::VerificationKey::<Binding>::from_affine(affine[start])
                        .create_batch_item(binding_signature, &sighash),
                );

                let proof = {
                    let n = instances.len();
                    let cv_nets = &affine[start + 1..start + 1 + n];
                    let rks = &affine[start + 1 + n..start + 1 + 2 * n];
                    proof.into_batch_input(Instance::to_halo2_instances_with_affine(
                        &instances, cv_nets, rks,
                    ))
                };
                (signatures, proof)
            },
        );

        let mut signatures = Vec::with_capacity(prepared.iter().map(|(s, _)| s.len()).sum());
        let mut proofs = Vec::with_capacity(count);
        for (bundle_signatures, proof) in prepared {
            signatures.extend(bundle_signatures);
            proofs.push(proof);
        }

        let mut validator = redpallas::batch::Verifier::new();
        for signature in signatures {
            validator.queue(signature);
        }
        if let Err(e) = validator.verify(&mut rng) {
            debug!("RedPallas batch validation failed: {}", e);
            return false;
        }
        let mut batch = plonk::BatchVerifier::new();
        for (instances, proof) in proofs {
            batch.add_proof(instances, proof);
        }
        batch.finalize(&self.vk.params, &self.vk.vk)
    }
}

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
