//! Utility functions for computing bundle commitments

use alloc::vec::Vec;
use blake2b_simd::{Hash as Blake2bHash, Params, State};

use crate::{
    bundle::{Authorization, Authorized, Bundle, BundleVersion, CommitmentError, TxVersion},
    note_encryption::{COMPACT_NOTE_SIZE_VANILLA, COMPACT_NOTE_SIZE_ZSA, MEMO_SIZE},
    sighash_kind::OrchardSighashKind,
    ProtocolVersion, ValuePool,
};

#[cfg(feature = "zsa-issuance")]
mod issuance;

#[cfg(feature = "zsa-issuance")]
pub(crate) use issuance::{hash_issue_bundle_auth_data, hash_issue_bundle_txid_data};

#[cfg(feature = "zsa-issuance")]
pub use issuance::{hash_issue_bundle_auth_empty, hash_issue_bundle_txid_empty};

const ZCASH_ORCHARD_V5_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardHash";
const ZCASH_ORCHARD_V6_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardH_v6";
const ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActCHash";
const ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActMHash";
const ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcActNHash";
const ZCASH_ORCHARD_V5_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaHash";
const ZCASH_ORCHARD_V6_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrchaH_v6";
const ZCASH_IRONWOOD_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIronwd_H_v6";
const ZCASH_IRONWOOD_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActCH_v6";
const ZCASH_IRONWOOD_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActMH_v6";
const ZCASH_IRONWOOD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdIrnActNH_v6";
const ZCASH_IRONWOOD_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthIrnwdH_v6";

// These strings are specific to this fork. They are not the ZIP-246 strings,
// because ZIP-246 is withdrawn. ZIP-229 does not specify the ZSA values at this time.
//
// TODO Re-derive these from the ZSA digest spec once published, and add its test vectors.
// Changing any string here changes consensus-visible digests.
//
// TODO Settle the naming convention at the same time: the burn and auth strings below still say
// `Orc`, and the suffixes mix `Hash` with `_v6`.
const ZCASH_ZSA_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrchardZsaH";
const ZCASH_ZSA_ACTION_GROUPS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdZsaActGHash";
const ZCASH_ZSA_ACTIONS_COMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdZsaActCHash";
const ZCASH_ZSA_ACTIONS_MEMOS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdZsaActMH_v6";
const ZCASH_ZSA_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdZsaActNHash";
const ZCASH_ZSA_BURN_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxIdOrcBurnHash";
const ZCASH_ZSA_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthZsaHashv6";
const ZCASH_ZSA_ACTION_GROUPS_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrcAGHash";
const ZCASH_ZSA_SPEND_AUTH_SIGS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxAuthOrSASHash";

#[derive(Clone, Copy, Debug)]
struct BundleCommitmentPersonalizations {
    bundle: &'static [u8; 16],
    actions_compact: &'static [u8; 16],
    actions_memos: &'static [u8; 16],
    actions_noncompact: &'static [u8; 16],
    auth: &'static [u8; 16],
    zsa: Option<ZSAPersonalizations>,
}

#[derive(Clone, Copy, Debug)]
struct ZSAPersonalizations {
    action_groups: &'static [u8; 16],
    ironwood_burn: &'static [u8; 16],
    action_groups_auth: &'static [u8; 16],
    zsa_spend_auth: &'static [u8; 16],
}

const ORCHARD_V5_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_ORCHARD_V5_HASH_PERSONALIZATION,
        actions_compact: ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_ORCHARD_V5_SIGS_HASH_PERSONALIZATION,
        zsa: None,
    };

// Orchard v6 deliberately reuses the v5 action-level personalizations
// (compact/memos/noncompact); only the top-level `bundle` and `auth` strings gain `_v6`.
// Ironwood instead uses fresh `_v6` strings throughout. Either way the top-level digest is
// domain-separated by its `bundle`/`auth` personalization, so reusing the action-level ones
// cannot collide across formats.
const ORCHARD_V6_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_ORCHARD_V6_HASH_PERSONALIZATION,
        actions_compact: ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_ORCHARD_V6_SIGS_HASH_PERSONALIZATION,
        zsa: None,
    };

const IRONWOOD_V6_PERSONALIZATIONS: BundleCommitmentPersonalizations =
    BundleCommitmentPersonalizations {
        bundle: ZCASH_IRONWOOD_HASH_PERSONALIZATION,
        actions_compact: ZCASH_IRONWOOD_ACTIONS_COMPACT_HASH_PERSONALIZATION,
        actions_memos: ZCASH_IRONWOOD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
        actions_noncompact: ZCASH_IRONWOOD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
        auth: ZCASH_IRONWOOD_SIGS_HASH_PERSONALIZATION,
        zsa: None,
    };

const ZSA_PERSONALIZATIONS: BundleCommitmentPersonalizations = BundleCommitmentPersonalizations {
    bundle: ZCASH_ZSA_HASH_PERSONALIZATION,
    actions_compact: ZCASH_ZSA_ACTIONS_COMPACT_HASH_PERSONALIZATION,
    actions_memos: ZCASH_ZSA_ACTIONS_MEMOS_HASH_PERSONALIZATION,
    actions_noncompact: ZCASH_ZSA_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION,
    auth: ZCASH_ZSA_SIGS_HASH_PERSONALIZATION,
    zsa: Some(ZSAPersonalizations {
        action_groups: ZCASH_ZSA_ACTION_GROUPS_HASH_PERSONALIZATION,
        ironwood_burn: ZCASH_ZSA_BURN_HASH_PERSONALIZATION,
        action_groups_auth: ZCASH_ZSA_ACTION_GROUPS_SIGS_HASH_PERSONALIZATION,
        zsa_spend_auth: ZCASH_ZSA_SPEND_AUTH_SIGS_HASH_PERSONALIZATION,
    }),
};

/// The hash format used to compute a bundle's transaction-ID and authorizing digests,
/// selected from the bundle's pool and the version of the transaction it is encoded in.
/// Orchard bundles use the v5 or v6 format according to the transaction; Ironwood bundles
/// exist only in v6 transactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum BundleCommitmentFormat {
    OrchardV5,
    OrchardV6,
    IronwoodV6,
    ZSA,
}

impl ValuePool {
    pub(crate) fn commitment_format(
        self,
        tx_version: TxVersion,
    ) -> Result<BundleCommitmentFormat, CommitmentError> {
        match (self, tx_version) {
            (ValuePool::Orchard, TxVersion::V5) => Ok(BundleCommitmentFormat::OrchardV5),
            (ValuePool::Orchard, TxVersion::V6) => Ok(BundleCommitmentFormat::OrchardV6),
            (ValuePool::Orchard, TxVersion::ZSA) => Err(CommitmentError::InvalidTransactionVersion),
            (ValuePool::Ironwood, TxVersion::V5) => Err(CommitmentError::InvalidTransactionVersion),
            (ValuePool::Ironwood, TxVersion::V6) => Ok(BundleCommitmentFormat::IronwoodV6),
            (ValuePool::Ironwood, TxVersion::ZSA) => Ok(BundleCommitmentFormat::ZSA),
        }
    }
}

impl BundleVersion {
    fn check_bundle_version_tx_version_compatibility(
        self,
        tx_version: TxVersion,
    ) -> Result<(), CommitmentError> {
        match (self.value_pool, self.protocol_version, tx_version) {
            (ValuePool::Orchard, ProtocolVersion::ZSA, _) => {
                Err(CommitmentError::InvalidTransactionVersion)
            }
            (ValuePool::Orchard, _, TxVersion::ZSA) => {
                Err(CommitmentError::InvalidTransactionVersion)
            }
            (ValuePool::Orchard, _, _) => Ok(()),
            (ValuePool::Ironwood, ProtocolVersion::V3, TxVersion::V6) => Ok(()),
            (ValuePool::Ironwood, ProtocolVersion::ZSA, TxVersion::ZSA) => Ok(()),
            (ValuePool::Ironwood, _, _) => Err(CommitmentError::InvalidTransactionVersion),
        }
    }
}

impl BundleCommitmentFormat {
    fn personalizations(self) -> BundleCommitmentPersonalizations {
        match self {
            BundleCommitmentFormat::OrchardV5 => ORCHARD_V5_PERSONALIZATIONS,
            BundleCommitmentFormat::OrchardV6 => ORCHARD_V6_PERSONALIZATIONS,
            BundleCommitmentFormat::IronwoodV6 => IRONWOOD_V6_PERSONALIZATIONS,
            BundleCommitmentFormat::ZSA => ZSA_PERSONALIZATIONS,
        }
    }

    pub(crate) fn includes_anchor_in_txid_digest(self) -> bool {
        matches!(self, BundleCommitmentFormat::OrchardV5)
    }

    /// Returns `true` if the anchor should be included in the authorizing digest
    /// for this bundle commitment format.
    ///
    /// Only `hash_bundle_auth_data_vanilla` branches on this. `hash_bundle_auth_data_zsa`
    /// appends the anchor unconditionally and just asserts this flag agrees.
    fn includes_anchor_in_authorizing_digest(self) -> bool {
        matches!(
            self,
            BundleCommitmentFormat::OrchardV6
                | BundleCommitmentFormat::IronwoodV6
                | BundleCommitmentFormat::ZSA
        )
    }
}

fn hasher(personal: &[u8; 16]) -> State {
    Params::new().hash_length(32).personal(personal).to_state()
}

/// Write disjoint parts of each bundle action as 3 separate hashes
/// as defined in [ZIP-244: Transaction Identifier Non-Malleability][zip244]:
/// * \[(nullifier, cmx, ephemeral_key, enc_ciphertext\[..52\])*\] personalized
///   with the format's compact-action personalization string
/// * \[enc_ciphertext\[52..564\]*\] (memo ciphertexts) personalized
///   with the format's action-memos personalization string
/// * \[(cv, rk, enc_ciphertext\[564..\], out_ciphertext)*\] personalized
///   with the format's non-compact-action personalization string
///
/// Then, hash these together along with (flags, value_balance_orchard, and — for the v5
/// transaction format only — anchor_orchard), personalized with the format's bundle
/// personalization string. In the v6 format the anchor is included by
/// `hash_bundle_auth_data` instead.
///
/// Returns [`CommitmentError::InvalidTransactionVersion`] if `tx_version` is not valid for the
/// bundle's [`BundleVersion`].
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [`BundleVersion`]: crate::bundle::BundleVersion
fn hash_bundle_txid_data_vanilla<A: Authorization, V: Copy + Into<i64>>(
    bundle: &Bundle<A, V>,
    tx_version: TxVersion,
) -> Result<Blake2bHash, CommitmentError> {
    let format = bundle
        .bundle_version()
        .value_pool
        .commitment_format(tx_version)?;
    let personalizations = format.personalizations();
    let mut h = hasher(personalizations.bundle);
    let mut ch = hasher(personalizations.actions_compact);
    let mut mh = hasher(personalizations.actions_memos);
    let mut nh = hasher(personalizations.actions_noncompact);

    for action in bundle.actions().iter() {
        ch.update(&action.nullifier().to_bytes());
        ch.update(&action.cmx().to_bytes());
        ch.update(&action.encrypted_note().epk_bytes);
        ch.update(&action.encrypted_note().enc_ciphertext.as_ref()[..COMPACT_NOTE_SIZE_VANILLA]);

        mh.update(
            &action.encrypted_note().enc_ciphertext.as_ref()
                [COMPACT_NOTE_SIZE_VANILLA..COMPACT_NOTE_SIZE_VANILLA + MEMO_SIZE],
        );

        nh.update(&action.cv_net().to_bytes());
        nh.update(&<[u8; 32]>::from(action.rk()));
        nh.update(
            &action.encrypted_note().enc_ciphertext.as_ref()
                [COMPACT_NOTE_SIZE_VANILLA + MEMO_SIZE..],
        );
        nh.update(&action.encrypted_note().out_ciphertext);
    }

    h.update(ch.finalize().as_bytes());
    h.update(mh.finalize().as_bytes());
    h.update(nh.finalize().as_bytes());
    h.update(&[bundle.flag_byte()]);
    h.update(&(*bundle.value_balance()).into().to_le_bytes());
    if format.includes_anchor_in_txid_digest() {
        h.update(&bundle.anchor().to_bytes());
    }
    Ok(h.finalize())
}

/// Evaluate `orchard_digest` for the ZSA bundle as defined in
/// [ZIP-246: Digests for the Withdrawn Version 6 Transaction Format][zip246]
///
/// [zip246]: https://zips.z.cash/zip-0246
fn hash_bundle_txid_data_zsa<A: Authorization, V: Copy + Into<i64>>(
    bundle: &Bundle<A, V>,
    tx_version: TxVersion,
) -> Result<Blake2bHash, CommitmentError> {
    let format = bundle
        .bundle_version()
        .value_pool
        .commitment_format(tx_version)?;

    let personalizations = format.personalizations();
    let zsa_personalizations = personalizations.zsa.unwrap();

    let mut h = hasher(personalizations.bundle);
    let mut agh = hasher(zsa_personalizations.action_groups);

    let mut ch = hasher(personalizations.actions_compact);
    let mut mh = hasher(personalizations.actions_memos);
    let mut nh = hasher(personalizations.actions_noncompact);

    for action in bundle.actions().iter() {
        ch.update(&action.nullifier().to_bytes());
        ch.update(&action.cmx().to_bytes());
        ch.update(&action.encrypted_note().epk_bytes);
        ch.update(&action.encrypted_note().enc_ciphertext.as_ref()[..COMPACT_NOTE_SIZE_ZSA]);

        mh.update(
            &action.encrypted_note().enc_ciphertext.as_ref()
                [COMPACT_NOTE_SIZE_ZSA..COMPACT_NOTE_SIZE_ZSA + MEMO_SIZE],
        );

        nh.update(&action.cv_net().to_bytes());
        nh.update(&<[u8; 32]>::from(action.rk()));
        nh.update(
            &action.encrypted_note().enc_ciphertext.as_ref()[COMPACT_NOTE_SIZE_ZSA + MEMO_SIZE..],
        );
        nh.update(&action.encrypted_note().out_ciphertext);
    }

    agh.update(ch.finalize().as_bytes());
    agh.update(mh.finalize().as_bytes());
    agh.update(nh.finalize().as_bytes());

    agh.update(&[bundle.flag_byte()]);
    // For the OrchardZSA protocol, `expiry_height` is set to 0, indicating no expiry.
    agh.update(&0u32.to_le_bytes());

    let mut burn_hasher = hasher(zsa_personalizations.ironwood_burn);
    for burn_item in bundle.burn() {
        burn_hasher.update(&burn_item.0.to_bytes());
        burn_hasher.update(&burn_item.1.to_bytes());
    }
    agh.update(burn_hasher.finalize().as_bytes());
    h.update(agh.finalize().as_bytes());

    h.update(&(*bundle.value_balance()).into().to_le_bytes());
    Ok(h.finalize())
}

/// Evaluate `orchard_digest` for the bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244] for Orchard, as defined in
/// [ZIP-229: Version 6 Transaction Format][zip229] for Ironwood, and as defined in
/// [ZIP-246: Digests for the Withdrawn Version 6 Transaction Format][zip246] for ZSA.
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip229]: https://zips.z.cash/zip-0229
/// [zip246]: https://zips.z.cash/zip-0246
pub(crate) fn hash_bundle_txid_data<A: Authorization, V: Copy + Into<i64>>(
    bundle: &Bundle<A, V>,
    tx_version: TxVersion,
) -> Result<Blake2bHash, CommitmentError> {
    bundle
        .bundle_version()
        .check_bundle_version_tx_version_compatibility(tx_version)?;
    match tx_version {
        TxVersion::V5 | TxVersion::V6 => hash_bundle_txid_data_vanilla(bundle, tx_version),
        TxVersion::ZSA => hash_bundle_txid_data_zsa(bundle, tx_version),
    }
}

/// Construct the commitment for the absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244] for Orchard V5, as defined in
/// [ZIP-229: Version 6 Transaction Format][zip229] for Orchard V6 and Ironwood V6, and as defined in
/// [ZIP-246: Digests for the Withdrawn Version 6 Transaction Format][zip246] for ZSA, according to the
/// (value_pool, tx_version) pair.
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip229]: https://zips.z.cash/zip-0229
/// [zip246]: https://zips.z.cash/zip-0246
pub fn hash_bundle_txid_empty(
    value_pool: ValuePool,
    tx_version: TxVersion,
) -> Result<Blake2bHash, CommitmentError> {
    Ok(hasher(
        value_pool
            .commitment_format(tx_version)?
            .personalizations()
            .bundle,
    )
    .finalize())
}

/// Construct the commitment to the authorizing data of an
/// authorized bundle as defined in [ZIP-244: Transaction
/// Identifier Non-Malleability][zip244]
///
/// # Panics
///
/// Panics if any signature in the bundle uses a sighash kind different from
/// `OrchardSighashKind::AllEffecting`. In Orchard/Ironwood v5/v6 transactions, this is the
/// only defined sighash kind.
///
/// [zip244]: https://zips.z.cash/zip-0244
fn hash_bundle_auth_data_vanilla<V>(
    bundle: &Bundle<Authorized, V>,
    tx_version: TxVersion,
    _sighash_info_for_kind: impl Fn(&OrchardSighashKind) -> Vec<u8>,
) -> Result<Blake2bHash, CommitmentError> {
    let format = bundle
        .bundle_version()
        .value_pool
        .commitment_format(tx_version)?;
    let mut h = hasher(format.personalizations().auth);
    h.update(bundle.authorization().proof().as_ref());
    for action in bundle.actions().iter() {
        assert_eq!(
            *action.authorization().sighash_kind(),
            OrchardSighashKind::AllEffecting
        );
        h.update(&<[u8; 64]>::from(action.authorization().sig()));
    }
    assert_eq!(
        *bundle.authorization().binding_signature().sighash_kind(),
        OrchardSighashKind::AllEffecting
    );
    h.update(&<[u8; 64]>::from(
        bundle.authorization().binding_signature().sig(),
    ));
    if format.includes_anchor_in_authorizing_digest() {
        h.update(&bundle.anchor().to_bytes());
    }
    Ok(h.finalize())
}

/// Evaluate `orchard_auth_digest` for the ZSA bundle as defined in
/// [ZIP-246: Digests for the Withdrawn Version 6 Transaction Format][zip246]
///
/// The `sighash_info_for_kind` closure returns the `SighashInfo` encoding
/// for a given [`OrchardSighashKind`].
///
/// # Panics
///
/// Panics if any signature in the bundle uses a sighash kind different from
/// `OrchardSighashKind::AllEffecting`, which is currently the only defined kind.
///
/// [zip246]: https://zips.z.cash/zip-0246
fn hash_bundle_auth_data_zsa<V>(
    bundle: &Bundle<Authorized, V>,
    tx_version: TxVersion,
    sighash_info_for_kind: impl Fn(&OrchardSighashKind) -> Vec<u8>,
) -> Result<Blake2bHash, CommitmentError> {
    let format = bundle
        .bundle_version()
        .value_pool
        .commitment_format(tx_version)?;

    let personalizations = format.personalizations();
    let zsa_personalizations = personalizations.zsa.unwrap();

    let mut h = hasher(personalizations.auth);
    let mut agh = hasher(zsa_personalizations.action_groups_auth);
    agh.update(bundle.authorization().proof().as_ref());
    let mut sash = hasher(zsa_personalizations.zsa_spend_auth);
    for action in bundle.actions().iter() {
        assert_eq!(
            *action.authorization().sighash_kind(),
            OrchardSighashKind::AllEffecting
        );
        let sighash_info = sighash_info_for_kind(action.authorization().sighash_kind());
        sash.update(&get_compact_size(sighash_info.len()));
        sash.update(sighash_info.as_slice());
        sash.update(&<[u8; 64]>::from(action.authorization().sig()));
    }
    agh.update(sash.finalize().as_bytes());
    h.update(agh.finalize().as_bytes());

    assert_eq!(
        *bundle.authorization().binding_signature().sighash_kind(),
        OrchardSighashKind::AllEffecting
    );
    let sighash_info =
        sighash_info_for_kind(bundle.authorization().binding_signature().sighash_kind());
    h.update(&get_compact_size(sighash_info.len()));
    h.update(sighash_info.as_slice());
    h.update(&<[u8; 64]>::from(
        bundle.authorization().binding_signature().sig(),
    ));

    assert!(format.includes_anchor_in_authorizing_digest());
    h.update(&bundle.anchor().to_bytes());

    Ok(h.finalize())
}

/// Evaluate `orchard_auth_digest` for the bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244] for Orchard, as defined in
/// [ZIP-229: Version 6 Transaction Format][zip229] for Ironwood, and as defined in
/// [ZIP-246: Digests for the Withdrawn Version 6 Transaction Format][zip246] for ZSA.
///
/// The `sighash_info_for_kind` closure returns the `SighashInfo` encoding
/// for a given [`OrchardSighashKind`].
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip229]: https://zips.z.cash/zip-0229
/// [zip246]: https://zips.z.cash/zip-0246
pub(crate) fn hash_bundle_auth_data<V>(
    bundle: &Bundle<Authorized, V>,
    tx_version: TxVersion,
    sighash_info_for_kind: impl Fn(&OrchardSighashKind) -> Vec<u8>,
) -> Result<Blake2bHash, CommitmentError> {
    bundle
        .bundle_version()
        .check_bundle_version_tx_version_compatibility(tx_version)?;
    match tx_version {
        TxVersion::V5 | TxVersion::V6 => {
            hash_bundle_auth_data_vanilla(bundle, tx_version, sighash_info_for_kind)
        }
        TxVersion::ZSA => hash_bundle_auth_data_zsa(bundle, tx_version, sighash_info_for_kind),
    }
}

/// Construct the commitment for the absent bundle as defined in
/// [ZIP-244: Transaction Identifier Non-Malleability][zip244] for Orchard V5, as defined in
/// [ZIP-229: Version 6 Transaction Format][zip229] for Orchard V6 and Ironwood V6, and as defined in
/// [ZIP-246: Digests for the Withdrawn Version 6 Transaction Format][zip246] for ZSA, according to the
/// (value_pool, tx_version) pair.
///
/// [zip244]: https://zips.z.cash/zip-0244
/// [zip229]: https://zips.z.cash/zip-0229
/// [zip246]: https://zips.z.cash/zip-0246
pub fn hash_bundle_auth_empty(
    value_pool: ValuePool,
    tx_version: TxVersion,
) -> Result<Blake2bHash, CommitmentError> {
    Ok(hasher(
        value_pool
            .commitment_format(tx_version)?
            .personalizations()
            .auth,
    )
    .finalize())
}

/// Encodes a size in the CompactSize format.
///
/// This implementation is inspired from `zcash_encoding::CompactSize::write` [code]
/// We cannot use zcash_encoding crate to avoid circular dependency.
///
/// [code]: https://github.com/zcash/librustzcash/blob/8be259c579762f1b0f569453a20c0d0dbeae6c07/components/zcash_encoding/src/lib.rs#L93
pub fn get_compact_size(size: usize) -> Vec<u8> {
    match size {
        s if s < 253 => vec![s as u8],
        s if s <= 0xFFFF => [&[253_u8], &(s as u16).to_le_bytes()[..]].concat(),
        s if s <= 0xFFFFFFFF => [&[254_u8], &(s as u32).to_le_bytes()[..]].concat(),
        s => [&[255_u8], &(s as u64).to_le_bytes()[..]].concat(),
    }
}

#[cfg(all(test, feature = "circuit"))]
mod tests {
    use crate::{
        builder::{Builder, BundleType, UnauthorizedBundle},
        bundle::{
            commitments::{get_compact_size, hash_bundle_auth_data, hash_bundle_txid_data},
            Authorized, Bundle, BundleVersion, TxVersion,
        },
        circuit::ProvingKey,
        keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
        note::AssetBase,
        sighash_kind::test_sighash_info_for_kind,
        value::NoteValue,
        Anchor,
    };
    use rand::{rngs::StdRng, SeedableRng};

    fn generate_bundle(bundle_version: BundleVersion) -> UnauthorizedBundle<i64> {
        let rng = StdRng::seed_from_u64(5);

        let sk = SpendingKey::from_bytes([7; 32]).unwrap();
        let recipient = FullViewingKey::from(&sk).address_at(0u32, Scope::External);

        let mut builder = Builder::new(
            BundleType::DEFAULT,
            bundle_version,
            bundle_version.default_flags(),
            Anchor::from_bytes([0; 32]).unwrap(),
        )
        .unwrap();
        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(10),
                AssetBase::zatoshi(),
                [0u8; 512],
            )
            .unwrap();

        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(20),
                AssetBase::zatoshi(),
                [0u8; 512],
            )
            .unwrap();

        builder.build::<i64>(rng).unwrap().unwrap().0
    }

    /// Like [`generate_bundle`], for an Orchard V3 bundle version that disables cross-address
    /// transfers.
    ///
    /// Such a version pairs every output with a spend addressed to the same expanded receiver,
    /// so `add_output` is rejected and the outputs must be supplied as wallet-controlled change
    /// outputs instead.
    fn generate_change_only_bundle_for_orchard_v3() -> UnauthorizedBundle<i64> {
        let rng = StdRng::seed_from_u64(5);

        let bundle_version = BundleVersion::orchard_v3();

        let sk = SpendingKey::from_bytes([7; 32]).unwrap();
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        let mut builder = Builder::new(
            BundleType::DEFAULT,
            bundle_version,
            bundle_version.default_flags(),
            Anchor::from_bytes([0; 32]).unwrap(),
        )
        .unwrap();

        builder
            .add_change_output(
                fvk.clone(),
                None,
                recipient,
                NoteValue::from_raw(10),
                AssetBase::zatoshi(),
                [0u8; 512],
            )
            .unwrap();

        builder
            .add_change_output(
                fvk,
                None,
                recipient,
                NoteValue::from_raw(20),
                AssetBase::zatoshi(),
                [0u8; 512],
            )
            .unwrap();

        builder.build::<i64>(rng).unwrap().unwrap().0
    }

    /// Verifies that the hash for an Orchard V2 bundle matches a fixed reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_txid_data_for_orchard_v2() {
        let bundle = generate_bundle(BundleVersion::orchard_v2());
        let sighash = hash_bundle_txid_data(&bundle, TxVersion::V5).unwrap();
        assert_eq!(
            sighash.to_hex().as_str(),
            // Bundle hash for Orchard V2 generated using
            // Zcash/Orchard commit: d9bfa9f
            "0ac1e319f6761a8561b7bd3fc0907a5c73ed5590a6c210c4d39ffae1d5741875"
        );
    }

    /// Verifies that the hash for an Orchard V3 bundle matches a fixed reference value.
    ///
    /// Orchard V3 mandates the cross-address restriction, so this bundle is built from change
    /// outputs; see [`generate_change_only_bundle_for_orchard_v3`].
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_txid_data_for_orchard_v3() {
        let bundle = generate_change_only_bundle_for_orchard_v3();
        let sighash = hash_bundle_txid_data(&bundle, TxVersion::V6).unwrap();
        assert_eq!(
            sighash.to_hex().as_str(),
            // Bundle hash for Orchard V3 generated using
            // Zcash/Orchard commit: d9bfa9f
            "ff9e7cdafd4b9f2ca5ac071f46a8066f7cf1ece9ce89f1760281bfe6b05154fb"
        );
    }

    /// Verifies that the hash for an Ironwood V3 bundle matches a fixed reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_txid_data_for_ironwood_v3() {
        let bundle = generate_bundle(BundleVersion::ironwood_v3());
        let sighash = hash_bundle_txid_data(&bundle, TxVersion::V6).unwrap();
        assert_eq!(
            sighash.to_hex().as_str(),
            // Bundle hash for Ironwood V3 generated using
            // Zcash/Orchard commit: d9bfa9f
            "5f408a48dff8e499487169a9adb4f59ec41d7d393e8d4ee15e70e51c67b018a3"
        );
    }

    /// Verifies that the hash for a ZSA bundle matches a fixed reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_txid_data_for_zsa() {
        let bundle = generate_bundle(BundleVersion::zsa());
        let sighash = hash_bundle_txid_data(&bundle, TxVersion::ZSA).unwrap();
        assert_eq!(
            sighash.to_hex().as_str(),
            "58ffd40ba5bbc328ac4a9ee00385a7938de0117a295782e38c4b1895ce197bf4"
        );
    }

    fn generate_auth_bundle(
        bundle_version: BundleVersion,
        tx_version: TxVersion,
    ) -> Bundle<Authorized, i64> {
        let mut rng = StdRng::seed_from_u64(6);
        let pk = ProvingKey::build(bundle_version.circuit_version());
        let bundle = generate_bundle(bundle_version)
            .create_proof(&pk, &mut rng)
            .unwrap();
        let sighash = bundle.commitment(tx_version).unwrap().into();
        bundle.prepare(rng, sighash).finalize().unwrap()
    }

    /// Like [`generate_auth_bundle`], for an Orchard V3 bundle version that disables
    /// cross-address transfers.
    ///
    /// Each change output pairs with a spend the wallet controls (`dummy_sk: None`), so unlike
    /// [`generate_auth_bundle`], whose spends are all padding dummies that `prepare` signs
    /// itself, this one has to sign with the spend authorizing key.
    fn generate_change_only_auth_bundle_for_orchard_v3() -> Bundle<Authorized, i64> {
        let mut rng = StdRng::seed_from_u64(6);
        let sk = SpendingKey::from_bytes([7; 32]).unwrap();
        let pk = ProvingKey::build(BundleVersion::orchard_v3().circuit_version());
        let bundle = generate_change_only_bundle_for_orchard_v3()
            .create_proof(&pk, &mut rng)
            .unwrap();
        let sighash = bundle.commitment(TxVersion::V6).unwrap().into();
        bundle
            .prepare(&mut rng, sighash)
            .sign(&mut rng, &SpendAuthorizingKey::from(&sk))
            .finalize()
            .unwrap()
    }

    /// Verifies that the authorizing data commitment for an Orchard V2 bundle matches a fixed
    /// reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_auth_data_for_orchard_v2() {
        let bundle = generate_auth_bundle(BundleVersion::orchard_v2(), TxVersion::V5);
        let orchard_auth_digest =
            hash_bundle_auth_data(&bundle, TxVersion::V5, test_sighash_info_for_kind).unwrap();
        assert_eq!(
            orchard_auth_digest.to_hex().as_str(),
            // Bundle hash for Orchard V2 generated using
            // Zcash/Orchard commit: d9bfa9f
            "37d6c29faa98c2cb54420f3f7cac0477fdb105df1cdfde7adb7fbf68a24e3085"
        );
    }

    /// Verifies that the authorizing data commitment for an Orchard V3 bundle matches a fixed
    /// reference value.
    ///
    /// Orchard V3 mandates the cross-address restriction, so this bundle is built from change
    /// outputs; see [`generate_change_only_bundle_for_orchard_v3`].
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_auth_data_for_orchard_v3() {
        let bundle = generate_change_only_auth_bundle_for_orchard_v3();
        let orchard_auth_digest =
            hash_bundle_auth_data(&bundle, TxVersion::V6, test_sighash_info_for_kind).unwrap();
        assert_eq!(
            orchard_auth_digest.to_hex().as_str(),
            // Bundle hash for Orchard V3 generated using
            // Zcash/Orchard commit: d9bfa9f
            "1b1e51877f967fb55d2d240e029471494754c2c6235f50eeb8740331dec47bcc"
        );
    }

    /// Verifies that the authorizing data commitment for an Ironwood V3 bundle matches a fixed
    /// reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_auth_data_for_ironwood_v3() {
        let bundle = generate_auth_bundle(BundleVersion::ironwood_v3(), TxVersion::V6);
        let orchard_auth_digest =
            hash_bundle_auth_data(&bundle, TxVersion::V6, test_sighash_info_for_kind).unwrap();
        assert_eq!(
            orchard_auth_digest.to_hex().as_str(),
            // Bundle hash for Ironwood V3 generated using
            // Zcash/Orchard commit: d9bfa9f
            "404871b295dd65dfe4b44c5bfe92bdd194c0f70bc090a4f50b43a51244a37e5f"
        );
    }

    /// Verifies that the authorizing data commitment for a ZSA bundle matches a fixed
    /// reference value.
    ///
    /// This is a regression test: inputs are fully deterministic (seeded RNG and fixed
    /// bundle contents), so the resulting digest must remain stable. The reference value
    /// was (re)generated after intentional changes that affect the digest, and
    /// is now treated as the expected output for this implementation.
    #[test]
    fn test_hash_bundle_auth_data_for_zsa() {
        let bundle = generate_auth_bundle(BundleVersion::zsa(), TxVersion::ZSA);
        let orchard_auth_digest =
            hash_bundle_auth_data(&bundle, TxVersion::ZSA, test_sighash_info_for_kind).unwrap();
        assert_eq!(
            orchard_auth_digest.to_hex().as_str(),
            "c0be0434184da59e1920bb32f9b036ac56be76369406374c8ace46b23b4e6267"
        );
    }

    #[test]
    fn test_compact_size() {
        assert_eq!(get_compact_size(0), vec![0]);
        assert_eq!(get_compact_size(1), vec![1]);
        assert_eq!(get_compact_size(252), vec![252]);
        assert_eq!(get_compact_size(253), vec![253, 253, 0]);
        assert_eq!(get_compact_size(254), vec![253, 254, 0]);
        assert_eq!(get_compact_size(255), vec![253, 255, 0]);
        assert_eq!(get_compact_size(65535), vec![253, 255, 255]);
        assert_eq!(get_compact_size(65536), vec![254, 0, 0, 1, 0]);
        assert_eq!(get_compact_size(65537), vec![254, 1, 0, 1, 0]);
        assert_eq!(get_compact_size(33554432), vec![254, 0, 0, 0, 2]);
    }
}
