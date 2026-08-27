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
//! * On the exact replayed proof string, the deployed `Blake2bRead` rejects truncation,
//!   non-canonical scalars, identity points, out-of-range coordinates, and non-residue
//!   coordinates, while a flipped point-sign bit decodes to the negated point.
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

use ff::{Field, PrimeField};
use group::{Curve, Group, GroupEncoding};
use halo2_proofs::plonk::fingerprint::{
    capture_proof_fingerprint, ChallengeRecorder, TranscriptEvent,
};
use halo2_proofs::transcript::{
    Blake2bRead, Blake2bWrite, Challenge255, EncodedChallenge, Transcript, TranscriptRead,
    TranscriptWrite,
};
use pasta_curves::vesta;
use rand_chacha::ChaCha20Rng;

use super::super::{OrchardCircuitVersion, VerifyingKey, K};
use super::{assert_pinned_verifying_key, fixture_rng, raw_instance_refs};

/// Public-instance rows per action for the pinned Post-NU6.3 circuit
/// (`Instance::to_halo2_instance` returns one column of ten rows).
const INSTANCE_ROWS: usize = 10;

/// Compressed Vesta points and scalar-field elements both occupy 32 bytes in Halo2 proofs.
const ENCODED_ELEMENT_BYTES: usize = 32;

/// The high bit of the last byte selects the compressed point's `y` sign.
const COMPRESSED_POINT_SIGN_MASK: u8 = 0x80;

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

/// Return the canonical field modulus in the same little-endian byte order as `PrimeField::Repr`.
/// It is derived from the canonical representation of `-1`, rather than duplicating Pasta's
/// modulus as a byte literal in this test.
fn field_modulus_bytes<F: PrimeField>() -> Vec<u8> {
    let mut modulus = (-F::ONE).to_repr().as_ref().to_vec();
    let mut carry = 1u16;
    for byte in &mut modulus {
        let sum = u16::from(*byte) + carry;
        *byte = sum as u8;
        carry = sum >> 8;
        if carry == 0 {
            break;
        }
    }
    assert_eq!(carry, 0, "field modulus must fit in its representation");
    modulus
}

/// Add equal-length little-endian integers, asserting that the sum fits in the same width.
fn add_le_bytes(lhs: &[u8], rhs: &[u8]) -> Vec<u8> {
    assert_eq!(lhs.len(), rhs.len());
    let mut sum = Vec::with_capacity(lhs.len());
    let mut carry = 0u16;
    for (&lhs_byte, &rhs_byte) in lhs.iter().zip(rhs) {
        let limb = u16::from(lhs_byte) + u16::from(rhs_byte) + carry;
        sum.push(limb as u8);
        carry = limb >> 8;
    }
    assert_eq!(carry, 0, "little-endian sum must fit in its encoding");
    sum
}

fn deployed_read_first_point(proof_bytes: &[u8]) -> io::Result<vesta::Affine> {
    let mut transcript =
        Blake2bRead::<_, vesta::Affine, Challenge255<vesta::Affine>>::init(proof_bytes);
    transcript.read_point()
}

fn deployed_read_scalar(encoded_scalar: &[u8]) -> io::Result<vesta::Scalar> {
    let mut transcript =
        Blake2bRead::<_, vesta::Affine, Challenge255<vesta::Affine>>::init(encoded_scalar);
    transcript.read_scalar()
}

/// Exercise the byte-decoding cases mirrored by Ironwood on the exact random proof string that
/// this driver exports. The successful replay above supplies the typed values at both edited
/// locations, which binds the offsets and mutations to the verifier's actual read schedule.
fn assert_deployed_decoder_cases(
    proof_bytes: &[u8],
    first_point: vesta::Affine,
    final_scalar: vesta::Scalar,
) {
    assert!(proof_bytes.len() >= 2 * ENCODED_ELEMENT_BYTES);

    let encoded_first_point = first_point.to_bytes();
    assert_eq!(
        &proof_bytes[..ENCODED_ELEMENT_BYTES],
        encoded_first_point.as_ref(),
        "the proof string must begin with the replay's first point"
    );
    assert_eq!(
        deployed_read_first_point(proof_bytes).unwrap(),
        first_point,
        "the deployed reader must recover the replay's first point"
    );

    let final_scalar_offset = proof_bytes.len() - ENCODED_ELEMENT_BYTES;
    let encoded_final_scalar = final_scalar.to_repr();
    assert_eq!(
        &proof_bytes[final_scalar_offset..],
        encoded_final_scalar.as_ref(),
        "the proof string must end with the replay's final scalar"
    );
    assert_eq!(
        deployed_read_scalar(&proof_bytes[final_scalar_offset..]).unwrap(),
        final_scalar,
        "the deployed reader must recover the replay's final scalar"
    );

    let mut truncated = proof_bytes.to_vec();
    truncated.pop().unwrap();
    assert!(
        deployed_read_scalar(&truncated[final_scalar_offset..]).is_err(),
        "the deployed reader must reject a truncated final scalar"
    );

    // Re-encode the final scalar as the same integer plus the scalar-field modulus. The value is
    // congruent modulo the field but its proof encoding is non-canonical.
    let scalar_modulus = field_modulus_bytes::<vesta::Scalar>();
    assert_eq!(scalar_modulus.len(), ENCODED_ELEMENT_BYTES);
    let non_canonical_scalar = add_le_bytes(encoded_final_scalar.as_ref(), &scalar_modulus);
    let mut non_canonical = proof_bytes.to_vec();
    non_canonical[final_scalar_offset..].copy_from_slice(&non_canonical_scalar);
    assert!(
        deployed_read_scalar(&non_canonical[final_scalar_offset..]).is_err(),
        "the deployed reader must reject a scalar encoded as value + modulus"
    );

    let mut identity = proof_bytes.to_vec();
    identity[..ENCODED_ELEMENT_BYTES].fill(0);
    assert!(
        deployed_read_first_point(&identity).is_err(),
        "the deployed transcript must reject the identity point"
    );

    // Vesta's base field is the Pallas scalar field. Its modulus is just outside the canonical
    // coordinate range, so this is not a valid compressed point encoding.
    let base_modulus = field_modulus_bytes::<vesta::Base>();
    assert_eq!(base_modulus.len(), ENCODED_ELEMENT_BYTES);
    let mut out_of_range = proof_bytes.to_vec();
    out_of_range[..ENCODED_ELEMENT_BYTES].copy_from_slice(&base_modulus);
    assert!(
        deployed_read_first_point(&out_of_range).is_err(),
        "the deployed reader must reject x equal to the base-field modulus"
    );

    // With the sign bit clear, x = 2 has curve-equation radicand 2^3 + 5 = 13, a non-residue in
    // the Vesta base field.
    let mut non_residue_encoding = [0u8; ENCODED_ELEMENT_BYTES];
    non_residue_encoding[0] = 2;
    let mut non_residue = proof_bytes.to_vec();
    non_residue[..ENCODED_ELEMENT_BYTES].copy_from_slice(&non_residue_encoding);
    assert!(
        deployed_read_first_point(&non_residue).is_err(),
        "the deployed reader must reject a compressed x with no curve point"
    );

    let mut flipped_sign = proof_bytes.to_vec();
    flipped_sign[ENCODED_ELEMENT_BYTES - 1] ^= COMPRESSED_POINT_SIGN_MASK;
    let decoded_flipped = deployed_read_first_point(&flipped_sign)
        .expect("flipping only the sign bit must remain a canonical point encoding");
    assert_eq!(
        decoded_flipped,
        (-vesta::Point::from(first_point)).to_affine(),
        "the compressed sign bit must select the negated point"
    );
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

    assert_deployed_decoder_cases(
        &proof_bytes,
        *replay
            .points
            .first()
            .expect("the deployed verifier must read at least one proof point"),
        *replay
            .scalars
            .last()
            .expect("the deployed verifier must read at least one proof scalar"),
    );

    std::eprintln!(
        "Captured {num_actions}-action random match-only Orchard fingerprint at k={K} with {} challenges and {} proof bytes",
        replay.challenges.len(),
        proof_bytes.len(),
    );

    // The fabricated proof bytes — exactly what the replay consumed — go to the exporter, which
    // checks that the replay's reads re-serialize to them and carries them as `capturedProofHex`.
    let fixture = vk.vk.dump_vesta_lean_fixture_match_only_with_proof_bytes(
        namespace,
        "PostNu6_3",
        K,
        &raw_instance_refs,
        &replay,
        &msm,
        &proof_bytes,
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
