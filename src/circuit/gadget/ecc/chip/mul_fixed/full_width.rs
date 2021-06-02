use super::super::{EccConfig, EccPoint, EccScalarFixed, OrchardFixedBasesFull};

use halo2::{arithmetic::CurveAffine, circuit::Region, plonk::Error};

pub struct Config<C: CurveAffine, const NUM_WINDOWS: usize>(super::Config<C, NUM_WINDOWS>);

impl<C: CurveAffine, const NUM_WINDOWS: usize> From<&EccConfig> for Config<C, NUM_WINDOWS> {
    fn from(config: &EccConfig) -> Self {
        Self(config.into())
    }
}

impl<C: CurveAffine, const NUM_WINDOWS: usize> std::ops::Deref for Config<C, NUM_WINDOWS> {
    type Target = super::Config<C, NUM_WINDOWS>;

    fn deref(&self) -> &super::Config<C, NUM_WINDOWS> {
        &self.0
    }
}

impl<C: CurveAffine, const NUM_WINDOWS: usize> Config<C, NUM_WINDOWS> {
    pub fn assign_region(
        &self,
        scalar: &EccScalarFixed<C>,
        base: OrchardFixedBasesFull<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        let (acc, mul_b) =
            (*self).assign_region_inner(region, offset, &scalar.into(), base.into())?;

        // Add to the accumulator and return the final result as `[scalar]B`.
        let result = self
            .add_config
            .assign_region(&mul_b, &acc, offset + NUM_WINDOWS, region)?;

        #[cfg(test)]
        // Check that the correct multiple is obtained.
        {
            use group::Curve;
            use halo2::arithmetic::FieldExt;

            let base: super::OrchardFixedBases<C> = base.into();
            let scalar = scalar
                .value
                .map(|scalar| C::Scalar::from_bytes(&scalar.to_bytes()).unwrap());
            let real_mul = scalar.map(|scalar| base.generator() * scalar);
            let result = result.point();

            assert_eq!(real_mul.unwrap().to_affine(), result.unwrap());
        }

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use ff::Field;
    use halo2::{
        arithmetic::{CurveAffine, FieldExt},
        circuit::Layouter,
        plonk::Error,
    };

    use crate::circuit::gadget::ecc::{
        chip::{EccChip, OrchardFixedBasesFull},
        FixedPoint, ScalarFixed,
    };
    use crate::constants;
    use std::marker::PhantomData;

    pub fn test_mul_fixed<C: CurveAffine>(
        chip: EccChip<C>,
        mut layouter: impl Layouter<C::Base>,
    ) -> Result<(), Error> {
        // commit_ivk_r
        let commit_ivk_r = OrchardFixedBasesFull::CommitIvkR(PhantomData);
        let commit_ivk_r = FixedPoint::from_inner(chip.clone(), commit_ivk_r);
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "commit_ivk_r"),
            commit_ivk_r,
        )?;

        // note_commit_r
        let note_commit_r = OrchardFixedBasesFull::NoteCommitR(PhantomData);
        let note_commit_r = FixedPoint::from_inner(chip.clone(), note_commit_r);
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "note_commit_r"),
            note_commit_r,
        )?;

        // nullifier_k
        let nullifier_k = OrchardFixedBasesFull::NullifierK(PhantomData);
        let nullifier_k = FixedPoint::from_inner(chip.clone(), nullifier_k);
        test_single_base(
            chip.clone(),
            layouter.namespace(|| "nullifier_k"),
            nullifier_k,
        )?;

        // value_commit_r
        let value_commit_r = OrchardFixedBasesFull::ValueCommitR(PhantomData);
        let value_commit_r = FixedPoint::from_inner(chip.clone(), value_commit_r);
        test_single_base(
            chip,
            layouter.namespace(|| "value_commit_r"),
            value_commit_r,
        )?;

        Ok(())
    }

    fn test_single_base<C: CurveAffine>(
        chip: EccChip<C>,
        mut layouter: impl Layouter<C::Base>,
        base: FixedPoint<C, EccChip<C>>,
    ) -> Result<(), Error> {
        // [a]B
        {
            let scalar_fixed = C::Scalar::rand();

            let scalar_fixed = ScalarFixed::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixed"),
                Some(scalar_fixed),
            )?;

            base.mul(layouter.namespace(|| "mul"), &scalar_fixed)?;
        }

        // There is a single canonical sequence of window values for which a doubling occurs on the last step:
        // 1333333333333333333333333333333333333333333333333333333333333333333333333333333333334 in octal.
        // (There is another *non-canonical* sequence
        // 5333333333333333333333333333333333333333332711161673731021062440252244051273333333333 in octal.)
        {
            let h = C::ScalarExt::from_u64(constants::H as u64);
            let scalar_fixed = "1333333333333333333333333333333333333333333333333333333333333333333333333333333333334"
                        .chars()
                        .fold(C::ScalarExt::zero(), |acc, c| {
                            acc * h + C::ScalarExt::from_u64(c.to_digit(8).unwrap().into())
                        });

            let scalar_fixed = ScalarFixed::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixed"),
                Some(scalar_fixed),
            )?;

            base.mul(layouter.namespace(|| "mul with double"), &scalar_fixed)?;
        }

        // [0]B should return (0,0) since it uses complete addition
        // on the last step.
        {
            let scalar_fixed = C::Scalar::zero();
            let scalar_fixed = ScalarFixed::new(
                chip,
                layouter.namespace(|| "ScalarFixed"),
                Some(scalar_fixed),
            )?;
            base.mul(layouter.namespace(|| "mul by zero"), &scalar_fixed)?;
        }

        Ok(())
    }
}
