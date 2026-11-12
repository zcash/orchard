//! Helpers shared by the integration tests.

use orchard::{
    bundle::{Authorized, TxVersion},
    circuit::VerifyingKey,
    sighash_kind::OrchardSighashKind,
    Bundle,
};

/// Checks a bundle's proof, spend authorization signatures and binding signature against the
/// sighash it commits to under `tx_version`.
pub fn verify_bundle(
    bundle: &Bundle<Authorized, i64>,
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
