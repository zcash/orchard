use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use rand::{CryptoRng, Rng};

use crate::{
    keys::SpendAuthorizingKey,
    primitives::redpallas,
    value::{ValueCommitTrapdoor, ValueCommitment},
};

use super::{SignerError, VerifyError};

impl super::Bundle {
    /// Finalizes the IO for this bundle, signing every dummy spend with a single sighash
    /// shared by all of them.
    ///
    /// This is the signature message convention of transaction formats that sign the
    /// whole bundle with one digest. Use [`Self::finalize_io_with_sighashes`] when each
    /// action has its own message.
    ///
    /// If the bundle disables cross-address transfers, the structural restriction is
    /// first checked with [`Bundle::verify_cross_address_restriction`] (which requires
    /// the spend and output `recipient` fields to be set); on failure the bundle is
    /// left unmodified.
    ///
    /// [`Bundle::verify_cross_address_restriction`]: super::Bundle::verify_cross_address_restriction
    pub fn finalize_io<R: Rng + CryptoRng>(
        &mut self,
        sighash: [u8; 32],
        rng: R,
    ) -> Result<(), IoFinalizerError> {
        let sighashes = vec![sighash; self.actions.len()];
        self.finalize_io_with_sighashes(&sighashes, rng)
    }

    /// Finalizes the IO for this bundle, signing each dummy spend with its own sighash.
    ///
    /// `sighashes` must contain exactly one digest per action, in action order: element
    /// `i` is the signature message for action `i`.
    ///
    /// If the bundle disables cross-address transfers, the structural restriction is
    /// first checked with [`Bundle::verify_cross_address_restriction`] (which requires
    /// the spend and output `recipient` fields to be set); on failure the bundle is
    /// left unmodified.
    ///
    /// # Errors
    ///
    /// Returns [`IoFinalizerError::SighashCountMismatch`] if `sighashes` does not contain
    /// exactly one digest per action; in that case the bundle is left unmodified.
    ///
    /// [`Bundle::verify_cross_address_restriction`]: super::Bundle::verify_cross_address_restriction
    pub fn finalize_io_with_sighashes<R: Rng + CryptoRng>(
        &mut self,
        sighashes: &[[u8; 32]],
        mut rng: R,
    ) -> Result<(), IoFinalizerError> {
        let expected = self.actions.len();
        if sighashes.len() != expected {
            return Err(IoFinalizerError::SighashCountMismatch {
                expected,
                actual: sighashes.len(),
            });
        }

        // A bundle that disables cross-address transfers can never be proven or mined
        // if any action violates the structural restriction; fail before mutating
        // anything.
        self.verify_cross_address_restriction()
            .map_err(IoFinalizerError::CrossAddressRestriction)?;

        // Compute the transaction binding signing key.
        let rcvs = self
            .actions
            .iter()
            .map(|a| {
                a.rcv
                    .as_ref()
                    .ok_or(IoFinalizerError::MissingValueCommitTrapdoor)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bsk = rcvs.into_iter().sum::<ValueCommitTrapdoor>().into_bsk();

        // Verify that bsk and bvk are consistent.
        let bvk = (self
            .actions
            .iter()
            .map(|a| a.cv_net())
            .sum::<ValueCommitment>()
            - ValueCommitment::derive(self.value_sum, ValueCommitTrapdoor::zero()))
        .into_bvk();
        if redpallas::VerificationKey::from(&bsk) != bvk {
            return Err(IoFinalizerError::ValueCommitMismatch);
        }
        self.bsk = Some(bsk);

        // Add signatures to dummy spends, each under its own action index.
        for (index, action) in self.actions.iter_mut().enumerate() {
            // The `Option::take` ensures we don't have any spend authorizing keys in the
            // PCZT after the IO Finalizer has run.
            if let Some(sk) = action.spend.dummy_sk.take() {
                let ask = SpendAuthorizingKey::from(&sk);
                action
                    .sign(sighashes[index], &ask, &mut rng)
                    .map_err(IoFinalizerError::DummySignature)?;
            }
        }

        Ok(())
    }
}

/// Errors that can occur while finalizing the I/O for a PCZT bundle.
#[derive(Debug)]
#[non_exhaustive]
pub enum IoFinalizerError {
    /// The bundle does not satisfy the cross-address restriction, or is missing the
    /// `recipient` fields needed to check it.
    CrossAddressRestriction(VerifyError),
    /// An error occurred while signing a dummy spend.
    DummySignature(SignerError),
    /// The IO Finalizer role requires all `rcv` fields to be set.
    MissingValueCommitTrapdoor,
    /// The number of per-action sighashes supplied did not match the number of actions in
    /// the bundle.
    SighashCountMismatch {
        /// The number of actions in the bundle, which is the number of sighashes required.
        expected: usize,
        /// The number of sighashes supplied.
        actual: usize,
    },
    /// The `cv_net`, `rcv`, and `value_sum` values within the Orchard bundle are
    /// inconsistent.
    ValueCommitMismatch,
}

impl fmt::Display for IoFinalizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoFinalizerError::CrossAddressRestriction(e) => {
                write!(
                    f,
                    "The bundle does not satisfy the cross-address restriction: {e}"
                )
            }
            IoFinalizerError::DummySignature(e) => {
                write!(f, "An error occurred while signing a dummy spend: {e}")
            }
            IoFinalizerError::MissingValueCommitTrapdoor => write!(
                f,
                "The IO Finalizer role requires all `rcv` fields to be set"
            ),
            IoFinalizerError::SighashCountMismatch { expected, actual } => write!(
                f,
                "Expected {expected} action sighashes, but received {actual}"
            ),
            IoFinalizerError::ValueCommitMismatch => write!(
                f,
                "`cv_net`, `rcv`, and `value_sum` within the Orchard bundle are inconsistent."
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IoFinalizerError {}
