use alloc::vec::Vec;
use core::fmt;

use halo2_proofs::plonk;
use pasta_curves::arithmetic::CurveExt;
use pasta_curves::{pallas, vesta};
use rand::{CryptoRng, Rng};

use tracing::debug;

use super::{binding_validating_key, Authorized, Bundle};
use crate::{
    circuit::{Instance, VerifyingKey},
    primitives::redpallas::{self, Binding, SpendAuth},
};

#[cfg(feature = "multicore")]
const MIN_PARALLEL_BUNDLES: usize = 4;

/// Let an enclosing Rayon batch own parallelism rather than nesting preparation jobs.
#[cfg(feature = "multicore")]
fn parallel_preparation(bundles: usize) -> bool {
    bundles >= MIN_PARALLEL_BUNDLES && rayon::current_thread_index().is_none()
}

/// Owned verification inputs, without the bundle's ciphertexts or cached commitments.
#[derive(Debug)]
struct BundleInput {
    instances: Vec<Instance>,
    spend_signatures: Vec<redpallas::Signature<SpendAuth>>,
    binding_signature: redpallas::Signature<Binding>,
    value_balance: i64,
    sighash: [u8; 32],
    proof: Vec<u8>,
}

struct PreparedBundle {
    instances: Vec<Vec<Vec<vesta::Scalar>>>,
    signatures: Vec<redpallas::batch::Item<SpendAuth, Binding>>,
    proof: Vec<u8>,
}

/// Maps bundles in order, distributing sufficiently large batches across the pool.
fn map_bundles<T: Send, U: Send>(items: Vec<T>, f: impl Fn(T) -> U + Send + Sync) -> Vec<U> {
    #[cfg(feature = "multicore")]
    if parallel_preparation(items.len()) {
        use rayon::prelude::*;
        return items.into_par_iter().map(f).collect();
    }
    items.into_iter().map(f).collect()
}

/// Normalizes public transaction points, with one inversion per worker.
fn normalize(points: &[pallas::Point], bundles: usize) -> Vec<pallas::Affine> {
    let mut affine = vec![pallas::Affine::default(); points.len()];
    // The points are transaction cv_net/rk encodings or binding keys derived solely
    // from public cv_net commitments and valueBalance; no witness or trapdoor enters here.
    #[cfg(feature = "multicore")]
    if parallel_preparation(bundles) {
        use rayon::prelude::*;
        let chunk = points.len().div_ceil(rayon::current_num_threads()).max(1);
        points
            .par_chunks(chunk)
            .zip(affine.par_chunks_mut(chunk))
            .for_each(|(points, affine)| pallas::Point::batch_normalize_vartime(points, affine));
        return affine;
    }
    let _ = bundles;
    pallas::Point::batch_normalize_vartime(points, &mut affine);
    affine
}

fn prepare_bundles(bundles: Vec<BundleInput>) -> Vec<PreparedBundle> {
    let bvks = map_bundles(bundles.iter().collect(), |bundle| {
        binding_validating_key(
            bundle.instances.iter().map(Instance::cv_net),
            bundle.value_balance,
        )
        .point()
    });
    let point_count = bundles.iter().map(|b| 1 + 2 * b.instances.len()).sum();
    let mut points = Vec::with_capacity(point_count);
    let mut starts = Vec::with_capacity(bundles.len());
    for (bundle, bvk) in bundles.iter().zip(bvks) {
        starts.push(points.len());
        points.push(bvk);
        points.extend(
            bundle
                .instances
                .iter()
                .map(|instance| instance.cv_net().point()),
        );
        points.extend(
            bundle
                .instances
                .iter()
                .map(|instance| instance.rk().point()),
        );
    }
    let affine = normalize(&points, bundles.len());
    drop(points);

    map_bundles(
        bundles.into_iter().zip(starts).collect(),
        |(bundle, start)| {
            let BundleInput {
                instances,
                spend_signatures,
                binding_signature,
                sighash,
                proof,
                ..
            } = bundle;
            let mut signatures = Vec::with_capacity(instances.len() + 1);
            signatures.extend(
                instances
                    .iter()
                    .zip(spend_signatures)
                    .map(|(instance, signature)| {
                        instance.rk().create_batch_item(signature, &sighash)
                    }),
            );
            signatures.push(
                redpallas::VerificationKey::<Binding>::from_affine(affine[start])
                    .create_batch_item(binding_signature, &sighash),
            );
            let n = instances.len();
            let instances = Instance::to_halo2_instances_with_affine(
                &instances,
                &affine[start + 1..start + 1 + n],
                &affine[start + 1 + n..start + 1 + 2 * n],
            )
            .into_iter()
            .map(|columns| columns.into_iter().map(|column| column.to_vec()).collect())
            .collect();
            PreparedBundle {
                instances,
                signatures,
                proof,
            }
        },
    )
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
    bundles: Vec<BundleInput>,
    /// The verifying key every queued bundle is validated against, and which
    /// [`Self::validate`] finalizes the proof batch with.
    vk: &'a VerifyingKey,
}

impl<'a> BatchValidator<'a> {
    /// Constructs a new batch validation context that validates against `vk`.
    pub fn new(vk: &'a VerifyingKey) -> Self {
        BatchValidator {
            bundles: vec![],
            vk,
        }
    }

    /// Adds the proof and RedPallas signatures from the given bundle to the validator.
    ///
    /// Returns [`BatchError::RestrictionUnsupportedByKey`] if the bundle disables cross-address
    /// transfers but the validator's verifying key's circuit version does not support the
    /// cross-address restriction; in that case the bundle is not added to the batch.
    ///
    /// Copies only the verification inputs. Binding-key derivation, point normalization,
    /// and signature/instance preparation are deferred until [`Self::validate`], where
    /// sufficiently large batches share that work across the multicore pool.
    pub fn add_bundle<V: Copy + Into<i64>>(
        &mut self,
        bundle: &Bundle<Authorized, V>,
        sighash: [u8; 32],
    ) -> Result<(), BatchError> {
        if !bundle.flags().cross_address_enabled() && !self.vk.supports_cross_address_restriction()
        {
            return Err(BatchError::RestrictionUnsupportedByKey);
        }

        self.bundles.push(BundleInput {
            instances: bundle.to_instances(),
            spend_signatures: bundle
                .actions()
                .iter()
                .map(|action| action.authorization().clone())
                .collect(),
            binding_signature: bundle.authorization().binding_signature().clone(),
            value_balance: (*bundle.value_balance()).into(),
            sighash,
            proof: bundle.authorization().proof().as_ref().to_vec(),
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
            return true;
        }

        let prepared = prepare_bundles(self.bundles);

        let mut validator = redpallas::batch::Verifier::new();
        let mut proofs = plonk::BatchVerifier::<vesta::Affine>::new();
        for bundle in prepared {
            for signature in bundle.signatures {
                validator.queue(signature);
            }
            proofs.add_proof(bundle.instances, bundle.proof);
        }

        match validator.verify(&mut rng) {
            // If signatures are valid, check the proofs.
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
