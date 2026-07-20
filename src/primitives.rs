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

mod orchard_primitives;
mod orchard_primitives_vanilla;
mod orchard_primitives_zsa;
pub mod redpallas;

pub use orchard_primitives::OrchardPrimitives;
