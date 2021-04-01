use super::super::ecc::chip::{EccChip, EccConfig};
use super::{CommitDomains, HashDomains, SinsemillaInstructions};

use crate::constants::OrchardFixedBases;
use crate::primitives::sinsemilla::K;

use ff::Field;
use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Cell, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

mod generator_table;
use generator_table::*;

/// A structure containing a cell and its assigned value.
#[derive(Clone, Debug)]
pub struct CellValue<F: FieldExt> {
    cell: Cell,
    value: Option<F>,
}

/// A message to be hashed.
#[derive(Clone, Debug)]
pub struct Message<F: FieldExt>(Vec<CellValue<F>>);

/// A domain in which $\mathsf{SinsemillaHashToPoint}$ and $\mathsf{SinsemillaHash}$ can
/// be used.
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct HashDomain<C: CurveAffine> {
    Q: C,
}

#[derive(Clone, Debug)]
pub enum OrchardHashDomains<C: CurveAffine> {
    NoteCommit(HashDomain<C>),
    CommitIvk(HashDomain<C>),
    MerkleCrh(HashDomain<C>),
}

impl<C: CurveAffine> HashDomains<C> for OrchardHashDomains<C> {}

/// A domain in which $\mathsf{SinsemillaCommit}$ and $\mathsf{SinsemillaShortCommit}$ can
/// be used.
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct OrchardCommitDomain<C: CurveAffine> {
    M: HashDomain<C>,
    R: OrchardFixedBases<C>,
}

impl<C: CurveAffine> OrchardCommitDomain<C> {
    fn M(&self) -> &HashDomain<C> {
        &self.M
    }

    fn R(&self) -> OrchardFixedBases<C> {
        self.R
    }
}

#[derive(Clone, Debug)]
pub enum OrchardCommitDomains<C: CurveAffine> {
    NoteCommit(OrchardCommitDomain<C>),
    CommitIvk(OrchardCommitDomain<C>),
}

impl<C: CurveAffine> From<OrchardCommitDomains<C>> for OrchardHashDomains<C> {
    fn from(commit_domain: OrchardCommitDomains<C>) -> Self {
        match commit_domain {
            OrchardCommitDomains::NoteCommit(domain) => Self::NoteCommit(domain.M().clone()),
            OrchardCommitDomains::CommitIvk(domain) => Self::CommitIvk(domain.M().clone()),
        }
    }
}

impl<C: CurveAffine> CommitDomains<C, OrchardFixedBases<C>, OrchardHashDomains<C>>
    for OrchardCommitDomains<C>
{
    fn r(&self) -> OrchardFixedBases<C> {
        match self {
            Self::NoteCommit(domain) => domain.R(),
            Self::CommitIvk(domain) => domain.R(),
            _ => unreachable!(),
        }
    }

    fn hash_domain(&self) -> OrchardHashDomains<C> {
        match self {
            Self::NoteCommit(_) => self.clone().into(),
            Self::CommitIvk(_) => self.clone().into(),
        }
    }
}

/// Configuration for the ECC chip
#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct SinsemillaConfig {
    ecc_config: EccConfig,
    generator_table: GeneratorTable,
    q_sinsemilla: Selector,
}

#[allow(non_snake_case)]
impl<C: CurveAffine> EccChip<C> {
    fn configure_sinsemilla(
        meta: &mut ConstraintSystem<C::Base>,
        q_sinsemilla: Selector,
        bits: Column<Advice>,
        u: Column<Advice>,
        A: (Column<Advice>, Column<Advice>),
        P: (Column<Advice>, Column<Advice>),
        lambda: (Column<Advice>, Column<Advice>),
        add_complete_bool: [Column<Advice>; 4],
        add_complete_inv: [Column<Advice>; 4],
    ) -> SinsemillaConfig {
        let ecc_config = EccChip::<C>::configure(
            meta,
            bits,
            u,
            A,
            P,
            lambda,
            add_complete_bool,
            add_complete_inv,
        );

        // Fixed column for Sinsemilla selector
        let sinsemilla_cur = meta.query_selector(q_sinsemilla, Rotation::cur());

        // m_i = z_{i + 1} - (z_i * 2^k)
        let z_cur = meta.query_advice(ecc_config.bits, Rotation::cur());
        let z_next = meta.query_advice(ecc_config.bits, Rotation::next());
        let m = z_next - z_cur * C::Base::from_u64((1 << K) as u64);

        // y_a = (1/2) ⋅ (lambda1 + lambda2) ⋅ (x_a - (lambda1^2 - x_a - x_p))
        let lambda1_cur = meta.query_advice(ecc_config.lambda.0, Rotation::cur());
        let lambda2_cur = meta.query_advice(ecc_config.lambda.1, Rotation::cur());
        let x_a_cur = meta.query_advice(ecc_config.A.0, Rotation::cur());
        let x_p_cur = meta.query_advice(ecc_config.P.0, Rotation::cur());
        let y_a_cur = (lambda1_cur.clone() + lambda2_cur.clone())
            * (x_a_cur.clone()
                - (lambda1_cur.clone() * lambda1_cur.clone() - x_a_cur.clone() - x_p_cur.clone()))
            * C::Base::TWO_INV;

        // y_p = y_a - lambda1 ⋅ (x_a - x_p)
        let y_p = y_a_cur.clone() - lambda1_cur.clone() * (x_a_cur.clone() - x_p_cur.clone());

        let (x_p_init, y_p_init) = get_s_by_idx::<C>(0).to_affine().get_xy().unwrap();

        let generator_table = GeneratorTable::configure::<C>(
            meta,
            sinsemilla_cur.clone() * m
                + (Expression::Constant(C::Base::one()) - sinsemilla_cur.clone()) * C::Base::zero(),
            sinsemilla_cur.clone() * x_p_cur.clone()
                + (Expression::Constant(C::Base::one()) - sinsemilla_cur.clone()) * x_p_init,
            sinsemilla_cur.clone() * y_p
                + (Expression::Constant(C::Base::one()) - sinsemilla_cur.clone()) * y_p_init,
        );

        // TODO: create gates

        SinsemillaConfig {
            ecc_config,
            generator_table,
            q_sinsemilla,
        }
    }
}

impl<C: CurveAffine> SinsemillaInstructions<C> for EccChip<C> {
    type Message = Message<C::Base>;
    type CommitDomains = OrchardCommitDomains<C>;
    type HashDomains = OrchardHashDomains<C>;
    type Q = Self::Point;

    #[allow(non_snake_case)]
    fn get_Q(
        layouter: &mut impl Layouter<Self>,
        domain: &Self::HashDomains,
    ) -> Result<Self::Q, Error> {
        todo!()
    }

    fn witness_message(
        layouter: &mut impl Layouter<Self>,
        message: Vec<bool>,
    ) -> Result<Self::Message, Error> {
        todo!()
    }

    fn extract(point: &Self::Point) -> Self::X {
        todo!()
    }

    #[allow(non_snake_case)]
    fn hash_to_point(
        layouter: &mut impl Layouter<Self>,
        Q: &Self::Q,
        message: Self::Message,
    ) -> Result<Self::Point, Error> {
        todo!()
    }
}
