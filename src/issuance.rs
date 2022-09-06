//! Structs related to issuance bundles and the associated logic.
use nonempty::NonEmpty;
use rand::{CryptoRng, RngCore};
use std::fmt;

use crate::keys::{IssuerAuthorizingKey, IssuerValidatingKey};
use crate::note::note_type::MAX_ASSET_DESCRIPTION_SIZE;
use crate::note::{NoteType, Nullifier};
use crate::value::NoteValue;
use crate::{
    primitives::redpallas::{self, SpendAuth},
    Address, Note,
};

impl IssueAction {
    /// Constructs a new `IssueAction`.
    pub fn new(ik: IssuerValidatingKey, asset_desc: String, note: &Note) -> Self {
        IssueAction {
            ik,
            asset_desc,
            notes: NonEmpty {
                head: *note,
                tail: vec![],
            },
            finalize: false,
        }
    }

    /// Constructs an `IssueAction` from its constituent parts.
    pub fn from_parts(
        ik: IssuerValidatingKey,
        asset_desc: String,
        notes: NonEmpty<Note>,
        finalize: bool,
    ) -> Self {
        IssueAction {
            ik,
            asset_desc,
            notes,
            finalize,
        }
    }

    /// Returns the issuer verification key for the note being created.
    pub fn ik(&self) -> &IssuerValidatingKey {
        &self.ik
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
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub(crate) mod testing {
    use super::IssueAction;
    use crate::{note::testing::arb_note, value::NoteValue};
    use nonempty::NonEmpty;
    use proptest::prelude::*;

    use crate::keys::{testing::arb_spending_key, IssuerAuthorizingKey, IssuerValidatingKey};

    prop_compose! {
        /// Generate an issue action with a single note and without authorization data.
        pub fn arb_unauthorized_issue_action(output_value: NoteValue)(
            sk in arb_spending_key(),
            vec in prop::collection::vec(any::<u8>(), 0..=255),
            note in arb_note(output_value),
        ) -> IssueAction {
            let isk: IssuerAuthorizingKey = (&sk).into();
            let ik: IssuerValidatingKey = (&isk).into();
            let asset_desc = String::from_utf8(vec).unwrap();

            IssueAction {
                ik,
                asset_desc,
                notes:NonEmpty::new(note), //todo: replace note type
                finalize: false,
            }
        }
    }

    // prop_compose! {
    //     /// Generate an issue action with invalid (random) authorization data.
    //     pub fn arb_issue_action(output_value: NoteValue)(
    //         sk in arb_spending_key(),
    //         vec in prop::collection::vec(any::<u8>(), 0..=255),
    //         note in arb_note(output_value),
    //         rng_seed in prop::array::uniform32(prop::num::u8::ANY),
    //         fake_sighash in prop::array::uniform32(prop::num::u8::ANY),
    //     ) -> IssueAction {
    //
    //         let mut rng = StdRng::from_seed(rng_seed);
    //         let isk: IssuerAuthorizingKey = (&sk).into();
    //         let ik: IssuerValidatingKey = (&isk).into();
    //
    //         IssueAction {
    //             ik,
    //             asset_desc: String::from_utf8(vec).unwrap(),
    //             notes: NonEmpty::new(note), //todo: replace note type
    //             finalize: false,
    //         }
    //     }
    // }
}

/// An issue action applied to the global ledger.
///
/// Externally, this creates new zsa notes (adding a commitment to the global ledger).
#[derive(Debug, Clone)]
pub struct IssueAction {
    /// The issuer key for the note being created.
    ik: IssuerValidatingKey,
    /// Asset description for verification.
    asset_desc: String,
    /// The newly issued notes.
    notes: NonEmpty<Note>,
    /// Finalize will prevent further issuance of the same asset.
    finalize: bool,
}

/// A bundle of actions to be applied to the ledger.
#[derive(Debug)]
pub struct IssueBundle<T: IssueAuth> {
    /// The list of issue actions that make up this bundle.
    actions: Vec<IssueAction>,
    /// The authorization for this action.
    authorization: T,
}

/// Defines the authorization type of an Issue bundle.
pub trait IssueAuth: fmt::Debug {}

/// Marker for an unauthorized bundle with no proofs or signatures.
#[derive(Debug)]
pub struct Unauthorized;

/// Marker for an unauthorized bundle with injected sighash.
#[derive(Debug)]
pub struct Prepared {
    sighash: [u8; 32],
}

/// Marker for an authorized bundle.
#[derive(Debug)]
pub struct Signed {
    signature: redpallas::Signature<SpendAuth>,
}

impl IssueAuth for Unauthorized {}
impl IssueAuth for Prepared {}
impl IssueAuth for Signed {}

impl Default for IssueBundle<Unauthorized> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: IssueAuth> IssueBundle<T> {
    /// Return the actions for a given `IssueBundle`.
    pub fn actions(&self) -> &Vec<IssueAction> {
        &self.actions
    }

    /// Returns the authorization for this action.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Find an action by `ik` and `asset_desc` for a given `IssueBundle`.
    pub fn get_action(&self, ik: IssuerValidatingKey, asset_desc: String) -> Option<&IssueAction> {
        self.actions
            .iter()
            .find(|a| a.ik.eq(&ik) && a.asset_desc.eq(&asset_desc))
    }

    /// Find an action by `note_type` for a given `IssueBundle`.
    pub fn get_action_by_type(&self, note_type: NoteType) -> Option<&IssueAction> {
        let action = self
            .actions
            .iter()
            .find(|a| NoteType::derive(&a.ik, &a.asset_desc).eq(&note_type));
        action
    }
}

impl IssueBundle<Unauthorized> {
    /// Constructs a new `IssueBundle`.
    pub fn new() -> IssueBundle<Unauthorized> {
        IssueBundle {
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
        ik: IssuerValidatingKey,
        asset_desc: String,
        recipient: Address,
        value: NoteValue,
        finalize: bool,
        mut rng: impl RngCore,
    ) -> Result<NoteType, Error> {
        assert!(!asset_desc.is_empty() && asset_desc.bytes().len() <= MAX_ASSET_DESCRIPTION_SIZE);

        let note_type = NoteType::derive(&ik, &asset_desc);

        let note = Note::new(
            recipient,
            value,
            note_type,
            Nullifier::dummy(&mut rng),
            &mut rng,
        );

        match self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.ik.eq(&ik) && issue_action.asset_desc.eq(&asset_desc))
        {
            // Append to an existing IssueAction.
            Some(action) => {
                if action.finalize {
                    return Err(Error::IssueActionAlreadyFinalized);
                };
                action.notes.push(note);
                finalize.then(|| action.finalize = true);
            }
            // Insert a new IssueAction.
            None => {
                let mut action = IssueAction::new(ik.clone(), asset_desc, &note);
                finalize.then(|| action.finalize = true);
                self.actions.push(action);
            }
        }

        Ok(note_type)
    }

    /// Finalizes a given `IssueAction`
    ///
    /// # Panics
    ///
    /// Panics if `asset_desc` is empty or longer than 512 bytes.
    pub fn finalize_action(
        &mut self,
        ik: IssuerValidatingKey,
        asset_desc: String,
    ) -> Result<(), Error> {
        assert!(!asset_desc.is_empty() && asset_desc.bytes().len() <= MAX_ASSET_DESCRIPTION_SIZE);

        match self
            .actions
            .iter_mut()
            .find(|issue_action| issue_action.ik.eq(&ik) && issue_action.asset_desc.eq(&asset_desc))
        {
            Some(issue_action) => {
                issue_action.finalize = true;
            }
            None => {
                return Err(Error::IssueActionNotFound);
            }
        }

        Ok(())
    }

    /// Loads the sighash into the bundle, as preparation for signing.
    pub fn prepare(self, sighash: [u8; 32]) -> IssueBundle<Prepared> {
        IssueBundle {
            actions: self.actions,
            authorization: Prepared { sighash },
        }
    }
}

impl IssueBundle<Prepared> {
    /// Sign all the relevant actions
    pub fn sign<R: RngCore + CryptoRng>(
        self,
        mut rng: R,
        isk: &IssuerAuthorizingKey,
    ) -> IssueBundle<Signed> {
        //let expected_ik: IssuerValidatingKey = (isk).into();
        // todo: check ik fits isk
        // assert_eq!(expected_ik, self.ik);
        IssueBundle {
            actions: self.actions,
            authorization: Signed {
                signature: isk.sign(&mut rng, &self.authorization.sighash),
            },
        }
    }
}

/// Errors produced during the issuance process
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Unable to add note to the IssueAction since it has already been finalized.
    IssueActionAlreadyFinalized,
    /// The requested IssueAction not exists in the bundle.
    IssueActionNotFound,
}

#[cfg(test)]
mod tests {
    use super::IssueBundle;
    use crate::issuance::Error::{IssueActionAlreadyFinalized, IssueActionNotFound};
    use crate::keys::{
        FullViewingKey, IssuerAuthorizingKey, IssuerValidatingKey, Scope, SpendingKey,
    };
    use crate::value::NoteValue;
    use crate::Address;
    use rand::rngs::OsRng;
    use rand::RngCore;

    fn setup_keys() -> (OsRng, IssuerAuthorizingKey, IssuerValidatingKey, Address) {
        let mut rng = OsRng;
        let sk = SpendingKey::random(&mut rng);
        let isk: IssuerAuthorizingKey = (&sk).into();
        let ik: IssuerValidatingKey = (&isk).into();

        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);
        (rng, isk, ik, recipient)
    }

    #[test]
    fn issue_bundle_basic() {
        let (rng, _, ik, recipient) = setup_keys();

        let mut bundle = IssueBundle::new();

        let str = String::from("Halo");
        let str2 = String::from("Halo2");

        let note_type = bundle
            .add_recipient(
                ik.clone(),
                str.clone(),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let another_note_type = bundle
            .add_recipient(
                ik.clone(),
                str,
                recipient,
                NoteValue::from_raw(10),
                false,
                rng,
            )
            .unwrap();
        assert_eq!(note_type, another_note_type);

        let third_note_type = bundle
            .add_recipient(
                ik.clone(),
                str2.clone(),
                recipient,
                NoteValue::from_raw(15),
                false,
                rng,
            )
            .unwrap();
        assert_ne!(note_type, third_note_type);

        let actions = bundle.actions();
        assert_eq!(actions.len(), 2);

        let action = bundle.get_action_by_type(note_type).unwrap();
        assert_eq!(action.notes.len(), 2);
        assert_eq!(action.notes.first().value().inner(), 5);
        assert_eq!(action.notes.first().note_type(), note_type);
        assert_eq!(action.notes.first().recipient(), recipient);

        assert_eq!(action.notes.tail().first().unwrap().value().inner(), 10);
        assert_eq!(action.notes.tail().first().unwrap().note_type(), note_type);
        assert_eq!(action.notes.tail().first().unwrap().recipient(), recipient);

        let action2 = bundle.get_action(ik, str2).unwrap();
        assert_eq!(action2.notes.len(), 1);
        assert_eq!(action2.notes().first().value().inner(), 15);
        assert_eq!(action2.notes().first().note_type(), third_note_type);
    }

    #[test]
    fn issue_bundle_finalize_asset() {
        let (rng, _, ik, recipient) = setup_keys();

        let mut bundle = IssueBundle::new();

        bundle
            .add_recipient(
                ik.clone(),
                String::from("Precious NFT"),
                recipient,
                NoteValue::from_raw(u64::MIN),
                false,
                rng,
            )
            .expect("Should properly add recipient");

        bundle
            .finalize_action(ik.clone(), String::from("Precious NFT"))
            .expect("Should finalize properly");

        assert_eq!(
            bundle
                .add_recipient(
                    ik.clone(),
                    String::from("Precious NFT"),
                    recipient,
                    NoteValue::from_raw(u64::MIN), //todo: create NoteValue wrapper
                    false,
                    rng,
                )
                .unwrap_err(),
            IssueActionAlreadyFinalized
        );

        assert_eq!(
            bundle
                .finalize_action(ik.clone(), String::from("Another precious NFT"))
                .unwrap_err(),
            IssueActionNotFound
        );

        bundle
            .add_recipient(
                ik.clone(),
                String::from("Another precious NFT"),
                recipient,
                NoteValue::from_raw(u64::MIN),
                true,
                rng,
            )
            .expect("should add and finalize");

        assert_eq!(
            bundle
                .add_recipient(
                    ik,
                    String::from("Another precious NFT"),
                    recipient,
                    NoteValue::from_raw(u64::MIN),
                    true,
                    rng,
                )
                .unwrap_err(),
            IssueActionAlreadyFinalized
        );
    }

    #[test]
    fn issue_bundle_prepare() {
        let (mut rng, _, ik, recipient) = setup_keys();

        let mut bundle = IssueBundle::new();

        bundle
            .add_recipient(
                ik,
                String::from("Frost"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let mut fake_sighash = [0; 32];
        rng.fill_bytes(&mut fake_sighash);

        let prepared = bundle.prepare(fake_sighash);
        assert_eq!(prepared.authorization().sighash, fake_sighash);
    }

    #[test]
    fn issue_bundle_sign() {
        let (mut rng, isk, ik, recipient) = setup_keys();

        let mut bundle = IssueBundle::new();

        bundle
            .add_recipient(
                ik.clone(),
                String::from("Frost"),
                recipient,
                NoteValue::from_raw(5),
                false,
                rng,
            )
            .unwrap();

        let mut rnd_sighash = [0; 32];
        rng.fill_bytes(&mut rnd_sighash);

        let signed = bundle.prepare(rnd_sighash).sign(rng, &isk);

        ik.verify(&rnd_sighash, &signed.authorization.signature)
            .expect("signature should be valid");
    }
}
