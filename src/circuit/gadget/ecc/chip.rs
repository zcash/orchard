use super::EccInstructions;
use crate::circuit::gadget::utilities::{copy, CellValue, Var};
use crate::constants::{self, OrchardFixedBasesFull, ValueCommitV};
use arrayvec::ArrayVec;
use ff::Field;
use halo2::{
    arithmetic::CurveAffine,
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
};
use std::marker::PhantomData;

pub(super) mod add;
pub(super) mod add_incomplete;
pub(super) mod mul;
pub(super) mod mul_fixed;
pub(super) mod witness_point;
pub(super) mod witness_scalar_fixed;

/// A curve point represented in affine (x, y) coordinates. Each coordinate is
/// assigned to a cell.
#[derive(Clone, Debug)]
pub struct EccPoint<C: CurveAffine> {
    /// x-coordinate
    pub x: CellValue<C::Base>,
    /// y-coordinate
    pub y: CellValue<C::Base>,
}

impl<C: CurveAffine> EccPoint<C> {
    /// Returns the value of this curve point, if known.
    pub fn point(&self) -> Option<C> {
        match (self.x.value(), self.y.value()) {
            (Some(x), Some(y)) => {
                if x == C::Base::zero() && y == C::Base::zero() {
                    Some(C::identity())
                } else {
                    Some(C::from_xy(x, y).unwrap())
                }
            }
            _ => None,
        }
    }
}

/// Configuration for the ECC chip
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(non_snake_case)]
pub struct EccConfig {
    /// Advice columns needed by instructions in the ECC chip.
    pub advices: [Column<Advice>; 10],

    /// Coefficients of interpolation polynomials for x-coordinates (used in fixed-base scalar multiplication)
    pub lagrange_coeffs: [Column<Fixed>; constants::H],
    /// Fixed z such that y + z = u^2 some square, and -y + z is a non-square. (Used in fixed-base scalar multiplication)
    pub fixed_z: Column<Fixed>,

    /// Incomplete addition
    pub q_add_incomplete: Selector,
    /// Complete addition
    pub q_add: Selector,
    /// Variable-base scalar multiplication (hi half)
    pub q_mul_hi: Selector,
    /// Variable-base scalar multiplication (lo half)
    pub q_mul_lo: Selector,
    /// Selector used in scalar decomposition for variable-base scalar mul
    pub q_mul_decompose_var: Selector,
    /// Variable-base scalar multiplication (final scalar)
    pub q_mul_complete: Selector,
    /// Fixed-base full-width scalar multiplication
    pub q_mul_fixed: Selector,
    /// Fixed-base signed short scalar multiplication
    pub q_mul_fixed_short: Selector,
    /// Witness point
    pub q_point: Selector,
    /// Witness full-width scalar for fixed-base scalar mul
    pub q_scalar_fixed: Selector,
    /// Witness signed short scalar for full-width fixed-base scalar mul
    pub q_scalar_fixed_short: Selector,
    /// Permutation
    pub perm: Permutation,
}

/// A chip implementing EccInstructions
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EccChip<C: CurveAffine> {
    config: EccConfig,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> Chip<C::Base> for EccChip<C> {
    type Config = EccConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<C: CurveAffine> EccChip<C> {
    pub fn construct(config: <Self as Chip<C::Base>>::Config) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    #[allow(non_snake_case)]
    pub fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        advices: [Column<Advice>; 10],
        perm: Permutation,
    ) -> <Self as Chip<C::Base>>::Config {
        let config = EccConfig {
            advices,
            lagrange_coeffs: [
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
            ],
            fixed_z: meta.fixed_column(),
            q_add_incomplete: meta.selector(),
            q_add: meta.selector(),
            q_mul_hi: meta.selector(),
            q_mul_lo: meta.selector(),
            q_mul_decompose_var: meta.selector(),
            q_mul_complete: meta.selector(),
            q_mul_fixed: meta.selector(),
            q_mul_fixed_short: meta.selector(),
            q_point: meta.selector(),
            q_scalar_fixed: meta.selector(),
            q_scalar_fixed_short: meta.selector(),
            perm,
        };

        // Create witness point gate
        {
            let config: witness_point::Config = (&config).into();
            config.create_gate::<C>(meta);
        }

        // Create witness scalar_fixed gates that apply to both full-width and
        // short scalars
        {
            let config: witness_scalar_fixed::Config<C> = (&config).into();
            config.create_gate(meta);
        }

        // Create witness scalar_fixed gates that only apply to short scalars
        {
            let config: witness_scalar_fixed::short::Config<C> = (&config).into();
            config.create_gate(meta);
        }

        // Create incomplete point addition gate
        {
            let config: add_incomplete::Config = (&config).into();
            config.create_gate(meta);
        }

        // Create complete point addition gate
        {
            let add_config: add::Config = (&config).into();
            add_config.create_gate(meta);
        }

        // Create fixed-base scalar mul gates that are used in both full-width
        // and short multiplication.
        {
            let mul_fixed_config: mul_fixed::Config<C, { constants::NUM_WINDOWS }> =
                (&config).into();
            mul_fixed_config.create_gate(meta);
        }

        // Create gates that are only used in short fixed-base scalar mul.
        {
            let short_config: mul_fixed::short::Config<C, { constants::NUM_WINDOWS_SHORT }> =
                (&config).into();
            short_config.create_gate(meta);
        }

        // Create variable-base scalar mul gates
        {
            let mul_config: mul::Config<C> = (&config).into();
            mul_config.create_gate(meta);
        }

        config
    }
}

/// A full-width scalar used for fixed-base scalar multiplication.
/// This is decomposed in chunks of `window_width` bits in little-endian order.
/// For example, if `window_width` = 3, we will have [k_0, k_1, ..., k_n]
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_n * (2^3)^n` and each `k_i` is
/// in the range [0..2^3).
#[derive(Clone, Debug)]
pub struct EccScalarFixed<C: CurveAffine> {
    value: Option<C::Scalar>,
    windows: ArrayVec<CellValue<C::Base>, { constants::NUM_WINDOWS }>,
}

/// A signed short scalar used for fixed-base scalar multiplication.
/// This is decomposed in chunks of `window_width` bits in little-endian order.
/// For example, if `window_width` = 3, we will have [k_0, k_1, ..., k_n]
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_n * (2^3)^n` and each `k_i` is
/// in the range [0..2^3).
#[derive(Clone, Debug)]
pub struct EccScalarFixedShort<C: CurveAffine> {
    magnitude: Option<C::Scalar>,
    sign: CellValue<C::Base>,
    windows: ArrayVec<CellValue<C::Base>, { constants::NUM_WINDOWS_SHORT }>,
}

impl<C: CurveAffine> EccInstructions<C> for EccChip<C> {
    type ScalarFixed = EccScalarFixed<C>;
    type ScalarFixedShort = EccScalarFixedShort<C>;
    type ScalarVar = CellValue<C::Base>;
    type Point = EccPoint<C>;
    type X = CellValue<C::Base>;
    type FixedPoints = OrchardFixedBasesFull<C>;
    type FixedPointsShort = ValueCommitV<C>;

    fn witness_scalar_var(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Base>,
    ) -> Result<Self::ScalarVar, Error> {
        layouter.assign_region(
            || "Witness scalar for variable-base mul",
            |mut region| {
                let cell = region.assign_advice(
                    || "Scalar var",
                    self.config().advices[0],
                    0,
                    || value.ok_or(Error::SynthesisError),
                )?;
                Ok(CellValue::new(cell, value))
            },
        )
    }

    fn witness_scalar_fixed(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixed, Error> {
        let config: witness_scalar_fixed::full_width::Config<C> = self.config().into();
        layouter.assign_region(
            || "witness scalar for fixed-base mul",
            |mut region| config.assign_region(value, 0, &mut region),
        )
    }

    fn witness_scalar_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixedShort, Error> {
        let config: witness_scalar_fixed::short::Config<C> = self.config().into();
        layouter.assign_region(
            || "witness short scalar for fixed-base mul",
            |mut region| config.assign_region(value, 0, &mut region),
        )
    }

    fn witness_point(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C>,
    ) -> Result<Self::Point, Error> {
        let config: witness_point::Config = self.config().into();
        layouter.assign_region(
            || "witness point",
            |mut region| config.assign_region(value, 0, &mut region),
        )
    }

    fn extract_p(point: &Self::Point) -> &Self::X {
        &point.x
    }

    fn add_incomplete(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config: add_incomplete::Config = self.config().into();
        layouter.assign_region(
            || "incomplete point addition",
            |mut region| config.assign_region(a, b, 0, &mut region),
        )
    }

    fn add(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config: add::Config = self.config().into();
        layouter.assign_region(
            || "complete point addition",
            |mut region| config.assign_region(a, b, 0, &mut region),
        )
    }

    fn mul(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarVar,
        base: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config: mul::Config<C> = self.config().into();
        layouter.assign_region(
            || "variable-base mul",
            |mut region| config.assign_region(scalar, base, 0, &mut region),
        )
    }

    fn mul_fixed(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixed,
        base: &Self::FixedPoints,
    ) -> Result<Self::Point, Error> {
        let config: mul_fixed::full_width::Config<C, { constants::NUM_WINDOWS }> =
            self.config().into();
        layouter.assign_region(
            || format!("fixed-base mul of {:?}", base),
            |mut region| config.assign_region(scalar, *base, 0, &mut region),
        )
    }

    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixedShort,
        base: &Self::FixedPointsShort,
    ) -> Result<Self::Point, Error> {
        let config: mul_fixed::short::Config<C, { constants::NUM_WINDOWS_SHORT }> =
            self.config().into();
        layouter.assign_region(
            || format!("short fixed-base mul of {:?}", base),
            |mut region| config.assign_region(scalar, base, 0, &mut region),
        )
    }
}
