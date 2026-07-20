#![cfg(feature = "circuit")]

use incrementalmerkletree::{Hashable, Marking, Retention};
use orchard::{
    builder::{Builder, BundleType},
    bundle::{Authorized, BatchValidator, BundleVersion, Flags, TxVersion},
    circuit::{OrchardCircuitVersion, ProvingKey, VerifyingKey},
    flavor::{OrchardFlavor, OrchardVanilla, OrchardZSA},
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
    note::{AssetBase, ExtractedNoteCommitment, NoteVersion},
    note_encryption::{
        DomainVersion, IronwoodDomain, NoteEncryptionDomain, OrchardDomain, OrchardVersion,
        ZSAVersion,
    },
    primitives::OrchardPrimitives,
    sighash_kind::OrchardSighashKind,
    tree::{MerkleHashOrchard, MerklePath},
    value::NoteValue,
    Address, Anchor, Bundle, Note,
};
use rand::rngs::OsRng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use shardtree::{store::memory::MemoryShardStore, ShardTree};
use zcash_note_encryption::try_note_decryption;

/// Builds a single-leaf note commitment tree containing `cmx`, returning the tree
/// root and a witness for the leaf.
fn single_leaf_witness(cmx: &ExtractedNoteCommitment) -> (MerkleHashOrchard, MerklePath) {
    let leaf = MerkleHashOrchard::from_cmx(cmx);
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
    assert_eq!(root, merkle_path.root(leaf));
    (root, merkle_path.into())
}

pub fn verify_bundle<Pr: OrchardPrimitives>(
    bundle: &Bundle<Authorized, i64, Pr>,
    vk: &VerifyingKey,
    tx_version: TxVersion,
    verify_proof: bool,
) {
    if verify_proof {
        assert!(matches!(bundle.verify_proof(vk), Ok(())));
    }
    let sighash: [u8; 32] = bundle
        .commitment(tx_version)
        .expect("bundle flags are representable in this format")
        .into();
    let bvk = bundle.binding_validating_key();
    for action in bundle.actions() {
        assert_eq!(
            action.authorization().sighash_kind(),
            &OrchardSighashKind::AllEffecting,
        );
        assert_eq!(
            action.rk().verify(&sighash, action.authorization().sig()),
            Ok(())
        );
    }
    assert_eq!(
        bvk.verify(&sighash, bundle.authorization().binding_signature().sig()),
        Ok(())
    );
}

pub fn build_merkle_path(note: &Note) -> (MerklePath, Anchor) {
    // Use the tree with a single leaf.
    let cmx: ExtractedNoteCommitment = note.commitment().into();
    let (root, merkle_path) = single_leaf_witness(&cmx);
    (merkle_path, root.into())
}

trait BundleOrchardFlavor: OrchardFlavor {
    const DEFAULT_BUNDLE_VERSION: BundleVersion;
    const TX_VERSION: TxVersion;
    const SPENDS_DISABLED_FLAGS: Flags;
    type DomainVersion: DomainVersion;
}

impl BundleOrchardFlavor for OrchardVanilla {
    const DEFAULT_BUNDLE_VERSION: BundleVersion = BundleVersion::orchard_v2();
    const TX_VERSION: TxVersion = TxVersion::V5;
    const SPENDS_DISABLED_FLAGS: Flags = Flags::SPENDS_DISABLED;
    type DomainVersion = OrchardVersion;
}

impl BundleOrchardFlavor for OrchardZSA {
    const DEFAULT_BUNDLE_VERSION: BundleVersion = BundleVersion::zsa();
    const TX_VERSION: TxVersion = TxVersion::ZSA;
    const SPENDS_DISABLED_FLAGS: Flags = Flags::SPENDS_DISABLED_WITH_ZSA;
    type DomainVersion = ZSAVersion;
}

/// The flags used by the output-only (shielding and coinbase) steps of these tests: spends
/// disabled, outputs enabled, cross-address transfers enabled. Every output-only bundle here
/// targets a pool that permits cross-address transfers (Orchard pre-NU6.3 and Ironwood).
const SHIELDING_FLAGS: Flags = Flags::SPENDS_DISABLED;

/// Creates a builder of the given `bundle_version` and `bundle_type` over the
/// empty-tree anchor, with a single 5000-zat output to `recipient`. The builder disables
/// spends, since these helpers build output-only (shielding or coinbase) bundles.
fn output_only_builder(
    bundle_version: BundleVersion,
    bundle_type: BundleType,
    recipient: Address,
) -> Builder {
    let anchor = MerkleHashOrchard::empty_root(32.into()).into();
    let mut builder = Builder::new(bundle_type, bundle_version, SHIELDING_FLAGS, anchor)
        .expect("shielding flags are valid for the bundle version");
    assert_eq!(
        builder.add_output(
            None,
            recipient,
            NoteValue::from_raw(5000),
            AssetBase::zatoshi(),
            [0u8; 512]
        ),
        Ok(())
    );
    builder
}

fn bundle_chain<FL: BundleOrchardFlavor>() -> ([u8; 32], [u8; 32]) {
    let mut rng = StdRng::seed_from_u64(1u64);
    let bundle_version = FL::DEFAULT_BUNDLE_VERSION;
    let pk = ProvingKey::build::<FL>(bundle_version.circuit_version());
    let vk = VerifyingKey::build::<FL>(bundle_version.circuit_version());

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    // Create a shielding bundle.
    let (shielding_bundle, orchard_digest_1): (Bundle<_, i64, FL>, [u8; 32]) = {
        // Use the empty tree.
        let anchor = MerkleHashOrchard::empty_root(32.into()).into();

        let mut builder = Builder::new(
            BundleType::DEFAULT,
            bundle_version,
            FL::SPENDS_DISABLED_FLAGS,
            anchor,
        )
        .expect("spends-disabled flags are valid for the bundle version");
        let note_value = NoteValue::from_raw(5000);
        assert_eq!(
            builder.add_output(
                None,
                recipient,
                note_value,
                AssetBase::zatoshi(),
                [0u8; 512]
            ),
            Ok(())
        );
        let (unauthorized, bundle_meta) = builder.build(&mut rng).unwrap().unwrap();

        assert_eq!(
            unauthorized
                .decrypt_output_with_key(
                    bundle_meta
                        .output_action_index(0)
                        .expect("Output 0 can be found"),
                    &fvk.to_ivk(Scope::External)
                )
                .map(|(note, _, _)| note.value()),
            Some(note_value)
        );

        let sighash = unauthorized
            .commitment(FL::TX_VERSION)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        (
            proven.apply_signatures(rng.clone(), sighash, &[]).unwrap(),
            sighash,
        )
    };

    // Verify the shielding bundle.
    verify_bundle(&shielding_bundle, &vk, FL::TX_VERSION, true);

    let note = {
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
        shielding_bundle
            .actions()
            .iter()
            .find_map(|action| {
                let domain = NoteEncryptionDomain::<FL::DomainVersion, FL>::for_action(action);
                try_note_decryption(&domain, &ivk, action)
            })
            .unwrap()
            .0
    };

    // Test that spend adding attempt fails when spends are disabled.
    // Note: We do not need a separate positive test for spends enabled
    // as the following code adds spends with spends enabled.
    {
        let (merkle_path, anchor) = build_merkle_path(&note);

        let mut builder = Builder::new(
            BundleType::DEFAULT,
            bundle_version,
            // Intentionally testing with SPENDS_DISABLED as SPENDS_DISABLED_WITH_ZSA is already
            // tested above (for OrchardZSA case). Both should work.
            Flags::SPENDS_DISABLED,
            anchor,
        )
        .expect("spends-disabled flags are valid for the bundle version");

        assert!(builder.add_spend(fvk.clone(), note, merkle_path).is_err());
    }

    // Create a shielded bundle spending the previous output.
    let (shielded_bundle, orchard_digest_2): (Bundle<_, i64, FL>, [u8; 32]) = {
        let (merkle_path, anchor) = build_merkle_path(&note);

        let mut builder = Builder::new(
            BundleType::DEFAULT,
            bundle_version,
            bundle_version.default_flags(),
            anchor,
        )
        .expect("default flags are valid for the bundle version");
        assert_eq!(builder.add_spend(fvk, note, merkle_path), Ok(()));
        assert_eq!(
            builder.add_output(
                None,
                recipient,
                NoteValue::from_raw(5000),
                AssetBase::zatoshi(),
                [0u8; 512]
            ),
            Ok(())
        );
        let (unauthorized, _) = builder.build(&mut rng).unwrap().unwrap();
        let sighash = unauthorized
            .commitment(FL::TX_VERSION)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        (
            proven
                .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
                .unwrap(),
            sighash,
        )
    };

    // Verify the shielded bundle.
    verify_bundle(&shielded_bundle, &vk, FL::TX_VERSION, true);
    (orchard_digest_1, orchard_digest_2)
}

#[test]
fn bundle_chain_vanilla() {
    let (orchard_digest_1, orchard_digest_2) = bundle_chain::<OrchardVanilla>();
    assert_eq!(
        orchard_digest_1,
        // Locks the `orchard_digest` for OrchardVanilla
        [
            165, 242, 106, 135, 168, 224, 110, 252, 175, 110, 63, 29, 78, 243, 33, 14, 152, 202,
            209, 47, 68, 32, 138, 96, 79, 213, 218, 93, 45, 87, 221, 174
        ]
    );
    assert_eq!(
        orchard_digest_2,
        // Locks the `orchard_digest` for OrchardVanilla
        [
            74, 174, 42, 41, 68, 92, 171, 110, 10, 148, 217, 61, 68, 50, 49, 1, 1, 180, 221, 210,
            97, 237, 25, 198, 195, 77, 19, 160, 186, 172, 8, 26
        ]
    );
}

#[test]
fn bundle_chain_zsa() {
    let (orchard_digest_1, orchard_digest_2) = bundle_chain::<OrchardZSA>();
    assert_eq!(
        orchard_digest_1,
        // Locks the `orchard_digest` for OrchardZSA
        [
            51, 125, 219, 53, 244, 237, 140, 156, 133, 175, 230, 45, 156, 75, 11, 151, 151, 34,
            245, 84, 208, 196, 248, 187, 20, 54, 111, 230, 69, 34, 114, 72
        ]
    );
    assert_eq!(
        orchard_digest_2,
        // Locks the `orchard_digest` for OrchardZSA
        [
            134, 103, 36, 170, 193, 49, 193, 89, 199, 73, 231, 32, 135, 130, 9, 119, 224, 62, 101,
            240, 132, 164, 83, 61, 147, 47, 159, 94, 172, 105, 132, 82
        ]
    );
}

// A bundle built with the circuit version set to `InsecurePreNu6_2` produces a proof against
// the historical (insecure) circuit, which verifies under the insecure verifying key but not
// the fixed one. This is the path that lets tests reproduce pre-NU6.2 proofs.
#[test]
fn builder_builds_for_insecure_circuit_version() {
    let mut rng = OsRng;
    let insecure_pk = ProvingKey::build::<OrchardVanilla>(OrchardCircuitVersion::InsecurePreNu6_2);
    let insecure_vk =
        VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::InsecurePreNu6_2);
    let fixed_vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::FixedPostNu6_2);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(
        BundleVersion::orchard_insecure_v1(),
        BundleType::DEFAULT,
        recipient,
    );

    let (unauthorized, _) = builder
        .build::<i64, OrchardVanilla>(&mut rng)
        .unwrap()
        .unwrap();
    let sighash: [u8; 32] = unauthorized
        .commitment(TxVersion::V5)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&insecure_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    assert!(matches!(bundle.verify_proof(&insecure_vk), Ok(())));
    assert!(bundle.verify_proof(&fixed_vk).is_err());
}

#[test]
fn builder_builds_for_post_nu6_3_circuit_version() {
    let mut rng = OsRng;
    let post_nu6_3_pk = ProvingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(BundleVersion::ironwood_v3(), BundleType::DEFAULT, recipient);

    let (unauthorized, _) = builder
        .build::<i64, OrchardVanilla>(&mut rng)
        .unwrap()
        .unwrap();
    assert_eq!(
        unauthorized.circuit_version(),
        OrchardCircuitVersion::PostNu6_3
    );

    let sighash: [u8; 32] = unauthorized
        .commitment(TxVersion::V6)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    verify_bundle(&bundle, &post_nu6_3_vk, TxVersion::V6, true);
}

#[test]
fn ironwood_builder_outputs_decrypt_with_ironwood_domain() {
    let mut rng = OsRng;
    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);
    let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));

    let builder = output_only_builder(BundleVersion::ironwood_v3(), BundleType::DEFAULT, recipient);
    let (bundle, bundle_meta) = builder
        .build::<i64, OrchardVanilla>(&mut rng)
        .unwrap()
        .unwrap();
    let action = &bundle.actions()[bundle_meta
        .output_action_index(0)
        .expect("Output 0 can be found")];

    let orchard_domain = OrchardDomain::for_action(action);
    assert!(try_note_decryption(&orchard_domain, &ivk, action).is_none());

    let ironwood_domain = IronwoodDomain::for_action(action);
    let (note, decrypted_to, memo) =
        try_note_decryption(&ironwood_domain, &ivk, action).expect("V3 output decrypts");

    assert_eq!(note.version(), NoteVersion::V3);
    assert_eq!(note.value(), NoteValue::from_raw(5000));
    assert_eq!(decrypted_to, recipient);
    assert_eq!(memo, [0u8; 512]);
}

#[test]
fn ironwood_bundle_helpers_decrypt_and_recover_outputs() {
    let mut rng = OsRng;
    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);
    let ivk = fvk.to_ivk(Scope::External);
    let ovk = fvk.to_ovk(Scope::External);
    let bundle_version = BundleVersion::ironwood_v3();
    let anchor = MerkleHashOrchard::empty_root(32.into()).into();

    let mut builder = Builder::new(BundleType::DEFAULT, bundle_version, SHIELDING_FLAGS, anchor)
        .expect("shielding flags are valid for the bundle version");
    assert_eq!(
        builder.add_output(
            Some(ovk.clone()),
            recipient,
            NoteValue::from_raw(5000),
            AssetBase::zatoshi(),
            [0u8; 512],
        ),
        Ok(())
    );
    let (bundle, bundle_meta) = builder
        .build::<i64, OrchardVanilla>(&mut rng)
        .unwrap()
        .unwrap();
    let action_idx = bundle_meta
        .output_action_index(0)
        .expect("Output 0 can be found");

    let (note, decrypted_to, memo) = bundle
        .decrypt_output_with_key(action_idx, &ivk)
        .expect("V3 output decrypts through the bundle helper");
    assert_eq!(note.version(), NoteVersion::V3);
    assert_eq!(note.value(), NoteValue::from_raw(5000));
    assert_eq!(decrypted_to, recipient);
    assert_eq!(memo, [0u8; 512]);

    let decrypted = bundle.decrypt_outputs_with_keys(&[ivk]);
    assert_eq!(decrypted.len(), 1);
    assert_eq!(decrypted[0].0, action_idx);
    assert_eq!(decrypted[0].2.version(), NoteVersion::V3);
    assert_eq!(decrypted[0].2.value(), NoteValue::from_raw(5000));
    assert_eq!(decrypted[0].3, recipient);
    assert_eq!(decrypted[0].4, [0u8; 512]);

    let (note, recovered_to, memo) = bundle
        .recover_output_with_ovk(action_idx, &ovk)
        .expect("V3 output recovers through the bundle helper");
    assert_eq!(note.version(), NoteVersion::V3);
    assert_eq!(note.value(), NoteValue::from_raw(5000));
    assert_eq!(recovered_to, recipient);
    assert_eq!(memo, [0u8; 512]);

    let recovered = bundle.recover_outputs_with_ovks(&[ovk]);
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].0, action_idx);
    assert_eq!(recovered[0].2.version(), NoteVersion::V3);
    assert_eq!(recovered[0].2.value(), NoteValue::from_raw(5000));
    assert_eq!(recovered[0].3, recipient);
    assert_eq!(recovered[0].4, [0u8; 512]);
}

// Coinbase bundles disable nonzero-valued spends. From NU6.3, consensus requires
// nActionsOrchard = 0 in a v5+ coinbase transaction (v4, still valid after NU6.3,
// has no Orchard bundle). So a post-NU6.3 coinbase bundle built by this crate must
// be an Ironwood bundle. There the builder leaves cross-address enabled by default,
// and therefore ordinary outputs build normally.
#[test]
fn post_nu6_3_coinbase_bundle_proves_and_verifies() {
    let mut rng = OsRng;
    let post_nu6_3_pk = ProvingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(
        BundleVersion::ironwood_v3(),
        BundleType::Coinbase,
        recipient,
    );

    let (unauthorized, _) = builder
        .build::<i64, OrchardVanilla>(&mut rng)
        .unwrap()
        .unwrap();
    assert_eq!(unauthorized.actions().len(), 1);
    assert!(!unauthorized.flags().spends_enabled());
    assert!(unauthorized.flags().cross_address_enabled());

    let sighash: [u8; 32] = unauthorized
        .commitment(TxVersion::V6)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    verify_bundle(&bundle, &post_nu6_3_vk, TxVersion::V6, true);
}

// An explicitly unpadded transactional bundle builds exactly the requested single action
// instead of padding to the 2-action minimum, and the result proves and verifies on the
// post-NU6.3 circuit like any other bundle (coinbase bundles already demonstrate that
// consensus accepts 1-action bundles).
#[test]
fn unpadded_ironwood_bundle_builds_single_action_and_verifies() {
    let mut rng = OsRng;
    let post_nu6_3_pk = ProvingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(
        BundleVersion::ironwood_v3(),
        BundleType::UNPADDED,
        recipient,
    );
    assert_eq!(builder.bundle_type(), BundleType::UNPADDED);

    let (unauthorized, bundle_meta) = builder
        .build::<i64, OrchardVanilla>(&mut rng)
        .unwrap()
        .unwrap();
    assert_eq!(unauthorized.actions().len(), 1);
    assert_eq!(bundle_meta.output_action_index(0), Some(0));

    let sighash: [u8; 32] = unauthorized
        .commitment(TxVersion::V6)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    verify_bundle(&bundle, &post_nu6_3_vk, TxVersion::V6, true);
}

// A post-NU 6.3 restricted bundle chain: an ordinary shielding bundle, followed by a bundle
// that disables cross-address transfers, withdraws part of the shielded value,
// and retains the rest as wallet-controlled change.
#[test]
fn post_nu6_3_restricted_bundle_chain() {
    let mut rng = OsRng;
    let post_nu6_3_pk = ProvingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);
    let fixed_pk = ProvingKey::build::<OrchardVanilla>(OrchardCircuitVersion::FixedPostNu6_2);
    let fixed_vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::FixedPostNu6_2);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let shielding_bundle: Bundle<_, i64, OrchardVanilla> = {
        let builder =
            output_only_builder(BundleVersion::orchard_v2(), BundleType::DEFAULT, recipient);

        let (unauthorized, _) = builder.build(&mut rng).unwrap().unwrap();
        let sighash = unauthorized
            .commitment(TxVersion::V5)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&fixed_pk, &mut rng).unwrap();
        proven.apply_signatures(rng, sighash, &[]).unwrap()
    };

    verify_bundle(&shielding_bundle, &fixed_vk, TxVersion::V5, true);

    let change_addr = fvk.address_at(0u32, Scope::Internal);
    let restricted_bundle: Bundle<_, i64, OrchardVanilla> = {
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
        let (note, _, _) = shielding_bundle
            .actions()
            .iter()
            .find_map(|action| {
                let domain = OrchardDomain::for_action(action);
                try_note_decryption(&domain, &ivk, action)
            })
            .unwrap();

        let cmx: ExtractedNoteCommitment = note.commitment().into();
        let (root, merkle_path) = single_leaf_witness(&cmx);

        let mut builder = Builder::new(
            BundleType::DEFAULT,
            BundleVersion::orchard_v3(),
            BundleVersion::orchard_v3().default_flags(),
            root.into(),
        )
        .unwrap();
        assert_eq!(builder.add_spend(fvk.clone(), note, merkle_path), Ok(()));
        assert_eq!(
            builder.add_change_output(
                fvk.clone(),
                Some(fvk.to_ovk(Scope::Internal)),
                change_addr,
                NoteValue::from_raw(3000),
                AssetBase::zatoshi(),
                [0u8; 512],
            ),
            Ok(())
        );
        let (unauthorized, bundle_meta) = builder.build(&mut rng).unwrap().unwrap();

        assert_eq!(unauthorized.actions().len(), 2);
        assert_ne!(
            bundle_meta.spend_action_index(0),
            bundle_meta.output_action_index(0)
        );
        assert_eq!(
            unauthorized
                .decrypt_output_with_key(
                    bundle_meta
                        .output_action_index(0)
                        .expect("Output 0 can be found"),
                    &fvk.to_ivk(Scope::Internal),
                )
                .map(|(note, recipient, _)| (note.value(), recipient)),
            Some((NoteValue::from_raw(3000), change_addr))
        );

        // The fabricated zero-valued output paired with the real spend is addressed to the spent
        // note's own (external) receiver, but its ciphertext is randomized, so even the owning
        // wallet's external ivk cannot trial-decrypt it -- which is what keeps the spend hidden
        // from anyone (including a quantum adversary) who recovers that ivk from the address.
        assert!(unauthorized
            .decrypt_output_with_key(
                bundle_meta
                    .spend_action_index(0)
                    .expect("Spend 0 can be found"),
                &fvk.to_ivk(Scope::External),
            )
            .is_none());

        let sighash = unauthorized
            .commitment(TxVersion::V5)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };

    assert_eq!(restricted_bundle.value_balance(), &2000);
    verify_bundle(&restricted_bundle, &post_nu6_3_vk, TxVersion::V5, true);
    assert!(restricted_bundle.verify_proof(&fixed_vk).is_err());

    let mut validator = BatchValidator::new(&post_nu6_3_vk);
    validator
        .add_bundle(
            &restricted_bundle,
            restricted_bundle
                .commitment(TxVersion::V5)
                .expect("bundle flags are representable in this format")
                .into(),
        )
        .unwrap();
    assert!(validator.validate(rng));

    // A validator backed by a key that cannot constrain the cross-address restriction
    // rejects the restricted bundle at insertion, rather than deferring the failure.
    let mut validator = BatchValidator::new(&fixed_vk);
    assert!(validator
        .add_bundle(
            &restricted_bundle,
            restricted_bundle
                .commitment(TxVersion::V5)
                .expect("bundle flags are representable in this format")
                .into(),
        )
        .is_err());
}

// `BundleVersion::ironwood_v3()` is the post-NU6.3 Ironwood bundle version, which allows
// any choice of the `enableCrossAddress` flag. It shares the post-NU6.3 circuit with
// `BundleVersion::orchard_v3()`, and uses V3 note plaintexts. A transactional
// Ironwood bundle is therefore an ordinary spend+output bundle on the post-NU6.3 circuit
// whose NU6.3 flag byte sets bit 2.
#[test]
fn ironwood_post_nu6_3_unrestricted_bundle_proves_and_verifies() {
    let mut rng = OsRng;
    let post_nu6_3_pk =
        ProvingKey::build::<OrchardVanilla>(BundleVersion::ironwood_v3().circuit_version());
    let post_nu6_3_vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    // Shield a note to spend (an unrestricted, output-only post-NU6.3 bundle).
    let shielding_bundle: Bundle<_, i64, OrchardVanilla> = {
        let builder =
            output_only_builder(BundleVersion::ironwood_v3(), BundleType::DEFAULT, recipient);
        let (unauthorized, _) = builder.build(&mut rng).unwrap().unwrap();
        let sighash = unauthorized
            .commitment(TxVersion::V6)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
        proven.apply_signatures(rng, sighash, &[]).unwrap()
    };

    let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
    let (note, _, _) = shielding_bundle
        .actions()
        .iter()
        .find_map(|action| {
            let orchard_domain = OrchardDomain::for_action(action);
            assert!(try_note_decryption(&orchard_domain, &ivk, action).is_none());

            let ironwood_domain = IronwoodDomain::for_action(action);
            try_note_decryption(&ironwood_domain, &ivk, action)
        })
        .unwrap();
    let cmx: ExtractedNoteCommitment = note.commitment().into();
    let (root, merkle_path) = single_leaf_witness(&cmx);

    // Spend the external-address note and send to a different (internal) address: a
    // cross-address transfer, which Ironwood permits but post-NU6.3 Orchard would forbid.
    let change_addr = fvk.address_at(0u32, Scope::Internal);
    let mut builder = Builder::new(
        BundleType::DEFAULT,
        BundleVersion::ironwood_v3(),
        BundleVersion::ironwood_v3().default_flags(),
        root.into(),
    )
    .unwrap();
    assert_eq!(builder.add_spend(fvk.clone(), note, merkle_path), Ok(()));
    assert_eq!(
        builder.add_output(
            None,
            change_addr,
            NoteValue::from_raw(5000),
            AssetBase::zatoshi(),
            [0u8; 512]
        ),
        Ok(())
    );
    let (unauthorized, _) = builder
        .build::<i64, OrchardVanilla>(&mut rng)
        .unwrap()
        .unwrap();

    assert_eq!(
        unauthorized.circuit_version(),
        OrchardCircuitVersion::PostNu6_3
    );
    // Cross-address transfers are enabled, so bit 2 of the NU6.3 flag byte is set.
    assert!(unauthorized.flags().cross_address_enabled());
    let flag_byte = unauthorized
        .flags()
        .to_byte(BundleVersion::ironwood_v3())
        .expect("flags are representable under Ironwood");
    assert_eq!(flag_byte & 0b100, 0b100);

    let sighash = unauthorized
        .commitment(TxVersion::V6)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
    let bundle = proven
        .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
        .unwrap();

    verify_bundle(&bundle, &post_nu6_3_vk, TxVersion::V6, true);
}
