use alloc::vec;

use super::{Bundle, BundleVersion};

/// `binding_validating_key` equals its constant-time definition
/// `sum(cv_net) - ValueCommit_0(valueBalance)` at the `i64` bounds, at every power of two, and
/// at random value balances, for Orchard and Ironwood bundles.
#[test]
fn binding_validating_key_matches_constant_time_derivation() {
    use proptest::strategy::{Strategy, ValueTree};
    use rand::{rngs::StdRng, RngExt, SeedableRng};

    use super::{testing::arb_unauthorized_bundle, EffectsOnly};
    use crate::value::{ValueCommitTrapdoor, ValueCommitment, ValueSum};

    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let bundles: vec::Vec<Bundle<EffectsOnly, i64>> = (1..=4)
        .flat_map(|n_actions| {
            let bundle = arb_unauthorized_bundle(n_actions)
                .new_tree(&mut runner)
                .expect("strategy can generate a bundle")
                .current();
            [BundleVersion::orchard_v2(), BundleVersion::ironwood_v3()].map(|version| {
                Bundle::from_parts_unchecked(
                    bundle.actions().clone(),
                    version.default_flags(),
                    0,
                    *bundle.anchor(),
                    EffectsOnly,
                    version,
                )
            })
        })
        .collect();

    let bounds_and_powers: vec::Vec<i64> = [0, i64::MIN, i64::MIN + 1, i64::MAX]
        .into_iter()
        .chain((0..=62).flat_map(|k| [1 << k, -(1 << k)]))
        .collect();
    let mut rng = StdRng::seed_from_u64(0);
    let random = (0..10_000).map(|_| rng.random::<i64>());
    let cases = bundles
        .iter()
        .flat_map(|bundle| bounds_and_powers.iter().map(move |&value| (bundle, value)))
        .chain(bundles.iter().cycle().zip(random));

    for (bundle, value_balance) in cases {
        let bundle = bundle
            .clone()
            .try_map_value_balance(|_| Ok::<_, ()>(value_balance))
            .unwrap();
        let expected = (bundle
            .actions()
            .iter()
            .map(|a| a.cv_net())
            .sum::<ValueCommitment>()
            - ValueCommitment::derive(
                ValueSum::from_raw(value_balance),
                ValueCommitTrapdoor::zero(),
            ))
        .into_bvk();

        assert_eq!(
            bundle.binding_validating_key(),
            expected,
            "value balance {value_balance}"
        );
    }
}
