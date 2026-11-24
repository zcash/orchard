use alloc::vec;
use core::fmt;

use nonempty::NonEmpty;
use rand::{CryptoRng, Rng};

use super::Action;
use crate::{
    bundle::{Authorization, Authorized, EffectsOnly},
    primitives::redpallas::{self, Binding, SpendAuth},
    Proof,
};

impl super::Bundle {
    /// Extracts the effects of this PCZT bundle as a [regular `Bundle`].
    ///
    /// This is used by the Signer role to produce the transaction sighash.
    ///
    /// [regular `Bundle`]: crate::Bundle
    pub fn extract_effects<V: TryFrom<i64>>(
        &self,
    ) -> Result<Option<crate::Bundle<EffectsOnly, V>>, TxExtractorError> {
        self.to_tx_data(Some(self.anchor), |_| Ok(()), |_| Ok(EffectsOnly))
    }

    /// Extracts a fully authorized [regular `Bundle`] from this PCZT bundle.
    ///
    /// This is used by the Transaction Extractor role to produce the final transaction.
    ///
    /// [regular `Bundle`]: crate::Bundle
    pub fn extract<V: TryFrom<i64>>(
        &self,
    ) -> Result<Option<crate::Bundle<Unbound, V>>, TxExtractorError> {
        let bundle = self.to_tx_data(
            self.anchor,
            |action| {
                action
                    .spend
                    .spend_auth_sig
                    .clone()
                    .ok_or(TxExtractorError::MissingSpendAuthSig)
            },
            |bundle| {
                Ok(Unbound {
                    proof: bundle
                        .zkproof
                        .clone()
                        .ok_or(TxExtractorError::MissingProof)?,
                    bsk: bundle
                        .bsk
                        .clone()
                        .ok_or(TxExtractorError::MissingBindingSignatureSigningKey)?,
                })
            },
        )?;

        // The proof comes straight from the (untrusted) PCZT, so reject
        // non-canonical proof lengths here. This makes "an `Authorized` bundle
        // always has a canonical proof" hold across the `Unbound` -> `Authorized`
        // transition in `apply_binding_signature`. Circuit-key support for bundle
        // flags is checked when proving or verifying.
        if let Some(bundle) = &bundle {
            crate::bundle::validate_proof_size(
                &bundle.authorization().proof,
                bundle.actions().len(),
            )?;
        }

        Ok(bundle)
    }

    /// Converts this PCZT bundle into a regular bundle with the given authorizations.
    fn to_tx_data<A, V, E, F, G>(
        &self,
        anchor: A::Anchor,
        action_auth: F,
        bundle_auth: G,
    ) -> Result<Option<crate::Bundle<A, V>>, E>
    where
        A: Authorization,
        E: From<TxExtractorError>,
        F: Fn(&Action) -> Result<<A as Authorization>::SpendAuth, E>,
        G: FnOnce(&Self) -> Result<A, E>,
        V: TryFrom<i64>,
    {
        let actions = self
            .actions
            .iter()
            .map(|action| {
                let authorization = action_auth(action)?;

                crate::Action::from_parts(
                    action.spend.nullifier,
                    action.spend.rk.clone(),
                    action.output.cmx,
                    action.output.encrypted_note.clone(),
                    action.cv_net.clone(),
                    authorization,
                )
                .map_err(|e| TxExtractorError::from(e).into())
            })
            .collect::<Result<_, E>>()?;

        Ok(if let Some(actions) = NonEmpty::from_vec(actions) {
            let value_balance = i64::try_from(self.value_sum)
                .ok()
                .and_then(|v| v.try_into().ok())
                .ok_or(TxExtractorError::ValueSumOutOfRange)?;

            let authorization = bundle_auth(self)?;

            Some(crate::Bundle::from_parts_unchecked(
                actions,
                self.flags,
                value_balance,
                anchor,
                authorization,
                self.bundle_version,
            ))
        } else {
            None
        })
    }
}

/// Errors that can occur while extracting a regular Orchard bundle from a PCZT bundle.
#[derive(Debug)]
#[non_exhaustive]
pub enum TxExtractorError {
    /// The Transaction Extractor role requires `bsk` to be set.
    MissingBindingSignatureSigningKey,
    /// The Transaction Extractor role requires `zkproof` to be set.
    MissingProof,
    /// The Transaction Extractor role requires all `spend_auth_sig` fields to be set.
    MissingSpendAuthSig,
    /// The number of per-action sighashes supplied did not match the number of actions.
    SighashCountMismatch {
        /// The number of actions in the bundle.
        expected: usize,
        /// The number of sighashes supplied.
        actual: usize,
    },
    /// A `spend_auth_sig` is invalid for its action's sighash.
    InvalidSpendAuthSig,
    /// The value sum does not fit into a `valueBalance`.
    ValueSumOutOfRange,
    /// An action has an identity `rk`, which is forbidden by the consensus
    /// rule introduced in zcashd v6.12.1 and Zebra 4.3.1.
    IdentityRk,
    /// An action has an `epk` that does not encode a non-identity Pallas point.
    InvalidEpk,
    /// The `zkproof` does not have the canonical length for the bundle's number of actions.
    NonCanonicalProofSize {
        /// The canonical proof length for the bundle's number of actions.
        expected: usize,
        /// The length of the proof that was provided.
        actual: usize,
    },
    /// The bundle's flags cannot be encoded under its value pool and protocol version.
    UnrepresentableFlags,
}

impl From<crate::ActionFromPartsError> for TxExtractorError {
    fn from(e: crate::ActionFromPartsError) -> Self {
        match e {
            crate::ActionFromPartsError::IdentityRk => TxExtractorError::IdentityRk,
            crate::ActionFromPartsError::InvalidEpk => TxExtractorError::InvalidEpk,
        }
    }
}

impl From<crate::bundle::BundleError> for TxExtractorError {
    fn from(e: crate::bundle::BundleError) -> Self {
        match e {
            crate::bundle::BundleError::NonCanonicalProofSize { expected, actual } => {
                TxExtractorError::NonCanonicalProofSize { expected, actual }
            }
            crate::bundle::BundleError::UnrepresentableFlags => {
                TxExtractorError::UnrepresentableFlags
            }
        }
    }
}

impl fmt::Display for TxExtractorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxExtractorError::MissingBindingSignatureSigningKey => {
                write!(f, "`bsk` must be set for the Transaction Extractor role")
            }
            TxExtractorError::MissingProof => write!(
                f,
                "Orchard `zkproof` must be set for the Transaction Extractor role"
            ),
            TxExtractorError::MissingSpendAuthSig => write!(
                f,
                "`spend_auth_sig` fields must all be set for the Transaction Extractor role"
            ),
            TxExtractorError::SighashCountMismatch { expected, actual } => write!(
                f,
                "expected {expected} action sighashes, but received {actual}"
            ),
            TxExtractorError::InvalidSpendAuthSig => {
                write!(f, "a `spend_auth_sig` is invalid for its action's sighash")
            }
            TxExtractorError::ValueSumOutOfRange => {
                write!(f, "value sum does not fit into a `valueBalance`")
            }
            TxExtractorError::IdentityRk => {
                write!(f, "an Orchard action with identity `rk` is not valid")
            }
            TxExtractorError::InvalidEpk => write!(
                f,
                "an Orchard action's `epk` is not a valid non-identity Pallas point"
            ),
            TxExtractorError::NonCanonicalProofSize { expected, actual } => write!(
                f,
                "Orchard `zkproof` has non-canonical length {actual}; expected {expected} bytes",
            ),
            TxExtractorError::UnrepresentableFlags => write!(
                f,
                "Orchard bundle flags are not representable under its value pool and protocol version",
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TxExtractorError {}

/// Authorizing data for a bundle of actions that is just missing a binding signature.
#[derive(Debug)]
pub struct Unbound {
    proof: Proof,
    bsk: redpallas::SigningKey<Binding>,
}

impl Authorization for Unbound {
    type Anchor = crate::Anchor;
    type SpendAuth = redpallas::Signature<SpendAuth>;
}

impl<V> crate::Bundle<Unbound, V> {
    /// Verifies the given sighash with every `spend_auth_sig`, and then binds the bundle.
    ///
    /// This is the signature message convention of transaction formats that sign the
    /// whole bundle with one digest. Use [`Self::apply_binding_signature_with_sighashes`]
    /// when each action and the binding signature have distinct messages.
    ///
    /// Returns `None` if the given sighash does not validate against every `spend_auth_sig`.
    pub fn apply_binding_signature<R: Rng + CryptoRng>(
        self,
        sighash: [u8; 32],
        rng: R,
    ) -> Option<crate::Bundle<Authorized, V>> {
        let sighashes = vec![sighash; self.actions().len()];
        self.apply_binding_signature_with_sighashes(&sighashes, sighash, rng)
            .ok()
    }

    /// Verifies each action's `spend_auth_sig` against its own sighash, and then binds the
    /// bundle with `binding_sighash`.
    ///
    /// `sighashes` must contain exactly one digest per action, in action order: element
    /// `i` is the signature message for action `i`. `binding_sighash` is the signature
    /// message for the binding signature only.
    ///
    /// # Errors
    ///
    /// Returns [`TxExtractorError::SighashCountMismatch`] if `sighashes` does not contain
    /// exactly one digest per action, or [`TxExtractorError::InvalidSpendAuthSig`] if any
    /// digest does not validate against the corresponding `spend_auth_sig`.
    /// The count is checked before verifying signatures. The bundle is consumed even
    /// on error.
    pub fn apply_binding_signature_with_sighashes<R: Rng + CryptoRng>(
        self,
        sighashes: &[[u8; 32]],
        binding_sighash: [u8; 32],
        rng: R,
    ) -> Result<crate::Bundle<Authorized, V>, TxExtractorError> {
        let expected = self.actions().len();
        if sighashes.len() != expected {
            return Err(TxExtractorError::SighashCountMismatch {
                expected,
                actual: sighashes.len(),
            });
        }
        if !self
            .actions()
            .iter()
            .zip(sighashes)
            .all(|(action, sighash)| action.rk().verify(sighash, action.authorization()).is_ok())
        {
            return Err(TxExtractorError::InvalidSpendAuthSig);
        }

        Ok(self.map_authorization(
            &mut (),
            |_, _, a| a,
            |_, Unbound { proof, bsk }| {
                Authorized::from_parts(proof, bsk.sign(rng, &binding_sighash))
            },
        ))
    }
}
