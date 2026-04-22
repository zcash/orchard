//! # orchard
//!
//! ## Nomenclature
//!
//! All types in the `orchard` crate, unless otherwise specified, are Orchard-specific
//! types. For example, [`Address`] is documented as being a shielded payment address; we
//! implicitly mean it is an Orchard payment address (as opposed to e.g. a Sapling payment
//! address, which is also shielded).

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

// Root-level re-exports.
#[doc(inline)]
pub use __impl::Action;
#[doc(inline)]
pub use __impl::Address;
#[doc(inline)]
pub use __impl::Anchor;
#[doc(inline)]
pub use __impl::Bundle;
#[doc(inline)]
pub use __impl::Note;
#[doc(inline)]
pub use __impl::Proof;
#[doc(inline)]
pub use __impl::NOTE_COMMITMENT_TREE_DEPTH;

/// Logic for building Orchard components of transactions.
pub mod builder {
    #[doc(inline)]
    pub use __impl::builder::{
        BuildError, Builder, BundleMetadata, BundleType, InProgress, InProgressSignatures,
        InputView, MaybeSigned, OutputError, OutputInfo, OutputView, PartiallyAuthorized,
        SigningMetadata, SigningParts, SpendError, SpendInfo, Unauthorized,
    };

    #[cfg(feature = "circuit")]
    #[doc(inline)]
    pub use __impl::builder::{bundle, UnauthorizedBundle, Unproven};

    /// Generators for property testing.
    #[cfg(all(feature = "circuit", feature = "test-dependencies"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
    pub mod testing {
        #[doc(inline)]
        pub use __impl::builder::testing::{arb_bundle, arb_bundle_with_key};
    }
}

/// Structs related to bundles of Orchard actions.
pub mod bundle {
    #[doc(inline)]
    pub use __impl::bundle::{
        Authorization, Authorized, Bundle, BundleAuthorizingCommitment, BundleCommitment,
        EffectsOnly, Flags,
    };

    #[cfg(feature = "circuit")]
    #[doc(inline)]
    pub use __impl::bundle::BatchValidator;

    /// Utility functions for computing bundle commitments
    pub mod commitments {
        #[doc(inline)]
        pub use __impl::bundle::commitments::{hash_bundle_auth_empty, hash_bundle_txid_empty};
    }

    /// Generators for property testing.
    #[cfg(feature = "test-dependencies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
    pub mod testing {
        #[doc(inline)]
        pub use __impl::bundle::testing::{
            arb_action, arb_action_n, arb_bundle, arb_flags, arb_unauthorized_action,
            arb_unauthorized_action_n, arb_unauthorized_bundle, Unauthorized,
        };
    }
}

/// The Orchard Action circuit implementation.
#[cfg(feature = "circuit")]
#[cfg_attr(docsrs, doc(cfg(feature = "circuit")))]
pub mod circuit {
    #[doc(inline)]
    pub use __impl::circuit::{Circuit, Config, Instance, Proof, ProvingKey, VerifyingKey};

    /// Gadgets used in the Orchard circuit.
    pub mod gadget {}
}

/// Key structures for Orchard.
pub mod keys {
    #[doc(inline)]
    pub use __impl::keys::{
        DiversifiedTransmissionKey, Diversifier, DiversifierIndex, EphemeralPublicKey,
        EphemeralSecretKey, FullViewingKey, IncomingViewingKey, OutgoingViewingKey,
        PreparedEphemeralPublicKey, PreparedIncomingViewingKey, Scope, SharedSecret,
        SpendAuthorizingKey, SpendValidatingKey, SpendingKey,
    };

    /// Generators for property testing.
    #[cfg(feature = "test-dependencies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
    pub mod testing {
        #[doc(inline)]
        pub use __impl::keys::testing::{arb_diversifier_index, arb_esk, arb_spending_key};
    }
}

/// Data structures used for note construction.
pub mod note {
    #[doc(inline)]
    pub use __impl::note::{
        ExtractedNoteCommitment, Note, NoteCommitment, Nullifier, RandomSeed, Rho,
        TransmittedNoteCiphertext,
    };

    /// Generators for property testing.
    #[cfg(feature = "test-dependencies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
    pub mod testing {
        #[doc(inline)]
        pub use __impl::note::testing::arb_note;
    }
}

/// In-band secret distribution for Orchard bundles.
pub mod note_encryption {
    #[doc(inline)]
    pub use __impl::note_encryption::{CompactAction, OrchardDomain, OrchardNoteEncryption};

    /// Utilities for constructing test data.
    #[cfg(feature = "test-dependencies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
    pub mod testing {
        #[doc(inline)]
        pub use __impl::note_encryption::testing::fake_compact_action;
    }
}

/// PCZT support for Orchard.
pub mod pczt {
    #[doc(inline)]
    pub use __impl::pczt::{
        Action, ActionUpdater, Bundle, IoFinalizerError, Output, ParseError, SignerError, Spend,
        TxExtractorError, Unbound, Updater, UpdaterError, VerifyError, Zip32Derivation,
    };

    #[cfg(feature = "circuit")]
    #[doc(inline)]
    pub use __impl::pczt::ProverError;
}

/// Primitives used in the Orchard protocol.
pub mod primitives {
    /// A minimal RedPallas implementation for use in Zcash.
    pub mod redpallas {
        #[doc(inline)]
        pub use __impl::primitives::redpallas::{
            Binding, SigType, Signature, SigningKey, SpendAuth, VerificationKey,
        };

        #[cfg(feature = "std")]
        #[doc(inline)]
        pub use __impl::primitives::redpallas::batch;

        /// Generators for property testing.
        #[cfg(feature = "test-dependencies")]
        #[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
        pub mod testing {
            #[doc(inline)]
            pub use __impl::primitives::redpallas::testing::{
                arb_binding_signing_key, arb_binding_verification_key, arb_spendauth_signing_key,
                arb_spendauth_verification_key, arb_valid_spendauth_keypair,
            };
        }
    }
}

/// Types related to Orchard note commitment trees and anchors.
pub mod tree {
    #[doc(inline)]
    pub use __impl::tree::{Anchor, MerkleHashOrchard, MerklePath};

    /// Test utilities available under the `test-dependencies` feature flag.
    #[cfg(feature = "test-dependencies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
    pub mod testing {}
}

/// Monetary values within the Orchard shielded pool.
///
/// Values are represented in three places within the Orchard protocol:
/// - [`NoteValue`](value::NoteValue), the value of an individual note. It is an unsigned
///   64-bit integer (with maximum value [`MAX_NOTE_VALUE`](value::MAX_NOTE_VALUE)), and is
///   serialized in a note plaintext.
/// - [`ValueSum`](value::ValueSum), the sum of note values within an Orchard [`Action`] or
///   [`Bundle`]. It is a signed 64-bit integer (with range
///   [`VALUE_SUM_RANGE`](value::VALUE_SUM_RANGE)).
/// - `valueBalanceOrchard`, which is a signed 63-bit integer. This is represented
///   by a user-defined type parameter on [`Bundle`], returned by
///   [`Bundle::value_balance`] and [`Builder::value_balance`](builder::Builder::value_balance).
///
/// If your specific instantiation of the Orchard protocol requires a smaller bound on
/// valid note values (for example, Zcash's `MAX_MONEY` fits into a 51-bit integer), you
/// should enforce this in two ways:
///
/// - Define your `valueBalanceOrchard` type to enforce your valid value range. This can
///   be checked in its `TryFrom<i64>` implementation.
/// - Define your own "amount" type for note values, and convert it to `NoteValue` prior
///   to calling [`Builder::add_output`](builder::Builder::add_output).
///
/// # Caution!
///
/// An `i64` is _not_ a signed 64-bit integer! The [Rust documentation] calls `i64` the
/// 64-bit signed integer type, which is true in the sense that its encoding in memory
/// takes up 64 bits. Numerically, however, `i64` is a signed 63-bit integer.
///
/// Fortunately, users of this crate should never need to construct
/// [`ValueSum`](value::ValueSum) directly; you should only need to interact with
/// [`NoteValue`](value::NoteValue) (which can be safely constructed from a `u64`) and
/// `valueBalanceOrchard` (which can be represented as an `i64`).
///
/// [Rust documentation]: https://doc.rust-lang.org/stable/std/primitive.i64.html
pub mod value {
    #[doc(inline)]
    pub use __impl::value::{
        BalanceError, NoteValue, Sign, ValueCommitTrapdoor, ValueCommitment, ValueSum,
        MAX_NOTE_VALUE, VALUE_SUM_RANGE,
    };

    /// Generators for property testing.
    #[cfg(feature = "test-dependencies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
    pub mod testing {
        #[doc(inline)]
        pub use __impl::value::testing::{
            arb_note_value, arb_note_value_bounded, arb_positive_note_value, arb_scalar,
            arb_trapdoor, arb_value_sum, arb_value_sum_bounded,
        };
    }
}

/// Key structures for Orchard.
pub mod zip32 {
    #[doc(inline)]
    pub use __impl::zip32::{ChildIndex, Error};
}
