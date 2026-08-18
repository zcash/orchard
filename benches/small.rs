use criterion::{black_box, criterion_group, criterion_main, Criterion};
use incrementalmerkletree::{Hashable, Level};
use orchard::keys::{FullViewingKey, Scope, SpendingKey};
use orchard::tree::MerkleHashOrchard;

/// Selects a nontrivial Merkle layer for the benchmarked compression.
const BENCHMARK_LEVEL: u8 = 7;
/// Selects a protocol-defined empty root for the left input.
const LEFT_ROOT_LEVEL: u8 = 4;
/// Selects a distinct protocol-defined empty root for the right input.
const RIGHT_ROOT_LEVEL: u8 = 11;

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

criterion_group!(benches, key_derivation, merkle_crh);
criterion_main!(benches);
