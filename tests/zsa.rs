mod builder;

use crate::builder::verify_bundle;
use incrementalmerkletree::bridgetree::BridgeTree;
use incrementalmerkletree::{Hashable, Tree};
use orchard::bundle::Authorized;
use orchard::issuance::{verify_issue_bundle, IssueBundle, Signed, Unauthorized};
use orchard::keys::{IssuanceAuthorizingKey, IssuanceValidatingKey};
use orchard::note::{AssetId, ExtractedNoteCommitment};
use orchard::note_encryption_v3::OrchardDomainV3;
use orchard::tree::{MerkleHashOrchard, MerklePath};
use orchard::{
    builder::Builder,
    bundle::Flags,
    circuit::{ProvingKey, VerifyingKey},
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
    value::NoteValue,
    Address, Anchor, Bundle, Note,
};
use rand::rngs::OsRng;
use std::collections::HashSet;
use zcash_note_encryption::try_note_decryption;

#[derive(Debug)]
struct Keychain {
    pk: ProvingKey,
    vk: VerifyingKey,
    sk: SpendingKey,
    fvk: FullViewingKey,
    isk: IssuanceAuthorizingKey,
    ik: IssuanceValidatingKey,
    recipient: Address,
}

impl Keychain {
    fn pk(&self) -> &ProvingKey {
        &self.pk
    }
    fn sk(&self) -> &SpendingKey {
        &self.sk
    }
    fn fvk(&self) -> &FullViewingKey {
        &self.fvk
    }
    fn isk(&self) -> &IssuanceAuthorizingKey {
        &self.isk
    }
    fn ik(&self) -> &IssuanceValidatingKey {
        &self.ik
    }
}

fn prepare_keys() -> Keychain {
    let pk = ProvingKey::build();
    let vk = VerifyingKey::build();

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let isk = IssuanceAuthorizingKey::from(&sk);
    let ik = IssuanceValidatingKey::from(&isk);
    Keychain {
        pk,
        vk,
        sk,
        fvk,
        isk,
        ik,
        recipient,
    }
}

fn sign_issue_bundle(
    unauthorized: IssueBundle<Unauthorized>,
    mut rng: OsRng,
    isk: &IssuanceAuthorizingKey,
) -> IssueBundle<Signed> {
    let sighash = unauthorized.commitment().into();
    let proven = unauthorized.prepare(sighash);
    proven.sign(&mut rng, isk).unwrap()
}

fn build_and_sign_bundle(
    builder: Builder,
    mut rng: OsRng,
    pk: &ProvingKey,
    sk: &SpendingKey,
) -> Bundle<Authorized, i64> {
    let unauthorized = builder.build(&mut rng).unwrap();
    let sighash = unauthorized.commitment().into();
    let proven = unauthorized.create_proof(pk, &mut rng).unwrap();
    proven
        .apply_signatures(&mut rng, sighash, &[SpendAuthorizingKey::from(sk)])
        .unwrap()
}

pub fn build_merkle_path_with_two_leaves(
    note1: &Note,
    note2: &Note,
) -> (MerklePath, MerklePath, Anchor) {
    let mut tree = BridgeTree::<MerkleHashOrchard, 32>::new(0);

    // Add first leaf
    let cmx1: ExtractedNoteCommitment = note1.commitment().into();
    let leaf1 = MerkleHashOrchard::from_cmx(&cmx1);
    tree.append(&leaf1);
    let position1 = tree.witness().unwrap();

    // Add second leaf
    let cmx2: ExtractedNoteCommitment = note2.commitment().into();
    let leaf2 = MerkleHashOrchard::from_cmx(&cmx2);
    tree.append(&leaf2);
    let position2 = tree.witness().unwrap();

    let root = tree.root(0).unwrap();
    let anchor = root.into();

    // Calculate first path
    let auth_path1 = tree.authentication_path(position1, &root).unwrap();
    let merkle_path1 = MerklePath::from_parts(
        u64::from(position1).try_into().unwrap(),
        auth_path1[..].try_into().unwrap(),
    );

    // Calculate second path
    let auth_path2 = tree.authentication_path(position2, &root).unwrap();
    let merkle_path2 = MerklePath::from_parts(
        u64::from(position2).try_into().unwrap(),
        auth_path2[..].try_into().unwrap(),
    );

    assert_eq!(anchor, merkle_path1.root(cmx1));
    assert_eq!(anchor, merkle_path2.root(cmx2));
    (merkle_path1, merkle_path2, anchor)
}

fn issue_zsa_notes(asset_descr: &str, keys: &Keychain) -> (Note, Note) {
    let mut rng = OsRng;
    // Create a issuance bundle
    let mut unauthorized = IssueBundle::new(keys.ik().clone());

    assert!(unauthorized
        .add_recipient(
            asset_descr.to_string(),
            keys.recipient,
            NoteValue::from_raw(40),
            false,
            &mut rng,
        )
        .is_ok());
    assert!(unauthorized
        .add_recipient(
            asset_descr.to_string(),
            keys.recipient,
            NoteValue::from_raw(2),
            false,
            &mut rng,
        )
        .is_ok());

    let issue_bundle = sign_issue_bundle(unauthorized, rng, keys.isk());

    // Take notes from first action
    let notes = issue_bundle.get_all_notes();
    let note1 = notes.get(0).unwrap();
    let note2 = notes.get(1).unwrap();

    assert!(verify_issue_bundle(
        &issue_bundle,
        issue_bundle.commitment().into(),
        &mut HashSet::new(),
    )
    .is_ok());

    (*note1, *note2)
}

fn create_native_note(keys: &Keychain) -> Note {
    let mut rng = OsRng;

    let shielding_bundle: Bundle<_, i64> = {
        // Use the empty tree.
        let anchor = MerkleHashOrchard::empty_root(32.into()).into();

        let mut builder = Builder::new(Flags::from_parts(false, true), anchor);
        assert_eq!(
            builder.add_recipient(
                None,
                keys.recipient,
                NoteValue::from_raw(100),
                AssetId::native(),
                None
            ),
            Ok(())
        );
        let unauthorized = builder.build(&mut rng).unwrap();
        let sighash = unauthorized.commitment().into();
        let proven = unauthorized.create_proof(keys.pk(), &mut rng).unwrap();
        proven.apply_signatures(&mut rng, sighash, &[]).unwrap()
    };
    let ivk = keys.fvk().to_ivk(Scope::External);
    let (native_note, _, _) = shielding_bundle
        .actions()
        .iter()
        .find_map(|action| {
            let domain = OrchardDomainV3::for_action(action);
            try_note_decryption(&domain, &PreparedIncomingViewingKey::new(&ivk), action)
        })
        .unwrap();

    native_note
}

struct TestSpendInfo {
    note: Note,
    merkle_path: MerklePath,
}

impl TestSpendInfo {
    fn merkle_path(&self) -> &MerklePath {
        &self.merkle_path
    }
}

struct TestOutputInfo {
    value: NoteValue,
    asset: AssetId,
}

fn build_and_verify_bundle(
    spends: Vec<&TestSpendInfo>,
    outputs: Vec<TestOutputInfo>,
    assets_to_burn: Vec<(AssetId, NoteValue)>,
    anchor: Anchor,
    expected_num_actions: usize,
    keys: &Keychain,
) -> Result<(), &'static str> {
    let rng = OsRng;
    let shielded_bundle: Bundle<_, i64> = {
        let mut builder = Builder::new(Flags::from_parts(true, true), anchor);

        spends.iter().try_for_each(|spend| {
            builder.add_spend(keys.fvk().clone(), spend.note, spend.merkle_path().clone())
        })?;
        outputs.iter().try_for_each(|output| {
            builder.add_recipient(None, keys.recipient, output.value, output.asset, None)
        })?;
        assets_to_burn
            .into_iter()
            .try_for_each(|(asset, value)| builder.add_burn(asset, value))?;
        build_and_sign_bundle(builder, rng, keys.pk(), keys.sk())
    };

    // Verify the shielded bundle, currently without the proof.
    verify_bundle(&shielded_bundle, &keys.vk, false);
    assert_eq!(shielded_bundle.actions().len(), expected_num_actions);
    Ok(())
}

/// Issue several ZSA and native notes and spend them in different combinations, e.g. split and join
#[test]
fn zsa_issue_and_transfer() {
    // --------------------------- Setup -----------------------------------------

    let keys = prepare_keys();
    let asset_descr = "zsa_asset";

    // Prepare ZSA
    let (zsa_note_1, zsa_note_2) = issue_zsa_notes(asset_descr, &keys);

    let (merkle_path1, merkle_path2, anchor) =
        build_merkle_path_with_two_leaves(&zsa_note_1, &zsa_note_2);

    let zsa_spend_1 = TestSpendInfo {
        note: zsa_note_1,
        merkle_path: merkle_path1,
    };
    let zsa_spend_2 = TestSpendInfo {
        note: zsa_note_2,
        merkle_path: merkle_path2,
    };

    let native_note = create_native_note(&keys);
    let (native_merkle_path_1, native_merkle_path_2, native_anchor) =
        build_merkle_path_with_two_leaves(&native_note, &zsa_note_1);
    let native_spend: TestSpendInfo = TestSpendInfo {
        note: native_note,
        merkle_path: native_merkle_path_1,
    };
    let zsa_spend_with_native: TestSpendInfo = TestSpendInfo {
        note: zsa_note_1,
        merkle_path: native_merkle_path_2,
    };

    // --------------------------- Tests -----------------------------------------

    // 1. Spend single ZSA note
    build_and_verify_bundle(
        vec![&zsa_spend_1],
        vec![TestOutputInfo {
            value: zsa_spend_1.note.value(),
            asset: zsa_spend_1.note.asset(),
        }],
        vec![],
        anchor,
        2,
        &keys,
    )
    .unwrap();

    // 2. Split single ZSA note into 2 notes
    let delta = 2; // arbitrary number for value manipulation
    build_and_verify_bundle(
        vec![&zsa_spend_1],
        vec![
            TestOutputInfo {
                value: NoteValue::from_raw(zsa_spend_1.note.value().inner() - delta),
                asset: zsa_spend_1.note.asset(),
            },
            TestOutputInfo {
                value: NoteValue::from_raw(delta),
                asset: zsa_spend_1.note.asset(),
            },
        ],
        vec![],
        anchor,
        2,
        &keys,
    )
    .unwrap();

    // 3. Join 2 ZSA notes into a single note
    build_and_verify_bundle(
        vec![&zsa_spend_1, &zsa_spend_2],
        vec![TestOutputInfo {
            value: NoteValue::from_raw(
                zsa_spend_1.note.value().inner() + zsa_spend_2.note.value().inner(),
            ),
            asset: zsa_spend_1.note.asset(),
        }],
        vec![],
        anchor,
        2,
        &keys,
    )
    .unwrap();

    // 4. Take 2 ZSA notes and send them as 2 notes with different denomination
    build_and_verify_bundle(
        vec![&zsa_spend_1, &zsa_spend_2],
        vec![
            TestOutputInfo {
                value: NoteValue::from_raw(zsa_spend_1.note.value().inner() - delta),
                asset: zsa_spend_1.note.asset(),
            },
            TestOutputInfo {
                value: NoteValue::from_raw(zsa_spend_2.note.value().inner() + delta),
                asset: zsa_spend_2.note.asset(),
            },
        ],
        vec![],
        anchor,
        2,
        &keys,
    )
    .unwrap();

    // 5. Spend single ZSA note, mixed with native note (shielding)
    build_and_verify_bundle(
        vec![&zsa_spend_1],
        vec![
            TestOutputInfo {
                value: zsa_spend_1.note.value(),
                asset: zsa_spend_1.note.asset(),
            },
            TestOutputInfo {
                value: NoteValue::from_raw(100),
                asset: AssetId::native(),
            },
        ],
        vec![],
        anchor,
        4,
        &keys,
    )
    .unwrap();

    // 6. Spend single ZSA note, mixed with native note (shielded to shielded)
    build_and_verify_bundle(
        vec![&zsa_spend_with_native, &native_spend],
        vec![
            TestOutputInfo {
                value: zsa_spend_1.note.value(),
                asset: zsa_spend_1.note.asset(),
            },
            TestOutputInfo {
                value: native_spend.note.value(),
                asset: AssetId::native(),
            },
        ],
        vec![],
        native_anchor,
        4,
        &keys,
    )
    .unwrap();

    // 7. Spend ZSA notes of different asset types
    let (zsa_note_t7, _) = issue_zsa_notes("zsa_asset2", &keys);
    let (merkle_path_t7_1, merkle_path_t7_2, anchor_t7) =
        build_merkle_path_with_two_leaves(&zsa_note_t7, &zsa_note_2);
    let zsa_spend_t7_1: TestSpendInfo = TestSpendInfo {
        note: zsa_note_t7,
        merkle_path: merkle_path_t7_1,
    };
    let zsa_spend_t7_2: TestSpendInfo = TestSpendInfo {
        note: zsa_note_2,
        merkle_path: merkle_path_t7_2,
    };

    build_and_verify_bundle(
        vec![&zsa_spend_t7_1, &zsa_spend_t7_2],
        vec![
            TestOutputInfo {
                value: zsa_spend_t7_1.note.value(),
                asset: zsa_spend_t7_1.note.asset(),
            },
            TestOutputInfo {
                value: zsa_spend_t7_2.note.value(),
                asset: zsa_spend_t7_2.note.asset(),
            },
        ],
        vec![],
        anchor_t7,
        4,
        &keys,
    )
    .unwrap();

    // 8. Same but wrong denomination
    let result = std::panic::catch_unwind(|| {
        build_and_verify_bundle(
            vec![&zsa_spend_t7_1, &zsa_spend_t7_2],
            vec![
                TestOutputInfo {
                    value: NoteValue::from_raw(zsa_spend_t7_1.note.value().inner() + delta),
                    asset: zsa_spend_t7_1.note.asset(),
                },
                TestOutputInfo {
                    value: NoteValue::from_raw(zsa_spend_t7_2.note.value().inner() - delta),
                    asset: zsa_spend_t7_2.note.asset(),
                },
            ],
            vec![],
            anchor_t7,
            4,
            &keys,
        )
        .unwrap();
    });
    assert!(result.is_err());

    // 9. Burn ZSA assets
    build_and_verify_bundle(
        vec![&zsa_spend_1],
        vec![],
        vec![(zsa_spend_1.note.asset(), zsa_spend_1.note.value())],
        anchor,
        2,
        &keys,
    )
    .unwrap();

    // 10. Burn a partial amount of the ZSA assets
    let value_to_burn = 3;
    let value_to_transfer = zsa_spend_1.note.value().inner() - value_to_burn;

    build_and_verify_bundle(
        vec![&zsa_spend_1],
        vec![TestOutputInfo {
            value: NoteValue::from_raw(value_to_transfer),
            asset: zsa_spend_1.note.asset(),
        }],
        vec![(zsa_spend_1.note.asset(), NoteValue::from_raw(value_to_burn))],
        anchor,
        2,
        &keys,
    )
    .unwrap();

    // 11. Try to burn native asset - should fail
    let result = build_and_verify_bundle(
        vec![&native_spend],
        vec![],
        vec![(AssetId::native(), native_spend.note.value())],
        native_anchor,
        2,
        &keys,
    );
    match result {
        Ok(_) => panic!("Test should fail"),
        Err(error) => assert_eq!(error, "Burning is only possible for non-native assets"),
    }
}
