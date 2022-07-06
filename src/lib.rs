//! # orchard
//!
//! ## Nomenclature
//!
//! All types in the `orchard` crate, unless otherwise specified, are Orchard-specific
//! types. For example, [`Address`] is documented as being a shielded payment address; we
//! implicitly mean it is an Orchard payment address (as opposed to e.g. a Sapling payment
//! address, which is also shielded).

#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Temporary until we have more of the crate implemented.
#![allow(dead_code)]
// Catch documentation errors caused by code changes.
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

mod action;
mod address;
#[cfg(feature = "std")]
pub mod builder;
#[cfg(feature = "std")]
pub mod bundle;
#[cfg(feature = "std")]
pub mod circuit;
mod constants;
pub mod keys;
pub mod note;
pub mod note_encryption;
pub mod primitives;
mod spec;
#[cfg(feature = "std")]
pub mod tree;
pub mod value;
pub mod zip32;

#[cfg(test)]
mod test_vectors;

pub use action::Action;
pub use address::Address;
#[cfg(feature = "std")]
pub use bundle::Bundle;
#[cfg(feature = "std")]
pub use circuit::Proof;
pub use note::Note;
#[cfg(feature = "std")]
pub use tree::Anchor;
