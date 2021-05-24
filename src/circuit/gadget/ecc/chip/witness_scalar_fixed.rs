use super::{CellValue, EccConfig, EccScalarFixed, EccScalarFixedShort};
use crate::constants::{self, util};
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

mod full_width;
mod short;

pub(super) struct Config {
    q_scalar_fixed: Selector,
    q_scalar_fixed_short: Selector,
    // k-bit decomposition of scalar.
    k_s: Column<Advice>,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_scalar_fixed: ecc_config.q_scalar_fixed,
            q_scalar_fixed_short: ecc_config.q_scalar_fixed_short,
            k_s: ecc_config.bits,
        }
    }
}

impl Config {
    pub(super) fn create_gate<C: CurveAffine>(&self, meta: &mut ConstraintSystem<C::Base>) {
        // Range check gate applies to both full-width and short scalars
        self.range_check_gate(meta);

        // Gate for short scalar
        let short_config: short::Config<C> = self.into();
        short_config.sign_check_gate(meta);
    }

    pub(super) fn assign_region_full<C: CurveAffine>(
        &self,
        value: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccScalarFixed<C>, Error> {
        let full_width_config: full_width::Config<C> = self.into();
        full_width_config.assign_region(value, offset, region)
    }

    pub(super) fn assign_region_short<C: CurveAffine>(
        &self,
        value: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccScalarFixedShort<C>, Error> {
        let short_config: short::Config<C> = self.into();
        short_config.assign_region(value, offset, region)
    }

    fn range_check_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        // Check that `k` is within the allowed window size
        meta.create_gate("witness scalar fixed", |meta| {
            let q_scalar_fixed = meta.query_selector(self.q_scalar_fixed, Rotation::cur());
            let k = meta.query_advice(self.k_s, Rotation::cur());

            let range_check = (0..constants::H).fold(Expression::Constant(F::one()), |acc, i| {
                acc * (k.clone() - Expression::Constant(F::from_u64(i as u64)))
            });
            q_scalar_fixed * range_check
        });
    }
}

trait WitnessScalarFixed<C: CurveAffine> {
    const SCALAR_NUM_BITS: usize;
    const NUM_WINDOWS: usize;
    type Scalar: Clone + std::fmt::Debug;

    fn q_scalar_fixed(&self) -> Selector;
    fn k(&self) -> Column<Advice>;

    fn assign_region(
        &self,
        value: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<Self::Scalar, Error>;

    fn decompose_scalar_fixed(
        &self,
        value: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<Vec<CellValue<C::Base>>, Error> {
        // Enable `q_scalar_fixed` selector
        for idx in 0..Self::NUM_WINDOWS {
            self.q_scalar_fixed().enable(region, offset + idx)?;
        }

        // Decompose scalar into windows
        let bits: Option<Vec<u8>> = value.map(|value| {
            util::decompose_scalar_fixed::<C>(
                value,
                Self::SCALAR_NUM_BITS,
                constants::FIXED_BASE_WINDOW_SIZE,
            )
        });

        // Store the scalar decomposition
        let mut k_bits: Vec<CellValue<C::Base>> = Vec::new();

        if let Some(bits) = bits {
            for (idx, window) in bits.iter().enumerate() {
                let window = C::Base::from_u64(*window as u64);
                let k_var = region.assign_advice(
                    || format!("k[{:?}]", offset + idx),
                    self.k(),
                    offset + idx,
                    || Ok(window),
                )?;
                k_bits.push(CellValue::new(k_var, Some(window)));
            }
        }

        Ok(k_bits)
    }
}
