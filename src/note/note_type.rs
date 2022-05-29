use group::GroupEncoding;
use halo2_proofs::arithmetic::CurveExt;
use pasta_curves::pallas;
use subtle::{Choice, ConstantTimeEq, CtOption};

use crate::constants::fixed_bases::{VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_V_BYTES};
use crate::keys::SpendValidatingKey;

/// Note type identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteType(pallas::Point);

// the hasher used to derive the assetID
#[allow(non_snake_case)]
fn assetID_hasher(msg: Vec<u8>) -> pallas::Point {
    pallas::Point::hash_to_curve(VALUE_COMMITMENT_PERSONALIZATION)(&msg)
}

impl NoteType {
    /// Deserialize the note_type from a byte array.
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Point::from_bytes(bytes).map(NoteType)
    }

    /// Serialize the note_type to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// $DeriveNoteType$.
    ///
    /// Defined in [Zcash Protocol Spec § TBD: Note Types][notetypes].
    ///
    /// [notetypes]: https://zips.z.cash/protocol/nu5.pdf#notetypes
    #[allow(non_snake_case)]
    pub(super) fn derive(ak: &SpendValidatingKey, assetDesc: &[u8; 64]) -> Self {
        let mut s = vec![];

        s.extend(ak.to_bytes());
        s.extend(assetDesc);

        NoteType(assetID_hasher(s))
    }

    /// Note type for the "native" currency (zec), maintains backward compatibility with Orchard untyped notes.
    pub fn native() -> Self {
        NoteType(assetID_hasher(VALUE_COMMITMENT_V_BYTES.to_vec()))
    }

    /// The base point used in value commitments.
    pub fn cv_base(&self) -> pallas::Point {
        self.0
    }

    /// Whether this note represents a native or ZSA asset.
    pub fn is_native(&self) -> Choice {
        self.0.ct_eq(&Self::native().0)
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use proptest::prelude::*;

    use crate::keys::{testing::arb_spending_key, FullViewingKey};

    use super::NoteType;

    prop_compose! {
        /// Generate a uniformly distributed note type
        pub fn arb_note_type()(
            sk in arb_spending_key(),
            bytes32a in prop::array::uniform32(prop::num::u8::ANY),
            bytes32b in prop::array::uniform32(prop::num::u8::ANY),
        ) -> NoteType {
            let bytes64 = [bytes32a, bytes32b].concat();
            let fvk = FullViewingKey::from(&sk);
            NoteType::derive(&fvk.into(), &bytes64.try_into().unwrap())
        }
    }
}
