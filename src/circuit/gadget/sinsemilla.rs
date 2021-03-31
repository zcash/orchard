//! Gadget and chips for the Sinsemilla hash function.
use std::fmt;

use halo2::{arithmetic::CurveAffine, circuit::Layouter, plonk::Error};

use crate::circuit::gadget::ecc::{self, EccInstructions};

// mod chip;
// pub use chip::{SinsemillaChip, SinsemillaColumns, SinsemillaConfig};

/// Trait allowing circuit's Sinsemilla domains to be enumerated.
pub trait Domains<C: CurveAffine, F: ecc::FixedPoints<C>>: Clone + fmt::Debug {
    /// Returns the fixed point corresponding to the R constant for this domain.
    fn get_r(&self) -> F;
}

/// The set of circuit instructions required to use the [`Sinsemilla`](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) gadget.
pub trait SinsemillaInstructions<C: CurveAffine>: EccInstructions<C> {
    /// Witnessed message.
    type Message: Clone + fmt::Debug;
    /// Variable representing the set of fixed bases in the circuit.
    type Domains: Domains<C, <Self as EccInstructions<C>>::FixedPoints>;
    /// Variable representing a Q fixed point for a domain.
    type Q: Clone + fmt::Debug;

    /// Gets the Q constant for the given domain.
    #[allow(non_snake_case)]
    fn get_Q(layouter: &mut impl Layouter<Self>, domain: &Self::Domains) -> Result<Self::Q, Error>;

    /// Witnesses a message in the form of a bitstring.
    fn witness_message(
        layouter: &mut impl Layouter<Self>,
        message: Vec<bool>,
    ) -> Result<Self::Message, Error>;

    /// Move to ECC chip
    /// Extracts the x-coordinate from a curve point.
    fn extract(point: &Self::Point) -> Self::X;

    /// Hashes a message to an ECC curve point.
    #[allow(non_snake_case)]
    fn hash_to_point(
        layouter: &mut impl Layouter<Self>,
        Q: &Self::Q,
        message: Self::Message,
    ) -> Result<Self::Point, Error>;
}

#[allow(non_snake_case)]
pub struct HashDomain<C: CurveAffine, SinsemillaChip: SinsemillaInstructions<C>> {
    Q: SinsemillaChip::Q,
}

impl<C: CurveAffine, SinsemillaChip: SinsemillaInstructions<C>> HashDomain<C, SinsemillaChip> {
    #[allow(non_snake_case)]
    /// Constructs a new `CommitDomain` for the given domain.
    pub fn new(
        mut layouter: impl Layouter<SinsemillaChip>,
        domain: &SinsemillaChip::Domains,
    ) -> Result<Self, Error> {
        SinsemillaChip::get_Q(&mut layouter, domain).map(|Q| HashDomain { Q })
    }

    /// $\mathsf{SinsemillaHashToPoint}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash
    pub fn hash_to_point(
        &self,
        mut layouter: impl Layouter<SinsemillaChip>,
        message: <SinsemillaChip as SinsemillaInstructions<C>>::Message,
    ) -> Result<ecc::Point<C, SinsemillaChip>, Error> {
        SinsemillaChip::hash_to_point(&mut layouter, &self.Q, message).map(ecc::Point::from_inner)
    }

    /// $\mathsf{SinsemillaHash}$ from [§ 5.4.1.9][concretesinsemillahash].
    ///
    /// [concretesinsemillahash]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillahash
    pub fn hash(
        &self,
        layouter: impl Layouter<SinsemillaChip>,
        message: <SinsemillaChip as SinsemillaInstructions<C>>::Message,
    ) -> Result<ecc::X<C, SinsemillaChip>, Error> {
        let p = self.hash_to_point(layouter, message);
        p.map(|p| p.extract_p())
    }
}

#[allow(non_snake_case)]
pub struct CommitDomain<C: CurveAffine, SinsemillaChip: SinsemillaInstructions<C>> {
    M: HashDomain<C, SinsemillaChip>,
    R: ecc::FixedPoint<C, SinsemillaChip>,
}

impl<C: CurveAffine, SinsemillaChip: SinsemillaInstructions<C>> CommitDomain<C, SinsemillaChip> {
    /// Constructs a new `CommitDomain` for the given domain.
    pub fn new(
        mut layouter: impl Layouter<SinsemillaChip>,
        domain: &SinsemillaChip::Domains,
    ) -> Result<Self, Error> {
        Ok(CommitDomain {
            M: HashDomain::new(layouter.namespace(|| "M"), domain)?,
            R: ecc::FixedPoint::get(layouter.namespace(|| "R"), domain.get_r())?,
        })
    }

    /// $\mathsf{SinsemillaCommit}$ from [§ 5.4.8.4][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub fn commit(
        &self,
        mut layouter: impl Layouter<SinsemillaChip>,
        message: <SinsemillaChip as SinsemillaInstructions<C>>::Message,
        r: ecc::ScalarFixed<C, SinsemillaChip>,
    ) -> Result<ecc::Point<C, SinsemillaChip>, Error> {
        let blind = self.R.mul(layouter.namespace(|| "[r] R"), &r)?;
        self.M
            .hash_to_point(layouter.namespace(|| "M"), message)?
            .add(layouter.namespace(|| "M + [r] R"), &blind)
    }

    /// $\mathsf{SinsemillaShortCommit}$ from [§ 5.4.8.4][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub fn short_commit(
        &self,
        mut layouter: impl Layouter<SinsemillaChip>,
        message: <SinsemillaChip as SinsemillaInstructions<C>>::Message,
        r: ecc::ScalarFixed<C, SinsemillaChip>,
    ) -> Result<ecc::X<C, SinsemillaChip>, Error> {
        let p = self.commit(layouter.namespace(|| "commit"), message, r);
        p.map(|p| p.extract_p())
    }
}
