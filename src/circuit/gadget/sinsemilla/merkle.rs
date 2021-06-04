use std::array;

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
use ff::{PrimeField, PrimeFieldBits};
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
    a_a0: Column<Advice>,
    b_a1: Column<Advice>,
    c_b0: Column<Advice>,
    left_b1: Column<Advice>,
    right: Column<Advice>,
    l_star_plus1: Column<Fixed>,
    perm: Permutation,
    pub cond_swap_config: CondSwapConfig,
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

impl MerkleChip {
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        sinsemilla_config: SinsemillaConfig,
    ) -> MerkleConfig {
        let advices = sinsemilla_config.advices();
        let cond_swap_config = CondSwapChip::configure(
            meta,
            advices[..5].try_into().unwrap(),
            sinsemilla_config.perm.clone(),
        );

        let a_a0 = advices[0];
        let b_a1 = advices[1];
        let c_b0 = advices[2];
        let left_b1 = advices[3];
        let right = advices[4];

        let l_star_plus1 = meta.fixed_column();

        // Check that pieces have been decomposed correctly for Sinsemilla hash.
        // <https://zips.z.cash/protocol/nu5.pdf#orchardmerklecrh>
        //
        // `a = a_0||a_1` = `l_star` || (bits 0..=239 of `left`)
        // `b = b_0||b_1` = (bits 240..=254 of `left`) || (bits 0..=234 of `right`)
        // `c = bits 235..=254 of `right`
        meta.create_gate("Merkle path validity check", |meta| {
            let a_whole = meta.query_advice(a_a0, Rotation::cur());
            let b_whole = meta.query_advice(b_a1, Rotation::cur());
            let c_whole = meta.query_advice(c_b0, Rotation::cur());
            let left_node = meta.query_advice(left_b1, Rotation::cur());
            let right_node = meta.query_advice(right, Rotation::cur());

            let a_0 = meta.query_advice(a_a0, Rotation::next());
            let a_1 = meta.query_advice(b_a1, Rotation::next());
            let b_0 = meta.query_advice(c_b0, Rotation::next());
            let b_1 = meta.query_advice(left_b1, Rotation::next());

            let l_star_plus1 = meta.query_fixed(l_star_plus1, Rotation::cur());

            // a = a_0||a_1` = `l_star` (10 bits) || (bits 0..=239 of `left`)
            // Check that a = a_0 || a_1
            let a_check = a_0.clone() + a_1.clone() * pallas::Base::from_u64(1 << 10) - a_whole;

            // Check that a_0 = l_star
            let l_star_check =
                a_0 - (l_star_plus1.clone() - Expression::Constant(pallas::Base::one()));

            // `b = b_0||b_1` = (bits 240..=254 of `left`) || (bits 0..=234 of `right`)
            // Check that b = b_0 (15 bits) || b_1 (235 bits)
            let b_check = b_0.clone() + b_1.clone() * pallas::Base::from_u64(1 << 15) - b_whole;

            // Check that left = a_1 (240 bits) || b_0 (15 bits)
            let two_pow_240 = pallas::Base::from_u128(1 << 120).square();
            let left_check = a_1 + b_0 * two_pow_240 - left_node;

            // Check that right = b_1 (235 bits) || c (20 bits)
            let two_pow_235 = pallas::Base::from_u64(1 << 47).pow(&[5, 0, 0, 0]);
            let right_check = b_1 + c_whole * two_pow_235 - right_node;

            array::IntoIter::new([a_check, l_star_check, b_check, left_check, right_check])
                .map(move |poly| l_star_plus1.clone() * poly)
        });

        MerkleConfig {
            a_a0,
            b_a1,
            c_b0,
            left_b1,
            right,
            l_star_plus1,
            perm: sinsemilla_config.perm.clone(),
            cond_swap_config,
            sinsemilla_config,
        }
    }

    pub fn construct(config: MerkleConfig) -> Self {
        MerkleChip { config }
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
