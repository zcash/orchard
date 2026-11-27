//! The one-Action proof of `orchard_k11_prover`, on every Rayon worker
//! instead of one: it measures how much of the prover runs in parallel.

use criterion::{criterion_group, criterion_main, Criterion};

#[path = "support/orchard_k11.rs"]
mod orchard_k11;

fn orchard_k11_prover_multithreaded(c: &mut Criterion) {
    let requested = std::env::var("RAYON_NUM_THREADS").ok();
    assert_ne!(
        requested.as_deref(),
        Some("1"),
        "this benchmark measures multi-threaded proving; \
         use orchard_k11_prover for one Rayon worker",
    );
    let workers = requested
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
    eprintln!("orchard_k11_prover_multithreaded: {workers} Rayon workers");

    orchard_k11::bench_prove_one_action(c, "orchard-k11-multithreaded");
}

criterion_group!(benches, orchard_k11_prover_multithreaded);
criterion_main!(benches);
