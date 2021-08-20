use super::gadget::{EccInstructions, FixedPoints, FIXED_BASE_WINDOW_SIZE, H};
use arrayvec::ArrayVec;
use std::marker::PhantomData;
use utilities::{
    copy, decompose_running_sum::RunningSumConfig, lookup_range_check::LookupRangeCheckConfig,
    CellValue, UtilitiesInstructions, Var,
};

use group::prime::PrimeCurveAffine;
use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Selector},
};
use pasta_curves::{arithmetic::CurveAffine, pallas};

pub(super) mod add;
pub(super) mod add_incomplete;
pub(super) mod mul;
pub(super) mod mul_fixed;
pub(super) mod witness_point;

/// Number of windows for a full-width scalar
pub const NUM_WINDOWS: usize =
    (L_ORCHARD_SCALAR + FIXED_BASE_WINDOW_SIZE - 1) / FIXED_BASE_WINDOW_SIZE;

/// Number of windows for a short signed scalar
pub const NUM_WINDOWS_SHORT: usize =
    (L_VALUE + FIXED_BASE_WINDOW_SIZE - 1) / FIXED_BASE_WINDOW_SIZE;

/// $\ell_\mathsf{value}$
/// Number of bits in an unsigned short scalar.
pub(crate) const L_VALUE: usize = 64;

/// $\ell^\mathsf{Orchard}_\mathsf{base}$
/// Number of bits in a Pallas base field element.
pub(crate) const L_ORCHARD_BASE: usize = 255;

/// $\ell^\mathsf{Orchard}_\mathsf{scalar}$
/// Number of bits in a Pallas scalar field element.
pub(crate) const L_ORCHARD_SCALAR: usize = 255;

/// The Pallas scalar field modulus is $q = 2^{254} + \mathsf{t_q}$.
/// <https://github.com/zcash/pasta>
pub(crate) const T_Q: u128 = 45560315531506369815346746415080538113;

/// The Pallas base field modulus is $p = 2^{254} + \mathsf{t_p}$.
/// <https://github.com/zcash/pasta>
pub(crate) const T_P: u128 = 45560315531419706090280762371685220353;

/// Number of bits in the lookup table.
const LOOKUP_K: usize = 10;

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
    /// Constructs a point from its coordinates, without checking they are on the curve.
    ///
    /// This is an internal API that we only use where we know we have a valid curve point
    /// (specifically inside Sinsemilla).
    ///
    /// TODO: Sinsemilla is now in a separate crate. We have to find a way to limit this
    /// function to just Sinsemilla.
    pub fn from_coordinates_unchecked(
        x: CellValue<pallas::Base>,
        y: CellValue<pallas::Base>,
    ) -> Self {
        EccPoint { x, y }
    }

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
    pub lagrange_coeffs: [Column<Fixed>; H],
    /// Fixed z such that y + z = u^2 some square, and -y + z is a non-square. (Used in fixed-base scalar multiplication)
    pub fixed_z: Column<Fixed>,

    /// Incomplete addition
    pub q_add_incomplete: Selector,

    /// Complete addition
    pub q_add: Selector,

    /// Variable-base scalar multiplication (hi half)
    pub q_mul_hi: (Selector, Selector, Selector),
    /// Variable-base scalar multiplication (lo half)
    pub q_mul_lo: (Selector, Selector, Selector),
    /// Selector used to enforce boolean decomposition in variable-base scalar mul
    pub q_mul_decompose_var: Selector,
    /// Selector used to enforce switching logic on LSB in variable-base scalar mul
    pub q_mul_lsb: Selector,
    /// Variable-base scalar multiplication (overflow check)
    pub q_mul_overflow: Selector,

    /// Fixed-base full-width scalar multiplication
    pub q_mul_fixed_full: Selector,
    /// Fixed-base signed short scalar multiplication
    pub q_mul_fixed_short: Selector,
    /// Canonicity checks on base field element used as scalar in fixed-base mul
    pub q_mul_fixed_base_field: Selector,
    /// Running sum decomposition of a scalar used in fixed-base mul. This is used
    /// when the scalar is a signed short exponent or a base-field element.
    pub q_mul_fixed_running_sum: Selector,

    /// Witness point
    pub q_point: Selector,

    /// Lookup range check using 10-bit lookup table
    pub lookup_config: LookupRangeCheckConfig<pallas::Base, LOOKUP_K>,
    /// Running sum decomposition.
    pub running_sum_config: RunningSumConfig<pallas::Base, { FIXED_BASE_WINDOW_SIZE }>,
}

/// A chip implementing EccInstructions
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EccChip<Fixed: FixedPoints<pallas::Affine>> {
    config: EccConfig,
    _marker: PhantomData<Fixed>,
}

impl<Fixed: FixedPoints<pallas::Affine>> Chip<pallas::Base> for EccChip<Fixed> {
    type Config = EccConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<Fixed: FixedPoints<pallas::Affine>> UtilitiesInstructions<pallas::Base> for EccChip<Fixed> {
    type Var = CellValue<pallas::Base>;
}

impl<F: FixedPoints<pallas::Affine>> EccChip<F> {
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    /// # Side effects
    ///
    /// All columns in `advices` will be equality-enabled.
    #[allow(non_snake_case)]
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 10],
        lagrange_coeffs: [Column<Fixed>; 8],
        range_check: LookupRangeCheckConfig<pallas::Base, LOOKUP_K>,
    ) -> <Self as Chip<pallas::Base>>::Config {
        // The following columns need to be equality-enabled for their use in sub-configs:
        //
        // add::Config and add_incomplete::Config:
        // - advices[0]: x_p,
        // - advices[1]: y_p,
        // - advices[2]: x_qr,
        // - advices[3]: y_qr,
        //
        // mul_fixed::Config:
        // - advices[4]: window
        // - advices[5]: u
        //
        // mul_fixed::base_field_element::Config:
        // - [advices[6], advices[7], advices[8]]: canon_advices
        //
        // mul::overflow::Config:
        // - [advices[0], advices[1], advices[2]]: advices
        //
        // mul::incomplete::Config
        // - advices[4]: lambda1
        // - advices[9]: z
        //
        // mul::complete::Config:
        // - advices[9]: z_complete
        //
        // TODO: Refactor away from `impl From<EccConfig> for _` so that sub-configs can
        // equality-enable the columns they need to.
        for column in &advices {
            meta.enable_equality((*column).into());
        }

        let q_mul_fixed_running_sum = meta.selector();
        let running_sum_config =
            RunningSumConfig::configure(meta, q_mul_fixed_running_sum, advices[4]);

        let config = EccConfig {
            advices,
            lagrange_coeffs,
            fixed_z: meta.fixed_column(),
            q_add_incomplete: meta.selector(),
            q_add: meta.selector(),
            q_mul_hi: (meta.selector(), meta.selector(), meta.selector()),
            q_mul_lo: (meta.selector(), meta.selector(), meta.selector()),
            q_mul_decompose_var: meta.selector(),
            q_mul_overflow: meta.selector(),
            q_mul_lsb: meta.selector(),
            q_mul_fixed_full: meta.selector(),
            q_mul_fixed_short: meta.selector(),
            q_mul_fixed_base_field: meta.selector(),
            q_mul_fixed_running_sum,
            q_point: meta.selector(),
            lookup_config: range_check,
            running_sum_config,
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

        // Create gate that is used both in fixed-base mul using a short signed exponent,
        // and fixed-base mul using a base field element.
        {
            // The const generic does not matter when creating gates.
            let mul_fixed_config: mul_fixed::Config<F, { NUM_WINDOWS }> = (&config).into();
            mul_fixed_config.running_sum_coords_gate(meta);
        }

        // Create gate that is only used in full-width fixed-base scalar mul.
        {
            let mul_fixed_full_config: mul_fixed::full_width::Config<F> = (&config).into();
            mul_fixed_full_config.create_gate(meta);
        }

        // Create gate that is only used in short fixed-base scalar mul.
        {
            let short_config: mul_fixed::short::Config<F> = (&config).into();
            short_config.create_gate(meta);
        }

        // Create gate that is only used in fixed-base mul using a base field element.
        {
            let base_field_config: mul_fixed::base_field_elem::Config<F> = (&config).into();
            base_field_config.create_gate(meta);
        }

        config
    }
}

/// A full-width scalar used for fixed-base scalar multiplication.
/// This is decomposed into 85 3-bit windows in little-endian order,
/// i.e. `windows` = [k_0, k_1, ..., k_84] (for a 255-bit scalar)
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_84 * (2^3)^84` and
/// each `k_i` is in the range [0..2^3).
#[derive(Clone, Debug)]
pub struct EccScalarFixed {
    value: Option<pallas::Scalar>,
    windows: ArrayVec<CellValue<pallas::Base>, { NUM_WINDOWS }>,
}

/// A signed short scalar used for fixed-base scalar multiplication.
/// A short scalar must have magnitude in the range [0..2^64), with
/// a sign of either 1 or -1.
/// This is decomposed into 3-bit windows in little-endian order
/// using a running sum `z`, where z_{i+1} = (z_i - a_i) / (2^3)
/// for element α = a_0 + (2^3) a_1 + ... + (2^{3(n-1)}) a_{n-1}.
/// Each `a_i` is in the range [0..2^3).
///
/// `windows` = [k_0, k_1, ..., k_21] (for a 64-bit magnitude)
/// where `scalar = k_0 + k_1 * (2^3) + ... + k_84 * (2^3)^84` and
/// each `k_i` is in the range [0..2^3).
/// k_21 must be a single bit, i.e. 0 or 1.
#[derive(Clone, Debug)]
pub struct EccScalarFixedShort {
    magnitude: CellValue<pallas::Base>,
    sign: CellValue<pallas::Base>,
    running_sum: ArrayVec<CellValue<pallas::Base>, { NUM_WINDOWS_SHORT + 1 }>,
}

/// A base field element used for fixed-base scalar multiplication.
/// This is decomposed into 3-bit windows in little-endian order
/// using a running sum `z`, where z_{i+1} = (z_i - a_i) / (2^3)
/// for element α = a_0 + (2^3) a_1 + ... + (2^{3(n-1)}) a_{n-1}.
/// Each `a_i` is in the range [0..2^3).
///
/// `running_sum` = [z_0, ..., z_85], where we expect z_85 = 0.
/// Since z_0 is initialized as the scalar α, we store it as
/// `base_field_elem`.
#[derive(Clone, Debug)]
struct EccBaseFieldElemFixed {
    base_field_elem: CellValue<pallas::Base>,
    running_sum: ArrayVec<CellValue<pallas::Base>, { NUM_WINDOWS + 1 }>,
}

impl EccBaseFieldElemFixed {
    #[cfg(test)]
    fn base_field_elem(&self) -> CellValue<pallas::Base> {
        self.base_field_elem
    }
}

impl<Fixed: FixedPoints<pallas::Affine>> EccInstructions<pallas::Affine> for EccChip<Fixed> {
    type ScalarFixed = EccScalarFixed;
    type ScalarFixedShort = EccScalarFixedShort;
    type ScalarVar = CellValue<pallas::Base>;
    type Point = EccPoint;
    type X = CellValue<pallas::Base>;
    type FixedPoints = Fixed;

    fn constrain_equal(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "constrain equal",
            |mut region| {
                // Constrain x-coordinates
                region.constrain_equal(a.x().cell(), b.x().cell())?;
                // Constrain x-coordinates
                region.constrain_equal(a.y().cell(), b.y().cell())
            },
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
        scalar: &Self::Var,
        base: &Self::Point,
    ) -> Result<(Self::Point, Self::ScalarVar), Error> {
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
        scalar: Option<pallas::Scalar>,
        base: &Self::FixedPoints,
    ) -> Result<(Self::Point, Self::ScalarFixed), Error> {
        let config: mul_fixed::full_width::Config<Fixed> = self.config().into();
        config.assign(
            layouter.namespace(|| format!("fixed-base mul of {:?}", base)),
            scalar,
            base,
        )
    }

    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        magnitude_sign: (CellValue<pallas::Base>, CellValue<pallas::Base>),
        base: &Self::FixedPoints,
    ) -> Result<(Self::Point, Self::ScalarFixedShort), Error> {
        let config: mul_fixed::short::Config<Fixed> = self.config().into();
        config.assign(
            layouter.namespace(|| format!("short fixed-base mul of {:?}", base)),
            magnitude_sign,
            base,
        )
    }

    fn mul_fixed_base_field_elem(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        base_field_elem: CellValue<pallas::Base>,
        base: &Self::FixedPoints,
    ) -> Result<Self::Point, Error> {
        let config: mul_fixed::base_field_elem::Config<Fixed> = self.config().into();
        config.assign(
            layouter.namespace(|| format!("base-field elem fixed-base mul of {:?}", base)),
            base_field_elem,
            base,
        )
    }
}
