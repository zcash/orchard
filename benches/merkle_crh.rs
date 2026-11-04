//! MerkleCRH^Orchard: `combine`, a level of per-pair `combine`s, and `combine_pairs` by batch size

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ff::{Field, PrimeField};
use incrementalmerkletree::{Hashable, Level};
use orchard::tree::MerkleHashOrchard;
use pasta_curves::pallas;
use rand::{rngs::ChaCha8Rng, SeedableRng};

/// Same children every run (A/B runs compare identical inputs)
const SEED: [u8; 32] = [7; 32];

/// Any level costs the same (the layer is one message word)
const LEVEL: u8 = 15;

/// Per-pair `combine` cost is batch-size independent (control for the A/B)
const LEVEL_PAIRS: [usize; 3] = [1, 32, 256];

/// `combine_pairs` sizes: a lone frontier carry up to a bulk-sync run's leaf level
const BATCH_PAIRS: [usize; 8] = [1, 16, 64, 128, 512, 2048, 8192, 32768];

fn children(pairs: usize) -> Vec<MerkleHashOrchard> {
    let rng = &mut ChaCha8Rng::from_seed(SEED);
    (0..2 * pairs)
        .map(|_| {
            let base = pallas::Base::random(&mut *rng);
            MerkleHashOrchard::from_bytes(&base.to_repr()).expect("a field element is canonical")
        })
        .collect()
}

fn bench_combine(c: &mut Criterion) {
    let level = Level::from(LEVEL);
    let two = children(1);
    c.bench_function("merkle_crh/combine", |b| {
        b.iter(|| MerkleHashOrchard::combine(level, black_box(&two[0]), black_box(&two[1])))
    });

    let mut group = c.benchmark_group("merkle_crh/combine_level");
    for pairs in LEVEL_PAIRS {
        let children = children(pairs);
        group.throughput(Throughput::Elements(pairs as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(pairs),
            &children,
            |b, children| {
                b.iter(|| {
                    black_box(children)
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|[left, right]| MerkleHashOrchard::combine(level, left, right))
                        .collect::<Vec<_>>()
                })
            },
        );
    }
    group.finish();
}

fn bench_combine_pairs(c: &mut Criterion) {
    let level = Level::from(LEVEL);
    let mut group = c.benchmark_group("merkle_crh/combine_pairs");
    for pairs in BATCH_PAIRS {
        let children = children(pairs);
        group.throughput(Throughput::Elements(pairs as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(pairs),
            &children,
            |b, children| b.iter(|| MerkleHashOrchard::combine_pairs(level, black_box(children))),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_combine, bench_combine_pairs);
criterion_main!(benches);
