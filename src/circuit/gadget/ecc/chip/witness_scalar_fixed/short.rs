use super::super::{CellValue, EccScalarFixedShort};
use crate::constants;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

pub(super) struct Config<C: CurveAffine> {
    q_scalar_fixed: Selector,
    q_scalar_fixed_short: Selector,
    // k-bit decomposition of scalar. Also used to witness the sign `s`
    // in the last row.
    k_s: Column<Advice>,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> From<&super::Config> for Config<C> {
    fn from(config: &super::Config) -> Self {
        Self {
            q_scalar_fixed: config.q_scalar_fixed,
            q_scalar_fixed_short: config.q_scalar_fixed_short,
            k_s: config.k_s,
            _marker: PhantomData,
        }
    }
}

impl<C: CurveAffine> Config<C> {
    pub(super) fn sign_check_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        // Check that sign s \in {1, -1}
        meta.create_gate("check sign", |meta| {
            let q_scalar_fixed_short =
                meta.query_selector(self.q_scalar_fixed_short, Rotation::cur());
            let s = meta.query_advice(self.k_s, Rotation::cur());

            q_scalar_fixed_short
                * (s.clone() + Expression::Constant(C::Base::one()))
                * (s - Expression::Constant(C::Base::one()))
        });
    }
}

impl<C: CurveAffine> super::WitnessScalarFixed<C> for Config<C> {
    const SCALAR_NUM_BITS: usize = constants::L_VALUE as usize;
    type Scalar = EccScalarFixedShort<C>;

    fn q_scalar_fixed(&self) -> Selector {
        self.q_scalar_fixed
    }
    fn k(&self) -> Column<Advice> {
        self.k_s
    }

    fn assign_region(
        &self,
        value: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
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

        // Decompose magnitude into `k`-bit windows
        let k_bits = self.decompose_scalar_fixed(magnitude, offset, region)?;

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
            self.k_s,
            offset + k_bits.len(),
            || sign.ok_or(Error::SynthesisError),
        )?;
        self.q_scalar_fixed_short
            .enable(region, offset + k_bits.len())?;

        Ok(EccScalarFixedShort {
            magnitude,
            sign: CellValue::<C::Base>::new(sign_cell, sign),
            k_bits,
        })
    }
}
