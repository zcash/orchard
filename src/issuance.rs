//! Structs related to issuance bundles and the associated logic.

use memuse::DynamicUsage;
use nonempty::NonEmpty;

use crate::bundle::Authorization;
use crate::{primitives::redpallas::{self, SpendAuth},
    Note,
};

impl<T> IssueAction<T> {
    /// Constructs an `IssueAction` from its constituent parts.
    pub fn from_parts(
        ik: redpallas::VerificationKey<SpendAuth>,
        asset_desc: Vec<u8>,
        notes: NonEmpty<Note>,
        finalize: bool,
        authorization: T,
    ) -> Self {
        IssueAction {
            ik,
            asset_desc,
            notes,
            finalize,
            authorization,
        }
    }

    /// Returns the issuer verification key for the note being created.
    pub fn ik(&self) -> &redpallas::VerificationKey<SpendAuth> {
        &self.ik
    }

    /// Returns the asset description for the note being created.
    pub fn asset_desc(&self) -> &[u8] {
        &self.asset_desc
    }

    /// Returns the issued notes.
    pub fn notes(&self) -> &NonEmpty<Note> {
        &self.notes
    }

    /// Returns the authorization for this action.
    pub fn authorization(&self) -> &T {
        &self.authorization
    }

    /// Returns whether the asset type was finalized in this action.
    pub fn is_finalized(&self) -> bool {
        self.finalize
    }

    /// Transitions this issue action from one authorization state to another.
    pub fn map<U>(self, step: impl FnOnce(T) -> U) -> IssueAction<U> {
        IssueAction {
            ik: self.ik,
            asset_desc: self.asset_desc,
            notes: self.notes,
            finalize: self.finalize,
            authorization: step(self.authorization),
        }
    }

    /// Transitions this issue action from one authorization state to another.
    pub fn try_map<U, E>(self, step: impl FnOnce(T) -> Result<U, E>) -> Result<IssueAction<U>, E> {
        Ok(IssueAction {
            ik: self.ik,
            asset_desc: self.asset_desc,
            notes: self.notes,
            finalize: self.finalize,
            authorization: step(self.authorization)?,
        })
    }
}

impl DynamicUsage for IssueAction<redpallas::Signature<SpendAuth>> {
    #[inline(always)]
    fn dynamic_usage(&self) -> usize {
        0
    }

    #[inline(always)]
    fn dynamic_usage_bounds(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub(crate) mod testing {
    use nonempty::NonEmpty;
    use proptest::prelude::*;
    use rand::{rngs::StdRng, SeedableRng};
    use reddsa::orchard::SpendAuth;

    use crate::{
        note::testing::arb_note,
        primitives::redpallas::{
            self,
            testing::{arb_spendauth_signing_key, arb_spendauth_verification_key},
        },
        value::NoteValue,
    };

    use super::IssueAction;

    prop_compose! {
        /// Generate an issue action with a single note and without authorization data.
        pub fn arb_unauthorized_issue_action(output_value: NoteValue)(
            ik in arb_spendauth_verification_key(),
            asset_desc in prop::collection::vec(any::<u8>(), 0..=255),
            note in arb_note(output_value),
        ) -> IssueAction<()> {
            IssueAction {
                ik,
                asset_desc,
                notes:NonEmpty::new(note), //todo: replace note type
                finalize: false,
                authorization: (),
            }
        }
    }

    prop_compose! {
        /// Generate an issue action with invalid (random) authorization data.
        pub fn arb_issue_action(output_value: NoteValue)(
            sk in arb_spendauth_signing_key(),
            asset_desc in prop::collection::vec(any::<u8>(), 0..=255),
            note in arb_note(output_value),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY),
            fake_sighash in prop::array::uniform32(prop::num::u8::ANY),
        ) -> IssueAction<redpallas::Signature<SpendAuth>> {

            let rng = StdRng::from_seed(rng_seed);

            IssueAction {
                ik: redpallas::VerificationKey::from(&sk),
                asset_desc,
                notes:NonEmpty::new(note), //todo: replace note type
                finalize: false,
                authorization: sk.sign(rng, &fake_sighash),
            }
        }
    }
}

/// An issue action applied to the global ledger.
///
/// Externally, this creates new zsa notes (adding a commitment to the global ledger).
#[derive(Debug, Clone)]
pub struct IssueAction<A> {
    /// The issuer key for the note being created.
    ik: redpallas::VerificationKey<SpendAuth>,
    /// Asset description for verification.
    asset_desc: Vec<u8>,
    /// The newly issued notes.
    notes: NonEmpty<Note>,
    /// Finalize will prevent further issuance of the same asset.
    finalize: bool,
    /// The authorization for this action.
    authorization: A,
}

/// A bundle of actions to be applied to the ledger.
#[derive(Debug, Clone)]
pub struct IssueBundle<T: Authorization> {
    /// The list of issue actions that make up this bundle.
    actions: NonEmpty<IssueAction<T::SpendAuth>>,
    /// The authorization for this bundle.
    authorization: T,
}

// mod tests {
//     use rand::rngs::OsRng;
//
//     use super::Builder;
//     // use crate::keys::{IssuerAuthorizingKey, IssuerValidatingKey};
//     use crate::note::NoteType;
//     use crate::{
//         bundle::{Authorized, Bundle, Flags},
//         circuit::ProvingKey,
//         constants::MERKLE_DEPTH_ORCHARD,
//         keys::{FullViewingKey, Scope, SpendingKey},
//         tree::EMPTY_ROOTS,
//         value::NoteValue,
//     };
//     use crate::builder::Builder;
//
//     #[test]
//     fn shielding_bundle() {
//         let pk = ProvingKey::build();
//         let mut rng = OsRng;
//
//         let sk = SpendingKey::random(&mut rng);
//         let fvk = FullViewingKey::from(&sk);
//         let recipient = fvk.address_at(0u32, Scope::External);
//
//         let mut builder = Builder::new(
//             Flags::from_parts(true, true),
//             EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
//         );
//
//         builder
//             .add_recipient(
//                 None,
//                 recipient,
//                 NoteValue::from_raw(5000),
//                 NoteType::native(),
//                 None,
//             )
//             .unwrap();
//
//
//         let bundle: Bundle<Authorized, i64> = builder
//             .build(&mut rng)
//             .unwrap()
//             .create_proof(&pk, &mut rng)
//             .unwrap()
//             .prepare(&mut rng, [0; 32])
//             .sign()
//             .finalize()
//             .unwrap();
//         assert_eq!(bundle.value_balance(), &(-5000));
//
//         verify_bundle(&bundle, &vk)
//     }
// }
