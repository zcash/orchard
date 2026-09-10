//! What one `MerkleCRH^Orchard` node costs.
//!
//! `Hashable::combine` is called once per internal node, so an incremental tree
//! pays 32 of these per appended leaf in the worst case and a full-tree
//! construction pays one per node. Anything constant-per-call that is not
//! hoisted out of it is multiplied by the whole shape of the tree.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use incrementalmerkletree::{Hashable, Level};
use orchard::tree::MerkleHashOrchard;

fn merkle_crh(c: &mut Criterion) {
    // Two arbitrary but canonical leaves. The cost of `combine` does not depend
    // on their values — it is a fixed number of Sinsemilla chunks either way —
    // so the only thing that matters here is that they are valid field
    // elements and are not equal.
    let left = MerkleHashOrchard::from_bytes(&{
        let mut bytes = [0u8; 32];
        bytes[0] = 1;
        bytes
    })
    .unwrap();
    let right = MerkleHashOrchard::from_bytes(&{
        let mut bytes = [0u8; 32];
        bytes[0] = 2;
        bytes
    })
    .unwrap();

    c.bench_function("merkle_crh_combine", |b| {
        b.iter(|| MerkleHashOrchard::combine(Level::from(0), &left, &right))
    });

    // One leaf's worth of a deep tree: the 32 combines an append walks up
    // through. This is the shape a chain indexer actually pays.
    c.bench_function("merkle_crh_path_32", |b| {
        b.iter_batched(
            || (left, right),
            |(mut node, right)| {
                for level in 0..32u8 {
                    node = MerkleHashOrchard::combine(Level::from(level), &node, &right);
                }
                node
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, merkle_crh);
criterion_main!(benches);
