//! Rejected and adversarial verifier-fingerprint captures.

use alloc::vec::Vec;

use ff::Field;
use halo2_proofs::plonk::fingerprint::{
    capture_proof_fingerprint, ChallengeRecorder, TranscriptEvent,
};
use halo2_proofs::plonk::{verify_proof, SingleVerifier};
use halo2_proofs::transcript::{Blake2bWrite, Challenge255, TranscriptWrite};
use pasta_curves::vesta;

use super::{
    assert_pinned_verifying_key, build_fixture_bundle, fixture_rng, raw_instance_refs,
    raw_instances,
};
use crate::circuit::{OrchardCircuitVersion, ProvingKey, VerifyingKey};

enum ScalarEventEdit {
    Write(vesta::Scalar),
    Skip,
    Stop,
}

fn proof_from_read_events(
    events: &[TranscriptEvent<vesta::Affine>],
    mut scalar_mutation: impl FnMut(usize, vesta::Scalar) -> ScalarEventEdit,
) -> Vec<u8> {
    let mut transcript = Blake2bWrite::<_, vesta::Affine, Challenge255<_>>::init(vec![]);
    let mut scalar_idx = 0usize;
    for event in events {
        match event {
            TranscriptEvent::ReadPoint(point) => transcript.write_point(*point).unwrap(),
            TranscriptEvent::ReadScalar(scalar) => {
                match scalar_mutation(scalar_idx, *scalar) {
                    ScalarEventEdit::Write(scalar) => transcript.write_scalar(scalar).unwrap(),
                    ScalarEventEdit::Skip => {}
                    ScalarEventEdit::Stop => break,
                }
                scalar_idx += 1;
            }
            TranscriptEvent::CommonPoint(_)
            | TranscriptEvent::CommonScalar(_)
            | TranscriptEvent::Squeeze(_) => {}
        }
    }
    transcript.finalize()
}

/// Drives the deployed verifier on malformed versions of the real two-action fixture proof while
/// recording the transcript prefix consumed before rejection.
#[test]
fn fingerprint_rejected_capture_two_actions() {
    let mut rng = fixture_rng(0x4e);
    let pk = ProvingKey::build(OrchardCircuitVersion::PostNu6_3);
    let vk = VerifyingKey::build(OrchardCircuitVersion::PostNu6_3);
    assert!(vk.supports_cross_address_restriction());
    assert_pinned_verifying_key(&vk);

    let bundle = build_fixture_bundle(&mut rng, &pk, 2);
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
    let valid_msm =
        capture_proof_fingerprint(&vk.params, &vk.vk, &raw_instance_refs, &mut transcript).unwrap();
    assert!(valid_msm.eval());
    assert_eq!(
        proof_from_read_events(&transcript.events, |_, scalar| ScalarEventEdit::Write(
            scalar
        )),
        proof.0,
        "captured read events should reserialize the original proof exactly"
    );

    let n_instance_evals = instances.len();
    let first_advice_eval = n_instance_evals;
    let tampered_proof = proof_from_read_events(&transcript.events, |idx, scalar| {
        ScalarEventEdit::Write(if idx == first_advice_eval {
            scalar + vesta::Scalar::ONE
        } else {
            scalar
        })
    });

    let mut tampered_transcript =
        ChallengeRecorder::<_, _, Challenge255<_>>::init(&tampered_proof[..]);
    let tampered_msm = capture_proof_fingerprint(
        &vk.params,
        &vk.vk,
        &raw_instance_refs,
        &mut tampered_transcript,
    )
    .unwrap();
    assert!(
        !tampered_msm.eval(),
        "tampered advice-eval capture should assemble a non-identity fingerprint"
    );

    let strategy = SingleVerifier::new(&vk.params);
    let mut tampered_reject_transcript =
        ChallengeRecorder::<_, _, Challenge255<_>>::init(&tampered_proof[..]);
    assert!(matches!(
        verify_proof(
            &vk.params,
            &vk.vk,
            strategy,
            &raw_instance_refs,
            &mut tampered_reject_transcript,
        ),
        Err(halo2_proofs::plonk::Error::ConstraintSystemFailure)
    ));

    // These are the Orchard action-circuit shape counts emitted by the Lean fixture dumper
    // (`shape` in `Fixture2.lean`). Keeping them explicit avoids reaching into Halo2's private
    // verifying-key internals from this crate-local negative capture test.
    let n_advice_queries = 25;
    let n_fixed_evals = 29;
    let n_permutation_common_evals = 15;
    let n_permutation_sets = 3;
    let n_lookups = 3;
    let n_permutation_set_evals = instances.len() * (3 * n_permutation_sets - 1);
    let n_lookup_evals = instances.len() * n_lookups * 5;
    let first_multiopen_u = n_instance_evals
        + instances.len() * n_advice_queries
        + n_fixed_evals
        + 1
        + n_permutation_common_evals
        + n_permutation_set_evals
        + n_lookup_evals;
    let n_multiopen_u = transcript.scalars.len() - first_multiopen_u - 2;
    assert_eq!(n_multiopen_u, 5);

    let truncated_u_proof = proof_from_read_events(&transcript.events, |idx, scalar| {
        if idx + 1 == first_multiopen_u + n_multiopen_u {
            ScalarEventEdit::Stop
        } else {
            ScalarEventEdit::Write(scalar)
        }
    });
    let strategy = SingleVerifier::new(&vk.params);
    let mut truncated_u_transcript =
        ChallengeRecorder::<_, _, Challenge255<_>>::init(&truncated_u_proof[..]);
    assert!(matches!(
        verify_proof(
            &vk.params,
            &vk.vk,
            strategy,
            &raw_instance_refs,
            &mut truncated_u_transcript,
        ),
        Err(halo2_proofs::plonk::Error::Opening)
    ));
    assert!(
        truncated_u_transcript.challenges.len() >= 8,
        "malformed-u capture should reach the multiopen x3 challenge before rejection"
    );

    let first_permutation_set_eval = n_instance_evals
        + instances.len() * n_advice_queries
        + n_fixed_evals
        + 1
        + n_permutation_common_evals;
    let first_nonlast_permutation_last_eval = first_permutation_set_eval + 2;
    let missing_permutation_last_eval_proof =
        proof_from_read_events(&transcript.events, |idx, scalar| {
            if idx == first_nonlast_permutation_last_eval {
                ScalarEventEdit::Skip
            } else {
                ScalarEventEdit::Write(scalar)
            }
        });
    let strategy = SingleVerifier::new(&vk.params);
    let mut missing_permutation_last_eval_transcript =
        ChallengeRecorder::<_, _, Challenge255<_>>::init(&missing_permutation_last_eval_proof[..]);
    assert!(verify_proof(
        &vk.params,
        &vk.vk,
        strategy,
        &raw_instance_refs,
        &mut missing_permutation_last_eval_transcript,
    )
    .is_err());

    std::eprintln!(
        "Orchard negative captures: tampered advice eval events={} challenges={} scalars={} points={}; truncated-u events={} challenges={} scalars={} points={}; missing permutation last-eval events={} challenges={} scalars={} points={}",
        tampered_reject_transcript.events.len(),
        tampered_reject_transcript.challenges.len(),
        tampered_reject_transcript.scalars.len(),
        tampered_reject_transcript.points.len(),
        truncated_u_transcript.events.len(),
        truncated_u_transcript.challenges.len(),
        truncated_u_transcript.scalars.len(),
        truncated_u_transcript.points.len(),
        missing_permutation_last_eval_transcript.events.len(),
        missing_permutation_last_eval_transcript.challenges.len(),
        missing_permutation_last_eval_transcript.scalars.len(),
        missing_permutation_last_eval_transcript.points.len(),
    );
}
