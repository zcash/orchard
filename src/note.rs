//! Data structures used for note construction.
use alloc::vec::Vec;
use core::fmt;
use memuse::DynamicUsage;

use blake2b_simd::Params as Blake2bParams;
use ff::PrimeField;
use group::GroupEncoding;
use pasta_curves::pallas;
use rand::RngCore;
use subtle::{Choice, ConditionallySelectable, CtOption};

use crate::{
    keys::{EphemeralSecretKey, FullViewingKey, Scope, SpendingKey},
    primitives::OrchardPrimitives,
    spec::{to_base, to_scalar, NonIdentityPallasPoint, NonZeroPallasScalar, PrfExpand},
    value::NoteValue,
    Address,
};

pub(crate) mod asset_base;
pub use self::asset_base::AssetBase;
#[cfg(feature = "zsa-issuance")]
pub use self::asset_base::AssetId;

const PRF_EXPAND_PERSONALIZATION: &[u8; 16] = b"Zcash_ExpandSeed";
const ZIP2005_ORCHARD_QR_RCM_DOMAIN_SEPARATOR: u8 = 0x0B;

#[cfg(not(feature = "unstable-voting-circuits"))]
pub(crate) mod commitment;
#[cfg(feature = "unstable-voting-circuits")]
pub mod commitment;
#[cfg(feature = "unstable-voting-circuits")]
pub use self::commitment::NoteCommitTrapdoor;
pub use self::commitment::{ExtractedNoteCommitment, NoteCommitment};

/// Note plaintext version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteVersion {
    /// The [ZIP 212] Orchard note plaintext format, identified by lead byte
    /// `0x02`.
    ///
    /// [ZIP 212]: https://zips.z.cash/zip-0212
    V2,
    /// The quantum-recoverable Ironwood note plaintext version defined in
    /// [ZIP 2005].
    ///
    /// [ZIP 2005]: https://zips.z.cash/zip-2005
    V3,
    /// ZSA
    ZSA,
}

impl NoteVersion {
    /// Returns the note plaintext lead byte signaling this version.
    pub(crate) const fn lead_byte(self) -> u8 {
        match self {
            Self::V2 => 0x02,
            Self::V3 => 0x03,
            Self::ZSA => 0x04,
        }
    }

    /// Parses a note plaintext lead byte into the corresponding version,
    /// returning `None` if the byte is not a recognized version.
    pub(crate) fn from_lead_byte(b: u8) -> Option<Self> {
        match b {
            0x02 => Some(Self::V2),
            0x03 => Some(Self::V3),
            0x04 => Some(Self::ZSA),
            _ => None,
        }
    }
}

#[cfg(not(feature = "unstable-voting-circuits"))]
pub(crate) mod nullifier;
#[cfg(feature = "unstable-voting-circuits")]
pub mod nullifier;
pub use self::nullifier::Nullifier;

/// The randomness used to construct a note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rho(pallas::Base);

// We know that `pallas::Base` doesn't allocate internally.
memuse::impl_no_dynamic_usage!(Rho);

impl Rho {
    /// Deserialize the rho value from a byte array.
    ///
    /// This should only be used in cases where the components of a `Note` are being serialized and
    /// stored individually. Use [`Action::rho`] or [`CompactAction::rho`] to obtain the [`Rho`]
    /// value otherwise.
    ///
    /// [`Action::rho`]: crate::action::Action::rho
    /// [`CompactAction::rho`]: crate::note_encryption::CompactAction::rho
    pub fn from_bytes(bytes: &[u8; 32]) -> CtOption<Self> {
        pallas::Base::from_repr(*bytes).map(Rho)
    }

    /// Serialize the rho value to its canonical byte representation.
    pub fn to_bytes(self) -> [u8; 32] {
        self.0.to_repr()
    }

    /// Constructs the [`Rho`] value to be used to construct a new note from the revealed nullifier
    /// of the note being spent in the [`Action`] under construction.
    ///
    /// [`Action`]: crate::action::Action
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn from_nf_old(nf: Nullifier) -> Self {
        Rho(nf.inner())
    }

    /// Consumes `self` and returns the inner field element.
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn into_inner(self) -> pallas::Base {
        self.0
    }
}

/// The ZIP 212 seed randomness for a note.
#[derive(Copy, Clone, Debug)]
pub struct RandomSeed([u8; 32]);

impl RandomSeed {
    pub(crate) fn random(rng: &mut impl RngCore, rho: &Rho) -> Self {
        loop {
            let mut bytes = [0; 32];
            rng.fill_bytes(&mut bytes);
            let rseed = RandomSeed::from_bytes(bytes, rho);
            if rseed.is_some().into() {
                break rseed.unwrap();
            }
        }
    }

    /// Reads a note's random seed from bytes, given the note's rho value.
    ///
    /// Returns `None` if the rho value is not for the same note as the seed.
    pub fn from_bytes(rseed: [u8; 32], rho: &Rho) -> CtOption<Self> {
        let rseed = RandomSeed(rseed);
        let esk = rseed.esk_inner(rho);
        CtOption::new(rseed, esk.is_some())
    }

    /// Returns the byte array corresponding to this seed.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn psi(&self, rho: &Rho) -> pallas::Base {
        to_base(PrfExpand::PSI.with(&self.0, &rho.to_bytes()))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn esk_inner(&self, rho: &Rho) -> CtOption<NonZeroPallasScalar> {
        NonZeroPallasScalar::from_scalar(to_scalar(
            PrfExpand::ORCHARD_ESK.with(&self.0, &rho.to_bytes()),
        ))
    }

    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn esk(&self, rho: &Rho) -> NonZeroPallasScalar {
        // We can't construct a RandomSeed for which this unwrap fails.
        self.esk_inner(rho).unwrap()
    }

    /// The rcm derivation for V2 (ZIP 212) notes.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn rcm_v2(&self, rho: &Rho) -> commitment::NoteCommitTrapdoor {
        commitment::NoteCommitTrapdoor(to_scalar(
            PrfExpand::ORCHARD_RCM.with(&self.0, &rho.to_bytes()),
        ))
    }

    /// Quantum-recoverable rcm derivation per [ZIP 2005].
    ///
    /// Binds rcm to all note fields for post-quantum commitment binding. This
    /// implements $\mathsf{H}^{\mathsf{rcm},\mathsf{Orchard}}\_{\mathsf{rseed}}$:
    ///
    /// $$
    /// \mathsf{pre}\_{\mathsf{rcm}} =
    /// [ \mathtt{0x0B} ]
    /// \mathbin\Vert \mathsf{g}^\star\_{\mathsf{d}}
    /// \mathbin\Vert \mathsf{pk}^\star\_{\mathsf{d}}
    /// \mathbin\Vert \mathsf{I2LEOSP}\_{64}(\mathsf{v})
    /// \mathbin\Vert \rho
    /// \mathbin\Vert \mathsf{I2LEOSP}\_{256}(\psi)
    /// $$
    ///
    /// $$
    /// \mathsf{rcm} =
    /// \mathsf{ToScalar}^{\mathsf{Orchard}}
    /// \left(\mathsf{PRF}^{\mathsf{expand}}\_{\mathsf{rseed}}
    /// (\mathsf{pre}\_{\mathsf{rcm}})\right)
    /// $$
    ///
    /// [ZIP 2005]: https://zips.z.cash/zip-2005
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn rcm_v3(
        &self,
        rho: &Rho,
        g_d: &NonIdentityPallasPoint,
        pk_d: &NonIdentityPallasPoint,
        value: u64,
        psi: &pallas::Base,
    ) -> commitment::NoteCommitTrapdoor {
        let mut h = Blake2bParams::new()
            .hash_length(64)
            .personal(PRF_EXPAND_PERSONALIZATION)
            .to_state();
        // rseed: raw bytes (32 bytes)
        h.update(&self.0);
        // domain separator: [0x0B] (1 byte, literal)
        h.update(&[ZIP2005_ORCHARD_QR_RCM_DOMAIN_SEPARATOR]);
        // g_d: LEBS2OSP_256(repr_P(g_d)) — compressed Pallas point (32 bytes)
        h.update(&g_d.to_bytes());
        // pk_d: LEBS2OSP_256(repr_P(pk_d)) — compressed Pallas point (32 bytes)
        h.update(&pk_d.to_bytes());
        // v: I2LEOSP_64(v) — unsigned 64-bit little-endian (8 bytes)
        h.update(&value.to_le_bytes());
        // rho: LEBS2OSP_256(repr_P(rho)) — Pallas base field canonical repr (32 bytes)
        h.update(&rho.0.to_repr());
        // psi: LEBS2OSP_256(repr_P(psi)) — Pallas base field canonical repr (32 bytes)
        h.update(&psi.to_repr());

        commitment::NoteCommitTrapdoor(to_scalar(*h.finalize().as_array()))
    }
}

impl ConditionallySelectable for RandomSeed {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let result: Vec<u8> =
            a.0.iter()
                .zip(b.0.iter())
                .map(|(a_i, b_i)| u8::conditional_select(a_i, b_i, choice))
                .collect();
        RandomSeed(<[u8; 32]>::try_from(result).unwrap())
    }
}

/// A discrete amount of funds received by an address.
#[derive(Debug, Copy, Clone)]
pub struct Note {
    /// The recipient of the funds.
    recipient: Address,
    /// The value of this note.
    value: NoteValue,
    /// The asset of this note.
    asset: AssetBase,
    /// A unique creation ID for this note.
    ///
    /// For notes created by spending an existing note, `rho` is derived from the
    /// nullifier of the spent note.
    ///
    /// For issuance notes ([ZIP-227]), `rho` is initially unset and later
    /// deterministically derived from the issuance context when
    /// `update_rho_for_issuance_note` is called.
    ///
    /// The `rho` value is used as domain-separated randomness in the note
    /// commitment and must be initialized before commitment or equality checks.
    ///
    /// [ZIP-227]: https://zips.z.cash/zip-0227
    rho: Option<Rho>,
    /// The seed randomness for various note components.
    rseed: RandomSeed,
    /// The seed randomness for split notes.
    ///
    /// If it is not a split note, this field is `None`.
    rseed_split_note: CtOption<RandomSeed>,
    /// The note plaintext version, determining rcm derivation strategy.
    version: NoteVersion,
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        // Notes are canonically defined by their commitments.
        ExtractedNoteCommitment::from(self.commitment())
            .eq(&ExtractedNoteCommitment::from(other.commitment()))
    }
}

impl Eq for Note {}

impl Note {
    /// Creates a `Note` from its component parts.
    ///
    /// Returns `None` if a valid [`NoteCommitment`] cannot be derived from the note.
    ///
    /// # Caveats
    ///
    /// This low-level constructor enforces that the provided arguments produce an
    /// internally valid `Note`. However, it allows notes to be constructed in a way that
    /// violates required security checks for note decryption, as specified in
    /// [Section 4.19] of the Zcash Protocol Specification. Users of this constructor
    /// should only call it with note components that have been fully validated by
    /// decrypting a received note according to [Section 4.19].
    ///
    /// [Section 4.19]: https://zips.z.cash/protocol/protocol.pdf#saplingandorchardinband
    pub fn from_parts(
        recipient: Address,
        value: NoteValue,
        asset: AssetBase,
        rho: Rho,
        rseed: RandomSeed,
        version: NoteVersion,
    ) -> CtOption<Self> {
        Self::from_parts_internal(
            recipient,
            value,
            asset,
            rho,
            rseed,
            CtOption::new(rseed, 0u8.into()),
            version,
        )
    }

    /// Creates a `Note` from its component parts.
    ///
    /// This additionally permits constructing a [Split Input note], which is necessary
    /// for constructing certain patterns of bundles containing ZSA outputs. It is used by
    /// the PCZT code, which is the only place where these notes are serialized.
    ///
    /// Returns `None` if a valid [`NoteCommitment`] cannot be derived from the note.
    ///
    /// # Caveats
    ///
    /// See [`Self::from_parts`].
    ///
    /// [Split Input note]: https://zips.z.cash/zip-0226#split-notes
    pub(crate) fn from_parts_internal(
        recipient: Address,
        value: NoteValue,
        asset: AssetBase,
        rho: Rho,
        rseed: RandomSeed,
        rseed_split_note: CtOption<RandomSeed>,
        version: NoteVersion,
    ) -> CtOption<Self> {
        let note = Note {
            recipient,
            value,
            asset,
            rho: Some(rho),
            rseed,
            rseed_split_note,
            version,
        };
        CtOption::new(note, note.commitment_inner().is_some())
    }

    /// Generates a new note.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn new(
        recipient: Address,
        value: NoteValue,
        asset: AssetBase,
        rho: Rho,
        version: NoteVersion,
        mut rng: impl RngCore,
    ) -> Self {
        loop {
            let note = Note::from_parts(
                recipient,
                value,
                asset,
                rho,
                RandomSeed::random(&mut rng, &rho),
                version,
            );
            if note.is_some().into() {
                break note.unwrap();
            }
        }
    }

    /// Generates a new issuance note with an uninitialized `rho`.
    ///
    /// For issuance notes ([ZIP-227]), the `rho` value is not known at creation
    /// time and is therefore left unset. It is later deterministically derived
    /// from the issuance context and assigned via `update_rho_for_issuance_note`.
    ///
    /// A temporary `rseed` is sampled at construction time and later updated
    /// by `update_rho_for_issuance_note` to ensure a valid note commitment.
    ///
    /// [ZIP-227]: https://zips.z.cash/zip-0227
    pub(crate) fn new_issue_note(
        recipient: Address,
        value: NoteValue,
        asset: AssetBase,
        version: NoteVersion,
        mut rng: impl RngCore,
    ) -> Self {
        let rseed = RandomSeed::random(&mut rng, &Rho(pallas::Base::zero()));
        Note {
            recipient,
            value,
            asset,
            rho: None,
            rseed,
            rseed_split_note: CtOption::new(rseed, 0u8.into()),
            version,
        }
    }

    /// Generates a dummy spent note.
    ///
    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(crate) fn dummy(
        rng: &mut impl RngCore,
        rho: Option<Rho>,
        note_version: NoteVersion,
    ) -> (SpendingKey, FullViewingKey, Self) {
        let sk = SpendingKey::random(rng);
        let fvk: FullViewingKey = (&sk).into();
        let recipient = fvk.address_at(0u32, Scope::External);

        let note = Note::new(
            recipient,
            NoteValue::ZERO,
            AssetBase::zatoshi(),
            rho.unwrap_or_else(|| Rho::from_nf_old(Nullifier::dummy(rng))),
            note_version,
            rng,
        );

        (sk, fvk, note)
    }

    /// Returns the recipient of this note.
    pub fn recipient(&self) -> Address {
        self.recipient
    }

    /// Returns the value of this note.
    pub fn value(&self) -> NoteValue {
        self.value
    }

    /// Returns the asset of this note.
    pub fn asset(&self) -> AssetBase {
        self.asset
    }

    /// Returns the rseed value of this note.
    pub fn rseed(&self) -> &RandomSeed {
        &self.rseed
    }

    /// Returns the rseed_split_note value of this note.
    pub(crate) fn rseed_split_note(&self) -> CtOption<RandomSeed> {
        self.rseed_split_note
    }

    /// Derives the ephemeral secret key for this note.
    pub(crate) fn esk(&self) -> EphemeralSecretKey {
        EphemeralSecretKey(self.rseed.esk(&self.rho()))
    }

    /// Returns rho of this note.
    pub fn rho(&self) -> Rho {
        self.rho
            .expect("must call Note::update_rho_for_issuance_note first")
    }

    #[cfg(test)]
    pub(crate) fn has_rho(&self) -> bool {
        self.rho.is_some()
    }

    /// Returns the version of this note.
    pub fn version(&self) -> NoteVersion {
        self.version
    }

    /// Derives the ψ value for this note.
    pub(crate) fn psi(&self) -> pallas::Base {
        self.rseed.psi(&self.rho())
    }

    /// Derives the note commitment trapdoor for this note.
    pub(crate) fn rcm(&self) -> commitment::NoteCommitTrapdoor {
        let rho = self.rho();

        match self.version {
            NoteVersion::V2 => self.rseed.rcm_v2(&rho),
            NoteVersion::V3 | NoteVersion::ZSA => {
                let g_d = self.recipient.g_d();
                let pk_d = self.recipient.pk_d().inner();
                let psi = self.rseed.psi(&rho);

                self.rseed
                    .rcm_v3(&rho, &g_d, &pk_d, self.value.inner(), &psi)
            }
        }
    }

    /// Derives the commitment to this note.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    pub fn commitment(&self) -> NoteCommitment {
        // `Note` will always have a note commitment by construction.
        self.commitment_inner().unwrap()
    }

    /// Derives the commitment to this note.
    ///
    /// This is the internal fallible API, used to check at construction time that the
    /// note has a commitment. Once you have a [`Note`] object, use `note.commitment()`
    /// instead.
    ///
    /// Defined in [Zcash Protocol Spec § 3.2: Notes][notes].
    ///
    /// [notes]: https://zips.z.cash/protocol/nu5.pdf#notes
    fn commitment_inner(&self) -> CtOption<NoteCommitment> {
        let g_d = self.recipient.g_d();
        let g_d_bytes = g_d.to_bytes();
        let pk_d = self.recipient.pk_d().inner();
        let pk_d_bytes = pk_d.to_bytes();
        let psi = self.psi();

        NoteCommitment::derive(
            g_d_bytes,
            pk_d_bytes,
            self.value,
            self.asset,
            self.rho().0,
            psi,
            self.rcm(),
        )
    }

    /// Derives the nullifier for this note.
    pub fn nullifier(&self, fvk: &FullViewingKey) -> Nullifier {
        let selected_rseed = self.rseed_split_note.unwrap_or(self.rseed);

        Nullifier::derive(
            fvk.nk(),
            self.rho().0,
            selected_rseed.psi(&self.rho()),
            self.commitment(),
            self.rseed_split_note.is_some(),
        )
    }

    /// Creates a [Split Input note] from a Custom Asset note, for use on the Spend side
    /// of an Output-only Action.
    ///
    /// # Panics
    ///
    /// Panics if `self.asset().is_zatoshi()`.
    ///
    /// [Split Input note]: https://zips.z.cash/zip-0226#split-notes
    pub(crate) fn create_split_note(self, rng: &mut impl RngCore) -> Self {
        assert!(bool::from(!self.asset().is_zatoshi()));
        Note {
            rseed_split_note: CtOption::new(RandomSeed::random(rng, &self.rho()), 1u8.into()),
            ..self
        }
    }

    /// Updates the `rho` value of an issuance note as specified in
    /// [ZIP-227: Issuance of Zcash Shielded Assets][zip227].
    ///
    /// The `rho` value is deterministically derived from the note context and used
    /// in the Sinsemilla-based note commitment. As required by
    /// [Section 5.4.8.4] of the Zcash Protocol Specification, the commitment must not
    /// evaluate to ⊥.
    ///
    /// Although the probability of observing ⊥ is negligible, this method enforces
    /// this invariant by resampling a random `rseed` until a valid commitment is produced.
    ///
    /// [zip227]: https://zips.z.cash/zip-0227
    /// [Section 5.4.8.4]: https://zips.z.cash/protocol/protocol.pdf#concretesinsemillacommit
    pub(crate) fn update_rho_for_issuance_note(
        &mut self,
        nullifier: &Nullifier,
        index_action: u32,
        index_note: u32,
        mut rng: impl RngCore,
    ) {
        let rho = rho_for_issuance_note(nullifier, index_action, index_note);
        self.rho = Some(rho);
        loop {
            self.rseed = RandomSeed::random(&mut rng, &rho);
            if self.commitment_inner().is_some().into() {
                break;
            }
        }
    }
}

/// Evaluate the rho value of the issuance note (see
/// [ZIP-227: Issuance of Zcash Shielded Assets][zip227]).
///
/// [zip227]: https://zips.z.cash/zip-0227
pub(crate) fn rho_for_issuance_note(
    nullifier: &Nullifier,
    index_action: u32,
    index_note: u32,
) -> Rho {
    Rho(to_base(PrfExpand::ORCHARD_DERIVED_ISSUE_RHO.with(
        &nullifier.to_bytes(),
        &index_action.to_le_bytes(),
        &index_note.to_le_bytes(),
    )))
}

/// An encrypted note.
#[derive(Clone)]
pub struct TransmittedNoteCiphertext<Pr: OrchardPrimitives> {
    /// The serialization of the ephemeral public key
    pub epk_bytes: [u8; 32],
    /// The encrypted note ciphertext
    pub enc_ciphertext: Pr::NoteCiphertextBytes,
    /// An encrypted value that allows the holder of the outgoing cipher
    /// key for the note to recover the note plaintext.
    pub out_ciphertext: [u8; 80],
}

impl<Pr: OrchardPrimitives> fmt::Debug for TransmittedNoteCiphertext<Pr> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransmittedNoteCiphertext")
            .field("epk_bytes", &self.epk_bytes)
            .field("enc_ciphertext", &hex::encode(self.enc_ciphertext))
            .field("out_ciphertext", &hex::encode(self.out_ciphertext))
            .finish()
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use proptest::prelude::*;

    use crate::{
        address::testing::arb_address,
        note::{asset_base::testing::arb_asset_base, nullifier::testing::arb_nullifier, AssetBase},
        value::{testing::arb_note_value, NoteValue},
    };

    #[cfg(feature = "zsa-issuance")]
    use crate::issuance::auth::{IssueValidatingKey, ZSASchnorr};

    use subtle::CtOption;

    use super::{Note, NoteVersion, RandomSeed, Rho};

    prop_compose! {
        /// Generate an arbitrary random seed
        pub(crate) fn arb_rseed()(elems in prop::array::uniform32(prop::num::u8::ANY)) -> RandomSeed {
            RandomSeed(elems)
        }
    }

    prop_compose! {
        /// Generate an arbitrary note with the given plaintext version.
        pub fn arb_note(value: NoteValue, version: NoteVersion)(
            recipient in arb_address(),
            rho in arb_nullifier().prop_map(Rho::from_nf_old),
            rseed in arb_rseed(),
            asset in arb_asset_base(),
        ) -> Note {
            Note {
                recipient,
                value,
                asset,
                rho: Some(rho),
                rseed,
                rseed_split_note: CtOption::new(rseed, 0u8.into()),
                version,
            }
        }
    }

    prop_compose! {
        /// Generate an arbitrary zatoshi note
        pub fn arb_zatoshi_note(version: NoteVersion)(
            recipient in arb_address(),
            value in arb_note_value(),
            rho in arb_nullifier().prop_map(Rho::from_nf_old),
            rseed in arb_rseed(),
        ) -> Note {
            Note {
                recipient,
                value,
                asset: AssetBase::zatoshi(),
                rho: Some(rho),
                rseed,
                rseed_split_note: CtOption::new(rseed, 0u8.into()),
                version,
            }
        }
    }

    #[cfg(feature = "zsa-issuance")]
    prop_compose! {
        /// Generate an arbitrary zsa note
        pub fn arb_zsa_note(ik: IssueValidatingKey<ZSASchnorr>, asset_desc_hash: [u8; 32], version: NoteVersion)(
            recipient in arb_address(),
            value in arb_note_value(),
            rho in arb_nullifier().prop_map(Rho::from_nf_old),
            rseed in arb_rseed(),
        ) -> Note {
            use crate::note::AssetId;
            Note {
                recipient,
                value,
                asset: AssetBase::custom(&AssetId::new_v0(&ik, &asset_desc_hash)),
                rho: Some(rho),
                rseed,
                rseed_split_note: CtOption::new(rseed, 0u8.into()),
                version,
            }
        }
    }
}

// TODO Constance: update the zcash_test_vectors repository so that keys.rs can be
// generated with post-quantum keys and issuance keys.
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        keys::{FullViewingKey, Scope, SpendingKey},
        test_vectors::keys::TestVector,
    };
    use ff::PrimeField;
    use group::GroupEncoding;

    struct QrRcmDerivation {
        rcm_old_repr: [u8; 32],
        rcm_new_repr: [u8; 32],
        cmx_old_bytes: [u8; 32],
        cmx_qr_bytes: [u8; 32],
    }

    fn qr_rcm_from_key_test_vector(tv: &TestVector) -> QrRcmDerivation {
        let sk = SpendingKey::from_bytes(tv.sk).unwrap();
        let fvk = FullViewingKey::from(&sk);
        let addr = fvk.address_at(0u32, Scope::External);
        let rho = Rho::from_bytes(&tv.note_rho).unwrap();
        let rseed = RandomSeed::from_bytes(tv.note_rseed, &rho).unwrap();

        let g_d = addr.g_d();
        let pk_d = addr.pk_d().inner();
        let g_d_bytes = g_d.to_bytes();
        let pk_d_bytes = pk_d.to_bytes();

        let rcm_old = rseed.rcm_v2(&rho);
        let psi = rseed.psi(&rho);
        let rcm_new = rseed.rcm_v3(&rho, &g_d, &pk_d, tv.note_v, &psi);

        let rcm_old_repr = rcm_old.0.to_repr();
        let rcm_new_repr = rcm_new.0.to_repr();
        let rho_inner = rho.into_inner();
        let value = NoteValue::from_raw(tv.note_v);

        let cmx_old = NoteCommitment::derive(
            g_d_bytes,
            pk_d_bytes,
            value,
            AssetBase::zatoshi(),
            rho_inner,
            psi,
            rcm_old,
        )
        .unwrap();
        let cmx_old_bytes = ExtractedNoteCommitment::from(cmx_old).to_bytes();

        let cmx_qr = NoteCommitment::derive(
            g_d_bytes,
            pk_d_bytes,
            value,
            AssetBase::zatoshi(),
            rho_inner,
            psi,
            rcm_new,
        )
        .unwrap();
        let cmx_qr_bytes = ExtractedNoteCommitment::from(cmx_qr).to_bytes();

        QrRcmDerivation {
            rcm_old_repr,
            rcm_new_repr,
            cmx_old_bytes,
            cmx_qr_bytes,
        }
    }

    #[test]
    fn qr_rcm_differs_from_old_rcm() {
        let tv = &crate::test_vectors::keys::test_vectors()[0];
        let derived = qr_rcm_from_key_test_vector(tv);

        assert_ne!(derived.rcm_old_repr, derived.rcm_new_repr);
        assert_eq!(
            derived.cmx_old_bytes, tv.note_cmx,
            "old cmx must match known test vector"
        );
        assert_ne!(derived.cmx_old_bytes, derived.cmx_qr_bytes);
    }

    #[test]
    fn qr_rcm_verify_stored_vectors() {
        for (i, key_tv) in crate::test_vectors::keys::test_vectors().iter().enumerate() {
            let derived = qr_rcm_from_key_test_vector(key_tv);

            assert_eq!(
                derived.rcm_new_repr, key_tv.note_qr_rcm,
                "vector {i}: rcm_qr mismatch"
            );
            assert_eq!(
                derived.cmx_old_bytes, key_tv.note_cmx,
                "vector {i}: cmx_old mismatch"
            );
            assert_eq!(
                derived.cmx_qr_bytes, key_tv.note_qr_cmx,
                "vector {i}: cmx_qr mismatch"
            );
        }
    }
}
*/
