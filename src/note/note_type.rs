use group::ff::PrimeField;
use halo2_proofs::arithmetic::CurveExt;
use pasta_curves::{pallas};
use subtle::CtOption;

use crate::constants::fixed_bases::{VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_V_BYTES};
use crate::keys::SpendValidatingKey;
use crate::spec::extract_p;

/// Note type identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoteType(pub(crate) pallas::Base);

// the hasher used to derive the assetID
#[allow(non_snake_case)]
fn assetID_hasher(msg: Vec<u8>) -> pallas::Base {
    let hasher = pallas::Point::hash_to_curve(VALUE_COMMITMENT_PERSONALIZATION);
    extract_p(&hasher(msg.as_bytes())))
}

impl NoteType {

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
    pub(super) fn derive(ak: &SpendValidatingKey, assetDesc: &[u8; 64]) -> Self {
        let mut s = vec![];

        s.extend_from_slice(&ak.to_bytes());
        s.extend_from_slice(assetDesc);

        NoteType(assetID_hasher(s))
    }

    /// Note type for the "native" currency (zec), maintains backward compatibility with Orchard untyped notes.
    #[allow(non_snake_case)]
    pub fn native() -> Self {
        NoteType(assetID_hasher(VALUE_COMMITMENT_V_BYTES.to_vec()))
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
