use incrementalmerkletree::{bridgetree::BridgeTree, Frontier, Hashable, Tree};
use orchard::{
    builder::Builder,
    bundle::{Authorized, Flags},
    circuit::{ProvingKey, VerifyingKey},
    keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
    note::ExtractedNoteCommitment,
    note_encryption::OrchardDomain,
    tree::{MerkleHashOrchard, MerklePath},
    value::NoteValue,
    Bundle,
};
use rand::rngs::OsRng;
use zcash_note_encryption::try_note_decryption;

fn verify_bundle(bundle: &Bundle<Authorized, i64>, vk: &VerifyingKey) {
    assert!(matches!(bundle.verify_proof(vk), Ok(())));
    let sighash: [u8; 32] = bundle.commitment().into();
    let bvk = bundle.binding_validating_key();
    for action in bundle.actions() {
        assert_eq!(action.rk().verify(&sighash, action.authorization()), Ok(()));
    }
    assert_eq!(
        bvk.verify(&sighash, bundle.authorization().binding_signature()),
        Ok(())
    );
}

#[test]
fn bundle_chain() {
    let mut rng = OsRng;
    let pk = ProvingKey::build();
    let vk = VerifyingKey::build();

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    // Create a shielding bundle.
    let shielding_bundle: Bundle<_, i64> = {
        // Use the empty tree.
        let anchor = MerkleHashOrchard::empty_root(32.into()).into();

        let mut builder = Builder::new(Flags::from_parts(false, true), anchor);
        assert_eq!(
            builder.add_recipient(None, recipient, NoteValue::from_raw(5000), None),
            Ok(())
        );
        let unauthorized = builder.build(&mut rng).unwrap();
        let sighash = unauthorized.commitment().into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        proven.apply_signatures(&mut rng, sighash, &[]).unwrap()
    };

    // Verify the shielding bundle.
    verify_bundle(&shielding_bundle, &vk);

    // Create a shielded bundle spending the previous output.
    let shielded_bundle: Bundle<_, i64> = {
        let ivk = fvk.to_ivk(Scope::External);
        let (note, _, _) = shielding_bundle
            .actions()
            .iter()
            .find_map(|action| {
                let domain = OrchardDomain::for_action(action);
                try_note_decryption(&domain, &ivk, action)
            })
            .unwrap();

        // Use the tree with a single leaf.
        let cmx: ExtractedNoteCommitment = note.commitment().into();
        let leaf = MerkleHashOrchard::from_cmx(&cmx);
        let mut tree = BridgeTree::<MerkleHashOrchard, 32>::new(0);
        tree.append(&leaf);
        let position = tree.witness().unwrap();
        let auth_path = tree.authentication_path(position).unwrap();
        let merkle_path = MerklePath::from_parts(
            u64::from(position).try_into().unwrap(),
            auth_path[..].try_into().unwrap(),
        );
        let anchor = tree.root().into();
        assert_eq!(anchor, merkle_path.root(cmx));

        let mut builder = Builder::new(Flags::from_parts(true, true), anchor);
        assert_eq!(builder.add_spend(fvk, note, merkle_path), Ok(()));
        assert_eq!(
            builder.add_recipient(None, recipient, NoteValue::from_raw(5000), None),
            Ok(())
        );
        let unauthorized = builder.build(&mut rng).unwrap();
        let sighash = unauthorized.commitment().into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        proven
            .apply_signatures(&mut rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };

    // Verify the shielded bundle.
    verify_bundle(&shielded_bundle, &vk);
}
