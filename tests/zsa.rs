mod builder;

use crate::builder::verify_bundle;
use incrementalmerkletree::bridgetree::BridgeTree;
use incrementalmerkletree::{Hashable, Tree};
use orchard::bundle::Authorized;
use orchard::issuance::{verify_issue_bundle, IssueBundle, Signed, Unauthorized};
use orchard::keys::{IssuerAuthorizingKey, IssuerValidatingKey};
use orchard::note::{ExtractedNoteCommitment, NoteType};
use orchard::note_encryption::OrchardDomain;
use orchard::tree::{MerkleHashOrchard, MerklePath};
use orchard::{
    builder::Builder,
    bundle::Flags,
    circuit::{ProvingKey, VerifyingKey},
    keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
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
    isk: IssuerAuthorizingKey,
    ik: IssuerValidatingKey,
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
    fn isk(&self) -> &IssuerAuthorizingKey {
        &self.isk
    }
    fn ik(&self) -> &IssuerValidatingKey {
        &self.ik
    }
}

fn prepare_keys() -> Keychain {
    let pk = ProvingKey::build();
    let vk = VerifyingKey::build();

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let isk = IssuerAuthorizingKey::from(&sk);
    let ik = IssuerValidatingKey::from(&isk);
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
    isk: IssuerAuthorizingKey,
) -> IssueBundle<Signed> {
    let sighash = unauthorized.commitment().into();
    let proven = unauthorized.prepare(sighash);
    proven.sign(&mut rng, &isk).unwrap()
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

    let issue_bundle = sign_issue_bundle(unauthorized, rng, keys.isk().clone());

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
                NoteType::native(),
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
            let domain = OrchardDomain::for_action(action);
            try_note_decryption(&domain, &ivk, action)
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
    note_type: NoteType,
}

fn build_and_verify_bundle(
    spends: Vec<&TestSpendInfo>,
    outputs: Vec<TestOutputInfo>,
    anchor: Anchor,
    expected_num_actions: usize,
    keys: &Keychain,
) {
    let rng = OsRng;
    let shielded_bundle: Bundle<_, i64> = {
        let mut builder = Builder::new(Flags::from_parts(true, true), anchor);

        spends.iter().for_each(|spend| {
            assert_eq!(
                builder.add_spend(keys.fvk().clone(), spend.note, spend.merkle_path().clone()),
                Ok(())
            );
        });
        outputs.iter().for_each(|output| {
            assert_eq!(
                builder.add_recipient(None, keys.recipient, output.value, output.note_type, None),
                Ok(())
            )
        });
        build_and_sign_bundle(builder, rng, keys.pk(), keys.sk())
    };

    // Verify the shielded bundle, currently without the proof.
    verify_bundle(&shielded_bundle, &keys.vk, false);
    assert_eq!(shielded_bundle.actions().len(), expected_num_actions);
}

/// Issue several ZSA and native notes and spend them in different combinations, e.g. split and join
#[test]
fn zsa_issue_and_transfer() {
    // --------------------------- Setup -----------------------------------------

    let keys = prepare_keys();
    let asset_descr = "zsa_asset";

    // Prepare ZSA
    let (zsa_note1, zsa_note2) = issue_zsa_notes(asset_descr, &keys);

    let (merkle_path1, merkle_path2, anchor) =
        build_merkle_path_with_two_leaves(&zsa_note1, &zsa_note2);

    let zsa_spend_1 = TestSpendInfo {
        note: zsa_note1,
        merkle_path: merkle_path1,
    };
    let zsa_spend_2 = TestSpendInfo {
        note: zsa_note2,
        merkle_path: merkle_path2,
    };

    // --------------------------- Tests -----------------------------------------

    // 1. Spend single ZSA note
    build_and_verify_bundle(
        vec![&zsa_spend_1],
        vec![TestOutputInfo {
            value: zsa_spend_1.note.value(),
            note_type: zsa_spend_1.note.note_type(),
        }],
        anchor,
        2,
        &keys,
    );

    // 2. Split single ZSA note into 2 notes
    let delta = 2; // arbitrary number for value manipulation
    build_and_verify_bundle(
        vec![&zsa_spend_1],
        vec![
            TestOutputInfo {
                value: NoteValue::from_raw(zsa_spend_1.note.value().inner() - delta),
                note_type: zsa_spend_1.note.note_type(),
            },
            TestOutputInfo {
                value: NoteValue::from_raw(delta),
                note_type: zsa_spend_1.note.note_type(),
            },
        ],
        anchor,
        2,
        &keys,
    );

    // 3. Join 2 ZSA notes into a single note
    build_and_verify_bundle(
        vec![&zsa_spend_1, &zsa_spend_2],
        vec![TestOutputInfo {
            value: NoteValue::from_raw(
                zsa_spend_1.note.value().inner() + zsa_spend_2.note.value().inner(),
            ),
            note_type: zsa_spend_1.note.note_type(),
        }],
        anchor,
        2,
        &keys,
    );

    // 4. Take 2 ZSA notes and send them as 2 notes with different denomination
    build_and_verify_bundle(
        vec![&zsa_spend_1, &zsa_spend_2],
        vec![
            TestOutputInfo {
                value: NoteValue::from_raw(zsa_spend_1.note.value().inner() - delta),
                note_type: zsa_spend_1.note.note_type(),
            },
            TestOutputInfo {
                value: NoteValue::from_raw(zsa_spend_2.note.value().inner() + delta),
                note_type: zsa_spend_2.note.note_type(),
            },
        ],
        anchor,
        2,
        &keys,
    );

    // 5. Spend single ZSA note, mixed with native note (shielding)
    build_and_verify_bundle(
        vec![&zsa_spend_1],
        vec![
            TestOutputInfo {
                value: zsa_spend_1.note.value(),
                note_type: zsa_spend_1.note.note_type(),
            },
            TestOutputInfo {
                value: NoteValue::from_raw(100),
                note_type: NoteType::native(),
            },
        ],
        anchor,
        4,
        &keys,
    );

    // 6. Spend single ZSA note, mixed with native note (shielded to shielded)
    let native_note = create_native_note(&keys);
    let (native_merkle_path1, native_merkle_path2, native_anchor) =
        build_merkle_path_with_two_leaves(&native_note, &zsa_note1);
    let native_spend: TestSpendInfo = TestSpendInfo {
        note: native_note,
        merkle_path: native_merkle_path1,
    };
    let zsa_spend_with_native: TestSpendInfo = TestSpendInfo {
        note: zsa_note1,
        merkle_path: native_merkle_path2,
    };

    build_and_verify_bundle(
        vec![&zsa_spend_with_native, &native_spend],
        vec![
            TestOutputInfo {
                value: zsa_spend_1.note.value(),
                note_type: zsa_spend_1.note.note_type(),
            },
            TestOutputInfo {
                value: native_spend.note.value(),
                note_type: NoteType::native(),
            },
        ],
        native_anchor,
        4,
        &keys,
    );

    // 7. Spend ZSA notes of different asset types
    let (zsa_note_t7, _) = issue_zsa_notes("zsa_asset2", &keys);
    let (merkle_path_t7_1, merkle_path_t7_2, anchor_t7) =
        build_merkle_path_with_two_leaves(&zsa_note_t7, &zsa_note2);
    let zsa_spend_t7_1: TestSpendInfo = TestSpendInfo {
        note: zsa_note_t7,
        merkle_path: merkle_path_t7_1,
    };
    let zsa_spend_t7_2: TestSpendInfo = TestSpendInfo {
        note: zsa_note2,
        merkle_path: merkle_path_t7_2,
    };

    build_and_verify_bundle(
        vec![&zsa_spend_t7_1, &zsa_spend_t7_2],
        vec![
            TestOutputInfo {
                value: zsa_spend_t7_1.note.value(),
                note_type: zsa_spend_t7_1.note.note_type(),
            },
            TestOutputInfo {
                value: zsa_spend_t7_2.note.value(),
                note_type: zsa_spend_t7_2.note.note_type(),
            },
        ],
        anchor_t7,
        4,
        &keys,
    );

    // 8. Same but wrong denomination
    let result = std::panic::catch_unwind(|| {
        build_and_verify_bundle(
            vec![&zsa_spend_t7_1, &zsa_spend_t7_2],
            vec![
                TestOutputInfo {
                    value: NoteValue::from_raw(zsa_spend_t7_1.note.value().inner() + delta),
                    note_type: zsa_spend_t7_1.note.note_type(),
                },
                TestOutputInfo {
                    value: NoteValue::from_raw(zsa_spend_t7_2.note.value().inner() - delta),
                    note_type: zsa_spend_t7_2.note.note_type(),
                },
            ],
            anchor_t7,
            4,
            &keys,
        );
    });
    assert!(result.is_err());
}
