//! Orchard-specific note encryption domain.

use crate::{
    action::Action, note::Rho, primitives::compact_action::CompactAction,
    primitives::orchard_primitives::OrchardPrimitives,
};

/// Orchard-specific note encryption logic.
#[derive(Debug, Clone)]
pub struct OrchardDomain<Pr: OrchardPrimitives> {
    /// A parameter needed to generate the nullifier.
    pub rho: Rho,
    phantom: core::marker::PhantomData<Pr>,
}

impl<Pr: OrchardPrimitives> memuse::DynamicUsage for OrchardDomain<Pr> {
    fn dynamic_usage(&self) -> usize {
        self.rho.dynamic_usage()
    }
    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        self.rho.dynamic_usage_bounds()
    }
}

impl<Pr: OrchardPrimitives> OrchardDomain<Pr> {
    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_action<T>(act: &Action<T, Pr>) -> Self {
        Self {
            rho: act.rho(),
            phantom: Default::default(),
        }
    }

    /// Constructs a domain that can be used to trial-decrypt a PCZT action's output note.
    pub fn for_pczt_action(act: &crate::pczt::Action) -> Self {
        Self {
            rho: Rho::from_nf_old(act.spend().nullifier),
            phantom: Default::default(),
        }
    }

    /// Constructs a domain that can be used to trial-decrypt this compact action's output note.
    pub fn for_compact_action(act: &CompactAction<Pr>) -> Self {
        Self {
            rho: act.rho(),
            phantom: Default::default(),
        }
    }

    /// Constructs a domain from a rho.
    #[cfg(test)]
    pub(crate) fn for_rho(rho: Rho) -> Self {
        Self {
            rho,
            phantom: Default::default(),
        }
    }
}
