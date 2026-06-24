//! In-band secret distribution for Orchard bundles.

use alloc::vec::Vec;
use core::fmt;

use blake2b_simd::{Hash, Params};
use group::ff::PrimeField;
use zcash_note_encryption::{
    BatchDomain, Domain, EphemeralKeyBytes, NotePlaintextBytes, OutPlaintextBytes,
    OutgoingCipherKey, ShieldedOutput, COMPACT_NOTE_SIZE, ENC_CIPHERTEXT_SIZE, NOTE_PLAINTEXT_SIZE,
    OUT_PLAINTEXT_SIZE,
};

use crate::{
    action::Action,
    bundle::BundlePoolRestrictions,
    keys::{
        DiversifiedTransmissionKey, Diversifier, EphemeralPublicKey, EphemeralSecretKey,
        OutgoingViewingKey, PreparedEphemeralPublicKey, PreparedIncomingViewingKey, SharedSecret,
    },
    note::{ExtractedNoteCommitment, NoteVersion, Nullifier, RandomSeed, Rho},
    value::{NoteValue, ValueCommitment},
    Address, Note,
};

const PRF_OCK_ORCHARD_PERSONALIZATION: &[u8; 16] = b"Zcash_Orchardock";

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
    domain: &SharedDomain,
    plaintext: &[u8],
    note_version: NoteVersion,
    get_pk_d: F,
) -> Option<(Note, Address)>
where
    F: FnOnce(&Diversifier) -> DiversifiedTransmissionKey,
{
    assert!(plaintext.len() >= COMPACT_NOTE_SIZE);

    // Check note plaintext version.
    if plaintext[0] != note_version.lead_byte() {
        return None;
    }

    // The unwraps below are guaranteed to succeed by the assertion above
    let diversifier = Diversifier::from_bytes(plaintext[1..12].try_into().unwrap());
    let value = NoteValue::from_bytes(plaintext[12..20].try_into().unwrap());
    let rseed = Option::from(RandomSeed::from_bytes(
        plaintext[20..COMPACT_NOTE_SIZE].try_into().unwrap(),
        &domain.rho,
    ))?;

    let pk_d = get_pk_d(&diversifier);

    let recipient = Address::from_parts(diversifier, pk_d);
    let note = Option::from(Note::from_parts(
        recipient,
        value,
        domain.rho,
        rseed,
        note_version,
    ))?;
    Some((note, recipient))
}

// A private, common domain for both Orchard and Ironwood.
// The two implementations differ in just choice of NoteVersion.
#[derive(Debug)]
struct SharedDomain {
    rho: Rho,
}

impl memuse::DynamicUsage for SharedDomain {
    fn dynamic_usage(&self) -> usize {
        self.rho.dynamic_usage()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        self.rho.dynamic_usage_bounds()
    }
}

impl SharedDomain {
    fn for_action<T>(act: &Action<T>) -> Self {
        Self { rho: act.rho() }
    }

    fn for_pczt_action(act: &crate::pczt::Action) -> Self {
        Self {
            rho: Rho::from_nf_old(act.spend().nullifier),
        }
    }

    fn for_compact_action(act: &CompactAction) -> Self {
        Self { rho: act.rho() }
    }

    fn derive_esk(note: &Note) -> Option<EphemeralSecretKey> {
        Some(note.esk())
    }

    fn get_pk_d(note: &Note) -> DiversifiedTransmissionKey {
        *note.recipient().pk_d()
    }

    fn prepare_epk(epk: EphemeralPublicKey) -> PreparedEphemeralPublicKey {
        PreparedEphemeralPublicKey::new(epk)
    }

    fn ka_derive_public(note: &Note, esk: &EphemeralSecretKey) -> EphemeralPublicKey {
        esk.derive_public(note.recipient().g_d())
    }

    fn ka_agree_enc(esk: &EphemeralSecretKey, pk_d: &DiversifiedTransmissionKey) -> SharedSecret {
        esk.agree(pk_d)
    }

    fn ka_agree_dec(
        ivk: &PreparedIncomingViewingKey,
        epk: &PreparedEphemeralPublicKey,
    ) -> SharedSecret {
        epk.agree(ivk)
    }

    fn kdf(secret: SharedSecret, ephemeral_key: &EphemeralKeyBytes) -> Hash {
        secret.kdf_orchard(ephemeral_key)
    }

    fn note_plaintext_bytes(note: &Note, memo: &[u8; 512]) -> NotePlaintextBytes {
        let mut np = [0; NOTE_PLAINTEXT_SIZE];
        np[0] = note.version().lead_byte();
        np[1..12].copy_from_slice(note.recipient().diversifier().as_array());
        np[12..20].copy_from_slice(&note.value().to_bytes());
        np[20..52].copy_from_slice(note.rseed().as_bytes());
        np[52..].copy_from_slice(memo);
        NotePlaintextBytes(np)
    }

    fn derive_ock(
        ovk: &OutgoingViewingKey,
        cv: &ValueCommitment,
        cmstar_bytes: &[u8; 32],
        ephemeral_key: &EphemeralKeyBytes,
    ) -> OutgoingCipherKey {
        prf_ock_orchard(ovk, cv, cmstar_bytes, ephemeral_key)
    }

    fn outgoing_plaintext_bytes(note: &Note, esk: &EphemeralSecretKey) -> OutPlaintextBytes {
        let mut op = [0; OUT_PLAINTEXT_SIZE];
        op[..32].copy_from_slice(&note.recipient().pk_d().to_bytes());
        op[32..].copy_from_slice(&esk.0.to_repr());
        OutPlaintextBytes(op)
    }

    fn epk_bytes(epk: &EphemeralPublicKey) -> EphemeralKeyBytes {
        epk.to_bytes()
    }

    fn epk(ephemeral_key: &EphemeralKeyBytes) -> Option<EphemeralPublicKey> {
        EphemeralPublicKey::from_bytes(&ephemeral_key.0).into()
    }

    fn cmstar(note: &Note) -> ExtractedNoteCommitment {
        note.commitment().into()
    }

    fn parse_note_plaintext_without_memo_ivk<F>(
        &self,
        ivk: &PreparedIncomingViewingKey,
        plaintext: &[u8],
        note_version: NoteVersion,
        get_pk_d: F,
    ) -> Option<(Note, Address)>
    where
        F: FnOnce(&PreparedIncomingViewingKey, &Diversifier) -> DiversifiedTransmissionKey,
    {
        parse_note_plaintext_without_memo(self, plaintext, note_version, |diversifier| {
            get_pk_d(ivk, diversifier)
        })
    }

    fn parse_note_plaintext_without_memo_ovk(
        &self,
        pk_d: &DiversifiedTransmissionKey,
        plaintext: &NotePlaintextBytes,
        note_version: NoteVersion,
    ) -> Option<(Note, Address)> {
        parse_note_plaintext_without_memo(self, &plaintext.0, note_version, |_| *pk_d)
    }

    fn extract_memo(&self, plaintext: &NotePlaintextBytes) -> [u8; 512] {
        plaintext.0[COMPACT_NOTE_SIZE..NOTE_PLAINTEXT_SIZE]
            .try_into()
            .unwrap()
    }

    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<DiversifiedTransmissionKey> {
        DiversifiedTransmissionKey::from_bytes(out_plaintext.0[0..32].try_into().unwrap()).into()
    }

    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<EphemeralSecretKey> {
        EphemeralSecretKey::from_bytes(out_plaintext.0[32..OUT_PLAINTEXT_SIZE].try_into().unwrap())
            .into()
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

macro_rules! impl_domain {
    ($domain:ty, $note_version:expr) => {
        impl memuse::DynamicUsage for $domain {
            fn dynamic_usage(&self) -> usize {
                self.shared.dynamic_usage()
            }

            fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
                self.shared.dynamic_usage_bounds()
            }
        }

        impl $domain {
            /// Constructs a domain that can be used to trial-decrypt this
            /// action's output note.
            pub fn for_action<T>(act: &Action<T>) -> Self {
                Self {
                    shared: SharedDomain::for_action(act),
                }
            }

            /// Constructs a domain that can be used to trial-decrypt a PCZT
            /// action's output note.
            pub fn for_pczt_action(act: &crate::pczt::Action) -> Self {
                Self {
                    shared: SharedDomain::for_pczt_action(act),
                }
            }

            /// Constructs a domain that can be used to trial-decrypt this
            /// action's output note.
            pub fn for_compact_action(act: &CompactAction) -> Self {
                Self {
                    shared: SharedDomain::for_compact_action(act),
                }
            }
        }

        impl Domain for $domain {
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
            type Memo = [u8; 512]; // TODO use a more interesting type

            fn derive_esk(note: &Self::Note) -> Option<Self::EphemeralSecretKey> {
                SharedDomain::derive_esk(note)
            }

            fn get_pk_d(note: &Self::Note) -> Self::DiversifiedTransmissionKey {
                SharedDomain::get_pk_d(note)
            }

            fn prepare_epk(epk: Self::EphemeralPublicKey) -> Self::PreparedEphemeralPublicKey {
                SharedDomain::prepare_epk(epk)
            }

            fn ka_derive_public(
                note: &Self::Note,
                esk: &Self::EphemeralSecretKey,
            ) -> Self::EphemeralPublicKey {
                SharedDomain::ka_derive_public(note, esk)
            }

            fn ka_agree_enc(
                esk: &Self::EphemeralSecretKey,
                pk_d: &Self::DiversifiedTransmissionKey,
            ) -> Self::SharedSecret {
                SharedDomain::ka_agree_enc(esk, pk_d)
            }

            fn ka_agree_dec(
                ivk: &Self::IncomingViewingKey,
                epk: &Self::PreparedEphemeralPublicKey,
            ) -> Self::SharedSecret {
                SharedDomain::ka_agree_dec(ivk, epk)
            }

            fn kdf(
                secret: Self::SharedSecret,
                ephemeral_key: &EphemeralKeyBytes,
            ) -> Self::SymmetricKey {
                SharedDomain::kdf(secret, ephemeral_key)
            }

            fn note_plaintext_bytes(note: &Self::Note, memo: &Self::Memo) -> NotePlaintextBytes {
                SharedDomain::note_plaintext_bytes(note, memo)
            }

            fn derive_ock(
                ovk: &Self::OutgoingViewingKey,
                cv: &Self::ValueCommitment,
                cmstar_bytes: &Self::ExtractedCommitmentBytes,
                ephemeral_key: &EphemeralKeyBytes,
            ) -> OutgoingCipherKey {
                SharedDomain::derive_ock(ovk, cv, cmstar_bytes, ephemeral_key)
            }

            fn outgoing_plaintext_bytes(
                note: &Self::Note,
                esk: &Self::EphemeralSecretKey,
            ) -> OutPlaintextBytes {
                SharedDomain::outgoing_plaintext_bytes(note, esk)
            }

            fn epk_bytes(epk: &Self::EphemeralPublicKey) -> EphemeralKeyBytes {
                SharedDomain::epk_bytes(epk)
            }

            fn epk(ephemeral_key: &EphemeralKeyBytes) -> Option<Self::EphemeralPublicKey> {
                SharedDomain::epk(ephemeral_key)
            }

            fn cmstar(note: &Self::Note) -> Self::ExtractedCommitment {
                SharedDomain::cmstar(note)
            }

            fn parse_note_plaintext_without_memo_ivk(
                &self,
                ivk: &Self::IncomingViewingKey,
                plaintext: &[u8],
            ) -> Option<(Self::Note, Self::Recipient)> {
                self.shared.parse_note_plaintext_without_memo_ivk(
                    ivk,
                    plaintext,
                    $note_version,
                    DiversifiedTransmissionKey::derive,
                )
            }

            fn parse_note_plaintext_without_memo_ovk(
                &self,
                pk_d: &Self::DiversifiedTransmissionKey,
                plaintext: &NotePlaintextBytes,
            ) -> Option<(Self::Note, Self::Recipient)> {
                self.shared
                    .parse_note_plaintext_without_memo_ovk(pk_d, plaintext, $note_version)
            }

            fn extract_memo(&self, plaintext: &NotePlaintextBytes) -> Self::Memo {
                self.shared.extract_memo(plaintext)
            }

            fn extract_pk_d(
                out_plaintext: &OutPlaintextBytes,
            ) -> Option<Self::DiversifiedTransmissionKey> {
                SharedDomain::extract_pk_d(out_plaintext)
            }

            fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<Self::EphemeralSecretKey> {
                SharedDomain::extract_esk(out_plaintext)
            }
        }

        impl BatchDomain for $domain {
            fn batch_kdf<'a>(
                items: impl Iterator<Item = (Option<Self::SharedSecret>, &'a EphemeralKeyBytes)>,
            ) -> Vec<Option<Self::SymmetricKey>> {
                batch_kdf(items)
            }
        }
    };
}

/// Orchard-specific note encryption logic.
///
/// This domain accepts only [`NoteVersion::V2`] note plaintexts, which use lead
/// byte `0x02`.
#[derive(Debug)]
pub struct OrchardDomain {
    shared: SharedDomain,
}

/// Ironwood-specific note encryption logic.
///
/// This domain is otherwise identical to [`OrchardDomain`], but accepts only
/// [`NoteVersion::V3`] note plaintexts, which use lead byte `0x03`.
#[derive(Debug)]
pub struct IronwoodDomain {
    shared: SharedDomain,
}

impl_domain!(OrchardDomain, NoteVersion::V2);
impl_domain!(IronwoodDomain, NoteVersion::V3);

/// Bundle-pool-restricted note encryption logic.
///
/// This domain is used by public bundle helpers that are given the target
/// [`BundlePoolRestrictions`]. Trial decryption still happens once; after
/// decryption succeeds, the revealed note plaintext lead byte selects the note
/// version and the bundle pool restrictions are enforced against it.
#[derive(Debug)]
pub(crate) struct BundleDomain {
    shared: SharedDomain,
    pool_restrictions: BundlePoolRestrictions,
}

impl memuse::DynamicUsage for BundleDomain {
    fn dynamic_usage(&self) -> usize {
        self.shared.dynamic_usage()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        self.shared.dynamic_usage_bounds()
    }
}

impl BundleDomain {
    /// Constructs a domain that can be used to trial-decrypt this action's
    /// output note under `pool_restrictions`.
    pub(crate) fn for_action<T>(
        act: &Action<T>,
        pool_restrictions: BundlePoolRestrictions,
    ) -> Self {
        Self {
            shared: SharedDomain::for_action(act),
            pool_restrictions,
        }
    }

    fn parse_note_version(&self, plaintext: &[u8]) -> Option<NoteVersion> {
        let note_version = NoteVersion::from_lead_byte(*plaintext.first()?)?;
        if note_version == self.pool_restrictions.note_version() {
            Some(note_version)
        } else {
            None
        }
    }
}

impl Domain for BundleDomain {
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
    type Memo = [u8; 512]; // TODO use a more interesting type

    fn derive_esk(note: &Self::Note) -> Option<Self::EphemeralSecretKey> {
        SharedDomain::derive_esk(note)
    }

    fn get_pk_d(note: &Self::Note) -> Self::DiversifiedTransmissionKey {
        SharedDomain::get_pk_d(note)
    }

    fn prepare_epk(epk: Self::EphemeralPublicKey) -> Self::PreparedEphemeralPublicKey {
        SharedDomain::prepare_epk(epk)
    }

    fn ka_derive_public(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> Self::EphemeralPublicKey {
        SharedDomain::ka_derive_public(note, esk)
    }

    fn ka_agree_enc(
        esk: &Self::EphemeralSecretKey,
        pk_d: &Self::DiversifiedTransmissionKey,
    ) -> Self::SharedSecret {
        SharedDomain::ka_agree_enc(esk, pk_d)
    }

    fn ka_agree_dec(
        ivk: &Self::IncomingViewingKey,
        epk: &Self::PreparedEphemeralPublicKey,
    ) -> Self::SharedSecret {
        SharedDomain::ka_agree_dec(ivk, epk)
    }

    fn kdf(secret: Self::SharedSecret, ephemeral_key: &EphemeralKeyBytes) -> Self::SymmetricKey {
        SharedDomain::kdf(secret, ephemeral_key)
    }

    fn note_plaintext_bytes(note: &Self::Note, memo: &Self::Memo) -> NotePlaintextBytes {
        SharedDomain::note_plaintext_bytes(note, memo)
    }

    fn derive_ock(
        ovk: &Self::OutgoingViewingKey,
        cv: &Self::ValueCommitment,
        cmstar_bytes: &Self::ExtractedCommitmentBytes,
        ephemeral_key: &EphemeralKeyBytes,
    ) -> OutgoingCipherKey {
        SharedDomain::derive_ock(ovk, cv, cmstar_bytes, ephemeral_key)
    }

    fn outgoing_plaintext_bytes(
        note: &Self::Note,
        esk: &Self::EphemeralSecretKey,
    ) -> OutPlaintextBytes {
        SharedDomain::outgoing_plaintext_bytes(note, esk)
    }

    fn epk_bytes(epk: &Self::EphemeralPublicKey) -> EphemeralKeyBytes {
        SharedDomain::epk_bytes(epk)
    }

    fn epk(ephemeral_key: &EphemeralKeyBytes) -> Option<Self::EphemeralPublicKey> {
        SharedDomain::epk(ephemeral_key)
    }

    fn cmstar(note: &Self::Note) -> Self::ExtractedCommitment {
        SharedDomain::cmstar(note)
    }

    fn parse_note_plaintext_without_memo_ivk(
        &self,
        ivk: &Self::IncomingViewingKey,
        plaintext: &[u8],
    ) -> Option<(Self::Note, Self::Recipient)> {
        let note_version = self.parse_note_version(plaintext)?;
        self.shared.parse_note_plaintext_without_memo_ivk(
            ivk,
            plaintext,
            note_version,
            DiversifiedTransmissionKey::derive,
        )
    }

    fn parse_note_plaintext_without_memo_ovk(
        &self,
        pk_d: &Self::DiversifiedTransmissionKey,
        plaintext: &NotePlaintextBytes,
    ) -> Option<(Self::Note, Self::Recipient)> {
        let note_version = self.parse_note_version(&plaintext.0)?;
        self.shared
            .parse_note_plaintext_without_memo_ovk(pk_d, plaintext, note_version)
    }

    fn extract_memo(&self, plaintext: &NotePlaintextBytes) -> Self::Memo {
        self.shared.extract_memo(plaintext)
    }

    fn extract_pk_d(out_plaintext: &OutPlaintextBytes) -> Option<Self::DiversifiedTransmissionKey> {
        SharedDomain::extract_pk_d(out_plaintext)
    }

    fn extract_esk(out_plaintext: &OutPlaintextBytes) -> Option<Self::EphemeralSecretKey> {
        SharedDomain::extract_esk(out_plaintext)
    }
}

macro_rules! impl_shielded_output {
    ($domain:ty) => {
        impl<T> ShieldedOutput<$domain, ENC_CIPHERTEXT_SIZE> for Action<T> {
            fn ephemeral_key(&self) -> EphemeralKeyBytes {
                EphemeralKeyBytes(self.encrypted_note().epk_bytes)
            }

            fn cmstar_bytes(&self) -> [u8; 32] {
                self.cmx().to_bytes()
            }

            fn enc_ciphertext(&self) -> &[u8; ENC_CIPHERTEXT_SIZE] {
                &self.encrypted_note().enc_ciphertext
            }
        }

        impl ShieldedOutput<$domain, ENC_CIPHERTEXT_SIZE> for crate::pczt::Action {
            fn ephemeral_key(&self) -> EphemeralKeyBytes {
                EphemeralKeyBytes(self.output().encrypted_note().epk_bytes)
            }

            fn cmstar_bytes(&self) -> [u8; 32] {
                self.output().cmx().to_bytes()
            }

            fn enc_ciphertext(&self) -> &[u8; ENC_CIPHERTEXT_SIZE] {
                &self.output().encrypted_note().enc_ciphertext
            }
        }

        impl ShieldedOutput<$domain, COMPACT_NOTE_SIZE> for CompactAction {
            fn ephemeral_key(&self) -> EphemeralKeyBytes {
                EphemeralKeyBytes(self.ephemeral_key.0)
            }

            fn cmstar_bytes(&self) -> [u8; 32] {
                self.cmx.to_bytes()
            }

            fn enc_ciphertext(&self) -> &[u8; COMPACT_NOTE_SIZE] {
                &self.enc_ciphertext
            }
        }
    };
}

impl_shielded_output!(OrchardDomain);
impl_shielded_output!(IronwoodDomain);
impl_shielded_output!(BundleDomain);

/// Implementation of in-band secret distribution for Orchard bundles.
pub type OrchardNoteEncryption = zcash_note_encryption::NoteEncryption<OrchardDomain>;
/// Implementation of in-band secret distribution for Ironwood bundles.
///
/// This is the [`NoteEncryption`] instantiation for [`IronwoodDomain`].
///
/// [`NoteEncryption`]: zcash_note_encryption::NoteEncryption
pub type IronwoodNoteEncryption = zcash_note_encryption::NoteEncryption<IronwoodDomain>;

/// A compact Action for light clients.
#[derive(Clone)]
pub struct CompactAction {
    nullifier: Nullifier,
    cmx: ExtractedNoteCommitment,
    ephemeral_key: EphemeralKeyBytes,
    enc_ciphertext: [u8; 52],
}

impl fmt::Debug for CompactAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactAction")
    }
}

impl<T> From<&Action<T>> for CompactAction {
    fn from(action: &Action<T>) -> Self {
        CompactAction {
            nullifier: *action.nullifier(),
            cmx: *action.cmx(),
            ephemeral_key: EphemeralKeyBytes(action.encrypted_note().epk_bytes),
            enc_ciphertext: action.encrypted_note().enc_ciphertext[..52]
                .try_into()
                .unwrap(),
        }
    }
}

impl CompactAction {
    /// Create a CompactAction from its constituent parts
    pub fn from_parts(
        nullifier: Nullifier,
        cmx: ExtractedNoteCommitment,
        ephemeral_key: EphemeralKeyBytes,
        enc_ciphertext: [u8; 52],
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
    use zcash_note_encryption::Domain;

    use crate::{
        keys::OutgoingViewingKey,
        note::{ExtractedNoteCommitment, NoteVersion, Nullifier, RandomSeed, Rho},
        value::NoteValue,
        Address, Note,
    };

    use super::{CompactAction, OrchardDomain, OrchardNoteEncryption};

    /// Creates a fake `CompactAction` paying the given recipient the specified value.
    ///
    /// Returns the `CompactAction` and the new note.
    pub fn fake_compact_action<R: RngCore>(
        rng: &mut R,
        nf_old: Nullifier,
        recipient: Address,
        value: NoteValue,
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
        let note = Note::from_parts(recipient, value, rho, rseed, NoteVersion::V2).unwrap();
        let encryptor = OrchardNoteEncryption::new(ovk, note, [0u8; 512]);
        let cmx = ExtractedNoteCommitment::from(note.commitment());
        let ephemeral_key = OrchardDomain::epk_bytes(encryptor.epk());
        let enc_ciphertext = encryptor.encrypt_note_plaintext();

        (
            CompactAction {
                nullifier: nf_old,
                cmx,
                ephemeral_key,
                enc_ciphertext: enc_ciphertext.as_ref()[..52].try_into().unwrap(),
            },
            note,
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;
    use zcash_note_encryption::{
        try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk, Domain,
        EphemeralKeyBytes,
    };

    use super::{
        prf_ock_orchard, CompactAction, IronwoodDomain, IronwoodNoteEncryption, OrchardDomain,
        OrchardNoteEncryption, SharedDomain,
    };
    use crate::{
        action::Action,
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, IncomingViewingKey,
            OutgoingViewingKey, PreparedIncomingViewingKey, Scope, SpendingKey,
        },
        note::{
            ExtractedNoteCommitment, NoteVersion, Nullifier, RandomSeed, Rho,
            TransmittedNoteCiphertext,
        },
        primitives::redpallas,
        value::{NoteValue, ValueCommitTrapdoor, ValueCommitment, ValueSum},
        Address, Note,
    };

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
            rho,
            NoteVersion::V3,
            &mut rng,
        );
        let memo = [7u8; 512];
        let cv_net = ValueCommitment::derive(ValueSum::from_raw(5), ValueCommitTrapdoor::zero());
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
        let test_vectors = crate::test_vectors::note_encryption::test_vectors();

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
            let note = Note::from_parts(recipient, value, rho, rseed, note_version).unwrap();
            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);

            let action = Action::from_parts(
                // nf_old is the nullifier revealed by the receiving Action.
                nf_old,
                // We don't need a real rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext {
                    epk_bytes: ephemeral_key.0,
                    enc_ciphertext: tv.c_enc,
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

            let domain = OrchardDomain {
                shared: SharedDomain { rho },
            };

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
            rho,
            NoteVersion::V2,
            &mut rng,
        );
        let note_v3 = Note::new(
            recipient,
            NoteValue::from_raw(5),
            rho,
            NoteVersion::V3,
            &mut rng,
        );
        let orchard_domain = OrchardDomain {
            shared: SharedDomain { rho },
        };
        let ironwood_domain = IronwoodDomain {
            shared: SharedDomain { rho },
        };

        let np_v2 = OrchardDomain::note_plaintext_bytes(&note_v2, &memo);
        let np_v3 = IronwoodDomain::note_plaintext_bytes(&note_v3, &memo);
        let pk_d = recipient.pk_d();

        assert_eq!(
            orchard_domain
                .parse_note_plaintext_without_memo_ovk(pk_d, &np_v2)
                .map(|(note, _)| note),
            Some(note_v2)
        );
        assert_eq!(
            ironwood_domain
                .parse_note_plaintext_without_memo_ovk(pk_d, &np_v3)
                .map(|(note, _)| note),
            Some(note_v3)
        );
        assert!(orchard_domain
            .parse_note_plaintext_without_memo_ovk(pk_d, &np_v3)
            .is_none());
        assert!(ironwood_domain
            .parse_note_plaintext_without_memo_ovk(pk_d, &np_v2)
            .is_none());
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
}
