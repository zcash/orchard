use crate::{
    constants::{
        ASSET_IDENTIFIER_LENGTH, ASSET_IDENTIFIER_PERSONALIZATION, GH_FIRST_BLOCK,
        VALUE_COMMITMENT_GENERATOR_PERSONALIZATION,
    },
    value::ValueCommitment,
};
use blake2s_simd::Params as Blake2sParams;
use borsh::{BorshDeserialize, BorshSerialize};
use group::{cofactor::CofactorGroup, Group, GroupEncoding};
use pasta_curves::pallas;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

/// A type representing an asset type
#[derive(Debug, BorshSerialize, BorshDeserialize, Clone, Copy, Eq)]
pub struct AssetType {
    identifier: [u8; ASSET_IDENTIFIER_LENGTH], //32 byte asset type preimage
    #[borsh_skip]
    nonce: Option<u8>,
}

// Abstract type representing an asset
impl AssetType {
    /// Create a new AsstType from a unique asset name
    /// Not constant-time, uses rejection sampling
    pub fn new(name: &[u8]) -> Result<AssetType, ()> {
        let mut nonce = 0u8;
        loop {
            if let Some(asset_type) = AssetType::new_with_nonce(name, nonce) {
                return Ok(asset_type);
            }
            nonce = nonce.checked_add(1).ok_or(())?;
        }
    }

    /// Attempt to create a new AssetType from a unique asset name and fixed nonce
    /// Not yet constant-time; assume not-constant-time
    pub fn new_with_nonce(name: &[u8], nonce: u8) -> Option<AssetType> {
        use std::slice::from_ref;

        // Check the personalization is acceptable length
        assert_eq!(ASSET_IDENTIFIER_PERSONALIZATION.len(), 8);

        // Create a new BLAKE2s state for deriving the asset identifier
        // TODO: sinsemilla hasに置き換える?そのままでもいい気がしてきた
        let h = Blake2sParams::new()
            .hash_length(ASSET_IDENTIFIER_LENGTH)
            .personal(ASSET_IDENTIFIER_PERSONALIZATION)
            .to_state()
            .update(GH_FIRST_BLOCK)
            .update(name)
            .update(from_ref(&nonce))
            .finalize();

        // If the hash state is a valid asset identifier, use it
        if AssetType::hash_to_point(h.as_array()).is_some() {
            Some(AssetType {
                identifier: *h.as_array(),
                nonce: Some(nonce),
            })
        } else {
            None
        }
    }

    //Attempt to hash an identifier to a curve point
    // https://crypto.stackexchange.com/questions/102414/where-is-hash-to-curve-used
    fn hash_to_point(identifier: &[u8; ASSET_IDENTIFIER_LENGTH]) -> Option<pallas::Point> {
        //check the personalization is acceptable length
        assert_eq!(ASSET_IDENTIFIER_PERSONALIZATION.len(), 8);

        //Check to see that scalar field is 255 bits
        use ff::PrimeField;
        assert_eq!(pallas::Base::NUM_BITS, 255);

        let h = Blake2sParams::new()
            .hash_length(32)
            .personal(VALUE_COMMITMENT_GENERATOR_PERSONALIZATION)
            .to_state()
            .update(identifier)
            .finalize();

        let p = pallas::Point::from_bytes(h.as_array());
        if p.is_some().into() {
            let p = p.unwrap();
            let p_prime = CofactorGroup::clear_cofactor(&p);
            if p_prime.is_identity().into() {
                None
            } else {
                Some(p)
            }
        } else {
            None
        }
    }

    /// Return the identifier of this asset type
    pub fn get_identifier(&self) -> &[u8; ASSET_IDENTIFIER_LENGTH] {
        &self.identifier
    }

    /// Attempt to construct an asset type from an existing asset identifier
    pub fn from_identifier(identifier: &[u8; ASSET_IDENTIFIER_LENGTH]) -> Option<AssetType> {
        // Attempt to hash to point
        if AssetType::hash_to_point(identifier).is_some() {
            Some(AssetType {
                identifier: *identifier,
                nonce: None,
            })
        } else {
            None
        }
    }
    /// Produces an asset generator without cofactor cleared
    pub fn asset_generator(&self) -> pallas::Point {
        AssetType::hash_to_point(self.get_identifier())
            .expect("AssetType::get_identifier() should never return None")
    }

    /// Produces a value commitment generator with cofactor cleared
    /// TODO:返り値をsubgroupにする
    pub fn value_commitment_generator(&self) -> () {
        CofactorGroup::clear_cofactor(&self.asset_generator());
    }

    /// Get the asset identifier as a vector of bools
    pub fn identifier_bits(&self) -> Vec<Option<bool>> {
        self.get_identifier()
            .iter()
            .flat_map(|&v| (0..8).map(move |i| Some((v >> i) & 1 == 1)))
            .collect()
    }

    /// Get the asset identifier as a vector of bools
    pub fn value_commitment(&self, randomness: pallas::Point) -> ValueCommitment {
        ValueCommitment(randomness)
    }

    /// Get the asset identifier as a vector of bools
    pub fn get_nonce(&self) -> Option<u8> {
        self.nonce
    }
}

impl PartialEq for AssetType {
    fn eq(&self, other: &Self) -> bool {
        self.get_identifier() == other.get_identifier()
    }
}

impl Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", hex::encode(self.get_identifier()))
    }
}

impl Hash for AssetType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_identifier().hash(state);
    }
}

impl PartialOrd for AssetType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_identifier().partial_cmp(other.get_identifier())
    }
}

impl Ord for AssetType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_identifier().cmp(other.get_identifier())
    }
}

impl std::str::FromStr for AssetType {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = hex::decode(s).map_err(|x| Self::Err::new(std::io::ErrorKind::InvalidData, x))?;
        Self::from_identifier(
            &vec.try_into()
                .map_err(|_| Self::Err::from(std::io::ErrorKind::InvalidData))?,
        )
        .ok_or_else(|| Self::Err::from(std::io::ErrorKind::InvalidData))
    }
}

#[cfg(any(test, feature = "test-dependencies"))]
pub mod testing {
    use proptest::prelude::*;

    prop_compose! {
        pub fn arb_asset_type()(name in proptest::collection::vec(prop::num::u8::ANY, 0..64)) -> super::AssetType {
            super::AssetType::new(&name).unwrap()
        }
    }
}
