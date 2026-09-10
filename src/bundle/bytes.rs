use alloc::vec::Vec;
use core::fmt;

use nonempty::NonEmpty;

use crate::{
    action::{Action, ActionBytes, DecompressionError},
    bundle::{
        validate_flags, validate_proof_size, Authorization, Authorized, Bundle, BundleError,
        BundleVersion, Flags,
    },
    tree::Anchor,
};

/// A [`Bundle`] whose actions are still encoded.
///
/// Upholds every rule a [`Bundle`] does except those needing a point, which
/// [`Self::decompress`] discharges.
#[derive(Clone)]
pub struct BundleBytes<T: Authorization, V> {
    actions: NonEmpty<ActionBytes<T::SpendAuth>>,
    flags: Flags,
    value_balance: V,
    anchor: Anchor,
    authorization: T,
    bundle_version: BundleVersion,
}

impl<T: Authorization, V: fmt::Debug> fmt::Debug for BundleBytes<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Actions<'a, T>(&'a NonEmpty<ActionBytes<T>>);
        impl<T: fmt::Debug> fmt::Debug for Actions<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_struct("BundleBytes")
            .field("actions", &Actions(&self.actions))
            .field("flags", &self.flags)
            .field("value_balance", &self.value_balance)
            .field("anchor", &self.anchor)
            .field("authorization", &self.authorization)
            .field("bundle_version", &self.bundle_version)
            .finish()
    }
}

impl<T: Authorization, V> BundleBytes<T, V> {
    /// Carries the same obligations as [`Bundle::from_parts_unchecked`].
    pub(crate) fn from_parts_unchecked(
        actions: NonEmpty<ActionBytes<T::SpendAuth>>,
        flags: Flags,
        value_balance: V,
        anchor: Anchor,
        authorization: T,
        bundle_version: BundleVersion,
    ) -> Self {
        debug_assert!(flags.to_byte(bundle_version).is_some());
        BundleBytes {
            actions,
            flags,
            value_balance,
            anchor,
            authorization,
            bundle_version,
        }
    }

    /// Returns the encoded actions that make up this bundle.
    pub fn actions(&self) -> &NonEmpty<ActionBytes<T::SpendAuth>> {
        &self.actions
    }

    /// Returns the Orchard-specific transaction-level flags for this bundle.
    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    /// Returns the net value moved into or out of the Orchard shielded pool.
    pub fn value_balance(&self) -> &V {
        &self.value_balance
    }

    /// Returns the root of the Orchard commitment tree that this bundle commits to.
    pub fn anchor(&self) -> &Anchor {
        &self.anchor
    }

    /// Returns the authorization for this bundle.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Returns the [`BundleVersion`] this bundle is encoded under.
    pub fn bundle_version(&self) -> BundleVersion {
        self.bundle_version
    }

    /// Returns this bundle's flag byte, as [`Bundle::flag_byte`] does.
    pub fn flag_byte(&self) -> u8 {
        self.flags
            .to_byte(self.bundle_version)
            .expect("flags are validated against the bundle version at construction")
    }

    /// Recovers the [`Bundle`]. 3 sqrt per action.
    ///
    /// # Errors
    ///
    /// First action that breaks a point rule.
    pub fn decompress(self) -> Result<Bundle<T, V>, BundleDecompressionError> {
        let actions = self
            .actions
            .into_iter()
            .enumerate()
            .map(|(action, a)| {
                a.decompress()
                    .map_err(|error| BundleDecompressionError { action, error })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Bundle::from_parts_unchecked(
            NonEmpty::from_vec(actions).expect("a NonEmpty maps to a NonEmpty"),
            self.flags,
            self.value_balance,
            self.anchor,
            self.authorization,
            self.bundle_version,
        ))
    }
}

impl<V> BundleBytes<Authorized, V> {
    /// Same checks as [`Bundle::try_from_parts`], both decidable without a point.
    ///
    /// # Errors
    ///
    /// [`BundleError::NonCanonicalProofSize`] or [`BundleError::UnrepresentableFlags`].
    pub fn try_from_parts(
        actions: NonEmpty<ActionBytes<<Authorized as Authorization>::SpendAuth>>,
        flags: Flags,
        value_balance: V,
        anchor: Anchor,
        authorization: Authorized,
        bundle_version: BundleVersion,
    ) -> Result<Self, BundleError> {
        if bundle_version.enforces_canonical_proof_size() {
            validate_proof_size(authorization.proof(), actions.len())?;
        }
        validate_flags(&flags, bundle_version)?;
        Ok(BundleBytes::from_parts_unchecked(
            actions,
            flags,
            value_balance,
            anchor,
            authorization,
            bundle_version,
        ))
    }
}

impl<T: Authorization, V> Bundle<T, V> {
    /// Drops to the encoded tier.
    ///
    /// Infallible: a [`Bundle`] cannot hold a point that fails to encode.
    pub fn compress(self) -> BundleBytes<T, V> {
        BundleBytes {
            actions: self.actions.map(Action::compress),
            flags: self.flags,
            value_balance: self.value_balance,
            anchor: self.anchor,
            authorization: self.authorization,
            bundle_version: self.bundle_version,
        }
    }
}

/// Action at bundle-relative index `action` breaks a point rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct BundleDecompressionError {
    /// Index of the offending action.
    pub action: usize,
    /// The rule it breaks.
    pub error: DecompressionError,
}

impl fmt::Display for BundleDecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "action {}: {}", self.action, self.error)
    }
}

impl core::error::Error for BundleDecompressionError {}

#[cfg(all(test, feature = "circuit"))]
mod tests {
    use alloc::vec::Vec;

    use nonempty::NonEmpty;

    use super::BundleBytes;
    use crate::{
        action::DecompressionError,
        bundle::{tests::sample_authorized_bundle, Authorized, Bundle, BundleVersion, Flags},
        ActionBytes, Proof,
    };

    // Not a canonical encoding of any Pallas point
    const NON_CANONICAL: [u8; 32] = [0xff; 32];

    /// `ValueSum` is not `Into<i64>`, which `Bundle`'s commitment APIs require.
    fn sample(n_actions: usize) -> Bundle<Authorized, i64> {
        sample_authorized_bundle(n_actions)
            .try_map_value_balance::<i64, (), _>(|_| Ok(0))
            .expect("the mapping cannot fail")
    }

    fn with_cv_net_replaced(
        bytes: &BundleBytes<Authorized, i64>,
        index: usize,
        cv_net: [u8; 32],
    ) -> BundleBytes<Authorized, i64> {
        let actions = bytes
            .actions()
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let mut encoded = a.to_bytes();
                if i == index {
                    encoded[0..32].copy_from_slice(&cv_net);
                }
                ActionBytes::from_bytes(&encoded)
                    .expect("only cv_net changed")
                    .with_authorization(a.authorization().clone())
            })
            .collect::<Vec<_>>();

        BundleBytes::try_from_parts(
            NonEmpty::from_vec(actions).expect("the bundle is non-empty"),
            *bytes.flags(),
            0,
            *bytes.anchor(),
            bytes.authorization().clone(),
            bytes.bundle_version(),
        )
        .expect("only cv_net changed")
    }

    #[test]
    fn decompress_names_the_offending_action() {
        let bytes = sample(3).compress();
        let tampered = with_cv_net_replaced(&bytes, 2, NON_CANONICAL);

        let err = tampered.decompress().unwrap_err();
        assert_eq!(err.action, 2);
        assert_eq!(err.error, DecompressionError::NonCanonicalValueCommitment);
    }

    /// `bundle_version` is interpretive context, so no encoding would catch its loss.
    #[test]
    fn bundle_round_trips_through_the_parse_tier() {
        let bundle = sample(3);
        let bytes = bundle.clone().compress();
        let encodings: Vec<_> = bytes.actions().iter().map(ActionBytes::to_bytes).collect();

        assert_eq!(bytes.flags(), bundle.flags());
        assert_eq!(bytes.value_balance(), bundle.value_balance());
        assert_eq!(bytes.anchor(), bundle.anchor());
        assert_eq!(bytes.bundle_version(), bundle.bundle_version());

        let recovered = bytes
            .decompress()
            .expect("a built bundle is valid")
            .compress();
        assert_eq!(
            recovered
                .actions()
                .iter()
                .map(ActionBytes::to_bytes)
                .collect::<Vec<_>>(),
            encodings,
        );
    }

    /// The checked constructor must reject exactly what the point tier rejects, notably a
    /// padded or truncated proof (GHSA-2x4w-pxqw-58v9) — before any curve arithmetic.
    #[test]
    fn try_from_parts_matches_the_point_tier() {
        let bundle = sample(3);
        let bytes = bundle.clone().compress();
        let expected = Proof::expected_proof_size(bundle.actions().len());
        // Enforces canonical proof size and accepts any cross-address flag value
        let bundle_version = BundleVersion::ironwood_v3();

        let authorization = |proof_len: usize| {
            Authorized::from_parts(
                Proof::new(alloc::vec![0u8; proof_len]),
                bundle.authorization().binding_signature().clone(),
            )
        };

        for (proof_len, flags) in [
            (expected, *bundle.flags()),
            (expected + 1, *bundle.flags()),
            (expected - 1, *bundle.flags()),
            (expected, Flags::CROSS_ADDRESS_DISABLED),
        ] {
            let on_bytes = BundleBytes::try_from_parts(
                bytes.actions().clone(),
                flags,
                0i64,
                *bytes.anchor(),
                authorization(proof_len),
                bundle_version,
            );
            let on_points = Bundle::try_from_parts(
                bundle.actions().clone(),
                flags,
                0i64,
                *bundle.anchor(),
                authorization(proof_len),
                bundle_version,
            );

            assert_eq!(on_bytes.err(), on_points.err());
        }
    }
}
