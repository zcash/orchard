use group::{Curve, GroupEncoding};
use halo2::{
    circuit::{layouter::SingleChipLayouter, Layouter},
    plonk::{self, Advice, Column, Fixed, Instance as InstanceColumn, Permutation, Selector},
    poly::{EvaluationDomain, LagrangeCoeff, Polynomial, Rotation},
    transcript::{Blake2bRead, Blake2bWrite},
};
use pasta_curves::{arithmetic::FieldExt, pallas, vesta};

use crate::{
    constants::{
        load::{OrchardFixedBasesFull, ValueCommitV},
        MERKLE_DEPTH_ORCHARD,
    },
    keys::{
        CommitIvkRandomness, DiversifiedTransmissionKey, NullifierDerivingKey, SpendValidatingKey,
    },
    note::{
        commitment::{NoteCommitTrapdoor, NoteCommitment},
        nullifier::Nullifier,
        ExtractedNoteCommitment,
    },
    primitives::{
        poseidon::{self, ConstantLength},
        redpallas::{SpendAuth, VerificationKey},
    },
    spec::NonIdentityPallasPoint,
    tree::Anchor,
    value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
};
use gadget::{
    ecc::{
        chip::{EccChip, EccConfig},
        FixedPoint, FixedPointShort, Point, ScalarFixed, ScalarFixedShort, ScalarVar,
    },
    poseidon::{
        Hash as PoseidonHash, Pow5T3Chip as PoseidonChip, Pow5T3Config as PoseidonConfig,
        StateWord, Word,
    },
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaCommitDomains, SinsemillaConfig},
        commit_ivk::CommitIvkConfig,
        merkle::{MerkleChip, MerkleConfig, MerkleInstructions},
        note_commit::NoteCommitConfig,
        CommitDomain,
    },
    utilities::{
        enable_flag::{EnableFlagChip, EnableFlagConfig},
        plonk::{PLONKChip, PLONKConfig, PLONKInstructions},
        CellValue, UtilitiesInstructions, Var,
    },
};

use std::convert::TryInto;

pub(crate) mod gadget;

/// Size of the Orchard circuit.
// FIXME: This circuit should fit within 2^11 rows.
const K: u32 = 12;

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
    commit_ivk_config: CommitIvkConfig,
    old_note_commit_config: NoteCommitConfig,
    new_note_commit_config: NoteCommitConfig,
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

impl UtilitiesInstructions<pallas::Base> for Circuit {
    type Var = CellValue<pallas::Base>;
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

        // Configuration to handle decomposition and canonicity checking
        // for CommitIvk.
        let commit_ivk_config = CommitIvkConfig::configure(meta, sinsemilla_config_1.clone());

        // Configuration to handle decomposition and canonicity checking
        // for NoteCommit_old.
        let old_note_commit_config = NoteCommitConfig::configure(meta, sinsemilla_config_2.clone());

        // Configuration to handle decomposition and canonicity checking
        // for NoteCommit_new.
        let new_note_commit_config = NoteCommitConfig::configure(meta, sinsemilla_config_1.clone());

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
            commit_ivk_config,
            old_note_commit_config,
            new_note_commit_config,
            constants,
            perm,
        }
    }

    fn synthesize(
        &self,
        cs: &mut impl plonk::Assignment<pallas::Base>,
        config: Self::Config,
    ) -> Result<(), plonk::Error> {
        // Initialize a layouter.
        let mut layouter = SingleChipLayouter::new(cs)?;

        // Load the Sinsemilla generator lookup table used by the whole circuit.
        SinsemillaChip::load(config.sinsemilla_config_1.clone(), &mut layouter)?;

        // Construct the ECC chip.
        let ecc_chip = config.ecc_chip();

        // Witness private inputs that are used across multiple checks.
        let (rho_old, psi_old, cm_old, g_d_old, ak, nk) = {
            // Witness psi_old
            let psi_old = self.load_private(
                layouter.namespace(|| "witness psi_old"),
                config.advices[0],
                self.psi_old,
            )?;

            // Witness rho_old
            let rho_old = self.load_private(
                layouter.namespace(|| "witness rho_old"),
                config.advices[0],
                self.rho_old.map(|rho| rho.0),
            )?;

            // Witness cm_old
            let cm_old = Point::new(
                config.ecc_chip(),
                layouter.namespace(|| "cm_old"),
                self.cm_old.as_ref().map(|cm| cm.to_affine()),
            )?;

            // Witness g_d_old
            let g_d_old = Point::new(
                config.ecc_chip(),
                layouter.namespace(|| "gd_old"),
                self.g_d_old.as_ref().map(|gd| gd.to_affine()),
            )?;

            // Witness ak.
            let ak: Option<pallas::Point> = self.ak.as_ref().map(|ak| ak.into());
            let ak = Point::new(
                ecc_chip.clone(),
                layouter.namespace(|| "ak"),
                ak.map(|ak| ak.to_affine()),
            )?;

            // Witness nk.
            let nk = self.load_private(
                layouter.namespace(|| "witness nk"),
                config.advices[0],
                self.nk.map(|nk| *nk),
            )?;

            (rho_old, psi_old, cm_old, g_d_old, ak, nk)
        };

        // Merkle path validity check.
        // TODO: constrain output to equal public input
        let _anchor = {
            // Cast path from Option<[pallas::Base]> to [Option<pallas::Base>]
            let path: [Option<pallas::Base>; MERKLE_DEPTH_ORCHARD] = if let Some(path) = self.path {
                path.iter()
                    .map(|node| Some(*node))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap()
            } else {
                [None; MERKLE_DEPTH_ORCHARD]
            };

            let half_merkle_depth = MERKLE_DEPTH_ORCHARD / 2;
            // Process lo half of the Merkle path from leaf to intermediate root.
            let intermediate_root = {
                let leaf = *cm_old.extract_p().inner();

                let lo_bitmask = (1 << (half_merkle_depth)) - 1;
                let pos_lo = self.pos.map(|pos| pos & lo_bitmask);

                config.merkle_chip_1().hash_path(
                    layouter.namespace(|| ""),
                    0,
                    (leaf, pos_lo),
                    path[0..(half_merkle_depth)].to_vec(),
                )?
            };

            // Process hi half of the Merkle path from intermediate root to root.
            let root = {
                let pos_hi = self.pos.map(|pos| pos >> (half_merkle_depth));

                config.merkle_chip_2().hash_path(
                    layouter.namespace(|| ""),
                    half_merkle_depth,
                    (intermediate_root, pos_hi),
                    path[(half_merkle_depth)..].to_vec(),
                )?
            };

            root
        };

        // Value commitment integrity.
        // TODO: constrain to equal public input cv_net
        let _cv_net = {
            // v_net = v_old - v_new
            let v_net = {
                let v_net_val = self.v_old.zip(self.v_new).map(|(v_old, v_new)| {
                    // Do the subtraction in the scalar field.
                    let v_old = pallas::Scalar::from_u64(v_old.inner());
                    let v_new = pallas::Scalar::from_u64(v_new.inner());
                    v_old - v_new
                });

                ScalarFixedShort::new(ecc_chip.clone(), layouter.namespace(|| "v_net"), v_net_val)?
            };

            // commitment = [v_net] ValueCommitV
            let commitment = {
                let value_commit_v = ValueCommitV::get();
                let value_commit_v = FixedPointShort::from_inner(ecc_chip.clone(), value_commit_v);
                value_commit_v.mul(layouter.namespace(|| "[v_net] ValueCommitV"), &v_net)?
            };

            // blind = [rcv] ValueCommitR
            let blind = {
                let rcv = ScalarFixed::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "rcv"),
                    self.rcv.as_ref().map(|rcv| **rcv),
                )?;
                let value_commit_r = OrchardFixedBasesFull::ValueCommitR;
                let value_commit_r = FixedPoint::from_inner(ecc_chip.clone(), value_commit_r);

                // [rcv] ValueCommitR
                value_commit_r.mul(layouter.namespace(|| "[rcv] ValueCommitR"), &rcv)?
            };

            // [v_net] ValueCommitV + [rcv] ValueCommitR
            commitment.add(layouter.namespace(|| "cv_net"), &blind)?
        };

        // Nullifier integrity
        // TODO: constrain to equal public input nf_old
        let nf_old = {
            // nk_rho_old = poseidon_hash(nk, rho_old)
            let nk_rho_old = {
                let message = [nk, rho_old];

                let poseidon_message = layouter.assign_region(
                    || "load message",
                    |mut region| {
                        let mut message_word = |i: usize| {
                            let value = message[i].value();
                            let var = region.assign_advice(
                                || format!("load message_{}", i),
                                config.poseidon_config.state[i],
                                0,
                                || value.ok_or(plonk::Error::SynthesisError),
                            )?;
                            region.constrain_equal(&config.perm, var, message[i].cell())?;
                            Ok(Word::<_, _, poseidon::OrchardNullifier, 3, 2> {
                                inner: StateWord::new(var, value),
                            })
                        };

                        Ok([message_word(0)?, message_word(1)?])
                    },
                )?;

                let poseidon_hasher = PoseidonHash::init(
                    config.poseidon_chip(),
                    layouter.namespace(|| "Poseidon init"),
                    ConstantLength::<2>,
                )?;
                let poseidon_output = poseidon_hasher.hash(
                    layouter.namespace(|| "Poseidon hash (nk, rho_old)"),
                    poseidon_message,
                )?;
                let poseidon_output: CellValue<pallas::Base> = poseidon_output.inner.into();
                poseidon_output
            };

            // Add hash output to psi using standard PLONK
            // `scalar` = poseidon_hash(nk, rho_old) + psi_old.
            //
            let scalar = {
                let scalar_val = nk_rho_old
                    .value()
                    .zip(psi_old.value())
                    .map(|(nk_rho_old, psi_old)| nk_rho_old + psi_old);
                let scalar = self.load_private(
                    layouter.namespace(|| "poseidon_hash(nk, rho_old) + psi_old"),
                    config.advices[0],
                    scalar_val,
                )?;

                config.plonk_chip().add(
                    layouter.namespace(|| "poseidon_hash(nk, rho_old) + psi_old"),
                    nk_rho_old,
                    psi_old,
                    scalar,
                    Some(pallas::Base::one()),
                    Some(pallas::Base::one()),
                    Some(pallas::Base::one()),
                )?;

                scalar
            };

            // Multiply scalar by NullifierK
            // `product` = [poseidon_hash(nk, rho_old) + psi_old] NullifierK.
            //
            let product = {
                let nullifier_k = OrchardFixedBasesFull::NullifierK;
                let nullifier_k = FixedPoint::from_inner(ecc_chip.clone(), nullifier_k);
                nullifier_k.mul_base_field_elem(
                    layouter.namespace(|| "[poseidon_output + psi_old] NullifierK"),
                    scalar,
                )?
            };

            // Add cm_old to multiplied fixed base to get nf_old
            // cm_old + [poseidon_output + psi_old] NullifierK
            cm_old
                .add(layouter.namespace(|| "nf_old"), &product)?
                .extract_p()
        };

        // Spend authority
        // TODO: constrain to equal public input rk
        let _rk = {
            // Witness alpha.
            let alpha =
                ScalarFixed::new(ecc_chip.clone(), layouter.namespace(|| "alpha"), self.alpha)?;

            // alpha_commitment = [alpha] SpendAuthG
            let alpha_commitment = {
                let spend_auth_g = OrchardFixedBasesFull::SpendAuthG;
                let spend_auth_g = FixedPoint::from_inner(ecc_chip.clone(), spend_auth_g);
                spend_auth_g.mul(layouter.namespace(|| "[alpha] SpendAuthG"), &alpha)?
            };

            // [alpha] SpendAuthG + ak
            alpha_commitment.add(layouter.namespace(|| "rk"), &ak)?
        };

        // Diversified address integrity.
        let pk_d_old = {
            let commit_ivk_config = config.commit_ivk_config.clone();

            let ivk = {
                // ak || nk
                let (ak_nk, subpieces) = commit_ivk_config.decompose(
                    config.sinsemilla_chip_1(),
                    layouter.namespace(|| "ak || nk"),
                    *ak.extract_p().inner(),
                    nk,
                )?;

                let domain = CommitDomain::new(
                    config.sinsemilla_chip_1(),
                    ecc_chip.clone(),
                    &SinsemillaCommitDomains::CommitIvk,
                );

                let rivk = ScalarFixed::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "rivk"),
                    self.rivk.map(|rivk| *rivk),
                )?;

                let (ivk, zs) =
                    domain.short_commit(layouter.namespace(|| "CommitIvk"), ak_nk, rivk)?;

                commit_ivk_config.check_canonicity(
                    layouter.namespace(|| "Check canonicity of CommitIvk inputs"),
                    subpieces,
                    zs,
                )?;

                ScalarVar::from_inner(ecc_chip.clone(), (*ivk.inner()).into())
            };

            // [ivk] g_d_old
            g_d_old.mul(layouter.namespace(|| "[ivk] g_d_old"), &ivk)?
        };

        // Old note commitment integrity.
        let _cm_old = {
            let old_note_commit_config = config.old_note_commit_config.clone();

            let v_old = {
                // Witness v_old.
                let v_old_val = self
                    .v_old
                    .map(|value| pallas::Base::from_u64(value.inner()));
                self.load_private(
                    layouter.namespace(|| "witness v_old"),
                    config.advices[0],
                    v_old_val,
                )?
            };

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let (message, subpieces) = old_note_commit_config.decompose(
                config.sinsemilla_chip_2(),
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                g_d_old.inner(),
                pk_d_old.inner(),
                v_old,
                rho_old,
                psi_old,
            )?;

            let domain = CommitDomain::new(
                config.sinsemilla_chip_2(),
                ecc_chip.clone(),
                &SinsemillaCommitDomains::NoteCommit,
            );

            let rcm_old = ScalarFixed::new(
                ecc_chip.clone(),
                layouter.namespace(|| "rcm_old"),
                self.rcm_old.as_ref().map(|rcm_old| **rcm_old),
            )?;

            let (commitment, zs) =
                domain.commit(layouter.namespace(|| "NoteCommit_old"), message, rcm_old)?;

            old_note_commit_config.check_canonicity(
                layouter.namespace(|| "Check canonicity of NoteCommit_old inputs"),
                subpieces,
                zs,
            )?;

            commitment
        };

        // new note commitment integrity.
        let _cmx = {
            let new_note_commit_config = config.new_note_commit_config.clone();

            let v_new = {
                // Witness v_new.
                let v_new_val = self
                    .v_new
                    .map(|value| pallas::Base::from_u64(value.inner()));
                self.load_private(
                    layouter.namespace(|| "witness v_new"),
                    config.advices[0],
                    v_new_val,
                )?
            };

            // Witness g_d_new_star
            let g_d_new = {
                let g_d_new = self
                    .g_d_new_star
                    .map(|bytes| pallas::Affine::from_bytes(&bytes).unwrap());
                Point::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness g_d_new_star"),
                    g_d_new,
                )?
            };

            // Witness pk_d_new_star
            let pk_d_new = {
                let pk_d_new = self
                    .pk_d_new_star
                    .map(|bytes| pallas::Affine::from_bytes(&bytes).unwrap());
                Point::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness pk_d_new"),
                    pk_d_new,
                )?
            };

            // Witness psi_new
            let psi_new = self.load_private(
                layouter.namespace(|| "witness psi_new"),
                config.advices[0],
                self.psi_new,
            )?;

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let (message, subpieces) = new_note_commit_config.decompose(
                config.sinsemilla_chip_1(),
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                g_d_new.inner(),
                pk_d_new.inner(),
                v_new,
                *nf_old.inner(),
                psi_new,
            )?;

            let domain = CommitDomain::new(
                config.sinsemilla_chip_1(),
                ecc_chip.clone(),
                &SinsemillaCommitDomains::NoteCommit,
            );

            let rcm_new = ScalarFixed::new(
                ecc_chip,
                layouter.namespace(|| "rcm_new"),
                self.rcm_new.as_ref().map(|rcm_new| **rcm_new),
            )?;

            let (commitment, zs) =
                domain.commit(layouter.namespace(|| "NoteCommit_new"), message, rcm_new)?;

            new_note_commit_config.check_canonicity(
                layouter.namespace(|| "Check canonicity of NoteCommit_new inputs"),
                subpieces,
                zs,
            )?;

            commitment.extract_p()
        };

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
