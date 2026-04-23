//! Orchard fixed bases.

#[cfg(feature = "circuit")]
use alloc::vec::Vec;

use super::{L_ORCHARD_SCALAR, L_VALUE};

#[cfg(feature = "circuit")]
use halo2_gadgets::ecc::{
    chip::{BaseFieldElem, FixedPoint, FullScalar, ShortScalar},
    FixedPoints,
};

#[cfg(feature = "circuit")]
use pasta_curves::pallas;

/// Precomputed table for the `CommitIvk` commitment randomness base.
pub mod commit_ivk_r;
/// Precomputed table for the `NoteCommit` commitment randomness base.
pub mod note_commit_r;
/// Precomputed table for the nullifier base `K^Orchard`.
pub mod nullifier_k;
/// Precomputed table for the spend authorization base `G^Orchard`.
pub mod spend_auth_g;
/// Precomputed table for the value commitment randomness base.
pub mod value_commit_r;
/// Precomputed table for the value commitment value base.
pub mod value_commit_v;

/// SWU hash-to-curve personalization for the spending key base point and
/// the nullifier base point K^Orchard
pub const ORCHARD_PERSONALIZATION: &str = "z.cash:Orchard";

/// SWU hash-to-curve personalization for the value commitment generator
pub const VALUE_COMMITMENT_PERSONALIZATION: &str = "z.cash:Orchard-cv";

/// SWU hash-to-curve value for the value commitment generator
pub const VALUE_COMMITMENT_V_BYTES: [u8; 1] = *b"v";

/// SWU hash-to-curve value for the value commitment generator
pub const VALUE_COMMITMENT_R_BYTES: [u8; 1] = *b"r";

/// SWU hash-to-curve personalization for the note commitment generator
pub const NOTE_COMMITMENT_PERSONALIZATION: &str = "z.cash:Orchard-NoteCommit";

/// SWU hash-to-curve personalization for the IVK commitment generator
pub const COMMIT_IVK_PERSONALIZATION: &str = "z.cash:Orchard-CommitIvk";

/// Window size for fixed-base scalar multiplication
pub const FIXED_BASE_WINDOW_SIZE: usize = 3;

/// $2^{`FIXED_BASE_WINDOW_SIZE`}$
pub const H: usize = 1 << FIXED_BASE_WINDOW_SIZE;

/// Number of windows for a full-width scalar
pub const NUM_WINDOWS: usize = L_ORCHARD_SCALAR.div_ceil(FIXED_BASE_WINDOW_SIZE);

/// Number of windows for a short signed scalar
pub const NUM_WINDOWS_SHORT: usize = L_VALUE.div_ceil(FIXED_BASE_WINDOW_SIZE);

/// Enumeration of every fixed base used in the Orchard circuit.
///
/// This enables the shared fixed-base scalar multiplication machinery in
/// `halo2_gadgets` to dispatch across full-width, base-field, and short-signed
/// bases using a single type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrchardFixedBases {
    /// A full-width scalar multiplication base.
    Full(OrchardFixedBasesFull),
    /// The nullifier base `K^Orchard`, used with a base-field scalar.
    NullifierK,
    /// The value commitment value base, used with a short signed scalar.
    ValueCommitV,
}

impl From<OrchardFixedBasesFull> for OrchardFixedBases {
    fn from(full_width_base: OrchardFixedBasesFull) -> Self {
        Self::Full(full_width_base)
    }
}

impl From<ValueCommitV> for OrchardFixedBases {
    fn from(_value_commit_v: ValueCommitV) -> Self {
        Self::ValueCommitV
    }
}

impl From<NullifierK> for OrchardFixedBases {
    fn from(_nullifier_k: NullifierK) -> Self {
        Self::NullifierK
    }
}

/// The Orchard fixed bases used in scalar mul with full-width scalars.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrchardFixedBasesFull {
    /// Randomness base for the `CommitIvk` commitment.
    CommitIvkR,
    /// Randomness base for the `NoteCommit` commitment.
    NoteCommitR,
    /// Randomness base for value commitments.
    ValueCommitR,
    /// Spend authorization base `G^Orchard`.
    SpendAuthG,
}

/// NullifierK is used in scalar mul with a base field element.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NullifierK;

/// ValueCommitV is used in scalar mul with a short signed scalar.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ValueCommitV;

#[cfg(feature = "circuit")]
impl FixedPoints<pallas::Affine> for OrchardFixedBases {
    type FullScalar = OrchardFixedBasesFull;
    type Base = NullifierK;
    type ShortScalar = ValueCommitV;
}

#[cfg(feature = "circuit")]
impl FixedPoint<pallas::Affine> for OrchardFixedBasesFull {
    type FixedScalarKind = FullScalar;

    fn generator(&self) -> pallas::Affine {
        match self {
            Self::CommitIvkR => commit_ivk_r::generator(),
            Self::NoteCommitR => note_commit_r::generator(),
            Self::ValueCommitR => value_commit_r::generator(),
            Self::SpendAuthG => spend_auth_g::generator(),
        }
    }

    fn u(&self) -> Vec<[[u8; 32]; H]> {
        match self {
            Self::CommitIvkR => commit_ivk_r::U.to_vec(),
            Self::NoteCommitR => note_commit_r::U.to_vec(),
            Self::ValueCommitR => value_commit_r::U.to_vec(),
            Self::SpendAuthG => spend_auth_g::U.to_vec(),
        }
    }

    fn z(&self) -> Vec<u64> {
        match self {
            Self::CommitIvkR => commit_ivk_r::Z.to_vec(),
            Self::NoteCommitR => note_commit_r::Z.to_vec(),
            Self::ValueCommitR => value_commit_r::Z.to_vec(),
            Self::SpendAuthG => spend_auth_g::Z.to_vec(),
        }
    }
}

#[cfg(feature = "circuit")]
impl FixedPoint<pallas::Affine> for NullifierK {
    type FixedScalarKind = BaseFieldElem;

    fn generator(&self) -> pallas::Affine {
        nullifier_k::generator()
    }

    fn u(&self) -> Vec<[[u8; 32]; H]> {
        nullifier_k::U.to_vec()
    }

    fn z(&self) -> Vec<u64> {
        nullifier_k::Z.to_vec()
    }
}

#[cfg(feature = "circuit")]
impl FixedPoint<pallas::Affine> for ValueCommitV {
    type FixedScalarKind = ShortScalar;

    fn generator(&self) -> pallas::Affine {
        value_commit_v::generator()
    }

    fn u(&self) -> Vec<[[u8; 32]; H]> {
        value_commit_v::U_SHORT.to_vec()
    }

    fn z(&self) -> Vec<u64> {
        value_commit_v::Z_SHORT.to_vec()
    }
}
