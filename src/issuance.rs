//! Structs related to issuance bundles and the associated logic.
use blake2b_simd::Hash as Blake2bHash;
use nonempty::NonEmpty;
use rand::{CryptoRng, RngCore};
use std::collections::HashSet;
use std::fmt;

use crate::bundle::commitments::{hash_issue_bundle_auth_data, hash_issue_bundle_txid_data};
use crate::issuance::Error::{
    IssueActionNotFound, IssueActionPreviouslyFinalizedAssetBase,
    IssueActionWithoutNoteNotFinalized, IssueBundleIkMismatchAssetBase,
    IssueBundleInvalidSignature, ValueSumOverflow, WrongAssetDescSize,
};
use crate::keys::{IssuanceAuthorizingKey, IssuanceValidatingKey};
use crate::note::asset_base::is_asset_desc_of_valid_size;
use crate::note::{AssetBase, Nullifier};
use crate::primitives::redpallas::Signature;

use crate::value::{NoteValue, ValueSum};
use crate::{
    primitives::redpallas::{self, SpendAuth},
    Address, Note,
};

use crate::supply_info::{AssetSupply, SupplyInfo};

/// A bundle of actions to be applied to the ledger.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct IssueAction {
    /// Asset description for verification.
    asset_desc: String,
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
    pub fn new_with_flags(asset_desc: String, notes: Vec<Note>, flags: u8) -> Option<Self> {
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
    pub fn from_parts(asset_desc: String, notes: Vec<Note>, finalize: bool) -> Self {
        IssueAction {
            asset_desc,
            notes,
            finalize,
        }
    }

    /// Returns the asset description for the note being created.
    pub fn asset_desc(&self) -> &str {
        &self.asset_desc
    }

    /// Returns the issued notes.
    pub fn notes(&self) -> &Vec<Note> {
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
            .try_fold(ValueSum::zero(), |value_sum, &note| {
                // All assets should be derived correctly
                note.asset()
                    .eq(&issue_asset)
                    .then(|| ())
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
    pub fn flags(&self) -> u8 {
        self.finalize.then(|| 0b0000_0001).unwrap_or(0b0000_0000)
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
#[derive(Debug, Clone)]
pub struct Signed {
    signature: redpallas::Signature<SpendAuth>,
}

impl Signed {
    /// Returns the signature for this authorization.
    pub fn signature(&self) -> &redpallas::Signature<SpendAuth> {
        &self.signature
    }

    /// Constructs an `Signed` from its constituent parts.
    pub fn from_parts(signature: Signature<SpendAuth>) -> Self {
        Signed { signature }
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
    pub fn get_all_notes(&self) -> Vec<Note> {
        self.actions
            .iter()
            .flat_map(|action| action.notes.clone().into_iter())
            .collect()
    }

    /// Returns the authorization for this action.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Find an action by `ik` and `asset_desc` for a given `IssueBundle`.
    pub fn get_action(&self, asset_desc: String) -> Option<&IssueAction> {
        self.actions.iter().find(|a| a.asset_desc.eq(&asset_desc))
    }

    /// Find an action by `asset` for a given `IssueBundle`.
    pub fn get_action_by_type(&self, asset: AssetBase) -> Option<&IssueAction> {
        let action = self
            .actions
            .iter()
            .find(|a| AssetBase::derive(&self.ik, &a.asset_desc).eq(&asset));
        action
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
    /// # Errors
    ///
    /// This function may return an error in any of the following cases:
    ///
    /// * `WrongAssetDescSize`: If `asset_desc` is empty or longer than 512 bytes.
    pub fn new(
        ik: IssuanceValidatingKey,
        asset_desc: String,
        issue_info: Option<IssueInfo>,
        mut rng: impl RngCore,
    ) -> Result<(IssueBundle<Unauthorized>, AssetBase), Error> {
        if !is_asset_desc_of_valid_size(&asset_desc) {
            return Err(WrongAssetDescSize);
        }

        let asset = AssetBase::derive(&ik, &asset_desc);

        let action = match issue_info {
            None => IssueAction {
                asset_desc,
                notes: vec![],
                finalize: true,
            },
            Some(issue_info) => {
                let note = Note::new(
                    issue_info.recipient,
                    issue_info.value,
                    asset,
                    Nullifier::dummy(&mut rng),
                    &mut rng,
                );

                IssueAction {
                    asset_desc,
                    notes: vec![note],
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
    ///
    /// # Errors
    ///
    /// This function may return an error in any of the following cases:
    ///
    /// * `WrongAssetDescSize`: If `asset_desc` is empty or longer than 512 bytes.
    pub fn add_recipient(
        &mut self,
        asset_desc: String,
        recipient: Address,
        value: NoteValue,
        mut rng: impl RngCore,
    ) -> Result<AssetBase, Error> {
        if !is_asset_desc_of_valid_size(&asset_desc) {
            return Err(WrongAssetDescSize);
        }

        let asset = AssetBase::derive(&self.ik, &asset_desc);

        let note = Note::new(
            recipient,
            value,
            asset,
            Nullifier::dummy(&mut rng),
            &mut rng,
        );

        let action = self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.asset_desc.eq(&asset_desc));

        match action {
            Some(action) => {
                // Append to an existing IssueAction.
                action.notes.push(note);
            }
            None => {
                // Insert a new IssueAction.
                self.actions.push(IssueAction {
                    asset_desc,
                    notes: vec![note],
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
    pub fn finalize_action(&mut self, asset_desc: String) -> Result<(), Error> {
        if !is_asset_desc_of_valid_size(&asset_desc) {
            return Err(WrongAssetDescSize);
        }

        match self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.asset_desc.eq(&asset_desc))
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

impl IssueBundle<Prepared> {
    /// Sign the `IssueBundle`.
    /// The call makes sure that the provided `isk` matches the `ik` and the driven `asset` for each note in the bundle.
    pub fn sign<R: RngCore + CryptoRng>(
        self,
        mut rng: R,
        isk: &IssuanceAuthorizingKey,
    ) -> Result<IssueBundle<Signed>, Error> {
        let expected_ik: IssuanceValidatingKey = (isk).into();

        // Make sure the `expected_ik` matches the `asset` for all notes.
        self.actions.iter().try_for_each(|action| {
            action.verify_supply(&expected_ik)?;
            Ok(())
        })?;

        Ok(IssueBundle {
            ik: self.ik,
            actions: self.actions,
            authorization: Signed {
                signature: isk.sign(&mut rng, &self.authorization.sighash),
            },
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
        .map_err(IssueBundleInvalidSignature)?;

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
    /// The provided `isk` and the driven `ik` does not match at least one note type.
    IssueBundleIkMismatchAssetBase,
    /// `asset_desc` should be between 1 and 512 bytes.
    WrongAssetDescSize,
    /// The `IssueAction` is not finalized but contains no notes.
    IssueActionWithoutNoteNotFinalized,

    /// Verification errors:
    /// Invalid signature.
    IssueBundleInvalidSignature(reddsa::Error),
    /// The provided `AssetBase` has been previously finalized.
    IssueActionPreviouslyFinalizedAssetBase(AssetBase),

    /// Overflow error occurred while calculating the value of the asset
    ValueSumOverflow,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueActionNotFound => {
                write!(f, "the requested IssueAction not exists in the bundle.")
            }
            IssueBundleIkMismatchAssetBase => {
                write!(
                    f,
                    "the provided `isk` and the driven `ik` does not match at least one note type"
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
            IssueBundleInvalidSignature(_) => {
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
    use crate::issuance::Error::{
        IssueActionNotFound, IssueActionPreviouslyFinalizedAssetBase,
        IssueBundleIkMismatchAssetBase, IssueBundleInvalidSignature, WrongAssetDescSize,
    };
    use crate::issuance::{verify_issue_bundle, IssueAction, Signed};
    use crate::keys::{
        FullViewingKey, IssuanceAuthorizingKey, IssuanceValidatingKey, Scope, SpendingKey,
    };
    use crate::note::{AssetBase, Nullifier};
    use crate::value::{NoteValue, ValueSum};
    use crate::{Address, Note};
    use rand::rngs::OsRng;
    use rand::RngCore;
    use reddsa::Error::InvalidSignature;
    use std::collections::HashSet;

    fn setup_params() -> (
        OsRng,
        IssuanceAuthorizingKey,
        IssuanceValidatingKey,
        Address,
        [u8; 32],
    ) {
        let mut rng = OsRng;

        let sk = SpendingKey::random(&mut rng);
        let isk: IssuanceAuthorizingKey = (&sk).into();
        let ik: IssuanceValidatingKey = (&isk).into();

        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let recipient = fvk.address_at(0u32, Scope::External);

        let mut sighash = [0u8; 32];
        rng.fill_bytes(&mut sighash);

        (rng, isk, ik, recipient, sighash)
    }

    fn setup_verify_supply_test_params(
        note1_value: u64,
        note2_value: u64,
        note1_asset_desc: &str,
        note2_asset_desc: Option<&str>, // if None, both notes use the same asset
        finalize: bool,
    ) -> (IssuanceValidatingKey, AssetBase, IssueAction) {
        let (mut rng, _, ik, recipient, _) = setup_params();

        let asset = AssetBase::derive(&ik, note1_asset_desc);
        let note2_asset = note2_asset_desc.map_or(asset, |desc| AssetBase::derive(&ik, desc));

        let note1 = Note::new(
            recipient,
            NoteValue::from_raw(note1_value),
            asset,
            Nullifier::dummy(&mut rng),
            &mut rng,
        );

        let note2 = Note::new(
            recipient,
            NoteValue::from_raw(note2_value),
            note2_asset,
            Nullifier::dummy(&mut rng),
            &mut rng,
        );

        (
            ik,
            asset,
            IssueAction::from_parts(note1_asset_desc.into(), vec![note1, note2], finalize),
        )
    }

    #[test]
    fn test_verify_supply_valid() {
        let (ik, test_asset, action) =
            setup_verify_supply_test_params(10, 20, "Asset 1", None, false);

        let result = action.verify_supply(&ik);

        assert!(result.is_ok());

        let (asset, supply) = result.unwrap();

        assert_eq!(asset, test_asset);
        assert_eq!(supply.amount, ValueSum::from_raw(30));
        assert!(!supply.is_finalized);
    }

    #[test]
    fn test_verify_supply_finalized() {
        let (ik, test_asset, action) =
            setup_verify_supply_test_params(10, 20, "Asset 1", None, true);

        let result = action.verify_supply(&ik);

        assert!(result.is_ok());

        let (asset, supply) = result.unwrap();

        assert_eq!(asset, test_asset);
        assert_eq!(supply.amount, ValueSum::from_raw(30));
        assert!(supply.is_finalized);
    }

    #[test]
    fn test_verify_supply_incorrect_asset_base() {
        let (ik, _, action) =
            setup_verify_supply_test_params(10, 20, "Asset 1", Some("Asset 2"), false);

        assert_eq!(
            action.verify_supply(&ik),
            Err(IssueBundleIkMismatchAssetBase)
        );
    }

    #[test]
    fn test_verify_supply_ik_mismatch_asset_base() {
        let (_, _, action) = setup_verify_supply_test_params(10, 20, "Asset 1", None, false);
        let (_, _, ik, _, _) = setup_params();

        assert_eq!(
            action.verify_supply(&ik),
            Err(IssueBundleIkMismatchAssetBase)
        );
    }

    #[test]
    fn issue_bundle_basic() {
        let (rng, _, ik, recipient, _) = setup_params();

        let str = String::from("Halo");
        let str2 = String::from("Halo2");

        assert_eq!(
            IssueBundle::new(
                ik.clone(),
                String::from_utf8(vec![b'X'; 513]).unwrap(),
                Some(IssueInfo {
                    recipient,
                    value: NoteValue::unsplittable()
                }),
                rng,
            )
            .unwrap_err(),
            WrongAssetDescSize
        );

        assert_eq!(
            IssueBundle::new(
                ik.clone(),
                "".to_string(),
                Some(IssueInfo {
                    recipient,
                    value: NoteValue::unsplittable()
                }),
                rng,
            )
            .unwrap_err(),
            WrongAssetDescSize
        );

        let (mut bundle, asset) = IssueBundle::new(
            ik,
            str.clone(),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let another_asset = bundle
            .add_recipient(str, recipient, NoteValue::from_raw(10), rng)
            .unwrap();
        assert_eq!(asset, another_asset);

        let third_asset = bundle
            .add_recipient(str2.clone(), recipient, NoteValue::from_raw(15), rng)
            .unwrap();
        assert_ne!(asset, third_asset);

        let actions = bundle.actions();
        assert_eq!(actions.len(), 2);

        let action = bundle.get_action_by_type(asset).unwrap();
        assert_eq!(action.notes.len(), 2);
        assert_eq!(action.notes.first().unwrap().value().inner(), 5);
        assert_eq!(action.notes.first().unwrap().asset(), asset);
        assert_eq!(action.notes.first().unwrap().recipient(), recipient);

        assert_eq!(action.notes.get(1).unwrap().value().inner(), 10);
        assert_eq!(action.notes.get(1).unwrap().asset(), asset);
        assert_eq!(action.notes.get(1).unwrap().recipient(), recipient);

        let action2 = bundle.get_action(str2).unwrap();
        assert_eq!(action2.notes.len(), 1);
        assert_eq!(action2.notes().first().unwrap().value().inner(), 15);
        assert_eq!(action2.notes().first().unwrap().asset(), third_asset);
    }

    #[test]
    fn issue_bundle_finalize_asset() {
        let (rng, _, ik, recipient, _) = setup_params();

        let (mut bundle, _) = IssueBundle::new(
            ik,
            String::from("NFT"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(u64::MIN),
            }),
            rng,
        )
        .expect("Should properly add recipient");

        bundle
            .finalize_action(String::from("NFT"))
            .expect("Should finalize properly");

        assert_eq!(
            bundle
                .finalize_action(String::from("Another NFT"))
                .unwrap_err(),
            IssueActionNotFound
        );

        assert_eq!(
            bundle
                .finalize_action(String::from_utf8(vec![b'X'; 513]).unwrap())
                .unwrap_err(),
            WrongAssetDescSize
        );

        assert_eq!(
            bundle.finalize_action("".to_string()).unwrap_err(),
            WrongAssetDescSize
        );
    }

    #[test]
    fn issue_bundle_prepare() {
        let (rng, _, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            String::from("Frost"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
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
            String::from("Sign"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();

        ik.verify(&sighash, &signed.authorization.signature)
            .expect("signature should be valid");
    }

    #[test]
    fn issue_bundle_invalid_isk_for_signature() {
        let (rng, _, ik, recipient, _) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            String::from("IssueBundle"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let wrong_isk: IssuanceAuthorizingKey = (&SpendingKey::random(&mut OsRng)).into();

        let err = bundle
            .prepare([0; 32])
            .sign(rng, &wrong_isk)
            .expect_err("should not be able to sign");

        assert_eq!(err, IssueBundleIkMismatchAssetBase);
    }

    #[test]
    fn issue_bundle_incorrect_asset_for_signature() {
        let (mut rng, isk, ik, recipient, _) = setup_params();

        // Create a bundle with "normal" note
        let (mut bundle, _) = IssueBundle::new(
            ik,
            String::from("IssueBundle"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::derive(bundle.ik(), "zsa_asset"),
            Nullifier::dummy(&mut rng),
            &mut rng,
        );
        bundle.actions.first_mut().notes.push(note);

        let err = bundle
            .prepare([0; 32])
            .sign(rng, &isk)
            .expect_err("should not be able to sign");

        assert_eq!(err, IssueBundleIkMismatchAssetBase);
    }

    #[test]
    fn issue_bundle_verify() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            String::from("Verify"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
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
            String::from("Verify with finalize"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(7),
            }),
            rng,
        )
        .unwrap();

        bundle
            .finalize_action(String::from("Verify with finalize"))
            .unwrap();

        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        let supply_info = verify_issue_bundle(&signed, sighash, prev_finalized).unwrap();

        supply_info.update_finalization_set(prev_finalized);

        assert_eq!(prev_finalized.len(), 1);
        assert!(prev_finalized.contains(&AssetBase::derive(&ik, "Verify with finalize")));
    }

    #[test]
    fn issue_bundle_verify_with_supply_info() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let asset1_desc = "Verify with supply info 1";
        let asset2_desc = "Verify with supply info 2";
        let asset3_desc = "Verify with supply info 3";

        let asset1_base = AssetBase::derive(&ik, &String::from(asset1_desc));
        let asset2_base = AssetBase::derive(&ik, &String::from(asset2_desc));
        let asset3_base = AssetBase::derive(&ik, &String::from(asset3_desc));

        let (mut bundle, _) = IssueBundle::new(
            ik,
            String::from(asset1_desc),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(7),
            }),
            rng,
        )
        .unwrap();

        bundle
            .add_recipient(
                String::from(asset1_desc),
                recipient,
                NoteValue::from_raw(8),
                rng,
            )
            .unwrap();

        bundle.finalize_action(String::from(asset1_desc)).unwrap();

        bundle
            .add_recipient(
                String::from(asset2_desc),
                recipient,
                NoteValue::from_raw(10),
                rng,
            )
            .unwrap();

        bundle.finalize_action(String::from(asset2_desc)).unwrap();

        bundle
            .add_recipient(
                String::from(asset3_desc),
                recipient,
                NoteValue::from_raw(5),
                rng,
            )
            .unwrap();

        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
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
            Some(&AssetSupply::new(ValueSum::from_raw(15), true))
        );
        assert_eq!(
            supply_info.assets.get(&asset2_base),
            Some(&AssetSupply::new(ValueSum::from_raw(10), true))
        );
        assert_eq!(
            supply_info.assets.get(&asset3_base),
            Some(&AssetSupply::new(ValueSum::from_raw(5), false))
        );
    }

    #[test]
    fn issue_bundle_verify_fail_previously_finalized() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik.clone(),
            String::from("already final"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        let final_type = AssetBase::derive(&ik, &String::from("already final"));

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

        let (mut rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            String::from("bad sig"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let wrong_isk: IssuanceAuthorizingKey = (&SpendingKey::random(&mut rng)).into();

        let mut signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();

        signed.set_authorization(Signed {
            signature: wrong_isk.sign(&mut rng, &sighash),
        });

        let prev_finalized = &HashSet::new();

        assert_eq!(
            verify_issue_bundle(&signed, sighash, prev_finalized).unwrap_err(),
            IssueBundleInvalidSignature(InvalidSignature)
        );
    }

    #[test]
    fn issue_bundle_verify_fail_wrong_sighash() {
        let (rng, isk, ik, recipient, random_sighash) = setup_params();
        let (bundle, _) = IssueBundle::new(
            ik,
            String::from("Asset description"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let sighash: [u8; 32] = bundle.commitment().into();
        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
        let prev_finalized = &HashSet::new();

        assert_eq!(
            verify_issue_bundle(&signed, random_sighash, prev_finalized).unwrap_err(),
            IssueBundleInvalidSignature(InvalidSignature)
        );
    }

    #[test]
    fn issue_bundle_verify_fail_incorrect_asset_description() {
        let (mut rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            String::from("Asset description"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let mut signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(5),
            AssetBase::derive(signed.ik(), "zsa_asset"),
            Nullifier::dummy(&mut rng),
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
        let asset_description = "Asset";

        let (mut rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            String::from(asset_description),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let mut signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();

        let incorrect_sk = SpendingKey::random(&mut rng);
        let incorrect_isk: IssuanceAuthorizingKey = (&incorrect_sk).into();
        let incorrect_ik: IssuanceValidatingKey = (&incorrect_isk).into();

        // Add "bad" note
        let note = Note::new(
            recipient,
            NoteValue::from_raw(55),
            AssetBase::derive(&incorrect_ik, asset_description),
            Nullifier::dummy(&mut rng),
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
        // we want to inject "bad" description for test purposes.
        impl IssueAction {
            pub fn modify_descr(&mut self, new_descr: String) {
                self.asset_desc = new_descr;
            }
        }

        let (rng, isk, ik, recipient, sighash) = setup_params();

        let (bundle, _) = IssueBundle::new(
            ik,
            String::from("Asset description"),
            Some(IssueInfo {
                recipient,
                value: NoteValue::from_raw(5),
            }),
            rng,
        )
        .unwrap();

        let mut signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
        let prev_finalized = HashSet::new();

        // 1. Try too long description
        signed
            .actions
            .first_mut()
            .modify_descr(String::from_utf8(vec![b'X'; 513]).unwrap());

        assert_eq!(
            verify_issue_bundle(&signed, sighash, &prev_finalized).unwrap_err(),
            WrongAssetDescSize
        );

        // 2. Try empty description
        signed.actions.first_mut().modify_descr("".to_string());

        assert_eq!(
            verify_issue_bundle(&signed, sighash, &prev_finalized).unwrap_err(),
            WrongAssetDescSize
        );
    }

    #[test]
    fn test_finalize_flag_serialization() {
        let mut rng = OsRng;
        let (_, _, note) = Note::dummy(&mut rng, None, AssetBase::native());

        let action =
            IssueAction::new_with_flags(String::from("Asset description"), vec![note], 0u8)
                .unwrap();
        assert_eq!(action.flags(), 0b0000_0000);

        let action =
            IssueAction::new_with_flags(String::from("Asset description"), vec![note], 1u8)
                .unwrap();
        assert_eq!(action.flags(), 0b0000_0001);

        let action =
            IssueAction::new_with_flags(String::from("Asset description"), vec![note], 2u8);
        assert!(action.is_none());
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use crate::issuance::{IssueAction, IssueBundle, Prepared, Signed, Unauthorized};
    use crate::keys::testing::{arb_issuance_authorizing_key, arb_issuance_validating_key};
    use crate::note::asset_base::testing::zsa_asset_id;
    use crate::note::testing::arb_zsa_note;
    use nonempty::NonEmpty;
    use proptest::collection::vec;
    use proptest::prelude::*;
    use proptest::prop_compose;
    use rand::{rngs::StdRng, SeedableRng};

    prop_compose! {
        /// Generate an issue action
        pub fn arb_issue_action(asset_desc: String)
        (
            asset in zsa_asset_id(asset_desc.clone()),
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
            actions in vec(arb_issue_action("asset_desc".to_string()), n_actions),
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
            actions in vec(arb_issue_action("asset_desc".to_string()), n_actions),
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
            actions in vec(arb_issue_action("asset_desc".to_string()), n_actions),
            ik in arb_issuance_validating_key(),
            isk in arb_issuance_authorizing_key(),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY),
            fake_sighash in prop::array::uniform32(prop::num::u8::ANY)
        ) -> IssueBundle<Signed> {
            let rng = StdRng::from_seed(rng_seed);
            let actions = NonEmpty::from_vec(actions).unwrap();
            IssueBundle {
                ik,
                actions,
                authorization: Prepared { sighash: fake_sighash },
            }.sign(rng, &isk).unwrap()
        }
    }
}
