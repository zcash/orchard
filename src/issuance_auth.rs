//! Issuance authorization logic for Zcash Shielded Assets (ZSAs).
//!
//! This module provides types and methods for working with issuance authorizing keys, validating
//! keys, and authorization signatures, as defined in [ZIP 227].
//!
//! # Example
//! ```
//! use rand::rngs::OsRng;
//! use orchard::issuance_auth::{IssueAuthKey, IssueValidatingKey, ZSASchnorr};
//!
//! let mut rng = OsRng;
//! let isk = IssueAuthKey::<ZSASchnorr>::random(&mut rng);
//! let ik = IssueValidatingKey::from(&isk);
//! let msg = [1u8; 32];
//! let sig = isk.try_sign(&msg).unwrap();
//! ik.verify(&msg, &sig).unwrap();
//! ```
//!
//! [ZIP 227]: https://zips.z.cash/zip-0227

use alloc::vec::Vec;
use core::{
    fmt::{Debug, Formatter},
    mem::size_of_val,
};

use k256::{
    schnorr,
    schnorr::{signature::hazmat::PrehashVerifier, VerifyingKey},
    NonZeroScalar,
};
use rand_core::CryptoRngCore;

use crate::{
    issuance::Error,
    zip32::{self, ExtendedSpendingKey},
};

// Preserve '::' which specifies the EXTERNAL 'zip32' crate
#[rustfmt::skip]
pub use ::zip32::{AccountId, ChildIndex, DiversifierIndex, Scope, hardened_only};

const ZIP32_PURPOSE_FOR_ISSUANCE: u32 = 227;

/// Trait that defines the common interface for issuance authorization signature schemes.
pub trait IssueAuthSigScheme {
    /// The byte corresponding to this signature scheme, used to encode the issuance validating key
    /// and issuance authorization signature.
    const ALGORITHM_BYTE: u8;

    /// The type of the issuance authorizing key.
    type IskType;
    /// The type of the issuance validating key.
    type IkType: Clone + PartialEq;
    /// The type of the issuance authorization signature.
    type IssueAuthSigType: Clone + Eq + PartialEq + Debug;

    /// Signs a sighash using the issuance authorizing key.
    fn try_sign(isk: &Self::IskType, sighash: &[u8]) -> Result<Self::IssueAuthSigType, Error>;

    /// Verifies that the provided signature for a given sighash is authentic.
    fn verify(
        ik: &Self::IkType,
        sighash: &[u8],
        signature: &Self::IssueAuthSigType,
    ) -> Result<(), Error>;
}

/// An issuance authorizing key.
///
/// This is denoted by `isk` as defined in [ZIP 227][issuancekeycomponents].
///
/// [issuancekeycomponents]: https://zips.z.cash/zip-0227#issuance-key-derivation
#[derive(Clone)]
pub struct IssueAuthKey<S: IssueAuthSigScheme>(S::IskType);

impl<S: IssueAuthSigScheme> IssueAuthKey<S> {
    /// Signs a sighash using the issuance authorizing key.
    pub fn try_sign(&self, sighash: &[u8]) -> Result<IssueAuthSig<S>, Error> {
        S::try_sign(&self.0, sighash).map(IssueAuthSig)
    }
}

/// An issuance validating key which is used to validate issuance authorization signatures.
///
/// This is denoted by `ik` and defined in [ZIP 227: Issuance of Zcash Shielded Assets § Issuance Key Generation][IssuanceZSA].
///
/// [IssuanceZSA]: https://zips.z.cash/zip-0227#issuance-key-derivation
#[derive(Clone, PartialEq, Eq)]
pub struct IssueValidatingKey<S: IssueAuthSigScheme>(S::IkType);

impl<S: IssueAuthSigScheme> IssueValidatingKey<S> {
    /// Verifies that the provided signature for a given sighash is authentic.
    pub fn verify(&self, sighash: &[u8], sig: &IssueAuthSig<S>) -> Result<(), Error> {
        S::verify(&self.0, sighash, &sig.0)
    }
}

/// An issuance authorization signature `issueAuthSig`,
///
/// as defined in [ZIP 227][issueauthsig].
///
/// [issueauthsig]: https://zips.z.cash/zip-0227#issuance-authorization-signature-scheme
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssueAuthSig<S: IssueAuthSigScheme>(S::IssueAuthSigType);

/// The Orchard-ZSA issuance authorization signature scheme, based on BIP 340 Schnorr.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZSASchnorr;

impl IssueAuthSigScheme for ZSASchnorr {
    const ALGORITHM_BYTE: u8 = 0x00;

    type IskType = NonZeroScalar;
    type IkType = VerifyingKey;
    type IssueAuthSigType = schnorr::Signature;

    fn try_sign(isk: &Self::IskType, sighash: &[u8]) -> Result<Self::IssueAuthSigType, Error> {
        schnorr::SigningKey::sign_raw(&schnorr::SigningKey::from(*isk), sighash, &[0u8; 32])
            .map_err(|_| Error::InvalidIssueBundleSig)
    }

    fn verify(
        ik: &Self::IkType,
        sighash: &[u8],
        sig: &Self::IssueAuthSigType,
    ) -> Result<(), Error> {
        ik.verify_prehash(sighash, sig)
            .map_err(|_| Error::InvalidIssueBundleSig)
    }
}

impl IssueAuthKey<ZSASchnorr> {
    /// Generates a random issuance authorizing key.
    ///
    /// This is only used when generating a random AssetBase.
    /// Real issuance keys should be derived according to [ZIP 32].
    ///
    /// [ZIP 32]: https://zips.z.cash/zip-0032
    pub fn random(rng: &mut impl CryptoRngCore) -> Self {
        Self(NonZeroScalar::random(rng))
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    /// Deserialize the issuance authorization signature from its canonical byte representation.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        NonZeroScalar::try_from(bytes).ok().map(Self)
    }

    /// Derives the Orchard-ZSA issuance key for the given seed, coin type, and account.
    pub fn from_zip32_seed(
        seed: &[u8],
        coin_type: u32,
        account: u32,
    ) -> Result<Self, zip32::Error> {
        if account != 0 {
            return Err(zip32::Error::NonZeroAccount);
        }

        // Call zip32 logic
        let path = &[
            ChildIndex::hardened(ZIP32_PURPOSE_FOR_ISSUANCE),
            ChildIndex::hardened(coin_type),
            ChildIndex::hardened(account),
        ];

        // we are reusing zip32 logic for deriving the key, zip32 should be updated as discussed
        let &isk_bytes = ExtendedSpendingKey::<zip32::Issuance>::from_path(seed, path)?
            .sk()
            .to_bytes();

        Self::from_bytes(&isk_bytes).ok_or(zip32::Error::InvalidSpendingKey)
    }
}

impl From<&IssueAuthKey<ZSASchnorr>> for IssueValidatingKey<ZSASchnorr> {
    fn from(isk: &IssueAuthKey<ZSASchnorr>) -> Self {
        Self(*schnorr::SigningKey::from(isk.0).verifying_key())
    }
}

impl IssueValidatingKey<ZSASchnorr> {
    /// Encodes the issuance validating key into a byte vector, in the manner defined in [ZIP 227][issuancekeycomponents].
    ///
    /// [issuancekeycomponents]: https://zips.z.cash/zip-0227#derivation-of-issuance-validating-key
    pub fn encode(&self) -> Vec<u8> {
        let ik_bytes = self.0.to_bytes().to_vec();
        let mut encoded =
            Vec::with_capacity(size_of_val(&ZSASchnorr::ALGORITHM_BYTE) + ik_bytes.len());
        encoded.push(ZSASchnorr::ALGORITHM_BYTE);
        encoded.extend(ik_bytes);
        encoded
    }

    /// Decodes an issuance validating key from the byte representation defined in [ZIP 227][issuancekeycomponents].
    ///
    /// [issuancekeycomponents]: https://zips.z.cash/zip-0227#derivation-of-issuance-validating-key
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if let Some((&algorithm_byte, key_bytes)) = bytes.split_first() {
            if algorithm_byte == ZSASchnorr::ALGORITHM_BYTE {
                return VerifyingKey::from_bytes(key_bytes)
                    .map(Self)
                    .map_err(|_| Error::InvalidIssueValidatingKey);
            }
        }
        Err(Error::InvalidIssueValidatingKey)
    }
}

impl IssueAuthSig<ZSASchnorr> {
    /// Encodes the issuance authorization signature into a byte vector, in the manner
    /// defined in [ZIP 227][issueauthsig].
    ///
    /// [issueauthsig]: https://zips.z.cash/zip-0227#issuance-authorization-signing-and-validation
    pub fn encode(&self) -> Vec<u8> {
        let sig_bytes = self.0.to_bytes().to_vec();
        let mut encoded =
            Vec::with_capacity(size_of_val(&ZSASchnorr::ALGORITHM_BYTE) + sig_bytes.len());
        encoded.push(ZSASchnorr::ALGORITHM_BYTE);
        encoded.extend(sig_bytes);
        encoded
    }

    /// Decodes an issuance authorization signature from the byte representation defined
    /// in [ZIP 227][issueauthsig].
    ///
    /// [issueauthsig]: https://zips.z.cash/zip-0227#issuance-authorization-signing-and-validation
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if let Some((&algorithm_byte, key_bytes)) = bytes.split_first() {
            if algorithm_byte == ZSASchnorr::ALGORITHM_BYTE {
                return schnorr::Signature::try_from(key_bytes)
                    .map(Self)
                    .map_err(|_| Error::InvalidIssueBundleSig);
            }
        }
        Err(Error::InvalidIssueBundleSig)
    }
}

impl Debug for IssueValidatingKey<ZSASchnorr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let ik_bytes = self.encode();
        let last4 = &ik_bytes[(ik_bytes.len() - 4)..];

        write!(
            f,
            "IssueValidatingKey {{ last4: 0x{:02x}{:02x}{:02x}{:02x} }}",
            last4[0], last4[1], last4[2], last4[3]
        )
    }
}

impl Debug for IssueAuthKey<ZSASchnorr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Do not print bytes from the issuance authorizing key.
        let ik = IssueValidatingKey::from(self);
        write!(f, "IssueAuthKey({:?})", ik)
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use super::{IssueAuthKey, IssueValidatingKey, ZSASchnorr};

    use proptest::prelude::*;

    prop_compose! {
        /// Generate a uniformly distributed Orchard issuance authorizing key.
        pub fn arb_issuance_authorizing_key()(
            key in prop::array::uniform32(prop::num::u8::ANY)
                .prop_map(|key| IssueAuthKey::from_bytes(&key))
                .prop_filter(
                    "Values must be valid Orchard-ZSA issuance authorizing keys.",
                    |opt| opt.is_some()
                )
        ) -> IssueAuthKey<ZSASchnorr> {
            key.unwrap()
        }
    }

    prop_compose! {
        /// Generate a uniformly distributed issuance validating key.
        pub fn arb_issuance_validating_key()(isk in arb_issuance_authorizing_key()) -> IssueValidatingKey<ZSASchnorr> {
            IssueValidatingKey::from(&isk)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn issuance_authorizing_key_from_bytes_fail_on_zero() {
        // isk must not be the zero scalar for the ZSA Schnorr scheme.
        let zero_bytes = [0u8; 32];
        let isk = IssueAuthKey::<ZSASchnorr>::from_bytes(&zero_bytes);
        assert!(isk.is_none());
    }

    #[test]
    fn issuance_authorizing_key_from_bytes_to_bytes_roundtrip() {
        let isk: IssueAuthKey<ZSASchnorr> = IssueAuthKey::random(&mut OsRng);
        let isk_bytes = isk.to_bytes();
        let isk_roundtrip = IssueAuthKey::<ZSASchnorr>::from_bytes(&isk_bytes).unwrap();
        assert_eq!(isk_bytes, isk_roundtrip.to_bytes());
    }

    #[test]
    fn issuance_validating_key_encode_decode_roundtrip() {
        let isk: IssueAuthKey<ZSASchnorr> = IssueAuthKey::random(&mut OsRng);
        let ik = IssueValidatingKey::from(&isk);
        let ik_bytes = ik.encode();
        let ik_roundtrip = IssueValidatingKey::decode(&ik_bytes).unwrap();
        assert_eq!(ik_bytes, ik_roundtrip.encode());
    }

    #[test]
    fn issuance_authorization_signature_encode_decode_roundtrip() {
        let isk: IssueAuthKey<ZSASchnorr> = IssueAuthKey::random(&mut OsRng);
        let sig = isk.try_sign(&[1u8; 32]).unwrap();
        let sig_bytes = sig.encode();
        let sig_roundtrip = IssueAuthSig::<ZSASchnorr>::decode(&sig_bytes).unwrap();
        assert_eq!(sig_bytes, sig_roundtrip.encode());
    }

    #[test]
    fn verify_fails_on_wrong_message() {
        let isk: IssueAuthKey<ZSASchnorr> = IssueAuthKey::random(&mut OsRng);
        let ik = IssueValidatingKey::from(&isk);
        let sighash = [1u8; 32];
        let incorrect_sighash = [2u8; 32];
        let sig = isk.try_sign(&sighash).unwrap();
        assert_eq!(
            ik.verify(&incorrect_sighash, &sig),
            Err(Error::InvalidIssueBundleSig)
        );
    }

    #[test]
    fn verify_fails_on_wrong_key() {
        let isk: IssueAuthKey<ZSASchnorr> = IssueAuthKey::random(&mut OsRng);
        let sighash = [1u8; 32];
        let sig = isk.try_sign(&sighash).unwrap();
        let incorrect_isk: IssueAuthKey<ZSASchnorr> = IssueAuthKey::random(&mut OsRng);
        let incorrect_ik = IssueValidatingKey::from(&incorrect_isk);
        assert_eq!(
            incorrect_ik.verify(&sighash, &sig),
            Err(Error::InvalidIssueBundleSig)
        );
    }

    #[test]
    fn issuance_auth_sig_test_vectors() {
        for tv in crate::test_vectors::issuance_auth_sig::TEST_VECTORS {
            let isk = IssueAuthKey::<ZSASchnorr>::from_bytes(&tv.isk).unwrap();

            let ik = IssueValidatingKey::from(&isk);
            assert_eq!(ik.encode(), &tv.ik_encoding);

            let sig = isk.try_sign(&tv.msg).unwrap();
            let sig_bytes = sig.encode();
            assert_eq!(sig_bytes, &tv.issue_auth_sig);

            assert!(ik.verify(&tv.msg, &sig).is_ok());
        }
    }
}
