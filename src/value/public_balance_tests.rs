use super::{testing::arb_trapdoor, ValueCommitTrapdoor, ValueCommitment, ValueSum};
use group::{Curve, Group};
use pasta_curves::{arithmetic::CurveAffine, pallas};
use proptest::prelude::*;

// Preserve the original hash-to-curve arithmetic as a differential oracle.
fn original_derive(value: ValueSum, rcv: ValueCommitTrapdoor) -> ValueCommitment {
    use crate::constants::fixed_bases::{
        VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_R_BYTES, VALUE_COMMITMENT_V_BYTES,
    };
    use pasta_curves::{arithmetic::CurveExt, pallas};

    let hasher = pallas::Point::hash_to_curve(VALUE_COMMITMENT_PERSONALIZATION);
    let v = hasher(&VALUE_COMMITMENT_V_BYTES);
    let r = hasher(&VALUE_COMMITMENT_R_BYTES);
    let magnitude = u64::try_from(value.0.abs()).expect("value must be in valid range");
    let scalar = pallas::Scalar::from(magnitude);
    let scalar = if value.0.is_negative() {
        -scalar
    } else {
        scalar
    };
    ValueCommitment(v * scalar + r * rcv.0)
}

// Preserve the previous public-value implementation as an independent oracle.
fn double_and_add(value: ValueSum) -> ValueCommitment {
    let magnitude = u64::try_from(value.0.abs()).expect("value must be in valid range");
    let generator = crate::constants::fixed_bases::value_commit_v::generator();
    let commitment = (0..u64::BITS - magnitude.leading_zeros()).rev().fold(
        pallas::Point::identity(),
        |acc, bit| {
            let acc = acc.double();
            if (magnitude >> bit) & 1 == 1 {
                acc + generator
            } else {
                acc
            }
        },
    );
    ValueCommitment(if value.0.is_negative() {
        -commitment
    } else {
        commitment
    })
}

#[test]
fn public_balance_table_matches_curve_arithmetic() {
    let mut base = pallas::Point::from(crate::constants::fixed_bases::value_commit_v::generator());
    assert_eq!(
        core::mem::size_of_val(&super::commitment_table::TABLE),
        22_528
    );
    for window in &super::commitment_table::TABLE {
        let mut expected = pallas::Point::identity();
        for point in window {
            expected += base;
            assert!(bool::from(point.is_on_curve()));
            assert_eq!(*point, expected.to_affine());
        }
        for _ in 0..6 {
            base = base.double();
        }
    }
}

#[test]
fn public_balance_matches_double_and_add_edges() {
    let check = |magnitude: u64| {
        for value in [i128::from(magnitude), -i128::from(magnitude)] {
            let value = ValueSum(value);
            assert_eq!(
                ValueCommitment::derive_public(value).to_bytes(),
                double_and_add(value).to_bytes(),
                "value {value:?}",
            );
        }
    };
    check(0);
    check(10_000);
    check(u64::MAX);
    // Exercise carries at every bit, including the highest signed-window carry.
    for bit in 0..64 {
        let power = 1u64 << bit;
        check(power - 1);
        check(power);
        check(power + 1);
    }
    // Exercise the sign boundary in every radix-64 digit, with and without carry.
    for window in 0..11 {
        for digit in [31u64, 32, 33, 63] {
            let magnitude = u128::from(digit) << (6 * window);
            if let Ok(magnitude) = u64::try_from(magnitude) {
                check(magnitude);
                check(magnitude + 1);
            }
        }
    }
}

#[test]
fn public_balance_matches_original_arithmetic() {
    use pasta_curves::pallas;

    for value in [
        -(i128::from(u64::MAX)),
        i128::from(i64::MIN),
        -1,
        0,
        1,
        i128::from(i64::MAX),
        i128::from(u64::MAX),
    ] {
        let value = ValueSum(value);
        let original = original_derive(value, ValueCommitTrapdoor::zero());
        assert_eq!(
            ValueCommitment::derive_public(value).to_bytes(),
            original.to_bytes()
        );
        for scalar in [
            pallas::Scalar::zero(),
            pallas::Scalar::one(),
            -pallas::Scalar::one(),
        ] {
            assert_eq!(
                ValueCommitment::derive(value, ValueCommitTrapdoor(scalar)).to_bytes(),
                original_derive(value, ValueCommitTrapdoor(scalar)).to_bytes(),
            );
        }
    }
}

proptest! {
    #[test]
    fn public_balance_matches_original_for_arbitrary_values(
        value in super::testing::arb_value_sum(),
        rcv in arb_trapdoor(),
    ) {
        let public = ValueCommitment::derive_public(value).to_bytes();
        let expected = original_derive(value, ValueCommitTrapdoor::zero()).to_bytes();
        prop_assert_eq!(public, expected);
        prop_assert_eq!(public, double_and_add(value).to_bytes());
        let derived = ValueCommitment::derive(value, rcv.clone()).to_bytes();
        let expected = original_derive(value, rcv).to_bytes();
        prop_assert_eq!(derived, expected);
    }
}
