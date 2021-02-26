//! Gadget and chips for the Sinsemilla hash function.
use std::fmt;

use halo2::{
    arithmetic::CurveAffine,
    circuit::{Chip, Layouter},
    plonk::Error,
};

/// The set of circuit instructions required to use the [`Sinsemilla`](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) gadget.
pub trait SinsemillaInstructions<C: CurveAffine>: Chip<Field = C::Base> {
    /// Witnessed message.
    type Message: Clone + fmt::Debug;
    /// Variable representing an elliptic curve point.
    type Point: Clone + fmt::Debug;
    /// Variable representing the x-coordinate of a point.
    type X: Clone + fmt::Debug;

    /// Witnesses a message in the form of a bitstring.
    fn witness_message(
        layouter: &mut impl Layouter<Self>,
        message: Vec<bool>,
    ) -> Result<Self::Message, Error>;

    /// Extracts the x-coordinate from a curve point.
    fn extract(point: &Self::Point) -> Self::X;

    #[allow(non_snake_case)]
    /// Loads the starting `Q` point for a Sinsemilla hash.
    fn load_Q(domain_prefix: &str) -> Result<Self::Point, Error>;

    /// Hashes a message to a curve point.
    fn hash_to_point(
        layouter: &mut impl Layouter<Self>,
        domain_prefix: &str,
        message: Self::Message,
    ) -> Result<Self::Point, Error>;

    /// Extracts the x-coordinate from the result of `hash_to_point`.
    fn hash(
        layouter: &mut impl Layouter<Self>,
        domain_prefix: &str,
        message: Self::Message,
    ) -> Result<Self::X, Error> {
        Self::hash_to_point(layouter, domain_prefix, message).map(|point| Self::extract(&point))
    }

    /// Returns a curve point that is a commitment to a message.
    fn commit(domain_prefix: &str, msg: Self::Message, r: &C::Scalar)
        -> Result<Self::Point, Error>;

    /// Extracts the x-coordinate from a commitment to a message.
    fn short_commit(
        domain_prefix: &str,
        msg: Self::Message,
        r: &C::Scalar,
    ) -> Result<Self::X, Error> {
        Self::commit(domain_prefix, msg, r).map(|point| Self::extract(&point))
    }
}
