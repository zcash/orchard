//! [`OrchardCircuitVersion`] and its proof-size logic.
//!
//! Kept outside the `circuit` module so proof-size validation stays available without the
//! `circuit` feature flag.

/// Selects which version of the Orchard Action circuit to build.
///
/// [`FixedPostNu6_2`] and [`InsecurePreNu6_2`] produce different verifying keys: the fixed
/// circuit anchors the variable-base scalar-multiplication base (see `halo2_gadgets`), while
/// the pre-NU6.2 one does not. [`PostNu6_3`] extends the fixed circuit by enforcing the
/// same-address check, i.e. `(g_d^old, pk_d^old) = (g_d^new, pk_d^new)`, when the
/// boolean `disableCrossAddress` public input is set.
///
/// This is a runtime value rather than a type parameter: it is carried in [`Circuit`] and
/// chosen when building a [`ProvingKey`] or [`VerifyingKey`], so the circuit version can be
/// threaded dynamically (e.g. across an FFI boundary).
///
/// Please note that the public exposure of APIs using `InsecurePreNu6_2` is intentional,
/// and is strictly necessary for verifying the block chain from NU5 activation and for
/// creating proofs needed by tests that operate at past epochs. These APIs cannot be
/// used accidentally without passing an `OrchardCircuitVersion` that is clearly labelled
/// "insecure". This is not a security vulnerability.
///
/// [`FixedPostNu6_2`]: OrchardCircuitVersion::FixedPostNu6_2
/// [`InsecurePreNu6_2`]: OrchardCircuitVersion::InsecurePreNu6_2
/// [`PostNu6_3`]: OrchardCircuitVersion::PostNu6_3
/// [`Circuit`]: crate::circuit::Circuit
/// [`ProvingKey`]: crate::circuit::ProvingKey
/// [`VerifyingKey`]: crate::circuit::VerifyingKey
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OrchardCircuitVersion {
    /// The insecure pre-NU6.2 circuit, in which the variable-base scalar-multiplication base
    /// is not anchored to the real base. For reconstructing the historical (NU5..NU6.2)
    /// verifying key only — never for proving or current verification.
    InsecurePreNu6_2,
    /// The fixed circuit, active from NU6.2 onward. Used for all current network proving and
    /// verification.
    FixedPostNu6_2,
    /// The post-NU 6.3 circuit. This uses the fixed circuit with additional constraints
    /// enforcing the `disableCrossAddress` public input.
    PostNu6_3,
    /// The OrchardZSA circuit, defined in [ZIP-226].
    ///
    /// [ZIP-226]: https://zips.z.cash/zip-0226
    ZSA,
}

impl OrchardCircuitVersion {
    /// Returns the `(base, per_action)` proof-size constants for the circuit that proves and
    /// verifies actions for this bundle version, such that a canonical proof for `num_actions`
    /// actions is exactly `base + per_action * num_actions` bytes.
    pub(crate) const fn proof_size_constants(&self) -> (usize, usize) {
        match self {
            OrchardCircuitVersion::InsecurePreNu6_2
            | OrchardCircuitVersion::FixedPostNu6_2
            | OrchardCircuitVersion::PostNu6_3 => (2720, 2272),
            OrchardCircuitVersion::ZSA => (2848, 2272),
        }
    }
}
