use std::collections::HashMap;

use rand::{rngs::OsRng, RngCore};

use orchard::{
    asset_record::AssetRecord,
    issuance::{
        verify_issue_bundle,
        Error::{
            IssueActionPreviouslyFinalizedAssetBase, MissingReferenceNoteOnFirstIssuance,
            ValueOverflow,
        },
        IssueBundle, IssueInfo, Signed,
    },
    keys::{FullViewingKey, IssuanceAuthorizingKey, IssuanceValidatingKey, Scope, SpendingKey},
    note::{AssetBase, Nullifier},
    value::NoteValue,
    Address, Note,
};

fn random_bytes<const N: usize>(mut rng: OsRng) -> [u8; N] {
    let mut bytes = [0; N];
    rng.fill_bytes(&mut bytes);
    bytes
}

#[derive(Clone)]
struct TestParams {
    rng: OsRng,
    isk: IssuanceAuthorizingKey,
    ik: IssuanceValidatingKey,
    recipient: Address,
    sighash: [u8; 32],
    first_nullifier: Nullifier,
}

// For testing global state only - should not be used in an actual setting.
fn setup_params() -> TestParams {
    use group::{ff::PrimeField, Curve, Group};
    use pasta_curves::{arithmetic::CurveAffine, pallas};

    let rng = OsRng;

    let isk = IssuanceAuthorizingKey::from_bytes(random_bytes(rng)).unwrap();
    let ik: IssuanceValidatingKey = (&isk).into();

    let fvk = FullViewingKey::from(&SpendingKey::from_bytes(random_bytes(rng)).unwrap());
    let recipient = fvk.address_at(0u32, Scope::External);

    let sighash = random_bytes(rng);

    // For testing purposes only: replicate the behavior of orchard's `Nullifier::dummy`
    // and `extract_p` functions, which are marked as `pub(crate)` in orchard and are therefore
    // not visible here.
    let first_nullifier = {
        let point = pallas::Point::random(rng);

        let base = point
            .to_affine()
            .coordinates()
            .map(|c| *c.x())
            .unwrap_or_else(pallas::Base::zero);

        Nullifier::from_bytes(&base.to_repr()).unwrap()
    };

    TestParams {
        rng,
        isk,
        ik,
        recipient,
        sighash,
        first_nullifier,
    }
}

fn build_state_entry(
    asset_base: &AssetBase,
    amount: u64,
    is_finalized: bool,
    reference_note: &Note,
) -> (AssetBase, AssetRecord) {
    (
        *asset_base,
        AssetRecord::new(NoteValue::from_raw(amount), is_finalized, *reference_note),
    )
}

#[derive(Clone)]
struct IssueTestNote {
    asset_desc: Vec<u8>,
    amount: u64,
    is_finalized: bool,
    first_issuance: bool,
}

impl IssueTestNote {
    fn new(asset_desc: &Vec<u8>, amount: u64, is_finalized: bool, first_issuance: bool) -> Self {
        Self {
            asset_desc: asset_desc.clone(),
            amount,
            is_finalized,
            first_issuance,
        }
    }
}

fn get_first_note(bundle: &IssueBundle<Signed>, action_index: usize) -> &Note {
    bundle.actions()[action_index].notes().first().unwrap()
}

fn build_issue_bundle(params: &TestParams, data: &[IssueTestNote]) -> IssueBundle<Signed> {
    let TestParams {
        rng,
        ref isk,
        ref ik,
        recipient,
        sighash,
        ref first_nullifier,
    } = *params;

    let IssueTestNote {
        asset_desc,
        amount,
        is_finalized,
        first_issuance,
    } = data.first().unwrap().clone();

    let (mut bundle, _) = IssueBundle::new(
        ik.clone(),
        asset_desc.clone(),
        Some(IssueInfo {
            recipient,
            value: NoteValue::from_raw(amount),
        }),
        first_issuance,
        rng,
    )
    .unwrap();

    if is_finalized {
        bundle.finalize_action(&asset_desc).unwrap();
    }

    for IssueTestNote {
        asset_desc,
        amount,
        is_finalized,
        first_issuance,
    } in data.into_iter().skip(1).cloned()
    {
        bundle
            .add_recipient(
                &asset_desc,
                recipient,
                NoteValue::from_raw(amount),
                first_issuance,
                rng,
            )
            .unwrap();

        if is_finalized {
            bundle.finalize_action(&asset_desc).unwrap();
        }
    }

    bundle
        .update_rho(&first_nullifier)
        .prepare(sighash)
        .sign(&isk)
        .unwrap()
}

// Issuance workflow test: performs a series of bundle creations and verifications,
// with a global state simulation
#[test]
fn issue_bundle_verify_with_global_state() {
    let params = setup_params();

    let TestParams { ik, sighash, .. } = params.clone();

    let asset1_desc = b"Verify with issued assets 1".to_vec();
    let asset2_desc = b"Verify with issued assets 2".to_vec();
    let asset3_desc = b"Verify with issued assets 3".to_vec();
    let asset4_desc = b"Verify with issued assets 4".to_vec();

    let asset1_base = AssetBase::derive(&ik, &asset1_desc);
    let asset2_base = AssetBase::derive(&ik, &asset2_desc);
    let asset3_base = AssetBase::derive(&ik, &asset3_desc);
    let asset4_base = AssetBase::derive(&ik, &asset4_desc);

    let mut global_state = HashMap::new();

    // We'll issue and verify a series of bundles. For valid bundles, the global
    // state is updated and must match the expected result. For invalid bundles,
    // we check the expected error, leaving the state unchanged.

    // ** Bundle1 (valid) **

    let bundle1 = build_issue_bundle(
        &params,
        &[
            IssueTestNote::new(&asset1_desc, 7, false, true),
            IssueTestNote::new(&asset1_desc, 8, false, false),
            IssueTestNote::new(&asset2_desc, 10, true, true),
            IssueTestNote::new(&asset3_desc, 5, false, true),
        ],
    );

    let expected_global_state1 = HashMap::from([
        build_state_entry(&asset1_base, 15, false, get_first_note(&bundle1, 0)),
        build_state_entry(&asset2_base, 10, true, get_first_note(&bundle1, 1)),
        build_state_entry(&asset3_base, 5, false, get_first_note(&bundle1, 2)),
    ]);

    global_state.extend(
        verify_issue_bundle(&bundle1, sighash, |asset| global_state.get(asset).cloned()).unwrap(),
    );
    assert_eq!(global_state, expected_global_state1);

    // ** Bundle2 (valid) **

    let bundle2 = build_issue_bundle(
        &params,
        &[
            IssueTestNote::new(&asset1_desc, 3, true, true),
            IssueTestNote::new(&asset3_desc, 20, false, false),
        ],
    );

    let expected_global_state2 = HashMap::from([
        build_state_entry(&asset1_base, 18, true, get_first_note(&bundle1, 0)),
        build_state_entry(&asset2_base, 10, true, get_first_note(&bundle1, 1)),
        build_state_entry(&asset3_base, 25, false, get_first_note(&bundle1, 2)),
    ]);

    global_state.extend(
        verify_issue_bundle(&bundle2, sighash, |asset| global_state.get(asset).cloned()).unwrap(),
    );
    assert_eq!(global_state, expected_global_state2);

    // ** Bundle3 (invalid) **

    let bundle3 = build_issue_bundle(
        &params,
        &[
            IssueTestNote::new(&asset1_desc, 3, false, true),
            IssueTestNote::new(&asset3_desc, 20, false, false),
        ],
    );

    let expected_global_state3 = expected_global_state2;

    assert_eq!(
        verify_issue_bundle(&bundle3, sighash, |asset| global_state.get(asset).cloned())
            .unwrap_err(),
        IssueActionPreviouslyFinalizedAssetBase,
    );
    assert_eq!(global_state, expected_global_state3);

    // ** Bundle4 (invalid) **

    let bundle4 = build_issue_bundle(
        &params,
        &[
            IssueTestNote::new(&asset3_desc, 50, true, true),
            IssueTestNote::new(&asset4_desc, 77, false, false),
        ],
    );

    let expected_global_state4 = expected_global_state3;

    assert_eq!(
        verify_issue_bundle(&bundle4, sighash, |asset| global_state.get(asset).cloned())
            .unwrap_err(),
        MissingReferenceNoteOnFirstIssuance,
    );
    assert_eq!(global_state, expected_global_state4);

    // ** Bundle5 (invalid) **

    let bundle5 = build_issue_bundle(
        &params,
        &[
            IssueTestNote::new(&asset3_desc, u64::MAX - 20, true, true),
            IssueTestNote::new(&asset4_desc, 77, false, true),
        ],
    );

    let expected_global_state5 = expected_global_state4;

    assert_eq!(
        verify_issue_bundle(&bundle5, sighash, |asset| global_state.get(asset).cloned())
            .unwrap_err(),
        ValueOverflow,
    );
    assert_eq!(global_state, expected_global_state5);

    // ** Bundle6 (valid) **

    let bundle6 = build_issue_bundle(
        &params,
        &[
            IssueTestNote::new(&asset3_desc, 50, true, true),
            IssueTestNote::new(&asset4_desc, 77, false, true),
        ],
    );

    let expected_global_state6 = HashMap::from([
        build_state_entry(&asset1_base, 18, true, get_first_note(&bundle1, 0)),
        build_state_entry(&asset2_base, 10, true, get_first_note(&bundle1, 1)),
        build_state_entry(&asset3_base, 75, true, get_first_note(&bundle1, 2)),
        build_state_entry(&asset4_base, 77, false, get_first_note(&bundle6, 1)),
    ]);

    global_state.extend(
        verify_issue_bundle(&bundle6, sighash, |asset| global_state.get(asset).cloned()).unwrap(),
    );
    assert_eq!(global_state, expected_global_state6);
}
