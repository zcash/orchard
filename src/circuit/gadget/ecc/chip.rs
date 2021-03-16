use std::{collections::BTreeMap, marker::PhantomData};

use super::{EccInstructions, FixedPoints};
use crate::constants::{self, FixedBase, OrchardFixedBases};
use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Cell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Permutation, Selector},
    poly::Rotation,
};

mod add;
mod add_complete;
mod double;
mod mul_fixed;
mod mul_fixed_short;
mod util;
mod witness_point;
mod witness_scalar_fixed;
mod witness_scalar_fixed_short;
mod witness_scalar_var;

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
        let number_base = 1 << window_width;

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

        // Create witness point gate
        {
            let q_point = meta.query_selector(q_point, Rotation::cur());
            let P = (
                meta.query_advice(P.0, Rotation::cur()),
                meta.query_advice(P.1, Rotation::cur()),
            );
            witness_point::create_gate::<C>(meta, q_point, P.0, P.1);
        }

        // Create witness scalar_var gate
        {
            let q_scalar_var = meta.query_selector(q_scalar_var, Rotation::cur());
            let k = meta.query_advice(bits, Rotation::cur());

            witness_scalar_var::create_gate::<C>(meta, q_scalar_var, k);
        }

        // Create witness scalar_fixed gate
        {
            let q_scalar_fixed = meta.query_selector(q_scalar_fixed, Rotation::cur());
            let k = meta.query_advice(bits, Rotation::cur());
            witness_scalar_fixed::create_gate::<C>(meta, number_base, q_scalar_fixed, k);
        }

        // Create witness scalar_fixed_short gate
        {
            let q_scalar_fixed_short = meta.query_selector(q_scalar_fixed_short, Rotation::cur());
            let k = meta.query_advice(bits, Rotation::cur());
            witness_scalar_fixed_short::create_gate::<C>(meta, q_scalar_fixed_short, k);
        }

        // Create point doubling gate
        {
            let q_double = meta.query_selector(q_double, Rotation::cur());
            let x_a = meta.query_advice(A.0, Rotation::cur());
            let y_a = meta.query_advice(A.1, Rotation::cur());
            let x_p = meta.query_advice(P.0, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());

            double::create_gate::<C>(meta, q_double, x_a, y_a, x_p, y_p);
        }

        // Create point addition gate
        {
            let q_add = meta.query_selector(q_add, Rotation::cur());
            let x_p = meta.query_advice(P.0, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());
            let x_q = meta.query_advice(A.0, Rotation::cur());
            let y_q = meta.query_advice(A.1, Rotation::cur());
            let x_a = meta.query_advice(A.0, Rotation::next());
            let y_a = meta.query_advice(A.1, Rotation::next());

            add::create_gate::<C>(meta, q_add, x_p, y_p, x_q, y_q, x_a, y_a);
        }

        // Create complete point addition gate
        {
            let q_add_complete = meta.query_selector(q_add_complete, Rotation::cur());
            let x_p = meta.query_advice(P.0, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());
            let x_q = meta.query_advice(A.0, Rotation::cur());
            let y_q = meta.query_advice(A.1, Rotation::cur());
            let x_r = meta.query_advice(A.0, Rotation::next());
            let y_r = meta.query_advice(A.1, Rotation::next());
            let lambda = meta.query_advice(lambda.0, Rotation::cur());

            let a = meta.query_advice(add_complete_bool[0], Rotation::cur());
            let b = meta.query_advice(add_complete_bool[1], Rotation::cur());
            let c = meta.query_advice(add_complete_bool[2], Rotation::cur());
            let d = meta.query_advice(add_complete_bool[3], Rotation::cur());

            // \alpha = (x_q - x_p)^{-1}
            let alpha = meta.query_advice(add_complete_inv[0], Rotation::cur());
            // \beta = x_p^{-1}
            let beta = meta.query_advice(add_complete_inv[1], Rotation::cur());
            // \gamma = x_q^{-1}
            let gamma = meta.query_advice(add_complete_inv[2], Rotation::cur());
            // \delta = (y_p + y_q)^{-1}
            let delta = meta.query_advice(add_complete_inv[3], Rotation::cur());

            add_complete::create_gate::<C>(
                meta,
                q_add_complete,
                a,
                b,
                c,
                d,
                alpha,
                beta,
                gamma,
                delta,
                lambda,
                x_p,
                y_p,
                x_q,
                y_q,
                x_r,
                y_r,
            );
        }

        // Create fixed-base full-width scalar mul gate
        {
            let q_mul_fixed = meta.query_selector(q_mul_fixed, Rotation::cur());
            let x_p = meta.query_advice(P.0, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());
            let k = meta.query_advice(bits, Rotation::cur());
            let u = meta.query_advice(u, Rotation::cur());
            let z = meta.query_fixed(fixed_z, Rotation::cur());

            mul_fixed::create_gate::<C>(
                meta,
                number_base,
                lagrange_coeffs,
                q_mul_fixed,
                x_p,
                y_p,
                k,
                u,
                z,
            );
        }

        // Create fixed-base short signed scalar mul gate
        {
            let q_mul_fixed_short = meta.query_selector(q_mul_fixed_short, Rotation::cur());
            let s = meta.query_advice(bits, Rotation::cur());
            let y_a = meta.query_advice(A.1, Rotation::cur());
            let y_p = meta.query_advice(P.1, Rotation::cur());

            mul_fixed_short::create_gate::<C>(meta, q_mul_fixed_short, s, y_a, y_p);
        }

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
        let mut lagrange_coeffs = BTreeMap::<OrchardFixedBases<C>, Vec<Vec<C::Base>>>::new();
        let mut lagrange_coeffs_short = BTreeMap::<OrchardFixedBases<C>, Vec<Vec<C::Base>>>::new();
        let mut z = BTreeMap::<OrchardFixedBases<C>, [u64; constants::NUM_WINDOWS]>::new();
        let mut z_short =
            BTreeMap::<OrchardFixedBases<C>, [u64; constants::NUM_WINDOWS_SHORT]>::new();
        let mut u = BTreeMap::<OrchardFixedBases<C>, Vec<Vec<C::Base>>>::new();
        let mut u_short = BTreeMap::<OrchardFixedBases<C>, Vec<Vec<C::Base>>>::new();

        let bases: [(
            constants::OrchardFixedBases<C>,
            [u64; constants::NUM_WINDOWS],
            [u64; constants::NUM_WINDOWS_SHORT],
            [[[u8; 32]; constants::H]; constants::NUM_WINDOWS],
            [[[u8; 32]; constants::H]; constants::NUM_WINDOWS_SHORT],
        ); 5] = [
            (
                constants::commit_ivk_r::generator(),
                constants::commit_ivk_r::Z,
                constants::commit_ivk_r::Z_SHORT,
                constants::commit_ivk_r::U,
                constants::commit_ivk_r::U_SHORT,
            ),
            (
                constants::note_commit_r::generator(),
                constants::note_commit_r::Z,
                constants::note_commit_r::Z_SHORT,
                constants::note_commit_r::U,
                constants::note_commit_r::U_SHORT,
            ),
            (
                constants::nullifier_k::generator(),
                constants::nullifier_k::Z,
                constants::nullifier_k::Z_SHORT,
                constants::nullifier_k::U,
                constants::nullifier_k::U_SHORT,
            ),
            (
                constants::value_commit_r::generator(),
                constants::value_commit_r::Z,
                constants::value_commit_r::Z_SHORT,
                constants::value_commit_r::U,
                constants::value_commit_r::U_SHORT,
            ),
            (
                constants::value_commit_v::generator(),
                constants::value_commit_v::Z,
                constants::value_commit_v::Z_SHORT,
                constants::value_commit_v::U,
                constants::value_commit_v::U_SHORT,
            ),
        ];

        for base in bases.iter() {
            lagrange_coeffs.insert(
                base.0,
                base.0
                    .inner()
                    .compute_lagrange_coeffs(constants::NUM_WINDOWS)
                    .iter()
                    .map(|window| window.to_vec())
                    .collect(),
            );
            lagrange_coeffs_short.insert(
                base.0,
                base.0
                    .inner()
                    .compute_lagrange_coeffs(constants::NUM_WINDOWS_SHORT)
                    .iter()
                    .map(|window| window.to_vec())
                    .collect(),
            );
            z.insert(base.0, base.1);
            z_short.insert(base.0, base.2);
            u.insert(
                base.0,
                base.3
                    .iter()
                    .map(|window_us| {
                        window_us
                            .iter()
                            .map(|u| C::Base::from_bytes(&u).unwrap())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            );
            u_short.insert(
                base.0,
                base.4
                    .iter()
                    .map(|window_us| {
                        window_us
                            .iter()
                            .map(|u| C::Base::from_bytes(&u).unwrap())
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            );
        }

        Ok(EccLoaded {
            lagrange_coeffs,
            lagrange_coeffs_short,
            z,
            z_short,
            u,
            u_short,
        })
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
        let config = layouter.config().clone();

        let scalar = layouter.assign_region(
            || "witness scalar for variable-base mul",
            |mut region| witness_scalar_var::assign_region(value, 0, &mut region, config.clone()),
        )?;

        Ok(scalar)
    }

    fn witness_scalar_fixed(
        layouter: &mut impl Layouter<Self>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixed, Error> {
        let config = layouter.config().clone();

        let scalar = layouter.assign_region(
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
        )?;

        Ok(scalar)
    }

    fn witness_scalar_fixed_short(
        layouter: &mut impl Layouter<Self>,
        value: Option<C::Scalar>,
    ) -> Result<Self::ScalarFixedShort, Error> {
        let config = layouter.config().clone();

        let scalar = layouter.assign_region(
            || "witness scalar for fixed-base mul",
            |mut region| {
                witness_scalar_fixed_short::assign_region(
                    value,
                    constants::L_VALUE,
                    0,
                    &mut region,
                    config.clone(),
                )
            },
        )?;

        Ok(scalar)
    }

    fn witness_point(
        layouter: &mut impl Layouter<Self>,
        value: Option<C>,
    ) -> Result<Self::Point, Error> {
        let config = layouter.config().clone();

        let point = layouter.assign_region(
            || "witness point",
            |mut region| witness_point::assign_region(value, 0, &mut region, config.clone()),
        )?;

        Ok(point)
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
        let config = layouter.config().clone();

        let point = layouter.assign_region(
            || "point addition",
            |mut region| add::assign_region(a, b, 0, &mut region, config.clone()),
        )?;

        Ok(point)
    }

    fn add_complete(
        layouter: &mut impl Layouter<Self>,
        a: &Self::Point,
        b: &Self::Point,
    ) -> Result<Self::Point, Error> {
        let config = layouter.config().clone();

        let point = layouter.assign_region(
            || "point addition",
            |mut region| add_complete::assign_region(a, b, 0, &mut region, config.clone()),
        )?;

        Ok(point)
    }

    fn double(layouter: &mut impl Layouter<Self>, a: &Self::Point) -> Result<Self::Point, Error> {
        let config = layouter.config().clone();

        let point = layouter.assign_region(
            || "point doubling",
            |mut region| double::assign_region(a, 0, &mut region, config.clone()),
        )?;

        Ok(point)
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
        let config = layouter.config().clone();
        let num_windows = C::Scalar::NUM_BITS / config.window_width as u32;

        let point = layouter.assign_region(
            || format!("Multiply {:?}", base.fixed_point),
            |mut region| {
                mul_fixed::assign_region(
                    scalar,
                    base,
                    0,
                    &mut region,
                    config.clone(),
                    num_windows as usize,
                )
            },
        )?;

        Ok(point)
    }

    fn mul_fixed_short(
        layouter: &mut impl Layouter<Self>,
        scalar: &Self::ScalarFixedShort,
        base: &Self::FixedPoint,
    ) -> Result<Self::Point, Error> {
        let config = layouter.config().clone();

        // CEIL(l_value / window_width)
        let num_windows =
            (config.short_signed_bits + config.window_width - 1) / config.window_width;

        let point = layouter.assign_region(
            || format!("Multiply {:?}", base.fixed_point),
            |mut region| {
                mul_fixed_short::assign_region(
                    scalar,
                    base,
                    0,
                    &mut region,
                    config.clone(),
                    num_windows as usize,
                )
            },
        )?;

        Ok(point)
    }
}
