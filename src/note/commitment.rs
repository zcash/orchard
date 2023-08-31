use core::iter;

use bitvec::{array::BitArray, order::Lsb0};
use group::ff::{PrimeField, PrimeFieldBits};
use halo2_gadgets::sinsemilla::primitives as sinsemilla;
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
    /// $NoteCommit^Orchard$.
    ///
    /// Defined in [Zcash Protocol Spec § 5.4.8.4: Sinsemilla commitments][concretesinsemillacommit].
    ///
    /// [concretesinsemillacommit]: https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit
    pub(crate) fn derive(
        g_d: [u8; 32],
        pk_d: [u8; 32],
        v: NoteValue,
        asset: AssetBase,
        rho: pallas::Base,
        psi: pallas::Base,
        rcm: NoteCommitTrapdoor,
    ) -> CtOption<Self> {
        let g_d_bits = BitArray::<_, Lsb0>::new(g_d);
        let pk_d_bits = BitArray::<_, Lsb0>::new(pk_d);
        let v_bits = v.to_le_bits();
        let rho_bits = rho.to_le_bits();
        let psi_bits = psi.to_le_bits();

        let common_note_bits = iter::empty()
            .chain(g_d_bits.iter().by_vals())
            .chain(pk_d_bits.iter().by_vals())
            .chain(v_bits.iter().by_vals())
            .chain(rho_bits.iter().by_vals().take(L_ORCHARD_BASE))
            .chain(psi_bits.iter().by_vals().take(L_ORCHARD_BASE))
            .collect::<Vec<bool>>();

        let zec_note_bits = common_note_bits.clone().into_iter();

        let type_bits = BitArray::<_, Lsb0>::new(asset.to_bytes());
        let zsa_note_bits = common_note_bits
            .into_iter()
            .chain(type_bits.iter().by_vals());

        let zec_domain = sinsemilla::CommitDomain::new(NOTE_COMMITMENT_PERSONALIZATION);
        let zsa_domain = sinsemilla::CommitDomain::new_with_personalization(
            NOTE_ZSA_COMMITMENT_PERSONALIZATION,
            NOTE_COMMITMENT_PERSONALIZATION,
        );

        let zec_hash_point = zec_domain.M.hash_to_point(zec_note_bits);
        let zsa_hash_point = zsa_domain.M.hash_to_point(zsa_note_bits);

        // Select the desired hash point in constant-time
        let hash_point = zsa_hash_point.and_then(|zsa_hash| {
            zec_hash_point.map(|zec_hash| {
                pallas::Point::conditional_select(&zsa_hash, &zec_hash, asset.is_native())
            })
        });

        // To evaluate the commitment from the hash_point, we could use either zec_domain or
        // zsa_domain because they have both the same `R` constant.
        zec_domain
            .commit_from_hash_point(hash_point, &rcm.0)
            .map(NoteCommitment)
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

#[cfg(test)]
mod tests {
    use crate::constants::fixed_bases::{
        NOTE_COMMITMENT_PERSONALIZATION, NOTE_ZSA_COMMITMENT_PERSONALIZATION,
    };
    use crate::note::commitment::NoteCommitTrapdoor;
    use ff::Field;
    use halo2_gadgets::sinsemilla::primitives as sinsemilla;
    use pasta_curves::pallas;
    use rand::{rngs::OsRng, Rng};

    #[test]
    fn test_commit_in_several_steps() {
        let mut os_rng = OsRng::default();
        let msg: Vec<bool> = (0..36).map(|_| os_rng.gen::<bool>()).collect();

        let rcm = NoteCommitTrapdoor(pallas::Scalar::random(&mut os_rng));

        let domain_zec = sinsemilla::CommitDomain::new(NOTE_COMMITMENT_PERSONALIZATION);
        let domain_zsa = sinsemilla::CommitDomain::new_with_personalization(
            NOTE_ZSA_COMMITMENT_PERSONALIZATION,
            NOTE_COMMITMENT_PERSONALIZATION,
        );

        let expected_commit = domain_zsa.commit(msg.clone().into_iter(), &rcm.0);

        // Evaluating the commitment in one step with `commit` or in two steps with `hash_to_point`
        // and `commit_from_hash_point` must give the same commitment.
        let hash_point = domain_zsa.M.hash_to_point(msg.into_iter());
        let commit_r_zsa = domain_zsa.commit_from_hash_point(hash_point, &rcm.0);
        assert_eq!(expected_commit.unwrap(), commit_r_zsa.unwrap());

        // ZEC and ZSA note commitments must use the same R constant
        assert_eq!(domain_zec.R(), domain_zsa.R());
    }
}
