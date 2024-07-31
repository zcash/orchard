//! This module implements `Domain` and `BatchDomain` traits from the `zcash_note_encryption`
//! crate and contains the common logic for `OrchardVanilla` and `OrchardZSA` flavors.

use blake2b_simd::Hash;
use group::ff::PrimeField;

use blake2b_simd::Params;

use zcash_note_encryption_zsa::{
    note_bytes::NoteBytes, BatchDomain, Domain, EphemeralKeyBytes, OutPlaintextBytes,
    OutgoingCipherKey, MEMO_SIZE, OUT_PLAINTEXT_SIZE,
};

use crate::{
    address::Address,
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        OutgoingViewingKey, PreparedEphemeralPublicKey, PreparedIncomingViewingKey, SharedSecret,
    },
    note::{ExtractedNoteCommitment, Note, RandomSeed, Rho},
    value::{NoteValue, ValueCommitment},
};

use super::orchard_domain::{OrchardDomain, OrchardDomainCommon};

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
pub(super) const COMPACT_NOTE_SIZE_VANILLA: usize =
    NOTE_VERSION_SIZE + NOTE_DIVERSIFIER_SIZE + NOTE_VALUE_SIZE + NOTE_RSEED_SIZE;

/// The size of the encoding of a ZSA asset id.
const ZSA_ASSET_SIZE: usize = 32;

/// The size of a ZSA compact note.
pub(super) const COMPACT_NOTE_SIZE_ZSA: usize = COMPACT_NOTE_SIZE_VANILLA + ZSA_ASSET_SIZE;

/// The version byte for Vanilla.
pub(super) const NOTE_VERSION_BYTE_V2: u8 = 0x02;

/// The version byte for ZSA.
pub(super) const NOTE_VERSION_BYTE_V3: u8 = 0x03;

pub(super) type Memo = [u8; MEMO_SIZE];

/// Defined in [Zcash Protocol Spec § 5.4.2: Pseudo Random Functions][concreteprfs].
///
/// [concreteprfs]: https://zips.z.cash/protocol/nu5.pdf#concreteprfs
pub(super) fn prf_ock_orchard(
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

/// Retrieves the version of the note plaintext.
/// Returns `Some(u8)` if the version is recognized, otherwise `None`.
pub(super) fn parse_note_version(plaintext: &[u8]) -> Option<u8> {
    plaintext.first().and_then(|version| match *version {
        NOTE_VERSION_BYTE_V2 | NOTE_VERSION_BYTE_V3 => Some(*version),
        _ => None,
    })
}

/// Parses the note plaintext (excluding the memo) and extracts the note and address if valid.
/// Domain-specific requirements:
/// - If the note version is 3, the `plaintext` must contain a valid encoding of a ZSA asset type.
pub(super) fn parse_note_plaintext_without_memo<D: OrchardDomainCommon, F>(
    rho: Rho,
    plaintext: &D::CompactNotePlaintextBytes,
    get_validated_pk_d: F,
) -> Option<(Note, Address)>
where
    F: FnOnce(&Diversifier) -> Option<DiversifiedTransmissionKey>,
{
    parse_note_version(plaintext.as_ref())?;

    // The unwraps below are guaranteed to succeed
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

    let pk_d = get_validated_pk_d(&diversifier)?;
    let recipient = Address::from_parts(diversifier, pk_d);
    let asset = D::extract_asset(plaintext)?;
    let note = Option::from(Note::from_parts(recipient, value, asset, rho, rseed))?;

    Some((note, recipient))
}

// Constructs a note plaintext bytes array given note information.
pub(super) fn build_base_note_plaintext_bytes<const NOTE_PLAINTEXT_SIZE: usize>(
    version: u8,
    note: &Note,
) -> [u8; NOTE_PLAINTEXT_SIZE] {
    let mut np = [0; NOTE_PLAINTEXT_SIZE];

    np[NOTE_VERSION_OFFSET] = version;
    np[NOTE_DIVERSIFIER_OFFSET..NOTE_VALUE_OFFSET]
        .copy_from_slice(note.recipient().diversifier().as_array());
    np[NOTE_VALUE_OFFSET..NOTE_RSEED_OFFSET].copy_from_slice(&note.value().to_bytes());
    np[NOTE_RSEED_OFFSET..COMPACT_NOTE_SIZE_VANILLA].copy_from_slice(note.rseed().as_bytes());

    np
}

impl<D: OrchardDomainCommon> Domain for OrchardDomain<D> {
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

    type NotePlaintextBytes = D::NotePlaintextBytes;
    type NoteCiphertextBytes = D::NoteCiphertextBytes;
    type CompactNotePlaintextBytes = D::CompactNotePlaintextBytes;
    type CompactNoteCiphertextBytes = D::CompactNoteCiphertextBytes;

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

    fn note_plaintext_bytes(note: &Self::Note, memo: &Self::Memo) -> D::NotePlaintextBytes {
        D::build_note_plaintext_bytes(note, memo)
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
        plaintext: &D::CompactNotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        parse_note_plaintext_without_memo::<D, _>(self.rho, plaintext, |diversifier| {
            Some(DiversifiedTransmissionKey::derive(ivk, diversifier))
        })
    }

    fn parse_note_plaintext_without_memo_ovk(
        &self,
        pk_d: &Self::DiversifiedTransmissionKey,
        plaintext: &D::CompactNotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        parse_note_plaintext_without_memo::<D, _>(self.rho, plaintext, |_| Some(*pk_d))
    }

    fn split_plaintext_at_memo(
        &self,
        plaintext: &D::NotePlaintextBytes,
    ) -> Option<(Self::CompactNotePlaintextBytes, Self::Memo)> {
        let (compact, memo) = plaintext.as_ref().split_at(D::COMPACT_NOTE_SIZE);
        Some((
            Self::CompactNotePlaintextBytes::from_slice(compact)?,
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

impl<D: OrchardDomainCommon> BatchDomain for OrchardDomain<D> {
    fn batch_kdf<'a>(
        items: impl Iterator<Item = (Option<Self::SharedSecret>, &'a EphemeralKeyBytes)>,
    ) -> Vec<Option<Self::SymmetricKey>> {
        let (shared_secrets, ephemeral_keys): (Vec<_>, Vec<_>) = items.unzip();

        SharedSecret::batch_to_affine(shared_secrets)
            .zip(ephemeral_keys)
            .map(|(secret, ephemeral_key)| {
                secret.map(|dhsecret| SharedSecret::kdf_orchard_inner(dhsecret, ephemeral_key))
            })
            .collect()
    }
}
