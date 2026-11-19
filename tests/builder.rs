#![cfg(feature = "circuit")]

use incrementalmerkletree::{Hashable, Marking, Retention};
use orchard::{
    builder::{BuildError, Builder, BundleType, InProgress, MaybeSigned, Unauthorized},
    bundle::{Authorized, BatchError, BatchValidator, BundleVersion, Flags, TxVersion},
    circuit::{OrchardCircuitVersion, ProvingKey, VerifyingKey},
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
    note::{ExtractedNoteCommitment, NoteVersion},
    note_encryption::{IronwoodDomain, OrchardDomain},
    tree::{MerkleHashOrchard, MerklePath},
    value::NoteValue,
    Address, Bundle, Proof,
};
use rand::{rand_core::UnwrapErr, rngs::SysRng, CryptoRng};

/// `rand` 0.10 replaces `rand_core` 0.6's infallible `OsRng` with the
/// fallible `SysRng`; unwrapping its error type recovers the old interface.
#[allow(non_upper_case_globals)]
const OsRng: UnwrapErr<SysRng> = UnwrapErr(SysRng);
use shardtree::{store::memory::MemoryShardStore, ShardTree};
use std::sync::OnceLock;
use zcash_note_encryption::try_note_decryption;

// Integration tests run in a separate process from the library tests, so they
// need their own cache of immutable circuit keys.
struct CachedTestKeys {
    circuit_version: OrchardCircuitVersion,
    proving_key: OnceLock<ProvingKey>,
    verifying_key: OnceLock<VerifyingKey>,
}

impl CachedTestKeys {
    const fn new(circuit_version: OrchardCircuitVersion) -> Self {
        Self {
            circuit_version,
            proving_key: OnceLock::new(),
            verifying_key: OnceLock::new(),
        }
    }

    fn proving_key(&self) -> &ProvingKey {
        self.proving_key
            .get_or_init(|| ProvingKey::build(self.circuit_version))
    }

    fn verifying_key(&self) -> &VerifyingKey {
        self.verifying_key
            .get_or_init(|| VerifyingKey::build(self.circuit_version))
    }
}

fn cached_test_keys(circuit_version: OrchardCircuitVersion) -> &'static CachedTestKeys {
    static INSECURE: CachedTestKeys = CachedTestKeys::new(OrchardCircuitVersion::InsecurePreNu6_2);
    static FIXED: CachedTestKeys = CachedTestKeys::new(OrchardCircuitVersion::FixedPostNu6_2);
    static POST_NU6_3: CachedTestKeys = CachedTestKeys::new(OrchardCircuitVersion::PostNu6_3);

    match circuit_version {
        OrchardCircuitVersion::InsecurePreNu6_2 => &INSECURE,
        OrchardCircuitVersion::FixedPostNu6_2 => &FIXED,
        OrchardCircuitVersion::PostNu6_3 => &POST_NU6_3,
    }
}

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

fn verify_bundle(bundle: &Bundle<Authorized, i64>, vk: &VerifyingKey, tx_version: TxVersion) {
    assert!(matches!(bundle.verify_proof(vk), Ok(())));
    let sighash: [u8; 32] = bundle
        .commitment(tx_version)
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
        builder.add_output(None, recipient, NoteValue::from_raw(5000), [0u8; 512]),
        Ok(())
    );
    builder
}

#[test]
fn bundle_chain() {
    let mut rng = OsRng;
    let keys = cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2);
    let pk = keys.proving_key();
    let vk = keys.verifying_key();

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    // Create a shielding bundle.
    let shielding_bundle: Bundle<_, i64> = {
        let builder =
            output_only_builder(BundleVersion::orchard_v2(), BundleType::DEFAULT, recipient);
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
            .commitment(TxVersion::V5)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(pk, &mut rng).unwrap();
        proven.apply_signatures(rng, sighash, &[]).unwrap()
    };

    // Verify the shielding bundle.
    verify_bundle(&shielding_bundle, vk, TxVersion::V5);

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
            BundleType::DEFAULT,
            BundleVersion::orchard_v2(),
            BundleVersion::orchard_v2().default_flags(),
            root.into(),
        )
        .unwrap();
        assert_eq!(builder.add_spend(fvk, note, merkle_path), Ok(()));
        assert_eq!(
            builder.add_output(None, recipient, NoteValue::from_raw(5000), [0u8; 512]),
            Ok(())
        );
        let (unauthorized, _) = builder.build(&mut rng).unwrap().unwrap();
        let sighash = unauthorized
            .commitment(TxVersion::V5)
            .expect("bundle flags are representable in this format")
            .into();
        let proven = unauthorized.create_proof(pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };

    // Verify the shielded bundle.
    verify_bundle(&shielded_bundle, vk, TxVersion::V5);
}

// A bundle built with the circuit version set to `InsecurePreNu6_2` produces a proof against
// the historical (insecure) circuit, which verifies under the insecure verifying key but not
// the fixed one. This is the path that lets tests reproduce pre-NU6.2 proofs.
#[test]
fn builder_builds_for_insecure_circuit_version() {
    let mut rng = OsRng;
    let insecure_keys = cached_test_keys(OrchardCircuitVersion::InsecurePreNu6_2);
    let insecure_pk = insecure_keys.proving_key();
    let insecure_vk = insecure_keys.verifying_key();
    let fixed_vk = cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2).verifying_key();

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(
        BundleVersion::orchard_insecure_v1(),
        BundleType::DEFAULT,
        recipient,
    );

    let (unauthorized, _) = builder.build::<i64>(&mut rng).unwrap().unwrap();
    let sighash: [u8; 32] = unauthorized
        .commitment(TxVersion::V5)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(insecure_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    assert!(matches!(bundle.verify_proof(insecure_vk), Ok(())));
    assert!(bundle.verify_proof(fixed_vk).is_err());
}

#[test]
fn ironwood_builder_outputs_decrypt_with_ironwood_domain() {
    let mut rng = OsRng;
    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);
    let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));

    let builder = output_only_builder(BundleVersion::ironwood_v3(), BundleType::DEFAULT, recipient);
    let (bundle, bundle_meta) = builder.build::<i64>(&mut rng).unwrap().unwrap();
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
            [0u8; 512],
        ),
        Ok(())
    );
    let (bundle, bundle_meta) = builder.build::<i64>(&mut rng).unwrap().unwrap();
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
    let post_nu6_3_keys = cached_test_keys(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_pk = post_nu6_3_keys.proving_key();
    let post_nu6_3_vk = post_nu6_3_keys.verifying_key();

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(
        BundleVersion::ironwood_v3(),
        BundleType::Coinbase,
        recipient,
    );

    let (unauthorized, _) = builder.build::<i64>(&mut rng).unwrap().unwrap();
    assert_eq!(unauthorized.actions().len(), 1);
    assert!(!unauthorized.flags().spends_enabled());
    assert!(unauthorized.flags().cross_address_enabled());

    let sighash: [u8; 32] = unauthorized
        .commitment(TxVersion::V6)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(post_nu6_3_pk, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    verify_bundle(&bundle, post_nu6_3_vk, TxVersion::V6);
}

// An explicitly unpadded transactional bundle builds exactly the requested
// single action instead of padding to the 2-action minimum. The coinbase test
// above already proves and verifies a one-action bundle on the same circuit.
#[test]
fn unpadded_ironwood_bundle_builds_single_action() {
    let mut rng = OsRng;

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let builder = output_only_builder(
        BundleVersion::ironwood_v3(),
        BundleType::UNPADDED,
        recipient,
    );
    assert_eq!(builder.bundle_type(), BundleType::UNPADDED);

    let (unauthorized, bundle_meta) = builder.build::<i64>(&mut rng).unwrap().unwrap();
    assert_eq!(unauthorized.actions().len(), 1);
    assert_eq!(bundle_meta.output_action_index(0), Some(0));
}

// A post-NU 6.3 restricted bundle chain: an ordinary shielding bundle, followed by a bundle
// that disables cross-address transfers, withdraws part of the shielded value,
// and retains the rest as wallet-controlled change.
#[test]
fn post_nu6_3_restricted_bundle_chain() {
    let mut rng = OsRng;
    let post_nu6_3_keys = cached_test_keys(OrchardCircuitVersion::PostNu6_3);
    let post_nu6_3_pk = post_nu6_3_keys.proving_key();
    let post_nu6_3_vk = post_nu6_3_keys.verifying_key();
    let fixed_vk = cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2).verifying_key();

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let shielding_bundle: Bundle<_, i64> = {
        let builder =
            output_only_builder(BundleVersion::orchard_v2(), BundleType::DEFAULT, recipient);

        builder.build(&mut rng).unwrap().unwrap().0
    };

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
        let proven = unauthorized.create_proof(post_nu6_3_pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };

    assert_eq!(restricted_bundle.value_balance(), &2000);
    verify_bundle(&restricted_bundle, post_nu6_3_vk, TxVersion::V5);
    assert!(restricted_bundle.verify_proof(fixed_vk).is_err());

    let mut validator = BatchValidator::new(post_nu6_3_vk);
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
    let mut validator = BatchValidator::new(fixed_vk);
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
    let post_nu6_3_keys = cached_test_keys(BundleVersion::ironwood_v3().circuit_version());
    let post_nu6_3_pk = post_nu6_3_keys.proving_key();
    let post_nu6_3_vk = post_nu6_3_keys.verifying_key();

    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    // Shield a note to spend (an unrestricted, output-only post-NU6.3 bundle).
    let shielding_bundle: Bundle<_, i64> = {
        let builder =
            output_only_builder(BundleVersion::ironwood_v3(), BundleType::DEFAULT, recipient);
        builder.build(&mut rng).unwrap().unwrap().0
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
        builder.add_output(None, change_addr, NoteValue::from_raw(5000), [0u8; 512]),
        Ok(())
    );
    let (unauthorized, _) = builder.build(&mut rng).unwrap().unwrap();

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
    let proven = unauthorized.create_proof(post_nu6_3_pk, &mut rng).unwrap();
    let bundle = proven
        .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
        .unwrap();

    verify_bundle(&bundle, post_nu6_3_vk, TxVersion::V6);
}

/// One sighash per action, plus a separate binding sighash, all distinct.
const ACTION_SIGHASHES: [[u8; 32]; 2] = [[0x11; 32], [0x22; 32]];
const BINDING_SIGHASH: [u8; 32] = [0x33; 32];

/// Spends the note paid by `shielding_bundle` to `fvk`, adding one output so the builder
/// pads the bundle to two actions: one real spend, and one dummy spend that is signed
/// automatically. The bundle is proven but unauthorized.
fn proven_two_action_bundle(
    pk: &ProvingKey,
    fvk: &FullViewingKey,
    shielding_bundle: &Bundle<Authorized, i64>,
    rng: &mut impl CryptoRng,
) -> Bundle<InProgress<Proof, Unauthorized>, i64> {
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
        BundleVersion::orchard_v2(),
        BundleVersion::orchard_v2().default_flags(),
        root.into(),
    )
    .unwrap();
    assert_eq!(builder.add_spend(fvk.clone(), note, merkle_path), Ok(()));
    assert_eq!(
        builder.add_output(
            None,
            fvk.address_at(0u32, Scope::External),
            NoteValue::from_raw(5000),
            [0u8; 512],
        ),
        Ok(())
    );

    let (unauthorized, _) = builder.build::<i64>(&mut *rng).unwrap().unwrap();
    // The builder pads one spend to the default minimum of two actions, so exactly one
    // action is a dummy spend signed during `prepare_with_sighashes`.
    assert_eq!(unauthorized.actions().len(), 2);
    unauthorized.create_proof(pk, &mut *rng).unwrap()
}

/// Asserts that every action and the binding signature verify under one shared digest.
fn assert_all_signed_under(bundle: &Bundle<Authorized, i64>, sighash: [u8; 32]) {
    for action in bundle.actions() {
        assert_eq!(action.rk().verify(&sighash, action.authorization()), Ok(()));
    }
    assert_eq!(
        bundle
            .binding_validating_key()
            .verify(&sighash, bundle.authorization().binding_signature()),
        Ok(())
    );
}

/// Shields 5000 zats to `fvk`, returning the proven, authorized bundle.
fn shielding_bundle(
    pk: &ProvingKey,
    fvk: &FullViewingKey,
    rng: &mut impl CryptoRng,
) -> Bundle<Authorized, i64> {
    let recipient = fvk.address_at(0u32, Scope::External);
    let builder = output_only_builder(BundleVersion::orchard_v2(), BundleType::DEFAULT, recipient);
    let (unauthorized, _) = builder.build::<i64>(&mut *rng).unwrap().unwrap();
    let sighash = unauthorized
        .commitment(TxVersion::V5)
        .expect("shielding flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(pk, &mut *rng).unwrap();
    proven.apply_signatures(rng, sighash, &[]).unwrap()
}

fn assert_signed_under(
    bundle: &Bundle<Authorized, i64>,
    action_sighashes: &[[u8; 32]],
    binding_sighash: [u8; 32],
) {
    for (action, sighash) in bundle.actions().iter().zip(action_sighashes) {
        assert_eq!(action.rk().verify(sighash, action.authorization()), Ok(()));
        assert_ne!(
            action.rk().verify(&binding_sighash, action.authorization()),
            Ok(()),
            "an action must not verify under the binding sighash"
        );
    }
    assert_eq!(
        bundle
            .binding_validating_key()
            .verify(&binding_sighash, bundle.authorization().binding_signature()),
        Ok(())
    );
}

#[test]
fn apply_signatures_with_sighashes_signs_each_action_under_its_own_sighash() {
    let mut rng = OsRng;
    let keys = cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2);
    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let ask = SpendAuthorizingKey::from(&sk);

    let shielding = shielding_bundle(keys.proving_key(), &fvk, &mut rng);
    let proven = proven_two_action_bundle(keys.proving_key(), &fvk, &shielding, &mut rng);

    let bundle = proven
        .apply_signatures_with_sighashes(rng, &ACTION_SIGHASHES, BINDING_SIGHASH, &[ask])
        .unwrap();

    assert_signed_under(&bundle, &ACTION_SIGHASHES, BINDING_SIGHASH);
    assert!(bundle.verify_proof(keys.verifying_key()).is_ok());
}

#[test]
fn prepare_with_sighashes_signs_dummies_and_verifies_external_signatures_per_action() {
    let mut rng = OsRng;
    let keys = cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2);
    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let ask = SpendAuthorizingKey::from(&sk);

    let shielding = shielding_bundle(keys.proving_key(), &fvk, &mut rng);
    let proven = proven_two_action_bundle(keys.proving_key(), &fvk, &shielding, &mut rng);

    // Clone the unauthorized proven bundle to produce a second signing path
    // independently, while the original is prepared for external-signature append.
    let proven_for_sign = proven.clone();
    let partial = proven
        .prepare_with_sighashes(rng, &ACTION_SIGHASHES, BINDING_SIGHASH)
        .unwrap();

    // The dummy spend is already signed, and its signature verifies only under the digest
    // at its own index.
    for (index, action) in partial.actions().iter().enumerate() {
        if let MaybeSigned::Signature(sig) = action.authorization() {
            assert_eq!(action.rk().verify(&ACTION_SIGHASHES[index], sig), Ok(()));
            assert_ne!(
                action.rk().verify(&ACTION_SIGHASHES[1 - index], sig),
                Ok(())
            );
        }
    }

    // Use the cloned unauthorized proven bundle to produce one signature for the
    // real spend independently, then append it to the unsigned original as an
    // external signature.
    let real_index = partial
        .actions()
        .iter()
        .position(|action| matches!(action.authorization(), MaybeSigned::SigningMetadata(_)))
        .unwrap();
    let signed = proven_for_sign
        .prepare_with_sighashes(rng, &ACTION_SIGHASHES, BINDING_SIGHASH)
        .unwrap()
        .sign(rng, &ask);
    let signature = match signed.actions()[real_index].authorization() {
        MaybeSigned::Signature(sig) => sig.clone(),
        MaybeSigned::SigningMetadata(_) => panic!("the real spend's key was supplied"),
    };

    let bundle = partial
        .append_signatures(&[signature])
        .unwrap()
        .finalize()
        .unwrap();
    assert_signed_under(&bundle, &ACTION_SIGHASHES, BINDING_SIGHASH);
}

#[test]
fn distinct_sighash_count_mismatch_is_rejected() {
    let mut rng = OsRng;
    let keys = cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2);
    let fvk = FullViewingKey::from(&SpendingKey::from_bytes([0; 32]).unwrap());

    let shielding = shielding_bundle(keys.proving_key(), &fvk, &mut rng);
    let proven = proven_two_action_bundle(keys.proving_key(), &fvk, &shielding, &mut rng);

    // Too few, and too many, per-action digests.
    for sighashes in [
        &ACTION_SIGHASHES[..1],
        &[[0u8; 32], [0u8; 32], [0u8; 32]][..],
    ] {
        assert!(matches!(
            proven.clone().prepare_with_sighashes(
                rng,
                sighashes,
                BINDING_SIGHASH
            ),
            Err(BuildError::SighashCountMismatch {
                expected: 2,
                actual,
            }) if actual == sighashes.len()
        ));
        assert!(matches!(
            proven
                .clone()
                .apply_signatures_with_sighashes(rng, sighashes, BINDING_SIGHASH, &[]),
            Err(BuildError::SighashCountMismatch { .. })
        ));
    }
}

#[test]
fn shared_sighash_wrappers_still_sign_everything_under_one_digest() {
    let mut rng = OsRng;
    let keys = cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2);
    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let shared_sighash = [0x44u8; 32];

    let shielding = shielding_bundle(keys.proving_key(), &fvk, &mut rng);
    let proven = proven_two_action_bundle(keys.proving_key(), &fvk, &shielding, &mut rng);

    // `apply_signatures` is the shared-digest wrapper: every signature verifies under the
    // one digest, and the binding signature too.
    let bundle = proven
        .apply_signatures(rng, shared_sighash, &[SpendAuthorizingKey::from(&sk)])
        .unwrap();
    assert_all_signed_under(&bundle, shared_sighash);

    // The per-action API with a repeated digest produces the same convention.
    let shielding = shielding_bundle(keys.proving_key(), &fvk, &mut rng);
    let proven = proven_two_action_bundle(keys.proving_key(), &fvk, &shielding, &mut rng);
    let repeated = [shared_sighash; 2];
    let bundle = proven
        .apply_signatures_with_sighashes(
            rng,
            &repeated,
            shared_sighash,
            &[SpendAuthorizingKey::from(&sk)],
        )
        .unwrap();
    assert_all_signed_under(&bundle, shared_sighash);
}

#[test]
fn batch_validator_uses_each_action_sighash_and_rejects_mismatched_counts() {
    let mut rng = OsRng;
    let keys = cached_test_keys(OrchardCircuitVersion::FixedPostNu6_2);
    let sk = SpendingKey::from_bytes([0; 32]).unwrap();
    let fvk = FullViewingKey::from(&sk);
    let ask = SpendAuthorizingKey::from(&sk);

    let shielding = shielding_bundle(keys.proving_key(), &fvk, &mut rng);
    let proven = proven_two_action_bundle(keys.proving_key(), &fvk, &shielding, &mut rng);
    let bundle = proven
        .apply_signatures_with_sighashes(rng, &ACTION_SIGHASHES, BINDING_SIGHASH, &[ask])
        .unwrap();

    // A count mismatch is rejected before anything is queued.
    let mut validator = BatchValidator::new(keys.verifying_key());
    assert_eq!(
        validator.add_bundle_with_sighashes(&bundle, &ACTION_SIGHASHES[..1], BINDING_SIGHASH),
        Err(BatchError::SighashCountMismatch {
            expected: 2,
            actual: 1,
        })
    );

    // Distinct per-action digests plus a separate binding digest validate.
    let mut validator = BatchValidator::new(keys.verifying_key());
    validator
        .add_bundle_with_sighashes(&bundle, &ACTION_SIGHASHES, BINDING_SIGHASH)
        .unwrap();
    assert!(validator.validate(rng));

    // A digest that does not match the action it is indexed at fails validation.
    let mut wrong = ACTION_SIGHASHES;
    wrong.swap(0, 1);
    let mut validator = BatchValidator::new(keys.verifying_key());
    validator
        .add_bundle_with_sighashes(&bundle, &wrong, BINDING_SIGHASH)
        .unwrap();
    assert!(!validator.validate(rng));

    // The shared-digest wrapper validates the same bundle under one digest.
    let shared_sighash = [0x44u8; 32];
    let shielding = shielding_bundle(keys.proving_key(), &fvk, &mut rng);
    let proven = proven_two_action_bundle(keys.proving_key(), &fvk, &shielding, &mut rng);
    let bundle = proven
        .apply_signatures(rng, shared_sighash, &[SpendAuthorizingKey::from(&sk)])
        .unwrap();
    let mut validator = BatchValidator::new(keys.verifying_key());
    validator.add_bundle(&bundle, shared_sighash).unwrap();
    assert!(validator.validate(rng));
}
