use core::{fmt, ops::Range};

use memuse::DynamicUsage;

use crate::{
    note::{ExtractedNoteCommitment, Nullifier, Rho, TransmittedNoteCiphertext},
    primitives::redpallas::{self, SpendAuth},
    value::ValueCommitmentBytes,
};

use super::{Action, ActionFromPartsError};

/// Byte length of an Action description, per [Action Encoding and Consensus][actionenc].
///
/// [actionenc]: https://zips.z.cash/protocol/protocol.pdf#actionencodingandconsensus
pub const ACTION_DESCRIPTION_SIZE: usize = 820;

// Field offsets within an Action description. Ordering is consensus-critical.
const CV_NET: Range<usize> = 0..32;
const NULLIFIER: Range<usize> = 32..64;
const RK: Range<usize> = 64..96;
const CMX: Range<usize> = 96..128;
const EPK: Range<usize> = 128..160;
const ENC_CIPHERTEXT: Range<usize> = 160..740;
const OUT_CIPHERTEXT: Range<usize> = 740..ACTION_DESCRIPTION_SIZE;

/// An [`Action`] with `cv_net`, `rk` and `epk` left compressed.
///
/// Building, encoding, hashing and trial-decrypting one costs no curve arithmetic.
/// [`Self::decompress`] is the only way from here to an [`Action`].
#[derive(Clone, Debug)]
pub struct ActionBytes<A> {
    nf: Nullifier,
    rk: redpallas::VerificationKeyBytes<SpendAuth>,
    cmx: ExtractedNoteCommitment,
    encrypted_note: TransmittedNoteCiphertext,
    cv_net: ValueCommitmentBytes,
    authorization: A,
}

impl<A> ActionBytes<A> {
    /// Returns the nullifier of the note being spent.
    pub fn nullifier(&self) -> &Nullifier {
        &self.nf
    }

    /// Returns the encoded randomized verification key for the note being spent.
    pub fn rk(&self) -> &redpallas::VerificationKeyBytes<SpendAuth> {
        &self.rk
    }

    /// Returns the commitment to the new note being created.
    pub fn cmx(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    /// Returns the encrypted note ciphertext.
    pub fn encrypted_note(&self) -> &TransmittedNoteCiphertext {
        &self.encrypted_note
    }

    /// Obtains the [`Rho`] value that was used to construct the new note being created.
    pub fn rho(&self) -> Rho {
        Rho::from_nf_old(self.nf)
    }

    /// Returns the encoded commitment to the net value created or consumed by this action.
    pub fn cv_net(&self) -> &ValueCommitmentBytes {
        &self.cv_net
    }

    /// Returns the authorization for this action.
    pub fn authorization(&self) -> &A {
        &self.authorization
    }

    /// Attaches the authorization a transaction carries outside the Action description.
    pub fn with_authorization<U>(self, authorization: U) -> ActionBytes<U> {
        ActionBytes {
            nf: self.nf,
            rk: self.rk,
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            cv_net: self.cv_net,
            authorization,
        }
    }

    /// Recovers the [`Action`]. 3 sqrt.
    ///
    /// # Errors
    ///
    /// First point rule this action breaks. The identity-`rk` and `epk` rules are
    /// [`Action::from_parts`]'s, so they keep exactly one implementation.
    pub fn decompress(self) -> Result<Action<A>, DecompressionError> {
        let cv_net = self
            .cv_net
            .decompress()
            .map_err(|_| DecompressionError::NonCanonicalValueCommitment)?;
        let rk = self
            .rk
            .decompress()
            .map_err(|_| DecompressionError::NonCanonicalRandomizedKey)?;

        Ok(Action::from_parts(
            self.nf,
            rk,
            self.cmx,
            self.encrypted_note,
            cv_net,
            self.authorization,
        )?)
    }

    /// Encodes this Action description.
    ///
    /// `A` is not written: the transaction format carries every action's signature in a
    /// separate array.
    pub fn to_bytes(&self) -> [u8; ACTION_DESCRIPTION_SIZE] {
        let mut bytes = [0u8; ACTION_DESCRIPTION_SIZE];

        bytes[CV_NET].copy_from_slice(&self.cv_net.to_bytes());
        bytes[NULLIFIER].copy_from_slice(&self.nf.to_bytes());
        bytes[RK].copy_from_slice(&self.rk.to_bytes());
        bytes[CMX].copy_from_slice(&self.cmx.to_bytes());
        bytes[EPK].copy_from_slice(&self.encrypted_note.epk_bytes);
        bytes[ENC_CIPHERTEXT].copy_from_slice(&self.encrypted_note.enc_ciphertext);
        bytes[OUT_CIPHERTEXT].copy_from_slice(&self.encrypted_note.out_ciphertext);

        bytes
    }
}

impl ActionBytes<()> {
    /// Decodes an Action description, checking the `nullifier` and `cmx` encodings.
    ///
    /// Unauthorized: the transaction carries every action's signature in a separate array, so
    /// the caller pairs one back on with [`ActionBytes::with_authorization`].
    ///
    /// # Errors
    ///
    /// First field that is not a canonical field-element encoding.
    pub fn from_bytes(bytes: &[u8; ACTION_DESCRIPTION_SIZE]) -> Result<Self, ActionParseError> {
        let field = |range: Range<usize>| -> [u8; 32] {
            bytes[range]
                .try_into()
                .expect("every field range above is 32 bytes")
        };

        let nf = Option::from(Nullifier::from_bytes(&field(NULLIFIER)))
            .ok_or(ActionParseError::NonCanonicalNullifier)?;
        let cmx = Option::from(ExtractedNoteCommitment::from_bytes(&field(CMX)))
            .ok_or(ActionParseError::NonCanonicalExtractedNoteCommitment)?;

        Ok(ActionBytes {
            nf,
            rk: redpallas::VerificationKeyBytes::from(field(RK)),
            cmx,
            encrypted_note: TransmittedNoteCiphertext {
                epk_bytes: field(EPK),
                enc_ciphertext: bytes[ENC_CIPHERTEXT]
                    .try_into()
                    .expect("the enc_ciphertext range is 580 bytes"),
                out_ciphertext: bytes[OUT_CIPHERTEXT]
                    .try_into()
                    .expect("the out_ciphertext range is 80 bytes"),
            },
            cv_net: ValueCommitmentBytes::from(field(CV_NET)),
            authorization: (),
        })
    }
}

impl<A> Action<A> {
    /// Drops to the encoded tier.
    ///
    /// Infallible: an [`Action`] cannot hold a point that fails to encode.
    pub fn compress(self) -> ActionBytes<A> {
        ActionBytes {
            nf: self.nf,
            rk: redpallas::VerificationKeyBytes::from(&self.rk),
            cmx: self.cmx,
            encrypted_note: self.encrypted_note,
            cv_net: ValueCommitmentBytes::from(&self.cv_net),
            authorization: self.authorization,
        }
    }
}

// Slots into `zcash_client_backend`'s full-ciphertext batch scanner, which bounds its queue by
// `Output: DynamicUsage` and adds that to `size_of_val`. Every field here is fixed-size, so the
// whole 888 bytes are already covered by `size_of` and nothing is on the heap.
memuse::impl_no_dynamic_usage!(ActionBytes<redpallas::Signature<SpendAuth>>);

/// An Action description field that is not a valid encoding of its type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActionParseError {
    /// `nullifier` is not a canonical Pallas base field element encoding.
    NonCanonicalNullifier,
    /// `cmx` is not a canonical Pallas base field element encoding.
    NonCanonicalExtractedNoteCommitment,
}

impl fmt::Display for ActionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionParseError::NonCanonicalNullifier => {
                write!(f, "`nullifier` is not a canonical field element encoding")
            }
            ActionParseError::NonCanonicalExtractedNoteCommitment => {
                write!(f, "`cmx` is not a canonical field element encoding")
            }
        }
    }
}

impl core::error::Error for ActionParseError {}

/// An Action description field carrying a point no valid Action description may carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecompressionError {
    /// `cv_net` is not a canonical point encoding.
    NonCanonicalValueCommitment,
    /// `rk` is not a canonical point encoding.
    NonCanonicalRandomizedKey,
    /// The recovered points do not make a valid [`Action`].
    Action(ActionFromPartsError),
}

impl From<ActionFromPartsError> for DecompressionError {
    fn from(e: ActionFromPartsError) -> Self {
        DecompressionError::Action(e)
    }
}

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecompressionError::NonCanonicalValueCommitment => {
                write!(f, "`cv_net` is not a canonical point encoding")
            }
            DecompressionError::NonCanonicalRandomizedKey => {
                write!(f, "`rk` is not a canonical point encoding")
            }
            DecompressionError::Action(e) => e.fmt(f),
        }
    }
}

impl core::error::Error for DecompressionError {}

#[cfg(test)]
mod tests {
    use group::{
        ff::{Field as _, PrimeField as _},
        Group as _, GroupEncoding as _,
    };
    use pasta_curves::pallas;

    use super::{
        ActionBytes, ActionFromPartsError, ActionParseError, DecompressionError,
        ACTION_DESCRIPTION_SIZE, CMX, CV_NET, ENC_CIPHERTEXT, EPK, NULLIFIER, OUT_CIPHERTEXT, RK,
    };
    use alloc::vec::Vec;

    use crate::{
        note::{ExtractedNoteCommitment, Nullifier},
        primitives::redpallas::{self, SpendAuth},
        value::{ValueCommitTrapdoor, ValueCommitment, ValueSum},
    };

    // Not a canonical encoding of any Pallas point, nor of any Pallas base field element
    const NON_CANONICAL: [u8; 32] = [0xff; 32];
    // The identity is the one point the `rk` rule forbids outright
    const IDENTITY: [u8; 32] = [0u8; 32];

    /// The key for scalar 1 is the SpendAuthSig basepoint, which is not the identity.
    fn non_identity_rk() -> [u8; 32] {
        let sk = redpallas::SigningKey::<SpendAuth>::try_from(pallas::Scalar::ONE.to_repr())
            .expect("1 is a valid scalar");
        redpallas::VerificationKeyBytes::from(&redpallas::VerificationKey::<SpendAuth>::from(&sk))
            .to_bytes()
    }

    fn valid_cv_net() -> [u8; 32] {
        ValueCommitment::derive(ValueSum::from_raw(42), ValueCommitTrapdoor::zero()).to_bytes()
    }

    /// Every field but the ones named is valid, so only the field under test is ever at fault.
    fn action(rk: [u8; 32], cv_net: [u8; 32], epk_bytes: [u8; 32]) -> ActionBytes<()> {
        let mut bytes = [0u8; ACTION_DESCRIPTION_SIZE];
        bytes[CV_NET].copy_from_slice(&cv_net);
        bytes[NULLIFIER].fill(1);
        bytes[RK].copy_from_slice(&rk);
        bytes[CMX].fill(2);
        bytes[EPK].copy_from_slice(&epk_bytes);
        bytes[ENC_CIPHERTEXT].fill(4);
        bytes[OUT_CIPHERTEXT].fill(5);

        ActionBytes::from_bytes(&bytes).expect("`nf` and `cmx` are canonical")
    }

    fn valid_action() -> ActionBytes<()> {
        action(
            non_identity_rk(),
            valid_cv_net(),
            pallas::Point::generator().to_bytes(),
        )
    }

    #[test]
    fn decompress_enforces_the_point_rules() {
        let generator = pallas::Point::generator().to_bytes();

        assert!(valid_action().decompress().is_ok());

        for (action, expected) in [
            (
                action(non_identity_rk(), NON_CANONICAL, generator),
                DecompressionError::NonCanonicalValueCommitment,
            ),
            (
                action(NON_CANONICAL, valid_cv_net(), generator),
                DecompressionError::NonCanonicalRandomizedKey,
            ),
            (
                action(IDENTITY, valid_cv_net(), generator),
                DecompressionError::Action(ActionFromPartsError::IdentityRk),
            ),
            // Identity and undecodable are both invalid `epk`s
            (
                action(non_identity_rk(), valid_cv_net(), IDENTITY),
                DecompressionError::Action(ActionFromPartsError::InvalidEpk),
            ),
            (
                action(non_identity_rk(), valid_cv_net(), NON_CANONICAL),
                DecompressionError::Action(ActionFromPartsError::InvalidEpk),
            ),
        ] {
            assert_eq!(action.decompress().unwrap_err(), expected);
        }
    }

    /// Scanning goes through `ShieldedOutput`, so it must succeed on actions whose every point
    /// is garbage — and batched, which is how a wallet actually scans.
    #[test]
    fn scanning_needs_no_decompression() {
        use rand::rngs::OsRng;
        use zcash_note_encryption::{batch, Domain as _};

        use crate::{
            keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendingKey},
            note::Rho,
            note_encryption::{OrchardDomain, OrchardNoteEncryption},
            value::NoteValue,
            Note, NoteVersion,
        };

        let mut rng = OsRng;
        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let recipient = fvk.address_at(0u32, Scope::External);
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
        let memo = [9u8; 512];

        let actions: Vec<ActionBytes<()>> = (0..3u64)
            .map(|i| {
                let nf = Nullifier::dummy(&mut rng);
                let note = Note::new(
                    recipient,
                    NoteValue::from_raw(i + 1),
                    Rho::from_nf_old(nf),
                    NoteVersion::V2,
                    &mut rng,
                );
                let cmx = ExtractedNoteCommitment::from(note.commitment());
                let encryptor = OrchardNoteEncryption::new(None, note, memo);
                let out_ciphertext = encryptor.encrypt_outgoing_plaintext(
                    &ValueCommitment::derive(ValueSum::from_raw(1), ValueCommitTrapdoor::zero()),
                    &cmx,
                    &mut rng,
                );

                // Both points undecodable
                let mut bytes = [0u8; ACTION_DESCRIPTION_SIZE];
                bytes[CV_NET].copy_from_slice(&NON_CANONICAL);
                bytes[NULLIFIER].copy_from_slice(&nf.to_bytes());
                bytes[RK].copy_from_slice(&NON_CANONICAL);
                bytes[CMX].copy_from_slice(&cmx.to_bytes());
                bytes[EPK].copy_from_slice(&OrchardDomain::epk_bytes(encryptor.epk()).0);
                bytes[ENC_CIPHERTEXT].copy_from_slice(&encryptor.encrypt_note_plaintext());
                bytes[OUT_CIPHERTEXT].copy_from_slice(&out_ciphertext);

                ActionBytes::from_bytes(&bytes).expect("`nf` and `cmx` are canonical")
            })
            .collect();

        for action in &actions {
            assert!(action.clone().decompress().is_err());
        }

        let items: Vec<_> = actions
            .iter()
            .map(|a| (OrchardDomain::for_action_bytes(a), a.clone()))
            .collect();

        for (i, found) in batch::try_note_decryption(&[ivk], &items)
            .into_iter()
            .enumerate()
        {
            let ((note, _, found_memo), _) = found.expect("every action pays this ivk");
            assert_eq!(note.value().inner(), i as u64 + 1);
            assert_eq!(found_memo, memo);
        }
    }

    /// Field order is consensus-critical, so pin each offset rather than only round-tripping.
    #[test]
    fn to_bytes_matches_the_consensus_field_order() {
        let action = valid_action();
        let bytes = action.to_bytes();

        assert_eq!(bytes.len(), ACTION_DESCRIPTION_SIZE);
        assert_eq!(&bytes[0..32], &action.cv_net().to_bytes());
        assert_eq!(&bytes[32..64], &action.nullifier().to_bytes());
        assert_eq!(&bytes[64..96], &action.rk().to_bytes());
        assert_eq!(&bytes[96..128], &action.cmx().to_bytes());
        assert_eq!(&bytes[128..160], &action.encrypted_note().epk_bytes);
        assert_eq!(&bytes[160..740], &action.encrypted_note().enc_ciphertext);
        assert_eq!(&bytes[740..820], &action.encrypted_note().out_ciphertext);
    }

    /// The wire and the point tier are the only two ways in, and they must agree.
    #[test]
    fn round_trips_through_the_encoding_and_the_point_tier() {
        let bytes = valid_action().to_bytes();

        assert_eq!(
            ActionBytes::from_bytes(&bytes)
                .expect("a written description reads back")
                .to_bytes(),
            bytes,
        );

        let action = valid_action().decompress().expect("every field is valid");
        assert_eq!(action.compress().to_bytes(), bytes);
    }

    /// The point rules are `decompress`'s, not the codec's, so a peer's action is readable
    /// before any curve arithmetic runs.
    #[test]
    fn from_bytes_defers_the_point_rules_to_decompress() {
        // cv_net, rk, epk
        for range in [0..32, 64..96, 128..160] {
            let mut bytes = valid_action().to_bytes();
            bytes[range].copy_from_slice(&NON_CANONICAL);
            let decoded = ActionBytes::from_bytes(&bytes).expect("point rules are not the codec's");
            assert!(decoded.decompress().is_err());
        }
    }

    /// The signature array is read after the actions, so a decoded action is paired with its
    /// signature afterwards and must survive the round trip unchanged.
    #[test]
    fn with_authorization_preserves_the_description() {
        let action = valid_action();
        let signed = action.clone().with_authorization(7u8);

        assert_eq!(signed.to_bytes(), action.to_bytes());
        assert_eq!(*signed.authorization(), 7u8);
    }

    #[test]
    fn from_bytes_rejects_non_canonical_field_elements() {
        for (range, expected) in [
            (32..64, ActionParseError::NonCanonicalNullifier),
            (
                96..128,
                ActionParseError::NonCanonicalExtractedNoteCommitment,
            ),
        ] {
            let mut bytes = valid_action().to_bytes();
            bytes[range].copy_from_slice(&NON_CANONICAL);
            assert_eq!(ActionBytes::from_bytes(&bytes).unwrap_err(), expected);
        }
    }
}
