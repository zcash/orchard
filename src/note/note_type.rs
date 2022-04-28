use group::{ff::PrimeField};
use halo2_proofs::arithmetic::CurveExt;
use pasta_curves::pallas;
use subtle::CtOption;

use crate:: spec::{extract_p};
use crate::constants::fixed_bases::{VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_V_BYTES};
// use crate::keys::SpendValidatingKey;

/// Note type identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoteType(pub(crate) pallas::Base);

impl NoteType {
    /*
    /// Generates a dummy note type for use as $\rho$ in dummy spent notes.
    pub(crate) fn dummy(rng: &mut impl RngCore) -> Self {
    NoteType(extract_p(&pallas::Point::random(rng)))
    }
    */

    /// Deserialize the note_type from a byte array.
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Base::from_repr(*bytes).map(NoteType)
    }

    /// Serialize the note_type to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_repr()
    }

    /// $DeriveNoteType$.
    ///
    /// Defined in [Zcash Protocol Spec § TBD: Note Types][notetypes].
    ///
    /// [notetypes]: https://zips.z.cash/protocol/nu5.pdf#notetypes
    #[allow(non_snake_case)]
    pub(super) fn derive(
        asset_idx: u64
    ) -> Self {
        let hasher = pallas::Point::hash_to_curve(VALUE_COMMITMENT_PERSONALIZATION);
        let V = hasher(&VALUE_COMMITMENT_V_BYTES);

        let value = pallas::Scalar::from(asset_idx);

        NoteType(extract_p(&(V * value)))
    }

    /// note type for the "native" token (zec)
    pub fn native() -> Self {
        Self::derive(1)
    }

}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use group::Group;
    use pasta_curves::{arithmetic::FieldExt, pallas};
    use proptest::collection::vec;
    use proptest::prelude::*;
    use std::convert::TryFrom;

    use super::NoteType;
    use crate::spec::extract_p;

    prop_compose! {
        /// Generate a uniformly distributed note type
        pub fn arb_nullifier()(
            bytes in vec(any::<u8>(), 64)
        ) -> NoteType {
            let point = pallas::Point::generator() * pallas::Scalar::from_bytes_wide(&<[u8; 64]>::try_from(bytes).unwrap());
            NoteType(extract_p(&point))
        }
    }
}
