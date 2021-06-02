use super::super::{CellValue, EccConfig, EccScalarFixedShort};
use crate::constants::{L_VALUE, NUM_WINDOWS_SHORT};
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

pub struct Config<C: CurveAffine> {
    q_scalar_fixed_short: Selector,
    witness_scalar_fixed_config: super::Config<C>,
}

impl<C: CurveAffine> From<&EccConfig> for Config<C> {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_scalar_fixed_short: ecc_config.q_scalar_fixed_short,
            witness_scalar_fixed_config: ecc_config.into(),
        }
    }
}

impl<C: CurveAffine> std::ops::Deref for Config<C> {
    type Target = super::Config<C>;

    fn deref(&self) -> &super::Config<C> {
        &self.witness_scalar_fixed_config
    }
}

impl<C: CurveAffine> Config<C> {
    pub fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        // Check that sign is either 1 or -1.
        meta.create_gate("check sign", |meta| {
            let q_scalar_fixed_short =
                meta.query_selector(self.q_scalar_fixed_short, Rotation::cur());
            let sign = meta.query_advice(self.window, Rotation::cur());

            vec![
                q_scalar_fixed_short
                    * (sign.clone() + Expression::Constant(C::Base::one()))
                    * (sign - Expression::Constant(C::Base::one())),
            ]
        });
    }
}

impl<C: CurveAffine> Config<C> {
    pub fn assign_region(
        &self,
        value: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccScalarFixedShort<C>, Error> {
        // Enable `q_scalar_fixed_short`
        self.q_scalar_fixed_short
            .enable(region, offset + NUM_WINDOWS_SHORT)?;

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

        // Decompose magnitude into `k`-bit windows
        let windows =
            self.decompose_scalar_fixed::<NUM_WINDOWS_SHORT, L_VALUE>(magnitude, offset, region)?;

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
            self.window,
            NUM_WINDOWS_SHORT,
            || sign.ok_or(Error::SynthesisError),
        )?;

        Ok(EccScalarFixedShort {
            magnitude,
            sign: CellValue::<C::Base>::new(sign_cell, sign),
            windows,
        })
    }
}
