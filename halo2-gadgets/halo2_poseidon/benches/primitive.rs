use criterion::{criterion_group, criterion_main, Criterion};
use ff::Field;
use halo2_poseidon::primitive::{ConstantLength, Hash, P128Pow5T3};

use pasta_curves::pallas;
#[cfg(unix)]
use pprof::criterion::{Output, PProfProfiler};
use rand::rngs::OsRng;

fn bench_primitives(c: &mut Criterion) {
    let rng = OsRng;

    {
        let mut group = c.benchmark_group("Poseidon");

        let message = [pallas::Base::random(rng), pallas::Base::random(rng)];

        group.bench_function("2-to-1", |b| {
            b.iter(|| Hash::init(P128Pow5T3, ConstantLength).hash(message))
        });
    }
}

#[cfg(unix)]
criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench_primitives
}
#[cfg(not(unix))]
criterion_group!(benches, bench_primitives);
criterion_main!(benches);
