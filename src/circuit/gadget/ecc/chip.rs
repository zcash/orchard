use super::EccInstructions;
use crate::constants;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Cell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
};

mod add;
mod add_incomplete;
#[cfg(test)]
mod double;
mod load;
mod mul;
mod mul_fixed;
mod util;
mod witness_point;
mod witness_scalar_fixed;

pub use load::*;

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
pub struct EccPoint<F: FieldExt> {
    /// x-coordinate
    pub x: CellValue<F>,
    /// y-coordinate
    pub y: CellValue<F>,
}

#[derive(Clone, Debug)]
/// For each Orchard fixed base, we precompute:
/// * coefficients for x-coordinate interpolation polynomials, and
/// * z-values such that y + z = u^2 some square while -y + z is non-square.
pub struct EccLoaded<C: CurveAffine> {
    commit_ivk_r: OrchardFixedBase<C>,
    note_commit_r: OrchardFixedBase<C>,
    nullifier_k: OrchardFixedBase<C>,
    value_commit_r: OrchardFixedBase<C>,
    value_commit_v: OrchardFixedBaseShort<C>,
}

/// Configuration for the ECC chip
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct EccConfig {
    /// Advice column for scalar decomposition into bits
    pub bits: Column<Advice>,
    /// Holds a point (x_p, y_p)
    pub P: (Column<Advice>, Column<Advice>),
    /// A pair (lambda1, lambda2) representing gradients
    pub lambda: (Column<Advice>, Column<Advice>),
    /// Advice columns needed by instructions in the ECC chip.
    pub extras: [Column<Advice>; 5],

    /// Coefficients of interpolation polynomials for x-coordinates (used in fixed-base scalar multiplication)
    pub lagrange_coeffs: [Column<Fixed>; constants::H],
    /// Fixed z such that y + z = u^2 some square, and -y + z is a non-square. (Used in fixed-base scalar multiplication)
    pub fixed_z: Column<Fixed>,
    /// Fixed column used in scalar decomposition for variable-base scalar mul
    pub mul_decompose: Column<Fixed>,

    #[cfg(test)]
    /// Point doubling (not used in the Orchard circuit)
    pub q_double: Selector,

    /// Incomplete addition
    pub q_add_incomplete: Selector,
    /// Complete addition
    pub q_add: Selector,
    /// Variable-base scalar multiplication (hi half)
    pub q_mul_hi: Selector,
    /// Variable-base scalar multiplication (lo half)
    pub q_mul_lo: Selector,
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
#[derive(Clone, Debug)]
pub struct EccChip<C: CurveAffine> {
    id: u64,
    config: EccConfig,
    loaded: EccLoaded<C>,
}

impl<C: CurveAffine> PartialEq for EccChip<C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<C: CurveAffine> Eq for EccChip<C> {}

impl<C: CurveAffine> EccLoaded<C> {
    fn get(&self, point: OrchardFixedBases<C>) -> OrchardFixedBase<C> {
        match point {
            OrchardFixedBases::CommitIvkR(_) => self.commit_ivk_r.clone(),
            OrchardFixedBases::NoteCommitR(_) => self.note_commit_r.clone(),
            OrchardFixedBases::NullifierK(_) => self.nullifier_k.clone(),
            OrchardFixedBases::ValueCommitR(_) => self.value_commit_r.clone(),
        }
    }

    fn get_short(&self, _point: OrchardFixedBasesShort<C>) -> OrchardFixedBaseShort<C> {
        self.value_commit_v.clone()
    }
}

impl<C: CurveAffine> Chip<C::Base> for EccChip<C> {
    type Config = EccConfig;
    type Loaded = EccLoaded<C>;

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &self.loaded
    }
}

impl<C: CurveAffine> EccChip<C> {
    pub fn construct(
        config: <Self as Chip<C::Base>>::Config,
        loaded: <Self as Chip<C::Base>>::Loaded,
    ) -> Self {
        Self {
            id: rand::random::<u64>(),
            config,
            loaded,
        }
    }

    #[allow(non_snake_case)]
    pub fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        bits: Column<Advice>,
        P: (Column<Advice>, Column<Advice>),
        lambda: (Column<Advice>, Column<Advice>),
        extras: [Column<Advice>; 5],
    ) -> <Self as Chip<C::Base>>::Config {
        let config = EccConfig {
            bits,
            P,
            lambda,
            extras,
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
            mul_decompose: meta.fixed_column(),
            #[cfg(test)]
            q_double: meta.selector(),
            q_add_incomplete: meta.selector(),
            q_add: meta.selector(),
            q_mul_hi: meta.selector(),
            q_mul_lo: meta.selector(),
            q_mul_complete: meta.selector(),
            q_mul_fixed: meta.selector(),
            q_mul_fixed_short: meta.selector(),
            q_point: meta.selector(),
            q_scalar_fixed: meta.selector(),
            q_scalar_fixed_short: meta.selector(),
            perm: Permutation::new(
                meta,
                &[
                    P.0.into(),
                    P.1.into(),
                    bits.into(),
                    extras[0].into(),
                    extras[1].into(),
                    extras[2].into(),
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

        // Create point doubling gate
        #[cfg(test)]
        {
            let config: double::Config = (&config).into();
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

        // Create fixed-base scalar mul gates
        {
            let mul_fixed_config: mul_fixed::Config = (&config).into();
            mul_fixed_config.create_gate::<C>(meta);
        }

        // Create variable-base scalar mul gates
        {
            let mul_config: mul::Config<C> = (&config).into();
            mul_config.create_gate(meta);
        }

        config
    }

    pub fn load() -> <Self as Chip<C::Base>>::Loaded {
        let commit_ivk_r = load::commit_ivk_r();
        let note_commit_r = load::note_commit_r();
        let nullifier_k = load::nullifier_k();
        let value_commit_r = load::value_commit_r();
        let value_commit_v = load::value_commit_v();

        EccLoaded {
            commit_ivk_r,
            note_commit_r,
            nullifier_k,
            value_commit_r,
            value_commit_v,
        }
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
    type Point = EccPoint<C::Base>;
    type X = CellValue<C::Base>;
    type FixedPoint = OrchardFixedBase<C>;
    type FixedPointShort = OrchardFixedBaseShort<C>;
    type FixedPoints = OrchardFixedBases<C>;
    type FixedPointsShort = OrchardFixedBasesShort<C>;

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
                    self.config().P.0,
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

    fn get_fixed(&self, fixed_point: Self::FixedPoints) -> Result<Self::FixedPoint, Error> {
        Ok(self.loaded().get(fixed_point))
    }

    fn get_fixed_short(
        &self,
        fixed_point: Self::FixedPointsShort,
    ) -> Result<Self::FixedPointShort, Error> {
        Ok(self.loaded().get_short(fixed_point))
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

    #[cfg(test)]
    fn double(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config: double::Config = self.config().into();
        layouter.assign_region(
            || "point doubling",
            |mut region| config.assign_region(a, 0, &mut region),
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
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error> {
        let config: mul_fixed::Config = self.config().into();
        layouter.assign_region(
            || format!("Multiply {:?}", base.base),
            |mut region| config.assign_region_full::<C>(scalar, base, 0, &mut region),
        )
    }

    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixedShort,
        base: &Self::FixedPointShort,
    ) -> Result<Self::Point, Error> {
        let config: mul_fixed::Config = self.config().into();
        layouter.assign_region(
            || format!("Multiply {:?}", base.base),
            |mut region| config.assign_region_short::<C>(scalar, base, 0, &mut region),
        )
    }
}
