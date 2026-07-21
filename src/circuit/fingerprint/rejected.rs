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

    // Instance evaluations are read first. `instances.len()` is the number of instance-eval scalars
    // only because the pinned Post-NU6.3 key has exactly one instance query per proof
    // (`num_instance_columns == 1`, locked by `assert_pinned_verifying_key` above); a key with more
    // instance queries would break this identity.
    let n_instance_evals = instances.len();
    // Add one to the first advice evaluation. Every proof byte is still present and well-formed, so
    // the verifier parses the whole stream and rejects only at the final MSM identity check. This is
    // the one genuinely semantic rejection here (contrast the truncated/desynced cases below).
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
    // This non-identity MSM is only checked here in Rust; it is not exported to Lean, so the Lean
    // model is never cross-checked against a rejecting run. See the trust-boundary note in the
    // module docs (`super`) for the Halo2-side follow-up needed to close that gap.
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

    // The multiopen u-evaluations are exactly the ReadScalars between the x3 and x4 squeezes, so
    // recover their offset and count from the recorded event stream rather than re-deriving them
    // from the circuit's shape counts. Squeeze order: theta, beta, gamma, y, x, x1, x2, x3, x4, ...
    // The `== 5` below pins the count to the circuit asserted by `assert_pinned_verifying_key`.
    let scalars_before_squeeze = |squeeze_index: usize| -> usize {
        let (mut squeezes, mut scalars) = (0usize, 0usize);
        for event in &transcript.events {
            match event {
                TranscriptEvent::ReadScalar(_) => scalars += 1,
                TranscriptEvent::Squeeze(_) => {
                    if squeezes == squeeze_index {
                        break;
                    }
                    squeezes += 1;
                }
                _ => {}
            }
        }
        scalars
    };
    let first_multiopen_u = scalars_before_squeeze(7); // ReadScalars before the x3 squeeze
    let n_multiopen_u = scalars_before_squeeze(8) - first_multiopen_u; // x3 -> x4
    assert_eq!(n_multiopen_u, 5);

    // The permutation-set evaluations sit inside the post-`x` scalar block, which no squeeze
    // subdivides, so their offset is still derived from the pinned circuit's shape counts (emitted
    // as `shape` in `Fixture2.lean`); `assert_pinned_verifying_key` above keeps them from drifting.
    let n_advice_queries = 25;
    let n_fixed_evals = 29;
    let n_permutation_common_evals = 15;

    // Truncate the proof just before its final multiopen evaluation. The stream is now one scalar
    // short, so the verifier exhausts the transcript mid-multiopen and halo2 surfaces `Error::Opening`
    // (every multiopen error is mapped to `Opening` in `plonk::verify_proof`). This exercises a
    // short/malformed stream, not a failed polynomial-opening check; we only assert that the deployed
    // verifier reaches the multiopen stage (>= 8 challenges: through the `x3` squeeze) and rejects.
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

    // Drop one permutation-set evaluation from the middle of the read-scalar stream. Skipping a
    // scalar desynchronizes every later read and leaves the stream one element short, so rejection
    // follows from the misaligned/exhausted transcript rather than from a permutation-specific
    // consistency check. We therefore assert only that the deployed verifier fails (`is_err()`).
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
