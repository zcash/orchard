use super::super::ecc::chip::{EccChip, EccConfig};
use super::{CommitDomains, HashDomains, SinsemillaInstructions};

use crate::constants::OrchardFixedBases;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Cell, Layouter},
    plonk::{Error, Selector},
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
