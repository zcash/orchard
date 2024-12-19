//! Structs related to issuance bundles and the associated logic.
use blake2b_simd::Hash as Blake2bHash;
use group::Group;
use k256::schnorr;
use nonempty::NonEmpty;
use rand::RngCore;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::bundle::commitments::{hash_issue_bundle_auth_data, hash_issue_bundle_txid_data};
use crate::constants::reference_keys::ReferenceKeys;
use crate::issuance::Error::{
    AssetBaseCannotBeIdentityPoint, IssueActionNotFound, IssueActionPreviouslyFinalizedAssetBase,
    IssueActionWithoutNoteNotFinalized, IssueBundleIkMismatchAssetBase,
    IssueBundleInvalidSignature, ValueSumOverflow, WrongAssetDescSize,
};
use crate::keys::{IssuanceAuthorizingKey, IssuanceValidatingKey};
use crate::note::asset_base::is_asset_desc_of_valid_size;
use crate::note::{AssetBase, Nullifier, Rho};

use crate::value::NoteValue;
use crate::{Address, Note};

use crate::supply_info::{AssetSupply, SupplyInfo};

/// A bundle of actions to be applied to the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueBundle<T: IssueAuth> {
    /// The issuer key for the note being created.
    ik: IssuanceValidatingKey,
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
    /// Asset description for verification.
    asset_desc: Vec<u8>,
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

impl IssueAction {
    /// Constructs a new `IssueAction`.
    pub fn new_with_flags(asset_desc: Vec<u8>, notes: Vec<Note>, flags: u8) -> Option<Self> {
        let finalize = match flags {
            0b0000_0000 => false,
            0b0000_0001 => true,
            _ => return None,
        };
        Some(IssueAction {
            asset_desc,
            notes,
            finalize,
        })
    }

    /// Constructs an `IssueAction` from its constituent parts.
    pub fn from_parts(asset_desc: Vec<u8>, notes: Vec<Note>, finalize: bool) -> Self {
        IssueAction {
            asset_desc,
            notes,
            finalize,
        }
    }

    /// Returns the asset description for the note being created.
    pub fn asset_desc(&self) -> &[u8] {
        &self.asset_desc
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
    /// its supply as a tuple (`AssetBase`, `AssetSupply`) or an error if the asset was not
    ///  properly derived or an overflow occurred during the supply amount calculation.
    ///
    /// # Arguments
    ///
    /// * `ik` - A reference to the `IssuanceValidatingKey` used for deriving the asset.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple with an `AssetBase` and an `AssetSupply`, or an `Error`.
    ///
    /// # Errors
    ///
    /// This function may return an error in any of the following cases:
    ///
    /// * `ValueSumOverflow`: If the total amount value of all notes in the `IssueAction` overflows.
    ///
    /// * `IssueBundleIkMismatchAssetBase`: If the provided `ik` is not used to derive the
    ///   `AssetBase` for **all** internal notes.
    ///
    /// * `IssueActionWithoutNoteNotFinalized`:If the `IssueAction` contains no note and is not finalized.
    fn verify_supply(&self, ik: &IssuanceValidatingKey) -> Result<(AssetBase, AssetSupply), Error> {
        if self.notes.is_empty() && !self.is_finalized() {
            return Err(IssueActionWithoutNoteNotFinalized);
        }

        let issue_asset = AssetBase::derive(ik, &self.asset_desc);

        // Calculate the value of the asset as a sum of values of all its notes
        // and ensure all note types are equal the asset derived from asset_desc and ik.
        let value_sum = self
            .notes
            .iter()
            .try_fold(NoteValue::zero(), |value_sum, &note| {
                //The asset base should not be the identity point of the Pallas curve.
                if bool::from(note.asset().cv_base().is_identity()) {
                    return Err(AssetBaseCannotBeIdentityPoint);
                }

                // All assets should be derived correctly
                note.asset()
                    .eq(&issue_asset)
                    .then_some(())
                    .ok_or(IssueBundleIkMismatchAssetBase)?;

                // The total amount should not overflow
                (value_sum + note.value()).ok_or(ValueSumOverflow)
            })?;

        Ok((
            issue_asset,
            AssetSupply::new(value_sum, self.is_finalized()),
        ))
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
}

/// Defines the authorization type of an Issue bundle.
pub trait IssueAuth: fmt::Debug + Clone {}

/// Marker for an unauthorized bundle with no proofs or signatures.
#[derive(Debug, Clone)]
pub struct Unauthorized;

/// Marker for an unauthorized bundle with injected sighash.
#[derive(Debug, Clone)]
pub struct Prepared {
    sighash: [u8; 32],
}

/// Marker for an authorized bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signed {
    signature: schnorr::Signature,
}

impl Signed {
    /// Returns the signature for this authorization.
    pub fn signature(&self) -> &schnorr::Signature {
        &self.signature
    }

    /// Constructs a `Signed` from a byte array containing Schnorr signature bytes.
    pub fn from_data(data: [u8; 64]) -> Self {
        Signed {
            signature: schnorr::Signature::try_from(data.as_ref()).unwrap(),
        }
    }
}

impl IssueAuth for Unauthorized {}
impl IssueAuth for Prepared {}
impl IssueAuth for Signed {}

impl<T: IssueAuth> IssueBundle<T> {
    /// Returns the issuer verification key for the bundle.
    pub fn ik(&self) -> &IssuanceValidatingKey {
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

    /// Returns the authorization for this action.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Find the action corresponding to the `asset_desc` for a given `IssueBundle`.
    ///
    /// # Returns
    ///
    /// If a single matching action is found, it is returned as `Some(&IssueAction)`.
    /// If no action matches the given `asset_desc`, it returns `None`.
    ///
    /// # Panics
    ///
    /// Panics if multiple matching actions are found.
    pub fn get_action_by_desc(&self, asset_desc: &[u8]) -> Option<&IssueAction> {
        let issue_actions: Vec<&IssueAction> = self
            .actions
            .iter()
            .filter(|a| a.asset_desc.eq(asset_desc))
            .collect();
        match issue_actions.len() {
            0 => None,
            1 => Some(issue_actions[0]),
            _ => panic!("Multiple IssueActions with the same asset_desc"),
        }
    }

    /// Find the actions corresponding to an Asset Base `asset` for a given `IssueBundle`.
    ///
    /// # Returns
    ///
    /// If a single matching action is found, it is returned as `Some(&IssueAction)`.
    /// If no action matches the given Asset Base `asset`, it returns `None`.
    ///
    /// # Panics
    ///
    /// Panics if multiple matching actions are found.
    pub fn get_action_by_asset(&self, asset: &AssetBase) -> Option<&IssueAction> {
        let issue_actions: Vec<&IssueAction> = self
            .actions
            .iter()
            .filter(|a| AssetBase::derive(&self.ik, &a.asset_desc).eq(asset))
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
        ik: IssuanceValidatingKey,
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

impl IssueBundle<Unauthorized> {
    /// Constructs a new `IssueBundle`.
    ///
    /// If issue_info is None, the new `IssueBundle` will contain one `IssueAction` without notes
    /// and with `finalize` set to true.
    /// Otherwise, the new `IssueBundle` will contain one `IssueAction with one note created from
    /// issue_info values and with `finalize` set to false. In this created note, rho will be
    /// randomly sampled, similar to dummy note generation.
    ///
    /// If `first_issuance` is true, the `IssueBundle` will contain a reference note for the asset
    /// defined by (`asset_desc`, `ik`).
    ///
    /// # Errors
    ///
    /// This function may return an error in any of the following cases:
    ///
    /// * `WrongAssetDescSize`: If `asset_desc` is empty or longer than 512 bytes.
    pub fn new(
        ik: IssuanceValidatingKey,
        asset_desc: Vec<u8>,
        issue_info: Option<IssueInfo>,
        first_issuance: bool,
        mut rng: impl RngCore,
    ) -> Result<(IssueBundle<Unauthorized>, AssetBase), Error> {
        if !is_asset_desc_of_valid_size(&asset_desc) {
            return Err(WrongAssetDescSize);
        }

        let asset = AssetBase::derive(&ik, &asset_desc);

        let mut notes = vec![];
        if first_issuance {
            notes.push(create_reference_note(asset, &mut rng));
        };

        let action = match issue_info {
            None => IssueAction {
                asset_desc,
                notes,
                finalize: true,
            },
            Some(issue_info) => {
                let note = Note::new(
                    issue_info.recipient,
                    issue_info.value,
                    asset,
                    Rho::from_nf_old(Nullifier::dummy(&mut rng)),
                    &mut rng,
                );

                notes.push(note);

                IssueAction {
                    asset_desc,
                    notes,
                    finalize: false,
                }
            }
        };

        Ok((
            IssueBundle {
                ik,
                actions: NonEmpty::new(action),
                authorization: Unauthorized,
            },
            asset,
        ))
    }

    /// Add a new note to the `IssueBundle`.
    ///
    /// Rho will be randomly sampled, similar to dummy note generation.
    /// If `first_issuance` is true, we will also add a reference note for the asset defined by
    /// (`asset_desc`, `ik`).
    ///
    /// # Errors
    ///
    /// This function may return an error in any of the following cases:
    ///
    /// * `WrongAssetDescSize`: If `asset_desc` is empty or longer than 512 bytes.
    pub fn add_recipient(
        &mut self,
        asset_desc: &[u8],
        recipient: Address,
        value: NoteValue,
        first_issuance: bool,
        mut rng: impl RngCore,
    ) -> Result<AssetBase, Error> {
        if !is_asset_desc_of_valid_size(asset_desc) {
            return Err(WrongAssetDescSize);
        }

        let asset = AssetBase::derive(&self.ik, asset_desc);

        let note = Note::new(
            recipient,
            value,
            asset,
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );

        let notes = if first_issuance {
            vec![create_reference_note(asset, &mut rng), note]
        } else {
            vec![note]
        };

        let action = self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.asset_desc.eq(asset_desc));

        match action {
            Some(action) => {
                // Append to an existing IssueAction.
                action.notes.extend(notes);
            }
            None => {
                // Insert a new IssueAction.
                self.actions.push(IssueAction {
                    asset_desc: Vec::from(asset_desc),
                    notes,
                    finalize: false,
                });
            }
        };

        Ok(asset)
    }

    /// Finalizes a given `IssueAction`
    ///
    /// # Panics
    ///
    /// Panics if `asset_desc` is empty or longer than 512 bytes.
    pub fn finalize_action(&mut self, asset_desc: &[u8]) -> Result<(), Error> {
        if !is_asset_desc_of_valid_size(asset_desc) {
            return Err(WrongAssetDescSize);
        }

        match self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.asset_desc.eq(asset_desc))
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

    /// Loads the sighash into the bundle, as preparation for signing.
    pub fn prepare(self, sighash: [u8; 32]) -> IssueBundle<Prepared> {
        IssueBundle {
            ik: self.ik,
            actions: self.actions,
            authorization: Prepared { sighash },
        }
    }
}

impl<T: IssueAuth> IssueBundle<T> {
    /// Returns the reference notes for the `IssueBundle`.
    pub fn get_reference_notes(self) -> HashMap<AssetBase, Note> {
        let mut reference_notes = HashMap::new();
        self.actions.iter().for_each(|action| {
            action.notes.iter().for_each(|note| {
                if (note.recipient() == ReferenceKeys::recipient())
                    && (note.value() == NoteValue::zero())
                {
                    reference_notes.insert(note.asset(), *note);
                }
            })
        });
        reference_notes
    }
}

fn create_reference_note(asset: AssetBase, mut rng: impl RngCore) -> Note {
    Note::new(
        ReferenceKeys::recipient(),
        NoteValue::zero(),
        asset,
        Rho::from_nf_old(Nullifier::dummy(&mut rng)),
        &mut rng,
    )
}

impl IssueBundle<Prepared> {
    /// Sign the `IssueBundle`.
    /// The call makes sure that the provided `isk` matches the `ik` and the derived `asset` for each note in the bundle.
    pub fn sign(self, isk: &IssuanceAuthorizingKey) -> Result<IssueBundle<Signed>, Error> {
        let expected_ik: IssuanceValidatingKey = isk.into();

        // Make sure the `expected_ik` matches the `asset` for all notes.
        self.actions.iter().try_for_each(|action| {
            action.verify_supply(&expected_ik)?;
            Ok(())
        })?;

        // Make sure the signature can be generated.
        let signature = isk
            .try_sign(&self.authorization.sighash)
            .map_err(|_| IssueBundleInvalidSignature)?;

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
    /// Computes a commitment to the authorizing data within for this bundle.
    ///
    /// This together with `IssueBundle::commitment` bind the entire bundle.
    pub fn authorizing_commitment(&self) -> IssueBundleAuthorizingCommitment {
        IssueBundleAuthorizingCommitment(hash_issue_bundle_auth_data(self))
    }
}

/// Validation for Orchard IssueBundles
///
/// A set of previously finalized asset types must be provided in `finalized` argument.
///
/// The following checks are performed:
/// * For the `IssueBundle`:
///     * the Signature on top of the provided `sighash` verifies correctly.
/// * For each `IssueAction`:
///     * Asset description size is collect.
///     * `AssetBase` for the `IssueAction` has not been previously finalized.
/// * For each `Note` inside an `IssueAction`:
///     * All notes have the same, correct `AssetBase`.
///
// # Returns
///
/// A Result containing a SupplyInfo struct, which stores supply information in a HashMap.
/// The HashMap uses AssetBase as the key, and an AssetSupply struct as the value. The
/// AssetSupply contains a ValueSum (representing the total value of all notes for the asset)
/// and a bool indicating whether the asset is finalized.
///
/// # Errors
///
/// * `IssueBundleInvalidSignature`: This error occurs if the signature verification
///    for the provided `sighash` fails.
/// * `WrongAssetDescSize`: This error is raised if the asset description size for any
///    asset in the bundle is incorrect.
/// * `IssueActionPreviouslyFinalizedAssetBase`:  This error occurs if the asset has already been
///    finalized (inserted into the `finalized` collection).
/// * `ValueSumOverflow`: This error occurs if an overflow happens during the calculation of
///     the value sum for the notes in the asset.
/// * `IssueBundleIkMismatchAssetBase`: This error is raised if the `AssetBase` derived from
///    the `ik` (Issuance Validating Key) and the `asset_desc` (Asset Description) does not match
///    the expected `AssetBase`.
pub fn verify_issue_bundle(
    bundle: &IssueBundle<Signed>,
    sighash: [u8; 32],
    finalized: &HashSet<AssetBase>, // The finalization set.
) -> Result<SupplyInfo, Error> {
    bundle
        .ik
        .verify(&sighash, &bundle.authorization.signature)
        .map_err(|_| IssueBundleInvalidSignature)?;

    let supply_info =
        bundle
            .actions()
            .iter()
            .try_fold(SupplyInfo::new(), |mut supply_info, action| {
                if !is_asset_desc_of_valid_size(action.asset_desc()) {
                    return Err(WrongAssetDescSize);
                }

                let (asset, supply) = action.verify_supply(bundle.ik())?;

                // Fail if the asset was previously finalized.
                if finalized.contains(&asset) {
                    return Err(IssueActionPreviouslyFinalizedAssetBase(asset));
                }

                supply_info.add_supply(asset, supply)?;

                Ok(supply_info)
            })?;

    Ok(supply_info)
}

/// Errors produced during the issuance process
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The requested IssueAction not exists in the bundle.
    IssueActionNotFound,
    /// The provided `isk` and the derived `ik` does not match at least one note type.
    IssueBundleIkMismatchAssetBase,
    /// `asset_desc` should be between 1 and 512 bytes.
    WrongAssetDescSize,
    /// The `IssueAction` is not finalized but contains no notes.
    IssueActionWithoutNoteNotFinalized,
    /// The `AssetBase` is the Pallas identity point, which is invalid.
    AssetBaseCannotBeIdentityPoint,

    /// Verification errors:
    /// Invalid signature.
    IssueBundleInvalidSignature,
    /// The provided `AssetBase` has been previously finalized.
    IssueActionPreviouslyFinalizedAssetBase(AssetBase),

    /// Overflow error occurred while calculating the value of the asset
    ValueSumOverflow,
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
            WrongAssetDescSize => {
                write!(f, "`asset_desc` should be between 1 and 512 bytes")
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
            IssueBundleInvalidSignature => {
                write!(f, "invalid signature")
            }
            IssueActionPreviouslyFinalizedAssetBase(_) => {
                write!(f, "the provided `AssetBase` has been previously finalized")
            }
            ValueSumOverflow => {
                write!(
                    f,
                    "overflow error occurred while calculating the value of the asset"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AssetSupply, IssueBundle, IssueInfo};
    use crate::constants::reference_keys::ReferenceKeys;
    use crate::issuance::Error::{
        AssetBaseCannotBeIdentityPoint, IssueActionNotFound,
        IssueActionPreviouslyFinalizedAssetBase, IssueBundleIkMismatchAssetBase,
        IssueBundleInvalidSignature, WrongAssetDescSize,
    };
    use crate::issuance::{verify_issue_bundle, IssueAction, Signed, Unauthorized};
    use crate::keys::{
        FullViewingKey, IssuanceAuthorizingKey, IssuanceValidatingKey, Scope, SpendingKey,
    };
    use crate::note::{AssetBase, Nullifier, Rho};
    use crate::value::NoteValue;
    use crate::{Address, Note};
    use group::{Group, GroupEncoding};
    use nonempty::NonEmpty;
    use pasta_curves::pallas::{Point, Scalar};
    use rand::rngs::OsRng;
    use rand::RngCore;
    use std::collections::HashSet;

    /// Validation for reference note
    ///
    /// The following checks are performed:
    /// - the note value of the reference note is equal to 0
    /// - the asset of the reference note is equal to the provided asset
    /// - the recipient of the reference note is equal to the reference recipient
    fn verify_reference_note(note: &Note, asset: AssetBase) {
        assert_eq!(note.value(), NoteValue::from_raw(0));
        assert_eq!(note.asset(), asset);
        assert_eq!(note.recipient(), ReferenceKeys::recipient());
    }

    fn setup_params() -> (
        OsRng,
        IssuanceAuthorizingKey,
        IssuanceValidatingKey,
        Address,
        [u8; 32],
    ) {
        let mut rng = OsRng;

        let isk = IssuanceAuthorizingKey::random();
        let ik: IssuanceValidatingKey = (&isk).into();

        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let recipient = fvk.address_at(0u32, Scope::External);

        let mut sighash = [0u8; 32];
        rng.fill_bytes(&mut sighash);

        (rng, isk, ik, recipient, sighash)
    }

    /// Sets up test parameters for supply tests.
    ///
    /// This function generates two notes with the specified values and asset descriptions,
    /// and returns the issuance validating key, the asset base, and the issue action.
    fn supply_test_params(
        note1_value: u64,
        note2_value: u64,
        note1_asset_desc: &[u8],
        note2_asset_desc: Option<&[u8]>, // if None, both notes use the same asset
        finalize: bool,
    ) -> (IssuanceValidatingKey, AssetBase, IssueAction) {
        let (mut rng, _, ik, recipient, _) = setup_params();

        let asset = AssetBase::derive(&ik, note1_asset_desc);
        let note2_asset = note2_asset_desc.map_or(asset, |desc| AssetBase::derive(&ik, desc));

        let note1 = Note::new(
            recipient,
            NoteValue::from_raw(note1_value),
            asset,
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );

        let note2 = Note::new(
            recipient,
            NoteValue::from_raw(note2_value),
            note2_asset,
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );

        (
            ik,
            asset,
            IssueAction::from_parts(note1_asset_desc.to_vec(), vec![note1, note2], finalize),
        )
    }

    /// This function computes the identity point on the Pallas curve and returns an Asset Base with that value.
    fn identity_point() -> AssetBase {
        let identity_point = (Point::generator() * -Scalar::one()) + Point::generator();
        AssetBase::from_bytes(&identity_point.to_bytes()).unwrap()
    }

    /// Sets up test parameters for identity point tests.
    ///
    /// This function generates two notes with the identity point as their asset base,
    /// and returns the issuance authorizing key, an unauthorized issue bundle containing
    /// the notes, and a sighash
    fn identity_point_test_params(
        note1_value: u64,
        note2_value: u64,
    ) -> (IssuanceAuthorizingKey, IssueBundle<Unauthorized>, [u8; 32]) {
        let (mut rng, isk, ik, recipient, sighash) = setup_params();

        let note1 = Note::new(
            recipient,
            NoteValue::from_raw(note1_value),
            identity_point(),
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );

        let note2 = Note::new(
            recipient,
            NoteValue::from_raw(note2_value),
            identity_point(),
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );

        let action =
            IssueAction::from_parts("arbitrary asset_desc".into(), vec![note1, note2], false);

        let bundle = IssueBundle::from_parts(ik, NonEmpty::new(action), Unauthorized);

        (isk, bundle, sighash)
    }

    #[test]
    fn verify_supply_valid() {
        let (ik, test_asset, action) = supply_test_params(10, 20, b"Asset 1", None, false);

        let result = action.verify_supply(&ik);

        assert!(result.is_ok());

        let (asset, supply) = result.unwrap();

        assert_eq!(asset, test_asset);
        assert_eq!(supply.amount, NoteValue::from_raw(30));
        assert!(!supply.is_finalized);
    }

    #[test]
    fn verify_supply_invalid_for_asset_base_as_identity() {
        let (_, bundle, _) = identity_point_test_params(10, 20);

        assert_eq!(
            bundle.actions.head.verify_supply(&bundle.ik),
            Err(AssetBaseCannotBeIdentityPoint)
        );
    }

    #[test]
    fn verify_supply_finalized() {
        let (ik, test_asset, action) = supply_test_params(10, 20, b"Asset 1", None, true);

        let result = action.verify_supply(&ik);

        assert!(result.is_ok());

        let (asset, supply) = result.unwrap();

        assert_eq!(asset, test_asset);
        assert_eq!(supply.amount, NoteValue::from_raw(30));
        assert!(supply.is_finalized);
    }

    #[test]
    fn verify_supply_incorrect_asset_base() {
        let (ik, _, action) = supply_test_params(10, 20, b"Asset 1", Some(b"Asset 2"), false);

        assert_eq!(
            action.verify_supply(&ik),
            Err(IssueBundleIkMismatchAssetBase)
        );
    }

    #[test]
    fn verify_supply_ik_mismatch_asset_base() {
        let (_, _, action) = supply_test_params(10, 20, b"Asset 1", None, false);
        let (_, _, ik, _, _) = setup_params();

        assert_eq!(
            action.verify_supply(&ik),
            Err(IssueBundleIkMismatchAssetBase)
        );
    }

    #[test]
    fn issue_bundle_basic() {
        let (rng, _, ik, recipient, _) = setup_params();

        let str = "Halo".to_string();
        let str2 = "Halo2".to_string();

        assert_eq!(
            IssueBundle::new(
                ik.clone(),
                vec![b'X'; 513],
                Some(IssueInfo {
                    recipient,
                    value: NoteValue::unsplittable()
                }),
                true,
                rng,
            )
            .unwrap_err(),
            WrongAssetDescSize
        );

        assert_eq!(
            IssueBundle::new(
                ik.clone(),
                b"".to_vec(),
                Some(IssueInfo {
                    recipient,
                    value: NoteValue::unsplittable()
                }),
                true,
                rng,
            )
            .unwrap_err(),
            WrongAssetDescSize
        );

        let (mut bundle, asset) = IssueBundle::new(
            ik.clone(),
            str.clone().into_bytes(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let another_asset = bundle
            .add_recipient(
                &str.into_bytes(),
                recipient,
                NoteValue::from_raw(10),
                false,
                rng,
            )
            .unwrap();
        assert_eq!(asset, another_asset);

        let third_asset = bundle
            .add_recipient(
                str2.as_bytes(),
                recipient,
                NoteValue::from_raw(15),
                true,
                rng,
            )
            .unwrap();
        assert_ne!(asset, third_asset);

        let actions = bundle.actions();
        assert_eq!(actions.len(), 2);

        let action = bundle.get_action_by_asset(&asset).unwrap();
        assert_eq!(action.notes.len(), 3);
        let reference_note = action.notes.get(0).unwrap();
        verify_reference_note(reference_note, asset);
        let first_note = action.notes.get(1).unwrap();
        assert_eq!(first_note.value().inner(), 5);
        assert_eq!(first_note.asset(), asset);
        assert_eq!(first_note.recipient(), recipient);

        let second_note = action.notes.get(2).unwrap();
        assert_eq!(second_note.value().inner(), 10);
        assert_eq!(second_note.asset(), asset);
        assert_eq!(second_note.recipient(), recipient);

        let action2 = bundle.get_action_by_desc(str2.as_bytes()).unwrap();
        assert_eq!(action2.notes.len(), 2);
        let reference_note = action2.notes.get(0).unwrap();
        verify_reference_note(reference_note, AssetBase::derive(&ik, str2.as_bytes()));
        let first_note = action2.notes().get(1).unwrap();
        assert_eq!(first_note.value().inner(), 15);
        assert_eq!(first_note.asset(), third_asset);

        let reference_notes = bundle.get_reference_notes();
        assert_eq!(reference_notes.len(), 2);
        verify_reference_note(reference_notes.get(&asset).unwrap(), asset);
        verify_reference_note(reference_notes.get(&third_asset).unwrap(), third_asset);
    }

    #[test]
    fn issue_bundle_finalize_asset() {
        let (rng, _, ik, recipient, _) = setup_params();

        let (mut bundle, _) = IssueBundle::new(
            ik,
            b"NFT".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(u64::MIN),
            }),
            true,
            rng,
        )
        .expect("Should properly add recipient");

        bundle
            .finalize_action(b"NFT")
            .expect("Should finalize properly");

        assert_eq!(
            bundle.finalize_action(b"Another NFT").unwrap_err(),
            IssueActionNotFound
        );

        assert_eq!(
            bundle.finalize_action(&vec![b'X'; 513]).unwrap_err(),
            WrongAssetDescSize
        );

        assert_eq!(bundle.finalize_action(b"").unwrap_err(), WrongAssetDescSize);
    }

    #[test]
    fn issue_bundle_prepare() {
        let (rng, _, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            b"Frost".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let prepared = bundle.prepare(sighash);
        assert_eq!(prepared.authorization().sighash, sighash);
    }

    #[test]
    fn issue_bundle_sign() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik.clone(),
            b"Sign".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let signed = bundle.prepare(sighash).sign(&isk).unwrap();

        ik.verify(&sighash, &signed.authorization.signature)
            .expect("signature should be valid");
    }

    #[test]
    fn issue_bundle_invalid_isk_for_signature() {
        let (rng, _, ik, recipient, _) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            b"IssueBundle".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let wrong_isk: IssuanceAuthorizingKey = IssuanceAuthorizingKey::random();

        let err = bundle
            .prepare([0; 32])
            .sign(&wrong_isk)
            .expect_err("should not be able to sign");

        assert_eq!(err, IssueBundleIkMismatchAssetBase);
    }

    #[test]
    fn issue_bundle_incorrect_asset_for_signature() {
        let (mut rng, isk, ik, recipient, _) = setup_params();

        // Create a bundle with "normal" note
        let (mut bundle, _) = IssueBundle::new(
            ik,
            b"IssueBundle".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::derive(bundle.ik(), b"zsa_asset"),
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );
        bundle.actions.first_mut().notes.push(note);

        let err = bundle
            .prepare([0; 32])
            .sign(&isk)
            .expect_err("should not be able to sign");

        assert_eq!(err, IssueBundleIkMismatchAssetBase);
    }

    #[test]
    fn issue_bundle_verify() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            b"Verify".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let signed = bundle.prepare(sighash).sign(&isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        let supply_info = verify_issue_bundle(&signed, sighash, prev_finalized).unwrap();

        supply_info.update_finalization_set(prev_finalized);

        assert!(prev_finalized.is_empty());
    }

    #[test]
    fn issue_bundle_verify_with_finalize() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (mut bundle, _) = IssueBundle::new(
            ik.clone(),
            b"Verify with finalize".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(7),
            }),
            true,
            rng,
        )
        .unwrap();

        bundle.finalize_action(b"Verify with finalize").unwrap();

        let signed = bundle.prepare(sighash).sign(&isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        let supply_info = verify_issue_bundle(&signed, sighash, prev_finalized).unwrap();

        supply_info.update_finalization_set(prev_finalized);

        assert_eq!(prev_finalized.len(), 1);
        assert!(prev_finalized.contains(&AssetBase::derive(&ik, b"Verify with finalize")));
    }

    #[test]
    fn issue_bundle_verify_with_supply_info() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let asset1_desc = b"Verify with supply info 1".to_vec();
        let asset2_desc = b"Verify with supply info 2".to_vec();
        let asset3_desc = b"Verify with supply info 3".to_vec();

        let asset1_base = AssetBase::derive(&ik, &asset1_desc);
        let asset2_base = AssetBase::derive(&ik, &asset2_desc);
        let asset3_base = AssetBase::derive(&ik, &asset3_desc);

        let (mut bundle, _) = IssueBundle::new(
            ik,
            asset1_desc.clone(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(7),
            }),
            true,
            rng,
        )
        .unwrap();

        bundle
            .add_recipient(&asset1_desc, recipient, NoteValue::from_raw(8), false, rng)
            .unwrap();

        bundle.finalize_action(&asset1_desc).unwrap();

        bundle
            .add_recipient(&asset2_desc, recipient, NoteValue::from_raw(10), true, rng)
            .unwrap();

        bundle.finalize_action(&asset2_desc).unwrap();

        bundle
            .add_recipient(&asset3_desc, recipient, NoteValue::from_raw(5), true, rng)
            .unwrap();

        let signed = bundle.prepare(sighash).sign(&isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        let supply_info = verify_issue_bundle(&signed, sighash, prev_finalized).unwrap();

        supply_info.update_finalization_set(prev_finalized);

        assert_eq!(prev_finalized.len(), 2);

        assert!(prev_finalized.contains(&asset1_base));
        assert!(prev_finalized.contains(&asset2_base));
        assert!(!prev_finalized.contains(&asset3_base));

        assert_eq!(supply_info.assets.len(), 3);

        assert_eq!(
            supply_info.assets.get(&asset1_base),
            Some(&AssetSupply::new(NoteValue::from_raw(15), true))
        );
        assert_eq!(
            supply_info.assets.get(&asset2_base),
            Some(&AssetSupply::new(NoteValue::from_raw(10), true))
        );
        assert_eq!(
            supply_info.assets.get(&asset3_base),
            Some(&AssetSupply::new(NoteValue::from_raw(5), false))
        );
    }

    #[test]
    fn issue_bundle_verify_fail_previously_finalized() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik.clone(),
            b"already final".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let signed = bundle.prepare(sighash).sign(&isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        let final_type = AssetBase::derive(&ik, b"already final");

        prev_finalized.insert(final_type);

        assert_eq!(
            verify_issue_bundle(&signed, sighash, prev_finalized).unwrap_err(),
            IssueActionPreviouslyFinalizedAssetBase(final_type)
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

        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            b"bad sig".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let wrong_isk: IssuanceAuthorizingKey = IssuanceAuthorizingKey::random();

        let mut signed = bundle.prepare(sighash).sign(&isk).unwrap();

        signed.set_authorization(Signed {
            signature: wrong_isk.try_sign(&sighash).unwrap(),
        });

        let prev_finalized = &HashSet::new();

        assert_eq!(
            verify_issue_bundle(&signed, sighash, prev_finalized).unwrap_err(),
            IssueBundleInvalidSignature
        );
    }

    #[test]
    fn issue_bundle_verify_fail_wrong_sighash() {
        let (rng, isk, ik, recipient, random_sighash) = setup_params();
        let (bundle, _) = IssueBundle::new(
            ik,
            b"Asset description".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let sighash: [u8; 32] = bundle.commitment().into();
        let signed = bundle.prepare(sighash).sign(&isk).unwrap();
        let prev_finalized = &HashSet::new();

        assert_eq!(
            verify_issue_bundle(&signed, random_sighash, prev_finalized).unwrap_err(),
            IssueBundleInvalidSignature
        );
    }

    #[test]
    fn issue_bundle_verify_fail_incorrect_asset_description() {
        let (mut rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            b"Asset description".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let mut signed = bundle.prepare(sighash).sign(&isk).unwrap();

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::derive(signed.ik(), b"zsa_asset"),
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );

        signed.actions.first_mut().notes.push(note);

        let prev_finalized = &HashSet::new();

        assert_eq!(
            verify_issue_bundle(&signed, sighash, prev_finalized).unwrap_err(),
            IssueBundleIkMismatchAssetBase
        );
    }

    #[test]
    fn issue_bundle_verify_fail_incorrect_ik() {
        let asset_description = b"Asset".to_vec();

        let (mut rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            asset_description.clone(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let mut signed = bundle.prepare(sighash).sign(&isk).unwrap();

        let incorrect_isk = IssuanceAuthorizingKey::random();
        let incorrect_ik: IssuanceValidatingKey = (&incorrect_isk).into();

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(55),
            AssetBase::derive(&incorrect_ik, &asset_description),
            Rho::from_nf_old(Nullifier::dummy(&mut rng)),
            &mut rng,
        );

        signed.actions.first_mut().notes = vec![note];

        let prev_finalized = &HashSet::new();

        assert_eq!(
            verify_issue_bundle(&signed, sighash, prev_finalized).unwrap_err(),
            IssueBundleIkMismatchAssetBase
        );
    }

    #[test]
    fn issue_bundle_verify_fail_wrong_asset_descr_size() {
        // we want to inject a "malformed" description for test purposes.
        impl IssueAction {
            pub fn modify_descr(&mut self, new_descr: Vec<u8>) {
                self.asset_desc = new_descr;
            }
        }

        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            b"Asset description".to_vec(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let mut signed = bundle.prepare(sighash).sign(&isk).unwrap();
        let prev_finalized = HashSet::new();

        // 1. Try a description that is too long
        signed.actions.first_mut().modify_descr(vec![b'X'; 513]);

        assert_eq!(
            verify_issue_bundle(&signed, sighash, &prev_finalized).unwrap_err(),
            WrongAssetDescSize
        );

        // 2. Try a description that is empty
        signed.actions.first_mut().modify_descr(b"".to_vec());

        assert_eq!(
            verify_issue_bundle(&signed, sighash, &prev_finalized).unwrap_err(),
            WrongAssetDescSize
        );
    }

    #[test]
    fn issue_bundle_cannot_be_signed_with_asset_base_identity_point() {
        let (isk, bundle, sighash) = identity_point_test_params(10, 20);

        assert_eq!(
            bundle.prepare(sighash).sign(&isk).unwrap_err(),
            AssetBaseCannotBeIdentityPoint
        );
    }

    #[test]
    fn issue_bundle_verify_fail_asset_base_identity_point() {
        let (isk, bundle, sighash) = identity_point_test_params(10, 20);

        let signed = IssueBundle {
            ik: bundle.ik,
            actions: bundle.actions,
            authorization: Signed {
                signature: isk.try_sign(&sighash).unwrap(),
            },
        };

        assert_eq!(
            verify_issue_bundle(&signed, sighash, &HashSet::new()).unwrap_err(),
            AssetBaseCannotBeIdentityPoint
        );
    }

    #[test]
    fn finalize_flag_serialization() {
        let mut rng = OsRng;
        let (_, _, note) = Note::dummy(&mut rng, None, AssetBase::native());

        let action =
            IssueAction::new_with_flags(b"Asset description".to_vec(), vec![note], 0u8).unwrap();
        assert_eq!(action.flags(), 0b0000_0000);

        let action =
            IssueAction::new_with_flags(b"Asset description".to_vec(), vec![note], 1u8).unwrap();
        assert_eq!(action.flags(), 0b0000_0001);

        let action = IssueAction::new_with_flags(b"Asset description".to_vec(), vec![note], 2u8);
        assert!(action.is_none());
    }

    #[test]
    fn issue_bundle_asset_desc_roundtrip() {
        let (rng, _, ik, recipient, _) = setup_params();

        // Generated using https://onlinetools.com/utf8/generate-random-utf8
        let asset_desc_1 = "󅞞 򬪗YV8𱈇m0{둛򙎠[㷊V֤]9Ծ̖l󾓨2닯򗏟iȰ䣄˃Oߺ񗗼🦄"
            .to_string()
            .as_bytes()
            .to_vec();

        // Not well-formed as per Unicode 15.0 specification, Section 3.9, D92
        let asset_desc_2: Vec<u8> = vec![0xc0, 0xaf];

        // Confirm not valid UTF-8
        assert!(String::from_utf8(asset_desc_2.clone()).is_err());

        let (mut bundle, asset_base_1) = IssueBundle::new(
            ik,
            asset_desc_1.clone(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            true,
            rng,
        )
        .unwrap();

        let asset_base_2 = bundle
            .add_recipient(&asset_desc_2, recipient, NoteValue::from_raw(10), true, rng)
            .unwrap();

        // Checks for the case of UTF-8 encoded asset description.
        let action = bundle.get_action_by_asset(&asset_base_1).unwrap();
        assert_eq!(action.asset_desc(), &asset_desc_1);
        let reference_note = action.notes.get(0).unwrap();
        verify_reference_note(reference_note, asset_base_1);
        assert_eq!(action.notes.get(1).unwrap().value().inner(), 5);
        assert_eq!(bundle.get_action_by_desc(&asset_desc_1).unwrap(), action);

        // Checks for the case on non-UTF-8 encoded asset description.
        let action2 = bundle.get_action_by_asset(&asset_base_2).unwrap();
        assert_eq!(action2.asset_desc(), &asset_desc_2);
        let reference_note = action2.notes.get(0).unwrap();
        verify_reference_note(reference_note, asset_base_2);
        assert_eq!(action2.notes.get(1).unwrap().value().inner(), 10);
        assert_eq!(bundle.get_action_by_desc(&asset_desc_2).unwrap(), action2);
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use crate::issuance::{IssueAction, IssueBundle, Prepared, Signed, Unauthorized};
    use crate::keys::testing::arb_issuance_validating_key;
    use crate::note::asset_base::testing::zsa_asset_base;
    use crate::note::testing::arb_zsa_note;
    use k256::schnorr;
    use nonempty::NonEmpty;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::prop_compose;

    prop_compose! {
        /// Generate a uniformly distributed signature
        pub(crate) fn arb_signature()(
            sig_bytes in vec(prop::num::u8::ANY, 64)
        ) -> schnorr::Signature {
            schnorr::Signature::try_from(sig_bytes.as_slice()).unwrap()
        }
    }

    prop_compose! {
        /// Generate an issue action
        pub fn arb_issue_action(asset_desc: Vec<u8>)
        (
            asset in zsa_asset_base(asset_desc.clone()),
        )
        (
            note in arb_zsa_note(asset),
        )-> IssueAction {
            IssueAction{
                asset_desc: asset_desc.clone(),
                notes: vec![note],
                finalize: false,
            }
        }
    }

    prop_compose! {
        /// Generate an arbitrary issue bundle with fake authorization data.
        pub fn arb_unathorized_issue_bundle(n_actions: usize)
        (
            actions in vec(arb_issue_action(b"asset_desc".to_vec()), n_actions),
            ik in arb_issuance_validating_key()
        ) -> IssueBundle<Unauthorized> {
            let actions = NonEmpty::from_vec(actions).unwrap();
            IssueBundle {
                ik,
                actions,
                authorization: Unauthorized
            }
        }
    }

    prop_compose! {
        /// Generate an arbitrary issue bundle with fake authorization data. This bundle does not
        /// necessarily respect consensus rules
        pub fn arb_prepared_issue_bundle(n_actions: usize)
        (
            actions in vec(arb_issue_action(b"asset_desc".to_vec()), n_actions),
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
            actions in vec(arb_issue_action(b"asset_desc".to_vec()), n_actions),
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
