//! Opt-in deferral of value commitment decompression for encodings the caller already validated.

/// Runs `f` with decompression of value commitments parsed on this thread deferred to first use.
///
/// Only [`ValueCommitment`] is deferred. Keys are parsed by reddsa, which has its own scope
/// (`reddsa::trusted::with_trusted_encodings`).
///
/// Inside the scope, [`ValueCommitment::from_bytes`] accepts any 32 bytes, including off-curve
/// and non-canonical ones, and [`ValueCommitment::to_bytes`] returns them unchanged. The point is
/// decoded on first use and kept. If it is invalid, proof verification and
/// `BatchValidator::add_bundle` return an error, and other uses (such as
/// `Bundle::binding_validating_key`) panic.
///
/// Only use it for encodings this library accepted before, such as transactions a node reads
/// back from its own verified state. Never parse network input inside it.
///
/// The flag is per thread, so `f` must not block on rayon (`join`, `scope`, `par_iter`) or drive
/// an executor, or other jobs run on this thread meanwhile would parse with trust on.
///
/// Without the `std` feature this is a no-op: `f` runs and commitments are decoded eagerly.
///
/// [`ValueCommitment`]: crate::value::ValueCommitment
/// [`ValueCommitment::from_bytes`]: crate::value::ValueCommitment::from_bytes
/// [`ValueCommitment::to_bytes`]: crate::value::ValueCommitment::to_bytes
pub fn with_trusted_encodings<R>(f: impl FnOnce() -> R) -> R {
    #[cfg(feature = "std")]
    {
        struct Restore(bool);
        impl Drop for Restore {
            fn drop(&mut self) {
                TRUSTED.with(|trusted| trusted.set(self.0));
            }
        }
        let _restore = Restore(TRUSTED.with(|trusted| trusted.replace(true)));
        f()
    }
    #[cfg(not(feature = "std"))]
    f()
}

/// Returns true inside [`with_trusted_encodings`] on this thread.
#[cfg(feature = "std")]
pub(crate) fn encodings_are_trusted() -> bool {
    TRUSTED.with(|trusted| trusted.get())
}

#[cfg(feature = "std")]
std::thread_local! {
    static TRUSTED: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use group::{Group, GroupEncoding};
    use pasta_curves::pallas;
    use rand::SeedableRng;

    use super::*;
    use crate::value::{ValueCommitTrapdoor, ValueCommitment, ValueSum};

    #[test]
    fn invalid_commitments_are_deferred_inside_the_scope() {
        let bytes = [0xff; 32];
        assert!(bool::from(ValueCommitment::from_bytes(&bytes).is_none()));
        let trusted = with_trusted_encodings(|| ValueCommitment::from_bytes(&bytes)).unwrap();
        assert_eq!(trusted.to_bytes(), bytes);
        assert!(trusted.checked_point().is_none());
        assert!(std::panic::catch_unwind(|| trusted.x()).is_err());
        assert!(!encodings_are_trusted());
    }

    // Replaces the first action's `cv_net` with one parsed inside the scope from `cv_net`.
    #[cfg(feature = "circuit")]
    fn with_lazy_cv_net(
        bundle: &crate::Bundle<crate::bundle::Authorized, ValueSum>,
        cv_net: [u8; 32],
    ) -> crate::Bundle<crate::bundle::Authorized, ValueSum> {
        let cv_net = with_trusted_encodings(|| ValueCommitment::from_bytes(&cv_net)).unwrap();
        let mut actions = bundle.actions().clone();
        let first = actions.first().clone();
        actions.head = crate::Action::from_parts(
            *first.nullifier(),
            first.rk().clone(),
            *first.cmx(),
            first.encrypted_note().clone(),
            cv_net,
            first.authorization().clone(),
        )
        .unwrap();
        crate::Bundle::from_parts_unchecked(
            actions,
            *bundle.flags(),
            *bundle.value_balance(),
            *bundle.anchor(),
            bundle.authorization().clone(),
            bundle.bundle_version(),
        )
    }

    #[cfg(feature = "circuit")]
    #[test]
    fn invalid_commitments_parsed_inside_the_scope_fail_verification() {
        use crate::{
            bundle::{tests::sample_authorized_bundle, BatchError, BatchValidator},
            circuit::OrchardCircuitVersion,
        };

        let bundle = sample_authorized_bundle(1);
        let vk = crate::cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2).verifying_key();
        let add_bundle = |bundle: &crate::Bundle<_, ValueSum>| {
            let bundle = bundle
                .clone()
                .try_map_value_balance(i64::try_from)
                .expect("generated bundle value balance fits in i64");
            BatchValidator::new(vk).add_bundle(&bundle, [0; 32])
        };
        let cv_net = bundle.actions().first().cv_net().to_bytes();

        // The eager bundle gets past the instance checks.
        assert!(!matches!(
            bundle.verify_proof(vk),
            Err(halo2_proofs::plonk::Error::InvalidInstances)
        ));
        assert_eq!(add_bundle(&bundle), Ok(()));

        // The same commitment parsed lazily verifies like the eager one.
        let lazy = with_lazy_cv_net(&bundle, cv_net);
        assert_eq!(
            lazy.verify_proof(vk).is_ok(),
            bundle.verify_proof(vk).is_ok()
        );
        assert_eq!(add_bundle(&lazy), Ok(()));

        let invalid = with_lazy_cv_net(&bundle, [0xff; 32]);
        assert!(matches!(
            invalid.verify_proof(vk),
            Err(halo2_proofs::plonk::Error::InvalidInstances)
        ));
        assert_eq!(add_bundle(&invalid), Err(BatchError::InvalidEncoding));
    }

    #[test]
    fn trusted_commitments_match_decoded_commitments() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let mut bytes = alloc::vec::Vec::new();
        for value in 0..4 {
            let rcv = ValueCommitTrapdoor::random(&mut rng);
            bytes.push(ValueCommitment::derive(ValueSum::from_raw(value), rcv).to_bytes());
        }
        bytes.push(pallas::Point::identity().to_bytes());
        let decoded: alloc::vec::Vec<_> = bytes
            .iter()
            .map(|bytes| ValueCommitment::from_bytes(bytes).unwrap())
            .collect();
        let trusted: alloc::vec::Vec<_> = with_trusted_encodings(|| {
            bytes
                .iter()
                .map(|bytes| ValueCommitment::from_bytes(bytes).unwrap())
                .collect()
        });
        for (decoded, trusted) in decoded.iter().zip(&trusted) {
            assert_eq!(decoded.point(), trusted.point());
            assert_eq!(decoded.to_bytes(), trusted.to_bytes());
        }
        let decoded_sum: ValueCommitment = decoded.iter().sum();
        let trusted_sum: ValueCommitment = trusted.iter().sum();
        assert_eq!(decoded_sum.to_bytes(), trusted_sum.to_bytes());
    }

    #[test]
    fn the_scope_ends_even_after_a_panic() {
        let result = std::panic::catch_unwind(|| with_trusted_encodings(|| panic!("inside")));
        assert!(result.is_err());
        assert!(!encodings_are_trusted());
    }
}
