//! Defines types and traits for the variations ("flavors") of the Orchard protocol (Vanilla and ZSA).

use crate::{circuit::OrchardCircuit, domain::OrchardDomainCommon};

/// Represents the "Vanilla" variation ("flavor") of the Orchard protocol.  
#[derive(Debug, Clone, Default)]
pub struct OrchardVanilla;

/// Represents the "ZSA" variation ("flavor") of the Orchard protocol.
#[derive(Debug, Clone, Default)]
pub struct OrchardZSA;

/// Represents the flavor of the Orchard protocol.
///
/// This enables conditional execution during runtime based on the flavor of the Orchard protocol.
#[derive(Clone, Debug)]
pub enum Flavor {
    /// The "Vanilla" flavor of the Orchard protocol.
    OrchardVanillaFlavor,
    /// The "ZSA" flavor of the Orchard protocol.
    OrchardZSAFlavor,
}

/// A trait binding the common functionality between different Orchard protocol flavors.
pub trait OrchardFlavor: OrchardDomainCommon + OrchardCircuit {
    /// Flavor of the Orchard protocol.
    const FLAVOR: Flavor;
}

impl OrchardFlavor for OrchardVanilla {
    const FLAVOR: Flavor = Flavor::OrchardVanillaFlavor;
}
impl OrchardFlavor for OrchardZSA {
    const FLAVOR: Flavor = Flavor::OrchardZSAFlavor;
}
