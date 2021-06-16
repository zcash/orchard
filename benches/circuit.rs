#[macro_use]
extern crate criterion;

use std::{convert::TryInto, iter};

use criterion::Criterion;
use halo2::dev::MockProver;
use pprof::criterion::{Output, PProfProfiler};

use group::{ff::Field, GroupEncoding};
use orchard::{
    circuit::{Instance, ProvingKey, VerifyingKey},
    keys::SpendValidatingKey,
    tree::MerklePath,
    value::{ValueCommitTrapdoor, ValueCommitment},
    Anchor, Circuit, Note, Proof,
};
use pasta_curves::pallas;
use rand::{rngs::OsRng, RngCore};

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = OsRng;

    let (circuits, instances): (Vec<_>, Vec<_>) = iter::once(())
        .map(|()| {
            let (_, fvk, spent_note) = Note::dummy(&mut rng, None);
            let sender_address = fvk.default_address();
            let nk = *fvk.nk();
            let rivk = *fvk.rivk();
            let nf_old = spent_note.nullifier(&fvk);
            let ak: SpendValidatingKey = fvk.into();
            let alpha = pallas::Scalar::random(&mut rng);
            let rk = ak.randomize(&alpha);

            let (_, _, output_note) = Note::dummy(&mut rng, Some(nf_old));
            let cmx = output_note.commitment().into();

            let value = spent_note.value() - output_note.value();
            let cv_net = ValueCommitment::derive(value.unwrap(), ValueCommitTrapdoor::zero());

            let path = MerklePath::dummy(&mut rng);
            let anchor = path.root(spent_note.commitment().into());

            (
                Circuit {
                    path: Some(path.auth_path()),
                    pos: Some(path.position()),
                    g_d_old: Some(sender_address.g_d()),
                    pk_d_old: Some(*sender_address.pk_d()),
                    v_old: Some(spent_note.value()),
                    rho_old: Some(spent_note.rho()),
                    psi_old: Some(spent_note.rseed().psi(&spent_note.rho())),
                    rcm_old: Some(spent_note.rseed().rcm(&spent_note.rho())),
                    cm_old: Some(spent_note.commitment()),
                    alpha: Some(alpha),
                    ak: Some(ak),
                    nk: Some(nk),
                    rivk: Some(rivk),
                    g_d_new_star: Some((*output_note.recipient().g_d()).to_bytes()),
                    pk_d_new_star: Some(output_note.recipient().pk_d().to_bytes()),
                    v_new: Some(output_note.value()),
                    psi_new: Some(output_note.rseed().psi(&output_note.rho())),
                    rcm_new: Some(output_note.rseed().rcm(&output_note.rho())),
                    rcv: Some(ValueCommitTrapdoor::zero()),
                },
                Instance {
                    anchor,
                    cv_net,
                    nf_old,
                    rk,
                    cmx,
                    enable_spend: true,
                    enable_output: true,
                },
            )
        })
        .unzip();

    let vk = VerifyingKey::build();
    for (circuit, instance) in circuits.iter().zip(instances.iter()) {
        assert_eq!(
            MockProver::run(
                12,
                circuit,
                instance
                    .to_halo2_instance(vk.vk.get_domain())
                    .iter()
                    .map(|p| p.iter().cloned().collect())
                    .collect()
            )
            .unwrap()
            .verify(),
            Ok(())
        );
    }

    let pk = ProvingKey::build();

    {
        let mut group = c.benchmark_group("proving");
        group.sample_size(10);
        group.bench_function("action-circuit", |b| {
            b.iter(|| {
                // Create a proof
                Proof::create(&pk, &circuits, &instances).unwrap()
            });
        });
    }

    {
        let mut group = c.benchmark_group("verifying");
        let proof = Proof::create(&pk, &circuits, &instances).unwrap();
        assert!(proof.verify(&vk, &instances).is_ok());
        group.bench_function("action-circuit", |b| {
            b.iter(|| proof.verify(&vk, &instances));
        });
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
criterion_main!(benches);
