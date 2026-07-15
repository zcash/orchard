//! Feature-gated Orchard fixtures for Halo2 verifier fingerprints.

use alloc::vec::Vec;

use halo2_proofs::plonk::fingerprint::{capture_proof_fingerprint, ChallengeRecorder};
use halo2_proofs::transcript::Challenge255;
use incrementalmerkletree::Hashable;
use pasta_curves::vesta;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use super::{OrchardCircuitVersion, ProvingKey, VerifyingKey, K};
use crate::{
    builder::{Builder, BundleType},
    bundle::BundleVersion,
    constants::MERKLE_DEPTH_ORCHARD,
    tree::MerkleHashOrchard,
};

mod rejected;

fn fixture_rng(seed: u8) -> ChaCha20Rng {
    ChaCha20Rng::from_seed([seed; 32])
}

fn build_fixture_bundle(
    rng: &mut ChaCha20Rng,
    pk: &ProvingKey,
    num_actions: u8,
) -> crate::Bundle<crate::bundle::Authorized, i64> {
    let bundle_version = BundleVersion::orchard_v3();
    let builder = Builder::new(
        BundleType::Transactional {
            bundle_required: true,
            pad_to_minimum: Some(num_actions),
        },
        bundle_version,
        bundle_version.default_flags(),
        MerkleHashOrchard::empty_root((MERKLE_DEPTH_ORCHARD as u8).into()).into(),
    )
    .unwrap();
    let bundle = builder.build::<i64>(&mut *rng).unwrap().unwrap().0;
    assert_eq!(bundle.actions().len(), usize::from(num_actions));
    assert!(!bundle.flags().cross_address_enabled());

    bundle
        .create_proof(pk, &mut *rng)
        .unwrap()
        .apply_signatures(&mut *rng, [0; 32], &[])
        .unwrap()
}

fn raw_instances(instances: &[super::Instance]) -> Vec<Vec<Vec<vesta::Scalar>>> {
    instances
        .iter()
        .map(|instance| {
            instance
                .to_halo2_instance()
                .iter()
                .map(|column| column.to_vec())
                .collect()
        })
        .collect()
}

fn raw_instance_refs(raw_instances: &[Vec<Vec<vesta::Scalar>>]) -> Vec<Vec<&[vesta::Scalar]>> {
    raw_instances
        .iter()
        .map(|instance| instance.iter().map(|column| &column[..]).collect())
        .collect()
}

fn assert_pinned_verifying_key(vk: &VerifyingKey) {
    assert_eq!(
        format!("{:#?}\n", vk.vk.pinned()),
        include_str!("../../circuit_data/circuit_description_post_nu6_3").replace("\r\n", "\n")
    );
}

fn capture_fixture(seed: u8, num_actions: u8, namespace: &str, output_var: &str) {
    let mut rng = fixture_rng(seed);
    let pk = ProvingKey::build(OrchardCircuitVersion::PostNu6_3);
    let vk = VerifyingKey::build(OrchardCircuitVersion::PostNu6_3);
    assert!(vk.supports_cross_address_restriction());
    assert_pinned_verifying_key(&vk);

    let bundle = build_fixture_bundle(&mut rng, &pk, num_actions);
    let instances = bundle.to_instances();
    let proof = bundle.authorization().proof().clone();
    assert!(bundle.verify_proof(&vk).is_ok());

    let raw_instances = raw_instances(&instances);
    let raw_instance_refs = raw_instance_refs(&raw_instances);
    let raw_instance_refs: Vec<_> = raw_instance_refs
        .iter()
        .map(|instance| &instance[..])
        .collect();

    let mut transcript = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof.0[..]);
    let msm =
        capture_proof_fingerprint(&vk.params, &vk.vk, &raw_instance_refs, &mut transcript).unwrap();

    assert!(!transcript.challenges.is_empty());
    assert!(
        msm.clone().eval(),
        "captured Orchard fingerprint must be the identity for a valid proof"
    );
    std::eprintln!(
        "Captured {num_actions}-action Orchard verifier fingerprint at k={} with {} challenges",
        K,
        transcript.challenges.len(),
    );

    let fixture = vk.vk.dump_vesta_lean_fixture(
        namespace,
        "PostNu6_3",
        K,
        usize::from(num_actions),
        &transcript.common_points,
        &transcript.points,
        &transcript.scalars,
        &transcript.challenges,
        &transcript.events,
        &msm,
    );
    if let Some(path) = std::env::var_os(output_var) {
        std::fs::write(std::path::PathBuf::from(path), fixture).unwrap();
    }
}

/// Regenerate the deterministic checked-in single-action Ironwood fixture with:
///
/// ```text
/// ORCHARD_LEAN_SINGLE_FIXTURE_OUT=/path/to/ironwood/Zcash/Snark/Fixtures/SingleAction/Fixture.lean \
///   cargo test --features verifier-fingerprint circuit::fingerprint::fingerprint_capture -- --exact
/// ```
#[test]
fn fingerprint_capture() {
    capture_fixture(
        0x53,
        1,
        "Zcash.Snark.Fixture",
        "ORCHARD_LEAN_SINGLE_FIXTURE_OUT",
    );
}

/// Regenerate the deterministic checked-in two-action Ironwood fixture with:
///
/// ```text
/// ORCHARD_LEAN_MULTI_FIXTURE_OUT=/path/to/ironwood/Zcash/Snark/Fixtures/MultiAction/Fixture.lean \
///   cargo test --features verifier-fingerprint \
///     circuit::fingerprint::fingerprint_capture_two_actions -- --exact
/// ```
///
/// The real two-action bundle exercises the verifier's per-sub-proof read schedule, expression
/// folds, and query wiring that a single-action capture never reaches. See
/// https://github.com/zcash/ironwood/issues/17.
#[test]
fn fingerprint_capture_two_actions() {
    capture_fixture(
        0x4d,
        2,
        "Zcash.Snark.Fixture2",
        "ORCHARD_LEAN_MULTI_FIXTURE_OUT",
    );
}
