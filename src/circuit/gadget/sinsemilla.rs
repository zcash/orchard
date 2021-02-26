//! Gadget and chips for the Sinsemilla hash function.
use halo2::{
    arithmetic::CurveAffine,
    circuit::{Chip, Layouter},
    plonk::Error,
};

/// The set of circuit instructions required to use the [`Sinsemilla`](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html) gadget.
pub trait SinsemillaInstructions<C: CurveAffine>: Chip<Field = C::Base> {
    type Message: IntoIterator<Item = bool>;

    fn extract(point: &C::Curve) -> C::Base;

    #[allow(non_snake_case)]
    fn Q(domain_prefix: &str) -> C::CurveExt;

    fn hash_to_point(
        layouter: &mut impl Layouter<Self>,
        domain_prefix: &str,
        message: Self::Message,
    ) -> Result<C, Error>;

    fn hash(
        layouter: &mut impl Layouter<Self>,
        domain_prefix: &str,
        message: Self::Message,
    ) -> Result<C::Base, Error>;

    fn commit(domain_prefix: &str, msg: Self::Message, r: &C::Scalar)
        -> Result<C::CurveExt, Error>;

    fn short_commit(
        domain_prefix: &str,
        msg: Self::Message,
        r: &C::Scalar,
    ) -> Result<C::Base, Error>;
}
