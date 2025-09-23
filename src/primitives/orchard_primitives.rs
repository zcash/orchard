//! The OrchardPrimitives trait represents the difference between the `OrchardVanilla` and the
//! `OrchardZSA` commitment, encryption and decryption procedures.

use alloc::{collections::BTreeMap, vec::Vec};
use core::fmt;

use blake2b_simd::Hash as Blake2bHash;
use zcash_note_encryption::{note_bytes::NoteBytes, AEAD_TAG_SIZE};

use crate::{
    bundle::{Authorization, Authorized},
    note::AssetBase,
    orchard_sighash_versioning::OrchardSighashVersion,
    primitives::zcash_note_encryption_domain::{Memo, MEMO_SIZE},
    Bundle, Note,
};

/// Represents the Orchard protocol domain specifics required for commitment, note encryption and
/// decryption.
pub trait OrchardPrimitives: fmt::Debug + Clone {
    /// The size of a compact note, specific to the Orchard protocol.
    const COMPACT_NOTE_SIZE: usize;

    /// The size of a note plaintext, including memo and other metadata.
    const NOTE_PLAINTEXT_SIZE: usize = Self::COMPACT_NOTE_SIZE + MEMO_SIZE;

    /// The size of an encrypted note ciphertext, accounting for additional AEAD tag space.
    const ENC_CIPHERTEXT_SIZE: usize = Self::NOTE_PLAINTEXT_SIZE + AEAD_TAG_SIZE;

    /// The raw bytes of a note plaintext.
    type NotePlaintextBytes: NoteBytes;
    /// The raw bytes of an encrypted note plaintext.
    type NoteCiphertextBytes: NoteBytes;
    /// The raw bytes of a compact note.
    type CompactNotePlaintextBytes: NoteBytes;
    /// The raw bytes of an encrypted compact note.
    type CompactNoteCiphertextBytes: NoteBytes;

    /// Builds NotePlaintextBytes from Note and Memo.
    fn build_note_plaintext_bytes(note: &Note, memo: &Memo) -> Self::NotePlaintextBytes;

    /// Extracts the asset from the note plaintext.
    fn extract_asset(plaintext: &Self::CompactNotePlaintextBytes) -> Option<AssetBase>;

    /// Evaluate `orchard_digest` for the bundle as defined in
    /// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
    /// for OrchardVanilla and as defined in
    /// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
    /// for OrchardZSA
    ///
    /// [zip244]: https://zips.z.cash/zip-0244
    /// [zip246]: https://zips.z.cash/zip-0246
    fn hash_bundle_txid_data<A: Authorization, V: Copy + Into<i64>>(
        bundle: &Bundle<A, V, Self>,
    ) -> Blake2bHash;

    /// Evaluate `orchard_auth_digest` for the bundle as defined in
    /// [ZIP-244: Transaction Identifier Non-Malleability][zip244]
    /// for OrchardVanilla and as defined in
    /// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
    /// for OrchardZSA
    ///
    /// The `sighash_version_map` provides the mapping from each
    /// `OrchardSighashVersion` to the corresponding `SighashInfo`
    /// encoding.
    ///
    /// [zip244]: https://zips.z.cash/zip-0244
    /// [zip246]: https://zips.z.cash/zip-0246
    fn hash_bundle_auth_data<V>(
        bundle: &Bundle<Authorized, V, Self>,
        sighash_version_map: &BTreeMap<OrchardSighashVersion, Vec<u8>>,
    ) -> Blake2bHash;

    /// Returns the default Orchard sighash version.
    ///
    /// For OrchardVanilla, the default version is `OrchardSighashVersion::NoVersion`.
    /// For OrchardZSA, the default version is `OrchardSighashVersion::V0`.
    fn default_sighash_version() -> OrchardSighashVersion;
}
