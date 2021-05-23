use super::super::{add_incomplete, witness_point, EccPoint, EccScalarFixed, OrchardFixedBase};
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
    #[allow(non_snake_case)]
    pub(super) fn assign_region(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        scalar: &EccScalarFixed<C>,
        base: &OrchardFixedBase<C>,
    ) -> Result<EccPoint<C::Base>, Error> {
        let (acc, mul_b) =
            self.assign_region_inner(region, offset, &scalar.into(), &base.into())?;

        // Add to the accumulator and return the final result as `[scalar]B`.
        self.add_incomplete_config.assign_region(
            &mul_b,
            &acc,
            offset + constants::NUM_WINDOWS - 1,
            region,
        )
    }
}
