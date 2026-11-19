use alloc::vec::Vec;

use group::{Curve, GroupEncoding};
use halo2_proofs::plonk::BatchVerifier;
use pasta_curves::{arithmetic::CurveAffine, pallas, vesta};

use super::{
    Instance, Proof, ANCHOR, CMX, CV_NET_X, CV_NET_Y, DISABLE_CROSS_ADDRESS, ENABLE_OUTPUT,
    ENABLE_SPEND, INSTANCE_COLUMNS, INSTANCE_ROWS, NF_OLD, RK_X, RK_Y,
};

impl Instance {
    /// Original per-instance encoding, independent of shared affine normalization.
    pub(crate) fn to_halo2_instance_original(
        &self,
    ) -> [[vesta::Scalar; INSTANCE_ROWS]; INSTANCE_COLUMNS] {
        let mut instance = [vesta::Scalar::zero(); INSTANCE_ROWS];

        instance[ANCHOR] = self.anchor.inner();
        instance[CV_NET_X] = self.cv_net.x();
        instance[CV_NET_Y] = self.cv_net.y();
        instance[NF_OLD] = self.nf_old.inner();

        let rk = pallas::Point::from_bytes(&self.rk.clone().into())
            .expect("the cached byte encoding of a VerificationKey<_> is canonical")
            .to_affine()
            .coordinates()
            .expect("rk is non-identity by construction");

        instance[RK_X] = *rk.x();
        instance[RK_Y] = *rk.y();
        instance[CMX] = self.cmx.inner();
        instance[ENABLE_SPEND] = vesta::Scalar::from(u64::from(self.enable_spend));
        instance[ENABLE_OUTPUT] = vesta::Scalar::from(u64::from(self.enable_output));
        instance[DISABLE_CROSS_ADDRESS] =
            vesta::Scalar::from(u64::from(self.cross_address_disabled));

        [instance]
    }
}

impl Proof {
    /// Retains the original per-bundle preparation as the batch differential test's oracle.
    pub(crate) fn add_to_batch(
        &self,
        batch: &mut BatchVerifier<vesta::Affine>,
        instances: Vec<Instance>,
    ) {
        let instances = instances
            .iter()
            .map(|i| {
                i.to_halo2_instance_original()
                    .into_iter()
                    .map(|c| c.into_iter().collect())
                    .collect()
            })
            .collect();

        batch.add_proof(instances, self.0.clone());
    }
}
