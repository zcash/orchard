//! This module defines the sighash kinds used by issuance authorization signatures.

use crate::issuance::auth::{IssueAuthSig, IssueAuthSigScheme, ZSASchnorr};

/// The kind of data that a sighash commits to.
///
/// This is used to implement [sighash versioning] for issuance transactions.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IssueSighashKind {
    /// The "default" sighash that commits to all effecting data of the transaction, as defined in
    /// [ZIP-246: Digests for the Version 6 Transaction Format][sighashversioning]
    ///
    /// [sighashversioning]: https://zips.z.cash/zip-0246#sighash-versioning
    AllEffecting,
}

/// An issuance authorization signature together with its `IssueSighashKind`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueSig<S: IssueAuthSigScheme> {
    sighash_kind: IssueSighashKind,
    sig: IssueAuthSig<S>,
}

impl<S: IssueAuthSigScheme> IssueSig<S> {
    /// Constructs an `IssueSig` from its constituent parts.
    pub fn new(sighash_kind: IssueSighashKind, sig: IssueAuthSig<S>) -> Self {
        Self { sighash_kind, sig }
    }

    /// Returns the `IssueSighashKind` of the signature.
    pub fn sighash_kind(&self) -> &IssueSighashKind {
        &self.sighash_kind
    }

    /// Returns the signature.
    pub fn sig(&self) -> &IssueAuthSig<S> {
        &self.sig
    }
}

/// An issuance authorization signature based on BIP 340 Schnorr with its `IssueSighashKind`
pub type BIP340IssueAuthSig = IssueSig<ZSASchnorr>;

/// Returns the `SighashInfo` encoding for the given [`IssueSighashKind`].
///
/// This helper is only intended for use in tests.
#[cfg(test)]
pub fn test_sighash_info_for_kind(kind: &IssueSighashKind) -> &'static [u8] {
    match kind {
        IssueSighashKind::AllEffecting => &[0],
    }
}
