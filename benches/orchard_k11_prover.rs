use criterion::{criterion_group, criterion_main, Criterion};

#[path = "support/orchard_k11.rs"]
mod orchard_k11;

fn orchard_k11_prover(c: &mut Criterion) {
    assert_eq!(
        std::env::var("RAYON_NUM_THREADS").ok().as_deref(),
        Some("1"),
        "run this benchmark with RAYON_NUM_THREADS=1",
    );

    orchard_k11::bench_prove_one_action(c, "orchard-k11");
}

criterion_group!(benches, orchard_k11_prover);
criterion_main!(benches);
