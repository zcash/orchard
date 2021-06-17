use super::EccInstructions;
use crate::{
    circuit::gadget::utilities::{
        copy, lookup_range_check::LookupRangeCheckConfig, CellValue, Var,
    },
    constants::{self, OrchardFixedBasesFull, ValueCommitV},
    primitives::sinsemilla,
};
use arrayvec::ArrayVec;

use group::prime::PrimeCurveAffine;
use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
};
use pasta_curves::{arithmetic::CurveAffine, pallas};

pub(super) mod add;
pub(super) mod add_incomplete;
pub(super) mod mul;
pub(super) mod mul_fixed;
pub(super) mod witness_point;
pub(super) mod witness_scalar_fixed;

/// A curve point represented in affine (x, y) coordinates. Each coordinate is
/// assigned to a cell.
#[derive(Clone, Debug)]
pub struct EccPoint {
    /// x-coordinate
    x: CellValue<pallas::Base>,
    /// y-coordinate
    y: CellValue<pallas::Base>,
}

impl EccPoint {
    /// Returns the value of this curve point, if known.
    pub fn point(&self) -> Option<pallas::Affine> {
        match (self.x.value(), self.y.value()) {
            (Some(x), Some(y)) => {
                if x == pallas::Base::zero() && y == pallas::Base::zero() {
                    Some(pallas::Affine::identity())
                } else {
                    Some(pallas::Affine::from_xy(x, y).unwrap())
                }
            }
            _ => None,
        }
    }
    /// The cell containing the affine short-Weierstrass x-coordinate,
    /// or 0 for the zero point.
    pub fn x(&self) -> CellValue<pallas::Base> {
        self.x
    }
    /// The cell containing the affine short-Weierstrass y-coordinate,
    /// or 0 for the zero point.
    pub fn y(&self) -> CellValue<pallas::Base> {
        self.y
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
    /// Selector used in scalar decomposition for variable-base scalar mul
    pub q_init_z: Selector,
    /// Variable-base scalar multiplication (final scalar)
    pub q_mul_z: Selector,
    /// Variable-base scalar multiplication (overflow check)
    pub q_mul_overflow: Selector,

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
    /// 10-bit lookup
    pub lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
}

/// A chip implementing EccInstructions
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EccChip {
    config: EccConfig,
}

impl Chip<pallas::Base> for EccChip {
    type Config = EccConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl EccChip {
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self { config }
    }

    #[allow(non_snake_case)]
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 10],
        lookup_table: Column<Fixed>,
        perm: Permutation,
    ) -> <Self as Chip<pallas::Base>>::Config {
        let lookup_config =
            LookupRangeCheckConfig::configure(meta, advices[9], lookup_table, perm.clone());

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
            q_init_z: meta.selector(),
            q_mul_z: meta.selector(),
            q_mul_overflow: meta.selector(),
            q_mul_fixed: meta.selector(),
            q_mul_fixed_short: meta.selector(),
            q_point: meta.selector(),
            q_scalar_fixed: meta.selector(),
            q_scalar_fixed_short: meta.selector(),
            perm,
            lookup_config,
        };

        // Create witness point gate
        {
            let config: witness_point::Config = (&config).into();
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

        // Create variable-base scalar mul gates
        {
            let mul_config: mul::Config = (&config).into();
            mul_config.create_gate(meta);
        }

        // Create witness scalar_fixed gate that applies to both full-width and
        // short scalars
        {
            let config: witness_scalar_fixed::Config = (&config).into();
            config.create_gate(meta);
        }

        // Create witness scalar_fixed gate that only applies to short scalars
        {
            let config: witness_scalar_fixed::short::Config = (&config).into();
            config.create_gate(meta);
        }

        // Create fixed-base scalar mul gate that is used in both full-width
        // and short multiplication.
        {
            let mul_fixed_config: mul_fixed::Config<{ constants::NUM_WINDOWS }> = (&config).into();
            mul_fixed_config.create_gate(meta);
        }

        // Create gate that is only used in short fixed-base scalar mul.
        {
            let short_config: mul_fixed::short::Config<{ constants::NUM_WINDOWS_SHORT }> =
                (&config).into();
            short_config.create_gate(meta);
        }

        config
    }
}

/// A base-field element used as the scalar in variable-base scalar multiplication.
#[derive(Copy, Clone, Debug)]
pub struct EccScalarVar(CellValue<pallas::Base>);
impl std::ops::Deref for EccScalarVar {
    type Target = CellValue<pallas::Base>;

    fn deref(&self) -> &CellValue<pallas::Base> {
        &self.0
    }
}

/// A full-width scalar used for fixed-base scalar multiplication.
/// This is decomposed in chunks of `window_width` bits in little-endian order.
/// For example, if `window_width` = 3, we will have [k_0, k_1, ..., k_n]
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_n * (2^3)^n` and each `k_i` is
/// in the range [0..2^3).
#[derive(Clone, Debug)]
pub struct EccScalarFixed {
    value: Option<pallas::Scalar>,
    windows: ArrayVec<CellValue<pallas::Base>, { constants::NUM_WINDOWS }>,
}

/// A signed short scalar used for fixed-base scalar multiplication.
/// This is decomposed in chunks of `window_width` bits in little-endian order.
/// For example, if `window_width` = 3, we will have [k_0, k_1, ..., k_n]
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_n * (2^3)^n` and each `k_i` is
/// in the range [0..2^3).
#[derive(Clone, Debug)]
pub struct EccScalarFixedShort {
    magnitude: Option<pallas::Scalar>,
    sign: CellValue<pallas::Base>,
    windows: ArrayVec<CellValue<pallas::Base>, { constants::NUM_WINDOWS_SHORT }>,
}

impl EccInstructions<pallas::Affine> for EccChip {
    type ScalarFixed = EccScalarFixed;
    type ScalarFixedShort = EccScalarFixedShort;
    type ScalarVar = EccScalarVar;
    type Point = EccPoint;
    type X = CellValue<pallas::Base>;
    type FixedPoints = OrchardFixedBasesFull;
    type FixedPointsShort = ValueCommitV;

    fn constrain_equal(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<(), Error> {
        let config = self.config().clone();
        layouter.assign_region(
            || "constrain equal",
            |mut region| {
                // Constrain x-coordinates
                region.constrain_equal(&config.perm, a.x().cell(), b.x().cell())?;
                // Constrain x-coordinates
                region.constrain_equal(&config.perm, a.y().cell(), b.y().cell())
            },
        )
    }

    fn witness_scalar_var(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        value: Option<pallas::Base>,
    ) -> Result<Self::ScalarVar, Error> {
        let config = self.config().clone();
        layouter.assign_region(
            || "Witness scalar for variable-base mul",
            |mut region| {
                let cell = region.assign_advice(
                    || "witness scalar var",
                    config.advices[0],
                    0,
                    || value.ok_or(Error::SynthesisError),
                )?;
                Ok(EccScalarVar(CellValue::new(cell, value)))
            },
        )
    }

    fn witness_scalar_fixed(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        value: Option<pallas::Scalar>,
    ) -> Result<Self::ScalarFixed, Error> {
        let config: witness_scalar_fixed::full_width::Config = self.config().into();
        layouter.assign_region(
            || "witness scalar for fixed-base mul",
            |mut region| config.assign_region(value, 0, &mut region),
        )
    }

    fn witness_scalar_fixed_short(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        value: Option<pallas::Scalar>,
    ) -> Result<Self::ScalarFixedShort, Error> {
        let config: witness_scalar_fixed::short::Config = self.config().into();
        layouter.assign_region(
            || "witness short scalar for fixed-base mul",
            |mut region| config.assign_region(value, 0, &mut region),
        )
    }

    fn witness_point(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        value: Option<pallas::Affine>,
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
        layouter: &mut impl Layouter<pallas::Base>,
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
        layouter: &mut impl Layouter<pallas::Base>,
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
        layouter: &mut impl Layouter<pallas::Base>,
        scalar: &Self::ScalarVar,
        base: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config: mul::Config = self.config().into();
        config.assign(
            layouter.namespace(|| "variable-base scalar mul"),
            *scalar,
            base,
        )
    }

    fn mul_fixed(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        scalar: &Self::ScalarFixed,
        base: &Self::FixedPoints,
    ) -> Result<Self::Point, Error> {
        let config: mul_fixed::full_width::Config<{ constants::NUM_WINDOWS }> =
            self.config().into();
        layouter.assign_region(
            || format!("fixed-base mul of {:?}", base),
            |mut region| config.assign_region(scalar, *base, 0, &mut region),
        )
    }

    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        scalar: &Self::ScalarFixedShort,
        base: &Self::FixedPointsShort,
    ) -> Result<Self::Point, Error> {
        let config: mul_fixed::short::Config<{ constants::NUM_WINDOWS_SHORT }> =
            self.config().into();
        layouter.assign_region(
            || format!("short fixed-base mul of {:?}", base),
            |mut region| config.assign_region(scalar, base, 0, &mut region),
        )
    }
}
