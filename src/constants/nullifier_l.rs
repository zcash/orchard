//! $\mathcal{L}^{\mathsf{Orchard}}$ constant.
//!
//! This constant is used to evaluate the nullifier of a split note (ZIP 226).

use pasta_curves::{arithmetic::CurveAffine, pallas};

/// Constant used as $\mathcal{L}^{\mathsf{Orchard}}$ in DeriveNullifier.
pub const NULLIFIER_L: (pallas::Base, pallas::Base) = (
    pallas::Base::from_raw([
        0x7956_0ba9_2372_5d67,
        0xcb79_efc2_91c1_b0c8,
        0xbbed_d606_d1df_737e,
        0x032e_6e8c_f01d_3ea1,
    ]),
    pallas::Base::from_raw([
        0x8838_95de_f68a_1265,
        0xd861_4d8d_ab43_e39e,
        0xf9e6_ebb2_b7b5_6640,
        0x093c_af7e_c368_376e,
    ]),
);

pub fn nullifier_l() -> pallas::Affine {
    pallas::Affine::from_xy(NULLIFIER_L.0, NULLIFIER_L.1).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use group::Curve;
    use pasta_curves::arithmetic::CurveAffine;
    use pasta_curves::arithmetic::CurveExt;

    #[test]
    fn nullifier_l() {
        let expected_nullifier_l = pallas::Point::hash_to_curve("z.cash:Orchard")(b"L")
            .to_affine()
            .coordinates()
            .unwrap();

        assert_eq!(*expected_nullifier_l.x(), NULLIFIER_L.0);
        assert_eq!(*expected_nullifier_l.y(), NULLIFIER_L.1);
    }
}
