use super::{Binding, VerificationKey};
#[cfg(any(feature = "std", feature = "circuit"))]
use super::{SigningKey, SpendAuth};
use crate::value::{ValueCommitTrapdoor, ValueCommitment, ValueSum};
use ff::PrimeField;
use group::Group;
#[cfg(feature = "circuit")]
use group::GroupEncoding;
use pasta_curves::pallas;

#[test]
fn binding_keys_match_encoding_roundtrip_including_identity() {
    for scalar in [
        pallas::Scalar::zero(),
        pallas::Scalar::one(),
        -pallas::Scalar::one(),
    ] {
        let commitment = ValueCommitment::derive(
            ValueSum::from_raw(0),
            ValueCommitTrapdoor::from_bytes(scalar.to_repr()).unwrap(),
        );
        let decoded = VerificationKey::<Binding>::try_from(commitment.to_bytes()).unwrap();
        assert_eq!(commitment.into_bvk(), decoded);
    }
    let identity = VerificationKey::<Binding>::from_point(pallas::Point::identity());
    assert_eq!(<[u8; 32]>::from(identity), [0; 32]);
}

#[cfg(feature = "std")]
#[test]
fn retained_batch_items_match_encoded_keys() {
    use rand::{rand_core::UnwrapErr, rngs::SysRng};

    let spend_sk = SigningKey::<SpendAuth>::try_from(pallas::Scalar::one().to_repr()).unwrap();
    let binding_sk = SigningKey::<Binding>::try_from(pallas::Scalar::one().to_repr()).unwrap();
    let spend_vk = VerificationKey::from(&spend_sk);
    let binding_vk = VerificationKey::from(&binding_sk);
    let message = b"retained-key regression";
    let spend_sig = spend_sk.sign(UnwrapErr(SysRng), message);
    let binding_sig = binding_sk.sign(UnwrapErr(SysRng), message);

    for candidate in [message.as_slice(), b"different message".as_slice()] {
        let spend_retained = spend_vk.create_batch_item(spend_sig.clone(), &candidate);
        let spend_encoded = super::batch::Item::<SpendAuth, Binding>::from_spendauth(
            spend_vk.0.into(),
            spend_sig.0,
            &candidate,
        );
        let binding_retained = binding_vk.create_batch_item(binding_sig.clone(), &candidate);
        let binding_encoded = super::batch::Item::<SpendAuth, Binding>::from_binding(
            binding_vk.0.into(),
            binding_sig.0,
            &candidate,
        );
        let expected = candidate == message;
        assert_eq!(spend_retained.clone().verify_single().is_ok(), expected);
        assert_eq!(spend_encoded.verify_single().is_ok(), expected);
        assert_eq!(binding_retained.clone().verify_single().is_ok(), expected);
        assert_eq!(binding_encoded.verify_single().is_ok(), expected);
        let mut batch = super::batch::Verifier::new();
        batch.queue(spend_retained);
        batch.queue(binding_retained);
        assert_eq!(batch.verify(UnwrapErr(SysRng)).is_ok(), expected);
    }
}

#[cfg(feature = "circuit")]
#[test]
fn retained_spend_key_point_matches_decoding() {
    let sk = SigningKey::<SpendAuth>::try_from(pallas::Scalar::one().to_repr()).unwrap();
    let vk = VerificationKey::from(&sk);
    let encoded: [u8; 32] = (&vk).into();
    assert_eq!(vk.point(), pallas::Point::from_bytes(&encoded).unwrap());
}
