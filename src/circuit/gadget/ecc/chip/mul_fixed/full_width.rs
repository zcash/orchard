use super::super::{
    add, add_incomplete, witness_point, EccPoint, EccScalarFixed, OrchardFixedBase,
};
use super::MulFixed;

use crate::constants;
use std::marker::PhantomData;

use halo2::{
    arithmetic::CurveAffine,
    circuit::Region,
    plonk::{Advice, Column, Error, Fixed, Permutation, Selector},
};

pub struct Config<C: CurveAffine> {
    q_mul_fixed: Selector,
    // The fixed Lagrange interpolation coefficients for `x_p`.
    lagrange_coeffs: [Column<Fixed>; constants::H],
    // The fixed `z` for each window such that `y + z = u^2`.
    fixed_z: Column<Fixed>,
    // k-bit decomposition of an `n-1`-bit scalar:
    // a = a_0 + 2^k(a_1) + 2^{2k}(a_2) + ... + 2^{(n-1)k}(a_{n-1})
    k: Column<Advice>,
    // x-coordinate of the multiple of the fixed base at the current window.
    x_p: Column<Advice>,
    // y-coordinate of the multiple of the fixed base at the current window.
    y_p: Column<Advice>,
    // An integer `u` for the current window, s.t. `y + z = u^2`.
    u: Column<Advice>,
    // Permutation
    perm: Permutation,
    // Configuration for `add`
    add_config: add::Config,
    // Configuration for `add_incomplete`
    add_incomplete_config: add_incomplete::Config,
    // Configuration for `witness_point`
    witness_point_config: witness_point::Config,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> From<&super::Config> for Config<C> {
    fn from(config: &super::Config) -> Self {
        Self {
            q_mul_fixed: config.q_mul_fixed,
            lagrange_coeffs: config.lagrange_coeffs,
            fixed_z: config.fixed_z,
            k: config.k,
            x_p: config.x_p,
            y_p: config.y_p,
            u: config.u,
            perm: config.perm.clone(),
            add_config: config.add_config.clone(),
            add_incomplete_config: config.add_incomplete_config.clone(),
            witness_point_config: config.witness_point_config.clone(),
            _marker: PhantomData,
        }
    }
}

impl<C: CurveAffine> MulFixed<C> for Config<C> {
    const NUM_WINDOWS: usize = constants::NUM_WINDOWS;

    fn q_mul_fixed(&self) -> Selector {
        self.q_mul_fixed
    }
    fn lagrange_coeffs(&self) -> [Column<Fixed>; constants::H] {
        self.lagrange_coeffs
    }
    fn fixed_z(&self) -> Column<Fixed> {
        self.fixed_z
    }
    fn k(&self) -> Column<Advice> {
        self.k
    }
    fn x_p(&self) -> Column<Advice> {
        self.x_p
    }
    fn y_p(&self) -> Column<Advice> {
        self.y_p
    }
    fn u(&self) -> Column<Advice> {
        self.u
    }
    fn perm(&self) -> &Permutation {
        &self.perm
    }
    fn witness_point_config(&self) -> &witness_point::Config {
        &self.witness_point_config
    }
    fn add_config(&self) -> &add::Config {
        &self.add_config
    }
    fn add_incomplete_config(&self) -> &add_incomplete::Config {
        &self.add_incomplete_config
    }
}

impl<C: CurveAffine> Config<C> {
    pub(super) fn assign_region(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        scalar: &EccScalarFixed<C>,
        base: &OrchardFixedBase<C>,
    ) -> Result<EccPoint<C>, Error> {
        let (acc, mul_b) =
            self.assign_region_inner(region, offset, &scalar.into(), &base.into())?;

        // Add to the accumulator and return the final result as `[scalar]B`.
        let result =
            self.add_config
                .assign_region(&mul_b, &acc, offset + constants::NUM_WINDOWS, region)?;

        #[cfg(test)]
        // Check that the correct multiple is obtained.
        {
            use super::super::OrchardFixedBases;
            use group::Curve;
            use halo2::arithmetic::FieldExt;

            let base = match base.base {
                OrchardFixedBases::CommitIvkR(base) => base.0.value(),
                OrchardFixedBases::NoteCommitR(base) => base.0.value(),
                OrchardFixedBases::NullifierK(base) => base.0.value(),
                OrchardFixedBases::ValueCommitR(base) => base.0.value(),
            };
            let scalar = scalar
                .value
                .map(|scalar| C::Scalar::from_bytes(&scalar.to_bytes()).unwrap());
            let real_mul = scalar.map(|scalar| base * scalar);
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

    use crate::circuit::gadget::ecc::{EccInstructions, FixedPoint, ScalarFixed};
    use crate::constants;

    pub fn test_mul_fixed<
        C: CurveAffine,
        EccChip: EccInstructions<C> + Clone + Eq + std::fmt::Debug,
    >(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        nullifier_k: FixedPoint<C, EccChip>,
    ) -> Result<(), Error> {
        // [a]B
        {
            let scalar_fixed = C::Scalar::rand();

            let scalar_fixed = ScalarFixed::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixed"),
                Some(scalar_fixed),
            )?;

            nullifier_k.mul(layouter.namespace(|| "mul"), &scalar_fixed)?;
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

            nullifier_k.mul(layouter.namespace(|| "mul with double"), &scalar_fixed)?;
        }

        // [0]B should return (0,0) since it uses complete addition
        // on the last step.
        {
            let scalar_fixed = C::Scalar::zero();
            let scalar_fixed = ScalarFixed::new(
                chip.clone(),
                layouter.namespace(|| "ScalarFixed"),
                Some(scalar_fixed),
            )?;
            nullifier_k.mul(layouter.namespace(|| "mul by zero"), &scalar_fixed)?;
        }

        Ok(())
    }
}
