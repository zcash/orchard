use super::EccInstructions;
use crate::constants::{self, load::OrchardFixedBases};
use ff::Field;
use halo2::{
    arithmetic::CurveAffine,
    circuit::{Cell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
};
use std::marker::PhantomData;

pub(super) mod add;
pub(super) mod add_incomplete;
pub(super) mod mul;
pub(super) mod mul_fixed;
pub(super) mod util;
pub(super) mod witness_point;
pub(super) mod witness_scalar_fixed;

/// A structure containing a cell and its assigned value.
#[derive(Clone, Debug)]
pub struct CellValue<T> {
    /// The cell of this `CellValue`
    pub cell: Cell,
    /// The value assigned to this `CellValue`
    pub value: Option<T>,
}

impl<T> CellValue<T> {
    /// Construct a `CellValue`.
    pub fn new(cell: Cell, value: Option<T>) -> Self {
        CellValue { cell, value }
    }
}

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
        match (self.x.value, self.y.value) {
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
            perm: Permutation::new(
                meta,
                &[
                    advices[0].into(),
                    advices[1].into(),
                    advices[2].into(),
                    advices[3].into(),
                    advices[4].into(),
                    advices[6].into(),
                    advices[7].into(),
                    advices[8].into(),
                    advices[9].into(),
                ],
            ),
        };

        // Create witness point gate
        {
            let config: witness_point::Config = (&config).into();
            config.create_gate::<C>(meta);
        }

        // Create witness scalar_fixed gates (both full-width and short)
        {
            let config: witness_scalar_fixed::Config = (&config).into();
            config.create_gate::<C>(meta);
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

        // Create fixed-base scalar mul gates
        {
            let mul_fixed_config: mul_fixed::Config<C> = (&config).into();
            mul_fixed_config.create_gate(meta);
        }

        // Create variable-base scalar mul gates
        {
            let mul_config: mul::Config<C> = (&config).into();
            mul_config.create_gate(meta);
        }

        config
    }
}

/// A full-width scalar used for variable-base scalar multiplication.
/// This is decomposed in chunks of `window_width` bits in little-endian order.
/// For example, if `window_width` = 3, we will have [k_0, k_1, ..., k_n]
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_n * (2^3)^n`.
#[derive(Clone, Debug)]
pub struct EccScalarFixed<C: CurveAffine> {
    value: Option<C::Scalar>,
    k_bits: Vec<CellValue<C::Base>>,
}

/// A signed short scalar used for variable-base scalar multiplication.
/// This is decomposed in chunks of `window_width` bits in little-endian order.
/// For example, if `window_width` = 3, we will have [k_0, k_1, ..., k_n]
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_n * (2^3)^n`.
#[derive(Clone, Debug)]
pub struct EccScalarFixedShort<C: CurveAffine> {
    magnitude: Option<C::Scalar>,
    sign: CellValue<C::Base>,
    k_bits: Vec<CellValue<C::Base>>,
}

impl<C: CurveAffine> EccInstructions<C> for EccChip<C> {
    type ScalarFixed = EccScalarFixed<C>;
    type ScalarFixedShort = EccScalarFixedShort<C>;
    type ScalarVar = CellValue<C::Base>;
    type Point = EccPoint<C>;
    type X = CellValue<C::Base>;
    type FixedPoints = OrchardFixedBases<C>;

    fn witness_scalar_var(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Base>,
    ) -> Result<Self::ScalarVar, Error> {
        layouter.assign_region(
            || "Witness scalar var",
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
        let config: witness_scalar_fixed::Config = self.config().into();
        layouter.assign_region(
            || "witness scalar for fixed-base mul",
            |mut region| config.assign_region_full(value, 0, &mut region),
        )
    }

    fn witness_scalar_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixedShort, Error> {
        let config: witness_scalar_fixed::Config = self.config().into();
        layouter.assign_region(
            || "witness scalar for fixed-base mul",
            |mut region| config.assign_region_short(value, 0, &mut region),
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
            || "point addition",
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
            || "point addition",
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
        base: Self::FixedPoints,
    ) -> Result<Self::Point, Error> {
        // Full-width fixed-base scalar mul cannot be used with ValueCommitV.
        match base {
            OrchardFixedBases::ValueCommitV(_) => return Err(Error::SynthesisError),
            _ => (),
        };

        let config: mul_fixed::Config<C> = self.config().into();
        layouter.assign_region(
            || format!("Multiply {:?}", base),
            |mut region| config.assign_region_full(scalar, base, 0, &mut region),
        )
    }

    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixedShort,
        base: Self::FixedPoints,
    ) -> Result<Self::Point, Error> {
        // Short fixed-base scalar mul is only used with ValueCommitV.
        match base {
            OrchardFixedBases::ValueCommitV(_) => (),
            _ => return Err(Error::SynthesisError),
        };

        let config: mul_fixed::Config<C> = self.config().into();
        layouter.assign_region(
            || format!("Multiply {:?}", base),
            |mut region| config.assign_region_short(scalar, base, 0, &mut region),
        )
    }
}
