use super::super::{EccConfig, EccScalarFixed};
use crate::constants::{L_ORCHARD_SCALAR, NUM_WINDOWS};
use halo2::{arithmetic::CurveAffine, circuit::Region, plonk::Error};

pub struct Config<C: CurveAffine>(super::Config<C>);

impl<C: CurveAffine> From<&EccConfig> for Config<C> {
    fn from(ecc_config: &EccConfig) -> Self {
        Self(ecc_config.into())
    }
}

impl<C: CurveAffine> std::ops::Deref for Config<C> {
    type Target = super::Config<C>;

    fn deref(&self) -> &super::Config<C> {
        &self.0
    }
}

impl<C: CurveAffine> Config<C> {
    pub fn assign_region(
        &self,
        value: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccScalarFixed<C>, Error> {
        let windows =
            self.decompose_scalar_fixed::<NUM_WINDOWS, L_ORCHARD_SCALAR>(value, offset, region)?;

        Ok(EccScalarFixed { value, windows })
    }
}
