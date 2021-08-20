//! Gadget, chip, and primitives for the Poseidon hash function.
#[cfg(feature = "halo2")]
pub mod gadget;
#[cfg(feature = "halo2")]
pub mod pow5t3;
pub mod primitive;
