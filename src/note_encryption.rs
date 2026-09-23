//! In-band secret distribution for Orchard bundles.

use alloc::vec::Vec;
use core::fmt;

use blake2b_simd::{Hash, Params};
use group::ff::PrimeField;
use zcash_note_encryption::{
    note_bytes::{NoteBytes, NoteBytesData},
    BatchDomain, Domain, EphemeralKeyBytes, OutPlaintextBytes, OutgoingCipherKey, ShieldedOutput,
    AEAD_TAG_SIZE, OUT_PLAINTEXT_SIZE,
};

use crate::{
    action::Action,
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        OutgoingViewingKey, PreparedEphemeralPublicKey, PreparedIncomingViewingKey, SharedSecret,
    },
    note::{AssetBase, ExtractedNoteCommitment, NoteVersion, Nullifier, RandomSeed, Rho},
    value::{NoteValue, ValueCommitment},
    Address, Note,
};

const PRF_OCK_ORCHARD_PERSONALIZATION: &[u8; 16] = b"Zcash_Orchardock";

const NOTE_VERSION_SIZE: usize = 1;
const NOTE_DIVERSIFIER_SIZE: usize = 11;
const NOTE_VALUE_SIZE: usize = 8;
const NOTE_RSEED_SIZE: usize = 32; // rseed (or rcm prior to ZIP 212)

const NOTE_VERSION_OFFSET: usize = 0;
const NOTE_DIVERSIFIER_OFFSET: usize = NOTE_VERSION_OFFSET + NOTE_VERSION_SIZE;
const NOTE_VALUE_OFFSET: usize = NOTE_DIVERSIFIER_OFFSET + NOTE_DIVERSIFIER_SIZE;
const NOTE_RSEED_OFFSET: usize = NOTE_VALUE_OFFSET + NOTE_VALUE_SIZE;

/// The size of a Vanilla compact note.
pub const COMPACT_NOTE_SIZE_VANILLA: usize = NOTE_RSEED_OFFSET + NOTE_RSEED_SIZE;

/// The size of the encoding of a ZSA asset.
const ZSA_ASSET_SIZE: usize = 32;

/// The size of a ZSA compact note.
pub const COMPACT_NOTE_SIZE_ZSA: usize = COMPACT_NOTE_SIZE_VANILLA + ZSA_ASSET_SIZE;

/// The size of the memo.
pub(crate) const MEMO_SIZE: usize = 512;

pub(crate) type Memo = [u8; MEMO_SIZE];

/// The size of a Vanilla note plaintext, including memo.
const NOTE_PLAINTEXT_SIZE_VANILLA: usize = COMPACT_NOTE_SIZE_VANILLA + MEMO_SIZE;
/// The size of a ZSA note plaintext, including memo.
const NOTE_PLAINTEXT_SIZE_ZSA: usize = COMPACT_NOTE_SIZE_ZSA + MEMO_SIZE;
/// The size of a Vanilla encrypted note ciphertext, accounting for the AEAD tag.
pub const ENC_CIPHERTEXT_SIZE_VANILLA: usize = NOTE_PLAINTEXT_SIZE_VANILLA + AEAD_TAG_SIZE;
/// The size of a ZSA encrypted note ciphertext, accounting for the AEAD tag.
pub const ENC_CIPHERTEXT_SIZE_ZSA: usize = NOTE_PLAINTEXT_SIZE_ZSA + AEAD_TAG_SIZE;

/// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
///
/// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
pub(crate) fn prf_ock_orchard(
    ovk: &OutgoingViewingKey,
    cv: &ValueCommitment,
    cmx_bytes: &[u8; 32],
    ephemeral_key: &EphemeralKeyBytes,
) -> OutgoingCipherKey {
    OutgoingCipherKey(
        Params::new()
            .hash_length(32)
            .personal(PRF_OCK_ORCHARD_PERSONALIZATION)
            .to_state()
            .update(ovk.as_ref())
            .update(&cv.to_bytes())
            .update(cmx_bytes)
            .update(ephemeral_key.as_ref())
            .finalize()
            .as_bytes()
            .try_into()
            .unwrap(),
    )
}

fn parse_note_plaintext_without_memo<F>(
    rho: Rho,
    plaintext: &CompactNotePlaintextBytes,
    note_version: NoteVersion,
    get_pk_d: F,
) -> Option<(Note, Address)>
where
    F: FnOnce(&Diversifier) -> DiversifiedTransmissionKey,
{
    // The domain fixes `note_version` but the buffer's shape is a runtime variant, so pin the
    // length: exact, since every domain accepts one version and the encoder always emits this size.
    if plaintext.as_ref().len() != compact_note_size(note_version) {
        return None;
    }

    // The unwraps below are guaranteed to succeed by the length check above
    let diversifier = Diversifier::from_bytes(
        plaintext.as_ref()[NOTE_DIVERSIFIER_OFFSET..NOTE_VALUE_OFFSET]
            .try_into()
            .unwrap(),
    );

    let value = NoteValue::from_bytes(
        plaintext.as_ref()[NOTE_VALUE_OFFSET..NOTE_RSEED_OFFSET]
            .try_into()
            .unwrap(),
    );

    let rseed = Option::from(RandomSeed::from_bytes(
        plaintext.as_ref()[NOTE_RSEED_OFFSET..COMPACT_NOTE_SIZE_VANILLA]
            .try_into()
            .unwrap(),
        &rho,
    ))?;

    let pk_d = get_pk_d(&diversifier);

    let recipient = Address::from_parts(diversifier, pk_d);
    let asset = extract_asset(note_version, plaintext.as_ref())?;
    let note = Option::from(Note::from_parts(
        recipient,
        value,
        asset,
        rho,
        rseed,
        note_version,
    ))?;

    Some((note, recipient))
}

/// A note plaintext, in either its Vanilla or ZSA form. The two forms differ in size (the ZSA
/// form embeds the note's [`AssetBase`] after the Vanilla compact fields).
#[derive(Clone, Copy, Debug)]
pub enum NotePlaintextBytes {
    /// A Vanilla note plaintext.
    Vanilla(NoteBytesData<NOTE_PLAINTEXT_SIZE_VANILLA>),
    /// A ZSA note plaintext.
    Zsa(NoteBytesData<NOTE_PLAINTEXT_SIZE_ZSA>),
}

/// An encrypted note ciphertext, in either its Vanilla or ZSA form.
#[derive(Clone, Copy, Debug)]
pub enum NoteCiphertextBytes {
    /// A Vanilla note ciphertext.
    Vanilla(NoteBytesData<ENC_CIPHERTEXT_SIZE_VANILLA>),
    /// A ZSA note ciphertext.
    Zsa(NoteBytesData<ENC_CIPHERTEXT_SIZE_ZSA>),
}

/// A compact note plaintext, in either its Vanilla or ZSA form.
#[derive(Clone, Copy, Debug)]
pub enum CompactNotePlaintextBytes {
    /// A Vanilla compact note plaintext.
    Vanilla(NoteBytesData<COMPACT_NOTE_SIZE_VANILLA>),
    /// A ZSA compact note plaintext.
    Zsa(NoteBytesData<COMPACT_NOTE_SIZE_ZSA>),
}

/// A compact, encrypted note ciphertext, in either its Vanilla or ZSA form.
#[derive(Clone, Copy, Debug)]
pub enum CompactNoteCiphertextBytes {
    /// A Vanilla compact note ciphertext.
    Vanilla(NoteBytesData<COMPACT_NOTE_SIZE_VANILLA>),
    /// A ZSA compact note ciphertext.
    Zsa(NoteBytesData<COMPACT_NOTE_SIZE_ZSA>),
}

impl AsRef<[u8]> for NotePlaintextBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            NotePlaintextBytes::Vanilla(b) => b.as_ref(),
            NotePlaintextBytes::Zsa(b) => b.as_ref(),
        }
    }
}

impl AsMut<[u8]> for NotePlaintextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            NotePlaintextBytes::Vanilla(b) => b.as_mut(),
            NotePlaintextBytes::Zsa(b) => b.as_mut(),
        }
    }
}

impl NoteBytes for NotePlaintextBytes {
    fn from_slice(bytes: &[u8]) -> Option<Self> {
        match bytes.len() {
            NOTE_PLAINTEXT_SIZE_VANILLA => {
                NoteBytesData::from_slice(bytes).map(NotePlaintextBytes::Vanilla)
            }
            NOTE_PLAINTEXT_SIZE_ZSA => {
                NoteBytesData::from_slice(bytes).map(NotePlaintextBytes::Zsa)
            }
            _ => None,
        }
    }

    fn from_slice_with_tag<const TAG_SIZE: usize>(
        output: &[u8],
        tag: [u8; TAG_SIZE],
    ) -> Option<Self> {
        match output.len() + TAG_SIZE {
            NOTE_PLAINTEXT_SIZE_VANILLA => {
                NoteBytesData::from_slice_with_tag(output, tag).map(NotePlaintextBytes::Vanilla)
            }
            NOTE_PLAINTEXT_SIZE_ZSA => {
                NoteBytesData::from_slice_with_tag(output, tag).map(NotePlaintextBytes::Zsa)
            }
            _ => None,
        }
    }
}

impl AsRef<[u8]> for NoteCiphertextBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            NoteCiphertextBytes::Vanilla(b) => b.as_ref(),
            NoteCiphertextBytes::Zsa(b) => b.as_ref(),
        }
    }
}

impl AsMut<[u8]> for NoteCiphertextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            NoteCiphertextBytes::Vanilla(b) => b.as_mut(),
            NoteCiphertextBytes::Zsa(b) => b.as_mut(),
        }
    }
}

impl NoteBytes for NoteCiphertextBytes {
    fn from_slice(bytes: &[u8]) -> Option<Self> {
        match bytes.len() {
            ENC_CIPHERTEXT_SIZE_VANILLA => {
                NoteBytesData::from_slice(bytes).map(NoteCiphertextBytes::Vanilla)
            }
            ENC_CIPHERTEXT_SIZE_ZSA => {
                NoteBytesData::from_slice(bytes).map(NoteCiphertextBytes::Zsa)
            }
            _ => None,
        }
    }

    fn from_slice_with_tag<const TAG_SIZE: usize>(
        output: &[u8],
        tag: [u8; TAG_SIZE],
    ) -> Option<Self> {
        match output.len() + TAG_SIZE {
            ENC_CIPHERTEXT_SIZE_VANILLA => {
                NoteBytesData::from_slice_with_tag(output, tag).map(NoteCiphertextBytes::Vanilla)
            }
            ENC_CIPHERTEXT_SIZE_ZSA => {
                NoteBytesData::from_slice_with_tag(output, tag).map(NoteCiphertextBytes::Zsa)
            }
            _ => None,
        }
    }
}

impl AsRef<[u8]> for CompactNotePlaintextBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            CompactNotePlaintextBytes::Vanilla(b) => b.as_ref(),
            CompactNotePlaintextBytes::Zsa(b) => b.as_ref(),
        }
    }
}

impl AsMut<[u8]> for CompactNotePlaintextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            CompactNotePlaintextBytes::Vanilla(b) => b.as_mut(),
            CompactNotePlaintextBytes::Zsa(b) => b.as_mut(),
        }
    }
}

impl NoteBytes for CompactNotePlaintextBytes {
    fn from_slice(bytes: &[u8]) -> Option<Self> {
        match bytes.len() {
            COMPACT_NOTE_SIZE_VANILLA => {
                NoteBytesData::from_slice(bytes).map(CompactNotePlaintextBytes::Vanilla)
            }
            COMPACT_NOTE_SIZE_ZSA => {
                NoteBytesData::from_slice(bytes).map(CompactNotePlaintextBytes::Zsa)
            }
            _ => None,
        }
    }

    fn from_slice_with_tag<const TAG_SIZE: usize>(
        output: &[u8],
        tag: [u8; TAG_SIZE],
    ) -> Option<Self> {
        match output.len() + TAG_SIZE {
            COMPACT_NOTE_SIZE_VANILLA => NoteBytesData::from_slice_with_tag(output, tag)
                .map(CompactNotePlaintextBytes::Vanilla),
            COMPACT_NOTE_SIZE_ZSA => {
                NoteBytesData::from_slice_with_tag(output, tag).map(CompactNotePlaintextBytes::Zsa)
            }
            _ => None,
        }
    }
}

impl AsRef<[u8]> for CompactNoteCiphertextBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            CompactNoteCiphertextBytes::Vanilla(b) => b.as_ref(),
            CompactNoteCiphertextBytes::Zsa(b) => b.as_ref(),
        }
    }
}

impl AsMut<[u8]> for CompactNoteCiphertextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            CompactNoteCiphertextBytes::Vanilla(b) => b.as_mut(),
            CompactNoteCiphertextBytes::Zsa(b) => b.as_mut(),
        }
    }
}

impl NoteBytes for CompactNoteCiphertextBytes {
    fn from_slice(bytes: &[u8]) -> Option<Self> {
        match bytes.len() {
            COMPACT_NOTE_SIZE_VANILLA => {
                NoteBytesData::from_slice(bytes).map(CompactNoteCiphertextBytes::Vanilla)
            }
            COMPACT_NOTE_SIZE_ZSA => {
                NoteBytesData::from_slice(bytes).map(CompactNoteCiphertextBytes::Zsa)
            }
            _ => None,
        }
    }

    fn from_slice_with_tag<const TAG_SIZE: usize>(
        output: &[u8],
        tag: [u8; TAG_SIZE],
    ) -> Option<Self> {
        match output.len() + TAG_SIZE {
            COMPACT_NOTE_SIZE_VANILLA => NoteBytesData::from_slice_with_tag(output, tag)
                .map(CompactNoteCiphertextBytes::Vanilla),
            COMPACT_NOTE_SIZE_ZSA => {
                NoteBytesData::from_slice_with_tag(output, tag).map(CompactNoteCiphertextBytes::Zsa)
            }
            _ => None,
        }
    }
}

impl NoteCiphertextBytes {
    /// The length of the compact prefix of this ciphertext (nullifier-revealing fields), i.e.
    /// [`COMPACT_NOTE_SIZE_VANILLA`]/[`COMPACT_NOTE_SIZE_ZSA`] depending on the variant.
    fn compact_size(&self) -> usize {
        match self {
            NoteCiphertextBytes::Vanilla(_) => COMPACT_NOTE_SIZE_VANILLA,
            NoteCiphertextBytes::Zsa(_) => COMPACT_NOTE_SIZE_ZSA,
        }
    }
}

/// Returns the size of an encrypted note ciphertext for a note of `note_version`.
pub(crate) fn enc_ciphertext_size(note_version: NoteVersion) -> usize {
    match note_version {
        NoteVersion::V2 | NoteVersion::V3 => ENC_CIPHERTEXT_SIZE_VANILLA,
        NoteVersion::ZSA => ENC_CIPHERTEXT_SIZE_ZSA,
    }
}

/// Returns the size of a compact note plaintext for a note of `note_version`.
fn compact_note_size(note_version: NoteVersion) -> usize {
    match note_version {
        NoteVersion::V2 | NoteVersion::V3 => COMPACT_NOTE_SIZE_VANILLA,
        NoteVersion::ZSA => COMPACT_NOTE_SIZE_ZSA,
    }
}

// Constructs a note plaintext bytes array given note information.
pub(crate) fn build_base_note_plaintext_bytes<const NOTE_PLAINTEXT_SIZE: usize>(
    note: &Note,
) -> [u8; NOTE_PLAINTEXT_SIZE] {
    let mut np = [0; NOTE_PLAINTEXT_SIZE];

    np[NOTE_VERSION_OFFSET] = note.version().lead_byte();
    np[NOTE_DIVERSIFIER_OFFSET..NOTE_VALUE_OFFSET]
        .copy_from_slice(note.recipient().diversifier().as_array());
    np[NOTE_VALUE_OFFSET..NOTE_RSEED_OFFSET].copy_from_slice(&note.value().to_bytes());
    np[NOTE_RSEED_OFFSET..COMPACT_NOTE_SIZE_VANILLA].copy_from_slice(note.rseed().as_bytes());

    np
}

/// Builds the note plaintext bytes for `note`, with `memo` appended.
///
/// The plaintext shape (and therefore its size) is selected by `note.version()`: a
/// [`NoteVersion::ZSA`] note additionally embeds its [`AssetBase`] after the Vanilla compact
/// fields.
pub(crate) fn build_note_plaintext_bytes(note: &Note, memo: &Memo) -> NotePlaintextBytes {
    match note.version() {
        NoteVersion::V2 | NoteVersion::V3 => {
            let mut np = build_base_note_plaintext_bytes::<NOTE_PLAINTEXT_SIZE_VANILLA>(note);
            np[COMPACT_NOTE_SIZE_VANILLA..].copy_from_slice(memo);
            NotePlaintextBytes::Vanilla(NoteBytesData(np))
        }
        NoteVersion::ZSA => {
            let mut np = build_base_note_plaintext_bytes::<NOTE_PLAINTEXT_SIZE_ZSA>(note);
            np[COMPACT_NOTE_SIZE_VANILLA..COMPACT_NOTE_SIZE_ZSA]
                .copy_from_slice(&note.asset().to_bytes());
            np[COMPACT_NOTE_SIZE_ZSA..].copy_from_slice(memo);
            NotePlaintextBytes::Zsa(NoteBytesData(np))
        }
    }
}

/// Extracts the asset from a compact note plaintext.
///
/// [`NoteVersion::V2`] and [`NoteVersion::V3`] plaintexts always denote notes holding zatoshis.
/// [`NoteVersion::ZSA`] plaintexts additionally embed the asset in the bytes following the
/// Vanilla compact note fields.
fn extract_asset(note_version: NoteVersion, plaintext: &[u8]) -> Option<AssetBase> {
    match note_version {
        NoteVersion::V2 | NoteVersion::V3 => Some(AssetBase::zatoshi()),
        NoteVersion::ZSA => {
            let bytes = plaintext
                .get(COMPACT_NOTE_SIZE_VANILLA..COMPACT_NOTE_SIZE_ZSA)?
                .try_into()
                .ok()?;
            AssetBase::from_bytes(bytes).into()
        }
    }
}

mod sealed {
    /// Marker trait that prevents external `DomainVersion` implementations.
    pub trait Sealed {}
}

trait DomainPolicy {
    fn note_version(&self, plaintext: &[u8]) -> Option<NoteVersion>;
}

/// A sealed marker trait for note encryption domains with a fixed note plaintext version.
///
/// This trait is sealed so that only this crate can define supported note encryption
/// domains.
pub trait DomainVersion: sealed::Sealed + Default {
    /// The note plaintext version accepted by this domain during parsing and decryption.
    const NOTE_VERSION: NoteVersion;
}

impl<V: DomainVersion> DomainPolicy for V {
    fn note_version(&self, plaintext: &[u8]) -> Option<NoteVersion> {
        if plaintext.first().copied() == Some(V::NOTE_VERSION.lead_byte()) {
            Some(V::NOTE_VERSION)
        } else {
            None
        }
    }
}

/// Marker type for Orchard note encryption domains.
#[derive(Default, Debug)]
pub struct OrchardVersion;

impl sealed::Sealed for OrchardVersion {}

impl DomainVersion for OrchardVersion {
    const NOTE_VERSION: NoteVersion = NoteVersion::V2;
}

/// Marker type for Ironwood note encryption domains.
#[derive(Default, Debug)]
pub struct IronwoodVersion;

impl sealed::Sealed for IronwoodVersion {}

impl DomainVersion for IronwoodVersion {
    const NOTE_VERSION: NoteVersion = NoteVersion::V3;
}

/// Marker type for ZSA note encryption domains.
#[derive(Default, Debug)]
pub struct ZSAVersion;

impl sealed::Sealed for ZSAVersion {}

impl DomainVersion for ZSAVersion {
    const NOTE_VERSION: NoteVersion = NoteVersion::ZSA;
}

#[derive(Debug)]
pub(crate) struct BundleDomainPolicy {
    note_version: NoteVersion,
}

impl DomainPolicy for BundleDomainPolicy {
    fn note_version(&self, plaintext: &[u8]) -> Option<NoteVersion> {
        let note_version = NoteVersion::from_lead_byte(*plaintext.first()?)?;
        if note_version == self.note_version {
            Some(note_version)
        } else {
            None
        }
    }
}

/// Note encryption logic for a note plaintext version policy.
///
/// The policy type `P` selects which note plaintext version is accepted during
/// parsing and decryption. Encryption uses the version recorded by the note.
#[derive(Debug, Clone)]
pub struct NoteEncryptionDomain<P> {
    rho: Rho,
    policy: P,
}

impl<P> memuse::DynamicUsage for NoteEncryptionDomain<P> {
    fn dynamic_usage(&self) -> usize {
        self.rho.dynamic_usage()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        self.rho.dynamic_usage_bounds()
    }
}

impl<V: DomainVersion> NoteEncryptionDomain<V> {
    pub(crate) fn from_rho(rho: Rho) -> Self {
        Self {
            rho,
            policy: V::default(),
        }
    }

    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_action<T>(act: &Action<T>) -> Self {
        Self::from_rho(act.rho())
    }

    /// Constructs a domain that can be used to trial-decrypt a PCZT action's output note.
    pub fn for_pczt_action(act: &crate::pczt::Action) -> Self {
        Self::from_rho(Rho::from_nf_old(act.spend().nullifier))
    }

    /// Constructs a domain that can be used to trial-decrypt this compact action's output note.
    pub fn for_compact_action(act: &CompactAction) -> Self {
        Self::from_rho(act.rho())
    }
}

/// Orchard-specific note encryption logic.
///
/// This domain accepts only [`NoteVersion::V2`] note plaintexts, which use lead
/// byte `0x02`.
pub type OrchardDomain = NoteEncryptionDomain<OrchardVersion>;

/// Ironwood-specific note encryption logic.
///
/// This domain is otherwise identical to [`OrchardDomain`], but accepts only
/// [`NoteVersion::V3`] note plaintexts, which use lead byte `0x03`.
pub type IronwoodDomain = NoteEncryptionDomain<IronwoodVersion>;

/// ZSA-specific note encryption logic.
///
/// This domain is otherwise identical to [`OrchardDomain`], but accepts only
/// [`NoteVersion::ZSA`] note plaintexts, which use lead byte `0x04`.
pub type ZSADomain = NoteEncryptionDomain<ZSAVersion>;

/// Note encryption logic restricted to a single note plaintext version.
///
/// This domain is used by public bundle helpers that are given the bundle's
/// [`NoteVersion`]. Trial decryption still happens once; after decryption
/// succeeds, the revealed note plaintext lead byte selects the note version, which is
/// enforced to match the expected one.
pub(crate) type BundleDomain = NoteEncryptionDomain<BundleDomainPolicy>;

impl BundleDomain {
    /// Constructs a domain that can be used to trial-decrypt this action's
    /// output note as a note of `note_version`.
    pub(crate) fn for_action<T>(act: &Action<T>, note_version: NoteVersion) -> Self {
        Self {
            rho: act.rho(),
            policy: BundleDomainPolicy { note_version },
        }
    }
}

impl<P: DomainPolicy> Domain for NoteEncryptionDomain<P> {
    type EphemeralSecretKey = EphemeralSecretKey;
    type EphemeralPublicKey = EphemeralPublicKey;
    type PreparedEphemeralPublicKey = PreparedEphemeralPublicKey;
    type SharedSecret = SharedSecret;
    type SymmetricKey = Hash;
    type Note = Note;
    type Recipient = Address;
    type DiversifiedTransmissionKey = DiversifiedTransmissionKey;
    type IncomingViewingKey = PreparedIncomingViewingKey;
    type OutgoingViewingKey = OutgoingViewingKey;
    type ValueCommitment = ValueCommitment;
    type ExtractedCommitment = ExtractedNoteCommitment;
    type ExtractedCommitmentBytes = [u8; 32];
    type Memo = Memo;

    type NotePlaintextBytes = NotePlaintextBytes;
    type NoteCiphertextBytes = NoteCiphertextBytes;
    type CompactNotePlaintextBytes = CompactNotePlaintextBytes;
    type CompactNoteCiphertextBytes = CompactNoteCiphertextBytes;

    fn derive_esk(note: &Self::Note) -> Option<Self::EphemeralSecretKey> {
        Some(note.esk())
    }

    fn get_pk_d(note: &Self::Note) -> Self::DiversifiedTransmissionKey {
        *note.recipient().pk_d()
    }

    fn prepare_epk(epk: Self::EphemeralPublicKey) -> Self::PreparedEphemeralPublicKey {
        PreparedEphemeralPublicKey::new(epk)
    }

    fn ka_derive_public(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> Self::EphemeralPublicKey {
        esk.derive_public(note.recipient().g_d())
    }

    fn ka_agree_enc(
        esk: &Self::EphemeralSecretKey,
        pk_d: &Self::DiversifiedTransmissionKey,
    ) -> Self::SharedSecret {
        esk.agree(pk_d)
    }

    fn ka_agree_dec(
        ivk: &Self::IncomingViewingKey,
        epk: &Self::PreparedEphemeralPublicKey,
    ) -> Self::SharedSecret {
        epk.agree(ivk)
    }

    fn kdf(secret: Self::SharedSecret, ephemeral_key: &EphemeralKeyBytes) -> Self::SymmetricKey {
        secret.kdf_orchard(ephemeral_key)
    }

    fn note_plaintext_bytes(note: &Self::Note, memo: &Self::Memo) -> NotePlaintextBytes {
        build_note_plaintext_bytes(note, memo)
    }

    fn derive_ock(
        ovk: &Self::OutgoingViewingKey,
        cv: &Self::ValueCommitment,
        cmstar_bytes: &Self::ExtractedCommitmentBytes,
        ephemeral_key: &EphemeralKeyBytes,
    ) -> OutgoingCipherKey {
        prf_ock_orchard(ovk, cv, cmstar_bytes, ephemeral_key)
    }

    fn outgoing_plaintext_bytes(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> OutPlaintextBytes {
        let mut op = [0; OUT_PLAINTEXT_SIZE];
        op[..32].copy_from_slice(&note.recipient().pk_d().to_bytes());
        op[32..].copy_from_slice(&esk.0.to_repr());
        OutPlaintextBytes(op)
    }

    fn epk_bytes(epk: &Self::EphemeralPublicKey) -> EphemeralKeyBytes {
        epk.to_bytes()
    }

    fn epk(ephemeral_key: &EphemeralKeyBytes) -> Option<Self::EphemeralPublicKey> {
        EphemeralPublicKey::from_bytes(&ephemeral_key.0).into()
    }

    fn cmstar(note: &Self::Note) -> Self::ExtractedCommitment {
        note.commitment().into()
    }

    fn parse_note_plaintext_without_memo_ivk(
        &self,
        ivk: &Self::IncomingViewingKey,
        plaintext: &CompactNotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        let note_version = self.policy.note_version(plaintext.as_ref())?;
        parse_note_plaintext_without_memo(self.rho, plaintext, note_version, |diversifier| {
            DiversifiedTransmissionKey::derive(ivk, diversifier)
        })
    }

    fn parse_note_plaintext_without_memo_ovk(
        &self,
        pk_d: &Self::DiversifiedTransmissionKey,
        plaintext: &CompactNotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        let note_version = self.policy.note_version(plaintext.as_ref())?;
        parse_note_plaintext_without_memo(self.rho, plaintext, note_version, |_| *pk_d)
    }

    fn split_plaintext_at_memo(
        &self,
        plaintext: &NotePlaintextBytes,
    ) -> Option<(Self::CompactNotePlaintextBytes, Self::Memo)> {
        let compact_size = match plaintext {
            NotePlaintextBytes::Vanilla(_) => COMPACT_NOTE_SIZE_VANILLA,
            NotePlaintextBytes::Zsa(_) => COMPACT_NOTE_SIZE_ZSA,
        };
        let (compact, memo) = plaintext.as_ref().split_at(compact_size);
        Some((
            CompactNotePlaintextBytes::from_slice(compact)?,
            memo.try_into().ok()?,
        ))
    }

    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<Self::DiversifiedTransmissionKey> {
        DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap()).into()
    }

    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<Self::EphemeralSecretKey> {
        EphemeralSecretKey::from_bytes(out_plaintext.0[32..OUT_PLAINTEXT_SIZE].try_into().unwrap())
            .into()
    }
}

impl<P: DomainPolicy> BatchDomain for NoteEncryptionDomain<P> {
    fn batch_kdf<'a>(
        items: impl Iterator<Item = (Option<Self::SharedSecret>, &'a EphemeralKeyBytes)>,
    ) -> Vec<Option<Self::SymmetricKey>> {
        batch_kdf(items)
    }

    fn batch_epk(
        ephemeral_keys: impl Iterator<Item = EphemeralKeyBytes>,
    ) -> Vec<(Option<Self::PreparedEphemeralPublicKey>, EphemeralKeyBytes)> {
        // Prepare the whole batch with GLV windows, sharing one batch
        // normalization across every key (a single field inversion, where
        // per-item preparation pays one per key).
        let (epks, ephemeral_keys): (Vec<_>, Vec<_>) = ephemeral_keys
            .map(|ephemeral_key| (Self::epk(&ephemeral_key), ephemeral_key))
            .unzip();
        PreparedEphemeralPublicKey::batch_tabled(epks)
            .into_iter()
            .zip(ephemeral_keys)
            .collect()
    }

    fn batch_ka_agree_dec<'a>(
        ivk: &Self::IncomingViewingKey,
        epks: impl Iterator<Item = Option<&'a Self::PreparedEphemeralPublicKey>>,
    ) -> Vec<Option<Self::SharedSecret>>
    where
        Self::PreparedEphemeralPublicKey: 'a,
    {
        // One GLV decomposition and digit recoding of the viewing key for the
        // whole batch; each ephemeral key's window is then consumed by a
        // shared-doubling ladder.
        let decomposed =
            pasta_curves::glv::Decomposed::<pasta_curves::pallas::Point>::new(&ivk.raw_scalar());
        epks.map(|epk| epk.map(|epk| epk.agree_with(ivk, &decomposed)))
            .collect()
    }
}

fn batch_kdf<'a>(
    items: impl Iterator<Item = (Option<SharedSecret>, &'a EphemeralKeyBytes)>,
) -> Vec<Option<Hash>> {
    let (shared_secrets, ephemeral_keys): (Vec<_>, Vec<_>) = items.unzip();

    SharedSecret::batch_to_affine(shared_secrets)
        .zip(ephemeral_keys)
        .map(|(secret, ephemeral_key)| {
            secret.map(|dhsecret| SharedSecret::kdf_orchard_inner(dhsecret, ephemeral_key))
        })
        .collect()
}

impl<P: DomainPolicy, A> ShieldedOutput<NoteEncryptionDomain<P>> for Action<A> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.encrypted_note().epk_bytes)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        self.cmx()
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx().to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&NoteCiphertextBytes> {
        Some(&self.encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(&self) -> CompactNoteCiphertextBytes {
        let enc_ciphertext = &self.encrypted_note().enc_ciphertext;
        CompactNoteCiphertextBytes::from_slice(
            &enc_ciphertext.as_ref()[..enc_ciphertext.compact_size()],
        )
        .expect("enc_ciphertext is at least compact_size() bytes")
    }
}

impl<P: DomainPolicy> ShieldedOutput<NoteEncryptionDomain<P>> for crate::pczt::Action {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.output().encrypted_note().epk_bytes)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        self.output().cmx()
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.output().cmx().to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&NoteCiphertextBytes> {
        Some(&self.output().encrypted_note().enc_ciphertext)
    }

    fn enc_ciphertext_compact(&self) -> CompactNoteCiphertextBytes {
        let enc_ciphertext = &self.output().encrypted_note().enc_ciphertext;
        CompactNoteCiphertextBytes::from_slice(
            &enc_ciphertext.as_ref()[..enc_ciphertext.compact_size()],
        )
        .expect("enc_ciphertext is at least compact_size() bytes")
    }
}

impl<P: DomainPolicy> ShieldedOutput<NoteEncryptionDomain<P>> for CompactAction {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.ephemeral_key.0)
    }

    fn cmstar(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx.to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<&NoteCiphertextBytes> {
        None
    }

    fn enc_ciphertext_compact(&self) -> CompactNoteCiphertextBytes {
        self.enc_ciphertext
    }
}

/// Implementation of in-band secret distribution for Orchard bundles.
///
/// This is the [`NoteEncryption`] instantiation for [`OrchardDomain`]. Encryption
/// behavior is shared with [`IronwoodNoteEncryption`]: the note plaintext lead
/// byte is selected from [`crate::Note::version`], while the domain type
/// controls which note plaintext versions are accepted during parsing and
/// decryption.
///
/// [`NoteEncryption`]: zcash_note_encryption::NoteEncryption
pub type OrchardNoteEncryption = zcash_note_encryption::NoteEncryption<OrchardDomain>;
/// Implementation of in-band secret distribution for Ironwood bundles.
///
/// This is the [`NoteEncryption`] instantiation for [`IronwoodDomain`]. Encryption
/// behavior is shared with [`OrchardNoteEncryption`]: the note plaintext lead
/// byte is selected from [`crate::Note::version`], while the domain type
/// controls which note plaintext versions are accepted during parsing and
/// decryption.
///
/// [`NoteEncryption`]: zcash_note_encryption::NoteEncryption
pub type IronwoodNoteEncryption = zcash_note_encryption::NoteEncryption<IronwoodDomain>;
/// Implementation of in-band secret distribution for ZSA bundles.
///
/// This is the [`NoteEncryption`] instantiation for [`ZSADomain`]. Encryption
/// behavior is shared with [`IronwoodNoteEncryption`]: the note plaintext lead
/// byte is selected from [`crate::Note::version`], while the domain type
/// controls which note plaintext versions are accepted during parsing and
/// decryption.
///
/// [`NoteEncryption`]: zcash_note_encryption::NoteEncryption
pub type ZSANoteEncryption = zcash_note_encryption::NoteEncryption<ZSADomain>;

/// A compact Action for light clients.
#[derive(Clone)]
pub struct CompactAction {
    nullifier: Nullifier,
    cmx: ExtractedNoteCommitment,
    ephemeral_key: EphemeralKeyBytes,
    enc_ciphertext: CompactNoteCiphertextBytes,
}

impl fmt::Debug for CompactAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactAction")
    }
}

impl<A> From<&Action<A>> for CompactAction {
    fn from(action: &Action<A>) -> Self {
        let enc_ciphertext = &action.encrypted_note().enc_ciphertext;
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: EphemeralKeyBytes(action.encrypted_note().epk_bytes),
            enc_ciphertext: CompactNoteCiphertextBytes::from_slice(
                &enc_ciphertext.as_ref()[..enc_ciphertext.compact_size()],
            )
            .expect("enc_ciphertext is at least compact_size() bytes"),
        }
    }
}

impl CompactAction {
    /// Create a CompactAction from its constituent parts
    pub fn from_parts(
        nullifier: Nullifier,
        cmx: ExtractedNoteCommitment,
        ephemeral_key: EphemeralKeyBytes,
        enc_ciphertext: CompactNoteCiphertextBytes,
    ) -> Self {
        Self {
            nullifier,
            cmx,
            ephemeral_key,
            enc_ciphertext,
        }
    }

    /// Returns the nullifier of the note being spent.
    pub fn nullifier(&self) -> Nullifier {
        self.nullifier
    }

    /// Returns the commitment to the new note being created.
    pub fn cmx(&self) -> ExtractedNoteCommitment {
        self.cmx
    }

    /// Obtains the [`Rho`] value that was used to construct the new note being created.
    pub fn rho(&self) -> Rho {
        Rho::from_nf_old(self.nullifier)
    }
}

/// Utilities for constructing test data.
#[cfg(feature = "test-dependencies")]
pub mod testing {
    use rand::RngCore;

    use zcash_note_encryption::{note_bytes::NoteBytes, Domain, NoteEncryption};

    use crate::{
        address::Address,
        keys::OutgoingViewingKey,
        note::{AssetBase, ExtractedNoteCommitment, Note, NoteVersion, Nullifier, RandomSeed, Rho},
        value::NoteValue,
    };

    use super::{
        CompactAction, CompactNoteCiphertextBytes, NoteEncryptionDomain, OrchardVersion, MEMO_SIZE,
    };

    /// Creates a fake `CompactAction` paying the given recipient the specified value.
    ///
    /// Returns the `CompactAction` and the new note.
    #[allow(clippy::too_many_arguments)]
    pub fn fake_compact_action<R: RngCore>(
        rng: &mut R,
        nf_old: Nullifier,
        recipient: Address,
        value: NoteValue,
        note_version: NoteVersion,
        ovk: Option<OutgoingViewingKey>,
    ) -> (CompactAction, Note) {
        let rho = Rho::from_nf_old(nf_old);
        let rseed = {
            loop {
                let mut bytes = [0; 32];
                rng.fill_bytes(&mut bytes);
                let rseed = RandomSeed::from_bytes(bytes, &rho);
                if rseed.is_some().into() {
                    break rseed.unwrap();
                }
            }
        };
        let note = Note::from_parts(
            recipient,
            value,
            AssetBase::zatoshi(),
            rho,
            rseed,
            note_version,
        )
        .unwrap();
        let encryptor = NoteEncryption::<NoteEncryptionDomain<OrchardVersion>>::new(
            ovk,
            note,
            [0u8; MEMO_SIZE],
        );
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let ephemeral_key = NoteEncryptionDomain::<OrchardVersion>::epk_bytes(encryptor.epk());
        let enc_ciphertext = encryptor.encrypt_note_plaintext();

        (
            CompactAction {
                nullifier: nf_old,
                cmx,
                ephemeral_key,
                enc_ciphertext: CompactNoteCiphertextBytes::from_slice(
                    &enc_ciphertext.as_ref()[..enc_ciphertext.compact_size()],
                )
                .unwrap(),
            },
            note,
        )
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use proptest::prelude::*;
    use rand::rngs::OsRng;
    use zcash_note_encryption::{
        batch,
        note_bytes::{NoteBytes, NoteBytesData},
        try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk,
        BatchDomain, Domain, EphemeralKeyBytes, NoteEncryption,
    };

    use super::{
        compact_note_size, parse_note_plaintext_without_memo, prf_ock_orchard, CompactAction,
        CompactNoteCiphertextBytes, CompactNotePlaintextBytes, DomainVersion, IronwoodDomain,
        IronwoodNoteEncryption, IronwoodVersion, NoteCiphertextBytes, NoteEncryptionDomain,
        OrchardDomain, OrchardNoteEncryption, OrchardVersion, ZSADomain, ZSAVersion,
        COMPACT_NOTE_SIZE_VANILLA, COMPACT_NOTE_SIZE_ZSA,
    };
    use crate::note::AssetBase;
    use crate::{
        action::Action,
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, FullViewingKey,
            IncomingViewingKey, OutgoingViewingKey, PreparedIncomingViewingKey, Scope, SpendingKey,
        },
        note::{
            testing::{arb_note, arb_zatoshi_note},
            ExtractedNoteCommitment, NoteVersion, Nullifier, RandomSeed, Rho,
            TransmittedNoteCiphertext,
        },
        primitives::redpallas,
        value::{NoteValue, ValueCommitTrapdoor, ValueCommitment, ValueSum},
        Address, Note,
    };

    proptest! {
        #[test]
        fn encoding_roundtrip_vanilla(
            note in arb_zatoshi_note(NoteVersion::V2),
        ) {
            let memo = &crate::test_vectors::note_encryption_vanilla::TEST_VECTORS[0].memo;
            let rho = note.rho();

            // Encode.
            let plaintext = OrchardDomain::note_plaintext_bytes(&note, memo);

            // Decode.
            let domain = OrchardDomain::from_rho(rho);
            let (compact, parsed_memo) = domain.split_plaintext_at_memo(&plaintext).unwrap();

            let (parsed_note, parsed_recipient) = parse_note_plaintext_without_memo(
                rho,
                &compact,
                NoteVersion::V2,
                |diversifier| {
                    assert_eq!(diversifier, &note.recipient().diversifier());
                    *note.recipient().pk_d()
                },
            ).expect("Plaintext parsing failed");

            // Check.
            assert_eq!(parsed_note, note);
            assert_eq!(parsed_recipient, note.recipient());
            assert_eq!(&parsed_memo, memo);
        }

        #[test]
        fn encoding_roundtrip_zsa(
            note in arb_note(NoteValue::from_raw(100), NoteVersion::ZSA),
        ) {
            let memo = &crate::test_vectors::note_encryption_zsa::TEST_VECTORS[0].memo;
            let rho = note.rho();

            // Encode.
            let plaintext = ZSADomain::note_plaintext_bytes(&note, memo);

            // Decode.
            let domain = ZSADomain::from_rho(rho);
            let (compact, parsed_memo) = domain.split_plaintext_at_memo(&plaintext).unwrap();

            let (parsed_note, parsed_recipient) = parse_note_plaintext_without_memo(
                rho,
                &compact,
                NoteVersion::ZSA,
                |diversifier| {
                    assert_eq!(diversifier, &note.recipient().diversifier());
                    *note.recipient().pk_d()
                },
            ).expect("Plaintext parsing failed");

            // Check.
            assert_eq!(parsed_note, note);
            assert_eq!(parsed_recipient, note.recipient());
            assert_eq!(&parsed_memo, memo);
        }
    }

    // TODO Constance: cmx has been updated (we now use rcm_v3 instead of rcm_v2)
    // To make the tests pass, the test vectors (lead_byte and rcm) need to be updated.
    /*
    #[test]
    fn test_vectors_zsa() {
        let test_vectors = crate::test_vectors::note_encryption_zsa::TEST_VECTORS;

        for tv in test_vectors {
            //
            // Load the test vector components
            //

            // Recipient key material
            let ivk = PreparedIncomingViewingKey::new(
                &IncomingViewingKey::from_bytes(&tv.incoming_viewing_key).unwrap(),
            );
            let ovk = OutgoingViewingKey::from(tv.ovk);
            let d = Diversifier::from_bytes(tv.default_d);
            let pk_d = DiversifiedTransmissionKey::from_bytes(&tv.default_pk_d).unwrap();

            // Received Action
            let cv_net = ValueCommitment::from_bytes(&tv.cv_net).unwrap();
            let nf_old = Nullifier::from_bytes(&tv.nf_old).unwrap();
            let rho = Rho::from_nf_old(nf_old);
            let cmx = ExtractedNoteCommitment::from_bytes(&tv.cmx).unwrap();

            let esk = EphemeralSecretKey::from_bytes(&tv.esk).unwrap();
            let ephemeral_key = EphemeralKeyBytes(tv.ephemeral_key);

            // Details about the expected note
            let value = NoteValue::from_raw(tv.v);
            let rseed = RandomSeed::from_bytes(tv.rseed, &rho).unwrap();

            //
            // Test the individual components
            //

            let shared_secret = esk.agree(&pk_d);
            assert_eq!(shared_secret.to_bytes(), tv.shared_secret);

            let k_enc = shared_secret.kdf_orchard(&ephemeral_key);
            assert_eq!(k_enc.as_bytes(), tv.k_enc);

            let ock = prf_ock_orchard(&ovk, &cv_net, &cmx.to_bytes(), &ephemeral_key);
            assert_eq!(ock.as_ref(), tv.ock);

            let recipient = Address::from_parts(d, pk_d);

            let asset = AssetBase::from_bytes(&tv.asset).unwrap();

            let note =
                Note::from_parts(recipient, value, asset, rho, rseed, NoteVersion::ZSA).unwrap();

            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);

            let action = Action::from_parts(
                // nf_old is the nullifier revealed by the receiving Action.
                nf_old,
                // We don't need a real rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext {
                    epk_bytes: ephemeral_key.0,
                    enc_ciphertext: NoteCiphertextBytes::Zsa(NoteBytesData(tv.c_enc)),
                    out_ciphertext: tv.c_out,
                },
                cv_net.clone(),
                (),
            ).expect("a key returned by VerificationKey::dummy() is vanishingly unlikely to be the identity");

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            let domain = ZSADomain::from_rho(rho);

            match try_note_decryption(&domain, &ivk, &action) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Note decryption failed"),
            }

            match try_compact_note_decryption(&domain, &ivk, &CompactAction::from(&action)) {
                Some((decrypted_note, decrypted_to)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                }
                None => panic!("Compact note decryption failed"),
            }

            match try_output_recovery_with_ovk(&domain, &ovk, &action, &cv_net, &tv.c_out) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Output recovery failed"),
            }

            //
            // Test encryption
            //

            let ne = ZSANoteEncryption::new_with_esk(esk, Some(ovk), note, tv.memo);

            assert_eq!(ne.encrypt_note_plaintext().as_ref(), &tv.c_enc[..]);
            assert_eq!(
                &ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut OsRng)[..],
                &tv.c_out[..]
            );
        }
    }
    */

    fn v3_encrypted_action() -> (
        Action<()>,
        PreparedIncomingViewingKey,
        Note,
        Address,
        [u8; 512],
    ) {
        let mut rng = OsRng;
        let sk = SpendingKey::random(&mut rng);
        let fvk = crate::keys::FullViewingKey::from(&sk);
        let incoming_viewing_key = fvk.to_ivk(Scope::External);
        let prepared_ivk = PreparedIncomingViewingKey::new(&incoming_viewing_key);
        let recipient = fvk.address_at(0u32, Scope::External);
        let nf_old = Nullifier::dummy(&mut rng);
        let rho = Rho::from_nf_old(nf_old);
        let note = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::zatoshi(),
            rho,
            NoteVersion::V3,
            &mut rng,
        );
        let memo = [7u8; 512];
        let cv_net =
            ValueCommitment::derive(ValueSum::from_raw_inner(5), ValueCommitTrapdoor::zero());
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let encryptor = IronwoodNoteEncryption::new(Some(fvk.to_ovk(Scope::External)), note, memo);
        let encrypted_note = TransmittedNoteCiphertext {
            epk_bytes: IronwoodDomain::epk_bytes(encryptor.epk()).0,
            enc_ciphertext: encryptor.encrypt_note_plaintext(),
            out_ciphertext: encryptor.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut rng),
        };
        let action = Action::from_parts(
            nf_old,
            redpallas::VerificationKey::dummy(),
            cmx,
            encrypted_note,
            cv_net,
            (),
        )
        .expect("a dummy verification key is unlikely to be the identity");

        (action, prepared_ivk, note, recipient, memo)
    }

    #[test]
    fn test_vectors() {
        let test_vectors = crate::test_vectors::note_encryption_vanilla::TEST_VECTORS;

        for tv in test_vectors {
            //
            // Load the test vector components
            //

            // Recipient key material
            let ivk = PreparedIncomingViewingKey::new(
                &IncomingViewingKey::from_bytes(&tv.incoming_viewing_key).unwrap(),
            );
            let ovk = OutgoingViewingKey::from(tv.ovk);
            let d = Diversifier::from_bytes(tv.default_d);
            let pk_d = DiversifiedTransmissionKey::from_bytes(&tv.default_pk_d).unwrap();

            // Received Action
            let cv_net = ValueCommitment::from_bytes(&tv.cv_net).unwrap();
            let nf_old = Nullifier::from_bytes(&tv.nf_old).unwrap();
            let rho = Rho::from_nf_old(nf_old);
            let cmx = ExtractedNoteCommitment::from_bytes(&tv.cmx).unwrap();

            let esk = EphemeralSecretKey::from_bytes(&tv.esk).unwrap();
            let ephemeral_key = EphemeralKeyBytes(tv.ephemeral_key);

            // Details about the expected note
            let value = NoteValue::from_raw(tv.v);
            let rseed = RandomSeed::from_bytes(tv.rseed, &rho).unwrap();

            //
            // Test the individual components
            //

            let shared_secret = esk.agree(&pk_d);
            assert_eq!(shared_secret.to_bytes(), tv.shared_secret);

            let k_enc = shared_secret.kdf_orchard(&ephemeral_key);
            assert_eq!(k_enc.as_bytes(), tv.k_enc);

            let ock = prf_ock_orchard(&ovk, &cv_net, &cmx.to_bytes(), &ephemeral_key);
            assert_eq!(ock.as_ref(), tv.ock);

            let recipient = Address::from_parts(d, pk_d);
            let note_version = NoteVersion::V2;
            let note = Note::from_parts(
                recipient,
                value,
                AssetBase::zatoshi(),
                rho,
                rseed,
                note_version,
            )
            .unwrap();
            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);

            let action = Action::from_parts(
                // nf_old is the nullifier revealed by the receiving Action.
                nf_old,
                // We don't need a real rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext {
                    epk_bytes: ephemeral_key.0,
                    enc_ciphertext: NoteCiphertextBytes::Vanilla(NoteBytesData(tv.c_enc)),
                    out_ciphertext: tv.c_out,
                },
                cv_net.clone(),
                (),
            )
                .expect("a key returned by VerificationKey::dummy() is vanishingly unlikely to be the identity");

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            let domain = OrchardDomain::from_rho(rho);

            match try_note_decryption(&domain, &ivk, &action) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Note decryption failed"),
            }

            match try_compact_note_decryption(&domain, &ivk, &CompactAction::from(&action)) {
                Some((decrypted_note, decrypted_to)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                }
                None => panic!("Compact note decryption failed"),
            }

            match try_output_recovery_with_ovk(&domain, &ovk, &action, &cv_net, &tv.c_out) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Output recovery failed"),
            }

            //
            // Test encryption
            //

            let ne = OrchardNoteEncryption::new_with_esk(esk, Some(ovk), note, tv.memo);

            assert_eq!(ne.encrypt_note_plaintext().as_ref(), &tv.c_enc[..]);
            assert_eq!(
                &ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut OsRng)[..],
                &tv.c_out[..]
            );
        }
    }

    #[test]
    fn domains_accept_only_their_note_plaintext_versions() {
        let mut rng = OsRng;
        let sk = crate::keys::SpendingKey::random(&mut rng);
        let fvk = crate::keys::FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, crate::keys::Scope::External);
        let rho = Rho::from_nf_old(Nullifier::dummy(&mut rng));
        let memo = [0u8; 512];

        let note_v2 = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::zatoshi(),
            rho,
            NoteVersion::V2,
            &mut rng,
        );
        let note_v3 = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::zatoshi(),
            rho,
            NoteVersion::V3,
            &mut rng,
        );
        let orchard_domain = OrchardDomain::from_rho(rho);
        let ironwood_domain = IronwoodDomain::from_rho(rho);

        let np_v2 = OrchardDomain::note_plaintext_bytes(&note_v2, &memo);
        let np_v3 = IronwoodDomain::note_plaintext_bytes(&note_v3, &memo);
        let pk_d = recipient.pk_d();

        let (compact_v2, _) = orchard_domain.split_plaintext_at_memo(&np_v2).unwrap();
        let (compact_v3, _) = ironwood_domain.split_plaintext_at_memo(&np_v3).unwrap();

        assert_eq!(
            orchard_domain
                .parse_note_plaintext_without_memo_ovk(pk_d, &compact_v2)
                .map(|(note, _)| note),
            Some(note_v2)
        );
        assert_eq!(
            ironwood_domain
                .parse_note_plaintext_without_memo_ovk(pk_d, &compact_v3)
                .map(|(note, _)| note),
            Some(note_v3)
        );
        assert!(orchard_domain
            .parse_note_plaintext_without_memo_ovk(pk_d, &compact_v3)
            .is_none());
        assert!(ironwood_domain
            .parse_note_plaintext_without_memo_ovk(pk_d, &compact_v2)
            .is_none());

        // V2 and V3 are both Vanilla-shaped, so the pairs above never cross the Vanilla/ZSA buffer
        // boundary. The lead-byte check runs first, so crossing it needs a buffer whose lead byte
        // matches the domain while its length does not.
        let note_zsa = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::random(&mut rng),
            rho,
            NoteVersion::ZSA,
            &mut rng,
        );
        let zsa_domain = ZSADomain::from_rho(rho);
        let np_zsa = ZSADomain::note_plaintext_bytes(&note_zsa, &memo);
        let (compact_zsa, _) = zsa_domain.split_plaintext_at_memo(&np_zsa).unwrap();

        // ZSA lead byte, Vanilla-sized: the ZSA offsets run past the end (this used to panic).
        let truncated = CompactNotePlaintextBytes::Vanilla(NoteBytesData(
            compact_zsa.as_ref()[..COMPACT_NOTE_SIZE_VANILLA]
                .try_into()
                .unwrap(),
        ));
        assert!(zsa_domain
            .parse_note_plaintext_without_memo_ovk(pk_d, &truncated)
            .is_none());

        // Vanilla lead byte, ZSA-sized: Vanilla offsets would parse fine while ignoring the 32
        // trailing bytes where the asset lives.
        let mut padded = [0u8; COMPACT_NOTE_SIZE_ZSA];
        padded[..COMPACT_NOTE_SIZE_VANILLA].copy_from_slice(compact_v2.as_ref());
        assert!(orchard_domain
            .parse_note_plaintext_without_memo_ovk(
                pk_d,
                &CompactNotePlaintextBytes::Zsa(NoteBytesData(padded))
            )
            .is_none());

        // The matched ZSA pair still parses, so the length check is not rejecting everything.
        assert_eq!(
            zsa_domain
                .parse_note_plaintext_without_memo_ovk(pk_d, &compact_zsa)
                .map(|(note, _)| note),
            Some(note_zsa)
        );
    }

    #[test]
    fn ironwood_domain_decrypts_v3_encrypted_outputs() {
        let (action, ivk, note, recipient, memo) = v3_encrypted_action();
        let domain = IronwoodDomain::for_action(&action);

        assert_eq!(
            try_note_decryption(&domain, &ivk, &action),
            Some((note, recipient, memo))
        );
    }

    #[test]
    fn orchard_domain_rejects_v3_encrypted_outputs() {
        let (action, ivk, _, _, _) = v3_encrypted_action();
        let domain = OrchardDomain::for_action(&action);

        assert!(try_note_decryption(&domain, &ivk, &action).is_none());
    }

    #[test]
    fn ironwood_domain_decrypts_v3_compact_outputs() {
        let (action, ivk, note, recipient, _) = v3_encrypted_action();
        let domain = IronwoodDomain::for_action(&action);
        let compact = CompactAction::from(&action);

        assert_eq!(
            try_compact_note_decryption(&domain, &ivk, &compact),
            Some((note, recipient))
        );
    }

    /// Encrypts a compact output of the domain's note plaintext version to
    /// `recipient`, using a fresh ephemeral key.
    fn encrypted_compact_action<V: DomainVersion>(
        rng: &mut OsRng,
        recipient: Address,
    ) -> CompactAction {
        let nf_old = Nullifier::dummy(rng);
        let rho = Rho::from_nf_old(nf_old);
        let note = Note::new(
            recipient,
            NoteValue::from_raw(42),
            AssetBase::zatoshi(),
            rho,
            V::NOTE_VERSION,
            rng,
        );
        let encryptor = NoteEncryption::<NoteEncryptionDomain<V>>::new(None, note, [0u8; 512]);
        let ephemeral_key = NoteEncryptionDomain::<V>::epk_bytes(encryptor.epk());
        let enc_ciphertext = encryptor.encrypt_note_plaintext();
        CompactAction::from_parts(
            nf_old,
            ExtractedNoteCommitment::from(note.commitment()),
            ephemeral_key,
            CompactNoteCiphertextBytes::from_slice(
                &enc_ciphertext.as_ref()[..compact_note_size(V::NOTE_VERSION)],
            )
            .expect("the compact prefix has the length V::NOTE_VERSION implies"),
        )
    }

    /// The batched trial-decryption pipeline (GLV-window preparation and
    /// per-batch scalar decomposition) must produce exactly the per-item
    /// results, over hits on multiple viewing keys, misses, and an
    /// undecodable ephemeral key.
    fn check_batched_compact_decryption_matches_per_item<V: DomainVersion>() {
        let mut rng = OsRng;

        // Two accounts with external and internal scope each — the wallet
        // shape batched trial decryption runs with — plus a foreign account
        // whose outputs must not decrypt.
        let our_fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let other_fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let foreign_fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let ivks: Vec<PreparedIncomingViewingKey> = [
            (&our_fvk, Scope::External),
            (&our_fvk, Scope::Internal),
            (&other_fvk, Scope::External),
            (&other_fvk, Scope::Internal),
        ]
        .into_iter()
        .map(|(fvk, scope)| PreparedIncomingViewingKey::new(&fvk.to_ivk(scope)))
        .collect();

        let mut actions = vec![
            encrypted_compact_action::<V>(&mut rng, our_fvk.address_at(0u32, Scope::External)),
            encrypted_compact_action::<V>(&mut rng, our_fvk.address_at(0u32, Scope::Internal)),
            encrypted_compact_action::<V>(&mut rng, other_fvk.address_at(0u32, Scope::External)),
        ];
        for i in 0..5u32 {
            actions.push(encrypted_compact_action::<V>(
                &mut rng,
                foreign_fvk.address_at(i, Scope::External),
            ));
        }
        // An ephemeral key that decodes to the identity is rejected during
        // preparation; its lane must pass through as `None`.
        actions.push(CompactAction::from_parts(
            Nullifier::dummy(&mut rng),
            actions[0].cmx(),
            EphemeralKeyBytes([0u8; 32]),
            CompactNoteCiphertextBytes::from_slice(&alloc::vec![
                0u8;
                compact_note_size(V::NOTE_VERSION)
            ])
            .expect("zeroed buffer has the length V::NOTE_VERSION implies"),
        ));

        let items: Vec<(NoteEncryptionDomain<V>, CompactAction)> = actions
            .iter()
            .map(|a| (NoteEncryptionDomain::<V>::for_compact_action(a), a.clone()))
            .collect();
        let batched = batch::try_compact_note_decryption(&ivks, &items);

        let per_item: Vec<Option<((Note, Address), usize)>> = actions
            .iter()
            .map(|a| {
                let domain = NoteEncryptionDomain::<V>::for_compact_action(a);
                ivks.iter().enumerate().find_map(|(i, ivk)| {
                    try_compact_note_decryption(&domain, ivk, a).map(|r| (r, i))
                })
            })
            .collect();

        assert_eq!(batched, per_item);

        // The interesting lanes actually decrypted (guards against both
        // paths failing identically).
        assert_eq!(batched[0].as_ref().map(|(_, i)| *i), Some(0));
        assert_eq!(batched[1].as_ref().map(|(_, i)| *i), Some(1));
        assert_eq!(batched[2].as_ref().map(|(_, i)| *i), Some(2));
        assert!(batched[3..].iter().all(Option::is_none));
    }

    #[test]
    fn batched_compact_decryption_matches_per_item_orchard() {
        check_batched_compact_decryption_matches_per_item::<OrchardVersion>();
    }

    #[test]
    fn batched_compact_decryption_matches_per_item_ironwood() {
        check_batched_compact_decryption_matches_per_item::<IronwoodVersion>();
    }

    #[test]
    fn batched_compact_decryption_matches_per_item_zsa() {
        check_batched_compact_decryption_matches_per_item::<ZSAVersion>();
    }

    /// The batched agreement must produce byte-identical shared secrets to
    /// the per-item path, for both preparation routes (batch-built GLV
    /// windows and individually-built wNAF tables), on hit and miss lanes
    /// alike.
    #[test]
    fn batched_agreement_matches_per_item() {
        let mut rng = OsRng;

        let our_fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let foreign_fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let ivks: Vec<PreparedIncomingViewingKey> = [
            (&our_fvk, Scope::External),
            (&our_fvk, Scope::Internal),
            (&foreign_fvk, Scope::External),
        ]
        .into_iter()
        .map(|(fvk, scope)| PreparedIncomingViewingKey::new(&fvk.to_ivk(scope)))
        .collect();

        // Real ephemeral keys from real encryptions, plus an undecodable lane.
        let mut keys: Vec<EphemeralKeyBytes> = (0..12u32)
            .map(|i| {
                encrypted_compact_action::<OrchardVersion>(
                    &mut rng,
                    our_fvk.address_at(i, Scope::External),
                )
                .ephemeral_key
            })
            .collect();
        keys.push(EphemeralKeyBytes([0u8; 32]));

        let batch_prepared = <OrchardDomain as BatchDomain>::batch_epk(keys.iter().cloned());
        // The undecodable lane passes through preparation as `None`.
        assert!(batch_prepared.last().unwrap().0.is_none());
        assert!(batch_prepared[..12].iter().all(|(p, _)| p.is_some()));

        let wnaf_prepared: Vec<Option<crate::keys::PreparedEphemeralPublicKey>> = keys
            .iter()
            .map(|key| OrchardDomain::epk(key).map(OrchardDomain::prepare_epk))
            .collect();

        for ivk in &ivks {
            let expected: Vec<Option<[u8; 32]>> = keys
                .iter()
                .map(|key| {
                    OrchardDomain::epk(key)
                        .map(OrchardDomain::prepare_epk)
                        .map(|epk| OrchardDomain::ka_agree_dec(ivk, &epk).to_bytes())
                })
                .collect();

            let batched: Vec<Option<[u8; 32]>> =
                <OrchardDomain as BatchDomain>::batch_ka_agree_dec(
                    ivk,
                    batch_prepared.iter().map(|(p, _)| p.as_ref()),
                )
                .into_iter()
                .map(|s| s.map(|s| s.to_bytes()))
                .collect();
            assert_eq!(batched, expected);

            // The batched agreement's fallback arm (individually-prepared
            // inputs) must also match.
            let batched_wnaf: Vec<Option<[u8; 32]>> =
                <OrchardDomain as BatchDomain>::batch_ka_agree_dec(
                    ivk,
                    wnaf_prepared.iter().map(|p| p.as_ref()),
                )
                .into_iter()
                .map(|s| s.map(|s| s.to_bytes()))
                .collect();
            assert_eq!(batched_wnaf, expected);
        }
    }
}
