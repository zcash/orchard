use super::{tests::generate_circuit_instance, OrchardCircuitVersion};
use crate::value::{ValueCommitTrapdoor, ValueCommitment};
use pasta_curves::pallas;
use rand::{rand_core::UnwrapErr, rngs::SysRng};

#[test]
fn retained_instance_key_matches_original_decoding() {
    use group::{Curve, GroupEncoding};
    use pasta_curves::arithmetic::CurveAffine;

    let (_, mut instance) =
        generate_circuit_instance(UnwrapErr(SysRng), OrchardCircuitVersion::FixedPostNu6_2);
    for value in [-1, 0, 1, i64::MIN, i64::MAX] {
        instance.cv_net = ValueCommitment::derive(
            crate::value::ValueSum::from_raw(value),
            ValueCommitTrapdoor::zero(),
        );
        let encoded: [u8; 32] = (&instance.rk).into();
        let original = pallas::Point::from_bytes(&encoded)
            .unwrap()
            .to_affine()
            .coordinates()
            .unwrap();
        let instance_rows = instance.to_halo2_instance()[0];
        assert_eq!(instance_rows[super::RK_X], *original.x());
        assert_eq!(instance_rows[super::RK_Y], *original.y());
        let cv = pallas::Point::from_bytes(&instance.cv_net.to_bytes())
            .unwrap()
            .to_affine();
        let (x, y) = Option::from(cv.coordinates())
            .map(
                |coordinates: pasta_curves::arithmetic::Coordinates<pallas::Affine>| {
                    (*coordinates.x(), *coordinates.y())
                },
            )
            .unwrap_or((pallas::Base::zero(), pallas::Base::zero()));
        assert_eq!(instance_rows[super::CV_NET_X], x);
        assert_eq!(instance_rows[super::CV_NET_Y], y);
    }
}

#[test]
fn batched_public_instances_match_independent_constant_time_encoding() {
    use super::Instance;
    let bundle = crate::bundle::tests::sample_authorized_bundle(33);
    let mut instances = bundle.to_instances();
    instances[0].cv_net = ValueCommitment::derive(
        crate::value::ValueSum::from_raw(0),
        ValueCommitTrapdoor::zero(),
    );
    for n in [0, 1, 2, 3, 4, 15, 16, 17, 31, 32, 33] {
        let expected: alloc::vec::Vec<_> = instances[..n]
            .iter()
            .map(Instance::to_halo2_instance)
            .collect();
        assert_eq!(Instance::to_halo2_instances(&instances[..n]), expected);
    }
}
