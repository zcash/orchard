use super::super::{
    add_incomplete, util, witness_point, EccPoint, EccScalarFixedShort, OrchardFixedBaseShort,
};
use super::MulFixed;
use crate::constants;

use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, Field},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

pub struct Config<C: CurveAffine> {
    q_mul_fixed: Selector,
    q_mul_fixed_short: Selector,
    // The fixed Lagrange interpolation coefficients for `x_p`.
    lagrange_coeffs: [Column<Fixed>; constants::H],
    // The fixed `z` for each window such that `y + z = u^2`.
    fixed_z: Column<Fixed>,
    // k-bit decomposition of an `n-1`-bit scalar:
    // a = a_0 + 2^k(a_1) + 2^{2k}(a_2) + ... + 2^{(n-1)k}(a_{n-1})
    k_s: Column<Advice>,
    // x-coordinate of the multiple of the fixed base at the current window.
    x_p: Column<Advice>,
    // y-coordinate of the multiple of the fixed base at the current window.
    y_p: Column<Advice>,
    // y-coordinate of accumulator (only used in the final row).
    y_a: Column<Advice>,
    // An integer `u` for the current window, s.t. `y + z = u^2`.
    u: Column<Advice>,
    // Permutation
    perm: Permutation,
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
            q_mul_fixed_short: config.q_mul_fixed_short,
            lagrange_coeffs: config.lagrange_coeffs,
            fixed_z: config.fixed_z,
            k_s: config.k,
            x_p: config.x_p,
            y_p: config.y_p,
            y_a: config.y_a,
            u: config.u,
            perm: config.perm.clone(),
            add_incomplete_config: config.add_incomplete_config.clone(),
            witness_point_config: config.witness_point_config.clone(),
            _marker: PhantomData,
        }
    }
}

impl<C: CurveAffine> MulFixed<C> for Config<C> {
    const NUM_WINDOWS: usize = constants::NUM_WINDOWS_SHORT;

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
        self.k_s
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
    fn add_incomplete_config(&self) -> &add_incomplete::Config {
        &self.add_incomplete_config
    }
}

impl<C: CurveAffine> Config<C> {
    // We reuse the constraints in the `mul_fixed` gate so exclude them here.
    // Here, we add some new constraints specific to the short signed case.
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        let q_mul_fixed_short = meta.query_selector(self.q_mul_fixed_short, Rotation::cur());
        let y_p = meta.query_advice(self.y_p, Rotation::cur());
        let y_a = meta.query_advice(self.y_a, Rotation::cur());

        // `(x_a, y_a)` is the result of `[m]B`, where `m` is the magnitude.
        // We conditionally negate this result using `y_p = y_a * s`, where `s` is the sign.

        // Check that the final `y_p = y_a` or `y_p = -y_a`
        meta.create_gate("check y", |_| {
            q_mul_fixed_short.clone() * (y_p.clone() - y_a.clone()) * (y_p.clone() + y_a.clone())
        });

        // Check that s * y_p = y_a
        meta.create_gate("check negation", |meta| {
            let s = meta.query_advice(self.k_s, Rotation::cur());
            q_mul_fixed_short * (s * y_p - y_a)
        });
    }

    #[allow(non_snake_case)]
    pub(super) fn assign_region(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        scalar: &EccScalarFixedShort<C>,
        base: &OrchardFixedBaseShort<C>,
    ) -> Result<EccPoint<C::Base>, Error> {
        let (acc, mul_b) =
            self.assign_region_inner(region, offset, &scalar.into(), &base.into())?;

        // Add to the cumulative sum to get `[magnitude]B`.
        let magnitude_mul = self.add_incomplete_config.assign_region(
            &mul_b,
            &acc,
            offset + constants::NUM_WINDOWS_SHORT - 1,
            region,
        )?;

        // Assign sign to `bits` column
        let sign = util::assign_and_constrain(
            region,
            || "sign",
            self.k_s.into(),
            offset + constants::NUM_WINDOWS_SHORT,
            &scalar.sign,
            &self.perm,
        )?;

        // Conditionally negate `y`-coordinate
        let y_val = match sign.value {
            Some(sign) => {
                if sign == -C::Base::one() {
                    magnitude_mul.y.value.map(|y: C::Base| -y)
                } else {
                    magnitude_mul.y.value
                }
            }
            None => None,
        };

        // Enable mul_fixed_short selector on final row
        self.q_mul_fixed_short
            .enable(region, offset + constants::NUM_WINDOWS_SHORT)?;

        // Assign final `x, y` to `x_p, y_p` columns and return final point
        let x_val = magnitude_mul.x.value;
        let mul = x_val
            .zip(y_val)
            .map(|(x, y)| C::from_xy(x, y).unwrap().to_curve());
        self.witness_point_config.assign_region(
            mul.map(|point| point.to_affine()),
            offset + constants::NUM_WINDOWS_SHORT,
            region,
        )
    }
}
