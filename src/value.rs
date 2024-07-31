//! Monetary values within the Orchard shielded pool.
//!
//! Values are represented in three places within the Orchard protocol:
//! - [`NoteValue`], the value of an individual note. It is an unsigned 64-bit integer
//!   (with maximum value [`MAX_NOTE_VALUE`]), and is serialized in a note plaintext.
//! - [`ValueSum`], the sum of note values within an Orchard [`Action`] or [`Bundle`].
//!   It is a signed 64-bit integer (with range [`VALUE_SUM_RANGE`]).
//! - `valueBalanceOrchard`, which is a signed 63-bit integer. This is represented
//!    by a user-defined type parameter on [`Bundle`], returned by
//!    [`Bundle::value_balance`] and [`Builder::value_balance`].
//!
//! If your specific instantiation of the Orchard protocol requires a smaller bound on
//! valid note values (for example, Zcash's `MAX_MONEY` fits into a 51-bit integer), you
//! should enforce this in two ways:
//!
//! - Define your `valueBalanceOrchard` type to enforce your valid value range. This can
//!   be checked in its `TryFrom<i64>` implementation.
//! - Define your own "amount" type for note values, and convert it to `NoteValue` prior
//!   to calling [`Builder::add_output`].
//!
//! Inside the circuit, note values are constrained to be unsigned 64-bit integers.
//!
//! # Caution!
//!
//! An `i64` is _not_ a signed 64-bit integer! The [Rust documentation] calls `i64` the
//! 64-bit signed integer type, which is true in the sense that its encoding in memory
//! takes up 64 bits. Numerically, however, `i64` is a signed 63-bit integer.
//!
//! Fortunately, users of this crate should never need to construct [`ValueSum`] directly;
//! you should only need to interact with [`NoteValue`] (which can be safely constructed
//! from a `u64`) and `valueBalanceOrchard` (which can be represented as an `i64`).
//!
//! [`Action`]: crate::action::Action
//! [`Bundle`]: crate::bundle::Bundle
//! [`Bundle::value_balance`]: crate::bundle::Bundle::value_balance
//! [`Builder::value_balance`]: crate::builder::Builder::value_balance
//! [`Builder::add_output`]: crate::builder::Builder::add_output
//! [Rust documentation]: https://doc.rust-lang.org/stable/std/primitive.i64.html

use core::fmt::{self, Debug};
use core::iter::Sum;
use core::ops::{Add, RangeInclusive, Sub};
use std::ops::Neg;

use bitvec::{array::BitArray, order::Lsb0};
use ff::{Field, PrimeField};
use group::{Curve, Group, GroupEncoding};
use halo2_proofs::plonk::Assigned;
use pasta_curves::{
    arithmetic::{CurveAffine, CurveExt},
    pallas,
};
use rand::RngCore;
use subtle::CtOption;

use crate::{
    constants::fixed_bases::{VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_R_BYTES},
    primitives::redpallas::{self, Binding},
};

use crate::builder::BuildError;
use crate::note::AssetBase;

/// Maximum note value.
pub const MAX_NOTE_VALUE: u64 = u64::MAX;

/// The valid range of the scalar multiplication used in ValueCommit^Orchard.
///
/// Defined in a note in [Zcash Protocol Spec § 4.17.4: Action Statement (Orchard)][actionstatement].
///
/// [actionstatement]: https://zips.z.cash/protocol/nu5.pdf#actionstatement
pub const VALUE_SUM_RANGE: RangeInclusive<i128> =
    -(MAX_NOTE_VALUE as i128)..=MAX_NOTE_VALUE as i128;

/// A value operation overflowed.
#[derive(Debug)]
pub struct OverflowError;

impl fmt::Display for OverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Orchard value operation overflowed")
    }
}

impl std::error::Error for OverflowError {}

/// The non-negative value of an individual Orchard note.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoteValue(u64);

impl NoteValue {
    pub(crate) fn zero() -> Self {
        // Default for u64 is zero.
        Default::default()
    }

    /// Returns the raw underlying value.
    pub fn inner(&self) -> u64 {
        self.0
    }

    /// Creates a note value from its raw numeric value.
    ///
    /// This only enforces that the value is an unsigned 64-bit integer. Callers should
    /// enforce any additional constraints on the value's valid range themselves.
    pub fn from_raw(value: u64) -> Self {
        NoteValue(value)
    }

    /// Creates a note value from a byte array.
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        NoteValue(u64::from_le_bytes(bytes))
    }

    /// Converts the note value to a byte array.
    pub fn to_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub(crate) fn to_le_bits(self) -> BitArray<[u8; 8], Lsb0> {
        BitArray::<_, Lsb0>::new(self.0.to_le_bytes())
    }

    /// The minimum, greater than zero, note value that can not be split further.
    pub fn unsplittable() -> Self {
        NoteValue(1u64)
    }
}

impl From<&NoteValue> for Assigned<pallas::Base> {
    fn from(v: &NoteValue) -> Self {
        pallas::Base::from(v.inner()).into()
    }
}

impl From<NoteValue> for i128 {
    fn from(value: NoteValue) -> Self {
        value.0 as i128
    }
}

impl Sub for NoteValue {
    type Output = ValueSum;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self::Output {
        let a = self.0 as i128;
        let b = rhs.0 as i128;
        a.checked_sub(b)
            .filter(|v| VALUE_SUM_RANGE.contains(v))
            .map(ValueSum)
            .expect("u64 - u64 result is always in VALUE_SUM_RANGE")
    }
}

pub(crate) enum Sign {
    Positive,
    Negative,
}

/// A sum of Orchard note values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValueSum(i128);

impl ValueSum {
    pub(crate) fn zero() -> Self {
        // Default for i128 is zero.
        Default::default()
    }

    /// Creates a value sum from a raw i64 (which is always in range for this type).
    ///
    /// This only enforces that the value is a signed 63-bit integer. We use it internally
    /// in `Bundle::binding_validating_key`, where we are converting from the user-defined
    /// `valueBalance` type that enforces any additional constraints on the value's valid
    /// range.
    pub(crate) fn from_raw(value: i64) -> Self {
        ValueSum(value as i128)
    }

    /// Splits this value sum into its magnitude and sign.
    pub(crate) fn magnitude_sign(&self) -> (u64, Sign) {
        let (magnitude, sign) = if self.0.is_negative() {
            (-self.0, Sign::Negative)
        } else {
            (self.0, Sign::Positive)
        };
        (
            u64::try_from(magnitude)
                .expect("ValueSum magnitude is in range for u64 by construction"),
            sign,
        )
    }

    pub(crate) fn into<V: TryFrom<i64>>(self) -> Result<V, BuildError> {
        i64::try_from(self)
            .map_err(BuildError::ValueSum)
            .and_then(|i| V::try_from(i).map_err(|_| BuildError::ValueSum(OverflowError)))
    }
}

impl<T: Into<i128>> Add<T> for ValueSum {
    type Output = Option<ValueSum>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: T) -> Self::Output {
        self.0
            .checked_add(rhs.into())
            .filter(|v| VALUE_SUM_RANGE.contains(v))
            .map(ValueSum)
    }
}

impl Neg for ValueSum {
    type Output = Option<ValueSum>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn neg(self) -> Self::Output {
        self.0
            .checked_neg()
            .filter(|v| VALUE_SUM_RANGE.contains(v))
            .map(ValueSum)
    }
}

impl<'a> Sum<&'a ValueSum> for Result<ValueSum, OverflowError> {
    fn sum<I: Iterator<Item = &'a ValueSum>>(mut iter: I) -> Self {
        iter.try_fold(ValueSum(0), |acc, v| acc + *v)
            .ok_or(OverflowError)
    }
}

impl Sum<ValueSum> for Result<ValueSum, OverflowError> {
    fn sum<I: Iterator<Item = ValueSum>>(mut iter: I) -> Self {
        iter.try_fold(ValueSum(0), |acc, v| acc + v)
            .ok_or(OverflowError)
    }
}

impl TryFrom<ValueSum> for i64 {
    type Error = OverflowError;

    fn try_from(v: ValueSum) -> Result<i64, Self::Error> {
        i64::try_from(v.0).map_err(|_| OverflowError)
    }
}

impl From<ValueSum> for i128 {
    fn from(value: ValueSum) -> Self {
        value.0
    }
}

impl From<NoteValue> for ValueSum {
    fn from(value: NoteValue) -> Self {
        Self(value.into())
    }
}

/// The blinding factor for a [`ValueCommitment`].
#[derive(Clone, Copy, Debug)]
pub struct ValueCommitTrapdoor(pallas::Scalar);

impl ValueCommitTrapdoor {
    pub(crate) fn inner(&self) -> pallas::Scalar {
        self.0
    }

    /// Constructs `ValueCommitTrapdoor` from the byte representation of a scalar.
    /// Returns a `None` [`CtOption`] if `bytes` is not a canonical representation
    /// of a Pallas scalar.
    ///
    /// This is a low-level API, requiring a detailed understanding of the
    /// [use of value commitment trapdoors][orchardbalance] in the Zcash protocol
    /// to use correctly and securely. It is intended to be used in combination
    /// with [`ValueCommitment::derive`].
    ///
    /// [orchardbalance]: https://zips.z.cash/protocol/protocol.pdf#orchardbalance
    pub fn from_bytes(bytes: [u8; 32]) -> CtOption<Self> {
        pallas::Scalar::from_repr(bytes).map(ValueCommitTrapdoor)
    }
}

impl Add<&ValueCommitTrapdoor> for ValueCommitTrapdoor {
    type Output = ValueCommitTrapdoor;

    fn add(self, rhs: &Self) -> Self::Output {
        ValueCommitTrapdoor(self.0 + rhs.0)
    }
}

impl<'a> Sum<&'a ValueCommitTrapdoor> for ValueCommitTrapdoor {
    fn sum<I: Iterator<Item = &'a ValueCommitTrapdoor>>(iter: I) -> Self {
        iter.fold(ValueCommitTrapdoor::zero(), |acc, cv| acc + cv)
    }
}

impl ValueCommitTrapdoor {
    /// Generates a new value commitment trapdoor.
    pub(crate) fn random(rng: impl RngCore) -> Self {
        ValueCommitTrapdoor(pallas::Scalar::random(rng))
    }

    /// Returns the zero trapdoor, which provides no blinding.
    pub(crate) fn zero() -> Self {
        ValueCommitTrapdoor(pallas::Scalar::zero())
    }

    pub(crate) fn into_bsk(self) -> redpallas::SigningKey<Binding> {
        // TODO: impl From<pallas::Scalar> for redpallas::SigningKey.
        self.0.to_repr().try_into().unwrap()
    }
}

/// A commitment to a [`ValueSum`].
#[derive(Clone, Debug)]
pub struct ValueCommitment(pallas::Point);

impl Add<&ValueCommitment> for ValueCommitment {
    type Output = ValueCommitment;

    fn add(self, rhs: &Self) -> Self::Output {
        ValueCommitment(self.0 + rhs.0)
    }
}

impl Sub for ValueCommitment {
    type Output = ValueCommitment;

    fn sub(self, rhs: Self) -> Self::Output {
        ValueCommitment(self.0 - rhs.0)
    }
}

impl Sum for ValueCommitment {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ValueCommitment(pallas::Point::identity()), |acc, cv| {
            acc + &cv
        })
    }
}

impl<'a> Sum<&'a ValueCommitment> for ValueCommitment {
    fn sum<I: Iterator<Item = &'a ValueCommitment>>(iter: I) -> Self {
        iter.fold(ValueCommitment(pallas::Point::identity()), |acc, cv| {
            acc + cv
        })
    }
}

impl ValueCommitment {
    /// Derives a `ValueCommitment` by $\mathsf{ValueCommit^{Orchard}}$.
    ///
    /// Defined in [Zcash Protocol Spec § 5.4.8.3: Homomorphic Pedersen commitments (Sapling and Orchard)][concretehomomorphiccommit].
    ///
    /// [concretehomomorphiccommit]: https://zips.z.cash/protocol/nu5.pdf#concretehomomorphiccommit
    #[allow(non_snake_case)]
    pub fn derive(value: ValueSum, rcv: ValueCommitTrapdoor, asset: AssetBase) -> Self {
        let hasher = pallas::Point::hash_to_curve(VALUE_COMMITMENT_PERSONALIZATION);
        let R = hasher(&VALUE_COMMITMENT_R_BYTES);
        let abs_value = u64::try_from(value.0.abs()).expect("value must be in valid range");

        let value = if value.0.is_negative() {
            -pallas::Scalar::from(abs_value)
        } else {
            pallas::Scalar::from(abs_value)
        };

        let V_zsa = asset.cv_base();

        ValueCommitment(V_zsa * value + R * rcv.0)
    }

    pub(crate) fn into_bvk(self) -> redpallas::VerificationKey<Binding> {
        // TODO: impl From<pallas::Point> for redpallas::VerificationKey.
        self.0.to_bytes().try_into().unwrap()
    }

    /// Deserialize a value commitment from its byte representation
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<ValueCommitment> {
        pallas::Point::from_bytes(bytes).map(ValueCommitment)
    }

    /// Serialize this value commitment to its canonical byte representation.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// x-coordinate of this value commitment.
    pub(crate) fn x(&self) -> pallas::Base {
        if self.0 == pallas::Point::identity() {
            pallas::Base::zero()
        } else {
            *self.0.to_affine().coordinates().unwrap().x()
        }
    }

    /// y-coordinate of this value commitment.
    pub(crate) fn y(&self) -> pallas::Base {
        if self.0 == pallas::Point::identity() {
            pallas::Base::zero()
        } else {
            *self.0.to_affine().coordinates().unwrap().y()
        }
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use group::ff::FromUniformBytes;
    use pasta_curves::pallas;
    use proptest::prelude::*;

    use super::{NoteValue, ValueCommitTrapdoor, ValueSum, MAX_NOTE_VALUE, VALUE_SUM_RANGE};

    prop_compose! {
        /// Generate an arbitrary Pallas scalar.
        pub fn arb_scalar()(bytes in prop::array::uniform32(0u8..)) -> pallas::Scalar {
            // Instead of rejecting out-of-range bytes, let's reduce them.
            let mut buf = [0; 64];
            buf[..32].copy_from_slice(&bytes);
            pallas::Scalar::from_uniform_bytes(&buf)
        }
    }

    prop_compose! {
        /// Generate an arbitrary [`ValueSum`] in the range of valid Zcash values.
        pub fn arb_value_sum()(value in VALUE_SUM_RANGE) -> ValueSum {
            ValueSum(value)
        }
    }

    prop_compose! {
        /// Generate an arbitrary [`ValueSum`] in the range of valid Zcash values.
        pub fn arb_value_sum_bounded(bound: NoteValue)(value in -(bound.0 as i128)..=(bound.0 as i128)) -> ValueSum {
            ValueSum(value)
        }
    }

    prop_compose! {
        /// Generate an arbitrary ValueCommitTrapdoor
        pub fn arb_trapdoor()(rcv in arb_scalar()) -> ValueCommitTrapdoor {
            ValueCommitTrapdoor(rcv)
        }
    }

    prop_compose! {
        /// Generate an arbitrary value in the range of valid nonnegative Zcash amounts.
        pub fn arb_note_value()(value in 0u64..MAX_NOTE_VALUE) -> NoteValue {
            NoteValue(value)
        }
    }

    prop_compose! {
        /// Generate an arbitrary value in the range of valid positive Zcash amounts
        /// less than a specified value.
        pub fn arb_note_value_bounded(max: u64)(value in 0u64..max) -> NoteValue {
            NoteValue(value)
        }
    }

    prop_compose! {
        /// Generate an arbitrary value in the range of valid positive Zcash amounts
        /// less than a specified value.
        pub fn arb_positive_note_value(max: u64)(value in 1u64..max) -> NoteValue {
            NoteValue(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::note::asset_base::testing::{arb_asset_base, native_asset_base};

    use crate::note::AssetBase;
    use proptest::prelude::*;

    use super::{
        testing::{arb_note_value_bounded, arb_trapdoor, arb_value_sum_bounded},
        OverflowError, ValueCommitTrapdoor, ValueCommitment, ValueSum, MAX_NOTE_VALUE,
    };
    use crate::primitives::redpallas;

    fn check_binding_signature(
        native_values: &[(ValueSum, ValueCommitTrapdoor, AssetBase)],
        arb_values: &[(ValueSum, ValueCommitTrapdoor, AssetBase)],
        neg_trapdoors: &[ValueCommitTrapdoor],
        arb_values_to_burn: &[(ValueSum, ValueCommitTrapdoor, AssetBase)],
    ) {
        // for each arb value, create a negative value with a different trapdoor
        let neg_arb_values: Vec<_> = arb_values
            .iter()
            .cloned()
            .zip(neg_trapdoors.iter().cloned())
            .map(|((value, _, asset), rcv)| ((-value).unwrap(), rcv, asset))
            .collect();

        let native_value_balance = native_values
            .iter()
            .map(|(value, _, _)| value)
            .sum::<Result<ValueSum, OverflowError>>()
            .expect("we generate values that won't overflow");

        let values = [
            native_values,
            arb_values,
            &neg_arb_values,
            arb_values_to_burn,
        ]
        .concat();

        let bsk = values
            .iter()
            .map(|(_, rcv, _)| rcv)
            .sum::<ValueCommitTrapdoor>()
            .into_bsk();

        let bvk = (values
            .into_iter()
            .map(|(value, rcv, asset)| ValueCommitment::derive(value, rcv, asset))
            .sum::<ValueCommitment>()
            - ValueCommitment::derive(
                native_value_balance,
                ValueCommitTrapdoor::zero(),
                AssetBase::native(),
            )
            - arb_values_to_burn
                .iter()
                .map(|(value, _, asset)| {
                    ValueCommitment::derive(*value, ValueCommitTrapdoor::zero(), *asset)
                })
                .sum::<ValueCommitment>())
        .into_bvk();

        assert_eq!(redpallas::VerificationKey::from(&bsk), bvk);
    }

    proptest! {
        #[test]
        fn bsk_consistent_with_bvk_native_with_zsa_transfer_and_burning(
            native_values in (1usize..10).prop_flat_map(|n_values|
                arb_note_value_bounded(MAX_NOTE_VALUE / n_values as u64).prop_flat_map(move |bound|
                    prop::collection::vec((arb_value_sum_bounded(bound), arb_trapdoor(), native_asset_base()), n_values)
                )
            ),
            (asset_values, neg_trapdoors) in (1usize..10).prop_flat_map(|n_values|
                (arb_note_value_bounded(MAX_NOTE_VALUE / n_values as u64).prop_flat_map(move |bound|
                    prop::collection::vec((arb_value_sum_bounded(bound), arb_trapdoor(), arb_asset_base()), n_values)
                ), prop::collection::vec(arb_trapdoor(), n_values))
            ),
            burn_values in (1usize..10).prop_flat_map(|n_values|
                arb_note_value_bounded(MAX_NOTE_VALUE / n_values as u64)
                .prop_flat_map(move |bound| prop::collection::vec((arb_value_sum_bounded(bound), arb_trapdoor(), arb_asset_base()), n_values))
            )
        ) {
            check_binding_signature(&native_values, &[], &[], &[]);
            check_binding_signature(&native_values, &[], &[], &burn_values);
            check_binding_signature(&native_values, &asset_values, &neg_trapdoors, &[]);
            check_binding_signature(&native_values, &asset_values, &neg_trapdoors, &burn_values);
        }
    }
}
