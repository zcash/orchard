//! Issuance logic for Zcash Shielded Assets (ZSAs).
//!
//! This module defines structures and methods for creating, authorizing, and verifying
//! issuance bundles, which introduce new shielded assets into the Orchard protocol.
//!
//! The core components include:
//! - `IssueBundle`: Represents a collection of issuance actions with authorization states.
//! - `IssueAction`: Defines an individual issuance event, including asset details and notes.
//! - `IssueAuth` variants: Track issuance states from creation to finalization.
//! - `verify_issue_bundle`: Ensures issuance validity and prevents unauthorized asset creation.
//!
//! Errors related to issuance, such as invalid signatures or supply overflows,
//! are handled through the `Error` enum.

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use blake2b_simd::{Hash as Blake2bHash, Params};
use core::fmt;
use core::fmt::Debug;
use group::Group;
use nonempty::NonEmpty;
use rand::RngCore;

use crate::{
    asset_record::AssetRecord,
    bundle::commitments::{hash_issue_bundle_auth_data, hash_issue_bundle_txid_data},
    constants::reference_keys::ReferenceKeys,
    issuance_auth::{IssueAuthKey, IssueAuthSig, IssueValidatingKey},
    note::{rho_for_issuance_note, AssetBase, Nullifier, Rho},
    value::NoteValue,
    Address, Note,
};

use crate::issuance_auth::ZSASchnorr;
use Error::{
    AssetBaseCannotBeIdentityPoint, CannotBeFirstIssuance, IncorrectRhoDerivation,
    InvalidIssueAuthKey, InvalidIssueBundleSig, InvalidIssueValidatingKey, IssueActionNotFound,
    IssueActionPreviouslyFinalizedAssetBase, IssueActionWithoutNoteNotFinalized,
    IssueBundleIkMismatchAssetBase, MissingReferenceNoteOnFirstIssuance, ValueOverflow,
};

/// Checks if a given note is a reference note.
///
/// A reference note satisfies the following conditions:
/// - The note's value is zero.
/// - The note's recipient matches the reference recipient.
fn is_reference_note(note: &Note) -> bool {
    note.value() == NoteValue::zero() && note.recipient() == ReferenceKeys::recipient()
}

/// A bundle of actions to be applied to the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueBundle<T: IssueAuth> {
    /// The issuer key for the note being created.
    ik: IssueValidatingKey<ZSASchnorr>,
    /// The list of issue actions that make up this bundle.
    actions: NonEmpty<IssueAction>,
    /// The authorization for this action.
    authorization: T,
}

/// An issue action applied to the global ledger.
///
/// Externally, this creates new zsa notes (adding a commitment to the global ledger).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueAction {
    /// Asset description hash for verification.
    asset_desc_hash: [u8; 32],
    /// The newly issued notes.
    notes: Vec<Note>,
    /// `finalize` will prevent further issuance of the same asset type.
    finalize: bool,
}

/// The parameters required to add a Note into an IssueAction.
#[derive(Debug)]
pub struct IssueInfo {
    /// The recipient of the funds.
    pub recipient: Address,
    /// The value of this note.
    pub value: NoteValue,
}

/// Compute the asset description hash for a given asset description.
///
/// # Panics
///
/// Panics if `asset_desc` is not well-formed as per Unicode 15.0 specification, Section 3.9, D92.
pub fn compute_asset_desc_hash(asset_desc: &NonEmpty<u8>) -> [u8; 32] {
    if String::from_utf8(asset_desc.iter().copied().collect::<Vec<u8>>()).is_err() {
        panic!("asset_desc is not a well-formed Unicode string");
    }
    let mut ah = Params::new()
        .hash_length(32)
        .personal(b"ZSA-AssetDescCRH")
        .to_state();
    ah.update(&[asset_desc.head]);
    ah.update(asset_desc.tail.as_slice());
    ah.finalize()
        .as_bytes()
        .try_into()
        .expect("Invalid asset description hash length")
}

impl IssueAction {
    /// Constructs a new `IssueAction`.
    pub fn new_with_flags(asset_desc_hash: [u8; 32], notes: Vec<Note>, flags: u8) -> Option<Self> {
        let finalize = match flags {
            0b0000_0000 => false,
            0b0000_0001 => true,
            _ => return None,
        };
        Some(IssueAction {
            asset_desc_hash,
            notes,
            finalize,
        })
    }

    /// Constructs an `IssueAction` from its constituent parts.
    pub fn from_parts(asset_desc_hash: [u8; 32], notes: Vec<Note>, finalize: bool) -> Self {
        IssueAction {
            asset_desc_hash,
            notes,
            finalize,
        }
    }

    /// Returns the asset description hash for the note being created.
    pub fn asset_desc_hash(&self) -> &[u8; 32] {
        &self.asset_desc_hash
    }

    /// Returns the issued notes.
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// Returns whether the asset type was finalized in this action.
    pub fn is_finalized(&self) -> bool {
        self.finalize
    }

    /// Verifies and computes the new asset supply for an `IssueAction`.
    ///
    /// This function calculates the total value (supply) of the asset by summing the values
    /// of all its notes and ensures that all note types are equal. It returns the asset and
    /// its supply as a tuple (`AssetBase`, `NoteValue`) or an error if the asset was not
    /// properly derived or an overflow occurred during the supply amount calculation.
    ///
    /// # Arguments
    ///
    /// * `ik` - A reference to the `IssueValidatingKey` used for deriving the asset.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple with an `AssetBase` and an `NoteValue`, or an `Error`.
    ///
    /// # Errors
    ///
    /// This function may return an error in any of the following cases:
    ///
    /// * `ValueOverflow`: The total amount value of all notes in the `IssueAction` overflows.
    /// * `IssueBundleIkMismatchAssetBase`: The provided `ik` is not used to derive the
    ///   `AssetBase` for **all** internal notes.
    /// * `AssetBaseCannotBeIdentityPoint`: The derived `AssetBase` is the identity point of the
    ///    Pallas curve.
    /// * `IssueActionWithoutNoteNotFinalized`: The `IssueAction` contains no notes and is not finalized.
    fn verify(&self, ik: &IssueValidatingKey<ZSASchnorr>) -> Result<(AssetBase, NoteValue), Error> {
        if self.notes.is_empty() && !self.is_finalized() {
            return Err(IssueActionWithoutNoteNotFinalized);
        }

        let issue_asset = AssetBase::derive(ik, &self.asset_desc_hash);

        // The new asset should not be the identity point of the Pallas curve.
        if bool::from(issue_asset.cv_base().is_identity()) {
            return Err(AssetBaseCannotBeIdentityPoint);
        }

        // Calculate the value of the asset as a sum of values of all its notes
        // and ensure all note types are equal the asset derived from asset_desc_hash and ik.
        let value_sum = self
            .notes
            .iter()
            .try_fold(NoteValue::zero(), |value_sum, &note| {
                // All assets should be derived correctly
                if note.asset() != issue_asset {
                    return Err(IssueBundleIkMismatchAssetBase);
                }

                // The total amount should not overflow
                (value_sum + note.value()).ok_or(ValueOverflow)
            })?;

        Ok((issue_asset, value_sum))
    }

    /// Serialize `finalize` flag to a byte
    #[allow(clippy::bool_to_int_with_if)]
    pub fn flags(&self) -> u8 {
        if self.finalize {
            0b0000_0001
        } else {
            0b0000_0000
        }
    }

    /// Returns the reference note if the first note matches the reference note criteria.
    ///
    /// A reference note must be the first note in the `notes` vector and satisfy the following:
    /// - The note's value is zero.
    /// - The note's recipient matches the reference recipient.
    pub fn get_reference_note(&self) -> Option<&Note> {
        self.notes.first().filter(|note| is_reference_note(note))
    }
}

/// Defines the authorization type of an Issue bundle.
pub trait IssueAuth: fmt::Debug + Clone {}

/// Marker type for a bundle that contains no authorizing data.
#[derive(Clone, Debug)]
pub struct EffectsOnly;

impl IssueAuth for EffectsOnly {}

/// Marker for an unsigned bundle with no nullifier and no sighash injected.
#[derive(Debug, Clone)]
pub struct AwaitingNullifier;

/// Marker for an unsigned bundle with a Nullifier injected.
#[derive(Debug, Clone)]
pub struct AwaitingSighash;

/// Marker for an unsigned bundle with both Sighash and Nullifier injected.
#[derive(Debug, Clone)]
pub struct Prepared {
    sighash: [u8; 32],
}

/// Marker for an authorized bundle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signed {
    signature: IssueAuthSig<ZSASchnorr>,
}

impl Signed {
    /// Returns the signature for this authorization.
    pub fn signature(&self) -> &IssueAuthSig<ZSASchnorr> {
        &self.signature
    }

    /// Constructs a `Signed` from a byte array containing an `IssueAuthSig` in raw bytes.
    pub fn from_data(data: &[u8]) -> Self {
        Signed {
            signature: IssueAuthSig::decode(data).unwrap(),
        }
    }
}

impl IssueAuth for AwaitingNullifier {}
impl IssueAuth for AwaitingSighash {}
impl IssueAuth for Prepared {}
impl IssueAuth for Signed {}

impl<T: IssueAuth> IssueBundle<T> {
    /// Returns the issuer verification key for the bundle.
    pub fn ik(&self) -> &IssueValidatingKey<ZSASchnorr> {
        &self.ik
    }
    /// Return the actions for a given `IssueBundle`.
    pub fn actions(&self) -> &NonEmpty<IssueAction> {
        &self.actions
    }
    /// Return the notes from all actions for a given `IssueBundle`.
    pub fn get_all_notes(&self) -> Vec<&Note> {
        self.actions.iter().flat_map(|a| a.notes.iter()).collect()
    }

    /// Returns the authorization for this issue bundle.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Find the action corresponding to the `asset_desc_hash` for a given `IssueBundle`.
    ///
    /// # Returns
    ///
    /// If a single matching action is found, it is returned as `Some(&IssueAction)`.
    /// If no action matches the given `asset_desc_hash`, it returns `None`.
    ///
    /// # Panics
    ///
    /// Panics if multiple matching actions are found.
    pub fn get_action_by_desc_hash(&self, asset_desc_hash: &[u8; 32]) -> Option<&IssueAction> {
        let issue_actions: Vec<&IssueAction> = self
            .actions
            .iter()
            .filter(|a| a.asset_desc_hash.eq(asset_desc_hash))
            .collect();
        match issue_actions.len() {
            0 => None,
            1 => Some(issue_actions[0]),
            _ => panic!("Multiple IssueActions with the same asset_desc_hash"),
        }
    }

    /// Find the actions corresponding to an Asset Base `asset` for a given `IssueBundle`.
    ///
    /// # Returns
    ///
    /// Returns `Some(&IssueAction)` if a single matching action is found.
    /// Returns `None` if no action matches the given asset base.
    ///
    /// # Panics
    ///
    /// Panics if multiple matching actions are found.
    pub fn get_action_by_asset(&self, asset: &AssetBase) -> Option<&IssueAction> {
        let issue_actions: Vec<&IssueAction> = self
            .actions
            .iter()
            .filter(|a| AssetBase::derive(&self.ik, &a.asset_desc_hash).eq(asset))
            .collect();
        match issue_actions.len() {
            0 => None,
            1 => Some(issue_actions[0]),
            _ => panic!("Multiple IssueActions with the same AssetBase"),
        }
    }

    /// Computes a commitment to the effects of this bundle, suitable for inclusion within
    /// a transaction ID.
    pub fn commitment(&self) -> IssueBundleCommitment {
        IssueBundleCommitment(hash_issue_bundle_txid_data(self))
    }

    /// Constructs an `IssueBundle` from its constituent parts.
    pub fn from_parts(
        ik: IssueValidatingKey<ZSASchnorr>,
        actions: NonEmpty<IssueAction>,
        authorization: T,
    ) -> Self {
        IssueBundle {
            ik,
            actions,
            authorization,
        }
    }

    /// Transitions this bundle from one authorization state to another.
    pub fn map_authorization<T2: IssueAuth>(
        self,
        map_auth: impl FnOnce(T) -> T2,
    ) -> IssueBundle<T2> {
        let authorization = self.authorization;
        IssueBundle {
            ik: self.ik,
            actions: self.actions,
            authorization: map_auth(authorization),
        }
    }
}

impl IssueBundle<AwaitingNullifier> {
    /// Constructs a new `IssueBundle`.
    ///
    /// If issue_info is None, the new `IssueBundle` will contain one `IssueAction` without notes
    /// and with `finalize` set to true.
    /// Otherwise, the new `IssueBundle` will contain one `IssueAction` with one note created from
    /// issue_info values and with `finalize` set to false. In this created note, rho will be
    /// set to zero. The rho value will be updated later by calling the `update_rho` method.
    ///
    /// If `first_issuance` is true, the `IssueBundle` will contain a reference note for the asset
    /// defined by (`asset_desc_hash`, `ik`).
    pub fn new(
        ik: IssueValidatingKey<ZSASchnorr>,
        asset_desc_hash: [u8; 32],
        issue_info: Option<IssueInfo>,
        first_issuance: bool,
        mut rng: impl RngCore,
    ) -> (IssueBundle<AwaitingNullifier>, AssetBase) {
        let asset = AssetBase::derive(&ik, &asset_desc_hash);

        let mut notes = vec![];
        if first_issuance {
            notes.push(create_reference_note(asset, &mut rng));
        };

        let action = match issue_info {
            None => IssueAction {
                asset_desc_hash,
                notes,
                finalize: true,
            },
            Some(issue_info) => {
                let note = Note::new(
                    issue_info.recipient,
                    issue_info.value,
                    asset,
                    Rho::zero(),
                    &mut rng,
                );

                notes.push(note);

                IssueAction {
                    asset_desc_hash,
                    notes,
                    finalize: false,
                }
            }
        };

        (
            IssueBundle {
                ik,
                actions: NonEmpty::new(action),
                authorization: AwaitingNullifier,
            },
            asset,
        )
    }

    /// Add a new note to the `IssueBundle`.
    ///
    /// Rho is set to zero. The rho value will be updated later by calling the `update_rho` method.
    /// If `first_issuance` is true, we will also add a reference note for the asset defined by
    /// (`asset_desc_hash`, `ik`).
    pub fn add_recipient(
        &mut self,
        asset_desc_hash: [u8; 32],
        recipient: Address,
        value: NoteValue,
        first_issuance: bool,
        mut rng: impl RngCore,
    ) -> Result<AssetBase, Error> {
        let asset = AssetBase::derive(&self.ik, &asset_desc_hash);

        let note = Note::new(recipient, value, asset, Rho::zero(), &mut rng);

        let notes = if first_issuance {
            vec![create_reference_note(asset, &mut rng), note]
        } else {
            vec![note]
        };

        let action = self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.asset_desc_hash.eq(&asset_desc_hash));

        match action {
            Some(action) => {
                // Append to an existing IssueAction.
                if first_issuance {
                    // It cannot be first issuance because we have already some notes for this asset.
                    return Err(CannotBeFirstIssuance);
                }
                action.notes.extend(notes);
            }
            None => {
                // Insert a new IssueAction.
                self.actions.push(IssueAction {
                    asset_desc_hash,
                    notes,
                    finalize: false,
                });
            }
        };

        Ok(asset)
    }

    /// Finalizes a given `IssueAction`
    pub fn finalize_action(&mut self, asset_desc_hash: &[u8; 32]) -> Result<(), Error> {
        match self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.asset_desc_hash.eq(asset_desc_hash))
        {
            Some(issue_action) => {
                issue_action.finalize = true;
            }
            None => {
                return Err(IssueActionNotFound);
            }
        }

        Ok(())
    }

    /// Compute the correct rho value for each note in the bundle according to
    /// [ZIP-227: Issuance of Zcash Shielded Assets][zip227].
    ///
    /// [zip227]: https://zips.z.cash/zip-0227
    pub fn update_rho(self, first_nullifier: &Nullifier) -> IssueBundle<AwaitingSighash> {
        let mut bundle = self;
        bundle
            .actions
            .iter_mut()
            .enumerate()
            .for_each(|(index_action, action)| {
                action
                    .notes
                    .iter_mut()
                    .enumerate()
                    .for_each(|(index_note, note)| {
                        note.update_rho_for_issuance_note(
                            first_nullifier,
                            index_action.try_into().unwrap(),
                            index_note.try_into().unwrap(),
                        );
                    });
            });
        bundle.map_authorization(|_| AwaitingSighash)
    }
}

impl IssueBundle<AwaitingSighash> {
    /// Loads the sighash into the bundle, as preparation for signing.
    pub fn prepare(self, sighash: [u8; 32]) -> IssueBundle<Prepared> {
        IssueBundle {
            ik: self.ik,
            actions: self.actions,
            authorization: Prepared { sighash },
        }
    }
}

fn create_reference_note(asset: AssetBase, mut rng: impl RngCore) -> Note {
    Note::new(
        ReferenceKeys::recipient(),
        NoteValue::zero(),
        asset,
        Rho::zero(),
        &mut rng,
    )
}

impl IssueBundle<Prepared> {
    /// Sign the `IssueBundle`.
    /// The call makes sure that the provided `isk` matches the `ik` and the derived `asset` for each note in the bundle.
    pub fn sign(self, isk: &IssueAuthKey<ZSASchnorr>) -> Result<IssueBundle<Signed>, Error> {
        let expected_ik = IssueValidatingKey::from(isk);

        // Make sure the `expected_ik` matches the `asset` for all notes.
        self.actions.iter().try_for_each(|action| {
            action.verify(&expected_ik)?;
            Ok(())
        })?;

        // Make sure the signature can be generated.
        let signature = isk
            .try_sign(&self.authorization.sighash)
            .map_err(|_| InvalidIssueBundleSig)?;

        Ok(IssueBundle {
            ik: self.ik,
            actions: self.actions,
            authorization: Signed { signature },
        })
    }
}

/// A commitment to a bundle of actions.
///
/// This commitment is non-malleable, in the sense that a bundle's commitment will only
/// change if the effects of the bundle are altered.
#[derive(Debug)]
pub struct IssueBundleCommitment(pub Blake2bHash);

impl From<IssueBundleCommitment> for [u8; 32] {
    /// Serializes issue bundle commitment as byte array
    fn from(commitment: IssueBundleCommitment) -> Self {
        // The commitment uses BLAKE2b-256.
        commitment.0.as_bytes().try_into().unwrap()
    }
}

/// A commitment to the authorizing data within a bundle of actions.
#[derive(Debug)]
pub struct IssueBundleAuthorizingCommitment(pub Blake2bHash);

impl IssueBundle<Signed> {
    /// Computes a commitment to the authorizing data contained in this bundle.
    ///
    /// This together with `IssueBundle::commitment` bind the entire bundle.
    pub fn authorizing_commitment(&self) -> IssueBundleAuthorizingCommitment {
        IssueBundleAuthorizingCommitment(hash_issue_bundle_auth_data(self))
    }
}

/// Validates an [`IssueBundle`] by performing the following checks:
///
/// - **IssueBundle Auth signature verification**:
///   - Ensures the signature on the provided `sighash` matches the bundle's authorization.
/// - **Static IssueAction verification**:
///   - Runs checks using the `IssueAction::verify` method.
/// - **Node global state related verification**:
///   - Ensures the total supply value does not overflow when adding the new amount to the existing supply.
///   - Verifies that the `AssetBase` has not already been finalized.
///   - Requires a reference note for the *first issuance* of an asset; subsequent issuance may omit it.
/// - **Rho computation**:
///   - Ensures that the `rho` value of each issuance note is correctly computed from the given
///     `first_nullifier`.
///
/// # Arguments
///
/// * `bundle`: A reference to the [`IssueBundle`] to be validated.
/// * `sighash`: A 32-byte array representing the `sighash` used to verify the bundle's signature.
/// * `get_global_asset_state`: A closure that takes a reference to an [`AssetBase`] and returns an
///   [`Option<AssetRecord>`], representing the current state of the asset from a global store
///   of previously issued assets.
/// * `first_nullifier`: A reference to a [`Nullifier`] that is used to compute the `rho` value of
///   each issuance note.
///
/// # Returns
///
/// A `Result` containing a [`BTreeMap<AssetBase, AssetRecord>`] upon success, where each key-value
/// pair represents the new or updated state of an asset. The key is an [`AssetBase`], and the value
/// is the corresponding updated [`AssetRecord`].
///
/// # Errors
///
/// * `IssueBundleInvalidSignature`: Signature verification for the provided `sighash` fails.
/// * `ValueOverflow`: adding the new amount to the existing total supply causes an overflow.
/// * `IssueActionPreviouslyFinalizedAssetBase`: An action is attempted on an asset that has
///   already been finalized.
/// * `MissingReferenceNoteOnFirstIssuance`: No reference note is provided for the first
///   issuance of a new asset.
/// * `IncorrectRhoDerivation`: If the `rho` value of any issuance note is not correctly derived
///   from the `first_nullifier`.
/// * **Other Errors**: Any additional errors returned by the `IssueAction::verify` method are
///   propagated
pub fn verify_issue_bundle(
    bundle: &IssueBundle<Signed>,
    sighash: [u8; 32],
    get_global_records: impl Fn(&AssetBase) -> Option<AssetRecord>,
    first_nullifier: &Nullifier,
) -> Result<BTreeMap<AssetBase, AssetRecord>, Error> {
    bundle
        .ik()
        .verify(&sighash, bundle.authorization().signature())
        .map_err(|_| InvalidIssueBundleSig)?;

    bundle.actions().iter().enumerate().try_fold(
        BTreeMap::new(),
        |mut new_records, (index_action, action)| {
            // Check rho derivation for each note.
            for (index_note, note) in action.notes.iter().enumerate() {
                let expected_rho =
                    rho_for_issuance_note(first_nullifier, index_action as u32, index_note as u32);
                if note.rho() != expected_rho {
                    return Err(IncorrectRhoDerivation);
                }
            }

            let (asset, amount) = action.verify(bundle.ik())?;

            let is_finalized = action.is_finalized();
            let ref_note = action.get_reference_note();

            let new_asset_record = match new_records
                .get(&asset)
                .cloned()
                .or_else(|| get_global_records(&asset))
            {
                // The first issuance of the asset
                None => AssetRecord::new(
                    amount,
                    is_finalized,
                    *ref_note.ok_or(MissingReferenceNoteOnFirstIssuance)?,
                ),

                // Subsequent issuance of the asset
                Some(current_record) => {
                    let amount = (current_record.amount + amount).ok_or(ValueOverflow)?;

                    if current_record.is_finalized {
                        return Err(IssueActionPreviouslyFinalizedAssetBase);
                    }

                    AssetRecord::new(amount, is_finalized, current_record.reference_note)
                }
            };

            new_records.insert(asset, new_asset_record);

            Ok(new_records)
        },
    )
}

/// Errors produced during the issuance process
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The requested IssueAction not exists in the bundle.
    IssueActionNotFound,
    /// The provided `isk` and the derived `ik` does not match at least one note type.
    IssueBundleIkMismatchAssetBase,
    /// The `IssueAction` is not finalized but contains no notes.
    IssueActionWithoutNoteNotFinalized,
    /// The `AssetBase` is the Pallas identity point, which is invalid.
    AssetBaseCannotBeIdentityPoint,
    /// It cannot be first issuance because we have already some notes for this asset.
    CannotBeFirstIssuance,

    /// Signing errors:
    /// Invalid issuance authorizing key.
    InvalidIssueAuthKey,

    /// Verification errors:
    /// Invalid issuance validating key.
    InvalidIssueValidatingKey,
    /// Invalid IssueBundle signature.
    InvalidIssueBundleSig,
    /// The provided `AssetBase` has been previously finalized.
    IssueActionPreviouslyFinalizedAssetBase,
    /// The rho value of an issuance note is not correctly derived from the first nullifier.
    IncorrectRhoDerivation,

    /// Overflow error occurred while calculating the value of the asset
    ValueOverflow,

    /// No reference note is provided for the first issuance of a new asset.
    MissingReferenceNoteOnFirstIssuance,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueActionNotFound => {
                write!(f, "the requested IssueAction not exists in the bundle.")
            }
            IssueBundleIkMismatchAssetBase => {
                write!(
                    f,
                    "the provided `isk` and the derived `ik` do not match at least one note type"
                )
            }
            IssueActionWithoutNoteNotFinalized => {
                write!(
                    f,
                    "this `IssueAction` contains no notes but is not finalized"
                )
            }
            AssetBaseCannotBeIdentityPoint => {
                write!(
                    f,
                    "the AssetBase is the identity point of the Pallas curve, which is invalid."
                )
            }
            CannotBeFirstIssuance => {
                write!(
                    f,
                    "it cannot be first issuance because we have already some notes for this asset."
                )
            }
            InvalidIssueAuthKey => {
                write!(f, "invalid issuance authorizing key")
            }
            InvalidIssueValidatingKey => {
                write!(f, "invalid issuance validating key")
            }
            InvalidIssueBundleSig => {
                write!(f, "invalid IssueBundle signature")
            }
            IssueActionPreviouslyFinalizedAssetBase => {
                write!(f, "the provided `AssetBase` has been previously finalized")
            }
            IncorrectRhoDerivation => {
                write!(f, "incorrect rho value")
            }
            ValueOverflow => {
                write!(
                    f,
                    "overflow error occurred while calculating the value of the asset"
                )
            }
            MissingReferenceNoteOnFirstIssuance => {
                write!(
                    f,
                    "no reference note is provided for the first issuance of a new asset."
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        asset_record::AssetRecord,
        builder::{Builder, BundleType},
        circuit::ProvingKey,
        issuance::Error::{
            IncorrectRhoDerivation, InvalidIssueBundleSig, IssueActionNotFound,
            IssueActionPreviouslyFinalizedAssetBase, IssueBundleIkMismatchAssetBase,
        },
        issuance::{
            compute_asset_desc_hash, is_reference_note, verify_issue_bundle, IssueAction,
            IssueBundle, IssueInfo, Signed,
        },
        issuance_auth::{IssueAuthKey, IssueValidatingKey, ZSASchnorr},
        keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
        note::{rho_for_issuance_note, AssetBase, ExtractedNoteCommitment, Nullifier, Rho},
        orchard_flavor::OrchardZSA,
        tree::{MerkleHashOrchard, MerklePath},
        value::NoteValue,
        Address, Anchor, Bundle, Note,
    };
    use alloc::collections::BTreeMap;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use group::{Group, GroupEncoding};
    use incrementalmerkletree::{Marking, Retention};
    use nonempty::NonEmpty;
    use pasta_curves::pallas::{Point, Scalar};
    use rand::rngs::OsRng;
    use rand::RngCore;
    use shardtree::store::memory::MemoryShardStore;
    use shardtree::ShardTree;

    /// Validation for reference note
    ///
    /// The following checks are performed:
    /// - the note value of the reference note is equal to 0
    /// - the recipient of the reference note is equal to the reference recipient
    /// - the asset of the reference note is equal to the provided asset
    fn verify_reference_note(note: &Note, asset: AssetBase) {
        assert!(is_reference_note(note));
        assert_eq!(note.asset(), asset);
    }

    #[derive(Clone)]
    struct TestParams {
        rng: OsRng,
        isk: IssueAuthKey<ZSASchnorr>,
        ik: IssueValidatingKey<ZSASchnorr>,
        recipient: Address,
        sighash: [u8; 32],
        first_nullifier: Nullifier,
    }

    fn setup_params() -> TestParams {
        let mut rng = OsRng;

        let isk = IssueAuthKey::<ZSASchnorr>::random(&mut rng);
        let ik = IssueValidatingKey::from(&isk);

        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let recipient = fvk.address_at(0u32, Scope::External);

        let mut sighash = [0u8; 32];
        rng.fill_bytes(&mut sighash);

        let first_nullifier = Nullifier::dummy(&mut rng);

        TestParams {
            rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        }
    }

    /// Sets up test parameters for action verification tests.
    ///
    /// This function generates two notes with the specified values and asset descriptions,
    /// and returns the issuance validating key, the asset base, and the issue action.
    fn action_verify_test_params(
        note1_value: u64,
        note2_value: u64,
        note1_asset_desc: &[u8],
        note2_asset_desc: Option<&[u8]>, // if None, both notes use the same asset
        finalize: bool,
    ) -> (IssueValidatingKey<ZSASchnorr>, AssetBase, IssueAction) {
        let TestParams {
            mut rng,
            ik,
            recipient,
            ..
        } = setup_params();

        let note1_asset_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(note1_asset_desc).unwrap());
        let asset = AssetBase::derive(&ik, &note1_asset_desc_hash);
        let note2_asset = note2_asset_desc.map_or(asset, |desc| {
            AssetBase::derive(
                &ik,
                &compute_asset_desc_hash(&NonEmpty::from_slice(desc).unwrap()),
            )
        });

        let note1 = Note::new(
            recipient,
            NoteValue::from_raw(note1_value),
            asset,
            Rho::zero(),
            &mut rng,
        );

        let note2 = Note::new(
            recipient,
            NoteValue::from_raw(note2_value),
            note2_asset,
            Rho::zero(),
            &mut rng,
        );

        (
            ik,
            asset,
            IssueAction::from_parts(note1_asset_desc_hash, vec![note1, note2], finalize),
        )
    }

    /// This function computes the identity point on the Pallas curve and returns an Asset Base with that value.
    fn identity_point() -> AssetBase {
        let identity_point = (Point::generator() * -Scalar::one()) + Point::generator();
        AssetBase::from_bytes(&identity_point.to_bytes()).unwrap()
    }

    #[test]
    fn action_verify_valid() {
        let (ik, test_asset, action) = action_verify_test_params(10, 20, b"Asset 1", None, false);

        let result = action.verify(&ik);

        assert!(result.is_ok());

        let (asset, amount) = result.unwrap();

        assert_eq!(asset, test_asset);
        assert_eq!(amount, NoteValue::from_raw(30));
        assert!(!action.is_finalized());
    }

    #[test]
    fn action_verify_finalized() {
        let (ik, test_asset, action) = action_verify_test_params(10, 20, b"Asset 1", None, true);

        let result = action.verify(&ik);

        assert!(result.is_ok());

        let (asset, amount) = result.unwrap();

        assert_eq!(asset, test_asset);
        assert_eq!(amount, NoteValue::from_raw(30));
        assert!(action.is_finalized());
    }

    #[test]
    fn action_verify_incorrect_asset_base() {
        let (ik, _, action) =
            action_verify_test_params(10, 20, b"Asset 1", Some(b"Asset 2"), false);

        assert_eq!(action.verify(&ik), Err(IssueBundleIkMismatchAssetBase));
    }

    #[test]
    fn action_verify_ik_mismatch_asset_base() {
        let (_, _, action) = action_verify_test_params(10, 20, b"Asset 1", None, false);
        let TestParams { ik, .. } = setup_params();

        assert_eq!(action.verify(&ik), Err(IssueBundleIkMismatchAssetBase));
    }

    #[test]
    fn issue_bundle_basic() {
        let TestParams {
            rng,
            ik,
            recipient,
            first_nullifier,
            ..
        } = setup_params();

        let asset_desc_hash_1 = compute_asset_desc_hash(&NonEmpty::from_slice(b"Halo").unwrap());
        let asset_desc_hash_2 = compute_asset_desc_hash(&NonEmpty::from_slice(b"Halo2").unwrap());

        let (mut bundle, asset) = IssueBundle::new(
            ik.clone(),
            asset_desc_hash_1,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let another_asset = bundle
            .add_recipient(
                asset_desc_hash_1,
                recipient,
                NoteValue::from_raw(10),
                false,
                rng,
            )
            .unwrap();
        assert_eq!(asset, another_asset);

        let third_asset = bundle
            .add_recipient(
                asset_desc_hash_2,
                recipient,
                NoteValue::from_raw(15),
                true,
                rng,
            )
            .unwrap();
        assert_ne!(asset, third_asset);

        bundle.actions().iter().for_each(|action| {
            action
                .notes()
                .iter()
                .for_each(|note| assert_eq!(note.rho(), Rho::zero()))
        });
        let awaiting_sighash_bundle = bundle.update_rho(&first_nullifier);
        awaiting_sighash_bundle.actions().iter().for_each(|action| {
            action
                .notes()
                .iter()
                .for_each(|note| assert_ne!(note.rho(), Rho::zero()))
        });

        let actions = awaiting_sighash_bundle.actions();
        assert_eq!(actions.len(), 2);

        let action = awaiting_sighash_bundle.get_action_by_asset(&asset).unwrap();
        assert_eq!(action.notes.len(), 3);
        let reference_note = action.notes.first().unwrap();
        verify_reference_note(reference_note, asset);
        let first_note = action.notes.get(1).unwrap();
        assert_eq!(first_note.value().inner(), 5);
        assert_eq!(first_note.asset(), asset);
        assert_eq!(first_note.recipient(), recipient);

        let second_note = action.notes.get(2).unwrap();
        assert_eq!(second_note.value().inner(), 10);
        assert_eq!(second_note.asset(), asset);
        assert_eq!(second_note.recipient(), recipient);

        let action2 = awaiting_sighash_bundle
            .get_action_by_desc_hash(&asset_desc_hash_2)
            .unwrap();
        assert_eq!(action2.notes.len(), 2);
        let reference_note = action2.notes.first().unwrap();
        verify_reference_note(reference_note, AssetBase::derive(&ik, &asset_desc_hash_2));
        let first_note = action2.notes().get(1).unwrap();
        assert_eq!(first_note.value().inner(), 15);
        assert_eq!(first_note.asset(), third_asset);

        verify_reference_note(action.get_reference_note().unwrap(), asset);
        verify_reference_note(action2.get_reference_note().unwrap(), third_asset);
    }

    #[test]
    fn issue_bundle_finalize_asset() {
        let TestParams {
            rng, ik, recipient, ..
        } = setup_params();

        let nft_asset_desc_hash = compute_asset_desc_hash(&NonEmpty::from_slice(b"NFT").unwrap());
        let another_nft_asset_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"Another NFT").unwrap());

        let (mut bundle, _) = IssueBundle::new(
            ik,
            nft_asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(u64::MIN),
            }),
            true,
            rng,
        );

        bundle
            .finalize_action(&nft_asset_desc_hash)
            .expect("Should finalize properly");

        assert_eq!(
            bundle
                .finalize_action(&another_nft_asset_desc_hash)
                .unwrap_err(),
            IssueActionNotFound
        );
    }

    #[test]
    fn issue_bundle_prepare() {
        let TestParams {
            rng,
            ik,
            recipient,
            sighash,
            first_nullifier,
            ..
        } = setup_params();

        let asset_desc_hash = compute_asset_desc_hash(&NonEmpty::from_slice(b"Frost").unwrap());

        let (bundle, _) = IssueBundle::new(
            ik,
            asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let prepared = bundle.update_rho(&first_nullifier).prepare(sighash);
        assert_eq!(prepared.authorization().sighash, sighash);
    }

    #[test]
    fn issue_bundle_sign() {
        let TestParams {
            rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let asset_desc_hash = compute_asset_desc_hash(&NonEmpty::from_slice(b"Sign").unwrap());

        let (bundle, _) = IssueBundle::new(
            ik.clone(),
            asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        ik.verify(&sighash, &signed.authorization.signature)
            .expect("signature should be valid");
    }

    #[test]
    fn issue_bundle_invalid_isk_for_signature() {
        let TestParams {
            mut rng,
            ik,
            recipient,
            first_nullifier,
            ..
        } = setup_params();

        let asset_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"IssueBundle").unwrap());

        let (bundle, _) = IssueBundle::new(
            ik,
            asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let wrong_isk = IssueAuthKey::<ZSASchnorr>::random(&mut rng);

        let err = bundle
            .update_rho(&first_nullifier)
            .prepare([0; 32])
            .sign(&wrong_isk)
            .expect_err("should not be able to sign");

        assert_eq!(err, IssueBundleIkMismatchAssetBase);
    }

    #[test]
    fn issue_bundle_incorrect_asset_for_signature() {
        let TestParams {
            mut rng,
            isk,
            ik,
            recipient,
            first_nullifier,
            ..
        } = setup_params();

        // Create a bundle with "normal" note
        let (mut bundle, _) = IssueBundle::new(
            ik,
            compute_asset_desc_hash(&NonEmpty::from_slice(b"IssueBundle").unwrap()),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::derive(
                bundle.ik(),
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"zsa_asset").unwrap()),
            ),
            Rho::zero(),
            &mut rng,
        );
        bundle.actions.first_mut().notes.push(note);

        let err = bundle
            .update_rho(&first_nullifier)
            .prepare([0; 32])
            .sign(&isk)
            .expect_err("should not be able to sign");

        assert_eq!(err, IssueBundleIkMismatchAssetBase);
    }

    #[test]
    fn issue_bundle_verify() {
        let TestParams {
            rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let asset_desc_hash = compute_asset_desc_hash(&NonEmpty::from_slice(b"Verify").unwrap());

        let (bundle, _) = IssueBundle::new(
            ik.clone(),
            asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        let issued_assets =
            verify_issue_bundle(&signed, sighash, |_| None, &first_nullifier).unwrap();

        let first_note = *signed.actions().first().notes().first().unwrap();
        assert_eq!(
            issued_assets,
            BTreeMap::from([(
                AssetBase::derive(&ik, &asset_desc_hash),
                AssetRecord::new(NoteValue::from_raw(5), false, first_note)
            )])
        );
    }

    #[test]
    fn issue_bundle_verify_with_finalize() {
        let TestParams {
            rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let asset_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"Verify with finalize").unwrap());

        let (mut bundle, _) = IssueBundle::new(
            ik.clone(),
            asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(7),
            }),
            true,
            rng,
        );

        bundle.finalize_action(&asset_desc_hash).unwrap();

        let signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        let issued_assets =
            verify_issue_bundle(&signed, sighash, |_| None, &first_nullifier).unwrap();

        let first_note = *signed.actions().first().notes().first().unwrap();
        assert_eq!(
            issued_assets,
            BTreeMap::from([(
                AssetBase::derive(&ik, &asset_desc_hash),
                AssetRecord::new(NoteValue::from_raw(7), true, first_note)
            )])
        );
    }

    #[test]
    fn issue_bundle_verify_with_issued_assets() {
        let TestParams {
            rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let asset1_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"Verify with issued assets 1").unwrap());
        let asset2_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"Verify with issued assets 2").unwrap());
        let asset3_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"Verify with issued assets 3").unwrap());

        let asset1_base = AssetBase::derive(&ik, &asset1_desc_hash);
        let asset2_base = AssetBase::derive(&ik, &asset2_desc_hash);
        let asset3_base = AssetBase::derive(&ik, &asset3_desc_hash);

        let (mut bundle, _) = IssueBundle::new(
            ik,
            asset1_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(7),
            }),
            true,
            rng,
        );

        bundle
            .add_recipient(
                asset1_desc_hash,
                recipient,
                NoteValue::from_raw(8),
                false,
                rng,
            )
            .unwrap();

        bundle.finalize_action(&asset1_desc_hash).unwrap();

        bundle
            .add_recipient(
                asset2_desc_hash,
                recipient,
                NoteValue::from_raw(10),
                true,
                rng,
            )
            .unwrap();

        bundle.finalize_action(&asset2_desc_hash).unwrap();

        bundle
            .add_recipient(
                asset3_desc_hash,
                recipient,
                NoteValue::from_raw(5),
                true,
                rng,
            )
            .unwrap();

        let signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        let issued_assets =
            verify_issue_bundle(&signed, sighash, |_| None, &first_nullifier).unwrap();

        assert_eq!(issued_assets.keys().len(), 3);

        let reference_note1 = signed.actions()[0].notes()[0];
        let reference_note2 = signed.actions()[1].notes()[0];
        let reference_note3 = signed.actions()[2].notes()[0];

        assert_eq!(
            issued_assets.get(&asset1_base),
            Some(&AssetRecord::new(
                NoteValue::from_raw(15),
                true,
                reference_note1
            ))
        );
        assert_eq!(
            issued_assets.get(&asset2_base),
            Some(&AssetRecord::new(
                NoteValue::from_raw(10),
                true,
                reference_note2
            ))
        );
        assert_eq!(
            issued_assets.get(&asset3_base),
            Some(&AssetRecord::new(
                NoteValue::from_raw(5),
                false,
                reference_note3
            ))
        );
    }

    #[test]
    fn issue_bundle_verify_fail_incorrect_rho_derivation() {
        let TestParams {
            mut rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let asset_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"asset desc").unwrap());

        let (bundle, _) = IssueBundle::new(
            ik.clone(),
            asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        // Verify that `verify_issue_bundle` returns an error if `first_nullifier` is incorrect.
        assert_eq!(
            verify_issue_bundle(&signed, sighash, |_| None, &Nullifier::dummy(&mut rng)),
            Err(IncorrectRhoDerivation)
        );
    }

    #[test]
    fn issue_bundle_verify_fail_previously_finalized() {
        let TestParams {
            mut rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let asset_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"already final").unwrap());

        let (bundle, _) = IssueBundle::new(
            ik.clone(),
            asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        let final_type = AssetBase::derive(&ik, &asset_desc_hash);

        let issued_assets = [(
            final_type,
            AssetRecord::new(
                NoteValue::from_raw(20),
                true,
                Note::new(
                    recipient,
                    NoteValue::from_raw(10),
                    final_type,
                    Rho::zero(),
                    &mut rng,
                ),
            ),
        )]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

        assert_eq!(
            verify_issue_bundle(
                &signed,
                sighash,
                |asset| issued_assets.get(asset).copied(),
                &first_nullifier
            )
            .unwrap_err(),
            IssueActionPreviouslyFinalizedAssetBase
        );
    }

    #[test]
    fn issue_bundle_verify_fail_bad_signature() {
        // we want to inject "bad" signatures for test purposes.
        impl IssueBundle<Signed> {
            pub fn set_authorization(&mut self, authorization: Signed) {
                self.authorization = authorization;
            }
        }

        let TestParams {
            mut rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            crate::issuance::compute_asset_desc_hash(&NonEmpty::from_slice(b"bad sig").unwrap()),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let wrong_isk = IssueAuthKey::<ZSASchnorr>::random(&mut rng);

        let mut signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        signed.set_authorization(Signed {
            signature: wrong_isk.try_sign(&sighash).unwrap(),
        });

        assert_eq!(
            verify_issue_bundle(&signed, sighash, |_| None, &first_nullifier).unwrap_err(),
            InvalidIssueBundleSig
        );
    }

    #[test]
    fn issue_bundle_verify_fail_wrong_sighash() {
        let TestParams {
            rng,
            isk,
            ik,
            recipient,
            sighash: random_sighash,
            first_nullifier,
        } = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset description").unwrap()),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let sighash: [u8; 32] = bundle.commitment().into();
        let signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        assert_eq!(
            verify_issue_bundle(&signed, random_sighash, |_| None, &first_nullifier).unwrap_err(),
            InvalidIssueBundleSig
        );
    }

    #[test]
    fn issue_bundle_verify_fail_incorrect_asset_description() {
        let TestParams {
            mut rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset description").unwrap()),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let mut signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::derive(
                signed.ik(),
                &compute_asset_desc_hash(&NonEmpty::from_slice(b"zsa_asset").unwrap()),
            ),
            rho_for_issuance_note(&first_nullifier, 0, 2),
            &mut rng,
        );

        signed.actions.first_mut().notes.push(note);

        assert_eq!(
            verify_issue_bundle(&signed, sighash, |_| None, &first_nullifier).unwrap_err(),
            IssueBundleIkMismatchAssetBase
        );
    }

    #[test]
    fn issue_bundle_verify_fail_incorrect_ik() {
        let asset_desc_hash = compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset").unwrap());

        let TestParams {
            mut rng,
            isk,
            ik,
            recipient,
            sighash,
            first_nullifier,
        } = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            asset_desc_hash,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let mut signed = bundle
            .update_rho(&first_nullifier)
            .prepare(sighash)
            .sign(&isk)
            .unwrap();

        let incorrect_isk = IssueAuthKey::<ZSASchnorr>::random(&mut rng);
        let incorrect_ik = IssueValidatingKey::from(&incorrect_isk);

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(55),
            AssetBase::derive(&incorrect_ik, &asset_desc_hash),
            rho_for_issuance_note(&first_nullifier, 0, 0),
            &mut rng,
        );

        signed.actions.first_mut().notes = vec![note];

        assert_eq!(
            verify_issue_bundle(&signed, sighash, |_| None, &first_nullifier).unwrap_err(),
            IssueBundleIkMismatchAssetBase
        );
    }

    #[test]
    fn finalize_flag_serialization() {
        let mut rng = OsRng;
        let (_, _, note) = Note::dummy(&mut rng, None, AssetBase::native());

        let asset_desc_hash =
            compute_asset_desc_hash(&NonEmpty::from_slice(b"Asset description").unwrap());

        let action = IssueAction::new_with_flags(asset_desc_hash, vec![note], 0u8).unwrap();
        assert_eq!(action.flags(), 0b0000_0000);

        let action = IssueAction::new_with_flags(asset_desc_hash, vec![note], 1u8).unwrap();
        assert_eq!(action.flags(), 0b0000_0001);

        let action = IssueAction::new_with_flags(asset_desc_hash, vec![note], 2u8);
        assert!(action.is_none());
    }

    #[test]
    fn test_get_action_by_desc_hash() {
        let TestParams {
            rng, ik, recipient, ..
        } = setup_params();

        // UTF heavy test string
        let asset_desc_1 = "ΩΣ𐐷कあ한🐍★→".to_string().as_bytes().to_vec();

        let asset_desc_hash_1 =
            compute_asset_desc_hash(&NonEmpty::from_slice(&asset_desc_1).unwrap());

        let (bundle, asset_base_1) = IssueBundle::new(
            ik,
            asset_desc_hash_1,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        // Checks for the case of UTF-8 encoded asset description.
        let action = bundle.get_action_by_asset(&asset_base_1).unwrap();
        assert_eq!(action.asset_desc_hash(), &asset_desc_hash_1);
        let reference_note = action.notes.first().unwrap();
        verify_reference_note(reference_note, asset_base_1);
        assert_eq!(action.notes.get(1).unwrap().value().inner(), 5);
        assert_eq!(
            bundle.get_action_by_desc_hash(&asset_desc_hash_1).unwrap(),
            action
        );
    }

    #[test]
    #[should_panic(expected = "asset_desc is not a well-formed Unicode string")]
    fn not_well_formed_utf8() {
        // Not well-formed as per Unicode 15.0 specification, Section 3.9, D92
        let asset_desc: Vec<u8> = vec![0xc0, 0xaf];

        // Confirm not valid UTF-8
        assert!(String::from_utf8(asset_desc.clone()).is_err());

        // Should panic
        compute_asset_desc_hash(&NonEmpty::from_slice(&asset_desc).unwrap());
    }

    #[test]
    fn verify_rho_computation_for_issuance_notes() {
        // Setup keys
        let pk = ProvingKey::build::<OrchardZSA>();
        let sk = SpendingKey::from_bytes([1; 32]).unwrap();
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);
        let isk = IssueAuthKey::<ZSASchnorr>::from_bytes(&[2; 32]).unwrap();
        let ik = IssueValidatingKey::from(&isk);

        // Setup note and merkle tree
        let mut rng = OsRng;
        let asset1 = AssetBase::derive(
            &ik,
            &compute_asset_desc_hash(&NonEmpty::from_slice(b"zsa_asset1").unwrap()),
        );
        let note1 = Note::new(
            recipient,
            NoteValue::from_raw(10),
            asset1,
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );
        // Build the merkle tree with only note1
        let (merkle_path, anchor): (MerklePath, Anchor) = {
            let cmx: ExtractedNoteCommitment = note1.commitment().into();
            let leaf = MerkleHashOrchard::from_cmx(&cmx);
            let mut tree: ShardTree<MemoryShardStore<MerkleHashOrchard, u32>, 32, 16> =
                ShardTree::new(MemoryShardStore::empty(), 100);
            tree.append(
                leaf,
                Retention::Checkpoint {
                    id: 0,
                    marking: Marking::Marked,
                },
            )
            .unwrap();
            let root = tree.root_at_checkpoint_id(&0).unwrap().unwrap();
            let position = tree.max_leaf_position(None).unwrap().unwrap();
            let merkle_path = tree
                .witness_at_checkpoint_id(position, &0)
                .unwrap()
                .unwrap();
            assert_eq!(root, merkle_path.root(MerkleHashOrchard::from_cmx(&cmx)));

            (merkle_path.into(), root.into())
        };

        // Create a transfer bundle
        let mut builder = Builder::new(BundleType::DEFAULT_ZSA, anchor);
        builder.add_spend(fvk, note1, merkle_path).unwrap();
        builder
            .add_output(None, recipient, NoteValue::from_raw(5), asset1, [0u8; 512])
            .unwrap();
        builder
            .add_output(None, recipient, NoteValue::from_raw(5), asset1, [0u8; 512])
            .unwrap();
        let unauthorized = builder.build(&mut rng).unwrap().0;
        let sighash = unauthorized.commitment().into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        let authorized: Bundle<_, i64, OrchardZSA> = proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap();

        // Create an issue bundle
        let asset_desc_hash_2 = compute_asset_desc_hash(&NonEmpty::from_slice(b"asset2").unwrap());
        let asset_desc_hash_3 = compute_asset_desc_hash(&NonEmpty::from_slice(b"asset3").unwrap());
        let (mut bundle, asset) = IssueBundle::new(
            ik,
            asset_desc_hash_2,
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        );

        let another_asset = bundle
            .add_recipient(
                asset_desc_hash_2,
                recipient,
                NoteValue::from_raw(10),
                false,
                rng,
            )
            .unwrap();
        assert_eq!(asset, another_asset);

        let third_asset = bundle
            .add_recipient(
                asset_desc_hash_3,
                recipient,
                NoteValue::from_raw(10),
                true,
                rng,
            )
            .unwrap();
        assert_ne!(asset, third_asset);

        // Check that all rho values are zero.
        bundle.actions().iter().for_each(|action| {
            action
                .notes()
                .iter()
                .for_each(|note| assert_eq!(note.rho(), Rho::zero()))
        });

        let awaiting_sighash_bundle = bundle.update_rho(authorized.actions().first().nullifier());

        assert_eq!(awaiting_sighash_bundle.actions().len(), 2);
        assert_eq!(
            awaiting_sighash_bundle
                .actions()
                .get(0)
                .unwrap()
                .notes()
                .len(),
            3
        );
        assert_eq!(
            awaiting_sighash_bundle
                .actions()
                .get(1)
                .unwrap()
                .notes()
                .len(),
            2
        );

        // Check the rho value for each issuance note in the issue bundle
        for (index_action, action) in awaiting_sighash_bundle.actions.iter().enumerate() {
            for (index_note, note) in action.notes.iter().enumerate() {
                let expected_rho = rho_for_issuance_note(
                    authorized.actions().first().nullifier(),
                    index_action.try_into().unwrap(),
                    index_note.try_into().unwrap(),
                );
                assert_eq!(note.rho(), expected_rho);
            }
        }
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use crate::{
        issuance::{AwaitingNullifier, IssueAction, IssueBundle, Prepared, Signed},
        issuance_auth::{
            testing::arb_issuance_validating_key, IssueAuthSig, IssueAuthSigScheme, ZSASchnorr,
        },
        note::asset_base::testing::zsa_asset_base,
        note::testing::arb_zsa_note,
    };
    use nonempty::NonEmpty;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::prop_compose;

    prop_compose! {
        /// Generate a uniformly distributed ZSA Schnorr signature
        pub(crate) fn arb_signature()(
            sig_bytes in vec(prop::num::u8::ANY, 64)
        ) -> IssueAuthSig<ZSASchnorr> {
            let mut encoded = vec![ZSASchnorr::ALGORITHM_BYTE];
            encoded.extend(sig_bytes);
            IssueAuthSig::decode(&encoded).unwrap()
        }
    }

    prop_compose! {
        /// Generate an issue action
        pub fn arb_issue_action(asset_desc_hash: [u8; 32])
        (
            asset in zsa_asset_base(asset_desc_hash),
        )
        (
            note in arb_zsa_note(asset),
        )-> IssueAction {
            IssueAction{
                asset_desc_hash,
                notes: vec![note],
                finalize: false,
            }
        }
    }

    prop_compose! {
        /// Generate an arbitrary issue bundle with fake authorization data.
        pub fn arb_awaiting_nullifier_issue_bundle(n_actions: usize)
        (
            actions in vec(arb_issue_action([1u8; 32]), n_actions),
            ik in arb_issuance_validating_key()
        ) -> IssueBundle<AwaitingNullifier> {
            let actions = NonEmpty::from_vec(actions).unwrap();
            IssueBundle {
                ik,
                actions,
                authorization: AwaitingNullifier
            }
        }
    }

    prop_compose! {
        /// Generate an arbitrary issue bundle with fake authorization data. This bundle does not
        /// necessarily respect consensus rules
        pub fn arb_prepared_issue_bundle(n_actions: usize)
        (
            actions in vec(arb_issue_action([1u8; 32]), n_actions),
            ik in arb_issuance_validating_key(),
            fake_sighash in prop::array::uniform32(prop::num::u8::ANY)
        ) -> IssueBundle<Prepared> {
            let actions = NonEmpty::from_vec(actions).unwrap();
            IssueBundle {
                ik,
                actions,
                authorization: Prepared { sighash: fake_sighash }
            }
        }
    }

    prop_compose! {
        /// Generate an arbitrary issue bundle with fake authorization data. This bundle does not
        /// necessarily respect consensus rules
        pub fn arb_signed_issue_bundle(n_actions: usize)
        (
            actions in vec(arb_issue_action([1u8; 32]), n_actions),
            ik in arb_issuance_validating_key(),
            fake_sig in arb_signature(),
        ) -> IssueBundle<Signed> {
            let actions = NonEmpty::from_vec(actions).unwrap();
            IssueBundle {
                ik,
                actions,
                authorization: Signed { signature: fake_sig },
            }
        }
    }
}
