//! This module defines the sighash kinds used by Orchard signatures.

use crate::primitives::redpallas::{Binding, SigType, Signature, SpendAuth};

#[cfg(test)]
use alloc::vec::Vec;

/// The kind of data that a sighash commits to.
///
/// This is used to implement [sighash versioning] for transactions containing Orchard
/// bundles.
///
/// [sighashversioning]: https://zips.z.cash/zip-0246#sighash-versioning
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OrchardSighashKind {
    /// The "default" sighash that commits to all effecting data of the transaction, as defined in
    /// [ZIP-246: Digests for the Version 6 Transaction Format][sighashversioning]
    ///
    /// [sighashversioning]: https://zips.z.cash/zip-0246#sighash-versioning
    AllEffecting,
}

/// An Orchard signature together with its `OrchardSighashKind`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OrchardSig<T: SigType> {
    sighash_kind: OrchardSighashKind,
    sig: Signature<T>,
}

impl<T: SigType> OrchardSig<T> {
    /// Constructs an `OrchardSig` from its constituent parts.
    pub fn new(sighash_kind: OrchardSighashKind, sig: Signature<T>) -> Self {
        Self { sighash_kind, sig }
    }

    /// Returns the `OrchardSighashKind` of the signature.
    pub fn sighash_kind(&self) -> &OrchardSighashKind {
        &self.sighash_kind
    }

    /// Returns the signature.
    pub fn sig(&self) -> &Signature<T> {
        &self.sig
    }
}

/// An Orchard SpendAuth signature with its `OrchardSighashKind`.
pub type OrchardSpendAuthSig = OrchardSig<SpendAuth>;

/// An Orchard binding signature with its `OrchardSighashKind`.
pub type OrchardBindingSig = OrchardSig<Binding>;

/// Returns the `SighashInfo` encoding for the given [`OrchardSighashKind`].
///
/// This helper is only intended for use in tests.
#[cfg(test)]
pub(crate) fn test_sighash_info_for_kind(kind: &OrchardSighashKind) -> Vec<u8> {
    match kind {
        OrchardSighashKind::AllEffecting => vec![0],
    }
}
