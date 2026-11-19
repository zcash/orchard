use alloc::vec::Vec;

use halo2_proofs::plonk;
use rand::{rngs::StdRng, SeedableRng};

use super::{BatchError, BatchValidator, MIN_PARALLEL_BUNDLES};
use crate::{
    builder::{Builder, BundleType},
    bundle::{Authorized, Bundle, BundleVersion},
    circuit::{OrchardCircuitVersion, VerifyingKey},
    primitives::redpallas,
    Anchor, Proof,
};

// The original per-bundle add_bundle and validate paths, retained only as a test oracle.
fn old_path(
    bundles: &[(Bundle<Authorized, i64>, [u8; 32])],
    vk: &VerifyingKey,
) -> (Vec<Result<(), BatchError>>, bool) {
    let mut results = Vec::with_capacity(bundles.len());
    let mut signatures = redpallas::batch::Verifier::new();
    let mut proofs = plonk::BatchVerifier::new();
    let mut queued = false;
    for (bundle, sighash) in bundles {
        if !bundle.flags().cross_address_enabled() && !vk.supports_cross_address_restriction() {
            results.push(Err(BatchError::RestrictionUnsupportedByKey));
            continue;
        }
        results.push(Ok(()));
        queued = true;
        for action in bundle.actions() {
            signatures.queue(
                action
                    .rk()
                    .create_batch_item(action.authorization().clone(), sighash),
            );
        }
        signatures.queue(
            bundle
                .binding_validating_key()
                .create_batch_item(bundle.authorization().binding_signature().clone(), sighash),
        );
        bundle
            .authorization()
            .proof()
            .add_to_batch(&mut proofs, bundle.to_instances());
    }
    let valid = !queued
        || (signatures.verify(StdRng::from_seed([91; 32])).is_ok()
            && proofs.finalize(&vk.params, &vk.vk));
    (results, valid)
}

fn new_path(
    bundles: &[(Bundle<Authorized, i64>, [u8; 32])],
    vk: &VerifyingKey,
) -> (Vec<Result<(), BatchError>>, bool) {
    let mut validator = BatchValidator::new(vk);
    let results = bundles
        .iter()
        .map(|(bundle, sighash)| validator.add_bundle(bundle, *sighash))
        .collect();
    (results, validator.validate(StdRng::from_seed([91; 32])))
}

#[test]
fn batch_preparation_matches_original_for_valid_and_tampered_bundles() {
    let mut rng = StdRng::from_seed([92; 32]);
    for version in [
        BundleVersion::orchard_v2(),
        BundleVersion::orchard_v3(),
        BundleVersion::ironwood_v3(),
    ] {
        let keys = crate::cached_test_keys(version.circuit_version());
        let vk = keys.verifying_key();
        let fixtures = [1, 3].map(|actions| {
            let builder = Builder::new(
                BundleType::Transactional {
                    bundle_required: true,
                    pad_to_minimum: Some(actions),
                },
                version,
                version.default_flags(),
                Anchor::from_bytes([0; 32]).unwrap(),
            )
            .unwrap();
            builder
                .build::<i64>(&mut rng)
                .unwrap()
                .unwrap()
                .0
                .create_proof(keys.proving_key(), &mut rng)
                .unwrap()
                .apply_signatures(&mut rng, [0; 32], &[])
                .unwrap()
        });
        for count in [0, 1, 2, MIN_PARALLEL_BUNDLES, 9] {
            let bundles = (0..count)
                .map(|i| (fixtures[i % 2].clone(), [0; 32]))
                .collect::<Vec<_>>();
            let original = old_path(&bundles, vk);
            assert!(original.1, "valid {version:?} bundle batch");
            assert_eq!(original, new_path(&bundles, vk));
            if count == 0 {
                continue;
            }
            let position = count / 2;
            for mutation in 0..5 {
                let mut invalid = bundles.clone();
                let (bundle, sighash) = &mut invalid[position];
                match mutation {
                    0 => sighash[0] ^= 1,
                    1 => {
                        *bundle = bundle.clone().map_authorization(
                            &mut (),
                            |_, _, _| [0xff; 64].into(),
                            |_, authorization| authorization,
                        );
                    }
                    2 => {
                        let mut signature: [u8; 64] =
                            bundle.authorization().binding_signature().into();
                        signature[32] ^= 1;
                        *bundle = bundle.clone().map_authorization(
                            &mut (),
                            |_, _, spend| spend,
                            |_, authorization| {
                                Authorized::from_parts(
                                    authorization.proof().clone(),
                                    signature.into(),
                                )
                            },
                        );
                    }
                    3 => {
                        let balance = *bundle.value_balance();
                        *bundle = bundle
                            .clone()
                            .try_map_value_balance(|_| Ok::<_, ()>(balance + 1))
                            .unwrap();
                    }
                    4 => {
                        let mut bytes = bundle.authorization().proof().as_ref().to_vec();
                        bytes[0] ^= 1;
                        *bundle = bundle.clone().map_authorization(
                            &mut (),
                            |_, _, spend| spend,
                            |_, authorization| {
                                Authorized::from_parts(
                                    Proof::new(bytes),
                                    authorization.binding_signature().clone(),
                                )
                            },
                        );
                    }
                    _ => unreachable!(),
                }
                let original = old_path(&invalid, vk);
                assert!(!original.1, "mutation {mutation}, count {count}");
                assert_eq!(
                    new_path(&invalid, vk),
                    original,
                    "mutation {mutation}, count {count}"
                );
            }
        }
    }
}

#[test]
fn unsupported_restriction_keeps_early_error_and_skips_bundle() {
    use crate::bundle::tests::{sample_authorized_bundle, with_cross_address_disabled};
    let bundle = with_cross_address_disabled(sample_authorized_bundle(1))
        .try_map_value_balance(i64::try_from)
        .unwrap();
    for version in [
        OrchardCircuitVersion::InsecurePreNu6_2,
        OrchardCircuitVersion::FixedPostNu6_2,
    ] {
        let vk = crate::cached_test_keys(version).verifying_key();
        let input = [(bundle.clone(), [0; 32])];
        assert_eq!(new_path(&input, vk), old_path(&input, vk));
        assert_eq!(
            new_path(&input, vk),
            (vec![Err(BatchError::RestrictionUnsupportedByKey)], true)
        );
    }
}
