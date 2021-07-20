use std::iter;

use bitvec::{array::BitArray, order::Lsb0};
use ff::PrimeFieldBits;
use pasta_curves::{arithmetic::FieldExt, pallas};
use subtle::{ConstantTimeEq, CtOption};

use crate::{
    constants::{L_ORCHARD_BASE, NOTE_COMMITMENT_PERSONALIZATION},
    primitives::sinsemilla,
    spec::extract_p,
    value::NoteValue,
};

pub(super) struct NoteCommitTrapdoor(pub(super) pallas::Scalar);

/// A commitment to a note.
#[derive(Debug)]
pub struct NoteCommitment(pub(super) pallas::Point);

impl NoteCommitment {
    /// $NoteCommit^Orchard$.
    ///
    /// Defined in [Zcash Protocol Spec § 5.4.8.4: Sinsemilla commitments][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub(super) fn derive(
        g_d: [u8; 32],
        pk_d: [u8; 32],
        v: NoteValue,
        rho: pallas::Base,
        psi: pallas::Base,
        rcm: NoteCommitTrapdoor,
    ) -> CtOption<Self> {
        let domain = sinsemilla::CommitDomain::new(NOTE_COMMITMENT_PERSONALIZATION);
        domain
            .commit(
                iter::empty()
                    .chain(BitArray::<Lsb0, _>::new(g_d).iter().by_val())
                    .chain(BitArray::<Lsb0, _>::new(pk_d).iter().by_val())
                    .chain(v.to_le_bits().iter().by_val())
                    .chain(rho.to_le_bits().iter().by_val().take(L_ORCHARD_BASE))
                    .chain(psi.to_le_bits().iter().by_val().take(L_ORCHARD_BASE)),
                &rcm.0,
            )
            .map(NoteCommitment)
    }
}

/// The x-coordinate of the commitment to a note.
#[derive(Copy, Clone, Debug)]
pub struct ExtractedNoteCommitment(pub(super) pallas::Base);

impl ExtractedNoteCommitment {
    /// Deserialize the extracted note commitment from a byte array.
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Base::from_bytes(bytes).map(ExtractedNoteCommitment)
    }

    /// Serialize the value commitment to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl From<NoteCommitment> for ExtractedNoteCommitment {
    fn from(cm: NoteCommitment) -> Self {
        ExtractedNoteCommitment(extract_p(&cm.0))
    }
}

impl std::ops::Deref for ExtractedNoteCommitment {
    type Target = pallas::Base;

    fn deref(&self) -> &pallas::Base {
        &self.0
    }
}

impl From<&ExtractedNoteCommitment> for [u8; 32] {
    fn from(cmx: &ExtractedNoteCommitment) -> Self {
        cmx.to_bytes()
    }
}

impl ConstantTimeEq for ExtractedNoteCommitment {
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        self.0.ct_eq(&other.0)
    }
}

impl PartialEq for ExtractedNoteCommitment {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for ExtractedNoteCommitment {}
