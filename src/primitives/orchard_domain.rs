//! Orchard-specific note encryption domain.

// Review hint: this file is largely derived from src/note_encryption.rs

use crate::{
    action::Action, note::Rho, primitives::compact_action::CompactAction,
    primitives::orchard_primitives::OrchardPrimitives,
};

/// Orchard-specific note encryption logic.
#[derive(Debug, Clone)]
pub struct OrchardDomain<P: OrchardPrimitives> {
    /// A parameter needed to generate the nullifier.
    pub rho: Rho,
    phantom: core::marker::PhantomData<P>,
}

impl<P: OrchardPrimitives> OrchardDomain<P> {
    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_action<T>(act: &Action<T, P>) -> Self {
        Self {
            rho: act.rho(),
            phantom: Default::default(),
        }
    }

    /// Constructs a domain that can be used to trial-decrypt this compact action's output note.
    pub fn for_compact_action(act: &CompactAction<P>) -> Self {
        Self {
            rho: act.rho(),
            phantom: Default::default(),
        }
    }

    /// Constructs a domain from a rho.
    #[cfg(test)]
    pub fn for_rho(rho: Rho) -> Self {
        Self {
            rho,
            phantom: Default::default(),
        }
    }
}
