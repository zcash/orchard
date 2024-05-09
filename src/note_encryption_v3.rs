//! In-band secret distribution for Orchard bundles.

use blake2b_simd::{Hash, Params};
use core::fmt;
use group::ff::PrimeField;
use zcash_note_encryption_zsa::{
    BatchDomain, Domain, EphemeralKeyBytes, OutPlaintextBytes, OutgoingCipherKey, ShieldedOutput,
    AEAD_TAG_SIZE, MEMO_SIZE, OUT_PLAINTEXT_SIZE,
};

use crate::note::AssetBase;
use crate::{
    action::Action,
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        OutgoingViewingKey, PreparedEphemeralPublicKey, PreparedIncomingViewingKey, SharedSecret,
    },
    note::{ExtractedNoteCommitment, Nullifier, RandomSeed, Rho},
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

/// The size of the encoding of a ZSA asset id.
const ZSA_ASSET_SIZE: usize = 32;
/// The size of a v3 compact note.
pub const COMPACT_NOTE_SIZE_V3: usize = COMPACT_NOTE_SIZE_V2 + ZSA_ASSET_SIZE;
/// The size of [`NotePlaintextBytes`] for V3.
pub const NOTE_PLAINTEXT_SIZE_V3: usize = COMPACT_NOTE_SIZE_V3 + MEMO_SIZE;
/// The size of the encrypted ciphertext of the ZSA variant of a note.
pub const ENC_CIPHERTEXT_SIZE_V3: usize = NOTE_PLAINTEXT_SIZE_V3 + AEAD_TAG_SIZE;

/// a type to represent the raw bytes of a note plaintext.
#[derive(Clone, Debug)]
pub struct NotePlaintextBytes(pub [u8; NOTE_PLAINTEXT_SIZE_V3]);

/// a type to represent the raw bytes of an encrypted note plaintext.
#[derive(Clone, Debug)]
pub struct NoteCiphertextBytes(pub [u8; ENC_CIPHERTEXT_SIZE_V3]);

/// a type to represent the raw bytes of a compact note.
#[derive(Clone, Debug)]
pub struct CompactNotePlaintextBytes(pub [u8; COMPACT_NOTE_SIZE_V3]);

/// a type to represent the raw bytes of an encrypted compact note.
#[derive(Clone, Debug)]
pub struct CompactNoteCiphertextBytes(pub [u8; COMPACT_NOTE_SIZE_V3]);

impl AsMut<[u8]> for NotePlaintextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl From<&[u8]> for NotePlaintextBytes {
    fn from(s: &[u8]) -> Self
    where
        Self: Sized,
    {
        NotePlaintextBytes(s.try_into().unwrap())
    }
}

impl AsRef<[u8]> for NoteCiphertextBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsMut<[u8]> for NoteCiphertextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl From<(&[u8], &[u8])> for NoteCiphertextBytes {
    fn from(s: (&[u8], &[u8])) -> Self {
        Self([s.0, s.1].concat().try_into().unwrap())
    }
}

impl From<&[u8]> for NoteCiphertextBytes {
    fn from(s: &[u8]) -> Self
    where
        Self: Sized,
    {
        NoteCiphertextBytes(s.try_into().unwrap())
    }
}

impl AsMut<[u8]> for CompactNotePlaintextBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0.as_mut()
    }
}

impl From<&[u8]> for CompactNotePlaintextBytes {
    fn from(s: &[u8]) -> Self
    where
        Self: Sized,
    {
        CompactNotePlaintextBytes(s.try_into().unwrap())
    }
}

impl AsRef<[u8]> for CompactNoteCiphertextBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

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

/// note_version will return the version of the note plaintext.
pub fn note_version(plaintext: &[u8]) -> Option<u8> {
    match plaintext[0] {
        0x02 => Some(0x02),
        0x03 => Some(0x03),
        _ => None,
    }
}

/// Domain-specific requirements:
/// - If the note version is 3, the `plaintext` must contain a valid encoding of a ZSA asset type.
fn orchard_parse_note_plaintext_without_memo<F>(
    domain: &OrchardDomainV3,
    plaintext: &CompactNotePlaintextBytes,
    get_validated_pk_d: F,
) -> Option<(Note, Address)>
where
    F: FnOnce(&Diversifier) -> Option<DiversifiedTransmissionKey>,
{
    // The unwraps below are guaranteed to succeed by the assertion above
    let diversifier = Diversifier::from_bytes(plaintext.0[1..12].try_into().unwrap());
    let value = NoteValue::from_bytes(plaintext.0[12..20].try_into().unwrap());
    let rseed = Option::from(RandomSeed::from_bytes(
        plaintext.0[20..COMPACT_NOTE_SIZE_V2].try_into().unwrap(),
        &domain.rho,
    ))?;
    let pk_d = get_validated_pk_d(&diversifier)?;
    let recipient = Address::from_parts(diversifier, pk_d);

    let asset = match note_version(plaintext.0.as_ref())? {
        0x02 => AssetBase::native(),
        0x03 => {
            let bytes = plaintext.0[COMPACT_NOTE_SIZE_V2..COMPACT_NOTE_SIZE_V3]
                .try_into()
                .unwrap();
            AssetBase::from_bytes(bytes).unwrap()
        }
        _ => panic!("invalid note version"),
    };

    let note = Option::from(Note::from_parts(recipient, value, asset, domain.rho, rseed))?;
    Some((note, recipient))
}

/// Orchard-specific note encryption logic.
#[derive(Debug)]
pub struct OrchardDomainV3 {
    rho: Rho,
}

impl OrchardDomainV3 {
    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_action<T>(act: &Action<T>) -> Self {
        Self { rho: act.rho() }
    }

    /// Constructs a domain that can be used to trial-decrypt this action's output note.
    pub fn for_compact_action(act: &CompactAction) -> Self {
        Self { rho: act.rho() }
    }
}

impl Domain for OrchardDomainV3 {
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
    type Memo = [u8; MEMO_SIZE];

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
        let mut np = [0u8; NOTE_PLAINTEXT_SIZE_V3];
        np[0] = 0x03;
        np[1..12].copy_from_slice(note.recipient().diversifier().as_array());
        np[12..20].copy_from_slice(&note.value().to_bytes());
        np[20..52].copy_from_slice(note.rseed().as_bytes());
        np[52..84].copy_from_slice(&note.asset().to_bytes());
        np[84..].copy_from_slice(memo);
        NotePlaintextBytes(np)
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
        plaintext: &CompactNotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        orchard_parse_note_plaintext_without_memo(self, plaintext, |_| Some(*pk_d))
    }

    fn extract_memo(
        &self,
        plaintext: &NotePlaintextBytes,
    ) -> (Self::CompactNotePlaintextBytes, Self::Memo) {
        let (compact, memo) = plaintext.0.split_at(COMPACT_NOTE_SIZE_V3);
        (
            CompactNotePlaintextBytes(compact.try_into().unwrap()),
            memo.try_into().unwrap(),
        )
    }

    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<Self::DiversifiedTransmissionKey> {
        DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap()).into()
    }

    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<Self::EphemeralSecretKey> {
        EphemeralSecretKey::from_bytes(out_plaintext.0[32..OUT_PLAINTEXT_SIZE].try_into().unwrap())
            .into()
    }
}

impl BatchDomain for OrchardDomainV3 {
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
pub type OrchardNoteEncryption = zcash_note_encryption_zsa::NoteEncryption<OrchardDomainV3>;

impl<T> ShieldedOutput<OrchardDomainV3> for Action<T> {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(self.encrypted_note().epk_bytes)
    }

    fn cmstar_bytes(&self) -> [u8; 32] {
        self.cmx().to_bytes()
    }

    fn enc_ciphertext(&self) -> Option<NoteCiphertextBytes> {
        Some(NoteCiphertextBytes(self.encrypted_note().enc_ciphertext))
    }

    fn enc_ciphertext_compact(&self) -> CompactNoteCiphertextBytes {
        CompactNoteCiphertextBytes(
            self.encrypted_note().enc_ciphertext[..COMPACT_NOTE_SIZE_V3]
                .try_into()
                .unwrap(),
        )
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

impl<T> From<&Action<T>> for CompactAction
where
    Action<T>: ShieldedOutput<OrchardDomainV3>,
{
    fn from(action: &Action<T>) -> Self {
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: action.ephemeral_key(),
            enc_ciphertext: CompactNoteCiphertextBytes(
                action.encrypted_note().enc_ciphertext[..COMPACT_NOTE_SIZE_V3]
                    .try_into()
                    .unwrap(),
            ),
        }
    }
}

impl ShieldedOutput<OrchardDomainV3> for CompactAction {
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
        self.enc_ciphertext.clone()
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

    /// Obtains the [`Rho`] value that was used to construct the new note being created.
    pub fn rho(&self) -> Rho {
        Rho::from_nf_old(self.nullifier)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rand::rngs::OsRng;
    use zcash_note_encryption_zsa::{
        try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk, Domain,
        EphemeralKeyBytes,
    };

    use super::{
        note_version, orchard_parse_note_plaintext_without_memo, prf_ock_orchard, CompactAction,
        OrchardDomainV3, OrchardNoteEncryption,
    };
    use crate::{
        action::Action,
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, IncomingViewingKey,
            OutgoingViewingKey, PreparedIncomingViewingKey,
        },
        note::{
            testing::arb_note, AssetBase, ExtractedNoteCommitment, Nullifier, RandomSeed, Rho,
            TransmittedNoteCiphertext,
        },
        primitives::redpallas,
        value::{NoteValue, ValueCommitment},
        Address, Note,
    };

    proptest! {
        #[test]
        fn test_encoding_roundtrip(
            note in arb_note(NoteValue::from_raw(100)),
        ) {
            let memo = &crate::test_vectors::note_encryption::test_vectors()[0].memo;

            // Encode.
            let mut plaintext = OrchardDomainV3::note_plaintext_bytes(&note, memo);

            // Decode.
            let domain = OrchardDomainV3 { rho: note.rho() };
            let parsed_version = note_version(plaintext.as_mut()).unwrap();
            let (compact,parsed_memo) = domain.extract_memo(&plaintext);

            let (parsed_note, parsed_recipient) = orchard_parse_note_plaintext_without_memo(&domain, &compact,
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
        let test_vectors = crate::test_vectors::note_encryption_v3::test_vectors();

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
            let nf_old = Nullifier::from_bytes(&tv.rho).unwrap();
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

            let note = Note::from_parts(recipient, value, asset, rho, rseed).unwrap();
            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);

            let action = Action::from_parts(
                // rho is the nullifier in the receiving Action.
                nf_old,
                // We don't need a valid rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext {
                    epk_bytes: ephemeral_key.0,
                    enc_ciphertext: tv.c_enc,
                    out_ciphertext: tv.c_out,
                },
                cv_net.clone(),
                (),
            );

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            let domain = OrchardDomainV3 { rho };

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
}
