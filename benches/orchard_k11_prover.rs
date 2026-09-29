use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, SamplingMode};
use orchard::{
    builder::{Builder, BundleType},
    bundle::{BundleVersion, Flags},
    circuit::{OrchardCircuitVersion, ProvingKey, VerifyingKey},
    keys::{FullViewingKey, Scope, SpendingKey},
    value::NoteValue,
    Anchor, Bundle,
};
use rand::{rngs::StdRng, SeedableRng};

const FIXTURE_ACTIONS: usize = 1;
const FIXTURE_ADDRESS_INDEX: u32 = 0;
const FIXTURE_NOTE_VALUE: u64 = 10;
/// The Orchard builder API fixes memo fields at 512 bytes.
const FIXTURE_MEMO_SIZE: usize = 512;
const FIXTURE_MEMO: [u8; FIXTURE_MEMO_SIZE] = [0; FIXTURE_MEMO_SIZE];
const FIXTURE_SPENDING_KEY: [u8; 32] = [7; 32];
const FIXTURE_ANCHOR: [u8; 32] = [0; 32];
const FIXTURE_SEED: [u8; 32] = [0x42; 32];
const PROOF_SEED: [u8; 32] = [0x24; 32];

const BENCHMARK_SAMPLES: usize = 10;
const WARMUP_SECONDS: u64 = 2;
const MEASUREMENT_SECONDS: u64 = 15;

fn orchard_k11_prover(c: &mut Criterion) {
    assert_eq!(
        std::env::var("RAYON_NUM_THREADS").ok().as_deref(),
        Some("1"),
        "run this benchmark with RAYON_NUM_THREADS=1",
    );

    // Orchard fixes its Action circuit size at k = 11. Key generation is
    // deliberately performed before registering the timed proving routine.
    let version = OrchardCircuitVersion::FixedPostNu6_2;
    let vk = VerifyingKey::build(version);
    let pk = ProvingKey::build(version);

    let sk = SpendingKey::from_bytes(FIXTURE_SPENDING_KEY).unwrap();
    let recipient = FullViewingKey::from(&sk).address_at(FIXTURE_ADDRESS_INDEX, Scope::External);
    let mut fixture_rng = StdRng::from_seed(FIXTURE_SEED);
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

    // Check the exact fixture and proving path once, outside the timed region.
    let proof = bundle
        .authorization()
        .create_proof(&pk, &instances, StdRng::from_seed(PROOF_SEED))
        .unwrap();
    proof
        .verify(&vk, &instances)
        .expect("the benchmark proof must verify");

    let mut group = c.benchmark_group("orchard-k11");
    group.sample_size(BENCHMARK_SAMPLES);
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(WARMUP_SECONDS));
    group.measurement_time(Duration::from_secs(MEASUREMENT_SECONDS));
    group.bench_function("prove-one-action", |bencher| {
        bencher.iter_batched(
            || StdRng::from_seed(PROOF_SEED),
            |proof_rng| {
                black_box(
                    bundle
                        .authorization()
                        .create_proof(&pk, &instances, proof_rng)
                        .unwrap(),
                )
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();
}

criterion_group!(benches, orchard_k11_prover);
criterion_main!(benches);
