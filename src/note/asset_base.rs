use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use group::{Group, GroupEncoding};
use pasta_curves::{arithmetic::CurveExt, pallas};
use subtle::{Choice, ConstantTimeEq, CtOption};

use crate::constants::fixed_bases::{VALUE_COMMITMENT_PERSONALIZATION, ZATOSHI_ASSET_BASE_V_BYTES};

#[cfg(test)]
use rand_core::CryptoRngCore;

#[cfg(feature = "zsa-issuance")]
use {
    crate::constants::fixed_bases::ZSA_ASSET_BASE_PERSONALIZATION,
    crate::issuance::auth::{IssueValidatingKey, ZSASchnorr},
    alloc::vec::Vec,
    blake2b_simd::{Hash as Blake2bHash, Params},
};

/// Asset Identifier
#[cfg(feature = "zsa-issuance")]
#[derive(Debug)]
pub enum AssetId<'a> {
    /// Version V0 of AssetId
    V0 {
        /// Issue validating Key
        ik: &'a IssueValidatingKey<ZSASchnorr>,
        /// Asset description hash
        asset_desc_hash: &'a [u8; 32],
    },
}

#[cfg(feature = "zsa-issuance")]
impl<'a> AssetId<'a> {
    /// Generates a new V0 AssetId.
    pub fn new_v0(ik: &'a IssueValidatingKey<ZSASchnorr>, asset_desc_hash: &'a [u8; 32]) -> Self {
        AssetId::V0 {
            ik,
            asset_desc_hash,
        }
    }

    /// Encoding the Asset Identifier, as defined in [ZIP 227][assetidentifier].
    ///
    /// [assetidentifier]: https://zips.z.cash/zip-0227.html#specification-asset-identifier-asset-digest-and-asset-base
    fn encode_asset_id(&self) -> Vec<u8> {
        match self {
            AssetId::V0 {
                ik,
                asset_desc_hash,
            } => {
                let issuer = ik.encode();
                let mut asset_id = Vec::with_capacity(1 + issuer.len() + asset_desc_hash.len());
                asset_id.push(0u8); // version
                asset_id.extend(issuer);
                asset_id.extend_from_slice(&asset_desc_hash[..]);
                asset_id
            }
        }
    }

    /// Derives the Asset Digest for this ZSA asset.
    ///
    /// Defined in [ZIP-227: Issuance of Zcash Shielded Assets][assetdigest].
    ///
    /// [assetdigest]: https://zips.z.cash/zip-0227#asset-digests
    fn asset_digest(&self) -> Blake2bHash {
        Params::new()
            .hash_length(64)
            .personal(ZSA_ASSET_DIGEST_PERSONALIZATION)
            .to_state()
            .update(&self.encode_asset_id())
            .finalize()
    }
}

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
        self.0.to_bytes().cmp(&other.0.to_bytes())
    }
}

/// Personalization for the ZSA asset digest generator
#[cfg(feature = "zsa-issuance")]
pub const ZSA_ASSET_DIGEST_PERSONALIZATION: &[u8; 16] = b"ZSA-Asset-Digest";

impl AssetBase {
    /// Deserialize the AssetBase from a byte array.
    ///
    /// Returns `None` if the byte encoding is invalid or if it corresponds
    /// to the identity point.
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Point::from_bytes(bytes)
            .and_then(|asset| CtOption::new(AssetBase(asset), !asset.is_identity()))
    }

    /// Serialize the AssetBase to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Note type derivation.
    ///
    /// Defined in [ZIP 227: Issuance of Zcash Shielded Assets][assetbase].
    ///
    /// [assetbase]: https://zips.z.cash/zip-0227#asset-bases
    ///
    /// # Panics
    ///
    /// Panics if the derived AssetBase is the identity point.
    #[cfg(feature = "zsa-issuance")]
    #[allow(non_snake_case)]
    pub fn custom(asset_id: &AssetId<'_>) -> Self {
        let asset_digest = asset_id.asset_digest();

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

    /// Note type for zatoshi, maintains backward compatibility with Orchard untyped notes.
    pub fn zatoshi() -> Self {
        AssetBase(pallas::Point::hash_to_curve(
            VALUE_COMMITMENT_PERSONALIZATION,
        )(&ZATOSHI_ASSET_BASE_V_BYTES))
    }

    /// The base point used in value commitments.
    pub fn cv_base(&self) -> pallas::Point {
        self.0
    }

    /// Whether this note represents zatoshi or ZSA asset.
    pub fn is_zatoshi(&self) -> Choice {
        self.0.ct_eq(&Self::zatoshi().0)
    }

    /// Generates a ZSA random asset from a random non-identity Pallas point.
    ///
    /// Normally, an `AssetBase` is derived from an issuance validating key. For testing purposes,
    /// it is sufficient to use a random non-identity Pallas point. This allows generating a random
    /// `AssetBase` even when `zsa-issuance` feature is disabled.
    ///
    /// This is only used in tests.
    #[cfg(test)]
    pub(crate) fn random(rng: &mut impl CryptoRngCore) -> Self {
        loop {
            let random_point = pallas::Point::random(&mut *rng);
            // Extremely unlikely, but we explicitly reject the identity point.
            if bool::from(random_point.is_identity()) {
                continue;
            }
            return Self(random_point);
        }
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

    use crate::constants::fixed_bases::ZSA_ASSET_BASE_PERSONALIZATION;
    use group::Group;
    use pasta_curves::{arithmetic::CurveExt, pallas};

    prop_compose! {
        /// Generate a uniformly distributed asset base.
        pub fn arb_asset_base()
            (asset in prop_oneof![
                Just(AssetBase::zatoshi()),
                arb_zsa_asset_base(),
            ])
            -> AssetBase
        {
            asset
        }
    }

    prop_compose! {
        /// Generates a ZSA asset base from a random asset digest.
        ///
        /// Normally, an `AssetBase` is derived from an issuance validating key. For testing purposes,
        /// it is sufficient to use a random asset digest. This allows generating a random
        /// `AssetBase` even when `zsa-issuance` feature is disabled.
        pub fn arb_zsa_asset_base()(
            asset_digest in any::<[u8; 64]>().prop_filter("hash_to_curve must not be identity", |digest| {
                // Reject the extremely unlikely case where `hash_to_curve` returns the
                // identity point. `prop_filter` makes proptest discard such inputs and
                // regenerate new ones instead of failing the test.
                let asset_base = pallas::Point::hash_to_curve(ZSA_ASSET_BASE_PERSONALIZATION)(digest);
                !bool::from(asset_base.is_identity())
            })
        ) -> AssetBase {
            let asset_base = pallas::Point::hash_to_curve(ZSA_ASSET_BASE_PERSONALIZATION)(&asset_digest);
            AssetBase(asset_base)
        }
    }
}

#[cfg(test)]
#[cfg(feature = "zsa-issuance")]
mod tests {
    use crate::{
        issuance::auth::{IssueValidatingKey, ZSASchnorr},
        note::{AssetBase, AssetId},
    };

    #[test]
    fn test_vectors() {
        let test_vectors = crate::test_vectors::asset_base::TEST_VECTORS;

        for tv in test_vectors {
            let asset_desc_hash = crate::issuance::compute_asset_desc_hash(
                &nonempty::NonEmpty::from_slice(&tv.description).unwrap(),
            );
            let calculated_asset_base = AssetBase::custom(&AssetId::new_v0(
                &IssueValidatingKey::<ZSASchnorr>::decode(&tv.key).unwrap(),
                &asset_desc_hash,
            ));
            let test_vector_asset_base = AssetBase::from_bytes(&tv.asset_base).unwrap();

            assert_eq!(calculated_asset_base, test_vector_asset_base);
        }
    }
}
