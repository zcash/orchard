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
        assert_eq!(instance_rows[super::CV_NET_X], instance.cv_net.x());
        assert_eq!(instance_rows[super::CV_NET_Y], instance.cv_net.y());
    }
}
