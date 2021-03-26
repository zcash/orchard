use super::{CellValue, EccChip, EccScalarFixedShort};
use crate::constants::{self, util};
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::{Chip, Region},
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<C: CurveAffine>(
    meta: &mut ConstraintSystem<C::Base>,
    q_scalar_fixed_short: Expression<C::Base>,
    s: Expression<C::Base>,
) {
    // Check that s \in {1, -1}
    meta.create_gate("check sign", |_| {
        q_scalar_fixed_short
            * (s.clone() + Expression::Constant(C::Base::one()))
            * (s - Expression::Constant(C::Base::one()))
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    value: Option<C::Scalar>,
    offset: usize,
    region: &mut Region<'_, EccChip<C>>,
    config: <EccChip<C> as Chip>::Config,
) -> Result<EccScalarFixedShort<C>, Error> {
    // Compute the scalar's sign
    let sign = value.map(|value| {
        // (T - 1) / 2
        let t_minus1_over2 = C::Scalar::from_bytes_wide(
            &C::Scalar::T_MINUS1_OVER2
                .iter()
                .map(|u64_val| u64_val.to_le_bytes())
                .enumerate()
                .fold([0u8; 64], |mut u8_array, (u64_idx, u8_8)| {
                    for (u8_idx, u8_val) in u8_8.iter().enumerate() {
                        u8_array[u64_idx * 8 + u8_idx] = *u8_val;
                    }
                    u8_array
                }),
        );
        // T = (p - 1) / 2 = t_minus1_over * 2 + 1
        if value > (t_minus1_over2 * C::Scalar::from_u64(2) + C::Scalar::one()) {
            -C::Base::one()
        } else {
            C::Base::one()
        }
    });

    // Compute the scalar's magnitude
    let magnitude = sign.zip(value).map(|(sign, value)| {
        if sign == -C::Base::one() {
            -value
        } else {
            value
        }
    });

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
        sign: CellValue::new(sign_cell, sign),
        k_bits,
    })
}
