use core::iter;

use alloc::vec::Vec;
use bitvec::{array::BitArray, order::Lsb0};
use group::ff::{PrimeField, PrimeFieldBits};
use pasta_curves::pallas;
use subtle::{ConditionallySelectable, ConstantTimeEq, CtOption};

use crate::{
    constants::{
        fixed_bases::{NOTE_COMMITMENT_PERSONALIZATION, NOTE_ZSA_COMMITMENT_PERSONALIZATION},
        L_ORCHARD_BASE,
    },
    note::asset_base::AssetBase,
    spec::extract_p,
    value::NoteValue,
};

#[derive(Clone, Debug)]
pub(crate) struct NoteCommitTrapdoor(pub(crate) pallas::Scalar);

impl NoteCommitTrapdoor {
    pub(crate) fn inner(&self) -> pallas::Scalar {
        self.0
    }
}

/// A commitment to a note.
#[derive(Clone, Debug)]
pub struct NoteCommitment(pub(super) pallas::Point);

impl NoteCommitment {
    pub(crate) fn inner(&self) -> pallas::Point {
        self.0
    }
}

impl NoteCommitment {
    /// $NoteCommit^{Orchard}$ when the asset is zatoshi,
    /// and $NoteCommit^{OrchardZSA}$ otherwise.
    ///
    /// $NoteCommit^{Orchard}$ is defined in
    /// [Zcash Protocol Spec § 5.4.8.4: Sinsemilla commitments][concretesinsemillacommit].
    /// $NoteCommit^{OrchardZSA}$ is defined in
    /// [ZIP-226: Transfer and Burn of Zcash Shielded Assets][notecommitzsa].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    /// [notecommitzsa]: https://zips.z.cash/zip-0226#note-structure-commitment
    pub(crate) fn derive(
        g_d: [u8; 32],
        pk_d: [u8; 32],
        v: NoteValue,
        asset: AssetBase,
        rho: pallas::Base,
        psi: pallas::Base,
        rcm: NoteCommitTrapdoor,
    ) -> CtOption<Self> {
        let common_note_bits = iter::empty()
            .chain(BitArray::<_, Lsb0>::new(g_d).iter().by_vals())
            .chain(BitArray::<_, Lsb0>::new(pk_d).iter().by_vals())
            .chain(v.to_le_bits().iter().by_vals())
            .chain(rho.to_le_bits().iter().by_vals().take(L_ORCHARD_BASE))
            .chain(psi.to_le_bits().iter().by_vals().take(L_ORCHARD_BASE))
            .collect::<Vec<bool>>();

        let zec_note_bits = common_note_bits.clone().into_iter();

        let asset_bits = BitArray::<_, Lsb0>::new(asset.to_bytes());
        let zsa_note_bits = common_note_bits
            .into_iter()
            .chain(asset_bits.iter().by_vals());

        // Evaluate ZEC note commitment
        let zec_domain = sinsemilla::CommitDomain::new(NOTE_COMMITMENT_PERSONALIZATION);
        let commit_with_zec_domain = zec_domain.commit(zec_note_bits, &rcm.0);

        // Evaluate ZSA note commitment
        let zsa_domain = sinsemilla::CommitDomain::new_with_separate_domains(
            NOTE_ZSA_COMMITMENT_PERSONALIZATION,
            NOTE_COMMITMENT_PERSONALIZATION,
        );
        let commit_with_zsa_domain = zsa_domain.commit(zsa_note_bits, &rcm.0);

        // Select the desired commitment in constant-time
        let commit = commit_with_zsa_domain.and_then(|zsa_commit| {
            commit_with_zec_domain.map(|zec_commit| {
                pallas::Point::conditional_select(&zsa_commit, &zec_commit, asset.is_zatoshi())
            })
        });

        commit.map(NoteCommitment)
    }
}

/// The x-coordinate of the commitment to a note.
#[derive(Copy, Clone, Debug)]
pub struct ExtractedNoteCommitment(pub(super) pallas::Base);

impl ExtractedNoteCommitment {
    /// Deserialize the extracted note commitment from a byte array.
    ///
    /// This method enforces the [consensus rule][cmxcanon] that the
    /// byte representation of cmx MUST be canonical.
    ///
    /// [cmxcanon]: https://zips.z.cash/protocol/protocol.pdf#actionencodingandconsensus
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Base::from_repr(*bytes).map(ExtractedNoteCommitment)
    }

    /// Serialize the value commitment to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_repr()
    }
}

impl From<NoteCommitment> for ExtractedNoteCommitment {
    fn from(cm: NoteCommitment) -> Self {
        ExtractedNoteCommitment(extract_p(&cm.0))
    }
}

impl ExtractedNoteCommitment {
    pub(crate) fn inner(&self) -> pallas::Base {
        self.0
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
