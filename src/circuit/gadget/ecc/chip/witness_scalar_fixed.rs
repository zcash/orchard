use super::{CellValue, EccChip, EccScalarFixed};
use crate::constants::util;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::{Chip, Region},
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<C: CurveAffine>(
    meta: &mut ConstraintSystem<C::Base>,
    number_base: usize,
    q_scalar_fixed: Expression<C::Base>,
    k: Expression<C::Base>,
) {
    meta.create_gate("witness scalar fixed", |_| {
        // Check that 0 <= k <= 8
        let range_check = (0..(number_base + 1))
            .fold(Expression::Constant(C::Base::one()), |acc, i| {
                acc * (k.clone() - Expression::Constant(C::Base::from_u64(i as u64)))
            });
        q_scalar_fixed * range_check
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    value: Option<C::Scalar>,
    scalar_num_bits: usize,
    offset: usize,
    region: &mut Region<'_, EccChip<C>>,
    config: <EccChip<C> as Chip>::Config,
) -> Result<EccScalarFixed<C>, Error> {
    // Decompose scalar into three-bit windows
    let bits: Option<Vec<u8>> = value.map(|value| {
        util::decompose_scalar_fixed::<C>(value, scalar_num_bits, config.window_width)
    });

    // Store the scalar decomposition
    let mut k_bits: Vec<CellValue<C::Base>> = Vec::new();

    if let Some(bits) = bits {
        for (idx, three_bits) in bits.iter().enumerate() {
            // Enable `q_scalar_fixed` selector
            config.q_scalar_fixed.enable(region, offset + idx)?;

            let three_bits = C::Base::from_u64(*three_bits as u64);
            let k_var = region.assign_advice(
                || format!("k[{:?}]", offset + idx),
                config.bits,
                offset + idx,
                || Ok(three_bits),
            )?;
            k_bits.push(CellValue::new(k_var, Some(three_bits)));
        }
    }

    Ok(EccScalarFixed { value, k_bits })
}
