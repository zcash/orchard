//! Opt-in deferral of point decompression for encodings the caller already validated.

/// Runs `f` with point decompression deferred for keys and commitments parsed on this thread.
///
/// Only use it for encodings this library accepted before, such as transactions a node reads
/// back from its own verified state. Never parse network input inside it: the canonical-encoding
/// and small-order checks are skipped, and an invalid point panics when it is first used.
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
pub(crate) fn encodings_are_trusted() -> bool {
    #[cfg(feature = "std")]
    {
        TRUSTED.with(|trusted| trusted.get())
    }
    #[cfg(not(feature = "std"))]
    false
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
    fn invalid_commitments_are_rejected_unless_trusted() {
        let bytes = [0xff; 32];
        assert!(bool::from(ValueCommitment::from_bytes(&bytes).is_none()));
        let trusted = with_trusted_encodings(|| ValueCommitment::from_bytes(&bytes));
        assert_eq!(trusted.unwrap().to_bytes(), bytes);
        assert!(!encodings_are_trusted());
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
