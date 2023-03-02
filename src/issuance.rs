//! Structs related to issuance bundles and the associated logic.
use blake2b_simd::Hash as Blake2bHash;
use nonempty::NonEmpty;
use rand::{CryptoRng, RngCore};
use std::collections::HashSet;
use std::fmt;

use crate::bundle::commitments::{hash_issue_bundle_auth_data, hash_issue_bundle_txid_data};
use crate::issuance::Error::{
    IssueActionAlreadyFinalized, IssueActionIncorrectNoteType, IssueActionNotFound,
    IssueActionPreviouslyFinalizedNoteType, IssueBundleIkMismatchNoteType,
    IssueBundleInvalidSignature, WrongAssetDescSize,
};
use crate::keys::{IssuanceAuthorizingKey, IssuanceValidatingKey};
use crate::note::asset_base::is_asset_desc_of_valid_size;
use crate::note::{AssetBase, Nullifier};
use crate::primitives::redpallas::Signature;

use crate::value::NoteValue;
use crate::{
    primitives::redpallas::{self, SpendAuth},
    Address, Note,
};

/// A bundle of actions to be applied to the ledger.
#[derive(Debug, Clone)]
pub struct IssueBundle<T: IssueAuth> {
    /// The issuer key for the note being created.
    ik: IssuanceValidatingKey,
    /// The list of issue actions that make up this bundle.
    actions: Vec<IssueAction>,
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
    notes: NonEmpty<Note>,
    /// `finalize` will prevent further issuance of the same asset type.
    finalize: bool,
}

impl IssueAction {
    /// Constructs a new `IssueAction`.
    pub fn new(asset_desc: String, note: &Note) -> Self {
        IssueAction {
            asset_desc,
            notes: NonEmpty {
                head: *note,
                tail: vec![],
            },
            finalize: false,
        }
    }

    /// Constructs an `IssueAction` from its constituent parts.
    pub fn from_parts(asset_desc: String, notes: NonEmpty<Note>, finalize: bool) -> Self {
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
    pub fn notes(&self) -> &NonEmpty<Note> {
        &self.notes
    }

    /// Returns whether the asset type was finalized in this action.
    pub fn is_finalized(&self) -> bool {
        self.finalize
    }

    /// Return the `AssetBase` if the provided `ik` is used to derive the `asset_id` for **all** internal notes.
    fn are_note_asset_ids_derived_correctly(
        &self,
        ik: &IssuanceValidatingKey,
    ) -> Result<AssetBase, Error> {
        match self
            .notes
            .iter()
            .try_fold(self.notes().head.asset(), |asset, &note| {
                // Fail if not all note types are equal
                note.asset()
                    .eq(&asset)
                    .then(|| asset)
                    .ok_or(IssueActionIncorrectNoteType)
            }) {
            Ok(asset) => asset // check that the asset was properly derived.
                .eq(&AssetBase::derive(ik, &self.asset_desc))
                .then(|| asset)
                .ok_or(IssueBundleIkMismatchNoteType),
            Err(e) => Err(e),
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
    pub fn actions(&self) -> &Vec<IssueAction> {
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
        actions: Vec<IssueAction>,
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
    pub fn new(ik: IssuanceValidatingKey) -> IssueBundle<Unauthorized> {
        IssueBundle {
            ik,
            actions: Vec::new(),
            authorization: Unauthorized,
        }
    }

    /// Add a new note to the `IssueBundle`.
    ///
    /// Rho will be randomly sampled, similar to dummy note generation.
    ///
    /// # Panics
    ///
    /// Panics if `asset_desc` is empty or longer than 512 bytes.
    pub fn add_recipient(
        &mut self,
        asset_desc: String,
        recipient: Address,
        value: NoteValue,
        finalize: bool,
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

        match self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.asset_desc.eq(&asset_desc))
        {
            // Append to an existing IssueAction.
            Some(action) => {
                if action.finalize {
                    return Err(IssueActionAlreadyFinalized);
                };
                action.notes.push(note);
                finalize.then(|| action.finalize = true);
            }
            // Insert a new IssueAction.
            None => {
                let mut action = IssueAction::new(asset_desc, &note);
                finalize.then(|| action.finalize = true);
                self.actions.push(action);
            }
        }

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
            action
                .are_note_asset_ids_derived_correctly(&expected_ik)
                .map(|_| ()) // Transform Result<NoteType,Error> into Result<(),Error)>.
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
/// A set of previously finalized asset types must be provided.
/// In case of success, `finalized` will contain a set of the provided **and** the newly finalized `AssetBase`s
///
/// The following checks are performed:
/// * For the `IssueBundle`:
///     * the Signature on top of the provided `sighash` verifies correctly.
/// * For each `IssueAction`:
///     * Asset description size is collect.
///     * `AssetBase` for the `IssueAction` has not been previously finalized.
/// * For each `Note` inside an `IssueAction`:
///     * All notes have the same, correct `AssetBase`.
pub fn verify_issue_bundle(
    bundle: &IssueBundle<Signed>,
    sighash: [u8; 32],
    finalized: &mut HashSet<AssetBase>, // The finalization set.
) -> Result<(), Error> {
    if let Err(e) = bundle.ik.verify(&sighash, &bundle.authorization.signature) {
        return Err(IssueBundleInvalidSignature(e));
    };

    let s = &mut HashSet::<AssetBase>::new();

    let newly_finalized = bundle
        .actions()
        .iter()
        .try_fold(s, |newly_finalized, action| {
            if !is_asset_desc_of_valid_size(action.asset_desc()) {
                return Err(WrongAssetDescSize);
            }

            // Fail if any note in the IssueAction has incorrect note type.
            let asset = action.are_note_asset_ids_derived_correctly(bundle.ik())?;

            // Fail if the asset was previously finalized.
            if finalized.contains(&asset) || newly_finalized.contains(&asset) {
                return Err(IssueActionPreviouslyFinalizedNoteType(asset));
            }

            // Add to the finalization set, if needed.
            if action.is_finalized() {
                newly_finalized.insert(asset);
            }

            // Proceed with the new finalization set.
            Ok(newly_finalized)
        })?;

    finalized.extend(newly_finalized.iter());
    Ok(())
}

/// Errors produced during the issuance process
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Unable to add note to the IssueAction since it has already been finalized.
    IssueActionAlreadyFinalized,
    /// The requested IssueAction not exists in the bundle.
    IssueActionNotFound,
    /// Not all `NoteType`s are the same inside the action.
    IssueActionIncorrectNoteType,
    /// The provided `isk` and the driven `ik` does not match at least one note type.
    IssueBundleIkMismatchNoteType,
    /// `asset_desc` should be between 1 and 512 bytes.
    WrongAssetDescSize,

    /// Verification errors:
    /// Invalid signature.
    IssueBundleInvalidSignature(reddsa::Error),
    /// The provided `NoteType` has been previously finalized.
    IssueActionPreviouslyFinalizedNoteType(AssetBase),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueActionAlreadyFinalized => {
                write!(
                    f,
                    "unable to add note to the IssueAction since it has already been finalized"
                )
            }
            IssueActionNotFound => {
                write!(f, "the requested IssueAction not exists in the bundle.")
            }
            IssueActionIncorrectNoteType => {
                write!(f, "not all `NoteType`s are the same inside the action")
            }
            IssueBundleIkMismatchNoteType => {
                write!(
                    f,
                    "the provided `isk` and the driven `ik` does not match at least one note type"
                )
            }
            WrongAssetDescSize => {
                write!(f, "`asset_desc` should be between 1 and 512 bytes")
            }
            IssueBundleInvalidSignature(_) => {
                write!(f, "invalid signature")
            }
            IssueActionPreviouslyFinalizedNoteType(_) => {
                write!(f, "the provided `NoteType` has been previously finalized")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IssueBundle;
    use crate::issuance::Error::{
        IssueActionAlreadyFinalized, IssueActionIncorrectNoteType, IssueActionNotFound,
        IssueActionPreviouslyFinalizedNoteType, IssueBundleIkMismatchNoteType,
        IssueBundleInvalidSignature, WrongAssetDescSize,
    };
    use crate::issuance::{verify_issue_bundle, IssueAction, Signed};
    use crate::keys::{
        FullViewingKey, IssuanceAuthorizingKey, IssuanceValidatingKey, Scope, SpendingKey,
    };
    use crate::note::{AssetBase, Nullifier};
    use crate::value::NoteValue;
    use crate::{Address, Note};
    use nonempty::NonEmpty;
    use rand::rngs::OsRng;
    use rand::RngCore;
    use reddsa::Error::InvalidSignature;
    use std::borrow::BorrowMut;
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

        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        let mut sighash = [0u8; 32];
        rng.fill_bytes(&mut sighash);

        (rng, isk, ik, recipient, sighash)
    }

    #[test]
    fn issue_bundle_basic() {
        let (rng, _, ik, recipient, _) = setup_params();

        let mut bundle = IssueBundle::new(ik);

        let str = String::from("Halo");
        let str2 = String::from("Halo2");

        assert_eq!(
            bundle
                .add_recipient(
                    String::from_utf8(vec![b'X'; 513]).unwrap(),
                    recipient,
                    NoteValue::unsplittable(),
                    true,
                    rng,
                )
                .unwrap_err(),
            WrongAssetDescSize
        );

        assert_eq!(
            bundle
                .add_recipient(
                    "".to_string(),
                    recipient,
                    NoteValue::unsplittable(),
                    true,
                    rng,
                )
                .unwrap_err(),
            WrongAssetDescSize
        );

        let asset = bundle
            .add_recipient(str.clone(), recipient, NoteValue::from_raw(5), false, rng)
            .unwrap();

        let another_asset = bundle
            .add_recipient(str, recipient, NoteValue::from_raw(10), false, rng)
            .unwrap();
        assert_eq!(asset, another_asset);

        let third_asset = bundle
            .add_recipient(str2.clone(), recipient, NoteValue::from_raw(15), false, rng)
            .unwrap();
        assert_ne!(asset, third_asset);

        let actions = bundle.actions();
        assert_eq!(actions.len(), 2);

        let action = bundle.get_action_by_type(asset).unwrap();
        assert_eq!(action.notes.len(), 2);
        assert_eq!(action.notes.first().value().inner(), 5);
        assert_eq!(action.notes.first().asset(), asset);
        assert_eq!(action.notes.first().recipient(), recipient);

        assert_eq!(action.notes.tail().first().unwrap().value().inner(), 10);
        assert_eq!(action.notes.tail().first().unwrap().asset(), asset);
        assert_eq!(action.notes.tail().first().unwrap().recipient(), recipient);

        let action2 = bundle.get_action(str2).unwrap();
        assert_eq!(action2.notes.len(), 1);
        assert_eq!(action2.notes().first().value().inner(), 15);
        assert_eq!(action2.notes().first().asset(), third_asset);
    }

    #[test]
    fn issue_bundle_finalize_asset() {
        let (rng, _, ik, recipient, _) = setup_params();

        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from("NFT"),
                recipient,
                NoteValue::from_raw(u64::MIN),
                false,
                rng,
            )
            .expect("Should properly add recipient");

        bundle
            .finalize_action(String::from("NFT"))
            .expect("Should finalize properly");

        assert_eq!(
            bundle
                .add_recipient(
                    String::from("NFT"),
                    recipient,
                    NoteValue::unsplittable(),
                    false,
                    rng,
                )
                .unwrap_err(),
            IssueActionAlreadyFinalized
        );

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

        bundle
            .add_recipient(
                String::from("Another NFT"),
                recipient,
                NoteValue::unsplittable(),
                true,
                rng,
            )
            .expect("should add and finalize");

        assert_eq!(
            bundle
                .add_recipient(
                    String::from("Another NFT"),
                    recipient,
                    NoteValue::unsplittable(),
                    true,
                    rng,
                )
                .unwrap_err(),
            IssueActionAlreadyFinalized
        );
    }

    #[test]
    fn issue_bundle_prepare() {
        let (rng, _, ik, recipient, sighash) = setup_params();

        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from("Frost"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let prepared = bundle.prepare(sighash);
        assert_eq!(prepared.authorization().sighash, sighash);
    }

    #[test]
    fn issue_bundle_sign() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let mut bundle = IssueBundle::new(ik.clone());

        bundle
            .add_recipient(
                String::from("Sign"),
                recipient,
                NoteValue::from_raw(5),
                false,
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

        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from("IssueBundle"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let wrong_isk: IssuanceAuthorizingKey = (&SpendingKey::random(&mut OsRng)).into();

        let err = bundle
            .prepare([0; 32])
            .sign(rng, &wrong_isk)
            .expect_err("should not be able to sign");

        assert_eq!(err, IssueBundleIkMismatchNoteType);
    }

    #[test]
    fn issue_bundle_incorrect_asset_for_signature() {
        let (mut rng, isk, ik, recipient, _) = setup_params();

        let mut bundle = IssueBundle::new(ik);

        // Add "normal" note
        bundle
            .add_recipient(
                String::from("IssueBundle"),
                recipient,
                NoteValue::from_raw(5),
                false,
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
        bundle
            .actions
            .first_mut()
            .unwrap()
            .notes
            .borrow_mut()
            .push(note);

        let err = bundle
            .prepare([0; 32])
            .sign(rng, &isk)
            .expect_err("should not be able to sign");

        assert_eq!(err, IssueActionIncorrectNoteType);
    }

    #[test]
    fn issue_bundle_verify() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from("Verify"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();

        let prev_finalized = &mut HashSet::new();

        let res = verify_issue_bundle(&signed, sighash, prev_finalized);
        assert!(res.is_ok());
        assert!(prev_finalized.is_empty());
    }

    #[test]
    fn issue_bundle_verify_with_finalize() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let mut bundle = IssueBundle::new(ik.clone());

        bundle
            .add_recipient(
                String::from("Verify with finalize"),
                recipient,
                NoteValue::from_raw(7),
                true,
                rng,
            )
            .unwrap();

        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();

        let prev_finalized = &mut HashSet::new();

        let res = verify_issue_bundle(&signed, sighash, prev_finalized);
        assert!(res.is_ok());
        assert!(prev_finalized.contains(&AssetBase::derive(
            &ik,
            &String::from("Verify with finalize")
        )));
        assert_eq!(prev_finalized.len(), 1);
    }

    #[test]
    fn issue_bundle_verify_fail_previously_finalized() {
        let (rng, isk, ik, recipient, sighash) = setup_params();

        let mut bundle = IssueBundle::new(ik.clone());

        bundle
            .add_recipient(
                String::from("already final"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        let final_type = AssetBase::derive(&ik, &String::from("already final"));

        prev_finalized.insert(final_type);

        let finalized = verify_issue_bundle(&signed, sighash, prev_finalized);
        assert_eq!(
            finalized.unwrap_err(),
            IssueActionPreviouslyFinalizedNoteType(final_type)
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

        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from("bad sig"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let wrong_isk: IssuanceAuthorizingKey = (&SpendingKey::random(&mut rng)).into();

        let mut signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();

        signed.set_authorization(Signed {
            signature: wrong_isk.sign(&mut rng, &sighash),
        });

        let prev_finalized = &mut HashSet::new();

        assert_eq!(
            verify_issue_bundle(&signed, sighash, prev_finalized).unwrap_err(),
            IssueBundleInvalidSignature(InvalidSignature)
        );
    }

    #[test]
    fn issue_bundle_verify_fail_wrong_sighash() {
        let (rng, isk, ik, recipient, random_sighash) = setup_params();
        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from("Asset description"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let sighash: [u8; 32] = bundle.commitment().into();
        let signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        // 2. Try empty description
        let finalized = verify_issue_bundle(&signed, random_sighash, prev_finalized);

        assert_eq!(
            finalized.unwrap_err(),
            IssueBundleInvalidSignature(InvalidSignature)
        );
    }

    #[test]
    fn issue_bundle_verify_fail_incorrect_asset_description() {
        let (mut rng, isk, ik, recipient, sighash) = setup_params();

        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from("Asset description"),
                recipient,
                NoteValue::from_raw(5),
                false,
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

        signed
            .actions
            .first_mut()
            .unwrap()
            .notes
            .borrow_mut()
            .push(note);

        let prev_finalized = &mut HashSet::new();
        let err = verify_issue_bundle(&signed, sighash, prev_finalized).unwrap_err();

        assert_eq!(err, IssueActionIncorrectNoteType);
    }

    #[test]
    fn issue_bundle_verify_fail_incorrect_ik() {
        let asset_description = "Asset";

        let (mut rng, isk, ik, recipient, sighash) = setup_params();

        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from(asset_description),
                recipient,
                NoteValue::from_raw(5),
                false,
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

        signed.actions.first_mut().unwrap().notes = NonEmpty::new(note);

        let prev_finalized = &mut HashSet::new();
        let err = verify_issue_bundle(&signed, sighash, prev_finalized).unwrap_err();

        assert_eq!(err, IssueBundleIkMismatchNoteType);
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

        let mut bundle = IssueBundle::new(ik);

        bundle
            .add_recipient(
                String::from("Asset description"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let mut signed = bundle.prepare(sighash).sign(rng, &isk).unwrap();
        let prev_finalized = &mut HashSet::new();

        // 1. Try too long description
        signed
            .actions
            .first_mut()
            .unwrap()
            .modify_descr(String::from_utf8(vec![b'X'; 513]).unwrap());
        let finalized = verify_issue_bundle(&signed, sighash, prev_finalized);

        assert_eq!(finalized.unwrap_err(), WrongAssetDescSize);

        // 2. Try empty description
        signed
            .actions
            .first_mut()
            .unwrap()
            .modify_descr("".to_string());
        let finalized = verify_issue_bundle(&signed, sighash, prev_finalized);

        assert_eq!(finalized.unwrap_err(), WrongAssetDescSize);
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
            IssueAction::new(asset_desc.clone(), &note)
        }
    }

    prop_compose! {
        /// Generate an arbitrary issue bundle with fake authorization data.
        pub fn arb_unathorized_issue_bundle(n_actions: usize)
        (
            actions in vec(arb_issue_action("asset_desc".to_string()), n_actions),
            ik in arb_issuance_validating_key()
        ) -> IssueBundle<Unauthorized> {
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

            IssueBundle {
                ik,
                actions,
                authorization: Prepared { sighash: fake_sighash },
            }.sign(rng, &isk).unwrap()
        }
    }
}
