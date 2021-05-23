use super::super::EccScalarFixed;
use ff::PrimeField;
use halo2::{
    arithmetic::CurveAffine,
    circuit::Region,
    plonk::{Advice, Column, Error, Selector},
};
use std::marker::PhantomData;

pub(super) struct Config<C: CurveAffine> {
    pub q_scalar_fixed: Selector,
    // k-bit decomposition of scalar
    pub k: Column<Advice>,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> From<&super::Config> for Config<C> {
    fn from(config: &super::Config) -> Self {
        Self {
            q_scalar_fixed: config.q_scalar_fixed,
            k: config.k_s,
            _marker: PhantomData,
        }
    }
}

impl<C: CurveAffine> super::WitnessScalarFixed<C> for Config<C> {
    const SCALAR_NUM_BITS: usize = C::Scalar::NUM_BITS as usize;
    type Scalar = EccScalarFixed<C>;

    fn q_scalar_fixed(&self) -> Selector {
        self.q_scalar_fixed
    }
    fn k(&self) -> Column<Advice> {
        self.k
    }

    fn assign_region(
        &self,
        value: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccScalarFixed<C>, Error> {
        let k_bits = self.decompose_scalar_fixed(value, offset, region)?;

        Ok(EccScalarFixed { value, k_bits })
    }
}
