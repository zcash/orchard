use std::{collections::BTreeMap, marker::PhantomData};

use super::{EccInstructions, FixedPoints};
use crate::constants::OrchardFixedBases;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Cell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
};

/// Configuration for the ECC chip
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct EccConfig {
    // Width of windows used in fixed-base scalar multiplication
    window_width: usize,
    // Number of bits for which we use complete addition (in variable-base scalar multiplication)
    complete_bits: usize,
    // Number of bits in a short signed scalar
    short_signed_bits: usize,

    // Advice column for scalar decomposition into bits
    bits: Column<Advice>,
    // Witness u = (y + z).sqrt(), used in fixed-base scalar multiplication
    u: Column<Advice>,
    // Holds a point (x_a, y_a) that is usually the result of an addition
    A: (Column<Advice>, Column<Advice>),
    // Holds a point (x_p, y_p)
    P: (Column<Advice>, Column<Advice>),
    // A pair (lambda1, lambda2) representing gradients
    lambda: (Column<Advice>, Column<Advice>),
    // [A, B, C, D] boolean flags used in complete addition
    add_complete_bool: [Column<Advice>; 4],
    // [alpha, beta, gamma, delta] inverses used in complete addition
    add_complete_inv: [Column<Advice>; 4],
    // Coefficients of interpolation polynomials for x-coordinates (used in fixed-base scalar multiplication)
    lagrange_coeffs: [Column<Fixed>; 8],
    // Fixed z such that y + z = u^2 some square, and -y + z is a non-square. (Used in fixed-base scalar multiplication)
    fixed_z: Column<Fixed>,

    // Incomplete addition
    q_add: Selector,
    // Complete addition
    q_add_complete: Selector,
    // Point doubling
    q_double: Selector,
    // Variable-base scalar multiplication
    q_mul: Selector,
    // Fixed-base full-width scalar multiplication
    q_mul_fixed: Selector,
    // Fixed-base signed short scalar multiplication
    q_mul_fixed_short: Selector,
    // Witness point
    q_point: Selector,
    // Witness scalar for variable-base scalar mul
    q_scalar_var: Selector,
    // Witness full-width scalar for fixed-base scalar mul
    q_scalar_fixed: Selector,
    // Witness signed short scalar for full-width fixed-base scalar mul
    q_scalar_fixed_short: Selector,
    // Copy bits of decomposed scalars
    perm_scalar: Permutation,
    // Copy between (x_p, y_p) and (x_a, y_a)
    perm_sum: Permutation,
}

/// A chip implementing EccInstructions
#[derive(Debug)]
pub struct EccChip<C: CurveAffine> {
    _marker: PhantomData<C>,
}

#[allow(non_snake_case)]
impl<C: CurveAffine> EccChip<C> {
    fn configure(
        meta: &mut ConstraintSystem<C::Base>,
        window_width: usize,
        complete_bits: usize,
        short_signed_bits: usize,
        bits: Column<Advice>,
        u: Column<Advice>,
        A: (Column<Advice>, Column<Advice>),
        P: (Column<Advice>, Column<Advice>),
        lambda: (Column<Advice>, Column<Advice>),
        add_complete_bool: [Column<Advice>; 4],
        add_complete_inv: [Column<Advice>; 4],
    ) -> EccConfig {
        let q_add = meta.selector();
        let q_add_complete = meta.selector();
        let q_double = meta.selector();
        let q_mul = meta.selector();
        let q_mul_fixed = meta.selector();
        let q_mul_fixed_short = meta.selector();
        let q_point = meta.selector();
        let q_scalar_var = meta.selector();
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

        let perm_scalar = Permutation::new(meta, &[bits.into()]);

        let perm_sum = Permutation::new(meta, &[P.0.into(), P.1.into(), A.0.into(), A.1.into()]);

        EccConfig {
            window_width,
            complete_bits,
            short_signed_bits,
            bits,
            u,
            A,
            P,
            lambda,
            add_complete_bool,
            add_complete_inv,
            lagrange_coeffs,
            fixed_z,
            q_add,
            q_add_complete,
            q_double,
            q_mul,
            q_mul_fixed,
            q_mul_fixed_short,
            q_point,
            q_scalar_var,
            q_scalar_fixed,
            q_scalar_fixed_short,
            perm_scalar,
            perm_sum,
        }
    }
}

#[derive(Clone, Debug)]
/// For each Orchard fixed base, we precompute:
/// - coefficients for x-coordinate interpolation polynomials, and
/// - z-values such that y + z = u^2 some square while -y + z is non-square.
pub struct EccLoaded<C: CurveAffine> {
    lagrange_coeffs: BTreeMap<OrchardFixedBases<C>, Vec<Vec<C::Base>>>,
    lagrange_coeffs_short: BTreeMap<OrchardFixedBases<C>, Vec<Vec<C::Base>>>,
    z: BTreeMap<OrchardFixedBases<C>, [u64; 85]>,
    z_short: BTreeMap<OrchardFixedBases<C>, [u64; 22]>,
    u: BTreeMap<OrchardFixedBases<C>, Vec<Vec<C::Base>>>,
    u_short: BTreeMap<OrchardFixedBases<C>, Vec<Vec<C::Base>>>,
}

impl<C: CurveAffine> EccLoaded<C> {
    fn lagrange_coeffs(&self, point: OrchardFixedBases<C>) -> Option<Vec<Vec<C::Base>>> {
        self.lagrange_coeffs.get(&point).cloned()
    }

    fn lagrange_coeffs_short(&self, point: OrchardFixedBases<C>) -> Option<Vec<Vec<C::Base>>> {
        self.lagrange_coeffs_short.get(&point).cloned()
    }

    fn z(&self, point: OrchardFixedBases<C>) -> Option<[u64; 85]> {
        self.z.get(&point).cloned()
    }

    fn z_short(&self, point: OrchardFixedBases<C>) -> Option<[u64; 22]> {
        self.z_short.get(&point).cloned()
    }

    fn u(&self, point: OrchardFixedBases<C>) -> Option<Vec<Vec<C::Base>>> {
        self.u.get(&point).cloned()
    }

    fn u_short(&self, point: OrchardFixedBases<C>) -> Option<Vec<Vec<C::Base>>> {
        self.u_short.get(&point).cloned()
    }
}

impl<C: CurveAffine> FixedPoints<C> for OrchardFixedBases<C> {}

impl<C: CurveAffine> Chip for EccChip<C> {
    type Config = EccConfig;
    type Field = C::Base;
    type Loaded = EccLoaded<C>;

    fn load(layouter: &mut impl Layouter<Self>) -> Result<Self::Loaded, Error> {
        // Load fixed bases (interpolation polynomials)
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CellValue<F: FieldExt> {
    cell: Cell,
    value: Option<F>,
}

impl<F: FieldExt> CellValue<F> {
    pub fn new(cell: Cell, value: Option<F>) -> Self {
        CellValue { cell, value }
    }
}

#[derive(Clone, Debug)]
/// A scalar used for variable-base scalar multiplication. This is decomposed
/// bitwise in big-endian order, i.e. [k_n, k_{n-1}, ..., k_0] where
/// `scalar = k_0 + k_1 * 2 + ... + k_n * 2^n`.
pub struct EccScalarVar<C: CurveAffine> {
    value: Option<C::Scalar>,
    k_bits: Vec<CellValue<C::Base>>,
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

/// A curve point represented in affine (x, y) coordinates. Each coordinate is
/// assigned to a cell.
#[derive(Clone, Debug)]
pub struct EccPoint<F: FieldExt> {
    x: CellValue<F>,
    y: CellValue<F>,
}

/// A fixed point representing one of the Orchard fixed bases. Contains:
/// - coefficients for x-coordinate interpolation polynomials, and
/// - z-values such that y + z = u^2 some square while -y + z is non-square.
#[derive(Clone, Debug)]
pub struct EccFixedPoint<C: CurveAffine> {
    fixed_point: OrchardFixedBases<C>,
    lagrange_coeffs: Option<Vec<Vec<C::Base>>>,
    lagrange_coeffs_short: Option<Vec<Vec<C::Base>>>,
    z: Option<[u64; 85]>,
    z_short: Option<[u64; 22]>,
    u: Option<Vec<Vec<C::Base>>>,
    u_short: Option<Vec<Vec<C::Base>>>,
}

impl<C: CurveAffine> EccInstructions<C> for EccChip<C> {
    type ScalarVar = EccScalarVar<C>;
    type ScalarFixed = EccScalarFixed<C>;
    type ScalarFixedShort = EccScalarFixedShort<C>;
    type Point = EccPoint<C::Base>;
    type FixedPoint = EccFixedPoint<C>;
    type FixedPoints = OrchardFixedBases<C>;

    fn witness_scalar_var(
        layouter: &mut impl Layouter<Self>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarVar, Error> {
        todo!()
    }

    fn witness_scalar_fixed(
        layouter: &mut impl Layouter<Self>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixed, Error> {
        todo!()
    }

    fn witness_scalar_fixed_short(
        layouter: &mut impl Layouter<Self>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixedShort, Error> {
        todo!()
    }

    fn witness_point(
        layouter: &mut impl Layouter<Self>,
        value: Option<C>,
    ) -> Result<Self::Point, Error> {
        todo!()
    }

    fn get_fixed(
        layouter: &mut impl Layouter<Self>,
        fixed_point: Self::FixedPoints,
    ) -> Result<Self::FixedPoint, Error> {
        let loaded = layouter.loaded();

        let lagrange_coeffs = loaded.lagrange_coeffs(fixed_point);
        let lagrange_coeffs_short = loaded.lagrange_coeffs_short(fixed_point);
        let z = loaded.z(fixed_point);
        let z_short = loaded.z_short(fixed_point);
        let u = loaded.u(fixed_point);
        let u_short = loaded.u_short(fixed_point);

        Ok(EccFixedPoint {
            fixed_point,
            lagrange_coeffs,
            lagrange_coeffs_short,
            z,
            z_short,
            u,
            u_short,
        })
    }

    fn add(
        layouter: &mut impl Layouter<Self>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error> {
        todo!()
    }

    fn add_complete(
        layouter: &mut impl Layouter<Self>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error> {
        todo!()
    }

    fn double(layouter: &mut impl Layouter<Self>, a: &Self::Point) -> Result<Self::Point, Error> {
        todo!()
    }

    fn mul(
        layouter: &mut impl Layouter<Self>,
        scalar: &Self::ScalarVar,
        base: &Self::Point,
    ) -> Result<Self::Point, Error> {
        todo!()
    }

    fn mul_fixed(
        layouter: &mut impl Layouter<Self>,
        scalar: &Self::ScalarFixed,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error> {
        todo!()
    }

    fn mul_fixed_short(
        layouter: &mut impl Layouter<Self>,
        scalar: &Self::ScalarFixedShort,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error> {
        todo!()
    }
}
