use super::{CellValue, EccConfig, EccScalarFixedShort};
use crate::constants::{self, util};
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_scalar_fixed_short: Expression<F>,
    s: Expression<F>,
) {
    // Check that s \in {1, -1}
    meta.create_gate("check sign", |_| {
        q_scalar_fixed_short
            * (s.clone() + Expression::Constant(F::one()))
            * (s - Expression::Constant(F::one()))
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    value: Option<C::Scalar>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
) -> Result<EccScalarFixedShort<C>, Error> {
    // Compute the scalar's sign and magnitude
    let sign = value.map(|value| {
        // t = (p - 1)/2
        let t = (C::Scalar::zero() - C::Scalar::one()) * C::Scalar::TWO_INV;
        if value > t {
            -C::Scalar::one()
        } else {
            C::Scalar::one()
        }
    });
    let magnitude = sign.zip(value).map(|(sign, value)| sign * value);

    // Decompose magnitude into windows
    let bits: Option<Vec<u8>> = magnitude.map(|magnitude| {
        util::decompose_scalar_fixed::<C>(
            magnitude,
            constants::L_VALUE,
            constants::FIXED_BASE_WINDOW_SIZE,
        )
    });

    // Assign and store the magnitude decomposition
    let mut k_bits: Vec<CellValue<C::Base>> = Vec::new();

    if let Some(bits) = bits {
        for (idx, window) in bits.iter().enumerate() {
            // Enable `q_scalar_fixed` selector
            config.q_scalar_fixed.enable(region, offset + idx)?;

            let window = C::Base::from_u64(*window as u64);
            let k_var = region.assign_advice(
                || format!("k[{:?}]", offset + idx),
                config.bits,
                offset + idx,
                || Ok(window),
            )?;
            k_bits.push(CellValue::new(k_var, Some(window)));
        }
    }

    // Assign the sign and enable `q_scalar_fixed_short`
    let sign = sign.map(|sign| {
        assert!(sign == C::Scalar::one() || sign == -C::Scalar::one());
        if sign == C::Scalar::one() {
            C::Base::one()
        } else {
            -C::Base::one()
        }
    });
    let sign_cell = region.assign_advice(
        || "sign",
        config.bits,
        offset + k_bits.len(),
        || sign.ok_or(Error::SynthesisError),
    )?;
    config
        .q_scalar_fixed_short
        .enable(region, offset + k_bits.len())?;

    Ok(EccScalarFixedShort {
        magnitude,
        sign: CellValue::<C::Base>::new(sign_cell, sign),
        k_bits,
    })
}
