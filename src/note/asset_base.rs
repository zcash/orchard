use alloc::vec::Vec;
use blake2b_simd::{Hash as Blake2bHash, Params};
use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use group::{Curve, Group, GroupEncoding};
use nonempty::NonEmpty;
use pasta_curves::arithmetic::CurveAffine;
use pasta_curves::{arithmetic::CurveExt, pallas};
use rand_core::CryptoRngCore;
use subtle::{Choice, ConstantTimeEq, CtOption};

use crate::issuance_auth::ZSASchnorr;
use crate::{
    constants::fixed_bases::{
        NATIVE_ASSET_BASE_V_BYTES, VALUE_COMMITMENT_PERSONALIZATION, ZSA_ASSET_BASE_PERSONALIZATION,
    },
    issuance::compute_asset_desc_hash,
    issuance_auth::{IssueAuthKey, IssueValidatingKey},
};

/// Note type identifier.
#[derive(Clone, Copy, Debug, Eq)]
pub struct AssetBase(pallas::Point);

// AssetBase must implement PartialOrd and Ord to be used as a key in BTreeMap.
impl PartialOrd for AssetBase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AssetBase {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_coord = self.0.to_affine().coordinates().unwrap();
        let other_coord = other.0.to_affine().coordinates().unwrap();
        self_coord
            .x()
            .cmp(other_coord.x())
            .then_with(|| self_coord.y().cmp(other_coord.y()))
    }
}

/// Personalization for the ZSA asset digest generator
pub const ZSA_ASSET_DIGEST_PERSONALIZATION: &[u8; 16] = b"ZSA-Asset-Digest";

///    AssetDigest for the ZSA asset
///
///    Defined in [ZIP-227: Issuance of Zcash Shielded Assets][assetdigest].
///
///    [assetdigest]: https://zips.z.cash/zip-0227.html#specification-asset-identifier-asset-digest-and-asset-base
pub fn asset_digest(encode_asset_id: &[u8]) -> Blake2bHash {
    Params::new()
        .hash_length(64)
        .personal(ZSA_ASSET_DIGEST_PERSONALIZATION)
        .to_state()
        .update(encode_asset_id)
        .finalize()
}

/// Encoding the Asset Identifier, as defined in [ZIP 227][assetidentifier].
///
/// [assetidentifier]: https://zips.z.cash/zip-0227.html#specification-asset-identifier-asset-digest-and-asset-base
pub fn encode_asset_id(
    version: u8,
    ik: &IssueValidatingKey<ZSASchnorr>,
    asset_desc_hash: &[u8; 32],
) -> Vec<u8> {
    let ik_encoding = ik.encode();
    let mut asset_id = Vec::with_capacity(1 + ik_encoding.len() + asset_desc_hash.len());
    asset_id.push(version);
    asset_id.extend(ik_encoding);
    asset_id.extend_from_slice(&asset_desc_hash[..]);
    asset_id
}

impl AssetBase {
    /// Deserialize the AssetBase from a byte array.
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Point::from_bytes(bytes).map(AssetBase)
    }

    /// Serialize the AssetBase to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Note type derivation.
    ///
    /// Defined in [ZIP-226: Transfer and Burn of Zcash Shielded Assets][assetbase].
    ///
    /// [assetbase]: https://zips.z.cash/zip-0226.html#asset-identifiers
    ///
    /// # Panics
    ///
    /// Panics if the derived AssetBase is the identity point.
    #[allow(non_snake_case)]
    pub fn derive(ik: &IssueValidatingKey<ZSASchnorr>, asset_desc_hash: &[u8; 32]) -> Self {
        let version_byte: u8 = 0x00;

        // EncodeAssetId(ik, asset_desc_hash) = version_byte || ik || asset_desc_hash
        let asset_id = encode_asset_id(version_byte, ik, asset_desc_hash);
        let asset_digest = asset_digest(&asset_id);

        let asset_base =
            pallas::Point::hash_to_curve(ZSA_ASSET_BASE_PERSONALIZATION)(asset_digest.as_bytes());

        // this will happen with negligible probability.
        assert!(
            bool::from(!asset_base.is_identity()),
            "The Asset Base is the identity point, which is invalid."
        );

        // AssetBase = ZSAValueBase(AssetDigest)
        AssetBase(asset_base)
    }

    /// Note type for the "native" currency (zec), maintains backward compatibility with Orchard untyped notes.
    pub fn native() -> Self {
        AssetBase(pallas::Point::hash_to_curve(
            VALUE_COMMITMENT_PERSONALIZATION,
        )(&NATIVE_ASSET_BASE_V_BYTES[..]))
    }

    /// The base point used in value commitments.
    pub fn cv_base(&self) -> pallas::Point {
        self.0
    }

    /// Whether this note represents a native or ZSA asset.
    pub fn is_native(&self) -> Choice {
        self.0.ct_eq(&Self::native().0)
    }

    /// Generates a ZSA random asset.
    ///
    /// This is only used in tests.
    pub(crate) fn random(rng: &mut impl CryptoRngCore) -> Self {
        let isk = IssueAuthKey::<ZSASchnorr>::random(rng);
        let ik = IssueValidatingKey::from(&isk);
        AssetBase::derive(
            &ik,
            &compute_asset_desc_hash(&NonEmpty::from_slice(b"zsa_asset").unwrap()),
        )
    }
}

impl Hash for AssetBase {
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write(&self.to_bytes());
        h.finish();
    }
}

impl PartialEq for AssetBase {
    fn eq(&self, other: &Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use super::AssetBase;

    use proptest::prelude::*;

    use crate::issuance_auth::{testing::arb_issuance_authorizing_key, IssueValidatingKey};

    prop_compose! {
        /// Generate a uniformly distributed note type
        pub fn arb_asset_base()(
            is_native in prop::bool::ANY,
            isk in arb_issuance_authorizing_key(),
            asset_desc_hash in any::<[u8; 32]>(),
        ) -> AssetBase {
            if is_native {
                AssetBase::native()
            } else {
                AssetBase::derive(&IssueValidatingKey::from(&isk), &asset_desc_hash)
            }
        }
    }

    prop_compose! {
        /// Generate an asset ID
        pub fn arb_zsa_asset_base()(
            isk in arb_issuance_authorizing_key(),
            asset_desc_hash in any::<[u8; 32]>(),
        ) -> AssetBase {
            AssetBase::derive(&IssueValidatingKey::from(&isk), &asset_desc_hash)
        }
    }

    prop_compose! {
        /// Generate an asset ID using a specific description
        pub fn zsa_asset_base(asset_desc_hash: [u8; 32])(
            isk in arb_issuance_authorizing_key(),
        ) -> AssetBase {
            AssetBase::derive(&IssueValidatingKey::from(&isk), &asset_desc_hash)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        issuance_auth::{IssueValidatingKey, ZSASchnorr},
        note::AssetBase,
    };

    #[test]
    fn test_vectors() {
        let test_vectors = crate::test_vectors::asset_base::TEST_VECTORS;

        for tv in test_vectors {
            let asset_desc_hash = crate::issuance::compute_asset_desc_hash(
                &nonempty::NonEmpty::from_slice(&tv.description).unwrap(),
            );
            let calculated_asset_base = AssetBase::derive(
                &IssueValidatingKey::<ZSASchnorr>::decode(&tv.key).unwrap(),
                &asset_desc_hash,
            );
            let test_vector_asset_base = AssetBase::from_bytes(&tv.asset_base).unwrap();

            assert_eq!(calculated_asset_base, test_vector_asset_base);
        }
    }
}
