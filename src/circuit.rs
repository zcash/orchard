use group::Curve;
use halo2::{
    plonk::{self, Advice, Column, Fixed, Instance as InstanceColumn, Permutation, Selector},
    poly::{EvaluationDomain, LagrangeCoeff, Polynomial, Rotation},
    transcript::{Blake2bRead, Blake2bWrite},
};
use pasta_curves::{pallas, vesta};

use crate::{
    constants::MERKLE_DEPTH_ORCHARD,
    keys::{
        CommitIvkRandomness, DiversifiedTransmissionKey, NullifierDerivingKey, SpendValidatingKey,
    },
    note::{
        commitment::{NoteCommitTrapdoor, NoteCommitment},
        nullifier::Nullifier,
        ExtractedNoteCommitment,
    },
    primitives::{
        poseidon,
        redpallas::{SpendAuth, VerificationKey},
    },
    spec::NonIdentityPallasPoint,
    tree::Anchor,
    value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
};
use gadget::{
    ecc::chip::{EccChip, EccConfig},
    poseidon::{Pow5T3Chip as PoseidonChip, Pow5T3Config as PoseidonConfig},
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        merkle::{MerkleChip, MerkleConfig},
    },
    utilities::{
        enable_flag::{EnableFlagChip, EnableFlagConfig},
        plonk::{PLONKChip, PLONKConfig},
    },
};

use std::convert::TryInto;

pub(crate) mod gadget;

/// Size of the Orchard circuit.
const K: u32 = 11;

/// Configuration needed to use the Orchard Action circuit.
#[derive(Clone, Debug)]
pub struct Config {
    q_primary: Selector,
    primary: Column<InstanceColumn>,
    advices: [Column<Advice>; 10],
    enable_flag_config: EnableFlagConfig,
    ecc_config: EccConfig,
    poseidon_config: PoseidonConfig<pallas::Base>,
    plonk_config: PLONKConfig,
    merkle_config_1: MerkleConfig,
    merkle_config_2: MerkleConfig,
    sinsemilla_config_1: SinsemillaConfig,
    sinsemilla_config_2: SinsemillaConfig,
    constants: Column<Fixed>,
    perm: Permutation,
}

/// The Orchard Action circuit.
#[derive(Debug, Default)]
pub struct Circuit {
    pub(crate) path: Option<[pallas::Base; MERKLE_DEPTH_ORCHARD]>,
    pub(crate) pos: Option<u32>,
    pub(crate) g_d_old: Option<NonIdentityPallasPoint>,
    pub(crate) pk_d_old: Option<DiversifiedTransmissionKey>,
    pub(crate) v_old: Option<NoteValue>,
    pub(crate) rho_old: Option<Nullifier>,
    pub(crate) psi_old: Option<pallas::Base>,
    pub(crate) rcm_old: Option<NoteCommitTrapdoor>,
    pub(crate) cm_old: Option<NoteCommitment>,
    pub(crate) alpha: Option<pallas::Scalar>,
    pub(crate) ak: Option<SpendValidatingKey>,
    pub(crate) nk: Option<NullifierDerivingKey>,
    pub(crate) rivk: Option<CommitIvkRandomness>,
    pub(crate) g_d_new_star: Option<[u8; 32]>,
    pub(crate) pk_d_new_star: Option<[u8; 32]>,
    pub(crate) v_new: Option<NoteValue>,
    pub(crate) psi_new: Option<pallas::Base>,
    pub(crate) rcm_new: Option<NoteCommitTrapdoor>,
    pub(crate) rcv: Option<ValueCommitTrapdoor>,
}

impl plonk::Circuit<pallas::Base> for Circuit {
    type Config = Config;

    fn configure(meta: &mut plonk::ConstraintSystem<pallas::Base>) -> Self::Config {
        // Advice columns used in the Orchard circuit.
        let advices = [
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
            meta.advice_column(),
        ];

        // Fixed columns for the Sinsemilla generator lookup table
        let table_idx = meta.fixed_column();
        let lookup = (table_idx, meta.fixed_column(), meta.fixed_column());

        // Shared fixed column used to load constants.
        let constants = meta.fixed_column();

        // Permutation over all advice columns
        let perm = meta.permutation(
            &advices
                .iter()
                .map(|advice| (*advice).into())
                .chain(Some(constants.into()))
                .collect::<Vec<_>>(),
        );

        // Configuration for `enable_spends` and `enable_outputs` flags logic
        // TODO: this may change with public inputs API.
        let enable_flag_config =
            EnableFlagChip::configure(meta, [advices[0], advices[1]], perm.clone());

        // Configuration for curve point operations.
        // This uses 10 advice columns and spans the whole circuit.
        let ecc_config = EccChip::configure(meta, advices, table_idx, constants, perm.clone());

        // Configuration for the Poseidon hash.
        let poseidon_config = PoseidonChip::configure(
            meta,
            poseidon::OrchardNullifier,
            [advices[0], advices[1], advices[2]],
            advices[3],
        );

        // Configuration for standard PLONK (addition and multiplication).
        let plonk_config =
            PLONKChip::configure(meta, [advices[0], advices[1], advices[2]], perm.clone());

        // Configuration for a Sinsemilla hash instantiation and a
        // Merkle hash instantiation using this Sinsemilla instance.
        // Since the Sinsemilla config uses only 5 advice columns,
        // we can fit two instances side-by-side.
        let (sinsemilla_config_1, merkle_config_1) = {
            let sinsemilla_config_1 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                lookup,
                constants,
                perm.clone(),
            );
            let merkle_config_1 = MerkleChip::configure(meta, sinsemilla_config_1.clone());

            (sinsemilla_config_1, merkle_config_1)
        };

        // Configuration for a Sinsemilla hash instantiation and a
        // Merkle hash instantiation using this Sinsemilla instance.
        // Since the Sinsemilla config uses only 5 advice columns,
        // we can fit two instances side-by-side.
        let (sinsemilla_config_2, merkle_config_2) = {
            let sinsemilla_config_2 = SinsemillaChip::configure(
                meta,
                advices[5..].try_into().unwrap(),
                lookup,
                constants,
                perm.clone(),
            );
            let merkle_config_2 = MerkleChip::configure(meta, sinsemilla_config_2.clone());

            (sinsemilla_config_2, merkle_config_2)
        };

        // TODO: Infrastructure to handle public inputs.
        let q_primary = meta.selector();
        let primary = meta.instance_column();

        // TODO: Constrain cells in first few rows to equal public inputs.
        meta.create_gate("Public inputs", |meta| {
            let _public = meta.query_instance(primary, Rotation::cur());
            let _q_primary = meta.query_selector(q_primary);

            // Temporary placeholder
            vec![_q_primary.clone() - _q_primary]
        });

        Config {
            q_primary,
            primary,
            advices,
            enable_flag_config,
            ecc_config,
            poseidon_config,
            plonk_config,
            merkle_config_1,
            merkle_config_2,
            sinsemilla_config_1,
            sinsemilla_config_2,
            constants,
            perm,
        }
    }

    fn synthesize(
        &self,
        _cs: &mut impl plonk::Assignment<pallas::Base>,
        _config: Self::Config,
    ) -> Result<(), plonk::Error> {
        Ok(())
    }
}

/// The verifying key for the Orchard Action circuit.
#[derive(Debug)]
pub struct VerifyingKey {
    params: halo2::poly::commitment::Params<vesta::Affine>,
    vk: plonk::VerifyingKey<vesta::Affine>,
}

impl VerifyingKey {
    /// Builds the verifying key.
    pub fn build() -> Self {
        let params = halo2::poly::commitment::Params::new(K);
        let circuit: Circuit = Default::default(); // TODO

        let vk = plonk::keygen_vk(&params, &circuit).unwrap();

        VerifyingKey { params, vk }
    }
}

/// The proving key for the Orchard Action circuit.
#[derive(Debug)]
pub struct ProvingKey {
    params: halo2::poly::commitment::Params<vesta::Affine>,
    pk: plonk::ProvingKey<vesta::Affine>,
}

impl ProvingKey {
    /// Builds the proving key.
    pub fn build() -> Self {
        let params = halo2::poly::commitment::Params::new(K);
        let circuit: Circuit = Default::default(); // TODO

        let vk = plonk::keygen_vk(&params, &circuit).unwrap();
        let pk = plonk::keygen_pk(&params, vk, &circuit).unwrap();

        ProvingKey { params, pk }
    }
}

/// Public inputs to the Orchard Action circuit.
#[derive(Debug)]
pub struct Instance {
    pub(crate) anchor: Anchor,
    pub(crate) cv_net: ValueCommitment,
    pub(crate) nf_old: Nullifier,
    pub(crate) rk: VerificationKey<SpendAuth>,
    pub(crate) cmx: ExtractedNoteCommitment,
    pub(crate) enable_spend: bool,
    pub(crate) enable_output: bool,
}

impl Instance {
    fn to_halo2_instance(
        &self,
        domain: &EvaluationDomain<vesta::Scalar>,
    ) -> [Polynomial<vesta::Scalar, LagrangeCoeff>; 1] {
        // TODO
        [domain.empty_lagrange()]
    }

    fn to_halo2_instance_commitments(&self, vk: &VerifyingKey) -> [vesta::Affine; 1] {
        [vk.params
            .commit_lagrange(
                &self.to_halo2_instance(vk.vk.get_domain())[0],
                Default::default(),
            )
            .to_affine()]
    }
}

/// A proof of the validity of an Orchard [`Bundle`].
///
/// [`Bundle`]: crate::bundle::Bundle
#[derive(Debug, Clone)]
pub struct Proof(Vec<u8>);

impl AsRef<[u8]> for Proof {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Proof {
    /// Creates a proof for the given circuits and instances.
    pub fn create(
        pk: &ProvingKey,
        circuits: &[Circuit],
        instances: &[Instance],
    ) -> Result<Self, plonk::Error> {
        let instances: Vec<_> = instances
            .iter()
            .map(|i| i.to_halo2_instance(pk.pk.get_vk().get_domain()))
            .collect();
        let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

        let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);
        plonk::create_proof(&pk.params, &pk.pk, circuits, &instances, &mut transcript)?;
        Ok(Proof(transcript.finalize()))
    }

    /// Verifies this proof with the given instances.
    pub fn verify(&self, vk: &VerifyingKey, instances: &[Instance]) -> Result<(), plonk::Error> {
        let instances: Vec<_> = instances
            .iter()
            .map(|i| i.to_halo2_instance_commitments(vk))
            .collect();
        let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

        let msm = vk.params.empty_msm();
        let mut transcript = Blake2bRead::init(&self.0[..]);
        let guard = plonk::verify_proof(&vk.params, &vk.vk, msm, &instances, &mut transcript)?;
        let msm = guard.clone().use_challenges();
        if msm.eval() {
            Ok(())
        } else {
            Err(plonk::Error::ConstraintSystemFailure)
        }
    }

    /// Constructs a new Proof value.
    pub fn new(bytes: Vec<u8>) -> Self {
        Proof(bytes)
    }
}

#[cfg(test)]
mod tests {
    use ff::Field;
    use group::GroupEncoding;
    use halo2::dev::MockProver;
    use pasta_curves::pallas;
    use rand::rngs::OsRng;
    use std::iter;

    use super::{Circuit, Instance, Proof, ProvingKey, VerifyingKey, K};
    use crate::{
        keys::SpendValidatingKey,
        note::Note,
        tree::MerklePath,
        value::{ValueCommitTrapdoor, ValueCommitment},
    };

    // TODO: recast as a proptest
    #[test]
    fn round_trip() {
        let mut rng = OsRng;

        let (circuits, instances): (Vec<_>, Vec<_>) = iter::once(())
            .map(|()| {
                let (_, fvk, spent_note) = Note::dummy(&mut rng, None);
                let sender_address = fvk.default_address();
                let nk = *fvk.nk();
                let rivk = *fvk.rivk();
                let nf_old = spent_note.nullifier(&fvk);
                let ak: SpendValidatingKey = fvk.into();
                let alpha = pallas::Scalar::random(&mut rng);
                let rk = ak.randomize(&alpha);

                let (_, _, output_note) = Note::dummy(&mut rng, Some(nf_old));
                let cmx = output_note.commitment().into();

                let value = spent_note.value() - output_note.value();
                let cv_net = ValueCommitment::derive(value.unwrap(), ValueCommitTrapdoor::zero());

                let path = MerklePath::dummy(&mut rng);
                let anchor = path.root(spent_note.commitment().into());

                (
                    Circuit {
                        path: Some(path.auth_path()),
                        pos: Some(path.position()),
                        g_d_old: Some(sender_address.g_d()),
                        pk_d_old: Some(*sender_address.pk_d()),
                        v_old: Some(spent_note.value()),
                        rho_old: Some(spent_note.rho()),
                        psi_old: Some(spent_note.rseed().psi(&spent_note.rho())),
                        rcm_old: Some(spent_note.rseed().rcm(&spent_note.rho())),
                        cm_old: Some(spent_note.commitment()),
                        alpha: Some(alpha),
                        ak: Some(ak),
                        nk: Some(nk),
                        rivk: Some(rivk),
                        g_d_new_star: Some((*output_note.recipient().g_d()).to_bytes()),
                        pk_d_new_star: Some(output_note.recipient().pk_d().to_bytes()),
                        v_new: Some(output_note.value()),
                        psi_new: Some(output_note.rseed().psi(&output_note.rho())),
                        rcm_new: Some(output_note.rseed().rcm(&output_note.rho())),
                        rcv: Some(ValueCommitTrapdoor::zero()),
                    },
                    Instance {
                        anchor,
                        cv_net,
                        nf_old,
                        rk,
                        cmx,
                        enable_spend: true,
                        enable_output: true,
                    },
                )
            })
            .unzip();

        let vk = VerifyingKey::build();
        for (circuit, instance) in circuits.iter().zip(instances.iter()) {
            assert_eq!(
                MockProver::run(
                    K,
                    circuit,
                    instance
                        .to_halo2_instance(vk.vk.get_domain())
                        .iter()
                        .map(|p| p.iter().cloned().collect())
                        .collect()
                )
                .unwrap()
                .verify(),
                Ok(())
            );
        }

        let pk = ProvingKey::build();
        let proof = Proof::create(&pk, &circuits, &instances).unwrap();
        assert!(proof.verify(&vk, &instances).is_ok());
    }
}
