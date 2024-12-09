//! In-band secret distribution for Orchard bundles.
//!
//! This module handles the encryption and decryption of notes within the Orchard protocol,
//! It includes functionality for handling both the standard "Vanilla" variantion and the ZSA
//! variantion, with different implementations for each. The different implementations are
//! organized into separate submodules.

mod compact_action;
mod orchard_domain;
mod orchard_domain_vanilla;
mod orchard_domain_zsa;
mod zcash_note_encryption_domain;

pub use {
    compact_action::CompactAction,
    orchard_domain::{OrchardDomain, OrchardDomainCommon},
};
