#![cfg(feature = "circuit")]

use incrementalmerkletree::{Hashable, Marking, Retention};
use orchard::{
    builder::{Builder, BundleType},
    bundle::{commitments::BundleCommitmentDomain, Authorized, BatchValidator, BundleProtocol},
    circuit::{OrchardCircuitVersion, ProvingKey, VerifyingKey},
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
    note::ExtractedNoteCommitment,
    note_encryption::OrchardDomain,
    tree::{MerkleHashOrchard, MerklePath},
    value::NoteValue,
    Address, Bundle,
};
use rand::rngs::OsRng;
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

fn verify_bundle(
    bundle: &Bundle<Authorized, i64>,
    vk: &VerifyingKey,
    domain: BundleCommitmentDomain,
) {
    assert!(matches!(bundle.verify_proof(vk), Ok(())));
    let sighash: [u8; 32] = bundle
        .commitment(domain)
        .expect("bundle flags are representable in this format")
        .into();
    let bvk = bundle.binding_validating_key();
    for action in bundle.actions() {
        assert_eq!(action.rk().verify(&sighash, action.authorization()), Ok(()));
    }
    assert_eq!(
        bvk.verify(&sighash, bundle.authorization().binding_signature()),
        Ok(())
    );
}

/// Creates a builder of the given protocol over the empty-tree anchor, with a
/// single 5000-zat output to `recipient`.
fn output_only_builder(protocol: BundleProtocol, recipient: Address) -> Builder {
    let anchor = MerkleHashOrchard::empty_root(32.into()).into();
    let mut builder = Builder::new(protocol, BundleType::DEFAULT, anchor);
    assert_eq!(
        builder.add_output(None, recipient, NoteValue::from_raw(5000), [0u8; 512]),
        Ok(())
    );
    builder
}

/// Creates a coinbase builder over the empty-tree anchor, with a single 5000-zat
/// output to `recipient`.
fn coinbase_output_only_builder(protocol: BundleProtocol, recipient: Address) -> Builder {
    let anchor = MerkleHashOrchard::empty_root(32.into()).into();
    let mut builder = Builder::new(protocol, BundleType::Coinbase, anchor);
    assert_eq!(
        builder.add_output(None, recipient, NoteValue::from_raw(5000), [0u8; 512]),
        Ok(())
    );
    builder
}

#[test]
fn bundle_chain() {
    let mut rng = OsRng;
    let pk = ProvingKey::build(OrchardCircuitVersion::FixedPostNu6_2);
    let vk = VerifyingKey::build(OrchardCircuitVersion::FixedPostNu6_2);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    // Create a shielding bundle.
    let shielding_bundle: Bundle<_, i64> = {
        let builder = output_only_builder(BundleProtocol::OrchardPreNu6_3, recipient);
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
            Some(NoteValue::from_raw(5000))
        );

        let sighash = unauthorized
            .commitment(BundleCommitmentDomain::ORCHARD_V5_PRE_NU6_3)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        proven.apply_signatures(rng, sighash, &[]).unwrap()
    };

    // Verify the shielding bundle.
    verify_bundle(
        &shielding_bundle,
        &vk,
        BundleCommitmentDomain::ORCHARD_V5_PRE_NU6_3,
    );

    // Create a shielded bundle spending the previous output.
    let shielded_bundle: Bundle<_, i64> = {
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
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
        let (root, merkle_path) = single_leaf_witness(&cmx);

        let mut builder = Builder::new(
            BundleProtocol::OrchardPreNu6_3,
            BundleType::DEFAULT,
            root.into(),
        );
        assert_eq!(builder.add_spend(fvk, note, merkle_path), Ok(()));
        assert_eq!(
            builder.add_output(None, recipient, NoteValue::from_raw(5000), [0u8; 512]),
            Ok(())
        );
        let (unauthorized, _) = builder.build(&mut rng).unwrap().unwrap();
        let sighash = unauthorized
            .commitment(BundleCommitmentDomain::ORCHARD_V5_PRE_NU6_3)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };

    // Verify the shielded bundle.
    verify_bundle(
        &shielded_bundle,
        &vk,
        BundleCommitmentDomain::ORCHARD_V5_PRE_NU6_3,
    );
}

#[test]
fn builder_builds_for_legacy_orchard_circuit_version() {
    let mut rng = OsRng;
    let fixed_pk = ProvingKey::build(OrchardCircuitVersion::FixedPostNu6_2);
    let fixed_vk = VerifyingKey::build(OrchardCircuitVersion::FixedPostNu6_2);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(BundleProtocol::OrchardPreNu6_3, recipient);

    let (unauthorized, _) = builder.build::<i64>(&mut rng).unwrap().unwrap();
    assert_eq!(
        unauthorized.circuit_version(),
        OrchardCircuitVersion::FixedPostNu6_2
    );
    let sighash: [u8; 32] = unauthorized
        .commitment(BundleCommitmentDomain::ORCHARD_V5_PRE_NU6_3)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&fixed_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    assert!(matches!(bundle.verify_proof(&fixed_vk), Ok(())));
}

#[test]
fn builder_builds_for_post_nu6_3_circuit_version() {
    let mut rng = OsRng;
    let post_nu6_3_pk = ProvingKey::build(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_vk = VerifyingKey::build(OrchardCircuitVersion::PostNu6_3);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(BundleProtocol::IronwoodPostNu6_3, recipient);

    let (unauthorized, _) = builder.build::<i64>(&mut rng).unwrap().unwrap();
    assert_eq!(
        unauthorized.circuit_version(),
        OrchardCircuitVersion::PostNu6_3
    );

    let sighash: [u8; 32] = unauthorized
        .commitment(BundleCommitmentDomain::ORCHARD_V5_NU6_3)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    verify_bundle(
        &bundle,
        &post_nu6_3_vk,
        BundleCommitmentDomain::ORCHARD_V5_NU6_3,
    );
}

// Coinbase bundles disable nonzero Orchard spends, but each Orchard action still
// has a zero-valued dummy/fabricated spend half. If cross-address transfers were
// also disabled, each output would need to be addressed to that dummy spend's
// expanded receiver, which is not useful for shielded coinbase payments. Coinbase bundles
// therefore use Flags::SPENDS_DISABLED, with cross-address transfers enabled; a
// pool whose consensus rules require the cross-address restriction on every bundle
// prohibits coinbase entirely, outside this crate.
#[test]
fn post_nu6_3_coinbase_bundle_proves_and_verifies() {
    let mut rng = OsRng;
    let post_nu6_3_pk = ProvingKey::build(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_vk = VerifyingKey::build(OrchardCircuitVersion::PostNu6_3);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = coinbase_output_only_builder(BundleProtocol::IronwoodPostNu6_3, recipient);

    let (unauthorized, _) = builder.build::<i64>(&mut rng).unwrap().unwrap();
    assert_eq!(unauthorized.actions().len(), 1);
    assert!(!unauthorized.flags().spends_enabled());
    assert!(unauthorized.flags().cross_address_enabled());

    let sighash: [u8; 32] = unauthorized
        .commitment(BundleCommitmentDomain::ORCHARD_V5_NU6_3)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    verify_bundle(
        &bundle,
        &post_nu6_3_vk,
        BundleCommitmentDomain::ORCHARD_V5_NU6_3,
    );
}

// A post-NU 6.3 restricted bundle chain: an ordinary shielding bundle, followed by a bundle
// that disables cross-address transfers, withdraws part of the shielded value,
// and retains the rest as wallet-controlled change.
#[test]
fn post_nu6_3_restricted_bundle_chain() {
    let mut rng = OsRng;
    let post_nu6_3_pk = ProvingKey::build(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_vk = VerifyingKey::build(OrchardCircuitVersion::PostNu6_3);
    let fixed_vk = VerifyingKey::build(OrchardCircuitVersion::FixedPostNu6_2);

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let shielding_bundle: Bundle<_, i64> = {
        let mut builder = Builder::new(
            BundleProtocol::OrchardPostNu6_3,
            BundleType::DEFAULT,
            MerkleHashOrchard::empty_root(32.into()).into(),
        );
        assert_eq!(
            builder.add_change_output(
                fvk.clone(),
                Some(fvk.to_ovk(Scope::External)),
                recipient,
                NoteValue::from_raw(5000),
                [0u8; 512],
            ),
            Ok(())
        );

        let (unauthorized, _) = builder.build(&mut rng).unwrap().unwrap();
        let sighash = unauthorized
            .commitment(BundleCommitmentDomain::ORCHARD_V5_NU6_3)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };

    verify_bundle(
        &shielding_bundle,
        &post_nu6_3_vk,
        BundleCommitmentDomain::ORCHARD_V5_NU6_3,
    );
    assert!(shielding_bundle.verify_proof(&fixed_vk).is_err());

    let change_addr = fvk.address_at(0u32, Scope::Internal);
    let restricted_bundle: Bundle<_, i64> = {
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
            BundleProtocol::OrchardPostNu6_3,
            BundleType::DEFAULT,
            root.into(),
        );
        assert_eq!(builder.add_spend(fvk.clone(), note, merkle_path), Ok(()));
        assert_eq!(
            builder.add_change_output(
                fvk.clone(),
                Some(fvk.to_ovk(Scope::Internal)),
                change_addr,
                NoteValue::from_raw(3000),
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

        // The fabricated zero-value output paired with the real spend is addressed to the spent
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
            .commitment(BundleCommitmentDomain::ORCHARD_V5_NU6_3)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(&post_nu6_3_pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };

    assert_eq!(restricted_bundle.value_balance(), &2000);
    verify_bundle(
        &restricted_bundle,
        &post_nu6_3_vk,
        BundleCommitmentDomain::ORCHARD_V5_NU6_3,
    );
    assert!(restricted_bundle.verify_proof(&fixed_vk).is_err());

    let mut validator = BatchValidator::new(&post_nu6_3_vk);
    validator
        .add_bundle(
            &restricted_bundle,
            restricted_bundle
                .commitment(BundleCommitmentDomain::ORCHARD_V5_NU6_3)
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
                .commitment(BundleCommitmentDomain::ORCHARD_V5_NU6_3)
                .expect("bundle flags are representable in this format")
                .into(),
        )
        .is_err());
}
