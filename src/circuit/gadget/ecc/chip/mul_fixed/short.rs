use super::super::{util, CellValue, EccPoint, EccScalarFixedShort};
use crate::constants::{self, ValueCommitV};

use halo2::{
    arithmetic::{CurveAffine, Field},
    circuit::Region,
    plonk::{ConstraintSystem, Error},
    poly::Rotation,
};

pub struct Config<C: CurveAffine>(super::Config<C>);

impl<C: CurveAffine> From<&super::Config<C>> for Config<C> {
    fn from(config: &super::Config<C>) -> Self {
        Self(config.clone())
    }
}

impl<C: CurveAffine> std::ops::Deref for Config<C> {
    type Target = super::Config<C>;

    fn deref(&self) -> &super::Config<C> {
        &self.0
    }
}

impl<C: CurveAffine> Config<C> {
    // We reuse the constraints in the `mul_fixed` gate so exclude them here.
    // Here, we add some new constraints specific to the short signed case.
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        let q_mul_fixed_short = meta.query_selector(self.q_mul_fixed_short, Rotation::cur());
        let y_p = meta.query_advice(self.y_p, Rotation::cur());
        let y_a = meta.query_advice(self.add_config.y_qr, Rotation::cur());

        // `(x_a, y_a)` is the result of `[m]B`, where `m` is the magnitude.
        // We conditionally negate this result using `y_p = y_a * s`, where `s` is the sign.

        // Check that the final `y_p = y_a` or `y_p = -y_a`
        meta.create_gate("check y", |_| {
            q_mul_fixed_short.clone() * (y_p.clone() - y_a.clone()) * (y_p.clone() + y_a.clone())
        });

        // Check that s * y_p = y_a
        meta.create_gate("check negation", |meta| {
            let s = meta.query_advice(self.k, Rotation::cur());
            q_mul_fixed_short * (s * y_p - y_a)
        });
    }

    pub(super) fn assign_region(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        scalar: &EccScalarFixedShort<C>,
        base: &ValueCommitV<C>,
    ) -> Result<EccPoint<C>, Error> {
        let (acc, mul_b) = self.assign_region_inner::<{ constants::NUM_WINDOWS_SHORT }>(
            region,
            offset,
            &scalar.into(),
            base.clone().into(),
        )?;

        // Add to the cumulative sum to get `[magnitude]B`.
        let magnitude_mul = self.add_config.assign_region(
            &mul_b,
            &acc,
            offset + constants::NUM_WINDOWS_SHORT,
            region,
        )?;

        // Increase offset by 1 after complete addition
        let offset = offset + 1;

        // Assign sign to `bits` column
        let sign = util::assign_and_constrain(
            region,
            || "sign",
            self.k,
            offset + constants::NUM_WINDOWS_SHORT,
            &scalar.sign,
            &self.perm,
        )?;

        // Conditionally negate `y`-coordinate
        let y_val = if let Some(sign) = sign.value {
            if sign == -C::Base::one() {
                magnitude_mul.y.value.map(|y: C::Base| -y)
            } else {
                magnitude_mul.y.value
            }
        } else {
            None
        };

        // Enable mul_fixed_short selector on final row
        self.q_mul_fixed_short
            .enable(region, offset + constants::NUM_WINDOWS_SHORT)?;

        // Assign final `x, y` to `x_p, y_p` columns and return final point
        let x_val = magnitude_mul.x.value;
        let x_var = region.assign_advice(
            || "x_var",
            self.x_p,
            offset + constants::NUM_WINDOWS_SHORT,
            || x_val.ok_or(Error::SynthesisError),
        )?;
        let y_var = region.assign_advice(
            || "y_var",
            self.y_p,
            offset + constants::NUM_WINDOWS_SHORT,
            || y_val.ok_or(Error::SynthesisError),
        )?;

        let result = EccPoint::<C> {
            x: CellValue::new(x_var, x_val),
            y: CellValue::new(y_var, y_val),
        };

        #[cfg(test)]
        // Check that the correct multiple is obtained.
        {
            use group::Curve;

            let base: super::OrchardFixedBases<C> = base.clone().into();

            let scalar = scalar
                .magnitude
                .zip(scalar.sign.value)
                .map(|(magnitude, sign)| {
                    let sign = if sign == C::Base::one() {
                        C::Scalar::one()
                    } else if sign == -C::Base::one() {
                        -C::Scalar::one()
                    } else {
                        panic!("Sign should be 1 or -1.")
                    };
                    magnitude * sign
                });
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

    use crate::circuit::gadget::ecc::{chip::EccChip, FixedPointShort, ScalarFixedShort};
    use crate::constants::load::ValueCommitV;

    pub fn test_mul_fixed_short<C: CurveAffine>(
        chip: EccChip<C>,
        mut layouter: impl Layouter<C::Base>,
    ) -> Result<(), Error> {
        // value_commit_v
        let value_commit_v = ValueCommitV::<C>::get();
        let value_commit_v = FixedPointShort::from_inner(chip.clone(), value_commit_v);

        // [0]B should return (0,0) since it uses complete addition
        // on the last step.
        {
            let scalar_fixed = C::Scalar::zero();
            let scalar_fixed = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul"), &scalar_fixed)?;
        }

        // Random [a]B
        {
            let scalar_fixed_short = C::Scalar::from_u64(rand::random::<u64>());
            let mut sign = C::Scalar::one();
            if rand::random::<bool>() {
                sign = -sign;
            }
            let scalar_fixed_short = sign * scalar_fixed_short;

            let scalar_fixed_short = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed_short),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul fixed"), &scalar_fixed_short)?;
        }

        // [2^64 - 1]B
        {
            let scalar_fixed_short = C::Scalar::from_u64(0xFFFF_FFFF_FFFF_FFFFu64);

            let scalar_fixed_short = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed_short),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul fixed"), &scalar_fixed_short)?;
        }

        // [-(2^64 - 1)]B
        {
            let scalar_fixed_short = -C::Scalar::from_u64(0xFFFF_FFFF_FFFF_FFFFu64);

            let scalar_fixed_short = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed_short),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul fixed"), &scalar_fixed_short)?;
        }

        // There is a single canonical sequence of window values for which a doubling occurs on the last step:
        // 1333333333333333333334 in octal.
        // [0xB6DB_6DB6_DB6D_B6DC] B
        {
            let scalar_fixed_short = C::Scalar::from_u64(0xB6DB_6DB6_DB6D_B6DCu64);

            let scalar_fixed_short = ScalarFixedShort::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixedShort"),
                Some(scalar_fixed_short),
            )?;
            value_commit_v.mul(layouter.namespace(|| "mul fixed"), &scalar_fixed_short)?;
        }

        Ok(())
    }
}
