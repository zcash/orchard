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
    chip::{SinsemillaChip, SinsemillaConfig, SinsemillaHashDomains},
    message::MessageSubPiece,
    HashDomains, SinsemillaInstructions,
};

use crate::{
    circuit::gadget::utilities::{
        cond_swap::{CondSwapChip, CondSwapConfig, CondSwapInstructions},
        copy, CellValue, UtilitiesInstructions, Var,
    },
    constants::MERKLE_DEPTH_ORCHARD,
    primitives::sinsemilla,
    spec::i2lebsp,
};
use ff::PrimeField;
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
    + CondSwapInstructions<pallas::Base, Var = CellValue<pallas::Base>>
{
    /// Check the validity of a Merkle path from a given node to a claimed root.
    /// The node may not be the leaf (height 0); we can start from a higher height.
    #[allow(non_snake_case)]
    fn hash_path(
        &self,
        layouter: impl Layouter<pallas::Base>,
        start_height: usize,
        node: (
            <Self as UtilitiesInstructions<pallas::Base>>::Var,
            Option<u32>,
        ),
        merkle_path: Vec<Option<pallas::Base>>,
    ) -> Result<<Self as UtilitiesInstructions<pallas::Base>>::Var, Error>;

    /// Compute MerkleCRH for a given `layer`. The root is at `layer 0`, and the
    /// leaves are at `layer MERKLE_DEPTH_ORCHARD` = `layer 32`.
    #[allow(non_snake_case)]
    fn hash_layer(
        &self,
        layouter: impl Layouter<pallas::Base>,
        Q: C,
        l_star: usize,
        left: <Self as UtilitiesInstructions<pallas::Base>>::Var,
        right: <Self as UtilitiesInstructions<pallas::Base>>::Var,
    ) -> Result<<Self as UtilitiesInstructions<pallas::Base>>::Var, Error>;
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

impl
    MerkleInstructions<
        pallas::Affine,
        { MERKLE_DEPTH_ORCHARD / 2 },
        { sinsemilla::K },
        { sinsemilla::C },
    > for MerkleChip
{
    /// Hash a Merkle path from a given leaf and output the root.
    #[allow(non_snake_case)]
    fn hash_path(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        start_height: usize,
        node: (
            <Self as UtilitiesInstructions<pallas::Base>>::Var,
            Option<u32>,
        ),
        merkle_path: Vec<Option<pallas::Base>>,
    ) -> Result<<Self as UtilitiesInstructions<pallas::Base>>::Var, Error> {
        let domain = SinsemillaHashDomains::MerkleCrh;

        assert_eq!(merkle_path.len(), MERKLE_DEPTH_ORCHARD / 2);

        let config = self.config().clone();

        // Get position as a 32-bit bitstring (little-endian bit order).
        let pos = node.1;
        let pos: Option<[bool; MERKLE_DEPTH_ORCHARD / 2]> = pos.map(|pos| i2lebsp(pos as u64));
        let pos: [Option<bool>; MERKLE_DEPTH_ORCHARD / 2] = if let Some(pos) = pos {
            pos.iter()
                .map(|pos| Some(*pos))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        } else {
            [None; MERKLE_DEPTH_ORCHARD / 2]
        };

        let Q = domain.Q();

        let mut node = node.0;
        for (l_star, (sibling, pos)) in merkle_path.iter().zip(pos.iter()).enumerate() {
            // `l_star` = MERKLE_DEPTH_ORCHARD - layer - 1, which is the index obtained from
            // enumerating this Merkle path (going from node to root).
            // For example, when `layer = 31` (the first sibling on the Merkle path),
            // we have `l_star` = 32 - 31 - 1 = 0.
            // On the other hand, when `layer = 0` (the final sibling on the Merkle path),
            // we have `l_star` = 32 - 0 - 1 = 31.
            let l_star = l_star + start_height;

            let pair = {
                // Load sibling into circuit
                let sibling = self.load_private(
                    layouter.namespace(|| ""),
                    config.cond_swap_config.a,
                    *sibling,
                )?;
                let pair = (node, sibling);

                // Swap node and sibling if needed
                self.swap(layouter.namespace(|| ""), pair, *pos)?
            };

            // Each `hash_layer` consists of 52 Sinsemilla words:
            //  - l_star (10 bits) = 1 word
            //  - left (255 bits) || right (255 bits) = 51 words (510 bits)
            node = self.hash_layer(
                layouter.namespace(|| format!("hash l_star {}", l_star)),
                Q,
                l_star,
                pair.0,
                pair.1,
            )?;
        }

        Ok(node)
    }

    #[allow(non_snake_case)]
    fn hash_layer(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        l_star: usize,
        left: <Self as UtilitiesInstructions<pallas::Base>>::Var,
        right: <Self as UtilitiesInstructions<pallas::Base>>::Var,
    ) -> Result<<Self as UtilitiesInstructions<pallas::Base>>::Var, Error> {
        // <https://zips.z.cash/protocol/nu5.pdf#orchardmerklecrh>
        // We need to hash `l_star || left || right`, where `l_star` is a 10-bit value.
        // We allow `left` and `right` to be non-canonical 255-bit encodings.
        //
        // `a = a_0||a_1` = `l_star` || (bits 0..=239 of `left`)
        // `b = b_0||b_1` = (bits 240..=254 of `left`) || (bits 0..=234 of `right`)
        // `c = bits 235..=254 of `right`

        // `a = a_0||a_1` = `l` || (bits 0..=239 of `left`)
        let a = {
            // a_0 = l_star
            let a_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (Some(pallas::Base::from_u64(l_star as u64)), 0..10).into();

            // a_1 = (bits 0..=239 of `left`)
            let a_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (left.value(), 0..240).into();

            self.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness a = a_0 || a_1"),
                &[a_0, a_1],
            )?
        };

        // `b = b_0||b_1` = (bits 240..=254 of `left`) || (bits 0..=234 of `right`)
        let b = {
            // b_0 = (bits 240..=254 of `left`)
            let b_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (left.value(), 240..(pallas::Base::NUM_BITS as usize)).into();

            // b_1 = (bits 0..=234 of `right`)
            let b_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (right.value(), 0..235).into();

            self.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness b = b_0 || b_1"),
                &[b_0, b_1],
            )?
        };

        let c = {
            // `c = bits 235..=254 of `right`
            let c: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (right.value(), 235..(pallas::Base::NUM_BITS as usize)).into();

            self.witness_message_piece_field(layouter.namespace(|| "c"), c.field_elem_subset(), 2)?
        };

        let config = self.config().clone();
        // Check that the pieces have been decomposed properly.
        {
            layouter.assign_region(
                || "Check piece decomposition",
                |mut region| {
                    // Set the fixed column `l_star_plus1` to the current l_star + 1.
                    let l_star_plus1 = (l_star as u64) + 1;
                    region.assign_fixed(
                        || format!("l_star_plus1 {}", l_star_plus1),
                        config.l_star_plus1,
                        0,
                        || Ok(pallas::Base::from_u64(l_star_plus1)),
                    )?;

                    // Copy and assign `a` at the correct position.
                    copy(
                        &mut region,
                        || "copy a",
                        config.a_a0,
                        0,
                        &a.cell_value(),
                        &config.perm,
                    )?;
                    // Copy and assign `b` at the correct position.
                    copy(
                        &mut region,
                        || "copy b",
                        config.b_a1,
                        0,
                        &b.cell_value(),
                        &config.perm,
                    )?;
                    // Copy and assign `c` at the correct position.
                    copy(
                        &mut region,
                        || "copy c",
                        config.c_b0,
                        0,
                        &c.cell_value(),
                        &config.perm,
                    )?;
                    // Copy and assign the left node at the correct position.
                    copy(
                        &mut region,
                        || "left",
                        config.left_b1,
                        0,
                        &left,
                        &config.perm,
                    )?;
                    // Copy and assign the right node at the correct position.
                    copy(
                        &mut region,
                        || "right",
                        config.right,
                        0,
                        &right,
                        &config.perm,
                    )?;

                    // Copy and assign the subpiece `a_0` at the correct position.
                    copy(
                        &mut region,
                        || "a_0",
                        config.a_a0,
                        1,
                        &a.subpieces()[0].cell_value(),
                        &config.perm,
                    )?;
                    // Copy and assign the subpiece `a_1` at the correct position.
                    copy(
                        &mut region,
                        || "a_1",
                        config.b_a1,
                        1,
                        &a.subpieces()[1].cell_value(),
                        &config.perm,
                    )?;
                    // Copy and assign the subpiece `b_0` at the correct position.
                    copy(
                        &mut region,
                        || "b_0",
                        config.c_b0,
                        1,
                        &b.subpieces()[0].cell_value(),
                        &config.perm,
                    )?;
                    // Copy and assign the subpiece `b_1` at the correct position.
                    copy(
                        &mut region,
                        || "b_1",
                        config.left_b1,
                        1,
                        &b.subpieces()[1].cell_value(),
                        &config.perm,
                    )?;

                    Ok(())
                },
            )?;
        }

        let (point, _) = self.hash_to_point(
            layouter.namespace(|| format!("l_star {}", l_star)),
            Q,
            vec![a, b, c].into(),
        )?;
        let result = Self::extract(&point);

        // Check layer hash output against Sinsemilla primitives hash
        #[cfg(test)]
        {
            use crate::constants::{L_ORCHARD_BASE, MERKLE_CRH_PERSONALIZATION};
            use crate::primitives::sinsemilla::HashDomain;
            use ff::PrimeFieldBits;

            if let (Some(left), Some(right)) = (left.value(), right.value()) {
                let l_star = i2lebsp::<10>(l_star as u64);
                let left: Vec<_> = left
                    .to_le_bits()
                    .iter()
                    .by_val()
                    .take(L_ORCHARD_BASE)
                    .collect();
                let right: Vec<_> = right
                    .to_le_bits()
                    .iter()
                    .by_val()
                    .take(L_ORCHARD_BASE)
                    .collect();
                let merkle_crh = HashDomain::new(MERKLE_CRH_PERSONALIZATION);

                let mut message = l_star.to_vec();
                message.extend_from_slice(&left);
                message.extend_from_slice(&right);

                let expected = merkle_crh.hash(message.into_iter()).unwrap();

                assert_eq!(expected.to_bytes(), result.value().unwrap().to_bytes());
            }
        }

        Ok(result)
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
    type RunningSum = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::RunningSum;

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
    type FixedPoints = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::FixedPoints;

    type HashDomains = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::HashDomains;
    type CommitDomains = <SinsemillaChip as SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
    >>::CommitDomains;

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
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        message: Self::Message,
    ) -> Result<(Self::Point, Vec<Self::RunningSum>), Error> {
        let config = self.config().sinsemilla_config.clone();
        let chip = SinsemillaChip::construct(config);
        chip.hash_to_point(layouter, Q, message)
    }

    fn extract(point: &Self::Point) -> Self::X {
        SinsemillaChip::extract(point)
    }
}

#[cfg(test)]
pub mod tests {
    use super::{MerkleChip, MerkleConfig, MerkleInstructions};

    use crate::{
        circuit::gadget::{
            sinsemilla::chip::SinsemillaChip,
            utilities::{UtilitiesInstructions, Var},
        },
        constants::{L_ORCHARD_BASE, MERKLE_CRH_PERSONALIZATION, MERKLE_DEPTH_ORCHARD},
        primitives::sinsemilla::HashDomain,
        spec::i2lebsp,
    };

    use ff::PrimeFieldBits;
    use halo2::{
        arithmetic::FieldExt,
        circuit::{layouter::SingleChipLayouter, Layouter},
        dev::MockProver,
        pasta::pallas,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };

    use rand::random;
    use std::convert::TryInto;

    struct MyCircuit {
        leaf: (Option<pallas::Base>, Option<u32>),
        merkle_path: Vec<Option<pallas::Base>>,
    }

    impl Circuit<pallas::Base> for MyCircuit {
        type Config = (MerkleConfig, MerkleConfig);

        fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
            let advices = [
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
                meta.advice_column(),
            ];

            // Shared fixed column for loading constants
            let constants = meta.fixed_column();
            let perm = meta.permutation(
                &advices
                    .iter()
                    .map(|advice| (*advice).into())
                    .chain(Some(constants.into()))
                    .collect::<Vec<_>>(),
            );

            // Fixed columns for the Sinsemilla generator lookup table
            let lookup = (
                meta.fixed_column(),
                meta.fixed_column(),
                meta.fixed_column(),
            );

            let sinsemilla_config_1 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                lookup,
                constants,
                perm.clone(),
            );
            let config1 = MerkleChip::configure(meta, sinsemilla_config_1);

            let sinsemilla_config_2 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                lookup,
                constants,
                perm,
            );
            let config2 = MerkleChip::configure(meta, sinsemilla_config_2);

            (config1, config2)
        }

        fn synthesize(
            &self,
            cs: &mut impl Assignment<pallas::Base>,
            config: Self::Config,
        ) -> Result<(), Error> {
            let mut layouter = SingleChipLayouter::new(cs)?;

            // Load generator table (shared across both configs)
            SinsemillaChip::load(config.0.sinsemilla_config.clone(), &mut layouter)?;

            // Construct Merkle chips which will be placed side-by-side in the circuit.
            let merkle_chip_1 = MerkleChip::construct(config.0.clone());
            let merkle_chip_2 = MerkleChip::construct(config.1.clone());

            // Process lo half of the Merkle path from leaf to intermediate root.
            let leaf = merkle_chip_1.load_private(
                layouter.namespace(|| ""),
                config.0.cond_swap_config.a,
                self.leaf.0,
            )?;
            let pos_lo = self
                .leaf
                .1
                .map(|pos| pos & ((1 << (MERKLE_DEPTH_ORCHARD / 2)) - 1));

            let intermediate_root = merkle_chip_1.hash_path(
                layouter.namespace(|| ""),
                0,
                (leaf, pos_lo),
                self.merkle_path[0..(MERKLE_DEPTH_ORCHARD / 2)].to_vec(),
            )?;

            // Process hi half of the Merkle path from intermediate root to root.
            let pos_hi = self.leaf.1.map(|pos| pos >> (MERKLE_DEPTH_ORCHARD / 2));

            let computed_final_root = merkle_chip_2.hash_path(
                layouter.namespace(|| ""),
                MERKLE_DEPTH_ORCHARD / 2,
                (intermediate_root, pos_hi),
                self.merkle_path[(MERKLE_DEPTH_ORCHARD / 2)..].to_vec(),
            )?;

            // The expected final root
            let pos_bool = i2lebsp::<32>(self.leaf.1.unwrap() as u64);
            let path: Option<Vec<pallas::Base>> = self.merkle_path.to_vec().into_iter().collect();
            let final_root = hash_path(0, self.leaf.0.unwrap(), &pos_bool, &path.unwrap());

            // Check the computed final root against the expected final root.
            assert_eq!(computed_final_root.value().unwrap(), final_root);

            Ok(())
        }
    }

    fn hash_path(
        offset: usize,
        leaf: pallas::Base,
        pos_bool: &[bool],
        path: &[pallas::Base],
    ) -> pallas::Base {
        let domain = HashDomain::new(MERKLE_CRH_PERSONALIZATION);

        // Compute the root
        let mut node = leaf;
        for (l_star, (sibling, pos)) in path.iter().zip(pos_bool.iter()).enumerate() {
            let l_star = l_star + offset;

            let (left, right) = if *pos {
                (*sibling, node)
            } else {
                (node, *sibling)
            };

            let l_star = i2lebsp::<10>(l_star as u64);
            let left: Vec<_> = left
                .to_le_bits()
                .iter()
                .by_val()
                .take(L_ORCHARD_BASE)
                .collect();
            let right: Vec<_> = right
                .to_le_bits()
                .iter()
                .by_val()
                .take(L_ORCHARD_BASE)
                .collect();

            let mut message = l_star.to_vec();
            message.extend_from_slice(&left);
            message.extend_from_slice(&right);

            node = domain.hash(message.into_iter()).unwrap();
        }
        node
    }

    #[test]
    fn merkle_chip() {
        // Choose a random leaf and position
        let leaf = pallas::Base::rand();
        let pos = random::<u32>();
        let pos_bool = i2lebsp::<32>(pos as u64);

        // Choose a path of random inner nodes
        let path: Vec<_> = (0..(MERKLE_DEPTH_ORCHARD))
            .map(|_| pallas::Base::rand())
            .collect();

        // This root is provided as a public input in the Orchard circuit.
        let _root = hash_path(0, leaf, &pos_bool, &path);

        let circuit = MyCircuit {
            leaf: (Some(leaf), Some(pos)),
            merkle_path: path.into_iter().map(Some).collect(),
        };

        let prover = MockProver::run(11, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }
}
