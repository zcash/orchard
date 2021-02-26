use std::marker::PhantomData;

use super::super::ecc::{CellValue, EccPoint};
use super::SinsemillaInstructions;
use crate::primitives::sinsemilla::Q_PERSONALIZATION;
use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, CurveExt, FieldExt},
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

mod generator_table;
use generator_table::*;

const K: usize = 10;
const C: usize = 253;

fn lebs2ip_32(bits: &[bool]) -> u32 {
    bits.iter()
        .enumerate()
        .fold(0u32, |acc, (i, b)| acc + if *b { 1 << i } else { 0 })
}

/// Configuration for the Sinsemilla chip
#[derive(Clone, Debug)]
pub struct SinsemillaConfig {
    columns: SinsemillaColumns,
    lookup_table: GeneratorTable,
}

/// Columns needed for one Sinsemilla hash
#[derive(Clone, Debug)]
pub struct SinsemillaColumns {
    sinsemilla: Column<Fixed>,
    x_a: Column<Advice>,
    z: Column<Advice>,
    lambda1: Column<Advice>,
    lambda2: Column<Advice>,
    x_p: Column<Advice>,
}

impl SinsemillaColumns {
    /// Construct a new instance of `SinsemillaColumns`
    pub fn new(
        sinsemilla: Column<Fixed>,
        x_a: Column<Advice>,
        z: Column<Advice>,
        lambda1: Column<Advice>,
        lambda2: Column<Advice>,
        x_p: Column<Advice>,
    ) -> Self {
        SinsemillaColumns {
            sinsemilla,
            x_a,
            z,
            lambda1,
            lambda2,
            x_p,
        }
    }
}

/// A chip implementing SinsemillaInstructions
#[derive(Debug)]
pub struct SinsemillaChip<C: CurveAffine> {
    _marker_c: PhantomData<C>,
}

impl<F: FieldExt, C: CurveAffine<Base = F>> SinsemillaChip<C> {
    /// Configures this chip for use in a circuit.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        columns: SinsemillaColumns,
    ) -> SinsemillaConfig {
        // Fixed column for Sinsemilla selector
        let sinsemilla_cur = meta.query_fixed(columns.sinsemilla, Rotation::cur());

        // m_i = z_{i + 1} - (z_i * 2^k)
        let z_cur = meta.query_advice(columns.z, Rotation::cur());
        let z_next = meta.query_advice(columns.z, Rotation::next());
        let m = z_next - z_cur * F::from_u64((1 << K) as u64);

        // y_a = (1/2) ⋅ (lambda1 + lambda2) ⋅ (x_a - (lambda1^2 - x_a - x_p))
        let lambda1_cur = meta.query_advice(columns.lambda1, Rotation::cur());
        let lambda2_cur = meta.query_advice(columns.lambda2, Rotation::cur());
        let x_a_cur = meta.query_advice(columns.x_a, Rotation::cur());
        let x_p_cur = meta.query_advice(columns.x_p, Rotation::cur());
        let y_a_cur = (lambda1_cur.clone() + lambda2_cur.clone())
            * (x_a_cur.clone()
                - (lambda1_cur.clone() * lambda1_cur.clone() - x_a_cur.clone() - x_p_cur.clone()))
            * F::TWO_INV;

        // y_p = y_a - lambda1 ⋅ (x_a - x_p)
        let y_p = y_a_cur.clone() - lambda1_cur.clone() * (x_a_cur.clone() - x_p_cur.clone());

        let (x_p_init, y_p_init) = get_s_by_idx::<C>(0).to_affine().get_xy().unwrap();

        let lookup_table = GeneratorTable::configure::<C>(
            meta,
            sinsemilla_cur.clone() * m
                + (Expression::Constant(F::one()) - sinsemilla_cur.clone()) * F::zero(),
            sinsemilla_cur.clone() * x_p_cur.clone()
                + (Expression::Constant(F::one()) - sinsemilla_cur.clone()) * x_p_init,
            sinsemilla_cur.clone() * y_p
                + (Expression::Constant(F::one()) - sinsemilla_cur.clone()) * y_p_init,
        );

        // TODO: Create gate

        SinsemillaConfig {
            columns,
            lookup_table,
        }
    }
}

impl<C: CurveAffine> Chip for SinsemillaChip<C> {
    type Config = SinsemillaConfig;
    type Field = C::Base;
    type Loaded = ();

    fn load(layouter: &mut impl Layouter<Self>) -> Result<Self::Loaded, Error> {
        let table = layouter.config().lookup_table.clone();
        table.load(layouter)
    }
}

impl<C: CurveAffine> SinsemillaInstructions<C> for SinsemillaChip<C> {
    type Message = Vec<bool>;
    type Point = EccPoint<C::Base>;
    type X = CellValue<C::Base>;

    fn witness_message(
        layouter: &mut impl Layouter<Self>,
        message: Vec<bool>,
    ) -> Result<Self::Message, Error> {
        todo!()
    }

    fn extract(point: &Self::Point) -> Self::X {
        todo!()
    }

    fn load_Q(domain_prefix: &str) -> Result<Self::Point, Error> {
        todo!()
    }

    fn hash_to_point(
        layouter: &mut impl Layouter<Self>,
        domain_prefix: &str,
        message: Self::Message,
    ) -> Result<Self::Point, Error> {
        let config = layouter.config().clone();

        // Pad message to nearest multiple of K bits
        assert!(message.len() <= K * C);
        let pad = message.len() % K;
        let padded: Vec<_> = message
            .into_iter()
            .chain(std::iter::repeat(false).take(pad))
            .collect();

        // Parse message into `K`-bit words
        let words: Vec<u32> = padded.chunks(K).map(|chunk| lebs2ip_32(chunk)).collect();

        // Get (x_p, y_p) for each word. We precompute this here so that we can use `batch_normalize()`.
        let generators_projective: Vec<_> =
            words.iter().map(|word| get_s_by_idx::<C>(*word)).collect();
        let mut generators = vec![C::default(); generators_projective.len()];
        C::Curve::batch_normalize(&generators_projective, &mut generators);
        let generators: Vec<(C::Base, C::Base)> =
            generators.iter().map(|gen| gen.get_xy().unwrap()).collect();

        // Initialize `(x_a, y_a)` to be `(x_q, y_q)`
        let q = Self::load_Q(domain_prefix);
        // let (mut x_a, mut y_a) = q.to_affine().get_xy().unwrap();

        // TODO: Assign cells

        todo!()
    }

    fn commit(
        domain_prefix: &str,
        msg: Self::Message,
        r: &C::Scalar,
    ) -> Result<Self::Point, Error> {
        todo!()
    }
}
