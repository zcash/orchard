//! The Orchard Action circuit implementation for the ZSA variation of the Orchard protocol.
//!
//! Includes the configuration, synthesis, and proof verification logic.

use ff::Field;

use group::Curve;

use pasta_curves::{arithmetic::CurveAffine, pallas};

use halo2_gadgets::{
    ecc::{
        chip::{EccChip, EccConfig},
        FixedPoint, NonIdentityPoint, Point, ScalarFixed, ScalarVar,
    },
    poseidon::{primitives as poseidon, Pow5Chip as PoseidonChip, Pow5Config as PoseidonConfig},
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        merkle::{
            chip::{MerkleChip, MerkleConfig},
            MerklePath,
        },
    },
    utilities::{
        bool_check,
        cond_swap::{CondSwapChip, CondSwapConfig},
        lookup_range_check::{LookupRangeCheck45BConfig, PallasLookupRangeCheck45BConfig},
    },
};

use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{self, Advice, Column, Constraints, Expression, Instance as InstanceColumn, Selector},
    poly::Rotation,
};

use crate::{
    constants::OrchardFixedBasesFull,
    constants::{OrchardCommitDomains, OrchardFixedBases, OrchardHashDomains},
    note::AssetBase,
    orchard_flavor::OrchardZSA,
};

use super::{
    commit_ivk::{
        self, {CommitIvkChip, CommitIvkConfig},
    },
    gadget::{
        add_chip::{self, AddChip, AddConfig},
        AddInstruction,
    },
    Circuit, OrchardCircuit, ANCHOR, CMX, CV_NET_X, CV_NET_Y, ENABLE_OUTPUT, ENABLE_SPEND,
    ENABLE_ZSA, NF_OLD, RK_X, RK_Y,
};

use self::{
    gadget::{assign_free_advice, assign_is_native_asset, assign_split_flag},
    note_commit::{NoteCommitChip, NoteCommitConfig},
};

pub mod gadget;
mod note_commit;
mod value_commit_orchard;

/// Configuration needed to use the Orchard Action circuit.
#[derive(Clone, Debug)]
pub struct Config {
    primary: Column<InstanceColumn>,
    q_orchard: Selector,
    advices: [Column<Advice>; 10],
    add_config: AddConfig,
    ecc_config: EccConfig<OrchardFixedBases, PallasLookupRangeCheck45BConfig>,
    poseidon_config: PoseidonConfig<pallas::Base, 3, 2>,
    merkle_config_1: MerkleConfig<
        OrchardHashDomains,
        OrchardCommitDomains,
        OrchardFixedBases,
        PallasLookupRangeCheck45BConfig,
    >,
    merkle_config_2: MerkleConfig<
        OrchardHashDomains,
        OrchardCommitDomains,
        OrchardFixedBases,
        PallasLookupRangeCheck45BConfig,
    >,
    sinsemilla_config_1: SinsemillaConfig<
        OrchardHashDomains,
        OrchardCommitDomains,
        OrchardFixedBases,
        PallasLookupRangeCheck45BConfig,
    >,
    sinsemilla_config_2: SinsemillaConfig<
        OrchardHashDomains,
        OrchardCommitDomains,
        OrchardFixedBases,
        PallasLookupRangeCheck45BConfig,
    >,
    commit_ivk_config: CommitIvkConfig,
    old_note_commit_config: NoteCommitConfig,
    new_note_commit_config: NoteCommitConfig,
    cond_swap_config: CondSwapConfig,
}

impl OrchardCircuit for OrchardZSA {
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

        // Constrain split_flag to be boolean
        // Constrain v_old * (1 - split_flag) - v_new = magnitude * sign    (https://p.z.cash/ZKS:action-cv-net-integrity?partial).
        // Constrain (v_old = 0 and is_native_asset = 1) or (calculated root = anchor) (https://p.z.cash/ZKS:action-merkle-path-validity?partial).
        // Constrain v_old = 0 or enable_spends = 1      (https://p.z.cash/ZKS:action-enable-spend).
        // Constrain v_new = 0 or enable_outputs = 1     (https://p.z.cash/ZKS:action-enable-output).
        // Constrain is_native_asset to be boolean
        // Constraint if is_native_asset = 1 then asset = native_asset else asset != native_asset
        // Constraint if split_flag = 0 then psi_old = psi_nf
        // Constraint if split_flag = 1, then is_native_asset = 0
        // Constraint if enable_zsa = 0, then is_native_asset = 1
        let q_orchard = meta.selector();
        meta.create_gate("Orchard circuit checks", |meta| {
            let q_orchard = meta.query_selector(q_orchard);
            let v_old = meta.query_advice(advices[0], Rotation::cur());
            let v_new = meta.query_advice(advices[1], Rotation::cur());
            let magnitude = meta.query_advice(advices[2], Rotation::cur());
            let sign = meta.query_advice(advices[3], Rotation::cur());

            let root = meta.query_advice(advices[4], Rotation::cur());
            let anchor = meta.query_advice(advices[5], Rotation::cur());

            let enable_spends = meta.query_advice(advices[6], Rotation::cur());
            let enable_outputs = meta.query_advice(advices[7], Rotation::cur());

            let split_flag = meta.query_advice(advices[8], Rotation::cur());

            let is_native_asset = meta.query_advice(advices[9], Rotation::cur());
            let asset_x = meta.query_advice(advices[0], Rotation::next());
            let asset_y = meta.query_advice(advices[1], Rotation::next());
            let diff_asset_x_inv = meta.query_advice(advices[2], Rotation::next());
            let diff_asset_y_inv = meta.query_advice(advices[3], Rotation::next());

            let one = Expression::Constant(pallas::Base::one());

            let native_asset = AssetBase::native()
                .cv_base()
                .to_affine()
                .coordinates()
                .unwrap();

            let diff_asset_x = asset_x - Expression::Constant(*native_asset.x());
            let diff_asset_y = asset_y - Expression::Constant(*native_asset.y());

            let psi_old = meta.query_advice(advices[4], Rotation::next());
            let psi_nf = meta.query_advice(advices[5], Rotation::next());

            let enable_zsa = meta.query_advice(advices[6], Rotation::next());

            Constraints::with_selector(
                q_orchard,
                [
                    ("bool_check split_flag", bool_check(split_flag.clone())),
                    (
                        "v_old * (1 - split_flag) - v_new = magnitude * sign",
                        v_old.clone() * (one.clone() - split_flag.clone())
                            - v_new.clone()
                            - magnitude * sign,
                    ),
                    // We already checked that
                    // * is_native_asset is boolean (just below), and
                    // * v_old is a 64 bit unsigned integer (in the note commitment evaluation).
                    // So, 1 - is_native_asset + v_old = 0 only when (is_native_asset = 1 and v_old = 0), no overflow can occur.
                    (
                        "(v_old = 0 and is_native_asset = 1) or (root = anchor)",
                        (v_old.clone() + one.clone() - is_native_asset.clone()) * (root - anchor),
                    ),
                    (
                        "v_old = 0 or enable_spends = 1",
                        v_old * (one.clone() - enable_spends),
                    ),
                    (
                        "v_new = 0 or enable_outputs = 1",
                        v_new * (one.clone() - enable_outputs),
                    ),
                    (
                        "bool_check is_native_asset",
                        bool_check(is_native_asset.clone()),
                    ),
                    (
                        "(is_native_asset = 1) =>  (asset_x = native_asset_x)",
                        is_native_asset.clone() * diff_asset_x.clone(),
                    ),
                    (
                        "(is_native_asset = 1) => (asset_y = native_asset_y)",
                        is_native_asset.clone() * diff_asset_y.clone(),
                    ),
                    // To prove that `asset` is not equal to `native_asset`, we will prove that at
                    // least one of `x(asset) - x(native_asset)` or `y(asset) - y(native_asset)` is
                    // not equal to zero.
                    // To prove that `x(asset) - x(native_asset)` (resp `y(asset) - y(native_asset)`)
                    // is not equal to zero, we will prove that it is invertible.
                    (
                        "(is_native_asset = 0) => (asset != native_asset)",
                        (one.clone() - is_native_asset.clone())
                            * (diff_asset_x * diff_asset_x_inv - one.clone())
                            * (diff_asset_y * diff_asset_y_inv - one.clone()),
                    ),
                    (
                        "(split_flag = 0) => (psi_old = psi_nf)",
                        (one.clone() - split_flag.clone()) * (psi_old - psi_nf),
                    ),
                    (
                        "(split_flag = 1) => (is_native_asset = 0)",
                        split_flag * is_native_asset.clone(),
                    ),
                    (
                        "(enable_zsa = 0) => (is_native_asset = 1)",
                        (one.clone() - enable_zsa) * (one - is_native_asset),
                    ),
                ],
            )
        });

        // Addition of two field elements.
        let add_config = AddChip::configure(meta, advices[7], advices[8], advices[6]);

        // Fixed columns for the Sinsemilla generator lookup table
        let table_idx = meta.lookup_table_column();
        let table_range_check_tag = meta.lookup_table_column();
        let lookup = (
            table_idx,
            meta.lookup_table_column(),
            meta.lookup_table_column(),
        );

        // Instance column used for public inputs
        let primary = meta.instance_column();
        meta.enable_equality(primary);

        // Permutation over all advice columns.
        for advice in advices.iter() {
            meta.enable_equality(*advice);
        }

        // Poseidon requires four advice columns, while ECC incomplete addition requires
        // six, so we could choose to configure them in parallel. However, we only use a
        // single Poseidon invocation, and we have the rows to accommodate it serially.
        // Instead, we reduce the proof size by sharing fixed columns between the ECC and
        // Poseidon chips.
        let lagrange_coeffs = [
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
            meta.fixed_column(),
        ];
        let rc_a = lagrange_coeffs[2..5].try_into().unwrap();
        let rc_b = lagrange_coeffs[5..8].try_into().unwrap();

        // Also use the first Lagrange coefficient column for loading global constants.
        // It's free real estate :)
        meta.enable_constant(lagrange_coeffs[0]);

        // We have a lot of free space in the right-most advice columns; use one of them
        // for all of our range checks.
        let range_check = LookupRangeCheck45BConfig::configure_with_tag(
            meta,
            advices[9],
            table_idx,
            table_range_check_tag,
        );

        // Configuration for curve point operations.
        // This uses 10 advice columns and spans the whole circuit.
        let ecc_config = EccChip::<OrchardFixedBases, PallasLookupRangeCheck45BConfig>::configure(
            meta,
            advices,
            lagrange_coeffs,
            range_check,
        );

        // Configuration for the Poseidon hash.
        let poseidon_config = PoseidonChip::configure::<poseidon::P128Pow5T3>(
            meta,
            // We place the state columns after the partial_sbox column so that the
            // pad-and-add region can be laid out more efficiently.
            advices[6..9].try_into().unwrap(),
            advices[5],
            rc_a,
            rc_b,
        );

        // Configuration for a Sinsemilla hash instantiation and a
        // Merkle hash instantiation using this Sinsemilla instance.
        // Since the Sinsemilla config uses only 5 advice columns,
        // we can fit two instances side-by-side.
        let (sinsemilla_config_1, merkle_config_1) = {
            let sinsemilla_config_1 = SinsemillaChip::configure(
                meta,
                advices[..5].try_into().unwrap(),
                advices[6],
                lagrange_coeffs[0],
                lookup,
                range_check,
                true,
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
                advices[7],
                lagrange_coeffs[1],
                lookup,
                range_check,
                true,
            );
            let merkle_config_2 = MerkleChip::configure(meta, sinsemilla_config_2.clone());

            (sinsemilla_config_2, merkle_config_2)
        };

        // Configuration to handle decomposition and canonicity checking
        // for CommitIvk.
        let commit_ivk_config = CommitIvkChip::configure(meta, advices);

        // Configuration to handle decomposition and canonicity checking
        // for NoteCommit_old.
        let old_note_commit_config =
            NoteCommitChip::configure(meta, advices, sinsemilla_config_1.clone());

        // Configuration to handle decomposition and canonicity checking
        // for NoteCommit_new.
        let new_note_commit_config =
            NoteCommitChip::configure(meta, advices, sinsemilla_config_2.clone());

        let cond_swap_config = CondSwapChip::configure(meta, advices[0..5].try_into().unwrap());

        Config {
            primary,
            q_orchard,
            advices,
            add_config,
            ecc_config,
            poseidon_config,
            merkle_config_1,
            merkle_config_2,
            sinsemilla_config_1,
            sinsemilla_config_2,
            commit_ivk_config,
            old_note_commit_config,
            new_note_commit_config,
            cond_swap_config,
        }
    }

    #[allow(non_snake_case)]
    fn synthesize(
        circuit: &Circuit<Self>,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), plonk::Error> {
        // Load the Sinsemilla generator lookup table used by the whole circuit.
        SinsemillaChip::load(config.sinsemilla_config_1.clone(), &mut layouter)?;

        // Construct the ECC chip.
        let ecc_chip = config.ecc_chip();

        // Witness private inputs that are used across multiple checks.
        let (psi_nf, psi_old, rho_old, cm_old, g_d_old, ak_P, nk, v_old, v_new, asset) = {
            // Witness psi_nf
            let psi_nf = assign_free_advice(
                layouter.namespace(|| "witness psi_nf"),
                config.advices[0],
                circuit.psi_nf,
            )?;

            // Witness psi_old
            let psi_old = assign_free_advice(
                layouter.namespace(|| "witness psi_old"),
                config.advices[0],
                circuit.psi_old,
            )?;

            // Witness rho_old
            let rho_old = assign_free_advice(
                layouter.namespace(|| "witness rho_old"),
                config.advices[0],
                circuit.rho_old.map(|rho| rho.into_inner()),
            )?;

            // Witness cm_old
            let cm_old = Point::new(
                ecc_chip.clone(),
                layouter.namespace(|| "cm_old"),
                circuit.cm_old.as_ref().map(|cm| cm.inner().to_affine()),
            )?;

            // Witness g_d_old
            let g_d_old = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "gd_old"),
                circuit.g_d_old.as_ref().map(|gd| gd.to_affine()),
            )?;

            // Witness ak_P.
            let ak_P: Value<pallas::Point> = circuit.ak.as_ref().map(|ak| ak.into());
            let ak_P = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness ak_P"),
                ak_P.map(|ak_P| ak_P.to_affine()),
            )?;

            // Witness nk.
            let nk = assign_free_advice(
                layouter.namespace(|| "witness nk"),
                config.advices[0],
                circuit.nk.map(|nk| nk.inner()),
            )?;

            // Witness v_old.
            let v_old = assign_free_advice(
                layouter.namespace(|| "witness v_old"),
                config.advices[0],
                circuit.v_old,
            )?;

            // Witness v_new.
            let v_new = assign_free_advice(
                layouter.namespace(|| "witness v_new"),
                config.advices[0],
                circuit.v_new,
            )?;

            // Witness asset
            let asset = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness asset"),
                circuit.asset.map(|asset| asset.cv_base().to_affine()),
            )?;

            (
                psi_nf, psi_old, rho_old, cm_old, g_d_old, ak_P, nk, v_old, v_new, asset,
            )
        };

        // Witness split_flag
        let split_flag = assign_split_flag(
            layouter.namespace(|| "witness split_flag"),
            config.advices[0],
            circuit.split_flag,
        )?;

        // Witness is_native_asset which is equal to
        // 1 if asset is equal to native asset, and
        // 0 if asset is not equal to native asset.
        let is_native_asset = assign_is_native_asset(
            layouter.namespace(|| "witness is_native_asset"),
            config.advices[0],
            circuit.asset,
        )?;

        // Merkle path validity check (https://p.z.cash/ZKS:action-merkle-path-validity?partial).
        let root = {
            let path = circuit
                .path
                .map(|typed_path| typed_path.map(|node| node.inner()));
            let merkle_inputs = MerklePath::construct(
                [config.merkle_chip_1(), config.merkle_chip_2()],
                OrchardHashDomains::MerkleCrh,
                circuit.pos,
                path,
            );
            let leaf = cm_old.extract_p().inner().clone();
            merkle_inputs.calculate_root(layouter.namespace(|| "Merkle path"), leaf)?
        };

        // Value commitment integrity (https://p.z.cash/ZKS:action-cv-net-integrity?partial).
        let v_net_magnitude_sign = {
            // Witness the magnitude and sign of v_net = v_old - v_new
            let v_net_magnitude_sign = {
                // v_net is equal to
                //   (-v_new) if split_flag = true
                //   v_old - v_new if split_flag = false
                let v_net = circuit.split_flag.and_then(|split_flag| {
                    if split_flag {
                        Value::known(crate::value::NoteValue::zero()) - circuit.v_new
                    } else {
                        circuit.v_old - circuit.v_new
                    }
                });

                let magnitude_sign = v_net.map(|v_net| {
                    let (magnitude, sign) = v_net.magnitude_sign();

                    (
                        // magnitude is guaranteed to be an unsigned 64-bit value.
                        // Therefore, we can move it into the base field.
                        pallas::Base::from(magnitude),
                        match sign {
                            crate::value::Sign::Positive => pallas::Base::one(),
                            crate::value::Sign::Negative => -pallas::Base::one(),
                        },
                    )
                });

                let magnitude = assign_free_advice(
                    layouter.namespace(|| "v_net magnitude"),
                    config.advices[9],
                    magnitude_sign.map(|m_s| m_s.0),
                )?;
                let sign = assign_free_advice(
                    layouter.namespace(|| "v_net sign"),
                    config.advices[9],
                    magnitude_sign.map(|m_s| m_s.1),
                )?;
                (magnitude, sign)
            };

            let rcv = ScalarFixed::new(
                ecc_chip.clone(),
                layouter.namespace(|| "rcv"),
                circuit.rcv.as_ref().map(|rcv| rcv.inner()),
            )?;

            let cv_net = gadget::value_commit_orchard(
                layouter.namespace(|| "cv_net = ValueCommit^Orchard_rcv(v_net_magnitude_sign)"),
                config.sinsemilla_chip_1(),
                ecc_chip.clone(),
                v_net_magnitude_sign.clone(),
                rcv,
                asset.clone(),
            )?;

            // Constrain cv_net to equal public input
            layouter.constrain_instance(cv_net.inner().x().cell(), config.primary, CV_NET_X)?;
            layouter.constrain_instance(cv_net.inner().y().cell(), config.primary, CV_NET_Y)?;

            // Return the magnitude and sign so we can use them in the Orchard gate.
            v_net_magnitude_sign
        };

        // Nullifier integrity (https://p.z.cash/ZKS:action-nullifier-integrity).
        let nf_old = {
            let nf_old = gadget::derive_nullifier(
                layouter.namespace(|| "nf_old = DeriveNullifier_nk(rho_old, psi_nf, cm_old)"),
                config.poseidon_chip(),
                config.add_chip(),
                ecc_chip.clone(),
                config.cond_swap_chip(),
                rho_old.clone(),
                &psi_nf,
                &cm_old,
                nk.clone(),
                split_flag.clone(),
            )?;

            // Constrain nf_old to equal public input
            layouter.constrain_instance(nf_old.inner().cell(), config.primary, NF_OLD)?;

            nf_old
        };

        // Spend authority (https://p.z.cash/ZKS:action-spend-authority)
        {
            let alpha = ScalarFixed::new(
                ecc_chip.clone(),
                layouter.namespace(|| "alpha"),
                circuit.alpha,
            )?;

            // alpha_commitment = [alpha] SpendAuthG
            let (alpha_commitment, _) = {
                let spend_auth_g = OrchardFixedBasesFull::SpendAuthG;
                let spend_auth_g = FixedPoint::from_inner(ecc_chip.clone(), spend_auth_g);
                spend_auth_g.mul(layouter.namespace(|| "[alpha] SpendAuthG"), alpha)?
            };

            // [alpha] SpendAuthG + ak_P
            let rk = alpha_commitment.add(layouter.namespace(|| "rk"), &ak_P)?;

            // Constrain rk to equal public input
            layouter.constrain_instance(rk.inner().x().cell(), config.primary, RK_X)?;
            layouter.constrain_instance(rk.inner().y().cell(), config.primary, RK_Y)?;
        }

        // Diversified address integrity (https://p.z.cash/ZKS:action-addr-integrity?partial).
        let pk_d_old = {
            let ivk = {
                let ak = ak_P.extract_p().inner().clone();
                let rivk = ScalarFixed::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "rivk"),
                    circuit.rivk.map(|rivk| rivk.inner()),
                )?;

                gadget::commit_ivk(
                    config.sinsemilla_chip_1(),
                    ecc_chip.clone(),
                    config.commit_ivk_chip(),
                    layouter.namespace(|| "CommitIvk"),
                    ak,
                    nk,
                    rivk,
                )?
            };
            let ivk =
                ScalarVar::from_base(ecc_chip.clone(), layouter.namespace(|| "ivk"), ivk.inner())?;

            // [ivk] g_d_old
            // The scalar value is passed through and discarded.
            let (derived_pk_d_old, _ivk) =
                g_d_old.mul(layouter.namespace(|| "[ivk] g_d_old"), ivk)?;

            // Constrain derived pk_d_old to equal witnessed pk_d_old
            //
            // This equality constraint is technically superfluous, because the assigned
            // value of `derived_pk_d_old` is an equivalent witness. But it's nice to see
            // an explicit connection between circuit-synthesized values, and explicit
            // prover witnesses. We could get the best of both worlds with a write-on-copy
            // abstraction (https://github.com/zcash/halo2/issues/334).
            let pk_d_old = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness pk_d_old"),
                circuit
                    .pk_d_old
                    .map(|pk_d_old| pk_d_old.inner().to_affine()),
            )?;
            derived_pk_d_old
                .constrain_equal(layouter.namespace(|| "pk_d_old equality"), &pk_d_old)?;

            pk_d_old
        };

        // Old note commitment integrity (https://p.z.cash/ZKS:action-cm-old-integrity?partial).
        {
            let rcm_old = ScalarFixed::new(
                ecc_chip.clone(),
                layouter.namespace(|| "rcm_old"),
                circuit.rcm_old.as_ref().map(|rcm_old| rcm_old.inner()),
            )?;

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let derived_cm_old = gadget::note_commit(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_1(),
                config.ecc_chip(),
                config.note_commit_chip_old(),
                config.cond_swap_chip(),
                g_d_old.inner(),
                pk_d_old.inner(),
                v_old.clone(),
                rho_old,
                psi_old.clone(),
                asset.inner(),
                rcm_old,
                is_native_asset.clone(),
            )?;

            // Constrain derived cm_old to equal witnessed cm_old
            derived_cm_old.constrain_equal(layouter.namespace(|| "cm_old equality"), &cm_old)?;
        }

        // New note commitment integrity (https://p.z.cash/ZKS:action-cmx-new-integrity?partial).
        {
            // Witness g_d_new
            let g_d_new = {
                let g_d_new = circuit.g_d_new.map(|g_d_new| g_d_new.to_affine());
                NonIdentityPoint::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness g_d_new_star"),
                    g_d_new,
                )?
            };

            // Witness pk_d_new
            let pk_d_new = {
                let pk_d_new = circuit
                    .pk_d_new
                    .map(|pk_d_new| pk_d_new.inner().to_affine());
                NonIdentityPoint::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness pk_d_new"),
                    pk_d_new,
                )?
            };

            // ρ^new = nf^old
            let rho_new = nf_old.inner().clone();

            // Witness psi_new
            let psi_new = assign_free_advice(
                layouter.namespace(|| "witness psi_new"),
                config.advices[0],
                circuit.psi_new,
            )?;

            let rcm_new = ScalarFixed::new(
                ecc_chip,
                layouter.namespace(|| "rcm_new"),
                circuit.rcm_new.as_ref().map(|rcm_new| rcm_new.inner()),
            )?;

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let cm_new = gadget::note_commit(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_2(),
                config.ecc_chip(),
                config.note_commit_chip_new(),
                config.cond_swap_chip(),
                g_d_new.inner(),
                pk_d_new.inner(),
                v_new.clone(),
                rho_new,
                psi_new,
                asset.inner(),
                rcm_new,
                is_native_asset.clone(),
            )?;

            let cmx = cm_new.extract_p();

            // Constrain cmx to equal public input
            layouter.constrain_instance(cmx.inner().cell(), config.primary, CMX)?;
        }

        // Constrain the remaining Orchard circuit checks.
        layouter.assign_region(
            || "Orchard circuit checks",
            |mut region| {
                v_old.copy_advice(|| "v_old", &mut region, config.advices[0], 0)?;
                v_new.copy_advice(|| "v_new", &mut region, config.advices[1], 0)?;
                v_net_magnitude_sign.0.copy_advice(
                    || "v_net magnitude",
                    &mut region,
                    config.advices[2],
                    0,
                )?;
                v_net_magnitude_sign.1.copy_advice(
                    || "v_net sign",
                    &mut region,
                    config.advices[3],
                    0,
                )?;

                root.copy_advice(|| "calculated root", &mut region, config.advices[4], 0)?;
                region.assign_advice_from_instance(
                    || "pub input anchor",
                    config.primary,
                    ANCHOR,
                    config.advices[5],
                    0,
                )?;

                region.assign_advice_from_instance(
                    || "enable spends",
                    config.primary,
                    ENABLE_SPEND,
                    config.advices[6],
                    0,
                )?;

                region.assign_advice_from_instance(
                    || "enable outputs",
                    config.primary,
                    ENABLE_OUTPUT,
                    config.advices[7],
                    0,
                )?;

                split_flag.copy_advice(|| "split_flag", &mut region, config.advices[8], 0)?;

                is_native_asset.copy_advice(
                    || "is_native_asset",
                    &mut region,
                    config.advices[9],
                    0,
                )?;
                asset
                    .inner()
                    .x()
                    .copy_advice(|| "asset_x", &mut region, config.advices[0], 1)?;
                asset
                    .inner()
                    .y()
                    .copy_advice(|| "asset_y", &mut region, config.advices[1], 1)?;

                // `diff_asset_x_inv` and `diff_asset_y_inv` will be used to prove that
                // if is_native_asset = 0, then asset != native_asset.
                region.assign_advice(
                    || "diff_asset_x_inv",
                    config.advices[2],
                    1,
                    || {
                        circuit.asset.map(|asset| {
                            let asset_x = *asset.cv_base().to_affine().coordinates().unwrap().x();
                            let native_asset_x = *AssetBase::native()
                                .cv_base()
                                .to_affine()
                                .coordinates()
                                .unwrap()
                                .x();

                            let diff_asset_x = asset_x - native_asset_x;

                            if diff_asset_x == pallas::Base::zero() {
                                pallas::Base::zero()
                            } else {
                                diff_asset_x.invert().unwrap()
                            }
                        })
                    },
                )?;
                region.assign_advice(
                    || "diff_asset_y_inv",
                    config.advices[3],
                    1,
                    || {
                        circuit.asset.map(|asset| {
                            let asset_y = *asset.cv_base().to_affine().coordinates().unwrap().y();
                            let native_asset_y = *AssetBase::native()
                                .cv_base()
                                .to_affine()
                                .coordinates()
                                .unwrap()
                                .y();

                            let diff_asset_y = asset_y - native_asset_y;

                            if diff_asset_y == pallas::Base::zero() {
                                pallas::Base::zero()
                            } else {
                                diff_asset_y.invert().unwrap()
                            }
                        })
                    },
                )?;

                psi_old.copy_advice(|| "psi_old", &mut region, config.advices[4], 1)?;
                psi_nf.copy_advice(|| "psi_nf", &mut region, config.advices[5], 1)?;

                region.assign_advice_from_instance(
                    || "enable zsa",
                    config.primary,
                    ENABLE_ZSA,
                    config.advices[6],
                    1,
                )?;

                config.q_orchard.enable(&mut region, 0)
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::iter;

    use ff::Field;
    use group::{Curve, Group, GroupEncoding};
    use halo2_proofs::{circuit::Value, dev::MockProver};
    use pasta_curves::pallas;
    use rand::{rngs::OsRng, RngCore};

    use crate::{
        builder::SpendInfo,
        bundle::Flags,
        circuit::{Circuit, Instance, Proof, ProvingKey, VerifyingKey, K},
        keys::{FullViewingKey, Scope, SpendValidatingKey, SpendingKey},
        note::{commitment::NoteCommitTrapdoor, AssetBase, Note, NoteCommitment, Nullifier, Rho},
        orchard_flavor::OrchardZSA,
        primitives::redpallas::VerificationKey,
        tree::MerklePath,
        value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
    };

    type OrchardCircuitZSA = Circuit<OrchardZSA>;

    fn generate_dummy_circuit_instance<R: RngCore>(mut rng: R) -> (OrchardCircuitZSA, Instance) {
        let (_, fvk, spent_note) = Note::dummy(&mut rng, None, AssetBase::native());

        let sender_address = spent_note.recipient();
        let nk = *fvk.nk();
        let rivk = fvk.rivk(fvk.scope_for_address(&spent_note.recipient()).unwrap());
        let nf_old = spent_note.nullifier(&fvk);
        let rho = Rho::from_nf_old(nf_old);
        let ak: SpendValidatingKey = fvk.into();
        let alpha = pallas::Scalar::random(&mut rng);
        let rk = ak.randomize(&alpha);

        let (_, _, output_note) = Note::dummy(&mut rng, Some(rho), AssetBase::native());
        let cmx = output_note.commitment().into();

        let value = spent_note.value() - output_note.value();
        let rcv = ValueCommitTrapdoor::random(&mut rng);
        let cv_net = ValueCommitment::derive(value, rcv, AssetBase::native());

        let path = MerklePath::dummy(&mut rng);
        let anchor = path.root(spent_note.commitment().into());

        let psi_old = spent_note.rseed().psi(&spent_note.rho());

        (
            OrchardCircuitZSA {
                path: Value::known(path.auth_path()),
                pos: Value::known(path.position()),
                g_d_old: Value::known(sender_address.g_d()),
                pk_d_old: Value::known(*sender_address.pk_d()),
                v_old: Value::known(spent_note.value()),
                rho_old: Value::known(spent_note.rho()),
                psi_old: Value::known(psi_old),
                rcm_old: Value::known(spent_note.rseed().rcm(&spent_note.rho())),
                cm_old: Value::known(spent_note.commitment()),
                // For non split note, psi_nf is equal to psi_old
                psi_nf: Value::known(psi_old),
                alpha: Value::known(alpha),
                ak: Value::known(ak),
                nk: Value::known(nk),
                rivk: Value::known(rivk),
                g_d_new: Value::known(output_note.recipient().g_d()),
                pk_d_new: Value::known(*output_note.recipient().pk_d()),
                v_new: Value::known(output_note.value()),
                psi_new: Value::known(output_note.rseed().psi(&output_note.rho())),
                rcm_new: Value::known(output_note.rseed().rcm(&output_note.rho())),
                rcv: Value::known(rcv),
                asset: Value::known(spent_note.asset()),
                split_flag: Value::known(false),
                phantom: std::marker::PhantomData,
            },
            Instance {
                anchor,
                cv_net,
                nf_old,
                rk,
                cmx,
                enable_spend: true,
                enable_output: true,
                enable_zsa: false,
            },
        )
    }

    // TODO: recast as a proptest
    #[test]
    fn round_trip() {
        let mut rng = OsRng;

        let (circuits, instances): (Vec<_>, Vec<_>) = iter::once(())
            .map(|()| generate_dummy_circuit_instance(&mut rng))
            .unzip();

        let vk = VerifyingKey::build::<OrchardZSA>();

        // Test that the pinned verification key (representing the circuit)
        // is as expected.
        {
            // panic!("{:#?}", vk.vk.pinned());
            assert_eq!(
                format!("{:#?}\n", vk.vk.pinned()),
                include_str!("circuit_description_zsa").replace("\r\n", "\n")
            );
        }

        // Test that the proof size is as expected.
        let expected_proof_size = {
            let circuit_cost =
                halo2_proofs::dev::CircuitCost::<pasta_curves::vesta::Point, _>::measure(
                    K,
                    &circuits[0],
                );
            assert_eq!(usize::from(circuit_cost.proof_size(1)), 5120);
            assert_eq!(usize::from(circuit_cost.proof_size(2)), 7392);
            usize::from(circuit_cost.proof_size(instances.len()))
        };

        for (circuit, instance) in circuits.iter().zip(instances.iter()) {
            assert_eq!(
                MockProver::run(
                    K,
                    circuit,
                    instance
                        .to_halo2_instance()
                        .iter()
                        .map(|p| p.to_vec())
                        .collect()
                )
                .unwrap()
                .verify(),
                Ok(())
            );
        }

        let pk = ProvingKey::build::<OrchardZSA>();
        let proof = Proof::create(&pk, &circuits, &instances, &mut rng).unwrap();
        assert!(proof.verify(&vk, &instances).is_ok());
        assert_eq!(proof.0.len(), expected_proof_size);
    }

    #[test]
    fn serialized_proof_test_case() {
        use std::fs;
        use std::io::{Read, Write};

        let vk = VerifyingKey::build::<OrchardZSA>();

        fn write_test_case<W: Write>(
            mut w: W,
            instance: &Instance,
            proof: &Proof,
        ) -> std::io::Result<()> {
            w.write_all(&instance.anchor.to_bytes())?;
            w.write_all(&instance.cv_net.to_bytes())?;
            w.write_all(&instance.nf_old.to_bytes())?;
            w.write_all(&<[u8; 32]>::from(instance.rk.clone()))?;
            w.write_all(&instance.cmx.to_bytes())?;
            w.write_all(&[
                u8::from(instance.enable_spend),
                u8::from(instance.enable_output),
                u8::from(instance.enable_zsa),
            ])?;

            w.write_all(proof.as_ref())?;
            Ok(())
        }

        fn read_test_case<R: Read>(mut r: R) -> std::io::Result<(Instance, Proof)> {
            let read_32_bytes = |r: &mut R| {
                let mut ret = [0u8; 32];
                r.read_exact(&mut ret).unwrap();
                ret
            };
            let read_bool = |r: &mut R| {
                let mut byte = [0u8; 1];
                r.read_exact(&mut byte).unwrap();
                match byte {
                    [0] => false,
                    [1] => true,
                    _ => panic!("Unexpected non-boolean byte"),
                }
            };

            let anchor = crate::Anchor::from_bytes(read_32_bytes(&mut r)).unwrap();
            let cv_net = ValueCommitment::from_bytes(&read_32_bytes(&mut r)).unwrap();
            let nf_old = crate::note::Nullifier::from_bytes(&read_32_bytes(&mut r)).unwrap();
            let rk = read_32_bytes(&mut r).try_into().unwrap();
            let cmx =
                crate::note::ExtractedNoteCommitment::from_bytes(&read_32_bytes(&mut r)).unwrap();
            let enable_spend = read_bool(&mut r);
            let enable_output = read_bool(&mut r);
            let enable_zsa = read_bool(&mut r);
            let instance = Instance::from_parts(
                anchor,
                cv_net,
                nf_old,
                rk,
                cmx,
                Flags::from_parts(enable_spend, enable_output, enable_zsa),
            );

            let mut proof_bytes = vec![];
            r.read_to_end(&mut proof_bytes)?;
            let proof = Proof::new(proof_bytes);

            Ok((instance, proof))
        }

        if std::env::var_os("ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF").is_some() {
            let create_proof = || -> std::io::Result<()> {
                let mut rng = OsRng;

                let (circuit, instance) = generate_dummy_circuit_instance(OsRng);
                let instances = &[instance.clone()];

                let pk = ProvingKey::build::<OrchardZSA>();
                let proof = Proof::create(&pk, &[circuit], instances, &mut rng).unwrap();
                assert!(proof.verify(&vk, instances).is_ok());

                let file = std::fs::File::create("src/circuit/circuit_proof_test_case_zsa.bin")?;
                write_test_case(file, &instance, &proof)
            };
            create_proof().expect("should be able to write new proof");
        }

        // Parse the hardcoded proof test case.
        let (instance, proof) = {
            let test_case_bytes = fs::read("src/circuit/circuit_proof_test_case_zsa.bin").unwrap();
            read_test_case(&test_case_bytes[..]).expect("proof must be valid")
        };
        assert_eq!(proof.0.len(), 5120);

        assert!(proof.verify(&vk, &[instance]).is_ok());
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_action_circuit() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("action-circuit-layout.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Orchard Action Circuit", ("sans-serif", 60))
            .unwrap();

        let circuit = OrchardCircuitZSA {
            path: Value::unknown(),
            pos: Value::unknown(),
            g_d_old: Value::unknown(),
            pk_d_old: Value::unknown(),
            v_old: Value::unknown(),
            rho_old: Value::unknown(),
            psi_old: Value::unknown(),
            rcm_old: Value::unknown(),
            cm_old: Value::unknown(),
            psi_nf: Value::unknown(),
            alpha: Value::unknown(),
            ak: Value::unknown(),
            nk: Value::unknown(),
            rivk: Value::unknown(),
            g_d_new: Value::unknown(),
            pk_d_new: Value::unknown(),
            v_new: Value::unknown(),
            psi_new: Value::unknown(),
            rcm_new: Value::unknown(),
            rcv: Value::unknown(),
            asset: Value::unknown(),
            split_flag: Value::unknown(),
            phantom: std::marker::PhantomData,
        };
        halo2_proofs::dev::CircuitLayout::default()
            .show_labels(false)
            .view_height(0..(1 << 11))
            .render(K, &circuit, &root)
            .unwrap();
    }

    fn check_proof_of_orchard_circuit(
        circuit: &OrchardCircuitZSA,
        instance: &Instance,
        should_pass: bool,
    ) {
        let proof_verify = MockProver::run(
            K,
            circuit,
            instance
                .to_halo2_instance()
                .iter()
                .map(|p| p.to_vec())
                .collect(),
        )
        .unwrap()
        .verify();
        if should_pass {
            assert!(proof_verify.is_ok());
        } else {
            assert!(proof_verify.is_err());
        }
    }

    fn generate_circuit_instance<R: RngCore>(
        is_native_asset: bool,
        split_flag: bool,
        mut rng: R,
    ) -> (OrchardCircuitZSA, Instance) {
        // Create asset
        let asset_base = if is_native_asset {
            AssetBase::native()
        } else {
            AssetBase::random()
        };

        // Create spent_note
        let (spent_note_fvk, spent_note) = {
            let sk = SpendingKey::random(&mut rng);
            let fvk: FullViewingKey = (&sk).into();
            let sender_address = fvk.address_at(0u32, Scope::External);
            let nf_old = Nullifier::dummy(&mut rng);
            let rho = Rho::from_nf_old(nf_old);
            let note = Note::new(
                sender_address,
                NoteValue::from_raw(40),
                asset_base,
                rho,
                &mut rng,
            );
            let spent_note = if split_flag {
                note.create_split_note(&mut rng)
            } else {
                note
            };
            (fvk, spent_note)
        };

        let output_value = NoteValue::from_raw(10);

        let (scope, v_net) = if split_flag {
            (
                Scope::External,
                // Split notes do not contribute to v_net.
                // Therefore, if split_flag is true, v_net = - output_value
                NoteValue::zero() - output_value,
            )
        } else {
            (
                spent_note_fvk
                    .scope_for_address(&spent_note.recipient())
                    .unwrap(),
                spent_note.value() - output_value,
            )
        };

        let nf_old = spent_note.nullifier(&spent_note_fvk);
        let rho = Rho::from_nf_old(nf_old);
        let ak: SpendValidatingKey = spent_note_fvk.clone().into();
        let alpha = pallas::Scalar::random(&mut rng);
        let rk = ak.randomize(&alpha);

        let output_note = {
            let sk = SpendingKey::random(&mut rng);
            let fvk: FullViewingKey = (&sk).into();
            let sender_address = fvk.address_at(0u32, Scope::External);

            Note::new(sender_address, output_value, asset_base, rho, &mut rng)
        };

        let cmx = output_note.commitment().into();

        let rcv = ValueCommitTrapdoor::random(&mut rng);
        let cv_net = ValueCommitment::derive(v_net, rcv, asset_base);

        let path = MerklePath::dummy(&mut rng);
        let anchor = path.root(spent_note.commitment().into());

        let spend_info = SpendInfo {
            dummy_sk: None,
            fvk: spent_note_fvk,
            scope,
            note: spent_note,
            merkle_path: path,
            split_flag,
        };

        (
            OrchardCircuitZSA::from_action_context_unchecked(spend_info, output_note, alpha, rcv),
            Instance {
                anchor,
                cv_net,
                nf_old,
                rk,
                cmx,
                enable_spend: true,
                enable_output: true,
                enable_zsa: true,
            },
        )
    }

    fn random_note_commitment(mut rng: impl RngCore) -> NoteCommitment {
        NoteCommitment::derive(
            pallas::Point::random(&mut rng).to_affine().to_bytes(),
            pallas::Point::random(&mut rng).to_affine().to_bytes(),
            NoteValue::from_raw(rng.next_u64()),
            AssetBase::random(),
            pallas::Base::random(&mut rng),
            pallas::Base::random(&mut rng),
            NoteCommitTrapdoor(pallas::Scalar::random(&mut rng)),
        )
        .unwrap()
    }

    #[test]
    fn orchard_circuit_negative_test() {
        let mut rng = OsRng;

        for is_native_asset in [true, false] {
            for split_flag in [true, false] {
                let (circuit, instance) =
                    generate_circuit_instance(is_native_asset, split_flag, &mut rng);

                let should_pass = !(matches!((is_native_asset, split_flag), (true, true)));

                check_proof_of_orchard_circuit(&circuit, &instance, should_pass);

                // Set cv_net to be zero
                // The proof should fail
                let instance_wrong_cv_net = Instance {
                    anchor: instance.anchor,
                    cv_net: ValueCommitment::from_bytes(&[0u8; 32]).unwrap(),
                    nf_old: instance.nf_old,
                    rk: instance.rk.clone(),
                    cmx: instance.cmx,
                    enable_spend: instance.enable_spend,
                    enable_output: instance.enable_output,
                    enable_zsa: instance.enable_zsa,
                };
                check_proof_of_orchard_circuit(&circuit, &instance_wrong_cv_net, false);

                // Set rk_pub to be a dummy VerificationKey
                // The proof should fail
                let instance_wrong_rk = Instance {
                    anchor: instance.anchor,
                    cv_net: instance.cv_net.clone(),
                    nf_old: instance.nf_old,
                    rk: VerificationKey::dummy(),
                    cmx: instance.cmx,
                    enable_spend: instance.enable_spend,
                    enable_output: instance.enable_output,
                    enable_zsa: instance.enable_zsa,
                };
                check_proof_of_orchard_circuit(&circuit, &instance_wrong_rk, false);

                // Set cm_old to be a random NoteCommitment
                // The proof should fail
                let circuit_wrong_cm_old = OrchardCircuitZSA {
                    path: circuit.path,
                    pos: circuit.pos,
                    g_d_old: circuit.g_d_old,
                    pk_d_old: circuit.pk_d_old,
                    v_old: circuit.v_old,
                    rho_old: circuit.rho_old,
                    psi_old: circuit.psi_old,
                    rcm_old: circuit.rcm_old.clone(),
                    cm_old: Value::known(random_note_commitment(&mut rng)),
                    psi_nf: circuit.psi_nf,
                    alpha: circuit.alpha,
                    ak: circuit.ak.clone(),
                    nk: circuit.nk,
                    rivk: circuit.rivk,
                    g_d_new: circuit.g_d_new,
                    pk_d_new: circuit.pk_d_new,
                    v_new: circuit.v_new,
                    psi_new: circuit.psi_new,
                    rcm_new: circuit.rcm_new.clone(),
                    rcv: circuit.rcv,
                    asset: circuit.asset,
                    split_flag: circuit.split_flag,
                    phantom: std::marker::PhantomData,
                };
                check_proof_of_orchard_circuit(&circuit_wrong_cm_old, &instance, false);

                // Set cmx_pub to be a random NoteCommitment
                // The proof should fail
                let instance_wrong_cmx_pub = Instance {
                    anchor: instance.anchor,
                    cv_net: instance.cv_net.clone(),
                    nf_old: instance.nf_old,
                    rk: instance.rk.clone(),
                    cmx: random_note_commitment(&mut rng).into(),
                    enable_spend: instance.enable_spend,
                    enable_output: instance.enable_output,
                    enable_zsa: instance.enable_zsa,
                };
                check_proof_of_orchard_circuit(&circuit, &instance_wrong_cmx_pub, false);

                // Set nf_old_pub to be a random Nullifier
                // The proof should fail
                let instance_wrong_nf_old_pub = Instance {
                    anchor: instance.anchor,
                    cv_net: instance.cv_net.clone(),
                    nf_old: Nullifier::dummy(&mut rng),
                    rk: instance.rk.clone(),
                    cmx: instance.cmx,
                    enable_spend: instance.enable_spend,
                    enable_output: instance.enable_output,
                    enable_zsa: instance.enable_zsa,
                };
                check_proof_of_orchard_circuit(&circuit, &instance_wrong_nf_old_pub, false);

                // If split_flag = 0 , set psi_nf to be a random Pallas base element
                // The proof should fail
                if !split_flag {
                    let circuit_wrong_psi_nf = OrchardCircuitZSA {
                        path: circuit.path,
                        pos: circuit.pos,
                        g_d_old: circuit.g_d_old,
                        pk_d_old: circuit.pk_d_old,
                        v_old: circuit.v_old,
                        rho_old: circuit.rho_old,
                        psi_old: circuit.psi_old,
                        rcm_old: circuit.rcm_old.clone(),
                        cm_old: circuit.cm_old.clone(),
                        psi_nf: Value::known(pallas::Base::random(&mut rng)),
                        alpha: circuit.alpha,
                        ak: circuit.ak.clone(),
                        nk: circuit.nk,
                        rivk: circuit.rivk,
                        g_d_new: circuit.g_d_new,
                        pk_d_new: circuit.pk_d_new,
                        v_new: circuit.v_new,
                        psi_new: circuit.psi_new,
                        rcm_new: circuit.rcm_new.clone(),
                        rcv: circuit.rcv,
                        asset: circuit.asset,
                        split_flag: circuit.split_flag,
                        phantom: std::marker::PhantomData,
                    };
                    check_proof_of_orchard_circuit(&circuit_wrong_psi_nf, &instance, false);
                }

                // If asset is not equal to the native asset, set enable_zsa = 0
                // The proof should fail
                if !is_native_asset {
                    let instance_wrong_enable_zsa = Instance {
                        anchor: instance.anchor,
                        cv_net: instance.cv_net.clone(),
                        nf_old: instance.nf_old,
                        rk: instance.rk.clone(),
                        cmx: instance.cmx,
                        enable_spend: instance.enable_spend,
                        enable_output: instance.enable_output,
                        enable_zsa: false,
                    };
                    check_proof_of_orchard_circuit(&circuit, &instance_wrong_enable_zsa, false);
                }
            }
        }
    }
}
