use group::GroupEncoding;
use halo2_proofs::arithmetic::CurveExt;
use pasta_curves::pallas;
use subtle::CtOption;

use crate::constants::fixed_bases::{VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_V_BYTES};
use crate::keys::IssuerValidatingKey;

/// Note type identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteType(pub(crate) pallas::Point);

const MAX_ASSET_DESCRIPTION_SIZE: usize = 512;

// the hasher used to derive the assetID
#[allow(non_snake_case)]
fn assetID_hasher(msg: Vec<u8>) -> pallas::Point {
    // TODO(zsa) replace personalization, will require circuit change?
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
    pub fn derive(ik: &IssuerValidatingKey, assetDesc: Vec<u8>) -> Self {
        assert!(assetDesc.len() < MAX_ASSET_DESCRIPTION_SIZE);

        let mut s = vec![];
        s.extend(ik.to_bytes());
        s.extend(assetDesc);

        NoteType(assetID_hasher(s))
    }

    /// Note type for the "native" currency (zec), maintains backward compatibility with Orchard untyped notes.
    pub fn native() -> Self {
        NoteType(assetID_hasher(VALUE_COMMITMENT_V_BYTES.to_vec()))
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use proptest::prelude::*;

    use super::NoteType;

    use crate::keys::{testing::arb_spending_key, IssuerAuthorizingKey, IssuerValidatingKey};

    prop_compose! {
        /// Generate a uniformly distributed note type
        pub fn arb_note_type()(
            sk in arb_spending_key(),
            bytes32a in prop::array::uniform32(prop::num::u8::ANY),
            bytes32b in prop::array::uniform32(prop::num::u8::ANY),
        ) -> NoteType {
            let bytes64 = [bytes32a, bytes32b].concat();
            let isk = IssuerAuthorizingKey::from(&sk);
            NoteType::derive(&IssuerValidatingKey::from(&isk), bytes64)
        }
    }
}
