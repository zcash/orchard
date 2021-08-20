//! Gadget, chips, and primitives for the Sinsemilla hash function.
#[cfg(feature = "halo2")]
pub mod gadget;
#[cfg(feature = "halo2")]
pub mod chip;
#[cfg(feature = "halo2")]
mod message;
#[cfg(feature = "halo2")]
pub mod merkle;
pub mod primitive;
