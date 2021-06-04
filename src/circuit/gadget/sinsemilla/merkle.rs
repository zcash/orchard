use halo2::{
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Permutation},
    poly::Rotation,
};
use pasta_curves::{
    arithmetic::{CurveAffine, FieldExt},
    pallas,
};

use super::{
    chip::{SinsemillaChip, SinsemillaConfig},
    message::MessagePiece,
    HashDomains, SinsemillaInstructions,
};

use crate::{
    circuit::gadget::utilities::{
        cond_swap::{CondSwapChip, CondSwapConfig, CondSwapInstructions},
        copy, CellValue, UtilitiesInstructions, Var,
    },
    primitives::sinsemilla,
    spec::i2lebsp,
};
use ff::{Field, PrimeField, PrimeFieldBits};
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
    SinsemillaInstructions<pallas::Affine, K, MAX_WORDS>
    + CondSwapInstructions<C::Base, Var = CellValue<C::Base>>
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

#[derive(Clone, Debug)]
pub struct MerkleConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    left: Column<Advice>,
    right: Column<Advice>,
    l_star: Column<Fixed>,
    perm: Permutation,
    cond_swap_config: CondSwapConfig,
    sinsemilla_config: SinsemillaConfig,
}

pub struct MerkleChip {
    config: MerkleConfig,
}

impl Chip<pallas::Base> for MerkleChip {
    type Config = MerkleConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl UtilitiesInstructions<pallas::Base> for MerkleChip {
    type Var = CellValue<pallas::Base>;
}

impl CondSwapInstructions<pallas::Base> for MerkleChip {
    #[allow(clippy::type_complexity)]
    fn swap(
        &self,
        layouter: impl Layouter<pallas::Base>,
        pair: (Self::Var, Self::Var),
        swap: Option<bool>,
    ) -> Result<(Self::Var, Self::Var), Error> {
        let config = self.config().cond_swap_config.clone();
        let chip = CondSwapChip::<pallas::Base>::construct(config);
        chip.swap(layouter, pair, swap)
    }
}

impl SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }> for MerkleChip {
    type CellValue = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::CellValue;

    type Message = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::Message;
    type MessagePiece = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::MessagePiece;
    type MessageSubPiece = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::MessageSubPiece;

    type X = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::X;
    type Point = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::Point;

    type HashDomains = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::HashDomains;

    fn witness_message(
        &self,
        layouter: impl Layouter<pallas::Base>,
        message: Vec<Option<bool>>,
    ) -> Result<Self::Message, Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::construct(config);
        chip.witness_message(layouter, message)
    }

    fn witness_message_piece_bitstring(
        &self,
        layouter: impl Layouter<pallas::Base>,
        message: &[Option<bool>],
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::construct(config);
        chip.witness_message_piece_bitstring(layouter, message)
    }

    fn witness_message_piece_field(
        &self,
        layouter: impl Layouter<pallas::Base>,
        value: Option<pallas::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::construct(config);
        chip.witness_message_piece_field(layouter, value, num_words)
    }

    fn witness_message_piece_subpieces(
        &self,
        layouter: impl Layouter<pallas::Base>,
        subpieces: &[Self::MessageSubPiece],
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::construct(config);
        chip.witness_message_piece_subpieces(layouter, subpieces)
    }

    #[allow(non_snake_case)]
    fn hash_to_point(
        &self,
        layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        message: Self::Message,
    ) -> Result<(Self::Point, Vec<Vec<Self::CellValue>>), Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::construct(config);
        chip.hash_to_point(layouter, Q, message)
    }

    fn extract(point: &Self::Point) -> Self::X {
        SinsemillaChip::extract(point)
    }
}
