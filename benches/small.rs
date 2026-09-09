use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use incrementalmerkletree::{Hashable, Level};
use orchard::keys::{FullViewingKey, Scope, SpendingKey};
use orchard::tree::MerkleHashOrchard;

/// Selects a nontrivial Merkle layer for the benchmarked compression.
const BENCHMARK_LEVEL: u8 = 7;
/// Selects a protocol-defined empty root for the left input.
const LEFT_ROOT_LEVEL: u8 = 4;
/// Selects a distinct protocol-defined empty root for the right input.
const RIGHT_ROOT_LEVEL: u8 = 11;
/// Batch widths span every power of two through the widest parent level of a
/// 1,024-leaf subtree.
const BATCH_WIDTHS: [usize; 10] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512];
/// Empty roots seed deterministic, protocol-valid benchmark inputs.
const INPUT_ROOT_LEVELS: [u8; 8] = [1, 4, 7, 10, 13, 16, 19, 22];

fn key_derivation(c: &mut Criterion) {
    // Meaningless random spending key.
    let sk = SpendingKey::from_bytes([
        0x2e, 0x0f, 0xd6, 0xc0, 0xed, 0x0b, 0xcf, 0xd8, 0x07, 0xf5, 0xdb, 0xff, 0x47, 0x4e, 0xdc,
        0x78, 0x8c, 0xe0, 0x09, 0x30, 0x66, 0x10, 0x1e, 0x95, 0x82, 0x87, 0xb1, 0x00, 0x50, 0x9b,
        0xf7, 0x9a,
    ])
    .unwrap();
    let fvk = FullViewingKey::from(&sk);

    c.bench_function("derive_fvk", |b| b.iter(|| FullViewingKey::from(&sk)));
    c.bench_function("default_address", |b| {
        b.iter(|| fvk.address_at(0u32, Scope::External))
    });
}

fn merkle_crh(c: &mut Criterion) {
    let level = Level::from(BENCHMARK_LEVEL);
    let left = MerkleHashOrchard::empty_root(Level::from(LEFT_ROOT_LEVEL));
    let right = MerkleHashOrchard::empty_root(Level::from(RIGHT_ROOT_LEVEL));

    c.bench_function("merkle_crh", |b| {
        b.iter(|| MerkleHashOrchard::combine(level, black_box(&left), black_box(&right)))
    });
}

fn combine_scalar(
    level: Level,
    pairs: &[(MerkleHashOrchard, MerkleHashOrchard)],
) -> Vec<MerkleHashOrchard> {
    pairs
        .iter()
        .map(|(left, right)| MerkleHashOrchard::combine(level, left, right))
        .collect()
}

fn merkle_crh_batch(c: &mut Criterion) {
    let level = Level::from(BENCHMARK_LEVEL);
    let mut group = c.benchmark_group("merkle_crh_batch");
    let widest_batch = *BATCH_WIDTHS.last().expect("batch widths are nonempty");
    let mut nodes: Vec<_> = INPUT_ROOT_LEVELS
        .into_iter()
        .map(|root_level| MerkleHashOrchard::empty_root(Level::from(root_level)))
        .collect();

    while nodes.len() < widest_batch * 2 {
        let index = nodes.len();
        let next = MerkleHashOrchard::combine(
            level,
            &nodes[index - 1],
            &nodes[index % INPUT_ROOT_LEVELS.len()],
        );
        nodes.push(next);
    }

    for width in BATCH_WIDTHS {
        let pairs: Vec<_> = nodes[..width * 2]
            .chunks_exact(2)
            .map(|pair| (pair[0], pair[1]))
            .collect();
        let expected = combine_scalar(level, &pairs);
        let actual = MerkleHashOrchard::combine_batch(
            level,
            pairs.iter().map(|(left, right)| (left, right)),
        );
        assert_eq!(actual, expected, "scalar/batch mismatch at width {width}");

        group.throughput(Throughput::Elements(
            u64::try_from(width).expect("batch width fits in u64"),
        ));
        group.bench_with_input(BenchmarkId::new("scalar", width), &width, |b, _| {
            b.iter(|| combine_scalar(level, black_box(&pairs)))
        });
        group.bench_with_input(BenchmarkId::new("batch", width), &width, |b, _| {
            b.iter(|| {
                MerkleHashOrchard::combine_batch(
                    level,
                    black_box(&pairs).iter().map(|(left, right)| (left, right)),
                )
            })
        });
    }

    group.finish();
}

criterion_group!(benches, key_derivation, merkle_crh, merkle_crh_batch);
criterion_main!(benches);
