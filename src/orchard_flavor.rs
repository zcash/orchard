//! Defines types and traits for the variations ("flavors") of the Orchard protocol (Vanilla and ZSA).

use crate::{bundle::OrchardHash, circuit::OrchardCircuit, note_encryption::OrchardDomainCommon};

/// Represents the "Vanilla" variation ("flavor") of the Orchard protocol.  
#[derive(Debug, Clone, Default)]
pub struct OrchardVanilla;

/// Represents the "ZSA" variation ("flavor") of the Orchard protocol.
#[derive(Debug, Clone, Default)]
pub struct OrchardZSA;

/// A trait binding the common functionality between different Orchard protocol flavors.
pub trait OrchardFlavor: OrchardDomainCommon + OrchardCircuit + OrchardHash {}

impl OrchardFlavor for OrchardVanilla {}
impl OrchardFlavor for OrchardZSA {}
