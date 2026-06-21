//! Utility functions for computing bundle commitments

use blake2b_simd::{Hash as Blake2bHash, Params, State};

use crate::bundle::{Authorization, Authorized, Bundle, BundleFormat};

const ZCASH_ORCHARD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";
const ZCASH_ORCHARD_V6_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardH_v6";
const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActCHash";
const ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActMHash";
const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActNHash";
const ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaHash";
const ZCASH_ORCHARD_V6_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaH_v6";
const ZCASH_IRONWOOD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIronwd_H_v6";
const ZCASH_IRONWOOD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActCH_v6";
const ZCASH_IRONWOOD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActMH_v6";
const ZCASH_IRONWOOD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActNH_v6";
const ZCASH_IRONWOOD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthIrnwdH_v6";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnchorCommitment {
    Include,
    Omit,
}

#[derive(Clone, Copy, Debug)]
struct BundleCommitmentPersonalizations {
    bundle: &'static [u8; 16],
    actions_compact: &'static [u8; 16],
    actions_memos: &'static [u8; 16],
    actions_noncompact: &'static [u8; 16],
    auth: &'static [u8; 16],
}

const ORCHARD_V5_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_ORCHARD_HASH_PERSONALIZATION,
        actions_compact: ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION,
    };

const ORCHARD_V6_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_ORCHARD_V6_HASH_PERSONALIZATION,
        actions_compact: ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_ORCHARD_V6_SIGS_HASH_PERSONALIZATION,
    };

const IRONWOOD_V6_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_IRONWOOD_HASH_PERSONALIZATION,
        actions_compact: ZCASH_IRONWOOD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_IRONWOOD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_IRONWOOD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_IRONWOOD_SIGS_HASH_PERSONALIZATION,
    };

/// Parameters for computing a bundle commitment under a fixed transaction domain.
///
/// This type selects the domain-specific pieces of the commitment algorithm:
/// protocol personalization strings, flag-byte encoding format, and anchor
/// placement. Use the associated constants to select the domain required by the
/// transaction format.
#[derive(Clone, Copy, Debug)]
pub struct BundleCommitmentDomain {
    personalizations: BundleCommitmentPersonalizations,
    format: BundleFormat,
    effects_anchor: AnchorCommitment,
    auth_anchor: AnchorCommitment,
}

impl BundleCommitmentDomain {
    /// The Orchard v5 commitment domain for transaction formats before NU6.3.
    pub const ORCHARD_V5_PRE_NU6_3: Self = Self {
        personalizations: ORCHARD_V5_PERSONALIZATIONS,
        format: BundleFormat::PreNu6_3,
        effects_anchor: AnchorCommitment::Include,
        auth_anchor: AnchorCommitment::Omit,
    };

    /// The Orchard v5 commitment domain for NU6.3 flag-byte encoding.
    pub const ORCHARD_V5_NU6_3: Self = Self {
        personalizations: ORCHARD_V5_PERSONALIZATIONS,
        format: BundleFormat::Nu6_3,
        effects_anchor: AnchorCommitment::Include,
        auth_anchor: AnchorCommitment::Omit,
    };

    /// The version 6 Orchard transaction commitment domain.
    ///
    /// Version 6 Orchard bundles keep the original Orchard action
    /// personalization strings but use distinct bundle and authorization
    /// personalizations.
    pub const ORCHARD_V6: Self = Self {
        personalizations: ORCHARD_V6_PERSONALIZATIONS,
        format: BundleFormat::Nu6_3,
        effects_anchor: AnchorCommitment::Omit,
        auth_anchor: AnchorCommitment::Include,
    };

    /// The version 6 Ironwood transaction commitment domain.
    pub const IRONWOOD_V6: Self = Self {
        personalizations: IRONWOOD_V6_PERSONALIZATIONS,
        format: BundleFormat::Nu6_3,
        effects_anchor: AnchorCommitment::Omit,
        auth_anchor: AnchorCommitment::Include,
    };
}

fn hasher(personal: &[u8; 16]) -> State {
    Params::new().hash_length(32).personal(personal).to_state()
}

/// Write disjoint parts of each Orchard shielded action as 3 separate hashes
/// as defined in [ZIP-244: Transaction Identifier Non-Malleability][zip244]:
/// * \[(nullifier, cmx, ephemeral_key, enc_ciphertext\[..52\])*\] personalized
///   with ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION
/// * \[enc_ciphertext\[52..564\]*\] (memo ciphertexts) personalized
///   with ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION
/// * \[(cv, rk, enc_ciphertext\[564..\], out_ciphertext)*\] personalized
///   with ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION
///
/// Then, hash these together along with (flags, value_balance_orchard, anchor_orchard),
/// personalized with ZCASH_ORCHARD_ACTIONS_HASH_PERSONALIZATION
///
/// Returns `None` if the bundle flags cannot be encoded in the domain's bundle format.
///
/// [zip244]: https://zips.z.cash/zip-0244
pub(crate) fn hash_bundle_txid_data<A: Authorization, V: Copy + Into<i64>>(
    bundle: &Bundle<A, V>,
    domain: BundleCommitmentDomain,
) -> Option<Blake2bHash> {
    let mut h = hasher(domain.personalizations.bundle);
    let mut ch = hasher(domain.personalizations.actions_compact);
    let mut mh = hasher(domain.personalizations.actions_memos);
    let mut nh = hasher(domain.personalizations.actions_noncompact);

    for action in bundle.actions().iter() {
        ch.update(&action.nullifier().to_bytes());
        ch.update(&action.cmx().to_bytes());
        ch.update(&action.encrypted_note().epk_bytes);
        ch.update(&action.encrypted_note().enc_ciphertext[..52]);

        mh.update(&action.encrypted_note().enc_ciphertext[52..564]);

        nh.update(&action.cv_net().to_bytes());
        nh.update(&<[u8; 32]>::from(action.rk()));
        nh.update(&action.encrypted_note().enc_ciphertext[564..]);
        nh.update(&action.encrypted_note().out_ciphertext);
    }

    h.update(ch.finalize().as_bytes());
    h.update(mh.finalize().as_bytes());
    h.update(nh.finalize().as_bytes());
    h.update(&[bundle.flags().to_byte(domain.format)?]);
    h.update(&(*bundle.value_balance()).into().to_le_bytes());
    if domain.effects_anchor == AnchorCommitment::Include {
        h.update(&bundle.anchor().to_bytes());
    }
    Some(h.finalize())
}

/// Construct the commitment for the absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_txid_empty(domain: BundleCommitmentDomain) -> Blake2bHash {
    hasher(domain.personalizations.bundle).finalize()
}

/// Construct the commitment to the authorizing data of an
/// authorized bundle as defined in [ZIP-244: Transaction
/// Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub(crate) fn hash_bundle_auth_data<V>(
    bundle: &Bundle<Authorized, V>,
    domain: BundleCommitmentDomain,
) -> Blake2bHash {
    let mut h = hasher(domain.personalizations.auth);
    h.update(bundle.authorization().proof().as_ref());
    for action in bundle.actions().iter() {
        h.update(&<[u8; 64]>::from(action.authorization()));
    }
    h.update(&<[u8; 64]>::from(
        bundle.authorization().binding_signature(),
    ));
    if domain.auth_anchor == AnchorCommitment::Include {
        h.update(&bundle.anchor().to_bytes());
    }
    h.finalize()
}

/// Construct the commitment for an absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
///
/// [zip244]: https://zips.z.cash/zip-0244
pub fn hash_bundle_auth_empty(domain: BundleCommitmentDomain) -> Blake2bHash {
    hasher(domain.personalizations.auth).finalize()
}
