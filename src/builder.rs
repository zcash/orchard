//! Logic for building Orchard components of transactions.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt;
use core::iter;

use ff::Field;
use pasta_curves::pallas;
use rand::{prelude::SliceRandom, CryptoRng, RngCore};

use crate::{
    address::Address,
    bundle::{Authorization, Authorized, Bundle, Flags},
    keys::{
        FullViewingKey, OutgoingViewingKey, Scope, SpendAuthorizingKey, SpendValidatingKey,
        SpendingKey,
    },
    note::{ExtractedNoteCommitment, Note, Nullifier, Rho, TransmittedNoteCiphertext},
    note_encryption::OrchardNoteEncryption,
    primitives::redpallas::{self, Binding, SpendAuth},
    tree::{Anchor, MerklePath},
    value::{self, BalanceError, NoteValue, ValueCommitTrapdoor, ValueCommitment, ValueSum},
    Proof,
};

#[cfg(feature = "circuit")]
use {
    crate::{
        action::Action,
        circuit::{Circuit, Instance, OrchardCircuitVersion, ProvingKey},
    },
    nonempty::NonEmpty,
};

const MIN_ACTIONS: usize = 2;

/// An enumeration of rules for Orchard bundle construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BundleType {
    /// A transactional bundle will be padded if necessary to contain at least 2 actions,
    /// irrespective of whether any genuine actions are required.
    Transactional {
        /// The flags used for the bundle, including whether spends and outputs are enabled.
        flags: Flags,
        /// A flag that, when set to `true`, indicates that a bundle should be produced even if no
        /// spends or outputs have been added to the bundle; in such a circumstance, all of the
        /// actions in the resulting bundle will be dummies.
        bundle_required: bool,
    },
    /// A coinbase bundle is required to have no non-dummy spends, and is built with
    /// [`Flags::SPENDS_DISABLED`]: spends disabled, outputs enabled, and
    /// `disableCrossAddress` unset. No padding is performed.
    ///
    /// Coinbase bundles never set `disableCrossAddress`. Whether an Orchard-format
    /// shielded coinbase bundle is permitted at all is a consensus rule outside this
    /// crate, decided per pool: a pool may allow it (for example, a pool validating
    /// bundles under the `FixedPostNu6_2` circuit version, or a pool that accepts
    /// `disableCrossAddress = 0` bundles under the `Ironwood` circuit version), while
    /// a pool whose rules require `disableCrossAddress = 1` on every bundle thereby
    /// prohibits coinbase bundles entirely.
    Coinbase,
}

impl BundleType {
    /// The default bundle type enables spends and outputs, leaves `disableCrossAddress` unset,
    /// and does not require a bundle to be produced if no spends or outputs have been added to
    /// the bundle.
    pub const DEFAULT: BundleType = BundleType::Transactional {
        flags: Flags::ENABLED,
        bundle_required: false,
    };

    /// The DISABLED bundle type does not permit any bundle to be produced, and when used in the
    /// builder will prevent any spends or outputs from being added.
    pub const DISABLED: BundleType = BundleType::Transactional {
        flags: Flags::from_parts(false, false),
        bundle_required: false,
    };

    /// Returns the number of logical actions that builder will produce in constructing a bundle
    /// of this type, given the specified numbers of spends and outputs.
    ///
    /// For [`BundleType::Transactional`] bundles whose flags set `disableCrossAddress`,
    /// a requested spend and a requested output never share an action (each is paired
    /// with a fabricated zero-value counterpart), so the number of requested actions is
    /// `num_spends + num_outputs` rather than `max(num_spends, num_outputs)`. Wallets
    /// estimating fees (e.g. per [ZIP 317]) must account for this larger action count.
    ///
    /// Returns an error if the specified number of spends and outputs is incompatible with
    /// this bundle type.
    ///
    /// [ZIP 317]: https://zips.z.cash/zip-0317
    pub fn num_actions(
        &self,
        num_spends: usize,
        num_outputs: usize,
    ) -> Result<usize, &'static str> {
        match self {
            BundleType::Transactional {
                flags,
                bundle_required,
            } => {
                // When cross-address transfers are disabled, every action's output is
                // addressed to the note it spends, so a requested spend and a requested
                // output can never share an action: each is paired with a fabricated
                // zero-value counterpart instead.
                let num_requested_actions = if flags.disable_cross_address() {
                    num_spends
                        .checked_add(num_outputs)
                        .ok_or("num_spends + num_outputs overflowed")?
                } else {
                    core::cmp::max(num_spends, num_outputs)
                };

                if !flags.spends_enabled() && num_spends > 0 {
                    Err("Spends are disabled, so num_spends must be zero")
                } else if !flags.outputs_enabled() && num_outputs > 0 {
                    Err("Outputs are disabled, so num_outputs must be zero")
                } else {
                    Ok(if *bundle_required || num_requested_actions > 0 {
                        core::cmp::max(num_requested_actions, MIN_ACTIONS)
                    } else {
                        0
                    })
                }
            }
            BundleType::Coinbase => {
                if num_spends > 0 {
                    Err("Coinbase bundles have spends disabled, so num_spends must be zero")
                } else {
                    Ok(num_outputs)
                }
            }
        }
    }

    /// Returns the set of flags that will be used for bundle construction.
    pub fn flags(&self) -> Flags {
        match self {
            BundleType::Transactional { flags, .. } => *flags,
            BundleType::Coinbase => Flags::SPENDS_DISABLED,
        }
    }
}

/// An error type for the kinds of errors that can occur during bundle construction.
#[derive(Debug)]
#[non_exhaustive]
pub enum BuildError {
    /// Spends are disabled for the provided bundle type.
    SpendsDisabled,
    /// Outputs are disabled for the provided bundle type.
    OutputsDisabled,
    /// The anchor provided to this builder doesn't match the Merkle path used to add a spend.
    AnchorMismatch,
    /// A bundle could not be built because required signatures were missing.
    MissingSignatures,
    /// An error occurred in the process of producing a proof for a bundle.
    #[cfg(feature = "circuit")]
    Proof(halo2_proofs::plonk::Error),
    /// An overflow error occurred while attempting to construct the value
    /// for a bundle.
    ValueSum(value::BalanceError),
    /// External signature is not valid.
    InvalidExternalSignature,
    /// A signature is valid for more than one input. This should never happen if `alpha`
    /// is sampled correctly, and indicates a critical failure in randomness generation.
    DuplicateSignature,
    /// The bundle being constructed violated the construction rules for the requested bundle type.
    BundleTypeNotSatisfiable,
    /// Cross-address transfers are disabled for the bundle being constructed, and an
    /// output is not a wallet-controlled change output.
    CrossAddressDisabled,
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BuildError::*;
        match self {
            MissingSignatures => f.write_str("Required signatures were missing during build"),
            #[cfg(feature = "circuit")]
            Proof(halo2_proofs::plonk::Error::InvalidInstances) => {
                f.write_str(
                    "Could not create proof: provided instances do not match the circuit, \
                     or `disableCrossAddress` is not supported by the proving key's circuit version",
                )
            }
            #[cfg(feature = "circuit")]
            Proof(e) => write!(f, "Could not create proof: {e}"),
            ValueSum(_) => f.write_str("Overflow occurred during value construction"),
            InvalidExternalSignature => f.write_str("External signature was invalid"),
            DuplicateSignature => f.write_str("Signature valid for more than one input"),
            BundleTypeNotSatisfiable => {
                f.write_str("Bundle structure did not conform to requested bundle type.")
            }
            SpendsDisabled => f.write_str("Spends are not enabled for the requested bundle type."),
            OutputsDisabled => f.write_str("Outputs are not enabled for the requested bundle type."),
            AnchorMismatch => {
                f.write_str("All spends must share the anchor requested for the transaction.")
            }
            CrossAddressDisabled => f.write_str(
                "Cross-address transfers are disabled for this bundle: every output must \
                 be a wallet-controlled change output.",
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BuildError {}

#[cfg(feature = "circuit")]
impl From<halo2_proofs::plonk::Error> for BuildError {
    fn from(e: halo2_proofs::plonk::Error) -> Self {
        BuildError::Proof(e)
    }
}

impl From<value::BalanceError> for BuildError {
    fn from(e: value::BalanceError) -> Self {
        BuildError::ValueSum(e)
    }
}

/// An error type for adding a spend to the builder.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SpendError {
    /// Spends aren't enabled for this builder.
    SpendsDisabled,
    /// The anchor provided to this builder doesn't match the merkle path used to add a spend.
    AnchorMismatch,
    /// The full viewing key provided didn't match the note provided
    FvkMismatch,
}

impl fmt::Display for SpendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use SpendError::*;
        f.write_str(match self {
            SpendsDisabled => "Spends are not enabled for this builder",
            AnchorMismatch => "All anchors must be equal.",
            FvkMismatch => "FullViewingKey does not correspond to the given note",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SpendError {}

/// An error type for adding an output to the builder.
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutputError {
    /// Outputs aren't enabled for this builder.
    OutputsDisabled,
    /// Cross-address transfers are disabled for this builder, so ordinary outputs cannot
    /// be added; use [`Builder::add_change_output`] for wallet-controlled change.
    CrossAddressDisabled,
    /// The full viewing key provided does not own the recipient address.
    FvkMismatch,
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OutputError::*;
        f.write_str(match self {
            OutputsDisabled => "Outputs are not enabled for this builder",
            CrossAddressDisabled => {
                "Cross-address transfers are disabled for this builder; use \
                 add_change_output for wallet-controlled change"
            }
            FvkMismatch => "FullViewingKey does not own the recipient address",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OutputError {}

/// Information about a specific note to be spent in an [`Action`].
#[derive(Debug)]
pub struct SpendInfo {
    pub(crate) dummy_sk: Option<SpendingKey>,
    pub(crate) fvk: FullViewingKey,
    pub(crate) scope: Scope,
    pub(crate) note: Note,
    pub(crate) merkle_path: MerklePath,
}

impl SpendInfo {
    /// This constructor is public to enable creation of custom builders.
    /// If you are not creating a custom builder, use [`Builder::add_spend`] instead.
    ///
    /// Creates a `SpendInfo` from note, full viewing key owning the note,
    /// and merkle path witness of the note.
    ///
    /// Returns `None` if the `fvk` does not own the `note`.
    ///
    /// [`Builder::add_spend`]: Builder::add_spend
    pub fn new(fvk: FullViewingKey, note: Note, merkle_path: MerklePath) -> Option<Self> {
        let scope = fvk.scope_for_address(&note.recipient())?;
        Some(SpendInfo {
            dummy_sk: None,
            fvk,
            scope,
            note,
            merkle_path,
        })
    }

    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    fn dummy(rng: &mut impl RngCore) -> Self {
        let (sk, fvk, note) = Note::dummy(rng, None);
        let merkle_path = MerklePath::dummy(rng);

        SpendInfo {
            dummy_sk: Some(sk),
            fvk,
            // We use external scope to avoid unnecessary derivations, because the dummy
            // note's spending key is random and thus scoping is irrelevant.
            scope: Scope::External,
            note,
            merkle_path,
        }
    }

    fn has_matching_anchor(&self, anchor: &Anchor) -> bool {
        if self.note.value() == NoteValue::ZERO {
            true
        } else {
            let cm = self.note.commitment();
            let path_root = self.merkle_path.root(cm.into());
            &path_root == anchor
        }
    }

    /// Builds the spend half of an action.
    ///
    /// The returned values are chosen as in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn build(
        &self,
        mut rng: impl RngCore,
    ) -> (
        Nullifier,
        SpendValidatingKey,
        pallas::Scalar,
        redpallas::VerificationKey<SpendAuth>,
    ) {
        let nf_old = self.note.nullifier(&self.fvk);
        let ak: SpendValidatingKey = self.fvk.clone().into();
        let alpha = pallas::Scalar::random(&mut rng);
        let rk = ak.randomize(&alpha);

        (nf_old, ak, alpha, rk)
    }

    fn into_pczt(self, rng: impl RngCore) -> crate::pczt::Spend {
        let (nf_old, _, alpha, rk) = self.build(rng);

        crate::pczt::Spend {
            nullifier: nf_old,
            rk,
            spend_auth_sig: None,
            recipient: Some(self.note.recipient()),
            value: Some(self.note.value()),
            rho: Some(self.note.rho()),
            rseed: Some(*self.note.rseed()),
            fvk: Some(self.fvk),
            witness: Some(self.merkle_path),
            alpha: Some(alpha),
            zip32_derivation: None,
            dummy_sk: self.dummy_sk,
            proprietary: BTreeMap::new(),
        }
    }
}

/// Information about a specific output to receive funds in an [`Action`].
#[derive(Debug)]
pub struct OutputInfo {
    ovk: Option<OutgoingViewingKey>,
    recipient: Address,
    value: NoteValue,
    memo: [u8; 512],
    /// For wallet-controlled change outputs: the full viewing key that owns `recipient`,
    /// and the scope it owns it under. This enables the builder to fabricate the paired
    /// zero-value spend in bundles that disable cross-address transfers. `None` for
    /// ordinary outputs.
    change_fvk: Option<(FullViewingKey, Scope)>,
}

impl OutputInfo {
    /// Constructs a new OutputInfo from its constituent parts.
    pub fn new(
        ovk: Option<OutgoingViewingKey>,
        recipient: Address,
        value: NoteValue,
        memo: [u8; 512],
    ) -> Self {
        Self {
            ovk,
            recipient,
            value,
            memo,
            change_fvk: None,
        }
    }

    /// Constructs a wallet-controlled change output.
    ///
    /// In a bundle that disables cross-address transfers, the builder pairs this output
    /// with a fabricated zero-value spend controlled by `fvk` at `recipient`, in the
    /// same action; this is the only way to retain shielded value in such a bundle. In
    /// other bundles it behaves exactly like [`OutputInfo::new`].
    ///
    /// Returns `None` if `fvk` does not own `recipient`.
    pub fn change(
        fvk: FullViewingKey,
        ovk: Option<OutgoingViewingKey>,
        recipient: Address,
        value: NoteValue,
        memo: [u8; 512],
    ) -> Option<Self> {
        let scope = fvk.scope_for_address(&recipient)?;
        Some(Self {
            ovk,
            recipient,
            value,
            memo,
            change_fvk: Some((fvk, scope)),
        })
    }

    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    pub fn dummy(rng: &mut impl RngCore) -> Self {
        let fvk: FullViewingKey = (&SpendingKey::random(rng)).into();
        let recipient = fvk.address_at(0u32, Scope::External);

        Self::new(None, recipient, NoteValue::ZERO, [0u8; 512])
    }

    /// Builds the output half of an action.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn build(
        &self,
        cv_net: &ValueCommitment,
        nf_old: Nullifier,
        mut rng: impl RngCore,
    ) -> (Note, ExtractedNoteCommitment, TransmittedNoteCiphertext) {
        let rho = Rho::from_nf_old(nf_old);
        let note = Note::new(self.recipient, self.value, rho, &mut rng);
        let cm_new = note.commitment();
        let cmx = cm_new.into();

        let encryptor = OrchardNoteEncryption::new(self.ovk.clone(), note, self.memo);

        let encrypted_note = TransmittedNoteCiphertext {
            epk_bytes: encryptor.epk().to_bytes().0,
            enc_ciphertext: encryptor.encrypt_note_plaintext(),
            out_ciphertext: encryptor.encrypt_outgoing_plaintext(cv_net, &cmx, &mut rng),
        };

        (note, cmx, encrypted_note)
    }

    fn into_pczt(
        self,
        cv_net: &ValueCommitment,
        nf_old: Nullifier,
        rng: impl RngCore,
    ) -> crate::pczt::Output {
        let (note, cmx, encrypted_note) = self.build(cv_net, nf_old, rng);

        crate::pczt::Output {
            cmx,
            encrypted_note,
            recipient: Some(self.recipient),
            value: Some(self.value),
            rseed: Some(*note.rseed()),
            // TODO: Extract ock from the encryptor and save it so
            // Signers can check `out_ciphertext`.
            ock: None,
            zip32_derivation: None,
            user_address: None,
            proprietary: BTreeMap::new(),
        }
    }
}

/// Information about a specific [`Action`] we plan to build.
#[derive(Debug)]
struct ActionInfo {
    spend: SpendInfo,
    output: OutputInfo,
    rcv: ValueCommitTrapdoor,
}

impl ActionInfo {
    fn new(spend: SpendInfo, output: OutputInfo, rng: impl RngCore) -> Self {
        ActionInfo {
            spend,
            output,
            rcv: ValueCommitTrapdoor::random(rng),
        }
    }

    /// Returns the value sum for this action.
    fn value_sum(&self) -> ValueSum {
        self.spend.note.value() - self.output.value
    }

    /// Builds the action for a given circuit version.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    /// The circuit version must be consistent between actions in a bundle.
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    #[cfg(feature = "circuit")]
    fn build(
        self,
        mut rng: impl RngCore,
        circuit_version: OrchardCircuitVersion,
    ) -> (Action<SigningMetadata>, Circuit) {
        let v_net = self.value_sum();
        let cv_net = ValueCommitment::derive(v_net, self.rcv.clone());

        let (nf_old, ak, alpha, rk) = self.spend.build(&mut rng);
        let (note, cmx, encrypted_note) = self.output.build(&cv_net, nf_old, &mut rng);

        (
            Action::from_parts(
                nf_old,
                rk,
                cmx,
                encrypted_note,
                cv_net,
                SigningMetadata {
                    dummy_ask: self.spend.dummy_sk.as_ref().map(SpendAuthorizingKey::from),
                    parts: SigningParts { ak, alpha },
                },
            )
            .expect(
                "rk is non-identity (α was generated randomly) and epk is a \
                 valid non-identity point by construction",
            ),
            Circuit::from_action_context_unchecked(
                self.spend,
                note,
                alpha,
                self.rcv,
                circuit_version,
            ),
        )
    }

    fn build_for_pczt(self, mut rng: impl RngCore) -> crate::pczt::Action {
        let v_net = self.value_sum();
        let cv_net = ValueCommitment::derive(v_net, self.rcv.clone());

        let spend = self.spend.into_pczt(&mut rng);
        let output = self.output.into_pczt(&cv_net, spend.nullifier, &mut rng);

        crate::pczt::Action {
            cv_net,
            spend,
            output,
            rcv: Some(self.rcv),
        }
    }
}

/// Type alias for an in-progress bundle that has no proofs or signatures.
///
/// This is returned by [`Builder::build`].
#[cfg(feature = "circuit")]
pub type UnauthorizedBundle<V> = Bundle<InProgress<Unproven, Unauthorized>, V>;

/// Metadata about a bundle created by [`bundle`] or [`Builder::build`] that is not necessarily
/// recoverable from the bundle itself.
///
/// This includes information about how [`Action`]s within the bundle are ordered (after
/// padding and randomization) relative to the order in which spends and outputs were provided
/// (to [`bundle`]), or the order in which [`Builder`] mutations were performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleMetadata {
    spend_indices: Vec<usize>,
    output_indices: Vec<usize>,
}

impl BundleMetadata {
    fn new(num_requested_spends: usize, num_requested_outputs: usize) -> Self {
        BundleMetadata {
            spend_indices: vec![0; num_requested_spends],
            output_indices: vec![0; num_requested_outputs],
        }
    }

    /// Returns the metadata for a [`Bundle`] that contains only dummy actions, if any.
    pub fn empty() -> Self {
        Self::new(0, 0)
    }

    /// Returns the index within the bundle of the [`Action`] corresponding to the `n`-th
    /// spend specified in bundle construction. If a [`Builder`] was used, this refers to
    /// the spend added by the `n`-th call to [`Builder::add_spend`].
    ///
    /// For the purpose of improving indistinguishability, actions are padded and note
    /// positions are randomized when building bundles. This means that the bundle
    /// consumer cannot assume that e.g. the first spend they added corresponds to the
    /// first action in the bundle. In a bundle that disables cross-address transfers,
    /// each spend's action contains a fabricated zero-value output (to the spent note's
    /// own address), so no requested output shares the returned action index.
    pub fn spend_action_index(&self, n: usize) -> Option<usize> {
        self.spend_indices.get(n).copied()
    }

    /// Returns the index within the bundle of the [`Action`] corresponding to the `n`-th
    /// output specified in bundle construction. If a [`Builder`] was used, this refers to
    /// the output added by the `n`-th call to [`Builder::add_output`] or
    /// [`Builder::add_change_output`].
    ///
    /// For the purpose of improving indistinguishability, actions are padded and note
    /// positions are randomized when building bundles. This means that the bundle
    /// consumer cannot assume that e.g. the first output they added corresponds to the
    /// first action in the bundle. In a bundle that disables cross-address transfers,
    /// each output's action contains a fabricated wallet-controlled zero-value spend (at
    /// the change address), so no requested spend shares the returned action index.
    pub fn output_action_index(&self, n: usize) -> Option<usize> {
        self.output_indices.get(n).copied()
    }
}

/// A builder that constructs a [`Bundle`] from a set of notes to be spent, and outputs
/// to receive funds.
#[derive(Debug)]
pub struct Builder {
    spends: Vec<SpendInfo>,
    outputs: Vec<OutputInfo>,
    bundle_type: BundleType,
    anchor: Anchor,
}

impl Builder {
    /// Constructs a new empty builder for an Orchard bundle.
    pub fn new(bundle_type: BundleType, anchor: Anchor) -> Self {
        Builder {
            spends: vec![],
            outputs: vec![],
            bundle_type,
            anchor,
        }
    }

    /// Adds a note to be spent in this transaction.
    ///
    /// - `note` is a spendable note, obtained by trial-decrypting an [`Action`] using the
    ///   [`zcash_note_encryption`] crate instantiated with [`OrchardDomain`].
    /// - `merkle_path` can be obtained using the [`incrementalmerkletree`] crate
    ///   instantiated with [`MerkleHashOrchard`].
    ///
    /// Returns an error if the given Merkle path does not have the required anchor for
    /// the given note.
    ///
    /// In a bundle that disables cross-address transfers, each spend is paired with a
    /// fabricated zero-value output encrypted to the spent note's own address. The
    /// wallet that owns the spent note **will** trial-decrypt that output when scanning
    /// the chain, as a zero-value note with an all-zero memo field; wallets should
    /// expect and tolerate these notes.
    ///
    /// [`OrchardDomain`]: crate::note_encryption::OrchardDomain
    /// [`MerkleHashOrchard`]: crate::tree::MerkleHashOrchard
    pub fn add_spend(
        &mut self,
        fvk: FullViewingKey,
        note: Note,
        merkle_path: MerklePath,
    ) -> Result<(), SpendError> {
        let flags = self.bundle_type.flags();
        if !flags.spends_enabled() {
            return Err(SpendError::SpendsDisabled);
        }

        let spend = SpendInfo::new(fvk, note, merkle_path).ok_or(SpendError::FvkMismatch)?;

        // Consistency check: all anchors must be equal.
        if !spend.has_matching_anchor(&self.anchor) {
            return Err(SpendError::AnchorMismatch);
        }

        self.spends.push(spend);

        Ok(())
    }

    /// Adds an address which will receive funds in this transaction.
    ///
    /// In a bundle that disables cross-address transfers, ordinary outputs cannot be
    /// constructed (each action's output is addressed to the note it spends); retained
    /// value must be added with [`Builder::add_change_output`] instead.
    pub fn add_output(
        &mut self,
        ovk: Option<OutgoingViewingKey>,
        recipient: Address,
        value: NoteValue,
        memo: [u8; 512],
    ) -> Result<(), OutputError> {
        let flags = self.bundle_type.flags();
        if !flags.outputs_enabled() {
            return Err(OutputError::OutputsDisabled);
        }
        if flags.disable_cross_address() {
            return Err(OutputError::CrossAddressDisabled);
        }

        self.outputs
            .push(OutputInfo::new(ovk, recipient, value, memo));

        Ok(())
    }

    /// Adds a wallet-controlled change output, to an address owned by `fvk`.
    ///
    /// This is the only way to retain shielded value in a bundle that disables
    /// cross-address transfers: the builder pairs the change output with a fabricated
    /// zero-value spend at `recipient`, controlled by `fvk`, in the same action.
    /// (Withdrawals leave such a bundle through its positive value balance; its real
    /// spends are each paired with a fabricated zero-value output to the spent note's
    /// own address.) The fabricated spend's authorization is produced by the normal
    /// signing flow -- [`Bundle::apply_signatures`] with the [`SpendAuthorizingKey`]
    /// matching `fvk` -- exactly like the bundle's real spends.
    ///
    /// This may also be used in bundles that permit cross-address transfers, where it
    /// behaves like [`Builder::add_output`] plus an ownership check, so wallet change
    /// logic can be uniform across bundle kinds.
    ///
    /// Note that the builder does not constrain the sign of the bundle's value balance:
    /// a bundle that disables cross-address transfers can still have a negative balance
    /// (value entering the Orchard pool from the rest of the transaction). Whether such
    /// bundles are acceptable is a transaction-level concern outside this crate.
    ///
    /// Returns an error if outputs are disabled for this builder's bundle type, or if
    /// `fvk` does not own `recipient`.
    pub fn add_change_output(
        &mut self,
        fvk: FullViewingKey,
        ovk: Option<OutgoingViewingKey>,
        recipient: Address,
        value: NoteValue,
        memo: [u8; 512],
    ) -> Result<(), OutputError> {
        let flags = self.bundle_type.flags();
        if !flags.outputs_enabled() {
            return Err(OutputError::OutputsDisabled);
        }

        let output =
            OutputInfo::change(fvk, ovk, recipient, value, memo).ok_or(OutputError::FvkMismatch)?;
        self.outputs.push(output);

        Ok(())
    }

    /// Returns the action spend components that will be produced by the
    /// transaction being constructed
    pub fn spends(&self) -> &Vec<impl InputView<()>> {
        &self.spends
    }

    /// Returns the action output components that will be produced by the
    /// transaction being constructed
    pub fn outputs(&self) -> &Vec<impl OutputView> {
        &self.outputs
    }

    /// The net value of the bundle to be built. The value of all spends,
    /// minus the value of all outputs.
    ///
    /// Useful for balancing a transaction, as the value balance of an individual bundle
    /// can be non-zero. Each bundle's value balance is [added] to the transparent
    /// transaction value pool, which [must not have a negative value]. (If it were
    /// negative, the transaction would output more value than it receives in inputs.)
    ///
    /// [added]: https://zips.z.cash/protocol/protocol.pdf#orchardbalance
    /// [must not have a negative value]: https://zips.z.cash/protocol/protocol.pdf#transactions
    pub fn value_balance<V: TryFrom<i64>>(&self) -> Result<V, value::BalanceError> {
        let value_balance = self
            .spends
            .iter()
            .map(|spend| spend.note.value() - NoteValue::ZERO)
            .chain(
                self.outputs
                    .iter()
                    .map(|output| NoteValue::ZERO - output.value),
            )
            .try_fold(ValueSum::zero(), |acc, note_value| acc + note_value)
            .ok_or(BalanceError::Overflow)?;
        i64::try_from(value_balance)
            .and_then(|i| V::try_from(i).map_err(|_| value::BalanceError::Overflow))
    }

    /// Builds a bundle containing the given spent notes and outputs for a given circuit version.
    ///
    /// The returned bundle will have no proof or signatures; these can be applied with
    /// [`Bundle::create_proof`] and [`Bundle::apply_signatures`] respectively.
    /// See [`OrchardCircuitVersion`] for which version to use.
    #[cfg(feature = "circuit")]
    pub fn build<V: TryFrom<i64>>(
        self,
        rng: impl RngCore,
        circuit_version: OrchardCircuitVersion,
    ) -> Result<Option<(UnauthorizedBundle<V>, BundleMetadata)>, BuildError> {
        let anchor = self.anchor;
        let bundle_type = self.bundle_type;
        let spends = self.spends;
        let outputs = self.outputs;

        build_bundle(
            rng,
            anchor,
            bundle_type,
            spends,
            outputs,
            |pre_actions, flags, value_balance, bundle_meta, rng| {
                finish_unauthorized_bundle(
                    pre_actions,
                    flags,
                    value_balance,
                    bundle_meta,
                    rng,
                    anchor,
                    circuit_version,
                )
            },
        )
    }

    /// Builds a bundle containing the given spent notes and outputs along with their
    /// metadata, for inclusion in a PCZT.
    pub fn build_for_pczt(
        self,
        rng: impl RngCore,
    ) -> Result<(crate::pczt::Bundle, BundleMetadata), BuildError> {
        let anchor = self.anchor;
        let bundle_type = self.bundle_type;
        let spends = self.spends;
        let outputs = self.outputs;

        build_bundle(
            rng,
            anchor,
            bundle_type,
            spends,
            outputs,
            |pre_actions, flags, value_sum, bundle_meta, mut rng| {
                // Create the actions.
                let actions = pre_actions
                    .into_iter()
                    .map(|a| a.build_for_pczt(&mut rng))
                    .collect::<Vec<_>>();

                Ok((
                    crate::pczt::Bundle {
                        actions,
                        flags,
                        value_sum,
                        anchor,
                        zkproof: None,
                        bsk: None,
                    },
                    bundle_meta,
                ))
            },
        )
    }
}

/// Builds a bundle containing the given spent notes and outputs, with the Action circuits built
/// for the given `circuit_version`.
///
/// See [`OrchardCircuitVersion`] for which version to use.
#[cfg(feature = "circuit")]
pub fn bundle<V: TryFrom<i64>>(
    rng: impl RngCore,
    anchor: Anchor,
    bundle_type: BundleType,
    spends: Vec<SpendInfo>,
    outputs: Vec<OutputInfo>,
    circuit_version: OrchardCircuitVersion,
) -> Result<Option<(UnauthorizedBundle<V>, BundleMetadata)>, BuildError> {
    build_bundle(
        rng,
        anchor,
        bundle_type,
        spends,
        outputs,
        |pre_actions, flags, value_balance, bundle_meta, rng| {
            finish_unauthorized_bundle(
                pre_actions,
                flags,
                value_balance,
                bundle_meta,
                rng,
                anchor,
                circuit_version,
            )
        },
    )
}

#[cfg(feature = "circuit")]
fn finish_unauthorized_bundle<V: TryFrom<i64>, R: RngCore>(
    pre_actions: Vec<ActionInfo>,
    flags: Flags,
    value_balance: ValueSum,
    bundle_meta: BundleMetadata,
    mut rng: R,
    anchor: Anchor,
    circuit_version: OrchardCircuitVersion,
) -> Result<Option<(UnauthorizedBundle<V>, BundleMetadata)>, BuildError> {
    let result_value_balance: V = i64::try_from(value_balance)
        .map_err(BuildError::ValueSum)
        .and_then(|i| {
            V::try_from(i).map_err(|_| BuildError::ValueSum(value::BalanceError::Overflow))
        })?;

    // Compute the transaction binding signing key.
    let bsk = pre_actions
        .iter()
        .map(|a| &a.rcv)
        .sum::<ValueCommitTrapdoor>()
        .into_bsk();

    // Create the actions.
    let (actions, circuits): (Vec<_>, Vec<_>) = pre_actions
        .into_iter()
        .map(|a| a.build(&mut rng, circuit_version))
        .unzip();

    // Verify that bsk and bvk are consistent.
    let bvk = (actions.iter().map(|a| a.cv_net()).sum::<ValueCommitment>()
        - ValueCommitment::derive(value_balance, ValueCommitTrapdoor::zero()))
    .into_bvk();
    assert_eq!(redpallas::VerificationKey::from(&bsk), bvk);

    Ok(NonEmpty::from_vec(actions).map(|actions| {
        (
            Bundle::from_parts_unchecked(
                actions,
                flags,
                result_value_balance,
                anchor,
                InProgress {
                    proof: Unproven {
                        circuits,
                        circuit_version,
                    },
                    sigs: Unauthorized { bsk },
                },
            ),
            bundle_meta,
        )
    }))
}

fn build_bundle<B, R: RngCore>(
    mut rng: R,
    anchor: Anchor,
    bundle_type: BundleType,
    spends: Vec<SpendInfo>,
    outputs: Vec<OutputInfo>,
    finisher: impl FnOnce(Vec<ActionInfo>, Flags, ValueSum, BundleMetadata, R) -> Result<B, BuildError>,
) -> Result<B, BuildError> {
    let flags = bundle_type.flags();

    let num_requested_spends = spends.len();
    if !flags.spends_enabled() && num_requested_spends > 0 {
        return Err(BuildError::SpendsDisabled);
    }

    for spend in &spends {
        if !spend.has_matching_anchor(&anchor) {
            return Err(BuildError::AnchorMismatch);
        }
    }

    let num_requested_outputs = outputs.len();
    if !flags.outputs_enabled() && num_requested_outputs > 0 {
        return Err(BuildError::OutputsDisabled);
    }

    let num_actions = bundle_type
        .num_actions(num_requested_spends, num_requested_outputs)
        .map_err(|_| BuildError::BundleTypeNotSatisfiable)?;

    let (pre_actions, bundle_meta) = if flags.disable_cross_address() {
        // Every action's output must be addressed to the note it spends, so the
        // spend/output pairing within each action is intentional:
        //
        // - each requested spend is paired with a fabricated zero-value output to the
        //   spent note's own address;
        // - each requested (wallet-controlled change) output is paired with a fabricated
        //   zero-value spend controlled by the wallet at the change address, because
        //   withdrawn value leaves the bundle through its value balance, and retained
        //   value is exactly the wallet's change;
        // - padding actions pair a dummy spend with a zero-value output to the dummy's
        //   own address, since the cross-address checks apply to dummy actions too.
        //
        // Only complete pairs are shuffled.
        let mut pairs = Vec::with_capacity(num_actions);

        for (spend_idx, spend) in spends.into_iter().enumerate() {
            let output = OutputInfo::new(None, spend.note.recipient(), NoteValue::ZERO, [0u8; 512]);
            pairs.push((Some(spend_idx), None, spend, output));
        }

        for (out_idx, mut output) in outputs.into_iter().enumerate() {
            let (fvk, scope) = output
                .change_fvk
                .take()
                .ok_or(BuildError::CrossAddressDisabled)?;
            let rho = Rho::from_nf_old(Nullifier::dummy(&mut rng));
            let note = Note::new(output.recipient, NoteValue::ZERO, rho, &mut rng);
            let spend = SpendInfo {
                // The wallet controls this spend: it is signed through the normal
                // signing flow, by the spend authorizing key matching `fvk`.
                dummy_sk: None,
                fvk,
                scope,
                note,
                merkle_path: MerklePath::dummy(&mut rng),
            };
            pairs.push((None, Some(out_idx), spend, output));
        }

        while pairs.len() < num_actions {
            let spend = SpendInfo::dummy(&mut rng);
            let output = OutputInfo::new(None, spend.note.recipient(), NoteValue::ZERO, [0u8; 512]);
            pairs.push((None, None, spend, output));
        }

        // Shuffle the action pairs, so that learning the position of a specific action
        // doesn't reveal anything on its own about its meaning in the transaction
        // context. The spend/output pairing inside each action is intentional.
        pairs.shuffle(&mut rng);

        let mut bundle_meta = BundleMetadata::new(num_requested_spends, num_requested_outputs);
        let pre_actions = pairs
            .into_iter()
            .enumerate()
            .map(|(action_idx, (spend_idx, out_idx, spend, output))| {
                // Record the post-randomization spend location
                if let Some(spend_idx) = spend_idx {
                    bundle_meta.spend_indices[spend_idx] = action_idx;
                }

                // Record the post-randomization output location
                if let Some(out_idx) = out_idx {
                    bundle_meta.output_indices[out_idx] = action_idx;
                }

                debug_assert!(
                    spend.note.recipient().same_receiver(&output.recipient),
                    "cross-address-disabled actions pair a spend with an output to the \
                     same receiver by construction",
                );

                ActionInfo::new(spend, output, &mut rng)
            })
            .collect::<Vec<_>>();

        (pre_actions, bundle_meta)
    } else {
        // Pair up the spends and outputs, extending with dummy values as necessary.
        let mut indexed_spends = spends
            .into_iter()
            .chain(iter::repeat_with(|| SpendInfo::dummy(&mut rng)))
            .enumerate()
            .take(num_actions)
            .collect::<Vec<_>>();

        let mut indexed_outputs = outputs
            .into_iter()
            .chain(iter::repeat_with(|| OutputInfo::dummy(&mut rng)))
            .enumerate()
            .take(num_actions)
            .collect::<Vec<_>>();

        // Shuffle the spends and outputs, so that learning the position of a
        // specific spent note or output note doesn't reveal anything on its own about
        // the meaning of that note in the transaction context.
        indexed_spends.shuffle(&mut rng);
        indexed_outputs.shuffle(&mut rng);

        let mut bundle_meta = BundleMetadata::new(num_requested_spends, num_requested_outputs);
        let pre_actions = indexed_spends
            .into_iter()
            .zip(indexed_outputs)
            .enumerate()
            .map(|(action_idx, ((spend_idx, spend), (out_idx, output)))| {
                // Record the post-randomization spend location
                if spend_idx < num_requested_spends {
                    bundle_meta.spend_indices[spend_idx] = action_idx;
                }

                // Record the post-randomization output location
                if out_idx < num_requested_outputs {
                    bundle_meta.output_indices[out_idx] = action_idx;
                }

                ActionInfo::new(spend, output, &mut rng)
            })
            .collect::<Vec<_>>();

        (pre_actions, bundle_meta)
    };

    // Determine the value balance for this bundle, ensuring it is valid.
    let value_balance = pre_actions
        .iter()
        .try_fold(ValueSum::zero(), |acc, action| acc + action.value_sum())
        .ok_or(BalanceError::Overflow)?;

    finisher(pre_actions, flags, value_balance, bundle_meta, rng)
}

/// Marker trait representing bundle signatures in the process of being created.
pub trait InProgressSignatures: fmt::Debug {
    /// The authorization type of an Orchard action in the process of being authorized.
    type SpendAuth: fmt::Debug;
}

/// Marker for a bundle in the process of being built.
#[derive(Clone, Debug)]
pub struct InProgress<P, S: InProgressSignatures> {
    proof: P,
    sigs: S,
}

impl<P: fmt::Debug, S: InProgressSignatures> Authorization for InProgress<P, S> {
    type SpendAuth = S::SpendAuth;
}

/// Marker for a bundle without a proof.
///
/// This struct contains the private data needed to create a [`Proof`] for a [`Bundle`].
#[cfg(feature = "circuit")]
#[derive(Clone, Debug)]
pub struct Unproven {
    circuits: Vec<Circuit>,
    circuit_version: OrchardCircuitVersion,
}

#[cfg(feature = "circuit")]
impl<S: InProgressSignatures> InProgress<Unproven, S> {
    /// Creates the proof for this bundle.
    pub fn create_proof(
        &self,
        pk: &ProvingKey,
        instances: &[Instance],
        rng: impl RngCore,
    ) -> Result<Proof, halo2_proofs::plonk::Error> {
        Proof::create(pk, &self.proof.circuits, instances, rng)
    }
}

#[cfg(feature = "circuit")]
impl<S: InProgressSignatures, V> Bundle<InProgress<Unproven, S>, V> {
    /// The circuit version this bundle's actions were built for, and that its proof must
    /// therefore be created against (with a matching [`ProvingKey`]).
    pub fn circuit_version(&self) -> OrchardCircuitVersion {
        self.authorization().proof.circuit_version
    }

    /// Creates the proof for this bundle.
    pub fn create_proof(
        self,
        pk: &ProvingKey,
        mut rng: impl RngCore,
    ) -> Result<Bundle<InProgress<Proof, S>, V>, BuildError> {
        let instances: Vec<_> = self
            .actions()
            .iter()
            .map(|a| a.to_instance(*self.flags(), *self.anchor()))
            .collect();
        self.try_map_authorization(
            &mut (),
            |_, _, a| Ok(a),
            |_, auth| {
                let proof = auth.create_proof(pk, &instances, &mut rng)?;
                Ok(InProgress {
                    proof,
                    sigs: auth.sigs,
                })
            },
        )
    }
}

/// The parts needed to sign an [`Action`].
#[derive(Clone, Debug)]
pub struct SigningParts {
    /// The spend validating key for this action. Used to match spend authorizing keys to
    /// actions they can create signatures for.
    ak: SpendValidatingKey,
    /// The randomization needed to derive the actual signing key for this note.
    alpha: pallas::Scalar,
}

/// Marker for an unauthorized bundle with no signatures.
#[derive(Clone, Debug)]
pub struct Unauthorized {
    bsk: redpallas::SigningKey<Binding>,
}

impl InProgressSignatures for Unauthorized {
    type SpendAuth = SigningMetadata;
}

/// Container for metadata needed to sign an [`Action`].
#[derive(Clone, Debug)]
pub struct SigningMetadata {
    /// If this action is spending a dummy note, this field holds that note's spend
    /// authorizing key.
    ///
    /// These keys are used automatically in [`Bundle<Unauthorized>::prepare`] or
    /// [`Bundle<Unauthorized>::apply_signatures`] to sign dummy spends.
    dummy_ask: Option<SpendAuthorizingKey>,
    parts: SigningParts,
}

/// Marker for a partially-authorized bundle, in the process of being signed.
#[derive(Debug)]
pub struct PartiallyAuthorized {
    binding_signature: redpallas::Signature<Binding>,
    sighash: [u8; 32],
}

impl InProgressSignatures for PartiallyAuthorized {
    type SpendAuth = MaybeSigned;
}

/// A heisen[`Signature`] for a particular [`Action`].
///
/// [`Signature`]: redpallas::Signature
#[derive(Debug)]
pub enum MaybeSigned {
    /// The information needed to sign this [`Action`].
    SigningMetadata(SigningParts),
    /// The signature for this [`Action`].
    Signature(redpallas::Signature<SpendAuth>),
}

impl MaybeSigned {
    fn finalize(self) -> Result<redpallas::Signature<SpendAuth>, BuildError> {
        match self {
            Self::Signature(sig) => Ok(sig),
            _ => Err(BuildError::MissingSignatures),
        }
    }
}

impl<P: fmt::Debug, V> Bundle<InProgress<P, Unauthorized>, V> {
    /// Loads the sighash into this bundle, preparing it for signing.
    ///
    /// This API ensures that all signatures are created over the same sighash.
    pub fn prepare<R: RngCore + CryptoRng>(
        self,
        mut rng: R,
        sighash: [u8; 32],
    ) -> Bundle<InProgress<P, PartiallyAuthorized>, V> {
        self.map_authorization(
            &mut rng,
            |rng, _, SigningMetadata { dummy_ask, parts }| {
                // We can create signatures for dummy spends immediately.
                dummy_ask
                    .map(|ask| ask.randomize(&parts.alpha).sign(rng, &sighash))
                    .map(MaybeSigned::Signature)
                    .unwrap_or(MaybeSigned::SigningMetadata(parts))
            },
            |rng, auth| InProgress {
                proof: auth.proof,
                sigs: PartiallyAuthorized {
                    binding_signature: auth.sigs.bsk.sign(rng, &sighash),
                    sighash,
                },
            },
        )
    }
}

impl<V> Bundle<InProgress<Proof, Unauthorized>, V> {
    /// Applies signatures to this bundle, in order to authorize it.
    ///
    /// This is a helper method that wraps [`Bundle::prepare`], [`Bundle::sign`], and
    /// [`Bundle::finalize`].
    pub fn apply_signatures<R: RngCore + CryptoRng>(
        self,
        mut rng: R,
        sighash: [u8; 32],
        signing_keys: &[SpendAuthorizingKey],
    ) -> Result<Bundle<Authorized, V>, BuildError> {
        signing_keys
            .iter()
            .fold(self.prepare(&mut rng, sighash), |partial, ask| {
                partial.sign(&mut rng, ask)
            })
            .finalize()
    }
}

impl<P: fmt::Debug, V> Bundle<InProgress<P, PartiallyAuthorized>, V> {
    /// Signs this bundle with the given [`SpendAuthorizingKey`].
    ///
    /// This will apply signatures for all notes controlled by this spending key.
    pub fn sign<R: RngCore + CryptoRng>(self, mut rng: R, ask: &SpendAuthorizingKey) -> Self {
        let expected_ak = ask.into();
        self.map_authorization(
            &mut rng,
            |rng, partial, maybe| match maybe {
                MaybeSigned::SigningMetadata(parts) if parts.ak == expected_ak => {
                    MaybeSigned::Signature(
                        ask.randomize(&parts.alpha).sign(rng, &partial.sigs.sighash),
                    )
                }
                s => s,
            },
            |_, partial| partial,
        )
    }
    /// Appends externally computed [`Signature`]s.
    ///
    /// Each signature will be applied to the one input for which it is valid. An error
    /// will be returned if the signature is not valid for any inputs, or if it is valid
    /// for more than one input.
    ///
    /// [`Signature`]: redpallas::Signature
    pub fn append_signatures(
        self,
        signatures: &[redpallas::Signature<SpendAuth>],
    ) -> Result<Self, BuildError> {
        signatures.iter().try_fold(self, Self::append_signature)
    }

    fn append_signature(
        self,
        signature: &redpallas::Signature<SpendAuth>,
    ) -> Result<Self, BuildError> {
        let mut signature_valid_for = 0usize;
        let bundle = self.map_authorization(
            &mut signature_valid_for,
            |valid_for, partial, maybe| match maybe {
                MaybeSigned::SigningMetadata(parts) => {
                    let rk = parts.ak.randomize(&parts.alpha);
                    if rk.verify(&partial.sigs.sighash[..], signature).is_ok() {
                        *valid_for += 1;
                        MaybeSigned::Signature(signature.clone())
                    } else {
                        // Signature isn't for this input.
                        MaybeSigned::SigningMetadata(parts)
                    }
                }
                s => s,
            },
            |_, partial| partial,
        );
        match signature_valid_for {
            0 => Err(BuildError::InvalidExternalSignature),
            1 => Ok(bundle),
            _ => Err(BuildError::DuplicateSignature),
        }
    }
}

impl<V> Bundle<InProgress<Proof, PartiallyAuthorized>, V> {
    /// Finalizes this bundle, enabling it to be included in a transaction.
    ///
    /// Returns an error if any signatures are missing.
    pub fn finalize(self) -> Result<Bundle<Authorized, V>, BuildError> {
        self.try_map_authorization(
            &mut (),
            |_, _, maybe| maybe.finalize(),
            |_, partial| {
                Ok(Authorized::from_parts(
                    partial.proof,
                    partial.sigs.binding_signature,
                ))
            },
        )
    }
}

/// A trait that provides a minimized view of an Orchard input suitable for use in
/// fee and change calculation.
pub trait InputView<NoteRef> {
    /// An identifier for the input being spent.
    fn note_id(&self) -> &NoteRef;
    /// The value of the input being spent.
    fn value<V: From<u64>>(&self) -> V;
}

impl InputView<()> for SpendInfo {
    fn note_id(&self) -> &() {
        // The builder does not make use of note identifiers, so we can just return the unit value.
        &()
    }

    fn value<V: From<u64>>(&self) -> V {
        V::from(self.note.value().inner())
    }
}

/// A trait that provides a minimized view of an Orchard output suitable for use in
/// fee and change calculation.
pub trait OutputView {
    /// The value of the output being produced.
    fn value<V: From<u64>>(&self) -> V;
}

impl OutputView for OutputInfo {
    fn value<V: From<u64>>(&self) -> V {
        V::from(self.value.inner())
    }
}

/// Generators for property testing.
#[cfg(all(feature = "circuit", any(test, feature = "test-dependencies")))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use alloc::vec::Vec;
    use core::fmt::Debug;

    use incrementalmerkletree::{frontier::Frontier, Hashable};
    use rand::{rngs::StdRng, CryptoRng, SeedableRng};

    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::{
        address::testing::arb_address,
        bundle::{Authorized, Bundle},
        circuit::{OrchardCircuitVersion, ProvingKey},
        keys::{testing::arb_spending_key, FullViewingKey, SpendAuthorizingKey, SpendingKey},
        note::testing::arb_note,
        tree::{Anchor, MerkleHashOrchard, MerklePath},
        value::{testing::arb_positive_note_value, NoteValue, MAX_NOTE_VALUE},
        Address, Note,
    };

    use super::{Builder, BundleType};

    /// An intermediate type used for construction of arbitrary
    /// bundle values. This type is required because of a limitation
    /// of the proptest prop_compose! macro which does not correctly
    /// handle polymorphic generator functions. Instead of generating
    /// a bundle directly, we generate the bundle inputs, and then
    /// are able to use the `build` function to construct the bundle
    /// from these inputs, but using a `ValueBalance` implementation that
    /// is defined by the end user.
    #[derive(Debug)]
    struct ArbitraryBundleInputs<R> {
        rng: R,
        sk: SpendingKey,
        anchor: Anchor,
        notes: Vec<(Note, MerklePath)>,
        output_amounts: Vec<(Address, NoteValue)>,
    }

    impl<R: RngCore + CryptoRng> ArbitraryBundleInputs<R> {
        /// Create a bundle from the set of arbitrary bundle inputs.
        fn into_bundle<V: TryFrom<i64>>(mut self) -> Bundle<Authorized, V> {
            let fvk = FullViewingKey::from(&self.sk);
            let mut builder = Builder::new(BundleType::DEFAULT, self.anchor);

            for (note, path) in self.notes.into_iter() {
                builder.add_spend(fvk.clone(), note, path).unwrap();
            }

            for (addr, value) in self.output_amounts.into_iter() {
                let scope = fvk.scope_for_address(&addr).unwrap();
                let ovk = fvk.to_ovk(scope);

                builder
                    .add_output(Some(ovk.clone()), addr, value, [0u8; 512])
                    .unwrap();
            }

            let pk = ProvingKey::build(OrchardCircuitVersion::FixedPostNu6_2);
            builder
                .build(&mut self.rng, OrchardCircuitVersion::FixedPostNu6_2)
                .unwrap()
                .unwrap()
                .0
                .create_proof(&pk, &mut self.rng)
                .unwrap()
                .prepare(&mut self.rng, [0; 32])
                .sign(&mut self.rng, &SpendAuthorizingKey::from(&self.sk))
                .finalize()
                .unwrap()
        }
    }

    prop_compose! {
        /// Produce a random valid Orchard bundle.
        fn arb_bundle_inputs(sk: SpendingKey)
        (
            n_notes in 1usize..30,
            n_outputs in 1..30,

        )
        (
            // generate note values that we're certain won't exceed MAX_NOTE_VALUE in total
            notes in vec(
                arb_positive_note_value(MAX_NOTE_VALUE / n_notes as u64).prop_flat_map(arb_note),
                n_notes
            ),
            output_amounts in vec(
                arb_address().prop_flat_map(move |a| {
                    arb_positive_note_value(MAX_NOTE_VALUE / n_outputs as u64)
                        .prop_map(move |v| (a, v))
                }),
                n_outputs as usize
            ),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY)
        ) -> ArbitraryBundleInputs<StdRng> {
            use crate::constants::MERKLE_DEPTH_ORCHARD;
            let mut frontier = Frontier::<MerkleHashOrchard, { MERKLE_DEPTH_ORCHARD as u8 }>::empty();
            let mut notes_and_auth_paths: Vec<(Note, MerklePath)> = Vec::new();

            for note in notes.iter() {
                let leaf = MerkleHashOrchard::from_cmx(&note.commitment().into());
                frontier.append(leaf);

                let path = frontier
                    .witness(|addr| Some(<MerkleHashOrchard as Hashable>::empty_root(addr.level())))
                    .ok()
                    .flatten()
                    .expect("we can always construct a correct Merkle path");
                notes_and_auth_paths.push((*note, path.into()));
            }

            ArbitraryBundleInputs {
                rng: StdRng::from_seed(rng_seed),
                sk,
                anchor: frontier.root().into(),
                notes: notes_and_auth_paths,
                output_amounts
            }
        }
    }

    /// Produce an arbitrary valid Orchard bundle using a random spending key.
    pub fn arb_bundle<V: TryFrom<i64> + Debug>() -> impl Strategy<Value = Bundle<Authorized, V>> {
        arb_spending_key()
            .prop_flat_map(arb_bundle_inputs)
            .prop_map(|inputs| inputs.into_bundle::<V>())
    }

    /// Produce an arbitrary valid Orchard bundle using a specified spending key.
    pub fn arb_bundle_with_key<V: TryFrom<i64> + Debug>(
        k: SpendingKey,
    ) -> impl Strategy<Value = Bundle<Authorized, V>> {
        arb_bundle_inputs(k).prop_map(|inputs| inputs.into_bundle::<V>())
    }
}

#[cfg(all(test, feature = "circuit"))]
mod tests {
    use rand::rngs::OsRng;
    use rand::RngCore;

    use super::{bundle, BuildError, Builder, MaybeSigned, OutputError, OutputInfo};
    use crate::{
        builder::BundleType,
        bundle::{Authorized, Bundle, Flags},
        circuit::{OrchardCircuitVersion, ProvingKey},
        constants::MERKLE_DEPTH_ORCHARD,
        keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
        note::{Nullifier, Rho},
        pczt::{ProverError, VerifyError},
        tree::{MerklePath, EMPTY_ROOTS},
        value::NoteValue,
        Address, Anchor, Note,
    };

    fn note_with_path(
        rng: &mut impl RngCore,
        recipient: Address,
        value: NoteValue,
    ) -> (Note, MerklePath, Anchor) {
        let rho = Rho::from_nf_old(Nullifier::dummy(rng));
        let note = Note::new(recipient, value, rho, &mut *rng);
        let merkle_path = MerklePath::dummy(rng);
        let anchor = merkle_path.root(note.commitment().into());

        (note, merkle_path, anchor)
    }

    fn restricted_bundle_type(bundle_required: bool) -> BundleType {
        BundleType::Transactional {
            flags: Flags::CROSS_ADDRESS_DISABLED,
            bundle_required,
        }
    }

    /// Creates a builder of the given bundle type over the empty-tree anchor, with a
    /// single 5000-zat output to a freshly derived external address.
    fn output_only_builder(rng: &mut impl RngCore, bundle_type: BundleType) -> Builder {
        let sk = SpendingKey::random(rng);
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        let mut builder = Builder::new(bundle_type, EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into());
        builder
            .add_output(None, recipient, NoteValue::from_raw(5000), [0u8; 512])
            .expect("output-only builders accept ordinary outputs");
        builder
    }

    #[test]
    fn shielding_bundle() {
        let pk = ProvingKey::build(OrchardCircuitVersion::FixedPostNu6_2);
        let mut rng = OsRng;

        let builder = output_only_builder(&mut rng, BundleType::DEFAULT);
        let balance: i64 = builder.value_balance().unwrap();
        assert_eq!(balance, -5000);

        let bundle: Bundle<Authorized, i64> = builder
            .build(&mut rng, OrchardCircuitVersion::FixedPostNu6_2)
            .unwrap()
            .unwrap()
            .0
            .create_proof(&pk, &mut rng)
            .unwrap()
            .prepare(rng, [0; 32])
            .finalize()
            .unwrap();
        assert_eq!(bundle.value_balance(), &(-5000))
    }

    #[test]
    fn coinbase_bundle_builds_for_ironwood() {
        let mut rng = OsRng;

        // Coinbase bundles never set `disableCrossAddress`, so under the Ironwood
        // circuit version they serve a pool that accepts unrestricted
        // (`disableCrossAddress = 0`) bundles. A pool whose rules require
        // `disableCrossAddress = 1` on every bundle prohibits coinbase entirely;
        // that prohibition is a consensus rule outside this crate.
        let builder = output_only_builder(&mut rng, BundleType::Coinbase);

        let (bundle, _) = builder
            .build::<i64>(&mut rng, OrchardCircuitVersion::Ironwood)
            .expect("coinbase bundles build under the Ironwood circuit version")
            .expect("a bundle is produced for the requested output");
        assert_eq!(bundle.actions().len(), 1);
        assert_eq!(bundle.circuit_version(), OrchardCircuitVersion::Ironwood);
        assert!(!bundle.flags().spends_enabled());
        assert!(bundle.flags().outputs_enabled());
        assert!(!bundle.flags().disable_cross_address());
    }

    #[test]
    fn coinbase_bundle_type_uses_spends_disabled_flags() {
        assert_eq!(BundleType::Coinbase.flags(), Flags::SPENDS_DISABLED);
        assert!(!BundleType::Coinbase.flags().disable_cross_address());
    }

    #[test]
    fn cross_address_disabled_builder_pairs_actions() {
        let mut rng = OsRng;
        let spend_sk = SpendingKey::random(&mut rng);
        let spend_fvk = FullViewingKey::from(&spend_sk);
        let spend_recipient = spend_fvk.address_at(0u32, Scope::External);
        let change_sk = SpendingKey::random(&mut rng);
        let change_fvk = FullViewingKey::from(&change_sk);
        let change_recipient = change_fvk.address_at(0u32, Scope::Internal);
        let (note, merkle_path, anchor) =
            note_with_path(&mut rng, spend_recipient, NoteValue::from_raw(15_000));

        let mut builder = Builder::new(restricted_bundle_type(false), anchor);
        assert_eq!(
            builder.add_output(
                None,
                change_recipient,
                NoteValue::from_raw(5_000),
                [0u8; 512]
            ),
            Err(OutputError::CrossAddressDisabled)
        );
        assert_eq!(
            builder.add_change_output(
                FullViewingKey::from(&SpendingKey::random(&mut rng)),
                None,
                change_recipient,
                NoteValue::from_raw(5_000),
                [0u8; 512],
            ),
            Err(OutputError::FvkMismatch)
        );

        builder
            .add_spend(spend_fvk.clone(), note, merkle_path)
            .unwrap();
        builder
            .add_change_output(
                change_fvk.clone(),
                None,
                change_recipient,
                NoteValue::from_raw(5_000),
                [0u8; 512],
            )
            .unwrap();
        let balance: i64 = builder.value_balance().unwrap();
        assert_eq!(balance, 10_000);

        let (pczt_bundle, bundle_meta) = builder.build_for_pczt(&mut rng).unwrap();
        assert!(pczt_bundle.flags().disable_cross_address());
        assert_eq!(pczt_bundle.actions().len(), 2);
        assert_eq!(i64::try_from(pczt_bundle.value_sum).unwrap(), 10_000);
        pczt_bundle.verify_cross_address_restriction().unwrap();

        let spend_action_index = bundle_meta.spend_action_index(0).unwrap();
        let change_action_index = bundle_meta.output_action_index(0).unwrap();
        assert_ne!(spend_action_index, change_action_index);

        let spend_action = &pczt_bundle.actions()[spend_action_index];
        assert_eq!(
            spend_action.spend.recipient,
            Some(spend_recipient),
            "the real spend remains at the spent note's address"
        );
        assert_eq!(spend_action.spend.value, Some(NoteValue::from_raw(15_000)));
        assert!(spend_action.spend.dummy_sk.is_none());
        assert_eq!(spend_action.output.recipient, Some(spend_recipient));
        assert_eq!(spend_action.output.value, Some(NoteValue::ZERO));

        let change_action = &pczt_bundle.actions()[change_action_index];
        assert_eq!(change_action.spend.recipient, Some(change_recipient));
        assert_eq!(change_action.spend.value, Some(NoteValue::ZERO));
        assert!(change_action.spend.dummy_sk.is_none());
        assert_eq!(change_action.spend.fvk.as_ref(), Some(&change_fvk));
        assert_eq!(change_action.output.recipient, Some(change_recipient));
        assert_eq!(change_action.output.value, Some(NoteValue::from_raw(5_000)));

        for action in pczt_bundle.actions() {
            assert!(action
                .spend
                .recipient
                .as_ref()
                .unwrap()
                .same_receiver(action.output.recipient.as_ref().unwrap()));
        }
    }

    #[test]
    fn cross_address_disabled_padding_pairs_dummy_addresses() {
        let mut rng = OsRng;
        let sk = SpendingKey::random(&mut rng);
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::Internal);
        let mut builder = Builder::new(
            restricted_bundle_type(true),
            EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
        );

        builder
            .add_change_output(fvk, None, recipient, NoteValue::ZERO, [0u8; 512])
            .unwrap();

        let (pczt_bundle, bundle_meta) = builder.build_for_pczt(&mut rng).unwrap();
        assert_eq!(pczt_bundle.actions().len(), 2);
        pczt_bundle.verify_cross_address_restriction().unwrap();

        let change_action_index = bundle_meta.output_action_index(0).unwrap();
        let (_, padding_action) = pczt_bundle
            .actions()
            .iter()
            .enumerate()
            .find(|(idx, action)| *idx != change_action_index && action.spend.dummy_sk.is_some())
            .unwrap();

        assert_eq!(padding_action.spend.value, Some(NoteValue::ZERO));
        assert_eq!(padding_action.output.value, Some(NoteValue::ZERO));
        assert!(padding_action
            .spend
            .recipient
            .as_ref()
            .unwrap()
            .same_receiver(padding_action.output.recipient.as_ref().unwrap()));
    }

    #[test]
    fn cross_address_disabled_rejects_non_change_outputs() {
        let mut rng = OsRng;
        let sk = SpendingKey::random(&mut rng);
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        assert!(matches!(
            bundle::<i64>(
                &mut rng,
                EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
                restricted_bundle_type(false),
                vec![],
                vec![OutputInfo::new(
                    None,
                    recipient,
                    NoteValue::from_raw(5_000),
                    [0u8; 512],
                )],
                OrchardCircuitVersion::Ironwood,
            ),
            Err(BuildError::CrossAddressDisabled)
        ));

        let change_output =
            OutputInfo::change(fvk, None, recipient, NoteValue::from_raw(5_000), [0u8; 512])
                .unwrap();
        let (bundle, bundle_meta) = bundle::<i64>(
            &mut rng,
            EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
            restricted_bundle_type(false),
            vec![],
            vec![change_output],
            OrchardCircuitVersion::Ironwood,
        )
        .unwrap()
        .unwrap();

        assert!(bundle.flags().disable_cross_address());
        assert!(bundle_meta.output_action_index(0).is_some());
    }

    #[test]
    fn cross_address_disabled_non_pczt_signing_flow() {
        let mut rng = OsRng;
        let spend_sk = SpendingKey::random(&mut rng);
        let spend_fvk = FullViewingKey::from(&spend_sk);
        let spend_recipient = spend_fvk.address_at(0u32, Scope::External);
        let change_sk = SpendingKey::random(&mut rng);
        let change_fvk = FullViewingKey::from(&change_sk);
        let change_recipient = change_fvk.address_at(0u32, Scope::Internal);
        let (note, merkle_path, anchor) =
            note_with_path(&mut rng, spend_recipient, NoteValue::from_raw(15_000));

        let mut builder = Builder::new(restricted_bundle_type(false), anchor);
        builder.add_spend(spend_fvk, note, merkle_path).unwrap();
        builder
            .add_change_output(
                change_fvk.clone(),
                None,
                change_recipient,
                NoteValue::from_raw(5_000),
                [0u8; 512],
            )
            .unwrap();

        let bundle = builder
            .build::<i64>(&mut rng, OrchardCircuitVersion::Ironwood)
            .unwrap()
            .unwrap()
            .0;

        fn num_unsigned<P: core::fmt::Debug>(
            bundle: &Bundle<super::InProgress<P, super::PartiallyAuthorized>, i64>,
        ) -> usize {
            bundle
                .actions()
                .iter()
                .filter(|a| matches!(a.authorization(), MaybeSigned::SigningMetadata(_)))
                .count()
        }

        // Both the real spend and the fabricated change spend require real signatures.
        let bundle = bundle.prepare(rng, [0; 32]);
        assert_eq!(num_unsigned(&bundle), 2);

        let bundle = bundle.sign(rng, &SpendAuthorizingKey::from(&spend_sk));
        assert_eq!(num_unsigned(&bundle), 1);

        let bundle = bundle.sign(rng, &SpendAuthorizingKey::from(&change_sk));
        assert_eq!(num_unsigned(&bundle), 0);

        // A change-only bundle: the padding dummy spend is signed during `prepare`, so
        // a single `sign` call with the change key completes the actions.
        let mut builder = Builder::new(
            restricted_bundle_type(false),
            EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
        );
        builder
            .add_change_output(
                change_fvk,
                None,
                change_recipient,
                NoteValue::from_raw(5_000),
                [0u8; 512],
            )
            .unwrap();

        let bundle = builder
            .build::<i64>(&mut rng, OrchardCircuitVersion::Ironwood)
            .unwrap()
            .unwrap()
            .0
            .prepare(rng, [0; 32]);
        assert_eq!(bundle.actions().len(), 2);
        assert_eq!(num_unsigned(&bundle), 1);

        let bundle = bundle.sign(rng, &SpendAuthorizingKey::from(&change_sk));
        assert_eq!(num_unsigned(&bundle), 0);
    }

    #[test]
    fn restricted_pczt_structural_checks_reject_tampering() {
        let pk = ProvingKey::build(OrchardCircuitVersion::Ironwood);
        let mut rng = OsRng;
        let spend_sk = SpendingKey::random(&mut rng);
        let spend_fvk = FullViewingKey::from(&spend_sk);
        let spend_recipient = spend_fvk.address_at(0u32, Scope::External);
        let change_sk = SpendingKey::random(&mut rng);
        let change_fvk = FullViewingKey::from(&change_sk);
        let change_recipient = change_fvk.address_at(0u32, Scope::Internal);
        let (note, merkle_path, anchor) =
            note_with_path(&mut rng, spend_recipient, NoteValue::from_raw(15_000));

        let mut builder = Builder::new(restricted_bundle_type(false), anchor);
        builder.add_spend(spend_fvk, note, merkle_path).unwrap();
        builder
            .add_change_output(
                change_fvk,
                None,
                change_recipient,
                NoteValue::from_raw(5_000),
                [0u8; 512],
            )
            .unwrap();

        let (mut pczt_bundle, _) = builder.build_for_pczt(&mut rng).unwrap();
        pczt_bundle.verify_cross_address_restriction().unwrap();
        pczt_bundle.create_proof(&pk, rng).unwrap();

        let spend_recipient = pczt_bundle.actions()[0].spend.recipient.unwrap();
        let other_recipient = loop {
            let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
            let recipient = fvk.address_at(0u32, Scope::External);
            if !spend_recipient.same_receiver(&recipient) {
                break recipient;
            }
        };
        pczt_bundle.actions_mut()[0].output.recipient = Some(other_recipient);

        assert!(matches!(
            pczt_bundle.verify_cross_address_restriction(),
            Err(VerifyError::DisallowedCrossAddressTransfer)
        ));
        assert!(matches!(
            pczt_bundle.create_proof(&pk, rng),
            Err(ProverError::DisallowedCrossAddressTransfer)
        ));
    }

    #[test]
    fn create_proof_supports_disable_cross_address_only_for_ironwood() {
        let build_bundle = |rng: &mut OsRng, circuit_version: OrchardCircuitVersion| {
            let flags = Flags::CROSS_ADDRESS_DISABLED;

            let builder = Builder::new(
                BundleType::Transactional {
                    flags,
                    bundle_required: true,
                },
                EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
            );

            builder
                .build::<i64>(rng, circuit_version)
                .unwrap()
                .unwrap()
                .0
        };

        let mut rng = OsRng;
        let pk = ProvingKey::build(OrchardCircuitVersion::FixedPostNu6_2);
        let bundle = build_bundle(&mut rng, OrchardCircuitVersion::FixedPostNu6_2);

        assert!(matches!(
            bundle.create_proof(&pk, &mut rng),
            Err(BuildError::Proof(
                halo2_proofs::plonk::Error::InvalidInstances
            )),
        ));

        let pk = ProvingKey::build(OrchardCircuitVersion::Ironwood);
        let bundle = build_bundle(&mut rng, OrchardCircuitVersion::Ironwood);
        bundle.create_proof(&pk, &mut rng).unwrap();
    }
}
