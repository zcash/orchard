//! Gadget, chips, and primitives for the Sinsemilla hash function.
#[cfg(feature = "halo2")]
pub mod chip;
#[cfg(feature = "halo2")]
pub mod gadget;
#[cfg(feature = "halo2")]
pub mod merkle;
#[cfg(feature = "halo2")]
mod message;
pub mod primitive;
