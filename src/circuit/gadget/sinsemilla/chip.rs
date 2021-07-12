use super::{
    message::{Message, MessagePiece, MessageSubPiece},
    CommitDomains, HashDomains, SinsemillaInstructions,
};
use crate::{
    circuit::gadget::{
        ecc::chip::EccPoint,
        utilities::{CellValue, Var},
    },
    constants::OrchardFixedBasesFull,
    primitives::sinsemilla::{
        self, Q_COMMIT_IVK_M_GENERATOR, Q_MERKLE_CRH, Q_NOTE_COMMITMENT_M_GENERATOR,
    },
};

use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Selector},
    poly::Rotation,
};
use pasta_curves::pallas;

use std::convert::TryInto;

mod generator_table;
pub use generator_table::get_s_by_idx;
use generator_table::GeneratorTableConfig;

mod hash_to_point;

/// Configuration for the Sinsemilla hash chip
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaConfig {
    // Selector used in the lookup argument as well as Sinsemilla custom gates.
    q_sinsemilla1: Selector,
    // Fixed column used in Sinsemilla custom gates.
    q_sinsemilla2: Column<Fixed>,
    // Fixed column used to constrain hash initialization to be consistent with
    // the y-coordinate of the domain Q.
    fixed_y_q: Column<Fixed>,
    // Advice column used to store the x-coordinate of the accumulator at each
    // iteration of the hash.
    x_a: Column<Advice>,
    // Advice column used to store the x-coordinate of the generator corresponding
    // to the message word at each iteration of the hash. This is looked up in the
    // generator table.
    x_p: Column<Advice>,
    // Advice column used to load the message.
    bits: Column<Advice>,
    // Advice column used to store the lambda_1 intermediate value at each
    // iteration.
    lambda_1: Column<Advice>,
    // Advice column used to store the lambda_2 intermediate value at each
    // iteration.
    lambda_2: Column<Advice>,
    // The lookup table where (idx, x_p, y_p) are loaded for the 2^K generators
    // of the Sinsemilla hash.
    pub generator_table: GeneratorTableConfig,
    // Fixed column shared by the whole circuit. This is used to load the
    // x-coordinate of the domain Q, which is then constrained to equal the
    // initial x_a.
    constants: Column<Fixed>,
}

impl SinsemillaConfig {
    pub fn advices(&self) -> [Column<Advice>; 5] {
        [self.bits, self.x_a, self.x_p, self.lambda_1, self.lambda_2]
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SinsemillaChip {
    config: SinsemillaConfig,
}

impl Chip<pallas::Base> for SinsemillaChip {
    type Config = SinsemillaConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl SinsemillaChip {
    pub fn construct(config: <Self as Chip<pallas::Base>>::Config) -> Self {
        Self { config }
    }

    pub fn load(
        config: SinsemillaConfig,
        layouter: &mut impl Layouter<pallas::Base>,
    ) -> Result<<Self as Chip<pallas::Base>>::Loaded, Error> {
        // Load the lookup table.
        config.generator_table.load(layouter)
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(non_snake_case)]
    pub fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 5],
        lookup: (Column<Fixed>, Column<Fixed>, Column<Fixed>),
        constants: Column<Fixed>,
    ) -> <Self as Chip<pallas::Base>>::Config {
        let config = SinsemillaConfig {
            q_sinsemilla1: meta.selector(),
            q_sinsemilla2: meta.fixed_column(),
            fixed_y_q: meta.fixed_column(),
            x_a: advices[0],
            x_p: advices[1],
            bits: advices[2],
            lambda_1: advices[3],
            lambda_2: advices[4],
            generator_table: GeneratorTableConfig {
                table_idx: lookup.0,
                table_x: lookup.1,
                table_y: lookup.2,
            },
            constants,
        };

        let two = Expression::Constant(pallas::Base::from_u64(2));
        // Check that the initial x_A, x_P, lambda_1, lambda_2 are consistent with y_Q.
        meta.create_gate("Initial y_Q", |meta| {
            let fixed_y_q = meta.query_fixed(config.fixed_y_q, Rotation::cur());

            // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
            let Y_A = {
                let lambda_1 = meta.query_advice(config.lambda_1, Rotation::cur());
                let lambda_2 = meta.query_advice(config.lambda_2, Rotation::cur());
                let x_a = meta.query_advice(config.x_a, Rotation::cur());

                // x_r = lambda_1^2 - x_a - x_p
                let x_r = {
                    let x_p = meta.query_advice(config.x_p, Rotation::cur());
                    lambda_1.clone().square() - x_a.clone() - x_p
                };

                (lambda_1 + lambda_2) * (x_a - x_r)
            };

            // fixed_y_q * (2 * fixed_y_q - Y_{A,0}) = 0
            vec![fixed_y_q.clone() * (two.clone() * fixed_y_q - Y_A)]
        });

        meta.create_gate("Secant line", |meta| {
            let q_s1 = meta.query_selector(config.q_sinsemilla1);
            let lambda_1 = meta.query_advice(config.lambda_1, Rotation::cur());
            let lambda_2 = meta.query_advice(config.lambda_2, Rotation::cur());
            let x_a_cur = meta.query_advice(config.x_a, Rotation::cur());
            let x_a_next = meta.query_advice(config.x_a, Rotation::next());

            // x_r = lambda_1^2 - x_a_cur - x_p
            let x_r = {
                let x_p = meta.query_advice(config.x_p, Rotation::cur());
                lambda_1.square() - x_a_cur.clone() - x_p
            };

            // lambda2^2 - (x_a_next + x_r + x_a_cur) = 0
            let secant_line = lambda_2.square() - (x_a_next + x_r + x_a_cur);

            vec![q_s1 * secant_line]
        });

        meta.create_gate("Sinsemilla gate", |meta| {
            let q_s1 = meta.query_selector(config.q_sinsemilla1);
            let q_s2 = meta.query_fixed(config.q_sinsemilla2, Rotation::cur());
            let q_s3 = {
                let one = Expression::Constant(pallas::Base::one());
                q_s2.clone() * (q_s2 - one)
            };
            let x_a_cur = meta.query_advice(config.x_a, Rotation::cur());
            let x_a_next = meta.query_advice(config.x_a, Rotation::next());
            let lambda_1_cur = meta.query_advice(config.lambda_1, Rotation::cur());
            let lambda_2_cur = meta.query_advice(config.lambda_2, Rotation::cur());
            let lambda_1_next = meta.query_advice(config.lambda_1, Rotation::next());
            let lambda_2_next = meta.query_advice(config.lambda_2, Rotation::next());

            // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
            let Y_A_cur = {
                // x_r = lambda_1^2 - x_a - x_p
                let x_r = {
                    let x_p = meta.query_advice(config.x_p, Rotation::cur());
                    lambda_1_cur.clone().square() - x_a_cur.clone() - x_p
                };

                (lambda_1_cur + lambda_2_cur.clone()) * (x_a_cur.clone() - x_r)
            };

            // Y_A = (lambda_1 + lambda_2) * (x_a - x_r)
            let Y_A_next = {
                // x_r = lambda_1^2 - x_a - x_p
                let x_r = {
                    let x_p = meta.query_advice(config.x_p, Rotation::next());
                    lambda_1_next.clone().square() - x_a_next.clone() - x_p
                };

                (lambda_1_next.clone() + lambda_2_next) * (x_a_next.clone() - x_r)
            };

            // lhs = 4 * lambda_2_cur * (x_a_cur - x_a_next)
            let lhs = lambda_2_cur * pallas::Base::from_u64(4) * (x_a_cur - x_a_next);

            // rhs = 2 * Y_A_cur + (2 - q_s3) * Y_A_next + 2 * q_s3 * y_a_final
            let rhs = {
                // y_a_final is assigned to the lambda1 column on the next offset.
                let y_a_final = lambda_1_next;

                two.clone() * Y_A_cur
                    + (two.clone() - q_s3.clone()) * Y_A_next
                    + two * q_s3 * y_a_final
            };

            vec![q_s1 * (lhs - rhs)]
        });

        config
    }
}

// Implement `SinsemillaInstructions` for `SinsemillaChip`
impl SinsemillaInstructions<pallas::Affine, { sinsemilla::K }, { sinsemilla::C }>
    for SinsemillaChip
{
    type CellValue = CellValue<pallas::Base>;

    type Message = Message<pallas::Base, { sinsemilla::K }, { sinsemilla::C }>;
    type MessagePiece = MessagePiece<pallas::Base, { sinsemilla::K }>;
    type MessageSubPiece = MessageSubPiece<pallas::Base, { sinsemilla::K }>;

    type RunningSum = Vec<Self::CellValue>;

    type X = CellValue<pallas::Base>;
    type Point = EccPoint;
    type FixedPoints = OrchardFixedBasesFull;

    type HashDomains = SinsemillaHashDomains;
    type CommitDomains = SinsemillaCommitDomains;

    #[allow(non_snake_case)]
    fn witness_message(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        message: Vec<Option<bool>>,
    ) -> Result<Self::Message, Error> {
        // Message must be composed of `K`-bit words.
        assert_eq!(message.len() % sinsemilla::K, 0);

        // Message must have at most `sinsemilla::C` words.
        assert!(message.len() / sinsemilla::K <= sinsemilla::C);

        // Message piece must be at most `ceil(pallas::Base::NUM_BITS / sinsemilla::K)` bits
        let piece_num_words = pallas::Base::NUM_BITS as usize / sinsemilla::K;
        let pieces: Result<Vec<_>, _> = message
            .chunks(piece_num_words * sinsemilla::K)
            .enumerate()
            .map(|(i, piece)| -> Result<Self::MessagePiece, Error> {
                self.witness_message_piece_bitstring(
                    layouter.namespace(|| format!("message piece {}", i)),
                    piece,
                )
            })
            .collect();

        pieces.map(|pieces| pieces.into())
    }

    #[allow(non_snake_case)]
    fn witness_message_piece_bitstring(
        &self,
        layouter: impl Layouter<pallas::Base>,
        message_piece: &[Option<bool>],
    ) -> Result<Self::MessagePiece, Error> {
        // Message must be composed of `K`-bit words.
        assert_eq!(message_piece.len() % sinsemilla::K, 0);
        let num_words = message_piece.len() / sinsemilla::K;

        // Message piece must be at most `ceil(C::Base::NUM_BITS / sinsemilla::K)` bits
        let piece_max_num_words = pallas::Base::NUM_BITS as usize / sinsemilla::K;
        assert!(num_words <= piece_max_num_words as usize);

        // Closure to parse a bitstring (little-endian) into a base field element.
        let to_base_field = |bits: &[Option<bool>]| -> Option<pallas::Base> {
            assert!(bits.len() <= pallas::Base::NUM_BITS as usize);

            let bits: Option<Vec<bool>> = bits.iter().cloned().collect();
            let bytes: Option<Vec<u8>> = bits.map(|bits| {
                // Pad bits to 256 bits
                let pad_len = 256 - bits.len();
                let mut bits = bits;
                bits.extend_from_slice(&vec![false; pad_len]);

                bits.chunks_exact(8)
                    .map(|byte| byte.iter().rev().fold(0u8, |acc, bit| acc * 2 + *bit as u8))
                    .collect()
            });
            bytes.map(|bytes| pallas::Base::from_bytes(&bytes.try_into().unwrap()).unwrap())
        };

        let piece_value = to_base_field(message_piece);
        self.witness_message_piece_field(layouter, piece_value, num_words)
    }

    fn witness_message_piece_field(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        field_elem: Option<pallas::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().clone();
        let cell_value = {
            let cell = layouter.assign_region(
                || "witness message piece",
                |mut region| {
                    region.assign_advice(
                        || "witness message piece",
                        config.bits,
                        0,
                        || field_elem.ok_or(Error::SynthesisError),
                    )
                },
            )?;
            CellValue::new(cell, field_elem)
        };

        Ok(MessagePiece::new(cell_value, num_words, None))
    }

    fn witness_message_piece_subpieces(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        subpieces: &[Self::MessageSubPiece],
    ) -> Result<Self::MessagePiece, Error> {
        let config = self.config().clone();

        let total_num_bits = subpieces
            .iter()
            .map(|subpiece| subpiece.bit_range().len())
            .sum::<usize>();

        // The subpieces must form a message composed of whole `K`-bit words.
        assert!(total_num_bits % sinsemilla::K == 0);

        // The message must fit inside a single field element.
        let num_words = total_num_bits / sinsemilla::K;
        let piece_max_num_words = pallas::Base::NUM_BITS as usize / sinsemilla::K;
        assert!(num_words <= piece_max_num_words as usize);

        // Witness subpieces
        let subpieces = layouter.assign_region(
            || "Witness subpieces",
            |mut region| {
                let advices = config.advices();
                let subpieces: Result<Vec<Self::MessageSubPiece>, Error> = subpieces
                    .iter()
                    .enumerate()
                    .map(|(idx, subpiece)| -> Result<Self::MessageSubPiece, Error> {
                        let column = advices[idx % advices.len()];
                        let offset = idx / advices.len();

                        let value = subpiece.field_elem_subset();
                        let cell = region.assign_advice(
                            || format!("subpiece {:?}", idx),
                            column,
                            offset,
                            || value.ok_or(Error::SynthesisError),
                        )?;
                        let cell_value = CellValue::new(cell, value);
                        Ok((cell_value, subpiece.bit_range()).into())
                    })
                    .collect();

                subpieces
            },
        )?;

        let message_piece = {
            // The field element representing the complete message piece.
            let value = MessageSubPiece::assemble_field_elem(&subpieces);

            // Witness message piece.
            let cell = layouter.assign_region(
                || "witness message piece",
                |mut region| {
                    region.assign_advice(
                        || "witness message piece",
                        config.bits,
                        0,
                        || value.ok_or(Error::SynthesisError),
                    )
                },
            )?;
            CellValue::new(cell, value)
        };

        Ok(MessagePiece::new(message_piece, num_words, Some(subpieces)))
    }

    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        Q: pallas::Affine,
        message: Self::Message,
    ) -> Result<(Self::Point, Vec<Self::RunningSum>), Error> {
        layouter.assign_region(
            || "hash_to_point",
            |mut region| self.hash_message(&mut region, Q, &message),
        )
    }

    fn extract(point: &Self::Point) -> Self::X {
        point.x()
    }
}

#[derive(Clone, Debug)]
pub enum SinsemillaHashDomains {
    NoteCommit,
    CommitIvk,
    MerkleCrh,
}

#[allow(non_snake_case)]
impl HashDomains<pallas::Affine> for SinsemillaHashDomains {
    fn Q(&self) -> pallas::Affine {
        match self {
            SinsemillaHashDomains::CommitIvk => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_COMMIT_IVK_M_GENERATOR.0).unwrap(),
                pallas::Base::from_bytes(&Q_COMMIT_IVK_M_GENERATOR.1).unwrap(),
            )
            .unwrap(),
            SinsemillaHashDomains::NoteCommit => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_NOTE_COMMITMENT_M_GENERATOR.0).unwrap(),
                pallas::Base::from_bytes(&Q_NOTE_COMMITMENT_M_GENERATOR.1).unwrap(),
            )
            .unwrap(),
            SinsemillaHashDomains::MerkleCrh => pallas::Affine::from_xy(
                pallas::Base::from_bytes(&Q_MERKLE_CRH.0).unwrap(),
                pallas::Base::from_bytes(&Q_MERKLE_CRH.1).unwrap(),
            )
            .unwrap(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SinsemillaCommitDomains {
    NoteCommit,
    CommitIvk,
}

impl CommitDomains<pallas::Affine, OrchardFixedBasesFull, SinsemillaHashDomains>
    for SinsemillaCommitDomains
{
    fn r(&self) -> OrchardFixedBasesFull {
        match self {
            Self::NoteCommit => OrchardFixedBasesFull::NoteCommitR,
            Self::CommitIvk => OrchardFixedBasesFull::CommitIvkR,
        }
    }

    fn hash_domain(&self) -> SinsemillaHashDomains {
        match self {
            Self::NoteCommit => SinsemillaHashDomains::NoteCommit,
            Self::CommitIvk => SinsemillaHashDomains::CommitIvk,
        }
    }
}
