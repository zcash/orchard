use group::GroupEncoding;
use halo2_proofs::arithmetic::CurveExt;
use pasta_curves::pallas;
use std::hash::{Hash, Hasher};

use subtle::{Choice, ConstantTimeEq, CtOption};

use crate::constants::fixed_bases::{VALUE_COMMITMENT_PERSONALIZATION, VALUE_COMMITMENT_V_BYTES};
use crate::keys::IssuerValidatingKey;

/// Note type identifier.
#[derive(Clone, Copy, Debug, Eq)]
pub struct NoteType(pallas::Point);

pub const MAX_ASSET_DESCRIPTION_SIZE: usize = 512;

// the hasher used to derive the assetID
#[allow(non_snake_case)]
fn assetID_hasher(msg: Vec<u8>) -> pallas::Point {
    // TODO(zsa) replace personalization
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
    pub fn derive(ik: &IssuerValidatingKey, asset_desc: &str) -> Self {
        assert!(!asset_desc.is_empty() && asset_desc.len() <= MAX_ASSET_DESCRIPTION_SIZE);

        let mut s = vec![];
        s.extend(ik.to_bytes());
        s.extend(asset_desc.as_bytes());

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

impl Hash for NoteType {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write(&self.to_bytes());
        h.finish();
    }
}

impl PartialEq for NoteType {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use super::NoteType;

    use proptest::prelude::*;

    use crate::keys::{testing::arb_spending_key, IssuerAuthorizingKey, IssuerValidatingKey};

    prop_compose! {
        /// Generate a uniformly distributed note type
        pub fn arb_note_type()(
            is_native in prop::bool::ANY,
            sk in arb_spending_key(),
            // bytes32a in prop::array::uniform32(prop::num::u8::ANY),
            // bytes32b in prop::array::uniform32(prop::num::u8::ANY),
            vec in prop::collection::vec(any::<u8>(), 0..=255),
        ) -> NoteType {
            if is_native {
                NoteType::native()
            } else {
                //let bytes64 = [bytes32a, bytes32b].concat();
                let asset_desc = String::from_utf8(vec).unwrap();
                let isk = IssuerAuthorizingKey::from(&sk);
                NoteType::derive(&IssuerValidatingKey::from(&isk), asset_desc.as_str())
            }
        }
    }

    prop_compose! {
        /// Generate the native note type
        pub fn native_note_type()(_i in 0..10) -> NoteType {
            // TODO: remove _i
            NoteType::native()
        }
    }
}
