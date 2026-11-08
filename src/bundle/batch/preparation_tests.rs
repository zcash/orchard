use alloc::vec::Vec;

use halo2_proofs::plonk;
use rand::{rngs::StdRng, SeedableRng};

use super::{prepare_bundles, BatchValidator};
use crate::{
    builder::{Builder, BundleType},
    bundle::{tests::sample_authorized_bundle, Authorized, Bundle, BundleVersion},
    circuit::{OrchardCircuitVersion, VerifyingKey},
    primitives::redpallas,
    Anchor, Proof,
};

/// The previous per-bundle path, including its independent point normalizations.
fn validate_per_bundle(bundles: &[(Bundle<Authorized, i64>, [u8; 32])], vk: &VerifyingKey) -> bool {
    if bundles.is_empty() {
        return true;
    }
    let mut signatures = redpallas::batch::Verifier::new();
    let mut proofs = plonk::BatchVerifier::new();
    for (bundle, sighash) in bundles {
        for action in bundle.actions() {
            signatures.queue(
                action
                    .rk()
                    .create_batch_item(action.authorization().clone(), sighash),
            );
        }
        let binding_key = (bundle
            .actions()
            .iter()
            .map(|action| action.cv_net())
            .sum::<crate::value::ValueCommitment>()
            - crate::value::ValueCommitment::derive(
                crate::value::ValueSum::from_raw(*bundle.value_balance()),
                crate::value::ValueCommitTrapdoor::zero(),
            ))
        .into_bvk();
        signatures.queue(
            binding_key
                .create_batch_item(bundle.authorization().binding_signature().clone(), sighash),
        );
        let instances = bundle
            .to_instances()
            .iter()
            .map(|instance| {
                instance
                    .to_halo2_instance()
                    .into_iter()
                    .map(|column| column.to_vec())
                    .collect()
            })
            .collect();
        proofs.add_proof(instances, bundle.authorization().proof().as_ref().to_vec());
    }
    signatures.verify(StdRng::from_seed([91; 32])).is_ok() && proofs.finalize(&vk.params, &vk.vk)
}

fn validate_together(bundles: &[(Bundle<Authorized, i64>, [u8; 32])], vk: &VerifyingKey) -> bool {
    let mut validator = BatchValidator::new(vk);
    for (bundle, sighash) in bundles {
        validator.add_bundle(bundle, *sighash).unwrap();
    }
    validator.validate(StdRng::from_seed([91; 32]))
}

#[test]
fn shared_normalization_preserves_instance_and_bundle_order() {
    let vk = crate::cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2).verifying_key();
    for count in [1, 2, 4, 9] {
        let bundles = (0..count)
            .map(|index| {
                let bundle = sample_authorized_bundle(1 + index % 4)
                    .try_map_value_balance(i64::try_from)
                    .unwrap();
                let balance = [i64::MIN, 0, i64::MAX][index % 3];
                bundle
                    .try_map_value_balance(|_| Ok::<_, ()>(balance))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let expected = bundles
            .iter()
            .map(|bundle| {
                bundle
                    .to_instances()
                    .iter()
                    .map(|instance| {
                        instance
                            .to_halo2_instance()
                            .into_iter()
                            .map(|column| column.to_vec())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut validator = BatchValidator::new(vk);
        for bundle in &bundles {
            let old_binding_key = (bundle
                .actions()
                .iter()
                .map(|action| action.cv_net())
                .sum::<crate::value::ValueCommitment>()
                - crate::value::ValueCommitment::derive(
                    crate::value::ValueSum::from_raw(*bundle.value_balance()),
                    crate::value::ValueCommitTrapdoor::zero(),
                ))
            .into_bvk();
            assert_eq!(bundle.binding_validating_key(), old_binding_key);
            validator.add_bundle(bundle, [0; 32]).unwrap();
        }
        let actual = prepare_bundles(validator.bundles);
        assert_eq!(actual.len(), expected.len());
        for ((actual, expected), bundle) in actual.iter().zip(expected).zip(bundles) {
            assert_eq!(actual.instances, expected);
            assert_eq!(actual.signatures.len(), bundle.actions().len() + 1);
            assert_eq!(actual.proof, bundle.authorization().proof().as_ref());
        }
    }
}

#[test]
fn shared_normalization_handles_identity_value_commitments() {
    use crate::{
        circuit::Instance,
        value::{ValueCommitTrapdoor, ValueCommitment, ValueSum},
    };
    let vk = crate::cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2).verifying_key();
    let bundle = sample_authorized_bundle(1)
        .try_map_value_balance(i64::try_from)
        .unwrap();
    let instance = bundle.to_instances().remove(0);
    let identity = Instance::from_parts(
        *instance.anchor(),
        ValueCommitment::derive(ValueSum::from_raw(0), ValueCommitTrapdoor::zero()),
        *instance.nf_old(),
        instance.rk().clone(),
        *instance.cmx(),
        *bundle.flags(),
    )
    .unwrap();
    let expected = identity.to_halo2_instance();
    let mut validator = BatchValidator::new(vk);
    validator.add_bundle(&bundle, [0; 32]).unwrap();
    validator.bundles[0].instances[0] = identity;
    validator.bundles[0].value_balance = 0;
    let prepared = prepare_bundles(validator.bundles);
    assert_eq!(prepared[0].instances[0][0], expected[0]);
}

#[test]
fn prepared_batches_match_per_bundle_verification() {
    let mut rng = StdRng::from_seed([92; 32]);
    for version in [BundleVersion::orchard_v2(), BundleVersion::orchard_v3()] {
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
            let bundle = builder.build::<i64>(&mut rng).unwrap().unwrap().0;
            bundle
                .create_proof(keys.proving_key(), &mut rng)
                .unwrap()
                .apply_signatures(&mut rng, [0; 32], &[])
                .unwrap()
        });
        for count in [0, 1, 2, 3, 4, 8, 17] {
            let bundles = (0..count)
                .map(|index| (fixtures[index % fixtures.len()].clone(), [0; 32]))
                .collect::<Vec<_>>();
            assert!(validate_per_bundle(&bundles, vk));
            assert!(validate_together(&bundles, vk));
            if count == 0 {
                continue;
            }
            // Exercise the first, middle and last positions as bundle counts change.
            let position = [0, count / 2, count - 1][count % 3];
            for mutation in 0..4 {
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
                    2 => bundle.authorization.binding_signature = [0xff; 64].into(),
                    3 => {
                        let mut proof = bundle.authorization().proof().as_ref().to_vec();
                        proof[0] ^= 1;
                        bundle.authorization.proof = Proof::new(proof);
                    }
                    _ => unreachable!("the test uses four mutations"),
                }
                let reference = validate_per_bundle(&invalid, vk);
                assert!(!reference, "mutation {mutation}, count {count}");
                assert_eq!(
                    validate_together(&invalid, vk),
                    reference,
                    "mutation {mutation}, count {count}"
                );
            }
        }
    }
}

#[test]
fn public_normalization_matches_constant_time_for_worker_boundaries() {
    use group::{Curve, Group};
    use pasta_curves::pallas;
    let mut rng = StdRng::from_seed([93; 32]);
    for n in [0, 1, 2, 3, 4, 7, 31, 32, 33, 64, 65, 207, 1035] {
        let points: Vec<_> = (0..n)
            .map(|index| match index % 5 {
                0 => pallas::Point::identity(),
                1 => pallas::Point::generator(),
                2 => -pallas::Point::generator(),
                _ => pallas::Point::random(&mut rng),
            })
            .collect();
        let mut expected = vec![pallas::Affine::default(); points.len()];
        pallas::Point::batch_normalize(&points, &mut expected);
        let check = || {
            for bundles in [0, 1, 3, 4, 9, 207] {
                assert_eq!(super::normalize(&points, bundles), expected);
            }
        };
        check();
        #[cfg(feature = "multicore")]
        for threads in [1, 2, 4, 8] {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap()
                .install(check);
        }
    }
}

#[cfg(feature = "multicore")]
#[test]
fn preparation_uses_only_one_level_of_parallelism() {
    for count in [0, 1, 3, 4, 64, 207] {
        assert_eq!(super::parallel_preparation(count), count >= 4);
    }
    for threads in [1, 2, 4] {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                for count in [0, 1, 3, 4, 64, 207] {
                    assert!(!super::parallel_preparation(count));
                }
            });
    }
}
