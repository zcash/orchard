//! Manual benchmarks for the complete Orchard proving and verification paths.

use alloc::vec::Vec;
use ff::PrimeField;
use halo2_proofs::plonk::BatchVerifier;
use pasta_curves::vesta;
use rand::{rngs::StdRng, SeedableRng};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    println,
    time::Instant,
};

use super::{
    Instance, OrchardCircuitVersion, ProvingKey, VerifyingKey, INSTANCE_COLUMNS, INSTANCE_ROWS, K,
};
use crate::{
    builder::{Builder, BundleType},
    bundle::{BundleVersion, Flags},
    keys::{FullViewingKey, Scope, SpendingKey},
    value::NoteValue,
    Anchor, Bundle,
};

type Halo2Instances = Vec<Vec<Vec<vesta::Scalar>>>;
type EncodedFixture = (Halo2Instances, Vec<u8>);

const BATCH_BENCH_SIZES: [usize; 4] = [1, 2, 16, 64];
const BATCH_SCREEN_SIZES: [usize; 2] = [1, 64];
const BATCH_BENCH_WARMUPS: usize = 3;
const BATCH_BENCH_SAMPLES: usize = 15;
const BATCH_SCREEN_WARMUPS: usize = 1;
const BATCH_SCREEN_SAMPLES: usize = 7;

const FIXTURE_ACTIONS: usize = 1;
const FIXTURE_ADDRESS_INDEX: u32 = 0;
const FIXTURE_NOTE_VALUE: u64 = 10;
/// The Orchard builder API fixes memo fields at 512 bytes.
const FIXTURE_MEMO_SIZE: usize = 512;
const FIXTURE_MEMO: [u8; FIXTURE_MEMO_SIZE] = [0; FIXTURE_MEMO_SIZE];
const FIXTURE_SPENDING_KEY: [u8; 32] = [7; 32];
const FIXTURE_ANCHOR: [u8; 32] = [0; 32];
const FIXTURE_SEED_DOMAIN: u8 = 0x42;
const PROOF_SEED_DOMAIN: u8 = 0x24;
const INDEX_SEED_BYTES: usize = core::mem::size_of::<u64>();

const BATCH_FIXTURE_MAGIC: &[u8] = b"ZAKURA_ORCHARD_BATCH_CORPUS_V1";

fn benchmark_rng(domain: u8, index: usize) -> StdRng {
    let mut seed = [domain; 32];
    let index = u64::try_from(index).expect("fixture index fits into u64");
    seed[..INDEX_SEED_BYTES].copy_from_slice(&index.to_le_bytes());
    StdRng::from_seed(seed)
}

fn take_fixture_bytes<'a>(input: &mut &'a [u8], len: usize) -> &'a [u8] {
    assert!(input.len() >= len, "truncated batch fixture corpus");
    let (head, tail) = input.split_at(len);
    *input = tail;
    head
}

fn take_fixture_u64(input: &mut &[u8]) -> u64 {
    u64::from_le_bytes(
        take_fixture_bytes(input, INDEX_SEED_BYTES)
            .try_into()
            .expect("eight-byte fixture integer"),
    )
}

fn encode_batch_fixtures(encoded: &[EncodedFixture]) -> Vec<u8> {
    let mut corpus = Vec::new();
    corpus.extend_from_slice(BATCH_FIXTURE_MAGIC);
    corpus.extend_from_slice(
        &u64::try_from(encoded.len())
            .expect("fixture count fits into u64")
            .to_le_bytes(),
    );

    for (instances, proof) in encoded {
        assert_eq!(instances.len(), FIXTURE_ACTIONS);
        assert_eq!(instances[0].len(), INSTANCE_COLUMNS);
        assert_eq!(instances[0][0].len(), INSTANCE_ROWS);
        for scalar in &instances[0][0] {
            corpus.extend_from_slice(scalar.to_repr().as_ref());
        }
        corpus.extend_from_slice(
            &u64::try_from(proof.len())
                .expect("proof length fits into u64")
                .to_le_bytes(),
        );
        corpus.extend_from_slice(proof);
    }

    corpus
}

fn write_batch_fixtures(path: &std::path::Path, encoded: &[EncodedFixture]) {
    let corpus = encode_batch_fixtures(encoded);

    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .expect("batch fixture corpus path must not exist");
    output
        .write_all(&corpus)
        .expect("write batch fixture corpus");
}

fn decode_batch_fixtures(corpus: &[u8]) -> Vec<EncodedFixture> {
    let mut input = corpus;
    assert_eq!(
        take_fixture_bytes(&mut input, BATCH_FIXTURE_MAGIC.len()),
        BATCH_FIXTURE_MAGIC,
    );
    let proof_count =
        usize::try_from(take_fixture_u64(&mut input)).expect("fixture count fits usize");

    let fixtures = (0..proof_count)
        .map(|_| {
            let column = (0..INSTANCE_ROWS)
                .map(|_| {
                    let mut repr = <vesta::Scalar as PrimeField>::Repr::default();
                    let repr_bytes = repr.as_mut();
                    let encoded = take_fixture_bytes(&mut input, repr_bytes.len());
                    repr_bytes.copy_from_slice(encoded);
                    Option::<vesta::Scalar>::from(vesta::Scalar::from_repr(repr))
                        .expect("canonical fixture scalar")
                })
                .collect::<Vec<_>>();
            let proof_len =
                usize::try_from(take_fixture_u64(&mut input)).expect("proof length fits usize");
            let proof = take_fixture_bytes(&mut input, proof_len).to_vec();
            (vec![vec![column]], proof)
        })
        .collect::<Vec<_>>();
    assert!(input.is_empty(), "trailing batch fixture bytes");
    fixtures
}

fn read_batch_fixtures(path: &std::path::Path) -> Vec<EncodedFixture> {
    let corpus = fs::read(path).expect("read batch fixture corpus");
    decode_batch_fixtures(&corpus)
}

fn create_batch_fixtures(
    version: OrchardCircuitVersion,
    vk: &VerifyingKey,
    proof_count: usize,
) -> Vec<EncodedFixture> {
    let pk = ProvingKey::build(version);
    let sk = SpendingKey::from_bytes(FIXTURE_SPENDING_KEY).unwrap();
    let recipient = FullViewingKey::from(&sk).address_at(FIXTURE_ADDRESS_INDEX, Scope::External);

    (0..proof_count)
        .map(|index| {
            let mut fixture_rng = benchmark_rng(FIXTURE_SEED_DOMAIN, index);
            let mut builder = Builder::new(
                BundleType::Coinbase,
                BundleVersion::orchard_v2(),
                Flags::SPENDS_DISABLED,
                Anchor::from_bytes(FIXTURE_ANCHOR).unwrap(),
            )
            .unwrap();
            builder
                .add_output(
                    None,
                    recipient,
                    NoteValue::from_raw(FIXTURE_NOTE_VALUE),
                    FIXTURE_MEMO,
                )
                .unwrap();
            let bundle: Bundle<_, i64> = builder
                .build(&mut fixture_rng)
                .unwrap()
                .expect("one output produces an Orchard bundle")
                .0;
            assert_eq!(bundle.actions().len(), FIXTURE_ACTIONS);

            let instances = bundle
                .actions()
                .iter()
                .map(|action| action.to_instance(*bundle.flags(), *bundle.anchor()))
                .collect::<Vec<_>>();
            let proof = bundle
                .authorization()
                .create_proof(&pk, &instances, benchmark_rng(PROOF_SEED_DOMAIN, index))
                .expect("proof creation should succeed");
            proof
                .verify(vk, &instances)
                .expect("benchmark proof should verify");

            let instances = instances
                .iter()
                .map(Instance::to_halo2_instance)
                .map(|columns| {
                    columns
                        .into_iter()
                        .map(|column| column.into_iter().collect())
                        .collect()
                })
                .collect();
            (instances, proof.as_ref().to_vec())
        })
        .collect()
}

fn validate_batch_fixtures(encoded: &[EncodedFixture], vk: &VerifyingKey, expected_count: usize) {
    assert_eq!(encoded.len(), expected_count);
    let unique_proofs = encoded
        .iter()
        .map(|(_, proof)| proof.as_slice())
        .collect::<BTreeSet<_>>();
    assert_eq!(unique_proofs.len(), expected_count);

    let mut batch = BatchVerifier::new();
    for (instances, proof) in encoded.iter().cloned() {
        batch.add_proof(instances, proof);
    }
    assert!(batch.finalize(&vk.params, &vk.vk));
}

#[test]
fn batch_fixture_encoding_roundtrips() {
    const TEST_PROOF: &[u8] = b"fixture-codec-test";

    let column = (0..INSTANCE_ROWS)
        .map(|index| {
            vesta::Scalar::from(u64::try_from(index).expect("instance row index fits into u64"))
        })
        .collect::<Vec<_>>();
    let encoded = vec![(vec![vec![column]], TEST_PROOF.to_vec())];

    assert_eq!(
        decode_batch_fixtures(&encode_batch_fixtures(&encoded)),
        encoded
    );
}

#[test]
#[ignore = "manual single-thread performance benchmark"]
fn benchmark_batch_verifier() {
    assert_eq!(K, 11, "Orchard's protocol-fixed circuit size changed");
    assert_eq!(
        std::env::var("RAYON_NUM_THREADS").ok().as_deref(),
        Some("1"),
        "run this benchmark with RAYON_NUM_THREADS=1",
    );

    let version = OrchardCircuitVersion::FixedPostNu6_2;
    let vk = VerifyingKey::build(version);
    let proof_count = *BATCH_BENCH_SIZES.last().expect("batch sizes are nonempty");
    let fixture_path = std::env::var_os("ORCHARD_BATCH_FIXTURE_CORPUS")
        .map(std::path::PathBuf::from)
        .expect("set ORCHARD_BATCH_FIXTURE_CORPUS");
    let encoded = if std::env::var_os("ORCHARD_BATCH_FIXTURE_GENERATE").is_some() {
        let fixtures = create_batch_fixtures(version, &vk, proof_count);
        write_batch_fixtures(&fixture_path, &fixtures);
        fixtures
    } else {
        read_batch_fixtures(&fixture_path)
    };

    // Corpus loading and full proof validation are deliberately outside every
    // timed sample. Each benchmark binary independently performs this check.
    validate_batch_fixtures(&encoded, &vk, proof_count);

    let screen = std::env::var_os("ORCHARD_BATCH_SCREEN").is_some();
    let batch_sizes: &[usize] = if screen {
        &BATCH_SCREEN_SIZES
    } else {
        &BATCH_BENCH_SIZES
    };
    let (warmups, sample_count) = if screen {
        (BATCH_SCREEN_WARMUPS, BATCH_SCREEN_SAMPLES)
    } else {
        (BATCH_BENCH_WARMUPS, BATCH_BENCH_SAMPLES)
    };

    for &batch_size in batch_sizes {
        let mut samples = Vec::with_capacity(sample_count);
        for sample in 0..warmups + sample_count {
            let entries = encoded.iter().take(batch_size).cloned().collect::<Vec<_>>();

            let start = Instant::now();
            let mut batch = BatchVerifier::<vesta::Affine>::new();
            for (instances, proof) in entries {
                batch.add_proof(instances, proof);
            }
            assert!(batch.finalize(&vk.params, &vk.vk));
            let elapsed = start.elapsed();

            if sample >= warmups {
                samples.push(elapsed.as_nanos());
            }
        }
        println!("BATCH_BENCH size={batch_size} samples_ns={samples:?}");
    }
}
