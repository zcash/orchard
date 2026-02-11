//! Defines types and traits for the variations ("flavors") of the Orchard protocol (Vanilla and ZSA).

#[cfg(feature = "circuit")]
use crate::{circuit::OrchardCircuit, primitives::OrchardPrimitives};

/// Represents the "Vanilla" variation ("flavor") of the Orchard protocol.  
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrchardVanilla;

/// Represents the "ZSA" variation ("flavor") of the Orchard protocol.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrchardZSA;

/// A trait binding the common functionality between different Orchard protocol flavors.
#[cfg(feature = "circuit")]
pub trait OrchardFlavor: OrchardPrimitives + OrchardCircuit {}

#[cfg(feature = "circuit")]
impl OrchardFlavor for OrchardVanilla {}

#[cfg(feature = "circuit")]
impl OrchardFlavor for OrchardZSA {}
