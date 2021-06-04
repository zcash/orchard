use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Permutation},
    poly::Rotation,
};
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

use crate::spec::i2lebsp;

use super::{
    message::MessagePiece, HashDomains, SinsemillaChip, SinsemillaConfig, SinsemillaInstructions,
};

use crate::circuit::gadget::utilities::{
    cond_swap::{CondSwapChip, CondSwapConfig, CondSwapInstructions},
    copy, CellValue, UtilitiesInstructions, Var,
};
use ff::{Field, PrimeField};
use std::convert::TryInto;

/// Instructions to check the validity of a Merkle path of a given `PATH_LENGTH`.
/// The hash function used is a Sinsemilla instance with `K`-bit words.
/// The hash function can process `MAX_WORDS` words.
pub trait MerkleInstructions<
    C: CurveAffine,
    const PATH_LENGTH: usize,
    const K: usize,
    const MAX_WORDS: usize,
>:
    SinsemillaInstructions<C, K, MAX_WORDS> + CondSwapInstructions<C::Base, Var = CellValue<C::Base>>
{
    /// Check the validity of a Merkle path from a given node to a claimed root.
    /// The node may not be the leaf (height 0); we can start from a higher height.
    #[allow(non_snake_case)]
    fn hash_path(
        &self,
        layouter: impl Layouter<C::Base>,
        start_height: usize,
        node: (<Self as UtilitiesInstructions<C::Base>>::Var, Option<u32>),
        merkle_path: Vec<Option<C::Base>>,
    ) -> Result<<Self as UtilitiesInstructions<C::Base>>::Var, Error>;

    /// Compute MerkleCRH for a given `layer`. The root is at `layer 0`, and the
    /// leaves are at `layer MERKLE_DEPTH_ORCHARD` = `layer 32`.
    #[allow(non_snake_case)]
    fn hash_layer(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: C,
        l_star: usize,
        left: <Self as UtilitiesInstructions<C::Base>>::Var,
        right: <Self as UtilitiesInstructions<C::Base>>::Var,
    ) -> Result<<Self as UtilitiesInstructions<C::Base>>::Var, Error>;
}
