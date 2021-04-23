use super::{CellValue, EccConfig, EccScalarVar};

use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::Region,
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_scalar_var: Expression<F>,
    k: Expression<F>,
) {
    meta.create_gate("witness point", |_| {
        // Check that k \in {0, 1}
        q_scalar_var * (k.clone()) * (Expression::Constant(F::one()) - k)
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    value: Option<C::Scalar>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
) -> Result<EccScalarVar<C>, Error> {
    // The scalar field `F_q = 2^254 + t_q`
    // FIXME: Derive this from constants in `Fq` module
    let t_q = 45560315531506369815346746415080538113;

    // We will witness `k = scalar + t_q`
    // `k` is decomposed bitwise in-circuit for our double-and-add algorithm.
    let k = value.map(|value| value + C::Scalar::from_u128(t_q));

    // `k` is decomposed bitwise (big-endian) into `[k_n, ..., k_0]`, where
    // each `k_i` is a bit and `scalar = k_n * 2^n + ... + k_1 * 2 + k_0`.
    let bits: Option<Vec<bool>> = k.map(|k| {
        let mut bits: Vec<bool> = k
            .to_le_bits()
            .into_iter()
            .take(C::Scalar::NUM_BITS as usize)
            .collect();
        bits.reverse();
        bits
    });

    // Store the scalar decomposition
    let mut k_bits: Vec<CellValue<C::Base>> = Vec::new();

    if let Some(bits) = bits {
        for (idx, bit) in bits.iter().enumerate() {
            // Enable `q_scalar_var` selector
            config.q_scalar_var.enable(region, idx + offset)?;

            let bit = C::Base::from_u64(*bit as u64);
            let k_var = region.assign_advice(
                || format!("k[{:?}]", idx),
                config.bits,
                idx + offset,
                || Ok(bit),
            )?;
            k_bits.push(CellValue::new(k_var, Some(bit)));
        }
    }

    Ok(EccScalarVar { value, k_bits })
}
