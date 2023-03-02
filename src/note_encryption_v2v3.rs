//! In-band secret distribution for Orchard bundles.

use blake2b_simd::{Hash, Params};
use core::fmt;
use group::ff::PrimeField;
use zcash_note_encryption::{
    BatchDomain, Domain, EphemeralKeyBytes, OutPlaintextBytes, OutgoingCipherKey, ShieldedOutput,
    AEAD_TAG_SIZE, MEMO_SIZE, OUT_PLAINTEXT_SIZE,
};

use crate::note::AssetBase;
use crate::{
    action::Action,
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        IncomingViewingKey, OutgoingViewingKey, SharedSecret,
    },
    note::{ExtractedNoteCommitment, Nullifier, RandomSeed},
    spec::diversify_hash,
    value::{NoteValue, ValueCommitment},
    Address, Note,
};

const PRF_OCK_ORCHARD_PERSONALIZATION: &[u8; 16] = b"Zcash_Orchardock";

/// The size of a v2 compact note.
pub const COMPACT_NOTE_SIZE_V2: usize = 1 + // version
    11 + // diversifier
    8  + // value
    32; // rseed (or rcm prior to ZIP 212)
/// The size of [`NotePlaintextBytes`] for V2.
pub const NOTE_PLAINTEXT_SIZE_V2: usize = COMPACT_NOTE_SIZE_V2 + MEMO_SIZE;
/// The size of an encrypted note plaintext.
pub const ENC_CIPHERTEXT_SIZE_V2: usize = NOTE_PLAINTEXT_SIZE_V2 + AEAD_TAG_SIZE;

/// The size of the encoding of a ZSA asset id.
const ZSA_ASSET_SIZE: usize = 32;
/// The size of a v3 compact note.
const COMPACT_NOTE_SIZE_V3: usize = COMPACT_NOTE_SIZE_V2 + ZSA_ASSET_SIZE;
/// The size of [`NotePlaintextBytes`] for V3.
const NOTE_PLAINTEXT_SIZE_V3: usize = COMPACT_NOTE_SIZE_V3 + MEMO_SIZE;
/// The size of the encrypted ciphertext of the ZSA variant of a note.
const ENC_CIPHERTEXT_SIZE_V3: usize = NOTE_PLAINTEXT_SIZE_V3 + AEAD_TAG_SIZE;

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

// TODO: VA: Needs updating
/// Domain-specific requirements:
/// - If the note version is 3, the `plaintext` must contain a valid encoding of a ZSA asset type.
fn orchard_parse_note_plaintext_without_memo<F>(
    domain: &OrchardDomain,
    plaintext: &CompactNotePlaintextBytes,
    get_validated_pk_d: F,
) -> Option<(Note, Address)>
where
    F: FnOnce(&Diversifier) -> Option<DiversifiedTransmissionKey>,
{
    // Check note plaintext version
    // and parse the asset type accordingly.
    let asset = parse_version_and_asset_type(plaintext)?;

    let mut plaintext_inner = [0u8; COMPACT_NOTE_SIZE_V2];
    match plaintext {
        CompactNotePlaintextBytes::V2(x) => {
            plaintext_inner.copy_from_slice(&x[..COMPACT_NOTE_SIZE_V2])
        }
        CompactNotePlaintextBytes::V3(x) => {
            plaintext_inner.copy_from_slice(&x[..COMPACT_NOTE_SIZE_V2])
        }
    }

    // The unwraps below are guaranteed to succeed by the assertion above
    let diversifier = Diversifier::from_bytes(plaintext_inner[1..12].try_into().unwrap());
    let value = NoteValue::from_bytes(plaintext_inner[12..20].try_into().unwrap());
    let rseed = Option::from(RandomSeed::from_bytes(
        plaintext_inner[20..COMPACT_NOTE_SIZE_V2]
            .try_into()
            .unwrap(),
        &domain.rho,
    ))?;

    let pk_d = get_validated_pk_d(&diversifier)?;

    let recipient = Address::from_parts(diversifier, pk_d);
    let note = Option::from(Note::from_parts(recipient, value, asset, domain.rho, rseed))?;
    Some((note, recipient))
}

fn parse_version_and_asset_type(plaintext: &CompactNotePlaintextBytes) -> Option<AssetBase> {
    match plaintext {
        CompactNotePlaintextBytes::V2(x) if x[0] == 0x02 => Some(AssetBase::native()),
        CompactNotePlaintextBytes::V3(x) if x[0] == 0x03 => {
            let bytes = x[COMPACT_NOTE_SIZE_V2..COMPACT_NOTE_SIZE_V3]
                .try_into()
                .unwrap();
            AssetBase::from_bytes(bytes).into()
        }
        _ => None,
    }
}

/// Orchard-specific note encryption logic.
#[derive(Debug)]
pub struct OrchardDomain {
    rho: Nullifier,
}

/// Newtype for encoding the note plaintext post ZSA.
// pub struct NotePlaintextZSA (pub [u8; ZSA_NOTE_PLAINTEXT_SIZE]);
#[derive(Clone, Debug)]
pub enum NotePlaintextBytes {
    /// Variant for old note plaintexts.
    V2([u8; NOTE_PLAINTEXT_SIZE_V2]),
    /// Variant for the new note plaintexts post ZSA.
    V3([u8; NOTE_PLAINTEXT_SIZE_V3]),
}

impl AsMut<[u8]> for NotePlaintextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            NotePlaintextBytes::V2(x) => x.as_mut(),
            NotePlaintextBytes::V3(x) => x.as_mut(),
        }
    }
}

impl From<&[u8]> for NotePlaintextBytes {
    fn from(s: &[u8]) -> Self
    where
        Self: Sized,
    {
        match s.len() {
            NOTE_PLAINTEXT_SIZE_V2 => NotePlaintextBytes::V2(s.try_into().unwrap()),
            NOTE_PLAINTEXT_SIZE_V3 => NotePlaintextBytes::V3(s.try_into().unwrap()),
            _ => panic!("Invalid note plaintext size"),
        }
    }
}

/// Newtype for encoding the encrypted note ciphertext post ZSA.
// pub struct EncNoteCiphertextZSA (pub [u8; ZSA_ENC_CIPHERTEXT_SIZE]);
#[derive(Clone, Debug)]
pub enum NoteCiphertextBytes {
    /// Variant for old encrypted note ciphertexts.
    V2([u8; ENC_CIPHERTEXT_SIZE_V2]),
    /// Variant for new encrypted note ciphertexts post ZSA.
    V3([u8; ENC_CIPHERTEXT_SIZE_V3]),
}

impl AsRef<[u8]> for NoteCiphertextBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            NoteCiphertextBytes::V2(x) => x,
            NoteCiphertextBytes::V3(x) => x,
        }
    }
}

/// Panics if the given slice is not `ENC_CIPHERTEXT_SIZE_V2` or `ENC_CIPHERTEXT_SIZE_V3` bytes long.
impl From<&[u8]> for NoteCiphertextBytes {
    fn from(s: &[u8]) -> Self
    where
        Self: Sized,
    {
        match s.len() {
            ENC_CIPHERTEXT_SIZE_V2 => NoteCiphertextBytes::V2(s.try_into().unwrap()),
            ENC_CIPHERTEXT_SIZE_V3 => NoteCiphertextBytes::V3(s.try_into().unwrap()),
            _ => panic!("Invalid length for compact note plaintext"),
        }
    }
}

/// Newtype for encoding a compact note
#[derive(Clone, Debug)]
pub enum CompactNotePlaintextBytes {
    /// Variant for old compact notes.
    V2([u8; COMPACT_NOTE_SIZE_V2]),
    /// Variant for new compact notes post ZSA.
    V3([u8; COMPACT_NOTE_SIZE_V3]),
}

impl AsMut<[u8]> for CompactNotePlaintextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            CompactNotePlaintextBytes::V2(x) => x,
            CompactNotePlaintextBytes::V3(x) => x,
        }
    }
}

/// Panics if the given slice is not `COMPACT_NOTE_SIZE_V2` or `COMPACT_NOTE_SIZE_V3` bytes long.
impl From<&[u8]> for CompactNotePlaintextBytes {
    fn from(s: &[u8]) -> Self
    where
        Self: Sized,
    {
        match s.len() {
            COMPACT_NOTE_SIZE_V2 => CompactNotePlaintextBytes::V2(s.try_into().unwrap()),
            COMPACT_NOTE_SIZE_V3 => CompactNotePlaintextBytes::V3(s.try_into().unwrap()),
            _ => panic!("Invalid length for compact note plaintext"),
        }
    }
}

/// Newtype for encoding a compact note
#[derive(Clone, Debug)]
pub enum CompactNoteCiphertextBytes {
    /// Variant for old compact notes.
    V2([u8; COMPACT_NOTE_SIZE_V2]),
    /// Variant for new compact notes post ZSA.
    V3([u8; COMPACT_NOTE_SIZE_V3]),
}

impl AsRef<[u8]> for CompactNoteCiphertextBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            CompactNoteCiphertextBytes::V2(x) => x,
            CompactNoteCiphertextBytes::V3(x) => x,
        }
    }
}

impl OrchardDomain {
    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_action<T>(act: &Action<T>) -> Self {
        OrchardDomain {
            rho: *act.nullifier(),
        }
    }

    /// Constructs a domain from a nullifier.
    pub fn for_nullifier(nullifier: Nullifier) -> Self {
        OrchardDomain { rho: nullifier }
    }
}

impl Domain for OrchardDomain {
    type EphemeralSecretKey = EphemeralSecretKey;
    type EphemeralPublicKey = EphemeralPublicKey;
    type PreparedEphemeralPublicKey = EphemeralPublicKey;
    type SharedSecret = SharedSecret;
    type SymmetricKey = Hash;
    type Note = Note;
    type Recipient = Address;
    type DiversifiedTransmissionKey = DiversifiedTransmissionKey;
    type IncomingViewingKey = IncomingViewingKey;
    type OutgoingViewingKey = OutgoingViewingKey;
    type ValueCommitment = ValueCommitment;
    type ExtractedCommitment = ExtractedNoteCommitment;
    type ExtractedCommitmentBytes = [u8; 32];
    type Memo = [u8; MEMO_SIZE]; // TODO use a more interesting type

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
        epk
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

    fn note_plaintext_bytes(
        note: &Self::Note,
        _: &Self::Recipient,
        memo: &Self::Memo,
    ) -> NotePlaintextBytes {
        let mut np = [0u8; NOTE_PLAINTEXT_SIZE_V3];
        np[0] = 0x03;
        np[1..12].copy_from_slice(note.recipient().diversifier().as_array());
        np[12..20].copy_from_slice(&note.value().to_bytes());
        np[20..52].copy_from_slice(note.rseed().as_bytes());
        let zsa_type = note.asset().to_bytes();
        np[52..84].copy_from_slice(&zsa_type);
        np[84..].copy_from_slice(memo);
        NotePlaintextBytes::V3(np)
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
        orchard_parse_note_plaintext_without_memo(self, plaintext, |diversifier| {
            Some(DiversifiedTransmissionKey::derive(ivk, diversifier))
        })
    }

    fn parse_note_plaintext_without_memo_ovk(
        &self,
        pk_d: &Self::DiversifiedTransmissionKey,
        esk: &Self::EphemeralSecretKey,
        ephemeral_key: &EphemeralKeyBytes,
        plaintext: &CompactNotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        orchard_parse_note_plaintext_without_memo(self, plaintext, |diversifier| {
            if esk
                .derive_public(diversify_hash(diversifier.as_array()))
                .to_bytes()
                .0
                == ephemeral_key.0
            {
                Some(*pk_d)
            } else {
                None
            }
        })
    }

    fn extract_memo(
        &self,
        plaintext: &NotePlaintextBytes,
    ) -> (Self::CompactNotePlaintextBytes, Self::Memo) {
        match plaintext {
            NotePlaintextBytes::V2(np) => {
                let (compact, memo) = np.split_at(COMPACT_NOTE_SIZE_V2);
                (
                    CompactNotePlaintextBytes::V2(compact.try_into().unwrap()),
                    memo.try_into().unwrap(),
                )
            }
            NotePlaintextBytes::V3(np) => {
                let (compact, memo) = np.split_at(COMPACT_NOTE_SIZE_V3);
                (
                    CompactNotePlaintextBytes::V3(compact.try_into().unwrap()),
                    memo.try_into().unwrap(),
                )
            }
        }
    }

    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<Self::DiversifiedTransmissionKey> {
        DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap()).into()
    }

    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<Self::EphemeralSecretKey> {
        EphemeralSecretKey::from_bytes(out_plaintext.0[32..OUT_PLAINTEXT_SIZE].try_into().unwrap())
            .into()
    }
}

impl BatchDomain for OrchardDomain {
    fn batch_kdf<'a>(
        items: impl Iterator<Item = (Option<Self::SharedSecret>, &'a EphemeralKeyBytes)>,
    ) -> Vec<Option<Self::SymmetricKey>> {
        let (shared_secrets, ephemeral_keys): (Vec<_>, Vec<_>) = items.unzip();

        SharedSecret::batch_to_affine(shared_secrets)
            .zip(ephemeral_keys.into_iter())
            .map(|(secret, ephemeral_key)| {
                secret.map(|dhsecret| SharedSecret::kdf_orchard_inner(dhsecret, ephemeral_key))
            })
            .collect()
    }
}


/// Implementation of in-band secret distribution for Orchard bundles.
pub type OrchardNoteEncryption = zcash_note_encryption::NoteEncryption<OrchardDomain>;

impl<T> ShieldedOutput<OrchardDomain> for Action<T> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.encrypted_note().epk_bytes)
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx().to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<NoteCiphertextBytes> {
        let result = self.encrypted_note().enc_ciphertext.clone();
        Some(result)
    }

    fn enc_ciphertext_compact(&self) -> CompactNoteCiphertextBytes {
        match self.encrypted_note().enc_ciphertext {
            NoteCiphertextBytes::V2(ncx) => {
                CompactNoteCiphertextBytes::V2(ncx[..COMPACT_NOTE_SIZE_V2].try_into().unwrap())
            }
            NoteCiphertextBytes::V3(ncx) => {
                CompactNoteCiphertextBytes::V3(ncx[..COMPACT_NOTE_SIZE_V3].try_into().unwrap())
            }
        }
    }
}

/// A compact Action for light clients.
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

impl<T> From<&Action<T>> for CompactAction {
    fn from(action: &Action<T>) -> Self {
        let comp_ciphertext: CompactNoteCiphertextBytes =
            match action.encrypted_note().enc_ciphertext {
                NoteCiphertextBytes::V2(ncx) => {
                    CompactNoteCiphertextBytes::V2(ncx[..COMPACT_NOTE_SIZE_V2].try_into().unwrap())
                }
                NoteCiphertextBytes::V3(ncx) => {
                    CompactNoteCiphertextBytes::V3(ncx[..COMPACT_NOTE_SIZE_V3].try_into().unwrap())
                }
            };
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: action.ephemeral_key(),
            enc_ciphertext: comp_ciphertext,
        }
    }
}

impl ShieldedOutput<OrchardDomain> for CompactAction {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.ephemeral_key.0)
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx.to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<NoteCiphertextBytes> {
        None
    }

    fn enc_ciphertext_compact(&self) -> CompactNoteCiphertextBytes {
        &self.enc_ciphertext
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

    ///Returns the nullifier of the note being spent.
    pub fn nullifier(&self) -> Nullifier {
        self.nullifier
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rand::rngs::OsRng;
    use zcash_note_encryption::{
        try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk, Domain,
        EphemeralKeyBytes,
    };

    use super::{prf_ock_orchard, CompactAction, OrchardDomain, OrchardNoteEncryption};
    use crate::note::AssetBase;
    use crate::note_encryption::NoteCiphertextBytes;
    use crate::{
        action::Action,
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, IncomingViewingKey,
            OutgoingViewingKey,
        },
        note::{
            testing::arb_note, ExtractedNoteCommitment, Nullifier, RandomSeed,
            TransmittedNoteCiphertext,
        },
        primitives::redpallas,
        value::{NoteValue, ValueCommitment},
        Address, Note,
    };

    use super::{version, orchard_parse_note_plaintext_without_memo};

    proptest! {
        #[test]
        fn test_encoding_roundtrip(
            note in arb_note(NoteValue::from_raw(100)),
        ) {
            let memo = &crate::test_vectors::note_encryption::test_vectors()[0].memo;

            // Encode.
            let mut plaintext = OrchardDomain::note_plaintext_bytes(&note, &note.recipient(), memo);

            // Decode.
            let domain = OrchardDomain { rho: note.rho() };
            let parsed_version = version(plaintext.as_mut()).unwrap();
            let (mut compact,parsed_memo) = domain.extract_memo(&plaintext);

            let (parsed_note, parsed_recipient) = orchard_parse_note_plaintext_without_memo(&domain, &compact.as_mut(),
                |diversifier| {
                    assert_eq!(diversifier, &note.recipient().diversifier());
                    Some(*note.recipient().pk_d())
                }
            ).expect("Plaintext parsing failed");

            // Check.
            assert_eq!(parsed_note, note);
            assert_eq!(parsed_recipient, note.recipient());
            assert_eq!(&parsed_memo, memo);
            assert_eq!(parsed_version, 0x03);
        }
    }

    #[test]
    fn test_vectors() {
        let test_vectors = crate::test_vectors::note_encryption::test_vectors();

        for tv in test_vectors {
            //
            // Load the test vector components
            //

            // Recipient key material
            let ivk = IncomingViewingKey::from_bytes(&tv.incoming_viewing_key).unwrap();
            let ovk = OutgoingViewingKey::from(tv.ovk);
            let d = Diversifier::from_bytes(tv.default_d);
            let pk_d = DiversifiedTransmissionKey::from_bytes(&tv.default_pk_d).unwrap();

            // Received Action
            let cv_net = ValueCommitment::from_bytes(&tv.cv_net).unwrap();
            let rho = Nullifier::from_bytes(&tv.rho).unwrap();
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

            let asset = match tv.asset {
                None => AssetBase::native(),
                Some(type_bytes) => AssetBase::from_bytes(&type_bytes).unwrap(),
            };

            let note = Note::from_parts(recipient, value, asset, rho, rseed).unwrap();
            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);

            let action = Action::from_parts(
                // rho is the nullifier in the receiving Action.
                rho,
                // We don't need a valid rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext {
                    epk_bytes: ephemeral_key.0,
                    enc_ciphertext: NoteCiphertextBytes::V3(tv.c_enc), // TODO: VA: Would need a mix of V2 and V3 eventually
                    out_ciphertext: tv.c_out,
                },
                cv_net.clone(),
                (),
            );

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            let domain = OrchardDomain { rho };

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

            let ne = OrchardNoteEncryption::new_with_esk(esk, Some(ovk), note, recipient, tv.memo);

            // assert_eq!(ne.encrypt_note_plaintext().as_ref(), &tv.c_enc[..]);
            assert_eq!(
                &ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut OsRng)[..],
                &tv.c_out[..]
            );
        }
    }
}
