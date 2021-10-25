//! Gadget and chips for the Sinsemilla hash function.
use crate::{
    ecc::{self, EccInstructions, FixedPoints},
    utilities::Var,
};
use ff::PrimeField;
use halo2::{circuit::Layouter, plonk::Error};
use pasta_curves::arithmetic::{CurveAffine, FieldExt};
use std::{convert::TryInto, fmt::Debug};

pub mod chip;
pub mod merkle;
mod message;

/// The set of circuit instructions required to use the [`Sinsemilla`](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) gadget.
/// This trait is bounded on two constant parameters: `K`, the number of bits
/// in each word accepted by the Sinsemilla hash, and `MAX_WORDS`, the maximum
/// number of words that a single hash instance can process.
pub trait SinsemillaInstructions<C: CurveAffine, const K: usize, const MAX_WORDS: usize> {
    /// A variable in the circuit.
    type CellValue: Var<C::Base>;

    /// A message composed of [`Self::MessagePiece`]s.
    type Message: From<Vec<Self::MessagePiece>>;

    /// A piece in a message containing a number of `K`-bit words.
    /// A [`Self::MessagePiece`] fits in a single base field element,
    /// which means it can only contain up to `N` words, where
    /// `N*K <= C::Base::NUM_BITS`.
    ///
    /// For example, in the case `K = 10`, `NUM_BITS = 255`, we can fit
    /// up to `N = 25` words in a single base field element.
    type MessagePiece: Copy + Clone + Debug;

    /// A cumulative sum `z` is used to decompose a Sinsemilla message. It
    /// produces intermediate values for each word in the message, such
    /// that `z_next` = (`z_cur` - `word_next`) / `2^K`.
    ///
    /// These intermediate values are useful for range checks on subsets
    /// of the Sinsemilla message. Sinsemilla messages in the Orchard
    /// protocol are composed of field elements, and we need to check
    /// the canonicity of the field element encodings in certain cases.
    type RunningSum;

    /// The x-coordinate of a point output of [`Self::hash_to_point`].
    type X;
    /// A point output of [`Self::hash_to_point`].
    type NonIdentityPoint: Clone + Debug;
    /// A type enumerating the fixed points used in `CommitDomains`.
    type FixedPoints: FixedPoints<C>;

    /// HashDomains used in this instruction.
    type HashDomains: HashDomains<C>;
    /// CommitDomains used in this instruction.
    type CommitDomains: CommitDomains<C, Self::FixedPoints, Self::HashDomains>;

    /// Witness a message piece given a field element. Returns a [`Self::MessagePiece`]
    /// encoding the given message.
    ///
    /// # Panics
    ///
    /// Panics if `num_words` exceed the maximum number of `K`-bit words that
    /// can fit into a single base field element.
    fn witness_message_piece(
        &self,
        layouter: impl Layouter<C::Base>,
        value: Option<C::Base>,
        num_words: usize,
    ) -> Result<Self::MessagePiece, Error>;

    /// Hashes a message to an ECC curve point.
    /// This returns both the resulting point, as well as the message
    /// decomposition in the form of intermediate values in a cumulative
    /// sum.
    ///
    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    fn hash_to_point(
        &self,
        layouter: impl Layouter<C::Base>,
        Q: C,
        message: Self::Message,
    ) -> Result<(Self::NonIdentityPoint, Vec<Self::RunningSum>), Error>;

    /// Extracts the x-coordinate of the output of a Sinsemilla hash.
    fn extract(point: &Self::NonIdentityPoint) -> Self::X;
}

/// A message to be hashed.
///
/// Composed of [`MessagePiece`]s with bitlength some multiple of `K`.
///
/// [`MessagePiece`]: SinsemillaInstructions::MessagePiece
#[derive(Clone, Debug)]
pub struct Message<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    chip: SinsemillaChip,
    inner: SinsemillaChip::Message,
}

impl<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
    Message<C, SinsemillaChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    /// Constructs a message from a bitstring.
    pub fn from_bitstring(
        chip: SinsemillaChip,
        mut layouter: impl Layouter<C::Base>,
        bitstring: Vec<Option<bool>>,
    ) -> Result<Self, Error> {
        // Message must be composed of `K`-bit words.
        assert_eq!(bitstring.len() % K, 0);

        // Message must have at most `MAX_WORDS` words.
        assert!(bitstring.len() / K <= MAX_WORDS);

        // Message piece must be at most `ceil(C::NUM_BITS / K)` bits
        let piece_num_words = C::Base::NUM_BITS as usize / K;
        let pieces: Result<Vec<_>, _> = bitstring
            .chunks(piece_num_words * K)
            .enumerate()
            .map(
                |(i, piece)| -> Result<MessagePiece<C, SinsemillaChip, K, MAX_WORDS>, Error> {
                    MessagePiece::from_bitstring(
                        chip.clone(),
                        layouter.namespace(|| format!("message piece {}", i)),
                        piece,
                    )
                },
            )
            .collect();

        pieces.map(|pieces| Self::from_pieces(chip, pieces))
    }

    /// Constructs a message from a vector of [`MessagePiece`]s.
    ///
    /// [`MessagePiece`]: SinsemillaInstructions::MessagePiece
    pub fn from_pieces(
        chip: SinsemillaChip,
        pieces: Vec<MessagePiece<C, SinsemillaChip, K, MAX_WORDS>>,
    ) -> Self {
        Self {
            chip,
            inner: pieces
                .into_iter()
                .map(|piece| piece.inner)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MessagePiece<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    chip: SinsemillaChip,
    inner: SinsemillaChip::MessagePiece,
}

impl<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
    MessagePiece<C, SinsemillaChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    /// Returns the inner MessagePiece contained in this gadget.
    pub fn inner(&self) -> SinsemillaChip::MessagePiece {
        self.inner
    }
}

impl<C: CurveAffine, SinsemillaChip, const K: usize, const MAX_WORDS: usize>
    MessagePiece<C, SinsemillaChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
{
    pub fn from_bitstring(
        chip: SinsemillaChip,
        layouter: impl Layouter<C::Base>,
        bitstring: &[Option<bool>],
    ) -> Result<Self, Error> {
        // Message must be composed of `K`-bit words.
        assert_eq!(bitstring.len() % K, 0);
        let num_words = bitstring.len() / K;

        // Message piece must be at most `ceil(C::Base::NUM_BITS / K)` bits
        let piece_max_num_words = C::Base::NUM_BITS as usize / K;
        assert!(num_words <= piece_max_num_words as usize);

        // Closure to parse a bitstring (little-endian) into a base field element.
        let to_base_field = |bits: &[Option<bool>]| -> Option<C::Base> {
            assert!(bits.len() <= C::Base::NUM_BITS as usize);

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
            bytes.map(|bytes| C::Base::from_bytes(&bytes.try_into().unwrap()).unwrap())
        };

        let piece_value = to_base_field(bitstring);
        Self::from_field_elem(chip, layouter, piece_value, num_words)
    }

    pub fn from_field_elem(
        chip: SinsemillaChip,
        layouter: impl Layouter<C::Base>,
        field_elem: Option<C::Base>,
        num_words: usize,
    ) -> Result<Self, Error> {
        let inner = chip.witness_message_piece(layouter, field_elem, num_words)?;
        Ok(Self { chip, inner })
    }
}

/// A domain in which $\mathsf{SinsemillaHashToPoint}$ and $\mathsf{SinsemillaHash}$ can
/// be used.
#[allow(non_snake_case)]
pub struct HashDomain<
    C: CurveAffine,
    SinsemillaChip,
    EccChip,
    const K: usize,
    const MAX_WORDS: usize,
> where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
        > + Clone
        + Debug
        + Eq,
{
    sinsemilla_chip: SinsemillaChip,
    ecc_chip: EccChip,
    Q: C,
}

impl<C: CurveAffine, SinsemillaChip, EccChip, const K: usize, const MAX_WORDS: usize>
    HashDomain<C, SinsemillaChip, EccChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
        > + Clone
        + Debug
        + Eq,
{
    #[allow(non_snake_case)]
    /// Constructs a new `HashDomain` for the given domain.
    pub fn new(
        sinsemilla_chip: SinsemillaChip,
        ecc_chip: EccChip,
        domain: &SinsemillaChip::HashDomains,
    ) -> Self {
        HashDomain {
            sinsemilla_chip,
            ecc_chip,
            Q: domain.Q(),
        }
    }

    #[allow(clippy::type_complexity)]
    /// $\mathsf{SinsemillaHashToPoint}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash
    pub fn hash_to_point(
        &self,
        layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<(ecc::NonIdentityPoint<C, EccChip>, Vec<SinsemillaChip::RunningSum>), Error> {
        assert_eq!(self.sinsemilla_chip, message.chip);
        self.sinsemilla_chip
            .hash_to_point(layouter, self.Q, message.inner)
            .map(|(point, zs)| (ecc::NonIdentityPoint::from_inner(self.ecc_chip.clone(), point), zs))
    }

    /// $\mathsf{SinsemillaHash}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash
    #[allow(clippy::type_complexity)]
    pub fn hash(
        &self,
        layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
    ) -> Result<(ecc::X<C, EccChip>, Vec<SinsemillaChip::RunningSum>), Error> {
        assert_eq!(self.sinsemilla_chip, message.chip);
        let (p, zs) = self.hash_to_point(layouter, message)?;
        Ok((p.extract_p(), zs))
    }
}

/// Trait allowing circuit's Sinsemilla CommitDomains to be enumerated.
pub trait CommitDomains<C: CurveAffine, F: FixedPoints<C>, H: HashDomains<C>>:
    Clone + Debug + Eq
{
    /// Returns the fixed point corresponding to the R constant used for
    /// randomization in this CommitDomain.
    fn r(&self) -> F;

    /// Returns the HashDomain contained in this CommitDomain
    fn hash_domain(&self) -> H;
}

/// Trait allowing circuit's Sinsemilla HashDomains to be enumerated.
#[allow(non_snake_case)]
pub trait HashDomains<C: CurveAffine>: Clone + Debug + Eq {
    fn Q(&self) -> C;
}

#[allow(non_snake_case)]
pub struct CommitDomain<
    C: CurveAffine,
    SinsemillaChip,
    EccChip,
    const K: usize,
    const MAX_WORDS: usize,
> where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
        > + Clone
        + Debug
        + Eq,
{
    M: HashDomain<C, SinsemillaChip, EccChip, K, MAX_WORDS>,
    R: ecc::FixedPoint<C, EccChip>,
}

impl<C: CurveAffine, SinsemillaChip, EccChip, const K: usize, const MAX_WORDS: usize>
    CommitDomain<C, SinsemillaChip, EccChip, K, MAX_WORDS>
where
    SinsemillaChip: SinsemillaInstructions<C, K, MAX_WORDS> + Clone + Debug + Eq,
    EccChip: EccInstructions<
            C,
            NonIdentityPoint = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::NonIdentityPoint,
            FixedPoints = <SinsemillaChip as SinsemillaInstructions<C, K, MAX_WORDS>>::FixedPoints,
        > + Clone
        + Debug
        + Eq,
{
    /// Constructs a new `CommitDomain` for the given domain.
    pub fn new(
        sinsemilla_chip: SinsemillaChip,
        ecc_chip: EccChip,
        // Instead of using SinsemilllaChip::CommitDomains, just use something that implements a CommitDomains trait
        domain: &SinsemillaChip::CommitDomains,
    ) -> Self {
        CommitDomain {
            M: HashDomain::new(sinsemilla_chip, ecc_chip.clone(), &domain.hash_domain()),
            R: ecc::FixedPoint::from_inner(ecc_chip, domain.r()),
        }
    }

    #[allow(clippy::type_complexity)]
    /// $\mathsf{SinsemillaCommit}$ from [§ 5.4.8.4][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub fn commit(
        &self,
        mut layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
        r: Option<C::Scalar>,
    ) -> Result<
        (
            ecc::Point<C, EccChip>,
            Vec<SinsemillaChip::RunningSum>,
        ),
        Error,
    > {
        assert_eq!(self.M.sinsemilla_chip, message.chip);
        let (blind, _) = self.R.mul(layouter.namespace(|| "[r] R"), r)?;
        let (p, zs) = self.M.hash_to_point(layouter.namespace(|| "M"), message)?;
        let commitment = p.add(layouter.namespace(|| "M + [r] R"), &blind)?;
        Ok((commitment, zs))
    }

    #[allow(clippy::type_complexity)]
    /// $\mathsf{SinsemillaShortCommit}$ from [§ 5.4.8.4][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub fn short_commit(
        &self,
        mut layouter: impl Layouter<C::Base>,
        message: Message<C, SinsemillaChip, K, MAX_WORDS>,
        r: Option<C::Scalar>,
    ) -> Result<(ecc::X<C, EccChip>, Vec<SinsemillaChip::RunningSum>), Error> {
        assert_eq!(self.M.sinsemilla_chip, message.chip);
        let (p, zs) = self.commit(layouter.namespace(|| "commit"), message, r)?;
        Ok((p.extract_p(), zs))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        ecc::{
            chip::{compute_lagrange_coeffs, find_zs_and_us, EccChip, EccConfig, NUM_WINDOWS},
            FixedPoints, NonIdentityPoint, H,
        },
        primitives::sinsemilla,
        sinsemilla::{
            chip::{SinsemillaChip, SinsemillaConfig},
            CommitDomain, CommitDomains, HashDomain, HashDomains, Message,
        },
        utilities::lookup_range_check::LookupRangeCheckConfig,
    };

    use halo2::{
        circuit::{Layouter, SimpleFloorPlanner},
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use group::{prime::PrimeCurveAffine, Curve};
    use std::convert::TryInto;

    use lazy_static::lazy_static;

    lazy_static! {
        static ref PERSONALIZATION: &'static str = "personalization";
        static ref COMMIT_DOMAIN: sinsemilla::CommitDomain =
            sinsemilla::CommitDomain::new(*PERSONALIZATION);
        static ref Q: pallas::Affine = COMMIT_DOMAIN.Q().to_affine();
        static ref R: pallas::Affine = COMMIT_DOMAIN.R().to_affine();
        static ref ZS_AND_US: Vec<(u64, [[u8; 32]; H])> = find_zs_and_us(*R, NUM_WINDOWS).unwrap();
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct FixedBase;
    impl FixedPoints<pallas::Affine> for FixedBase {
        fn generator(&self) -> pallas::Affine {
            *R
        }

        fn u(&self) -> Vec<[[u8; 32]; H]> {
            ZS_AND_US.iter().map(|(_, us)| *us).collect()
        }

        fn z(&self) -> Vec<u64> {
            ZS_AND_US.iter().map(|(z, _)| *z).collect()
        }

        fn lagrange_coeffs(&self) -> Vec<[pallas::Base; H]> {
            compute_lagrange_coeffs(self.generator(), NUM_WINDOWS)
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Hash;
    impl HashDomains<pallas::Affine> for Hash {
        fn Q(&self) -> pallas::Affine {
            *Q
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Commit;
    impl CommitDomains<pallas::Affine, FixedBase, Hash> for Commit {
        fn r(&self) -> FixedBase {
            FixedBase
        }

        fn hash_domain(&self) -> Hash {
            Hash
        }
    }

    pub struct MyCircuit;

    impl Circuit<pallas::Base> for MyCircuit {
        type Config = (
            EccConfig,
            SinsemillaConfig<Hash, Commit, FixedBase>,
            SinsemillaConfig<Hash, Commit, FixedBase>,
        );
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            MyCircuit
        }

        #[allow(non_snake_case)]
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
            meta.enable_constant(constants);

            let table_idx = meta.lookup_table_column();
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

            // Fixed columns for the Sinsemilla generator lookup table
            let lookup = (
                table_idx,
                meta.lookup_table_column(),
                meta.lookup_table_column(),
            );

            let range_check = LookupRangeCheckConfig::configure(meta, advices[9], table_idx);

            let ecc_config = EccChip::<FixedBase>::configure(
                meta,
                advices,
                lagrange_coeffs,
                range_check.clone(),
            );

            let config1 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                advices[2],
                lagrange_coeffs[0],
                lookup,
                range_check.clone(),
            );
            let config2 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                advices[7],
                lagrange_coeffs[1],
                lookup,
                range_check,
            );
            (ecc_config, config1, config2)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<pallas::Base>,
        ) -> Result<(), Error> {
            let ecc_chip = EccChip::construct(config.0);

            // The two `SinsemillaChip`s share the same lookup table.
            SinsemillaChip::<Hash, Commit, FixedBase>::load(config.1.clone(), &mut layouter)?;

            // Test hash domain.
            {
                let chip1 = SinsemillaChip::construct(config.1.clone());

                let hash_domain = HashDomain::new(chip1.clone(), ecc_chip.clone(), &Hash);

                let message: Vec<Option<bool>> =
                    (0..500).map(|_| Some(rand::random::<bool>())).collect();

                let (result, _) = {
                    let message = Message::from_bitstring(
                        chip1,
                        layouter.namespace(|| "witness message"),
                        message.clone(),
                    )?;
                    hash_domain.hash_to_point(layouter.namespace(|| "hash"), message)?
                };

                let expected_result = {
                    let message: Option<Vec<bool>> = message.into_iter().collect();
                    let expected_result = if let Some(message) = message {
                        let point = sinsemilla::HashDomain {
                            Q: hash_domain.Q.to_curve(),
                        }
                        .hash_to_point(message.into_iter())
                        .unwrap();
                        Some(point.to_affine())
                    } else {
                        None
                    };

                    NonIdentityPoint::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "Witness expected result"),
                        expected_result,
                    )?
                };

                result.constrain_equal(
                    layouter.namespace(|| "result == expected result"),
                    &expected_result,
                )?;
            }

            // Test commit domain.
            {
                let chip2 = SinsemillaChip::construct(config.2.clone());

                let domain = Commit;
                let commit_domain = CommitDomain::new(chip2.clone(), ecc_chip.clone(), &domain);

                let r_val = pallas::Scalar::rand();
                let message: Vec<Option<bool>> =
                    (0..500).map(|_| Some(rand::random::<bool>())).collect();

                let (result, _) = {
                    let message = Message::from_bitstring(
                        chip2,
                        layouter.namespace(|| "witness message"),
                        message.clone(),
                    )?;
                    commit_domain.commit(layouter.namespace(|| "commit"), message, Some(r_val))?
                };

                // Witness expected result.
                let expected_result = {
                    let message: Option<Vec<bool>> = message.into_iter().collect();
                    let expected_result = if let Some(message) = message {
                        let point = sinsemilla::CommitDomain {
                            M: sinsemilla::HashDomain {
                                Q: domain.hash_domain().Q().to_curve(),
                            },
                            R: domain.r().generator().to_curve(),
                        }
                        .commit(message.into_iter(), &r_val)
                        .unwrap();
                        Some(point.to_affine())
                    } else {
                        None
                    };

                    NonIdentityPoint::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "Witness expected result"),
                        expected_result,
                    )?
                };

                result.constrain_equal(
                    layouter.namespace(|| "result == expected result"),
                    &expected_result,
                )?;
            }

            Ok(())
        }
    }

    #[test]
    fn sinsemilla_chip() {
        use halo2::dev::MockProver;

        let k = 11;
        let circuit = MyCircuit;
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()))
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_sinsemilla_chip() {
        use plotters::prelude::*;

        let root =
            BitMapBackend::new("sinsemilla-hash-layout.png", (1024, 7680)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("SinsemillaHash", ("sans-serif", 60)).unwrap();

        let circuit = MyCircuit;
        halo2::dev::CircuitLayout::default()
            .render(11, &circuit, &root)
            .unwrap();
    }
}
