//! A minimal RedPallas implementation for use in Zcash.

use core::cmp::{Ord, Ordering, PartialOrd};

use group::{Group as _, GroupEncoding as _};
use pasta_curves::pallas;
use rand::{CryptoRng, RngCore};

#[cfg(feature = "std")]
pub use reddsa::batch;

#[cfg(test)]
use rand::rngs::OsRng;

/// A RedPallas signature type.
pub trait SigType: reddsa::SigType + private::Sealed {}

/// A type variable corresponding to an Orchard spend authorization signature.
pub type SpendAuth = reddsa::orchard::SpendAuth;
impl SigType for SpendAuth {}

/// A type variable corresponding to an Orchard binding signature.
pub type Binding = reddsa::orchard::Binding;
impl SigType for Binding {}

/// A RedPallas signing key.
#[derive(Clone, Debug)]
pub struct SigningKey<T: SigType>(reddsa::SigningKey<T>);

impl<T: SigType> From<SigningKey<T>> for [u8; 32] {
    fn from(sk: SigningKey<T>) -> [u8; 32] {
        sk.0.into()
    }
}

impl<T: SigType> From<&SigningKey<T>> for [u8; 32] {
    fn from(sk: &SigningKey<T>) -> [u8; 32] {
        sk.0.into()
    }
}

impl<T: SigType> TryFrom<[u8; 32]> for SigningKey<T> {
    type Error = reddsa::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        bytes.try_into().map(SigningKey)
    }
}

impl SigningKey<SpendAuth> {
    /// Randomizes this signing key with the given `randomizer`.
    ///
    /// Randomization is only supported for `SpendAuth` keys.
    pub fn randomize(&self, randomizer: &pallas::Scalar) -> Self {
        SigningKey(self.0.randomize(randomizer))
    }
}

impl<T: SigType> SigningKey<T> {
    /// Creates a signature of type `T` on `msg` using this `SigningKey`.
    pub fn sign<R: RngCore + CryptoRng>(&self, rng: R, msg: &[u8]) -> Signature<T> {
        Signature(self.0.sign(rng, msg))
    }
}

/// A RedPallas verification key.
#[derive(Clone, Debug)]
pub struct VerificationKey<T: SigType>(reddsa::VerificationKey<T>);

impl<T: SigType> From<VerificationKey<T>> for [u8; 32] {
    fn from(vk: VerificationKey<T>) -> [u8; 32] {
        vk.0.into()
    }
}

impl<T: SigType> From<&VerificationKey<T>> for [u8; 32] {
    fn from(vk: &VerificationKey<T>) -> [u8; 32] {
        vk.0.into()
    }
}

impl TryFrom<[u8; 32]> for VerificationKey<SpendAuth> {
    type Error = reddsa::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        // Do not permit construction of `VerificationKey<SpendAuth>` if the underlying
        // `pallas::Point` is the identity. Note that `pallas::Point::from_bytes` decodes
        // the all-zeros encoding successfully (as the identity), so we must explicitly
        // check `is_identity()` rather than relying on the decoding to fail.
        let maybe_point = pallas::Point::from_bytes(&bytes);
        if bool::from(
            maybe_point
                .unwrap_or(pallas::Point::identity())
                .is_identity(),
        ) {
            Err(reddsa::Error::MalformedVerificationKey)
        } else {
            reddsa::VerificationKey::try_from(bytes).map(Self)
        }
    }
}

impl TryFrom<[u8; 32]> for VerificationKey<Binding> {
    type Error = reddsa::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        reddsa::VerificationKey::try_from(bytes).map(Self)
    }
}

impl<'a, T: SigType> From<&'a SigningKey<T>> for VerificationKey<T> {
    fn from(sk: &'a SigningKey<T>) -> VerificationKey<T> {
        VerificationKey((&sk.0).into())
    }
}

impl<T: SigType> PartialEq for VerificationKey<T> {
    fn eq(&self, other: &Self) -> bool {
        <[u8; 32]>::from(self).eq(&<[u8; 32]>::from(other))
    }
}

impl<T: SigType> Eq for VerificationKey<T> {}

impl<T: SigType> PartialOrd for VerificationKey<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: SigType> Ord for VerificationKey<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        <[u8; 32]>::from(self).cmp(&<[u8; 32]>::from(other))
    }
}

impl VerificationKey<SpendAuth> {
    /// Used in the note encryption tests.
    #[cfg(test)]
    pub(crate) fn dummy() -> Self {
        VerificationKey((&reddsa::SigningKey::new(OsRng)).into())
    }

    /// Randomizes this verification key with the given `randomizer`.
    ///
    /// Randomization is only supported for `SpendAuth` keys.
    ///
    /// Returns `None` if the [`pallas::Point`] corresponding to the randomized verification
    /// key is the identity.
    pub fn randomize(&self, randomizer: &pallas::Scalar) -> Option<Self> {
        let randomized = self.0.randomize(randomizer);
        let bytes = <[u8; 32]>::from(randomized);
        let point: pallas::Point = Option::from(pallas::Point::from_bytes(&bytes))
            .expect("VerificationKey bytes are always a canonical Pallas point encoding");

        if bool::from(point.is_identity()) {
            None
        } else {
            Some(VerificationKey(randomized))
        }
    }

    /// Creates a batch validation item from a `SpendAuth` signature.
    #[cfg(feature = "std")]
    pub fn create_batch_item<M: AsRef<[u8]>>(
        &self,
        sig: Signature<SpendAuth>,
        msg: &M,
    ) -> batch::Item<SpendAuth, Binding> {
        batch::Item::from_spendauth(self.0.into(), sig.0, msg)
    }
}

#[cfg(feature = "std")]
impl VerificationKey<Binding> {
    /// Creates a batch validation item from a `Binding` signature.
    pub fn create_batch_item<M: AsRef<[u8]>>(
        &self,
        sig: Signature<Binding>,
        msg: &M,
    ) -> batch::Item<SpendAuth, Binding> {
        batch::Item::from_binding(self.0.into(), sig.0, msg)
    }
}

impl<T: SigType> VerificationKey<T> {
    /// Verifies a purported `signature` over `msg` made by this verification key.
    pub fn verify(&self, msg: &[u8], signature: &Signature<T>) -> Result<(), reddsa::Error> {
        self.0.verify(msg, &signature.0)
    }
}

/// A RedPallas signature.
#[derive(Debug, Clone)]
pub struct Signature<T: SigType>(reddsa::Signature<T>);

impl<T: SigType> From<[u8; 64]> for Signature<T> {
    fn from(bytes: [u8; 64]) -> Self {
        Signature(bytes.into())
    }
}

impl<T: SigType> From<&Signature<T>> for [u8; 64] {
    fn from(sig: &Signature<T>) -> Self {
        sig.0.into()
    }
}

pub(crate) mod private {
    use super::{Binding, SpendAuth};

    pub trait Sealed {}

    impl Sealed for SpendAuth {}

    impl Sealed for Binding {}
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use proptest::prelude::*;

    use super::{Binding, SigningKey, SpendAuth, VerificationKey};

    prop_compose! {
        /// Generate a uniformly distributed RedDSA spend authorization signing key.
        pub fn arb_spendauth_signing_key()(
            sk in prop::array::uniform32(prop::num::u8::ANY)
                .prop_map(reddsa::SigningKey::try_from)
                .prop_filter("Values must be parseable as valid signing keys", |r| r.is_ok())
        ) -> SigningKey<SpendAuth> {
            SigningKey(sk.unwrap())
        }
    }

    prop_compose! {
        /// Generate a uniformly distributed RedDSA binding signing key.
        pub fn arb_binding_signing_key()(
            sk in prop::array::uniform32(prop::num::u8::ANY)
                .prop_map(reddsa::SigningKey::try_from)
                .prop_filter("Values must be parseable as valid signing keys", |r| r.is_ok())
        ) -> SigningKey<Binding> {
            SigningKey(sk.unwrap())
        }
    }

    prop_compose! {
        /// Generate a uniformly distributed RedDSA spend authorization verification key.
        pub fn arb_spendauth_verification_key()(sk in arb_spendauth_signing_key()) -> VerificationKey<SpendAuth> {
            VerificationKey::from(&sk)
        }
    }

    prop_compose! {
        /// Generate a uniformly distributed RedDSA binding verification key.
        pub fn arb_binding_verification_key()(sk in arb_binding_signing_key()) -> VerificationKey<Binding> {
            VerificationKey::from(&sk)
        }
    }
}

#[cfg(test)]
mod tests {
    use group::ff::{Field, PrimeField};
    use pasta_curves::pallas;

    use super::{Binding, SigningKey, SpendAuth, VerificationKey};

    #[test]
    fn try_from_identity_bytes_is_rejected() {
        // The all-zeros encoding is the canonical encoding of the identity point on Pallas.
        // `VerificationKey<SpendAuth>` must reject it, since an identity `rk` would allow
        // forgery of spend authorization signatures.
        let result = VerificationKey::<SpendAuth>::try_from([0u8; 32]);
        assert!(
            result.is_err(),
            "VerificationKey::<SpendAuth>::try_from([0u8; 32]) must reject the identity",
        );
    }

    #[test]
    fn binding_try_from_identity_bytes_is_accepted() {
        // Identity verification keys are permitted for `Binding` (Orchard uses a
        // prime-order group, and the RedDSA specification allows identity verification
        // keys; rejection is specific to `SpendAuth`, where an identity `rk` would permit
        // forgery of spend authorization signatures).
        let result = VerificationKey::<Binding>::try_from([0u8; 32]);
        assert!(
            result.is_ok(),
            "VerificationKey::<Binding>::try_from([0u8; 32]) must accept the identity",
        );
    }

    #[test]
    fn spendauth_randomize_to_identity_returns_none() {
        // Construct a `VerificationKey<SpendAuth>` corresponding to the
        // signing key with scalar 1 (so, equal to the `SpendAuthSig` base
        // point `G`), then randomize by `alpha = -1`. The resulting key is
        // `rk = [1] G + [-1] G = identity`, which must be rejected.
        let ask_bytes: [u8; 32] = pallas::Scalar::ONE.to_repr().into();
        let ask = <SigningKey<SpendAuth>>::try_from(ask_bytes).expect("1 is a valid scalar");
        let ak = <VerificationKey<SpendAuth>>::from(&ask);
        let alpha = -pallas::Scalar::ONE;
        assert!(
            ak.randomize(&alpha).is_none(),
            "randomizing the SpendAuthSig basepoint by -1 produces the identity, which must be rejected",
        );
    }
}
