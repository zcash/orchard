use super::EccInstructions;
use crate::constants;
use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Cell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
    poly::Rotation,
};

mod add;
mod add_incomplete;
mod double;
mod load;
mod mul;
mod mul_fixed;
mod mul_fixed_short;
mod util;
mod witness_point;
mod witness_scalar_fixed;
mod witness_scalar_fixed_short;

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

    /// Point doubling
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
    pub q_mul_decompose: Selector,
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
    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::too_many_arguments)]
    pub fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        bits: Column<Advice>,
        P: (Column<Advice>, Column<Advice>),
        lambda: (Column<Advice>, Column<Advice>),
        extras: [Column<Advice>; 5],
    ) -> <Self as Chip<C::Base>>::Config {
        let q_double = meta.selector();
        let q_add_incomplete = meta.selector();
        let q_add = meta.selector();
        let q_mul_hi = meta.selector();
        let q_mul_lo = meta.selector();
        let q_mul_decompose = meta.selector();
        let q_mul_fixed = meta.selector();
        let q_mul_fixed_short = meta.selector();
        let q_point = meta.selector();
        let q_scalar_fixed = meta.selector();
        let q_scalar_fixed_short = meta.selector();

        let lagrange_coeffs = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];
        let fixed_z = meta.fixed_column();
        let mul_decompose = meta.fixed_column();

        // Set up permutations
        let perm = Permutation::new(
            meta,
            &[
                P.0.into(),
                P.1.into(),
                bits.into(),
                extras[0].into(),
                extras[1].into(),
                extras[2].into(),
            ],
        );

        // Create witness point gate
        {
            let q_point = meta.query_selector(q_point, Rotation::cur());
            let P = (
                meta.query_advice(P.0, Rotation::cur()),
                meta.query_advice(P.1, Rotation::cur()),
            );
            witness_point::create_gate::<C>(meta, q_point, P.0, P.1);
        }

        // Create witness scalar_fixed gate
        {
            let q_scalar_fixed = meta.query_selector(q_scalar_fixed, Rotation::cur());
            let k = meta.query_advice(bits, Rotation::cur());
            witness_scalar_fixed::create_gate(meta, q_scalar_fixed, k);
        }

        // Create witness scalar_fixed_short gate
        {
            let q_scalar_fixed_short = meta.query_selector(q_scalar_fixed_short, Rotation::cur());
            let k = meta.query_advice(bits, Rotation::cur());
            witness_scalar_fixed_short::create_gate(meta, q_scalar_fixed_short, k);
        }

        // Create point doubling gate
        {
            let q_double = meta.query_selector(q_double, Rotation::cur());
            let x_a = meta.query_advice(extras[0], Rotation::cur());
            let y_a = meta.query_advice(extras[1], Rotation::cur());
            let x_p = meta.query_advice(P.0, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());

            double::create_gate(meta, q_double, x_a, y_a, x_p, y_p);
        }

        // Create incomplete point addition gate
        {
            let q_add = meta.query_selector(q_add_incomplete, Rotation::cur());
            let x_p = meta.query_advice(P.0, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());
            let x_q = meta.query_advice(extras[0], Rotation::cur());
            let y_q = meta.query_advice(extras[1], Rotation::cur());
            let x_a = meta.query_advice(extras[0], Rotation::next());
            let y_a = meta.query_advice(extras[1], Rotation::next());

            add_incomplete::create_gate(meta, q_add, x_p, y_p, x_q, y_q, x_a, y_a);
        }

        // Create complete point addition gate
        {
            let q_add = meta.query_selector(q_add, Rotation::cur());
            let x_p = meta.query_advice(P.0, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());
            let x_q = meta.query_advice(extras[0], Rotation::cur());
            let y_q = meta.query_advice(extras[1], Rotation::cur());
            let x_r = meta.query_advice(extras[0], Rotation::next());
            let y_r = meta.query_advice(extras[1], Rotation::next());
            let lambda_cur = meta.query_advice(lambda.0, Rotation::cur());

            let a = meta.query_advice(extras[2], Rotation::cur());
            let b = meta.query_advice(extras[3], Rotation::cur());
            let c = meta.query_advice(extras[4], Rotation::cur());
            let d = meta.query_advice(lambda.1, Rotation::cur());

            // \alpha = (x_q - x_p)^{-1}
            let alpha = meta.query_advice(extras[2], Rotation::next());
            // \beta = x_p^{-1}
            let beta = meta.query_advice(extras[3], Rotation::next());
            // \gamma = x_q^{-1}
            let gamma = meta.query_advice(extras[4], Rotation::next());
            // \delta = (y_p + y_q)^{-1}
            let delta = meta.query_advice(lambda.1, Rotation::next());

            add::create_gate(
                meta, q_add, a, b, c, d, alpha, beta, gamma, delta, lambda_cur, x_p, y_p, x_q, y_q,
                x_r, y_r,
            );
        }

        // Create fixed-base full-width scalar mul gate
        {
            let q_mul_fixed = meta.query_selector(q_mul_fixed, Rotation::cur());
            let x_p = meta.query_advice(P.0, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());
            let k = meta.query_advice(bits, Rotation::cur());
            let u = meta.query_advice(extras[2], Rotation::cur());
            let z = meta.query_fixed(fixed_z, Rotation::cur());

            mul_fixed::create_gate(meta, lagrange_coeffs, q_mul_fixed, x_p, y_p, k, u, z);
        }

        // Create fixed-base short signed scalar mul gate
        {
            let q_mul_fixed_short = meta.query_selector(q_mul_fixed_short, Rotation::cur());
            let s = meta.query_advice(bits, Rotation::cur());
            let y_a = meta.query_advice(extras[1], Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());

            mul_fixed_short::create_gate(meta, q_mul_fixed_short, s, y_a, y_p);
        }

        // Create variable-base scalar mul gate (hi half)
        {
            let q_mul = meta.query_selector(q_mul_hi, Rotation::cur());

            let z_cur = meta.query_advice(bits, Rotation::cur());
            let z_prev = meta.query_advice(bits, Rotation::prev());
            let x_a_cur = meta.query_advice(extras[0], Rotation::cur());
            let x_a_next = meta.query_advice(extras[0], Rotation::next());
            let x_p_cur = meta.query_advice(P.0, Rotation::cur());
            let x_p_next = meta.query_advice(P.0, Rotation::next());
            let y_p_cur = meta.query_advice(P.1, Rotation::cur());
            let y_p_next = meta.query_advice(P.1, Rotation::next());
            let lambda1_cur = meta.query_advice(lambda.0, Rotation::cur());
            let lambda2_cur = meta.query_advice(lambda.1, Rotation::cur());
            let lambda1_next = meta.query_advice(lambda.0, Rotation::next());
            let lambda2_next = meta.query_advice(lambda.1, Rotation::next());

            mul::create_gate(
                meta,
                q_mul,
                z_cur,
                z_prev,
                x_a_cur,
                x_a_next,
                x_p_cur,
                x_p_next,
                y_p_cur,
                y_p_next,
                lambda1_cur,
                lambda2_cur,
                lambda1_next,
                lambda2_next,
            )
        }

        // Create variable-base scalar mul gate (lo half)
        {
            let q_mul = meta.query_selector(q_mul_lo, Rotation::cur());

            let z_cur = meta.query_advice(extras[1], Rotation::cur());
            let z_prev = meta.query_advice(extras[1], Rotation::prev());
            let x_a_cur = meta.query_advice(extras[2], Rotation::cur());
            let x_a_next = meta.query_advice(extras[2], Rotation::next());
            let x_p_cur = meta.query_advice(P.0, Rotation::cur());
            let x_p_next = meta.query_advice(P.0, Rotation::next());
            let y_p_cur = meta.query_advice(P.1, Rotation::cur());
            let y_p_next = meta.query_advice(P.1, Rotation::next());
            let lambda1_cur = meta.query_advice(extras[3], Rotation::cur());
            let lambda2_cur = meta.query_advice(extras[4], Rotation::cur());
            let lambda1_next = meta.query_advice(extras[3], Rotation::next());
            let lambda2_next = meta.query_advice(extras[4], Rotation::next());

            mul::create_gate(
                meta,
                q_mul,
                z_cur,
                z_prev,
                x_a_cur,
                x_a_next,
                x_p_cur,
                x_p_next,
                y_p_cur,
                y_p_next,
                lambda1_cur,
                lambda2_cur,
                lambda1_next,
                lambda2_next,
            )
        }

        // Create scalar decomposition gate for complete addition part of variable-base scalar mul
        {
            let q_mul_decompose = meta.query_selector(q_mul_decompose, Rotation::cur());
            let z_cur = meta.query_advice(bits, Rotation::cur());
            let z_prev = meta.query_advice(bits, Rotation::prev());

            mul::create_decompose_gate(meta, q_mul_decompose, z_cur, z_prev)
        }

        // Create final scalar check gate for variable-base scalar mul
        {
            let mul_decompose = meta.query_fixed(mul_decompose, Rotation::cur());
            let z_cur = meta.query_advice(bits, Rotation::cur());

            mul::create_final_scalar_gate::<C>(meta, mul_decompose, z_cur)
        }

        EccConfig {
            bits,
            P,
            lambda,
            extras,
            lagrange_coeffs,
            fixed_z,
            mul_decompose,
            q_double,
            q_add_incomplete,
            q_add,
            q_mul_hi,
            q_mul_lo,
            q_mul_decompose,
            q_mul_fixed,
            q_mul_fixed_short,
            q_point,
            q_scalar_fixed,
            q_scalar_fixed_short,
            perm,
        }
    }

    #[allow(clippy::type_complexity)]
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
        let config = self.config();

        layouter.assign_region(
            || "witness scalar for fixed-base mul",
            |mut region| {
                witness_scalar_fixed::assign_region(
                    value,
                    C::Scalar::NUM_BITS as usize,
                    0,
                    &mut region,
                    config.clone(),
                )
            },
        )
    }

    fn witness_scalar_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixedShort, Error> {
        let config = self.config();

        layouter.assign_region(
            || "witness scalar for fixed-base mul",
            |mut region| {
                witness_scalar_fixed_short::assign_region(value, 0, &mut region, config.clone())
            },
        )
    }

    fn witness_point(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        value: Option<C>,
    ) -> Result<Self::Point, Error> {
        let config = self.config();

        layouter.assign_region(
            || "witness point",
            |mut region| witness_point::assign_region(value, 0, &mut region, config.clone()),
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
        let config = self.config();

        layouter.assign_region(
            || "point addition",
            |mut region| add_incomplete::assign_region(a, b, 0, &mut region, config.clone()),
        )
    }

    fn add(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config = self.config();

        layouter.assign_region(
            || "point addition",
            |mut region| add::assign_region::<C>(a, b, 0, &mut region, config.clone()),
        )
    }

    fn double(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        a: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config = self.config();

        layouter.assign_region(
            || "point doubling",
            |mut region| double::assign_region(a, 0, &mut region, config.clone()),
        )
    }

    fn mul(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarVar,
        base: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config = self.config();

        layouter.assign_region(
            || "variable-base mul",
            |mut region| mul::assign_region::<C>(scalar, base, 0, &mut region, config.clone()),
        )
    }

    fn mul_fixed(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixed,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error> {
        let config = self.config();

        layouter.assign_region(
            || format!("Multiply {:?}", base.base),
            |mut region| {
                mul_fixed::assign_region::<C>(scalar, base, 0, &mut region, config.clone())
            },
        )
    }

    fn mul_fixed_short(
        &self,
        layouter: &mut impl Layouter<C::Base>,
        scalar: &Self::ScalarFixedShort,
        base: &Self::FixedPointShort,
    ) -> Result<Self::Point, Error> {
        let config = self.config();

        layouter.assign_region(
            || format!("Multiply {:?}", base.base),
            |mut region| {
                mul_fixed_short::assign_region::<C>(scalar, base, 0, &mut region, config.clone())
            },
        )
    }
}
