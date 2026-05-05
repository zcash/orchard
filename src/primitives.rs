//! Primitives used in the Orchard protocol.
//!
//! This module handles
//! - the encryption and decryption of notes,
//! - the commitments,
//! - the redpallas signatures.
//!
//! It includes functionality for handling both the standard "Vanilla" variation and the ZSA
//! variation, with different implementations for each. The different implementations are
//! organized into separate submodules.

mod compact_action;
mod orchard_domain;
mod orchard_primitives;
mod orchard_primitives_vanilla;
mod orchard_primitives_zsa;
pub mod redpallas;
mod zcash_note_encryption_domain;

pub use {
    compact_action::CompactAction, orchard_domain::OrchardDomain,
    orchard_primitives::OrchardPrimitives,
};

#[cfg(feature = "test-dependencies")]
pub use compact_action::testing::fake_compact_action;
