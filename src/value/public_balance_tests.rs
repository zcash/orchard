use super::{testing::arb_trapdoor, ValueCommitTrapdoor, ValueCommitment, ValueSum};
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
        let derived = ValueCommitment::derive(value, rcv.clone()).to_bytes();
        let expected = original_derive(value, rcv).to_bytes();
        prop_assert_eq!(derived, expected);
    }
}
