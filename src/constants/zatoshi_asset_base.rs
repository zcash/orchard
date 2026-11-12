//! Asset base of the zatoshi asset.
//!
//! Every protocol version other than OrchardZSA uses this asset base for all of its notes.

use pasta_curves::{arithmetic::CurveAffine, pallas};

/// Constant used as the asset base of the zatoshi asset.
pub(crate) const ZATOSHI_ASSET_BASE: (pallas::Base, pallas::Base) = (
    pallas::Base::from_raw([
        0x2aa7_bd6e_3af9_4367,
        0xfe04_a37f_2b5a_7c8c,
        0xf7a8_6a70_4f9b_b232,
        0x2f70_597a_8e3d_0f42,
    ]),
    pallas::Base::from_raw([
        0xa413_c47e_af5a_f28e,
        0x1d9e_a766_a7ff_e3db,
        0x1e91_7f63_136d_6c42,
        0x2d0e_5169_3119_19af,
    ]),
);

pub(crate) fn zatoshi_asset_base() -> pallas::Affine {
    pallas::Affine::from_xy(ZATOSHI_ASSET_BASE.0, ZATOSHI_ASSET_BASE.1).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::fixed_bases::{
        VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_V_BYTES,
    };
    use group::Curve;
    use pasta_curves::arithmetic::CurveAffine;
    use pasta_curves::arithmetic::CurveExt;

    #[test]
    fn zatoshi_asset_base() {
        let expected_zatoshi_asset_base = pallas::Point::hash_to_curve(
            VALUE_COMMITMENT_PERSONALIZATION,
        )(&VALUE_COMMITMENT_V_BYTES)
        .to_affine()
        .coordinates()
        .unwrap();

        assert_eq!(*expected_zatoshi_asset_base.x(), ZATOSHI_ASSET_BASE.0);
        assert_eq!(*expected_zatoshi_asset_base.y(), ZATOSHI_ASSET_BASE.1);
    }
}
