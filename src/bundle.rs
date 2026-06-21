//! Structs related to bundles of Orchard actions.

use alloc::vec::Vec;

pub mod commitments;

#[cfg(feature = "circuit")]
mod batch;
#[cfg(feature = "circuit")]
pub use batch::{BatchError, BatchValidator};

use core::fmt;

use blake2b_simd::Hash as Blake2bHash;
use nonempty::NonEmpty;
use zcash_note_encryption::{try_note_decryption, try_output_recovery_with_ovk};

#[cfg(feature = "std")]
use memuse::DynamicUsage;

use crate::{
    action::Action,
    address::Address,
    bundle::commitments::{
        hash_bundle_auth_data, hash_bundle_auth_data_with_domain, hash_bundle_txid_data,
        hash_bundle_txid_data_with_domain, BundleCommitmentDomain,
    },
    keys::{IncomingViewingKey, OutgoingViewingKey, PreparedIncomingViewingKey},
    note::Note,
    note_encryption::OrchardDomain,
    primitives::redpallas::{self, Binding, SpendAuth},
    tree::Anchor,
    value::{ValueCommitTrapdoor, ValueCommitment, ValueSum},
    Proof,
};

#[cfg(feature = "circuit")]
use crate::circuit::{Instance, VerifyingKey};

#[cfg(feature = "circuit")]
impl<T> Action<T> {
    /// Prepares the public instance for this action, for creating and verifying the
    /// bundle proof.
    pub fn to_instance(&self, flags: Flags, anchor: Anchor) -> Instance {
        Instance::from_parts(
            anchor,
            self.cv_net().clone(),
            *self.nullifier(),
            self.rk().clone(),
            *self.cmx(),
            flags,
        )
        .expect("this Action's rk is non-identity by construction (Action::from_parts)")
    }
}

/// Selects the valid protocol and pool combination for a bundle.
///
/// Encodes the correlated choices a caller would otherwise have to pass
/// separately: pool, circuit version, flag-byte format, default note version,
/// and the cross-address policy for transaction bundles.
///
/// Coinbase bundles use the protocol for circuit selection and default note
/// version. Their flags are fixed to spends disabled, outputs enabled, and
/// cross-address transfers enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BundleProtocol {
    /// The Orchard pool before NU6.2.
    ///
    /// Uses the insecure historical Orchard circuit and pre-NU6.3 flag-byte format.
    /// Cross-address transfers are permitted and notes use the V2 plaintext format.
    OrchardPreNu6_2,
    /// The Orchard pool from NU6.2 until NU6.3.
    ///
    /// Uses the post-NU6.2 fixed Orchard circuit and pre-NU6.3 flag-byte format.
    /// Cross-address transfers are permitted and notes use the V2 plaintext format.
    OrchardPreNu6_3,
    /// The Orchard pool at NU6.3+.
    ///
    /// Uses the post-NU6.3 circuit and NU6.3 flag-byte format. For transactional
    /// bundles, `enableCrossAddress = 0` is required by consensus, so
    /// cross-address transfers are prohibited. Notes use V2 plaintexts.
    OrchardPostNu6_3,
    /// The Ironwood pool at NU6.3+.
    ///
    /// Uses the post-NU6.3 circuit and NU6.3 flag-byte format. Transactional
    /// bundles enable cross-address transfers. Notes use V3 quantum-recoverable
    /// plaintexts.
    IronwoodPostNu6_3,
}

#[cfg(feature = "circuit")]
impl BundleProtocol {
    /// Returns the [`OrchardCircuitVersion`] for this protocol.
    ///
    /// [`OrchardCircuitVersion`]: crate::circuit::OrchardCircuitVersion
    pub fn circuit_version(self) -> crate::circuit::OrchardCircuitVersion {
        match self {
            BundleProtocol::OrchardPreNu6_2 => {
                crate::circuit::OrchardCircuitVersion::InsecurePreNu6_2
            }
            BundleProtocol::OrchardPreNu6_3 => {
                crate::circuit::OrchardCircuitVersion::FixedPostNu6_2
            }
            BundleProtocol::OrchardPostNu6_3 | BundleProtocol::IronwoodPostNu6_3 => {
                crate::circuit::OrchardCircuitVersion::PostNu6_3
            }
        }
    }
}

impl BundleProtocol {
    /// Returns the [`BundleFormat`] for this protocol.
    pub(crate) const fn bundle_format(self) -> BundleFormat {
        match self {
            BundleProtocol::OrchardPreNu6_2 | BundleProtocol::OrchardPreNu6_3 => {
                BundleFormat::PreNu6_3
            }
            BundleProtocol::OrchardPostNu6_3 | BundleProtocol::IronwoodPostNu6_3 => {
                BundleFormat::Nu6_3
            }
        }
    }

    /// Returns the default [`NoteVersion`] for notes created in this protocol.
    ///
    /// [`NoteVersion`]: crate::note::NoteVersion
    pub fn default_note_version(self) -> crate::note::NoteVersion {
        match self {
            BundleProtocol::IronwoodPostNu6_3 => crate::note::NoteVersion::V3,
            BundleProtocol::OrchardPreNu6_2
            | BundleProtocol::OrchardPreNu6_3
            | BundleProtocol::OrchardPostNu6_3 => crate::note::NoteVersion::V2,
        }
    }

    /// Whether consensus requires transactional bundles under this protocol to disable
    /// cross-address transfers (`enableCrossAddress = 0`).
    pub(crate) fn requires_cross_address_restriction(self) -> bool {
        matches!(self, BundleProtocol::OrchardPostNu6_3)
    }
}

/// The transaction-format generation an Orchard bundle is encoded in.
///
/// This determines how the bundle's flag byte is interpreted, which changes at the
/// NU6.3 network upgrade. In pre-NU6.3 transaction formats, bit 2 is a reserved zero
/// bit and cross-address transfers are implicitly enabled. In NU6.3 transaction
/// formats, bit 2 is the `enableCrossAddress` flag.
///
/// # Choosing the correct format
///
/// A flag byte with bit 2 *clear* is valid in **both** generations but means **opposite**
/// things: under [`PreNu6_3`] it denotes an unrestricted bundle (cross-address transfers
/// implicitly enabled), while under [`Nu6_3`] it denotes a restricted bundle
/// (`enableCrossAddress` clear, cross-address transfers disabled). Decoding such a byte
/// under the wrong generation therefore *silently* produces the wrong [`Flags`] — there is
/// no error to catch the mistake.
///
/// The producing direction is guarded: [`Flags::to_byte`] and [`Bundle::commitment`] refuse
/// to encode a restricted bundle under [`PreNu6_3`] (returning `None` / an error). The silent
/// hazard is entirely on the **consuming** side, when parsing a flag byte under the wrong
/// generation ([`Flags::from_byte`], [`crate::pczt::Bundle::parse`]).
///
/// This crate has no concept of consensus branches or activation heights, so it cannot derive
/// the correct format itself — the disambiguating fact lives in the caller. The safe discipline
/// is therefore:
///
/// - Derive the `BundleFormat` **once**, from the concrete transaction or PCZT encoding
///   version at the boundary where this crate is integrated (e.g. in `zcash_primitives`).
///   Legacy v5 encodings use [`PreNu6_3`], while v6 encodings use [`Nu6_3`].
/// - Thread that single value into every format-taking method for the bundle, rather than
///   recomputing or hardcoding it per call site.
///
/// [`PreNu6_3`]: BundleFormat::PreNu6_3
/// [`Nu6_3`]: BundleFormat::Nu6_3
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BundleFormat {
    /// Transaction formats before NU6.3, where bit 2 of the flag byte is reserved.
    PreNu6_3,
    /// NU6.3 transaction formats, where bit 2 is `enableCrossAddress`.
    Nu6_3,
}

/// Orchard-specific flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flags {
    /// Flag denoting whether Orchard spends are enabled in the transaction.
    ///
    /// If `false`, spent notes within [`Action`]s in the transaction's [`Bundle`] are
    /// guaranteed to be dummy notes. If `true`, the spent notes may be either real or
    /// dummy notes.
    spends_enabled: bool,
    /// Flag denoting whether Orchard outputs are enabled in the transaction.
    ///
    /// If `false`, created notes within [`Action`]s in the transaction's [`Bundle`] are
    /// guaranteed to be dummy notes. If `true`, the created notes may be either real or
    /// dummy notes.
    outputs_enabled: bool,
    /// Flag denoting whether Orchard spends and outputs may use different expanded
    /// receivers.
    ///
    /// If `false`, every action's output is constrained to be addressed to the same
    /// expanded receiver as the note it spends; proving and verification must reject the
    /// bundle unless they use a circuit key that supports the restriction.
    cross_address_enabled: bool,
}

const FLAG_SPENDS_ENABLED: u8 = 0b0000_0001;
const FLAG_OUTPUTS_ENABLED: u8 = 0b0000_0010;
const FLAG_CROSS_ADDRESS_ENABLED: u8 = 0b0000_0100;
const PRE_NU6_3_FLAGS_EXPECTED_UNSET: u8 = !(FLAG_SPENDS_ENABLED | FLAG_OUTPUTS_ENABLED);
const NU6_3_FLAGS_EXPECTED_UNSET: u8 =
    !(FLAG_SPENDS_ENABLED | FLAG_OUTPUTS_ENABLED | FLAG_CROSS_ADDRESS_ENABLED);

impl Flags {
    /// Construct a set of flags from its constituent parts, with cross-address
    /// transfers enabled.
    ///
    /// Cross-address transfers are always enabled here by design. The only restricted flag
    /// set this crate exposes is [`Flags::CROSS_ADDRESS_DISABLED`], which keeps spends and
    /// outputs enabled. A flag set that *both* disables cross-address transfers *and* disables
    /// spends or outputs is intentionally unrepresentable, because it is not a useful
    /// combination: see the [`BundleType::Coinbase`] documentation for why a spends-disabled
    /// bundle must keep cross-address transfers enabled to pay an arbitrary recipient. If a
    /// use case for that combination ever arises, this constructor would need to take
    /// `cross_address_enabled`.
    ///
    /// [`BundleType::Coinbase`]: crate::builder::BundleType::Coinbase
    pub(crate) const fn from_parts(spends_enabled: bool, outputs_enabled: bool) -> Self {
        Flags {
            spends_enabled,
            outputs_enabled,
            cross_address_enabled: true,
        }
    }

    pub(crate) const fn from_parts_with_cross_address(
        spends_enabled: bool,
        outputs_enabled: bool,
        cross_address_enabled: bool,
    ) -> Self {
        Flags {
            spends_enabled,
            outputs_enabled,
            cross_address_enabled,
        }
    }

    /// The flag set for an unrestricted bundle: spends and outputs are both
    /// enabled, so its actions may spend and create real notes addressed to any
    /// expanded receivers.
    ///
    /// Like [`Self::SPENDS_DISABLED`] and [`Self::OUTPUTS_DISABLED`], this leaves
    /// cross-address transfers enabled; see [`Self::CROSS_ADDRESS_DISABLED`] for
    /// the restricted variant.
    pub const ENABLED: Flags = Flags {
        spends_enabled: true,
        outputs_enabled: true,
        cross_address_enabled: true,
    };

    /// The flag set for a bundle that may create notes but not spend them: every
    /// spent note is constrained to be a dummy, while outputs may be real.
    ///
    /// This is the flag set used by coinbase transactions, which mint value into
    /// the Orchard pool without consuming existing notes.
    pub const SPENDS_DISABLED: Flags = Flags {
        spends_enabled: false,
        outputs_enabled: true,
        cross_address_enabled: true,
    };

    /// The flag set for a bundle that may spend notes but not create them: every
    /// created note is constrained to be a dummy, while spends may be real.
    ///
    /// This is used to remove value from the Orchard pool without producing new
    /// notes within the bundle.
    pub const OUTPUTS_DISABLED: Flags = Flags {
        spends_enabled: true,
        outputs_enabled: false,
        cross_address_enabled: true,
    };

    /// The flag set with spends and outputs enabled and cross-address transfers disabled.
    ///
    /// This flag set cannot be encoded in pre-NU6.3 formats. Proof creation and
    /// verification for instances built with this flag require a post-NU 6.3 circuit key.
    pub const CROSS_ADDRESS_DISABLED: Flags = Flags {
        spends_enabled: true,
        outputs_enabled: true,
        cross_address_enabled: false,
    };

    /// Flag denoting whether Orchard spends are enabled in the transaction.
    ///
    /// If `false`, spent notes within [`Action`]s in the transaction's [`Bundle`] are
    /// guaranteed to be dummy notes. If `true`, the spent notes may be either real or
    /// dummy notes.
    pub fn spends_enabled(&self) -> bool {
        self.spends_enabled
    }

    /// Flag denoting whether Orchard outputs are enabled in the transaction.
    ///
    /// If `false`, created notes within [`Action`]s in the transaction's [`Bundle`] are
    /// guaranteed to be dummy notes. If `true`, the created notes may be either real or
    /// dummy notes.
    pub fn outputs_enabled(&self) -> bool {
        self.outputs_enabled
    }

    /// Flag denoting whether Orchard spends and outputs may use different expanded
    /// receivers.
    ///
    /// If `false`, every action's output is constrained to be addressed to the same
    /// expanded receiver as the note it spends; proving and verification must reject the
    /// bundle unless they use a circuit key that supports the restriction.
    pub fn cross_address_enabled(&self) -> bool {
        self.cross_address_enabled
    }

    /// Serialize flags to a byte as defined in [Zcash Protocol Spec § 7.1: Transaction
    /// Encoding And Consensus][txencoding], under the provided transaction format.
    ///
    /// Returns `None` if this flag set cannot be encoded in the provided format, i.e.
    /// cross-address transfers are disabled but `format` is pre-NU6.3 (where bit 2 is
    /// a reserved zero bit and cross-address transfers are implicitly enabled).
    ///
    /// See [`BundleFormat`] for how to choose `format`.
    ///
    /// [txencoding]: https://zips.z.cash/protocol/protocol.pdf#txnencoding
    pub fn to_byte(&self, format: BundleFormat) -> Option<u8> {
        let mut value = 0u8;
        if self.spends_enabled {
            value |= FLAG_SPENDS_ENABLED;
        }
        if self.outputs_enabled {
            value |= FLAG_OUTPUTS_ENABLED;
        }
        match format {
            BundleFormat::PreNu6_3 if !self.cross_address_enabled => None,
            BundleFormat::PreNu6_3 => Some(value),
            BundleFormat::Nu6_3 => {
                if self.cross_address_enabled {
                    value |= FLAG_CROSS_ADDRESS_ENABLED;
                }
                Some(value)
            }
        }
    }

    /// Parses flags from a single byte as defined in [Zcash Protocol Spec § 7.1:
    /// Transaction Encoding And Consensus][txencoding], under the provided transaction
    /// format. The protocol specification defines bits 0 and 1; bit 2 (the NU6.3
    /// `enableCrossAddress` flag) is interpreted according to `format`, and is a
    /// reserved zero bit in pre-NU6.3 formats, where cross-address transfers are
    /// implicitly enabled.
    ///
    /// Returns `None` if unexpected bits are set in the flag byte.
    ///
    /// `format` selects how bit 2 is interpreted; passing the generation that does not match
    /// the transaction silently mis-decodes an otherwise-valid byte (a byte with bit 2 clear
    /// is valid in both generations but means opposite things). See [`BundleFormat`] for how
    /// to choose it.
    ///
    /// [txencoding]: https://zips.z.cash/protocol/protocol.pdf#txnencoding
    pub fn from_byte(value: u8, format: BundleFormat) -> Option<Self> {
        let expected_unset = match format {
            // https://p.z.cash/TCR:bad-txns-v5-reserved-bits-nonzero
            BundleFormat::PreNu6_3 => PRE_NU6_3_FLAGS_EXPECTED_UNSET,
            // NU6.3 reinterprets bit 2 as `enableCrossAddress`; only bits 3.. are reserved.
            // The pre-NU6.3 rule above is anchored at
            // p.z.cash/TCR:bad-txns-v5-reserved-bits-nonzero.
            // TODO(resolve-before-release): the NU6.3 reserved-bits consensus rule (per the
            // NU6.3 ZIP defining `enableCrossAddress`) has no p.z.cash anchor yet; add the
            // canonical TCR link here once published, to match the pre-NU6.3 citation.
            BundleFormat::Nu6_3 => NU6_3_FLAGS_EXPECTED_UNSET,
        };

        if value & expected_unset == 0 {
            Some(Self {
                spends_enabled: value & FLAG_SPENDS_ENABLED != 0,
                outputs_enabled: value & FLAG_OUTPUTS_ENABLED != 0,
                cross_address_enabled: match format {
                    BundleFormat::PreNu6_3 => true,
                    BundleFormat::Nu6_3 => value & FLAG_CROSS_ADDRESS_ENABLED != 0,
                },
            })
        } else {
            None
        }
    }
}

/// Defines the authorization type of an Orchard bundle.
pub trait Authorization: fmt::Debug {
    /// The authorization type of an Orchard action.
    type SpendAuth: fmt::Debug;
}

/// A bundle of actions to be applied to the ledger.
#[derive(Clone)]
pub struct Bundle<T: Authorization, V> {
    /// The list of actions that make up this bundle.
    actions: NonEmpty<Action<T::SpendAuth>>,
    /// Orchard-specific transaction-level flags for this bundle.
    flags: Flags,
    /// The net value moved out of the Orchard shielded pool.
    ///
    /// This is the sum of Orchard spends minus the sum of Orchard outputs.
    value_balance: V,
    /// The root of the Orchard commitment tree that this bundle commits to.
    anchor: Anchor,
    /// The authorization for this bundle.
    authorization: T,
}

impl<T: Authorization, V: fmt::Debug> fmt::Debug for Bundle<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /// Helper struct for debug-printing actions without exposing `NonEmpty`.
        struct Actions<'a, T>(&'a NonEmpty<Action<T>>);
        impl<T: fmt::Debug> fmt::Debug for Actions<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_struct("Bundle")
            .field("actions", &Actions(&self.actions))
            .field("flags", &self.flags)
            .field("value_balance", &self.value_balance)
            .field("anchor", &self.anchor)
            .field("authorization", &self.authorization)
            .finish()
    }
}

/// Checks that `proof` has the canonical length for a bundle of `num_actions` actions.
///
/// Returns [`BundleError::NonCanonicalProofSize`] if it does not. This is the shared check
/// used by the proof-carrying bundle constructors to reject non-canonical (e.g. padded)
/// proofs; see [`Bundle::try_from_parts`] (GHSA-2x4w-pxqw-58v9).
pub(crate) fn validate_proof_size(proof: &Proof, num_actions: usize) -> Result<(), BundleError> {
    let expected = Proof::expected_proof_size(num_actions);
    let actual = proof.as_ref().len();
    if actual == expected {
        Ok(())
    } else {
        Err(BundleError::NonCanonicalProofSize { expected, actual })
    }
}

impl<T: Authorization, V> Bundle<T, V> {
    /// Constructs a `Bundle` from its constituent parts without validating the authorization.
    ///
    /// This does not check the proof size, so it must only be used with an authorization that
    /// either carries no proof or carries a proof that is already known to be canonical (e.g.
    /// one produced by [`Proof::create`]). Construction from untrusted parts must instead go
    /// through a checked, authorization-specific constructor such as [`Bundle::try_from_parts`].
    pub(crate) fn from_parts_unchecked(
        actions: NonEmpty<Action<T::SpendAuth>>,
        flags: Flags,
        value_balance: V,
        anchor: Anchor,
        authorization: T,
    ) -> Self {
        Bundle {
            actions,
            flags,
            value_balance,
            anchor,
            authorization,
        }
    }

    /// Returns the list of actions that make up this bundle.
    pub fn actions(&self) -> &NonEmpty<Action<T::SpendAuth>> {
        &self.actions
    }

    /// Returns the Orchard-specific transaction-level flags for this bundle.
    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    /// Returns the net value moved into or out of the Orchard shielded pool.
    ///
    /// This is the sum of Orchard spends minus the sum Orchard outputs.
    pub fn value_balance(&self) -> &V {
        &self.value_balance
    }

    /// Returns the root of the Orchard commitment tree that this bundle commits to.
    pub fn anchor(&self) -> &Anchor {
        &self.anchor
    }

    /// Returns the authorization for this bundle.
    ///
    /// In the case of a `Bundle<Authorized>`, this is the proof and binding signature.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Construct a new bundle by applying a transformation that might fail
    /// to the value balance.
    pub fn try_map_value_balance<V0, E, F: FnOnce(V) -> Result<V0, E>>(
        self,
        f: F,
    ) -> Result<Bundle<T, V0>, E> {
        Ok(Bundle {
            actions: self.actions,
            flags: self.flags,
            value_balance: f(self.value_balance)?,
            anchor: self.anchor,
            authorization: self.authorization,
        })
    }

    /// Transitions this bundle from one authorization state to another.
    pub fn map_authorization<R, U: Authorization>(
        self,
        context: &mut R,
        mut spend_auth: impl FnMut(&mut R, &T, T::SpendAuth) -> U::SpendAuth,
        step: impl FnOnce(&mut R, T) -> U,
    ) -> Bundle<U, V> {
        let authorization = self.authorization;
        Bundle {
            actions: self
                .actions
                .map(|a| a.map(|a_auth| spend_auth(context, &authorization, a_auth))),
            flags: self.flags,
            value_balance: self.value_balance,
            anchor: self.anchor,
            authorization: step(context, authorization),
        }
    }

    /// Transitions this bundle from one authorization state to another.
    pub fn try_map_authorization<R, U: Authorization, E>(
        self,
        context: &mut R,
        mut spend_auth: impl FnMut(&mut R, &T, T::SpendAuth) -> Result<U::SpendAuth, E>,
        step: impl FnOnce(&mut R, T) -> Result<U, E>,
    ) -> Result<Bundle<U, V>, E> {
        let authorization = self.authorization;
        let new_actions = self
            .actions
            .into_iter()
            .map(|a| a.try_map(|a_auth| spend_auth(context, &authorization, a_auth)))
            .collect::<Result<Vec<_>, E>>()?;

        Ok(Bundle {
            actions: NonEmpty::from_vec(new_actions).unwrap(),
            flags: self.flags,
            value_balance: self.value_balance,
            anchor: self.anchor,
            authorization: step(context, authorization)?,
        })
    }

    #[cfg(feature = "circuit")]
    pub(crate) fn to_instances(&self) -> Vec<Instance> {
        self.actions
            .iter()
            .map(|a| a.to_instance(self.flags, self.anchor))
            .collect()
    }

    /// Performs trial decryption of each action in the bundle with each of the
    /// specified incoming viewing keys, and returns a vector of each decrypted
    /// note plaintext contents along with the index of the action from which it
    /// was derived.
    pub fn decrypt_outputs_with_keys(
        &self,
        keys: &[IncomingViewingKey],
    ) -> Vec<(usize, IncomingViewingKey, Note, Address, [u8; 512])> {
        let prepared_keys: Vec<_> = keys
            .iter()
            .map(|ivk| (ivk, PreparedIncomingViewingKey::new(ivk)))
            .collect();
        self.actions
            .iter()
            .enumerate()
            .filter_map(|(idx, action)| {
                let domain = OrchardDomain::for_action(action);
                prepared_keys.iter().find_map(|(ivk, prepared_ivk)| {
                    try_note_decryption(&domain, prepared_ivk, action)
                        .map(|(n, a, m)| (idx, (*ivk).clone(), n, a, m))
                })
            })
            .collect()
    }

    /// Performs trial decryption of the action at `action_idx` in the bundle with the
    /// specified incoming viewing key, and returns the decrypted note plaintext
    /// contents if successful.
    pub fn decrypt_output_with_key(
        &self,
        action_idx: usize,
        key: &IncomingViewingKey,
    ) -> Option<(Note, Address, [u8; 512])> {
        let prepared_ivk = PreparedIncomingViewingKey::new(key);
        self.actions.get(action_idx).and_then(move |action| {
            let domain = OrchardDomain::for_action(action);
            try_note_decryption(&domain, &prepared_ivk, action)
        })
    }

    /// Performs trial decryption of each action in the bundle with each of the
    /// specified outgoing viewing keys, and returns a vector of each decrypted
    /// note plaintext contents along with the index of the action from which it
    /// was derived.
    pub fn recover_outputs_with_ovks(
        &self,
        keys: &[OutgoingViewingKey],
    ) -> Vec<(usize, OutgoingViewingKey, Note, Address, [u8; 512])> {
        self.actions
            .iter()
            .enumerate()
            .filter_map(|(idx, action)| {
                let domain = OrchardDomain::for_action(action);
                keys.iter().find_map(move |key| {
                    try_output_recovery_with_ovk(
                        &domain,
                        key,
                        action,
                        action.cv_net(),
                        &action.encrypted_note().out_ciphertext,
                    )
                    .map(|(n, a, m)| (idx, key.clone(), n, a, m))
                })
            })
            .collect()
    }

    /// Attempts to decrypt the action at the specified index with the specified
    /// outgoing viewing key, and returns the decrypted note plaintext contents
    /// if successful.
    pub fn recover_output_with_ovk(
        &self,
        action_idx: usize,
        key: &OutgoingViewingKey,
    ) -> Option<(Note, Address, [u8; 512])> {
        self.actions.get(action_idx).and_then(move |action| {
            let domain = OrchardDomain::for_action(action);
            try_output_recovery_with_ovk(
                &domain,
                key,
                action,
                action.cv_net(),
                &action.encrypted_note().out_ciphertext,
            )
        })
    }
}

impl<T: Authorization, V: Copy + Into<i64>> Bundle<T, V> {
    /// Computes a commitment to the effects of this bundle, suitable for inclusion within
    /// a transaction ID.
    ///
    /// The flag byte is hashed as encoded under `format`, the transaction encoding the
    /// bundle appears in (see [`BundleFormat`]), so the digest depends on the encoding
    /// era. See [`BundleFormat`] for how to derive `format` and why getting it wrong matters.
    ///
    /// # Errors
    ///
    /// Returns [`CommitmentError::UnrepresentableFlags`] if the flags cannot be encoded
    /// in `format` (cross-address transfers disabled under [`BundleFormat::PreNu6_3`]);
    /// such a bundle cannot appear in a pre-NU6.3 transaction.
    pub fn commitment(&self, format: BundleFormat) -> Result<BundleCommitment, CommitmentError> {
        hash_bundle_txid_data(self, format)
            .map(BundleCommitment)
            .ok_or(CommitmentError::UnrepresentableFlags)
    }

    /// Computes a commitment to the effects of this bundle under the specified
    /// bundle commitment domain.
    pub fn commitment_for_domain(
        &self,
        domain: BundleCommitmentDomain,
    ) -> Result<BundleCommitment, CommitmentError> {
        hash_bundle_txid_data_with_domain(self, domain)
            .map(BundleCommitment)
            .ok_or(CommitmentError::UnrepresentableFlags)
    }

    /// Returns the transaction binding validating key for this bundle.
    ///
    /// This can be used to validate the [`Authorized::binding_signature`] returned from
    /// [`Bundle::authorization`].
    pub fn binding_validating_key(&self) -> redpallas::VerificationKey<Binding> {
        // https://p.z.cash/TCR:bad-txns-orchard-binding-signature-invalid?partial
        (self
            .actions
            .iter()
            .map(|a| a.cv_net())
            .sum::<ValueCommitment>()
            - ValueCommitment::derive(
                ValueSum::from_raw(self.value_balance.into()),
                ValueCommitTrapdoor::zero(),
            ))
        .into_bvk()
    }
}

/// Marker type for a bundle that contains no authorizing data.
#[derive(Clone, Debug)]
pub struct EffectsOnly;

impl Authorization for EffectsOnly {
    type SpendAuth = ();
}

impl<V> Bundle<EffectsOnly, V> {
    /// Constructs an effects-only `Bundle` from its constituent parts.
    ///
    /// An effects-only bundle carries no proof, so there is no proof size to validate,
    /// and flags are not checked against circuit support (there is no proof key to
    /// check against).
    pub fn from_parts(
        actions: NonEmpty<Action<<EffectsOnly as Authorization>::SpendAuth>>,
        flags: Flags,
        value_balance: V,
        anchor: Anchor,
        authorization: EffectsOnly,
    ) -> Self {
        Bundle::from_parts_unchecked(actions, flags, value_balance, anchor, authorization)
    }
}

/// Authorizing data for a bundle of actions, ready to be committed to the ledger.
#[derive(Debug, Clone)]
pub struct Authorized {
    proof: Proof,
    binding_signature: redpallas::Signature<Binding>,
}

impl Authorization for Authorized {
    type SpendAuth = redpallas::Signature<SpendAuth>;
}

impl Authorized {
    /// Constructs the authorizing data for a bundle of actions from its constituent parts.
    pub fn from_parts(proof: Proof, binding_signature: redpallas::Signature<Binding>) -> Self {
        Authorized {
            proof,
            binding_signature,
        }
    }

    /// Return the proof component of the authorizing data.
    pub fn proof(&self) -> &Proof {
        &self.proof
    }

    /// Return the binding signature.
    pub fn binding_signature(&self) -> &redpallas::Signature<Binding> {
        &self.binding_signature
    }
}

/// Errors that can occur when constructing an authorized [`Bundle`] from untrusted parts.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BundleError {
    /// The proof does not have the canonical length for the bundle's number of actions.
    ///
    /// A valid Orchard proof authorizing `n` actions is always exactly
    /// [`Proof::expected_proof_size(n)`] bytes; any other length indicates a non-canonical
    /// encoding, such as a proof padded with arbitrary trailing data.
    ///
    /// [`Proof::expected_proof_size(n)`]: crate::Proof::expected_proof_size
    NonCanonicalProofSize {
        /// The canonical proof length for the bundle's number of actions.
        expected: usize,
        /// The length of the proof that was provided.
        actual: usize,
    },
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BundleError::NonCanonicalProofSize { expected, actual } => write!(
                f,
                "Orchard proof has non-canonical length {actual}; expected {expected} bytes",
            ),
        }
    }
}

impl core::error::Error for BundleError {}

/// Errors that can occur when computing a [`Bundle`] commitment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommitmentError {
    /// The bundle's flags cannot be encoded in the requested [`BundleFormat`].
    ///
    /// Cross-address transfers are disabled and `format` is
    /// [`BundleFormat::PreNu6_3`], where bit 2 is reserved; such a bundle cannot
    /// appear in a pre-NU6.3 transaction.
    UnrepresentableFlags,
}

impl fmt::Display for CommitmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitmentError::UnrepresentableFlags => write!(
                f,
                "bundle flags are not representable in the requested transaction \
                 format (cross-address transfers disabled under pre-NU6.3 encoding)",
            ),
        }
    }
}

impl core::error::Error for CommitmentError {}

/// A flag type that identifies whether proof sizes are checked in bundle construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofSizeEnforcement {
    /// Proofs may exceed the canonical size
    Unenforced,
    /// Proofs may not exceed the canonical size
    Strict,
}

impl<V> Bundle<Authorized, V> {
    /// Constructs an authorized `Bundle` from its constituent parts.
    ///
    /// This is the only constructor for an authorized bundle: it validates that the proof has
    /// exactly [`Proof::expected_proof_size`] bytes for `actions.len()`, so an authorized bundle
    /// can never hold a non-canonical proof. This matters when building a bundle from untrusted
    /// input (e.g. deserializing from bytes), as it prevents a proof from being padded with
    /// arbitrary data, which would otherwise impose unbounded bandwidth and storage costs without
    /// affecting proof validity (GHSA-2x4w-pxqw-58v9). Circuit-key support for the bundle flags is
    /// checked when proving or verifying the proof.
    pub fn try_from_parts(
        actions: NonEmpty<Action<<Authorized as Authorization>::SpendAuth>>,
        flags: Flags,
        value_balance: V,
        anchor: Anchor,
        authorization: Authorized,
        size_enforcement: ProofSizeEnforcement,
    ) -> Result<Self, BundleError> {
        if size_enforcement == ProofSizeEnforcement::Strict {
            validate_proof_size(authorization.proof(), actions.len())?;
        }
        Ok(Bundle::from_parts_unchecked(
            actions,
            flags,
            value_balance,
            anchor,
            authorization,
        ))
    }

    /// Computes a commitment to the authorizing data within for this bundle.
    ///
    /// This together with `Bundle::commitment` bind the entire bundle.
    pub fn authorizing_commitment(&self) -> BundleAuthorizingCommitment {
        BundleAuthorizingCommitment(hash_bundle_auth_data(self))
    }

    /// Computes a commitment to the authorizing data within this bundle under
    /// the specified bundle commitment domain.
    pub fn authorizing_commitment_for_domain(
        &self,
        domain: BundleCommitmentDomain,
    ) -> BundleAuthorizingCommitment {
        BundleAuthorizingCommitment(hash_bundle_auth_data_with_domain(self, domain))
    }

    /// Verifies the proof for this bundle.
    ///
    /// # Errors
    ///
    /// Returns `Err(`[`halo2_proofs::plonk::Error::InvalidInstances`]`)` if this
    /// bundle disables cross-address transfers and `vk` is not an
    /// [`OrchardCircuitVersion::PostNu6_3`] verifying key.
    ///
    /// Also returns an error if proof verification fails.
    ///
    /// [`OrchardCircuitVersion::PostNu6_3`]: crate::circuit::OrchardCircuitVersion::PostNu6_3
    #[cfg(feature = "circuit")]
    pub fn verify_proof(&self, vk: &VerifyingKey) -> Result<(), halo2_proofs::plonk::Error> {
        self.authorization()
            .proof()
            .verify(vk, &self.to_instances())
    }
}

#[cfg(feature = "std")]
impl<V: DynamicUsage> DynamicUsage for Bundle<Authorized, V> {
    fn dynamic_usage(&self) -> usize {
        self.actions.tail.dynamic_usage()
            + self.value_balance.dynamic_usage()
            + self.authorization.proof.dynamic_usage()
    }

    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        let bounds = (
            self.actions.tail.dynamic_usage_bounds(),
            self.value_balance.dynamic_usage_bounds(),
            self.authorization.proof.dynamic_usage_bounds(),
        );
        (
            bounds.0 .0 + bounds.1 .0 + bounds.2 .0,
            bounds
                .0
                 .1
                .zip(bounds.1 .1)
                .zip(bounds.2 .1)
                .map(|((a, b), c)| a + b + c),
        )
    }
}

/// A commitment to a bundle of actions.
///
/// This commitment is non-malleable, in the sense that a bundle's commitment will only
/// change if the effects of the bundle are altered.
#[derive(Debug)]
pub struct BundleCommitment(pub Blake2bHash);

impl From<BundleCommitment> for [u8; 32] {
    fn from(commitment: BundleCommitment) -> Self {
        // The commitment uses BLAKE2b-256.
        commitment.0.as_bytes().try_into().unwrap()
    }
}

/// A commitment to the authorizing data within a bundle of actions.
#[derive(Debug)]
pub struct BundleAuthorizingCommitment(pub Blake2bHash);

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use alloc::vec::Vec;

    use group::ff::FromUniformBytes;
    use nonempty::NonEmpty;
    use pasta_curves::pallas;
    use rand::{rngs::StdRng, SeedableRng};
    use reddsa::orchard::SpendAuth;

    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::{
        primitives::redpallas::{self, testing::arb_binding_signing_key},
        value::{testing::arb_note_value_bounded, NoteValue, ValueSum, MAX_NOTE_VALUE},
        Anchor, Proof,
    };

    use super::{Action, Authorized, Bundle, Flags};

    pub use crate::action::testing::{arb_action, arb_unauthorized_action};

    /// Marker type for a bundle that contains no authorizing data.
    pub type Unauthorized = super::EffectsOnly;

    /// Generate an unauthorized action having spend and output values less than MAX_NOTE_VALUE / n_actions.
    pub fn arb_unauthorized_action_n(
        n_actions: usize,
        flags: Flags,
    ) -> impl Strategy<Value = (ValueSum, Action<()>)> {
        let spend_value_gen = if flags.spends_enabled {
            Strategy::boxed(arb_note_value_bounded(MAX_NOTE_VALUE / n_actions as u64))
        } else {
            Strategy::boxed(Just(NoteValue::ZERO))
        };

        spend_value_gen.prop_flat_map(move |spend_value| {
            let output_value_gen = if flags.outputs_enabled {
                Strategy::boxed(arb_note_value_bounded(MAX_NOTE_VALUE / n_actions as u64))
            } else {
                Strategy::boxed(Just(NoteValue::ZERO))
            };

            output_value_gen.prop_flat_map(move |output_value| {
                arb_unauthorized_action(spend_value, output_value)
                    .prop_map(move |a| (spend_value - output_value, a))
            })
        })
    }

    /// Generate an authorized action having spend and output values less than MAX_NOTE_VALUE / n_actions.
    pub fn arb_action_n(
        n_actions: usize,
        flags: Flags,
    ) -> impl Strategy<Value = (ValueSum, Action<redpallas::Signature<SpendAuth>>)> {
        let spend_value_gen = if flags.spends_enabled {
            Strategy::boxed(arb_note_value_bounded(MAX_NOTE_VALUE / n_actions as u64))
        } else {
            Strategy::boxed(Just(NoteValue::ZERO))
        };

        spend_value_gen.prop_flat_map(move |spend_value| {
            let output_value_gen = if flags.outputs_enabled {
                Strategy::boxed(arb_note_value_bounded(MAX_NOTE_VALUE / n_actions as u64))
            } else {
                Strategy::boxed(Just(NoteValue::ZERO))
            };

            output_value_gen.prop_flat_map(move |output_value| {
                arb_action(spend_value, output_value)
                    .prop_map(move |a| (spend_value - output_value, a))
            })
        })
    }

    prop_compose! {
        /// Create an arbitrary set of flags with cross-address transfers enabled, so
        /// that the flag set is representable in both pre-NU6.3 and NU6.3 formats.
        ///
        /// Use `arb_flags_nu6_3` for a strategy that can also disable cross-address
        /// transfers, which is representable only under NU6.3 encoding rules.
        pub fn arb_flags()(spends_enabled in prop::bool::ANY, outputs_enabled in prop::bool::ANY) -> Flags {
            Flags::from_parts(spends_enabled, outputs_enabled)
        }
    }

    prop_compose! {
        /// Create an arbitrary set of flags under NU6.3 encoding rules.
        pub fn arb_flags_nu6_3()(
            spends_enabled in prop::bool::ANY,
            outputs_enabled in prop::bool::ANY,
            cross_address_enabled in prop::bool::ANY,
        ) -> Flags {
            Flags {
                spends_enabled,
                outputs_enabled,
                cross_address_enabled,
            }
        }
    }

    prop_compose! {
        fn arb_base()(bytes in prop::array::uniform32(0u8..)) -> pallas::Base {
            // Instead of rejecting out-of-range bytes, let's reduce them.
            let mut buf = [0; 64];
            buf[..32].copy_from_slice(&bytes);
            pallas::Base::from_uniform_bytes(&buf)
        }
    }

    prop_compose! {
        /// Generate an arbitrary unauthorized bundle. This bundle does not
        /// necessarily respect consensus rules; for that use
        /// [`crate::builder::testing::arb_bundle`]
        pub fn arb_unauthorized_bundle(n_actions: usize)
        (
            flags in arb_flags(),
        )
        (
            acts in vec(arb_unauthorized_action_n(n_actions, flags), n_actions),
            anchor in arb_base().prop_map(Anchor::from),
            flags in Just(flags)
        ) -> Bundle<Unauthorized, ValueSum> {
            let (balances, actions): (Vec<ValueSum>, Vec<Action<_>>) = acts.into_iter().unzip();

            Bundle::from_parts(
                NonEmpty::from_vec(actions).unwrap(),
                flags,
                balances.into_iter().sum::<Result<ValueSum, _>>().unwrap(),
                anchor,
                super::EffectsOnly,
            )
        }
    }

    prop_compose! {
        /// Generate an arbitrary bundle with fake authorization data. This bundle does not
        /// necessarily respect consensus rules; for that use
        /// [`crate::builder::testing::arb_bundle`]
        pub fn arb_bundle(n_actions: usize)
        (
            flags in arb_flags(),
        )
        (
            acts in vec(arb_action_n(n_actions, flags), n_actions),
            anchor in arb_base().prop_map(Anchor::from),
            sk in arb_binding_signing_key(),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY),
            // A fake proof of the canonical length, so the bundle passes `try_from_parts`.
            fake_proof in vec(prop::num::u8::ANY, Proof::expected_proof_size(n_actions)),
            fake_sighash in prop::array::uniform32(prop::num::u8::ANY),
            flags in Just(flags)
        ) -> Bundle<Authorized, ValueSum> {
            let (balances, actions): (Vec<ValueSum>, Vec<Action<_>>) = acts.into_iter().unzip();
            let rng = StdRng::from_seed(rng_seed);

            Bundle::try_from_parts(
                NonEmpty::from_vec(actions).unwrap(),
                flags,
                balances.into_iter().sum::<Result<ValueSum, _>>().unwrap(),
                anchor,
                Authorized {
                    proof: Proof::new(fake_proof),
                    binding_signature: sk.sign(rng, &fake_sighash),
                },
                super::ProofSizeEnforcement::Strict
            )
            .expect("fake proof has the canonical length")
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use alloc::vec;

    use proptest::prelude::*;

    use super::testing::{arb_bundle, arb_flags_nu6_3};
    use super::{
        Authorized, Bundle, BundleError, BundleFormat, BundleProtocol, CommitmentError, Flags,
    };
    use crate::Proof;

    #[cfg(feature = "circuit")]
    pub(crate) fn with_cross_address_disabled(
        bundle: Bundle<Authorized, crate::value::ValueSum>,
    ) -> Bundle<Authorized, crate::value::ValueSum> {
        let mut flags = *bundle.flags();
        flags.cross_address_enabled = false;

        Bundle::from_parts_unchecked(
            bundle.actions().clone(),
            flags,
            *bundle.value_balance(),
            *bundle.anchor(),
            bundle.authorization().clone(),
        )
    }

    #[cfg(feature = "circuit")]
    pub(crate) fn sample_authorized_bundle(
        n_actions: usize,
    ) -> Bundle<Authorized, crate::value::ValueSum> {
        use proptest::strategy::ValueTree;

        let mut runner = proptest::test_runner::TestRunner::deterministic();
        arb_bundle(n_actions)
            .new_tree(&mut runner)
            .expect("strategy can generate a bundle")
            .current()
    }

    #[test]
    fn flags_byte_encoding() {
        for (flags, pre_nu6_3, nu6_3) in [
            (Flags::ENABLED, Some(0b011), Some(0b111)),
            (Flags::SPENDS_DISABLED, Some(0b010), Some(0b110)),
            (Flags::OUTPUTS_DISABLED, Some(0b001), Some(0b101)),
            // Disabling cross-address transfers is representable only under NU6.3
            // encoding rules.
            (Flags::CROSS_ADDRESS_DISABLED, None, Some(0b011)),
        ] {
            assert_eq!(flags.to_byte(BundleFormat::PreNu6_3), pre_nu6_3);
            assert_eq!(flags.to_byte(BundleFormat::Nu6_3), nu6_3);
        }
    }

    #[test]
    fn bundle_protocol_selects_valid_semantics() {
        #[cfg(feature = "circuit")]
        {
            assert_eq!(
                BundleProtocol::OrchardPreNu6_2.circuit_version(),
                crate::circuit::OrchardCircuitVersion::InsecurePreNu6_2
            );
            assert_eq!(
                BundleProtocol::OrchardPreNu6_3.circuit_version(),
                crate::circuit::OrchardCircuitVersion::FixedPostNu6_2
            );
            assert_eq!(
                BundleProtocol::OrchardPostNu6_3.circuit_version(),
                crate::circuit::OrchardCircuitVersion::PostNu6_3
            );
            assert_eq!(
                BundleProtocol::IronwoodPostNu6_3.circuit_version(),
                crate::circuit::OrchardCircuitVersion::PostNu6_3
            );
        }

        assert_eq!(
            BundleProtocol::OrchardPreNu6_2.bundle_format(),
            BundleFormat::PreNu6_3
        );
        assert_eq!(
            BundleProtocol::OrchardPreNu6_3.bundle_format(),
            BundleFormat::PreNu6_3
        );
        assert_eq!(
            BundleProtocol::OrchardPostNu6_3.bundle_format(),
            BundleFormat::Nu6_3
        );
        assert_eq!(
            BundleProtocol::IronwoodPostNu6_3.bundle_format(),
            BundleFormat::Nu6_3
        );

        assert!(BundleProtocol::OrchardPostNu6_3.requires_cross_address_restriction());
        assert!(!BundleProtocol::IronwoodPostNu6_3.requires_cross_address_restriction());

        assert_eq!(
            BundleProtocol::OrchardPostNu6_3.default_note_version(),
            crate::note::NoteVersion::V2
        );
        assert_eq!(
            BundleProtocol::IronwoodPostNu6_3.default_note_version(),
            crate::note::NoteVersion::V3
        );
    }

    #[test]
    fn flags_parsing_diverges_between_eras() {
        // A byte with bit 2 clear parses as an unrestricted bundle pre-NU6.3 and a
        // restricted bundle under NU6.3.
        for value in 0b000..=0b011 {
            let pre_nu6_3_flags = Flags::from_byte(value, BundleFormat::PreNu6_3).unwrap();
            let nu6_3_flags = Flags::from_byte(value, BundleFormat::Nu6_3).unwrap();

            assert_eq!(
                pre_nu6_3_flags.spends_enabled(),
                nu6_3_flags.spends_enabled()
            );
            assert_eq!(
                pre_nu6_3_flags.outputs_enabled(),
                nu6_3_flags.outputs_enabled()
            );
            assert!(pre_nu6_3_flags.cross_address_enabled());
            assert!(!nu6_3_flags.cross_address_enabled());

            // Each parse round-trips to the same byte under its own era, but the
            // restricted set is unrepresentable pre-NU6.3.
            assert_eq!(pre_nu6_3_flags.to_byte(BundleFormat::PreNu6_3), Some(value));
            assert_eq!(nu6_3_flags.to_byte(BundleFormat::Nu6_3), Some(value));
            assert_eq!(nu6_3_flags.to_byte(BundleFormat::PreNu6_3), None);
        }

        assert_eq!(
            Flags::from_byte(0b011, BundleFormat::PreNu6_3),
            Some(Flags::ENABLED)
        );
        assert_eq!(
            Flags::from_byte(0b011, BundleFormat::Nu6_3),
            Some(Flags::CROSS_ADDRESS_DISABLED)
        );
    }

    #[test]
    fn pre_nu6_3_flags_parsing_rejects_reserved_bits() {
        for value in 0b100..=u8::MAX {
            assert_eq!(Flags::from_byte(value, BundleFormat::PreNu6_3), None);
        }
    }

    #[test]
    fn nu6_3_flags_parsing_recognizes_cross_address_enabled() {
        for value in 0b100..=0b111 {
            let flags = Flags::from_byte(value, BundleFormat::Nu6_3).unwrap();

            assert!(flags.cross_address_enabled());
            assert_eq!(flags.to_byte(BundleFormat::Nu6_3), Some(value));
            // Pre-NU6.3 formats encode the same flag set with bit 2 reserved zero.
            assert_eq!(flags.to_byte(BundleFormat::PreNu6_3), Some(value & 0b011));
        }

        for value in 0b1000..=u8::MAX {
            assert_eq!(Flags::from_byte(value, BundleFormat::Nu6_3), None);
        }
    }

    #[test]
    fn expected_proof_size_matches_known_values() {
        // The canonical proof sizes for one and two actions, fixed by the action circuit.
        assert_eq!(Proof::expected_proof_size(1), 4992);
        assert_eq!(Proof::expected_proof_size(2), 7264);

        // The size is affine in the number of actions: each action contributes a fixed amount.
        let per_action = Proof::expected_proof_size(2) - Proof::expected_proof_size(1);
        assert_eq!(
            Proof::expected_proof_size(3) - Proof::expected_proof_size(2),
            per_action,
        );
    }

    proptest! {
        // The property is deterministic given the actions, so a handful of cases suffices.
        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn arb_flags_nu6_3_round_trips(flags in arb_flags_nu6_3()) {
            let encoded = flags
                .to_byte(BundleFormat::Nu6_3)
                .expect("all NU6.3 flag strategy outputs encode under NU6.3");

            prop_assert_eq!(Flags::from_byte(encoded, BundleFormat::Nu6_3), Some(flags));
        }

        #[test]
        fn commitment_hashes_the_wire_flag_byte(bundle in arb_bundle(3)) {
            // Rebuild the bundle with `V = i64` so that `commitment()` is available.
            let bundle = Bundle::from_parts_unchecked(
                bundle.actions().clone(),
                *bundle.flags(),
                0i64,
                *bundle.anchor(),
                bundle.authorization().clone(),
            );
            let mut flags = *bundle.flags();
            flags.cross_address_enabled = false;

            let restricted = Bundle::from_parts_unchecked(
                bundle.actions().clone(),
                flags,
                *bundle.value_balance(),
                *bundle.anchor(),
                bundle.authorization().clone(),
            );

            // The restricted bundle's NU6.3 wire byte equals the unrestricted bundle's
            // pre-NU6.3 byte, so their commitments agree.
            prop_assert_eq!(
                restricted.flags().to_byte(BundleFormat::Nu6_3),
                bundle.flags().to_byte(BundleFormat::PreNu6_3)
            );
            let restricted_commitment: [u8; 32] = restricted
                .commitment(BundleFormat::Nu6_3)
                .expect("restricted flags are representable under NU6.3")
                .into();
            let legacy_commitment: [u8; 32] = bundle
                .commitment(BundleFormat::PreNu6_3)
                .expect("unrestricted flags are representable pre-NU6.3")
                .into();
            prop_assert_eq!(restricted_commitment, legacy_commitment);

            // The unrestricted NU6.3 encoding sets bit 2, producing a distinct digest.
            let unrestricted_commitment: [u8; 32] = bundle
                .commitment(BundleFormat::Nu6_3)
                .expect("unrestricted flags are representable under NU6.3")
                .into();
            prop_assert_ne!(unrestricted_commitment, restricted_commitment);

            // The restricted flag set cannot be committed under pre-NU6.3 encoding.
            prop_assert_eq!(restricted.flags().to_byte(BundleFormat::PreNu6_3), None);
            prop_assert!(matches!(
                restricted.commitment(BundleFormat::PreNu6_3),
                Err(CommitmentError::UnrepresentableFlags)
            ));
        }

        #[test]
        fn try_from_parts_enforces_canonical_proof_size(bundle in arb_bundle(3)) {
            let actions = bundle.actions().clone();
            let expected = Proof::expected_proof_size(actions.len());
            let flags = *bundle.flags();
            let value_balance = *bundle.value_balance();
            let anchor = *bundle.anchor();
            let binding_signature = bundle.authorization().binding_signature().clone();

            let with_proof_len = |proof_len: usize| {
                Bundle::try_from_parts(
                    actions.clone(),
                    flags,
                    value_balance,
                    anchor,
                    Authorized::from_parts(
                        Proof::new(vec![0u8; proof_len]),
                        binding_signature.clone(),
                    ),
                    crate::bundle::ProofSizeEnforcement::Strict
                )
            };

            // A canonically-sized proof is accepted.
            prop_assert!(with_proof_len(expected).is_ok());

            // A proof padded with trailing data is rejected (the GHSA-2x4w-pxqw-58v9 attack).
            prop_assert_eq!(
                with_proof_len(expected + 1).err(),
                Some(BundleError::NonCanonicalProofSize { expected, actual: expected + 1 })
            );

            // A truncated proof is rejected.
            prop_assert_eq!(
                with_proof_len(expected - 1).err(),
                Some(BundleError::NonCanonicalProofSize { expected, actual: expected - 1 })
            );
        }

        #[test]
        fn try_from_parts_preserves_cross_address_disabled(bundle in arb_bundle(3)) {
            let actions = bundle.actions().clone();
            let mut flags = *bundle.flags();
            flags.cross_address_enabled = false;
            let value_balance = *bundle.value_balance();
            let anchor = *bundle.anchor();
            let authorization = bundle.authorization().clone();

            let bundle = Bundle::try_from_parts(
                    actions,
                    flags,
                    value_balance,
                    anchor,
                    authorization,
                    crate::bundle::ProofSizeEnforcement::Strict,
                )
                .expect("canonical proof size is accepted");
            prop_assert!(!bundle.flags().cross_address_enabled());
        }

        #[test]
        fn try_from_parts_checks_proof_size_with_cross_address_disabled(bundle in arb_bundle(3)) {
            let actions = bundle.actions().clone();
            let expected = Proof::expected_proof_size(actions.len());
            let mut flags = *bundle.flags();
            flags.cross_address_enabled = false;
            let value_balance = *bundle.value_balance();
            let anchor = *bundle.anchor();
            let binding_signature = bundle.authorization().binding_signature().clone();

            prop_assert_eq!(
                Bundle::try_from_parts(
                    actions,
                    flags,
                    value_balance,
                    anchor,
                    Authorized::from_parts(
                        Proof::new(vec![0u8; expected + 1]),
                        binding_signature,
                    ),
                    crate::bundle::ProofSizeEnforcement::Strict,
                )
                .err(),
                Some(BundleError::NonCanonicalProofSize { expected, actual: expected + 1 })
            );
        }
    }

    #[cfg(feature = "circuit")]
    #[test]
    fn commitment_fails_for_unrepresentable_flags() {
        let bundle = with_cross_address_disabled(sample_authorized_bundle(1))
            .try_map_value_balance(i64::try_from)
            .expect("generated bundle value balance fits in i64");

        assert!(matches!(
            bundle.commitment(BundleFormat::PreNu6_3),
            Err(CommitmentError::UnrepresentableFlags)
        ));
    }

    #[cfg(feature = "circuit")]
    #[test]
    fn verify_proof_rejects_cross_address_disabled_for_unsupported_keys() {
        let bundle = with_cross_address_disabled(sample_authorized_bundle(1));

        for circuit_version in [
            crate::circuit::OrchardCircuitVersion::InsecurePreNu6_2,
            crate::circuit::OrchardCircuitVersion::FixedPostNu6_2,
        ] {
            let vk = crate::circuit::VerifyingKey::build(circuit_version);

            assert!(matches!(
                bundle.verify_proof(&vk),
                Err(halo2_proofs::plonk::Error::InvalidInstances)
            ));
        }
    }
}
