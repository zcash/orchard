//! This module defines the versioning for issuance authorization signatures.

use crate::issuance_auth::{IssueAuthSig, IssueAuthSigScheme, ZSASchnorr};

/// The Issuance Sighash version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSighashVersion {
    /// Version V0.
    V0,
}

/// The Issuance spend auth versioned signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueVersionedSig<S: IssueAuthSigScheme> {
    version: IssueSighashVersion,
    sig: IssueAuthSig<S>,
}

impl<S: IssueAuthSigScheme> IssueVersionedSig<S> {
    /// Constructs an `IssueVersionedSig` from its constituent parts.
    pub fn new(version: IssueSighashVersion, sig: IssueAuthSig<S>) -> Self {
        Self { version, sig }
    }

    /// Returns the version of the signature.
    pub fn version(&self) -> &IssueSighashVersion {
        &self.version
    }

    /// Returns the signature.
    pub fn sig(&self) -> &IssueAuthSig<S> {
        &self.sig
    }
}

/// A versioned Issuance authorization signature based on BIP 340 Schnorr.
pub type VerBIP340IssueAuthSig = IssueVersionedSig<ZSASchnorr>;
