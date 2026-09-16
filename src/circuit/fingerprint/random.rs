//! Random match-only fingerprint captures: the deployed verifier run on random proof strings.
//!
//! The honest captures in the parent module can only pin verifier behavior at honest proofs; any
//! Lean↔Rust divergence whose discrepancy vanishes on honest runs (e.g. sourcing a
//! transcript-claimed evaluation from a recomputation instead of the proof string) is invisible
//! to them. These captures instead run the deployed verifier on a *random* proof string, so every
//! proof-string slot carries an independently random value and the exported `MsmMatch` theorem
//! has bite at a generic point.
//!
//! # Fabricate → replay
//!
//! The proof string is produced in two passes, so its read schedule is the deployed verifier's
//! own and never hand-transcribed:
//!
//! 1. *Fabrication.* [`RandomizingTranscript`] wraps a real `Blake2bWrite` and implements the
//!    transcript-read traits by sampling a fresh random canonical value at each
//!    `read_point`/`read_scalar`, writing it through the inner transcript (absorb + buffer), and
//!    forwarding `common_point`/`common_scalar`/`squeeze_challenge` to the inner Blake2b state.
//!    Running the deployed `verify_proof` (via `capture_proof_fingerprint`) against it and
//!    finalizing the inner transcript yields the proof byte string.
//! 2. *Replay.* `ChallengeRecorder` over `Blake2bRead` of those bytes — identical to the honest
//!    capture pipeline — and the fixture is exported from the replay.
//!
//! `Blake2bWrite` and `Blake2bRead` drive byte-identical Fiat–Shamir state machines (same
//! personalization and domain-prefix bytes), so the replay reproduces the fabrication challenges
//! by construction; the driver asserts this, along with event-stream equality, rather than hoping.
//!
//! # What the driver asserts (fail loudly, never re-seed silently)
//!
//! * The deployed verifier ran to completion on the random proof string, twice. The accept path
//!   is straight-line in proof values post-decode; the only panics on it are challenge-degenerate
//!   inversions, so a panic here is a model-breaking discovery, not noise.
//! * Replay challenges and transcript events equal their fabrication counterparts.
//! * The captured MSM does **not** evaluate to the identity (a random proof string accepting
//!   would be its own discovery), asserted on both passes; the match-only exporter re-asserts it.
//! * The pinned verifying-key description and all of the honest exporter's structural guards
//!   (instance-commitment re-derivation, slot reconstruction, term counts) stay in force.
//!
//! Fabricated points are sampled as `generator * t` for nonzero random `t`, so their discrete
//! logs are known to the generator. That is harmless here — the capture is non-accepting and the
//! fixture witnesses coefficient agreement only — but it is recorded for honesty: these captures
//! are not adversarial proofs, just random points of the verifier's input space.

use alloc::vec::Vec;
use std::io;

use ff::Field;
use group::{Curve, Group};
use halo2_proofs::plonk::fingerprint::{
    capture_proof_fingerprint, ChallengeRecorder, TranscriptEvent,
};
use halo2_proofs::transcript::{
    Blake2bWrite, Challenge255, EncodedChallenge, Transcript, TranscriptRead, TranscriptWrite,
};
use pasta_curves::vesta;
use rand_chacha::ChaCha20Rng;

use super::super::{OrchardCircuitVersion, VerifyingKey, K};
use super::{assert_pinned_verifying_key, fixture_rng, raw_instance_refs};

/// Public-instance rows per action for the pinned Post-NU6.3 circuit
/// (`Instance::to_halo2_instance` returns one column of ten rows).
const INSTANCE_ROWS: usize = 10;

/// A transcript-read fabricator: samples a fresh random canonical value at each proof read,
/// absorbs and buffers it through an inner `Blake2bWrite`, and records the event stream and
/// squeezed challenges for comparison against the replay (see module docs).
struct RandomizingTranscript {
    inner: Blake2bWrite<Vec<u8>, vesta::Affine, Challenge255<vesta::Affine>>,
    rng: ChaCha20Rng,
    events: Vec<TranscriptEvent<vesta::Affine>>,
    challenges: Vec<vesta::Scalar>,
}

impl RandomizingTranscript {
    fn new(rng: ChaCha20Rng) -> Self {
        RandomizingTranscript {
            inner: Blake2bWrite::init(vec![]),
            rng,
            events: Vec::new(),
            challenges: Vec::new(),
        }
    }

    /// Rejection-sample a nonzero scalar, so a fabricated point is never the identity: the
    /// transcript rejects identity points, and the exporter's slot-reconstruction guard assumes
    /// non-identity bases. The loop is deterministic given the seed.
    fn sample_nonzero_scalar(&mut self) -> vesta::Scalar {
        loop {
            let t = vesta::Scalar::random(&mut self.rng);
            if !bool::from(t.is_zero()) {
                return t;
            }
        }
    }

    /// Finish fabrication: the proof byte string plus the recorded events and challenges.
    fn finalize(
        self,
    ) -> (
        Vec<u8>,
        Vec<TranscriptEvent<vesta::Affine>>,
        Vec<vesta::Scalar>,
    ) {
        (self.inner.finalize(), self.events, self.challenges)
    }
}

impl Transcript<vesta::Affine, Challenge255<vesta::Affine>> for RandomizingTranscript {
    fn squeeze_challenge(&mut self) -> Challenge255<vesta::Affine> {
        let challenge = self.inner.squeeze_challenge();
        let scalar = challenge.get_scalar();
        self.challenges.push(scalar);
        self.events.push(TranscriptEvent::Squeeze(scalar));
        challenge
    }

    fn common_point(&mut self, point: vesta::Affine) -> io::Result<()> {
        self.events.push(TranscriptEvent::CommonPoint(point));
        self.inner.common_point(point)
    }

    fn common_scalar(&mut self, scalar: vesta::Scalar) -> io::Result<()> {
        self.events.push(TranscriptEvent::CommonScalar(scalar));
        self.inner.common_scalar(scalar)
    }
}

impl TranscriptRead<vesta::Affine, Challenge255<vesta::Affine>> for RandomizingTranscript {
    fn read_point(&mut self) -> io::Result<vesta::Affine> {
        let t = self.sample_nonzero_scalar();
        let point = (vesta::Point::generator() * t).to_affine();
        // `write_point` absorbs via the inner `common_point` and buffers the compressed bytes;
        // calling `common_point` here as well would double-absorb and desynchronize the replay.
        self.inner.write_point(point)?;
        self.events.push(TranscriptEvent::ReadPoint(point));
        Ok(point)
    }

    fn read_scalar(&mut self) -> io::Result<vesta::Scalar> {
        // Zero is fine for scalars: every canonical field element round-trips through the proof
        // byte encoding.
        let scalar = vesta::Scalar::random(&mut self.rng);
        self.inner.write_scalar(scalar)?;
        self.events.push(TranscriptEvent::ReadScalar(scalar));
        Ok(scalar)
    }
}

/// Assert the replayed transcript event stream equals the fabricated one, value for value.
/// (`TranscriptEvent` does not implement `PartialEq`, so the comparison is spelled out.)
fn assert_transcript_events_eq(
    fabricated: &[TranscriptEvent<vesta::Affine>],
    replayed: &[TranscriptEvent<vesta::Affine>],
) {
    assert_eq!(
        fabricated.len(),
        replayed.len(),
        "transcript event counts diverged between fabrication and replay"
    );
    for (i, (fab, rep)) in fabricated.iter().zip(replayed.iter()).enumerate() {
        let same = match (fab, rep) {
            (TranscriptEvent::CommonPoint(a), TranscriptEvent::CommonPoint(b)) => a == b,
            (TranscriptEvent::CommonScalar(a), TranscriptEvent::CommonScalar(b)) => a == b,
            (TranscriptEvent::ReadPoint(a), TranscriptEvent::ReadPoint(b)) => a == b,
            (TranscriptEvent::ReadScalar(a), TranscriptEvent::ReadScalar(b)) => a == b,
            (TranscriptEvent::Squeeze(a), TranscriptEvent::Squeeze(b)) => a == b,
            _ => false,
        };
        assert!(
            same,
            "transcript event {i} diverged between fabrication and replay: {fab:?} vs {rep:?}"
        );
    }
}

/// Shared driver for the random match-only captures: fabricate a random proof string against the
/// deployed verifier's own read schedule, replay it through the honest capture pipeline, and
/// export a match-only fixture (see module docs for the assertion inventory).
pub(super) fn capture_random_fixture(
    seed: u8,
    num_actions: u8,
    namespace: &str,
    fixture_output_var: &str,
    proof_output_var: &str,
) {
    let mut rng = fixture_rng(seed);
    // No proving key and no bundle: random captures need synthesis (inside `keygen_vk`) plus two
    // verifier runs, never a prover run.
    let vk = VerifyingKey::build(OrchardCircuitVersion::PostNu6_3);
    assert!(vk.supports_cross_address_restriction());
    assert_pinned_verifying_key(&vk);

    // Random public instances with the honest captures' shape: one column of `INSTANCE_ROWS`
    // rows per action, sampled as raw field elements at the halo2 interface (the typed orchard
    // `Instance` would demand real curve points; the verifier only consumes field elements).
    // Instances are drawn from the seeded stream first; the rng then moves into the transcript.
    let raw_instances: Vec<Vec<Vec<vesta::Scalar>>> = (0..num_actions)
        .map(|_| {
            vec![(0..INSTANCE_ROWS)
                .map(|_| vesta::Scalar::random(&mut rng))
                .collect()]
        })
        .collect();
    let raw_instance_refs = raw_instance_refs(&raw_instances);
    let raw_instance_refs: Vec<_> = raw_instance_refs
        .iter()
        .map(|instance| &instance[..])
        .collect();

    // Fabrication pass: the deployed verifier's own read schedule drives the sampling.
    let mut fabricate = RandomizingTranscript::new(rng);
    let fabricated_msm =
        capture_proof_fingerprint(&vk.params, &vk.vk, &raw_instance_refs, &mut fabricate)
            .expect("deployed verifier must run to completion on the random proof string");
    assert!(
        !fabricated_msm.eval(),
        "random capture must not assemble the identity MSM"
    );
    let (proof_bytes, fabricated_events, fabricated_challenges) = fabricate.finalize();

    // Replay pass: identical to the honest capture pipeline.
    let mut replay = ChallengeRecorder::<_, _, Challenge255<_>>::init(&proof_bytes[..]);
    let msm = capture_proof_fingerprint(&vk.params, &vk.vk, &raw_instance_refs, &mut replay)
        .expect("replay of the fabricated proof bytes must parse and run to completion");
    assert_eq!(
        replay.challenges, fabricated_challenges,
        "replay challenges diverged from fabrication"
    );
    assert_transcript_events_eq(&fabricated_events, &replay.events);
    assert!(
        !msm.clone().eval(),
        "replayed random capture must not assemble the identity MSM"
    );

    std::eprintln!(
        "Captured {num_actions}-action random match-only Orchard fingerprint at k={K} with {} challenges and {} proof bytes",
        replay.challenges.len(),
        proof_bytes.len(),
    );

    let fixture = vk.vk.dump_vesta_lean_fixture_match_only(
        namespace,
        "PostNu6_3",
        K,
        &raw_instance_refs,
        &replay,
        &msm,
    );
    if let Some(path) = std::env::var_os(fixture_output_var) {
        std::fs::write(std::path::PathBuf::from(path), fixture).unwrap();
    }
    if let Some(path) = std::env::var_os(proof_output_var) {
        let mut encoded = hex::encode(&proof_bytes);
        encoded.push('\n');
        std::fs::write(std::path::PathBuf::from(path), encoded).unwrap();
    }
}
