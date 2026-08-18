#![cfg(feature = "circuit")]

use incrementalmerkletree::Hashable;
use orchard::{
    builder::{Builder, BundleType},
    bundle::{BatchValidator, BundleVersion, Flags, TxVersion},
    circuit::ProvingKey,
    keys::{FullViewingKey, Scope, SpendingKey},
    tree::MerkleHashOrchard,
    value::NoteValue,
};
use rand::rngs::OsRng;

const OUTPUT_VALUE: u64 = 5_000;
const TEST_MEMO: [u8; 512] = [0; 512];
const TEST_SPENDING_KEY: [u8; 32] = [0; 32];

#[test]
fn creates_and_verifies_proof_individually_and_in_batch() {
    let mut rng = OsRng;
    let bundle_version = BundleVersion::ironwood_v3();
    let circuit_version = bundle_version.circuit_version();
    let proving_key = ProvingKey::build(circuit_version);
    let verifying_key = proving_key.verifying_key();

    let spending_key = SpendingKey::from_bytes(TEST_SPENDING_KEY).unwrap();
    let recipient = FullViewingKey::from(&spending_key).address_at(0u32, Scope::External);
    let anchor = MerkleHashOrchard::empty_root(32.into()).into();
    let mut builder = Builder::new(
        BundleType::DEFAULT,
        bundle_version,
        Flags::SPENDS_DISABLED,
        anchor,
    )
    .unwrap();
    assert_eq!(
        builder.add_output(
            None,
            recipient,
            NoteValue::from_raw(OUTPUT_VALUE),
            TEST_MEMO,
        ),
        Ok(())
    );

    let (unauthorized, _) = builder.build::<i64>(&mut rng).unwrap().unwrap();
    assert_eq!(unauthorized.circuit_version(), circuit_version);

    let sighash: [u8; 32] = unauthorized
        .commitment(TxVersion::V6)
        .expect("bundle flags are representable in this format")
        .into();
    let proven = unauthorized.create_proof(&proving_key, &mut rng).unwrap();
    let bundle = proven.apply_signatures(rng, sighash, &[]).unwrap();

    assert!(matches!(bundle.verify_proof(&verifying_key), Ok(())));
    for action in bundle.actions() {
        assert_eq!(action.rk().verify(&sighash, action.authorization()), Ok(()));
    }
    let binding_validating_key = bundle.binding_validating_key();
    assert_eq!(
        binding_validating_key.verify(&sighash, bundle.authorization().binding_signature()),
        Ok(())
    );

    // Reuse the proof so this exercises batch aggregation without paying for a
    // second proving run.
    let mut validator = BatchValidator::new(&verifying_key);
    validator.add_bundle(&bundle, sighash).unwrap();
    validator.add_bundle(&bundle, sighash).unwrap();
    assert!(validator.validate(rng));
}
