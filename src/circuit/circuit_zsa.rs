//! The Orchard Action circuit implementation for the ZSA variation of the Orchard protocol.
//!
//! Includes the configuration, synthesis, and proof verification logic.

use ff::Field;

use group::Curve;

use pasta_curves::{arithmetic::CurveAffine, pallas};

use halo2_gadgets::{
    ecc::{FixedPoint, NonIdentityPoint, Point, ScalarFixed, ScalarVar},
    sinsemilla::{chip::SinsemillaChip, merkle::MerklePath},
    utilities::{bool_check, lookup_range_check::PallasLookupRangeCheck4_5BConfig},
};

use halo2_proofs::{
    circuit::{floor_planner, Layouter, Value},
    plonk::{self, Advice, Column, Constraints, Expression, Selector},
    poly::Rotation,
};

use crate::{
    circuit::{
        commit_ivk::gadgets::commit_ivk,
        configure_circuit,
        derive_nullifier::{gadgets::derive_nullifier, ZsaNullifierParams},
        gadget::{assign_free_advice, assign_is_zatoshi_asset, assign_split_flag},
        note_commit::{gadgets::note_commit, ZsaNoteCommitParams},
        value_commit_orchard::{gadgets::value_commit_orchard, ZsaValueCommitParams},
        AddressPoints, CircuitVanilla, Config, OrchardCircuitVersion, ANCHOR, CMX, CV_NET_X,
        CV_NET_Y, DISABLE_CROSS_ADDRESS, ENABLE_OUTPUT, ENABLE_SPEND, ENABLE_ZSA, NF_OLD, RK_X,
        RK_Y,
    },
    constants::{OrchardFixedBasesFull, OrchardHashDomains},
    note::AssetBase,
};

/// The ZSA-specific witnesses.
#[derive(Clone, Debug)]
pub(crate) struct AdditionalZsaWitnesses {
    pub(crate) psi_nf: Value<pallas::Base>,
    pub(crate) asset: Value<AssetBase>,
    pub(crate) split_flag: Value<bool>,
}

/// The OrchardZSA Action circuit.
#[derive(Clone, Debug)]
pub struct CircuitZsa {
    pub(crate) common_witnesses: CircuitVanilla,

    // The ZSA-specific witnesses.
    pub(crate) additional_zsa_witnesses: AdditionalZsaWitnesses,
}

impl CircuitZsa {
    /// Returns an empty circuit with all private witnesses unknown.
    ///
    /// This is used for circuit shape-dependent operations, such as generating keys
    /// or rendering the circuit layout, where witness values are not required.
    pub(crate) fn empty() -> Self {
        CircuitZsa {
            common_witnesses: CircuitVanilla::empty(OrchardCircuitVersion::ZSA),
            additional_zsa_witnesses: AdditionalZsaWitnesses {
                psi_nf: Value::unknown(),
                asset: Value::unknown(),
                split_flag: Value::unknown(),
            },
        }
    }
}

/// Creates the `q_orchard` gate checking the OrchardZSA Action statement, on the given
/// selector.
pub(super) fn configure_zsa_orchard_gate(
    meta: &mut plonk::ConstraintSystem<pallas::Base>,
    advices: [Column<Advice>; 10],
    q_orchard: Selector,
) {
    // The new or updated constraints for OrchardZSA are explained in
    // [ZIP-226: Transfer and Burn of Zcash Shielded Assets][circuitstatement].
    //
    // All OrchardZSA constraints:
    // Constrain split_flag to be boolean
    // Constrain v_old * (1 - split_flag) - v_new = magnitude * sign
    // Constrain (v_old = 0 and is_zatoshi_asset = 1) or (calculated root = anchor)
    // Constrain v_old = 0 or enable_spend = 1
    // Constrain v_new = 0 or enable_output = 1
    // Constrain is_zatoshi_asset to be boolean
    // Constrain if is_zatoshi_asset = 1 then asset = zatoshi_asset else asset != zatoshi_asset
    // Constrain if split_flag = 0 then psi_old = psi_nf
    // Constrain if split_flag = 1, then is_zatoshi_asset = 0
    // Constrain if enable_zsa = 0, then is_zatoshi_asset = 1
    // Constrain if disable_cross_address = 1, then split_flag = 0
    //
    // [circuitstatement]: https://zips.z.cash/zip-0226#circuit-statement
    meta.create_gate("Orchard circuit checks", |meta| {
        let q_orchard = meta.query_selector(q_orchard);
        let v_old = meta.query_advice(advices[0], Rotation::cur());
        let v_new = meta.query_advice(advices[1], Rotation::cur());
        let magnitude = meta.query_advice(advices[2], Rotation::cur());
        let sign = meta.query_advice(advices[3], Rotation::cur());

        let root = meta.query_advice(advices[4], Rotation::cur());
        let anchor = meta.query_advice(advices[5], Rotation::cur());

        let enable_spend = meta.query_advice(advices[6], Rotation::cur());
        let enable_output = meta.query_advice(advices[7], Rotation::cur());

        let split_flag = meta.query_advice(advices[8], Rotation::cur());

        let is_zatoshi_asset = meta.query_advice(advices[9], Rotation::cur());
        let asset_x = meta.query_advice(advices[0], Rotation::next());
        let asset_y = meta.query_advice(advices[1], Rotation::next());
        let diff_asset_x_inv = meta.query_advice(advices[2], Rotation::next());
        let diff_asset_y_inv = meta.query_advice(advices[3], Rotation::next());

        let one = Expression::Constant(pallas::Base::one());

        let zatoshi_asset = AssetBase::zatoshi()
            .cv_base()
            .to_affine()
            .coordinates()
            .unwrap();

        let diff_asset_x = asset_x - Expression::Constant(*zatoshi_asset.x());
        let diff_asset_y = asset_y - Expression::Constant(*zatoshi_asset.y());

        let psi_old = meta.query_advice(advices[4], Rotation::next());
        let psi_nf = meta.query_advice(advices[5], Rotation::next());

        let enable_zsa = meta.query_advice(advices[6], Rotation::next());
        let disable_cross_address = meta.query_advice(advices[7], Rotation::next());

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
                // * is_zatoshi_asset is boolean (just below), and
                // * v_old is a 64 bit unsigned integer (in the note commitment evaluation).
                // So, 1 - is_zatoshi_asset + v_old = 0 only when (is_zatoshi_asset = 1 and v_old = 0), no overflow can occur.
                (
                    "(v_old = 0 and is_zatoshi_asset = 1) or (root = anchor)",
                    (v_old.clone() + one.clone() - is_zatoshi_asset.clone()) * (root - anchor),
                ),
                (
                    "v_old = 0 or enable_spend = 1",
                    v_old * (one.clone() - enable_spend),
                ),
                (
                    "v_new = 0 or enable_output = 1",
                    v_new * (one.clone() - enable_output),
                ),
                (
                    "bool_check is_zatoshi_asset",
                    bool_check(is_zatoshi_asset.clone()),
                ),
                (
                    "(is_zatoshi_asset = 1) =>  (asset_x = zatoshi_asset_x)",
                    is_zatoshi_asset.clone() * diff_asset_x.clone(),
                ),
                (
                    "(is_zatoshi_asset = 1) => (asset_y = zatoshi_asset_y)",
                    is_zatoshi_asset.clone() * diff_asset_y.clone(),
                ),
                // To prove that `asset` is not equal to `zatoshi_asset`, we will prove that at
                // least one of `x(asset) - x(zatoshi_asset)` or `y(asset) - y(zatoshi_asset)` is
                // not equal to zero.
                // To prove that `x(asset) - x(zatoshi_asset)` (resp `y(asset) - y(zatoshi_asset)`)
                // is not equal to zero, we will prove that it is invertible.
                (
                    "(is_zatoshi_asset = 0) => (asset != zatoshi_asset)",
                    (one.clone() - is_zatoshi_asset.clone())
                        * (diff_asset_x * diff_asset_x_inv - one.clone())
                        * (diff_asset_y * diff_asset_y_inv - one.clone()),
                ),
                (
                    "(split_flag = 0) => (psi_old = psi_nf)",
                    (one.clone() - split_flag.clone()) * (psi_old - psi_nf),
                ),
                (
                    "(split_flag = 1) => (is_zatoshi_asset = 0)",
                    split_flag.clone() * is_zatoshi_asset.clone(),
                ),
                (
                    "(enable_zsa = 0) => (is_zatoshi_asset = 1)",
                    (one.clone() - enable_zsa) * (one - is_zatoshi_asset),
                ),
                // `split_flag` and `cross_address_disabled` cannot both be enabled at the
                // same time. A split note's output is forced to a negative net value, which
                // is not a meaningful self-transfer.
                (
                    "(disable_cross_address = 1) => (split_flag = 0)",
                    split_flag * disable_cross_address,
                ),
            ],
        )
    });
}

impl plonk::Circuit<pallas::Base> for CircuitZsa {
    type Config = Config<PallasLookupRangeCheck4_5BConfig>;
    type FloorPlanner = floor_planner::V1;

    fn without_witnesses(&self) -> Self {
        CircuitZsa::empty()
    }

    fn configure(meta: &mut plonk::ConstraintSystem<pallas::Base>) -> Self::Config {
        configure_circuit(meta, true)
    }

    #[allow(non_snake_case)]
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), plonk::Error> {
        if !self.common_witnesses.circuit_version.is_zsa() {
            return Err(plonk::Error::Synthesis);
        }

        // Load the Sinsemilla generator lookup table used by the whole circuit.
        SinsemillaChip::load(config.sinsemilla_config_1.clone(), &mut layouter)?;

        // Construct the ECC chip.
        let ecc_chip = config.ecc_chip(self.common_witnesses.circuit_version.halo2_version());

        // Witness private inputs that are used across multiple checks.
        let (psi_nf, psi_old, rho_old, cm_old, g_d_old, ak_P, nk, v_old, v_new, asset) = {
            // Witness psi_nf
            let psi_nf = assign_free_advice(
                layouter.namespace(|| "witness psi_nf"),
                config.advices[0],
                self.additional_zsa_witnesses.psi_nf,
            )?;

            // Witness psi_old
            let psi_old = assign_free_advice(
                layouter.namespace(|| "witness psi_old"),
                config.advices[0],
                self.common_witnesses.psi_old,
            )?;

            // Witness rho_old
            let rho_old = assign_free_advice(
                layouter.namespace(|| "witness rho_old"),
                config.advices[0],
                self.common_witnesses.rho_old.map(|rho| rho.into_inner()),
            )?;

            // Witness cm_old
            let cm_old = Point::new(
                ecc_chip.clone(),
                layouter.namespace(|| "cm_old"),
                self.common_witnesses
                    .cm_old
                    .as_ref()
                    .map(|cm| cm.inner().to_affine()),
            )?;

            // Witness g_d_old
            let g_d_old = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "gd_old"),
                self.common_witnesses
                    .g_d_old
                    .as_ref()
                    .map(|gd| gd.to_affine()),
            )?;

            // Witness ak_P.
            let ak_P: Value<pallas::Point> = self.common_witnesses.ak.as_ref().map(|ak| ak.into());
            let ak_P = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness ak_P"),
                ak_P.map(|ak_P| ak_P.to_affine()),
            )?;

            // Witness nk.
            let nk = assign_free_advice(
                layouter.namespace(|| "witness nk"),
                config.advices[0],
                self.common_witnesses.nk.map(|nk| nk.inner()),
            )?;

            // Witness v_old.
            let v_old = assign_free_advice(
                layouter.namespace(|| "witness v_old"),
                config.advices[0],
                self.common_witnesses.v_old,
            )?;

            // Witness v_new.
            let v_new = assign_free_advice(
                layouter.namespace(|| "witness v_new"),
                config.advices[0],
                self.common_witnesses.v_new,
            )?;

            // Witness asset
            let asset = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness asset"),
                self.additional_zsa_witnesses
                    .asset
                    .map(|asset| asset.cv_base().to_affine()),
            )?;

            (
                psi_nf, psi_old, rho_old, cm_old, g_d_old, ak_P, nk, v_old, v_new, asset,
            )
        };

        // Witness split_flag
        let split_flag = assign_split_flag(
            layouter.namespace(|| "witness split_flag"),
            config.advices[0],
            self.additional_zsa_witnesses.split_flag,
        )?;

        // Witness is_zatoshi_asset which is equal to
        // 1 if asset is equal to zatoshi asset, and
        // 0 if asset is not equal to zatoshi asset.
        let is_zatoshi_asset = assign_is_zatoshi_asset(
            layouter.namespace(|| "witness is_zatoshi_asset"),
            config.advices[0],
            self.additional_zsa_witnesses.asset,
        )?;

        // Merkle path validity check.
        let root = {
            let path = self
                .common_witnesses
                .path
                .map(|typed_path| typed_path.map(|node| node.inner()));
            let merkle_inputs = MerklePath::construct(
                [config.merkle_chip_1(), config.merkle_chip_2()],
                OrchardHashDomains::MerkleCrh,
                self.common_witnesses.pos,
                path,
            );
            let leaf = cm_old.extract_p().inner().clone();
            merkle_inputs.calculate_root(layouter.namespace(|| "Merkle path"), leaf)?
        };

        // Value commitment integrity.
        // See [ZIP-226: Transfer and Burn of Zcash Shielded Assets][valuecommitcorrectness] for more details.
        //
        // [valuecommitcorrectness]: https://zips.z.cash/zip-0226#value-commitment-correctness
        let v_net_magnitude_sign = {
            // Witness the magnitude and sign of v_net = v_old - v_new
            let v_net_magnitude_sign = {
                // v_net is equal to
                //   (-v_new) if split_flag = true
                //   v_old - v_new if split_flag = false
                let v_net = self
                    .additional_zsa_witnesses
                    .split_flag
                    .and_then(|split_flag| {
                        if split_flag {
                            Value::known(crate::value::NoteValue::ZERO)
                                - self.common_witnesses.v_new
                        } else {
                            self.common_witnesses.v_old - self.common_witnesses.v_new
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
                self.common_witnesses.rcv.as_ref().map(|rcv| rcv.inner()),
            )?;

            let cv_net = value_commit_orchard(
                layouter.namespace(|| "cv_net = ValueCommit^Orchard_rcv(v_net_magnitude_sign)"),
                ecc_chip.clone(),
                v_net_magnitude_sign.clone(),
                rcv,
                Some(ZsaValueCommitParams {
                    sinsemilla_chip: config.sinsemilla_chip_1(),
                    asset_base: asset.clone(),
                }),
            )?;

            // Constrain cv_net to equal public input
            layouter.constrain_instance(cv_net.inner().x().cell(), config.primary, CV_NET_X)?;
            layouter.constrain_instance(cv_net.inner().y().cell(), config.primary, CV_NET_Y)?;

            // Return the magnitude and sign so we can use them in the Orchard gate.
            v_net_magnitude_sign
        };

        // Nullifier integrity.
        // See [ZIP-226: Transfer and Burn of Zcash Shielded Assets][zip226] for more details.
        //
        // [zip226]: https://zips.z.cash/zip-0226
        let nf_old = {
            let nf_old = derive_nullifier(
                layouter.namespace(|| "nf_old = DeriveNullifier_nk(rho_old, psi_nf, cm_old)"),
                config.poseidon_chip(),
                config.add_chip(),
                ecc_chip.clone(),
                rho_old.clone(),
                &psi_nf,
                &cm_old,
                nk.clone(),
                Some(ZsaNullifierParams {
                    cond_swap_chip: config.cond_swap_chip(),
                    split_flag: split_flag.clone(),
                }),
            )?;

            // Constrain nf_old to equal public input
            layouter.constrain_instance(nf_old.inner().cell(), config.primary, NF_OLD)?;

            nf_old
        };

        // Spend authority
        {
            let alpha = ScalarFixed::new(
                ecc_chip.clone(),
                layouter.namespace(|| "alpha"),
                self.common_witnesses.alpha,
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

        // Diversified address integrity.
        let pk_d_old = {
            let ivk = {
                let ak = ak_P.extract_p().inner().clone();
                let rivk = ScalarFixed::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "rivk"),
                    self.common_witnesses.rivk.map(|rivk| rivk.inner()),
                )?;

                commit_ivk(
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
                self.common_witnesses
                    .pk_d_old
                    .map(|pk_d_old| pk_d_old.inner().to_affine()),
            )?;
            derived_pk_d_old
                .constrain_equal(layouter.namespace(|| "pk_d_old equality"), &pk_d_old)?;

            pk_d_old
        };

        // Old note commitment integrity.
        // See [ZIP-226: Transfer and Burn of Zcash Shielded Assets][notecommit] for more details.
        //
        // [notecommit]: https://zips.z.cash/zip-0226#note-structure-commitment.
        {
            let rcm_old = ScalarFixed::new(
                ecc_chip.clone(),
                layouter.namespace(|| "rcm_old"),
                self.common_witnesses
                    .rcm_old
                    .as_ref()
                    .map(|rcm_old| rcm_old.inner()),
            )?;

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let derived_cm_old = note_commit(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_1(),
                config.ecc_chip(self.common_witnesses.circuit_version.halo2_version()),
                config.note_commit_chip_old(),
                g_d_old.inner(),
                pk_d_old.inner(),
                v_old.clone(),
                rho_old,
                psi_old.clone(),
                rcm_old,
                Some(ZsaNoteCommitParams {
                    cond_swap_chip: config.cond_swap_chip(),
                    asset: asset.inner().clone(),
                    is_zatoshi_asset: is_zatoshi_asset.clone(),
                }),
            )?;

            // Constrain derived cm_old to equal witnessed cm_old
            derived_cm_old.constrain_equal(layouter.namespace(|| "cm_old equality"), &cm_old)?;
        }

        // Witness g_d_new
        let g_d_new = {
            let g_d_new = self
                .common_witnesses
                .g_d_new
                .map(|g_d_new| g_d_new.to_affine());
            NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness g_d_new_star"),
                g_d_new,
            )?
        };

        // Witness pk_d_new
        let pk_d_new = {
            let pk_d_new = self
                .common_witnesses
                .pk_d_new
                .map(|pk_d_new| pk_d_new.inner().to_affine());
            NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness pk_d_new"),
                pk_d_new,
            )?
        };

        // New note commitment integrity.
        // See [ZIP-226: Transfer and Burn of Zcash Shielded Assets][notecommit] for more details.
        //
        // [notecommit]: https://zips.z.cash/zip-0226#note-structure-commitment.
        {
            // ρ^new = nf^old
            let rho_new = nf_old.inner().clone();

            // Witness psi_new
            let psi_new = assign_free_advice(
                layouter.namespace(|| "witness psi_new"),
                config.advices[0],
                self.common_witnesses.psi_new,
            )?;

            let rcm_new = ScalarFixed::new(
                ecc_chip,
                layouter.namespace(|| "rcm_new"),
                self.common_witnesses
                    .rcm_new
                    .as_ref()
                    .map(|rcm_new| rcm_new.inner()),
            )?;

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let cm_new = note_commit(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_2(),
                config.ecc_chip(self.common_witnesses.circuit_version.halo2_version()),
                config.note_commit_chip_new(),
                g_d_new.inner(),
                pk_d_new.inner(),
                v_new.clone(),
                rho_new,
                psi_new,
                rcm_new,
                Some(ZsaNoteCommitParams {
                    cond_swap_chip: config.cond_swap_chip(),
                    asset: asset.inner().clone(),
                    is_zatoshi_asset: is_zatoshi_asset.clone(),
                }),
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
                    || "enable spend",
                    config.primary,
                    ENABLE_SPEND,
                    config.advices[6],
                    0,
                )?;

                region.assign_advice_from_instance(
                    || "enable output",
                    config.primary,
                    ENABLE_OUTPUT,
                    config.advices[7],
                    0,
                )?;

                split_flag.copy_advice(|| "split_flag", &mut region, config.advices[8], 0)?;

                is_zatoshi_asset.copy_advice(
                    || "is_zatoshi_asset",
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
                // if is_zatoshi_asset = 0, then asset != zatoshi_asset.
                region.assign_advice(
                    || "diff_asset_x_inv",
                    config.advices[2],
                    1,
                    || {
                        self.additional_zsa_witnesses.asset.map(|asset| {
                            let asset_x = *asset.cv_base().to_affine().coordinates().unwrap().x();
                            let zatoshi_asset_x = *AssetBase::zatoshi()
                                .cv_base()
                                .to_affine()
                                .coordinates()
                                .unwrap()
                                .x();

                            let diff_asset_x = asset_x - zatoshi_asset_x;

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
                        self.additional_zsa_witnesses.asset.map(|asset| {
                            let asset_y = *asset.cv_base().to_affine().coordinates().unwrap().y();
                            let zatoshi_asset_y = *AssetBase::zatoshi()
                                .cv_base()
                                .to_affine()
                                .coordinates()
                                .unwrap()
                                .y();

                            let diff_asset_y = asset_y - zatoshi_asset_y;

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

                region.assign_advice_from_instance(
                    || "disable_cross_address",
                    config.primary,
                    DISABLE_CROSS_ADDRESS,
                    config.advices[7],
                    1,
                )?;

                config.q_orchard.enable(&mut region, 0)
            },
        )?;

        let addrs = AddressPoints {
            g_d_old,
            pk_d_old,
            g_d_new,
            pk_d_new,
        };

        synthesize_cross_address_checks(&config, &mut layouter, &addrs)?;

        Ok(())
    }
}

/// Enforces the ZSA cross-address restriction for one action: when
/// `disableCrossAddress` is nonzero, the spent note and output note must be
/// addressed to the same expanded receiver, meaning equal `(g_d, pk_d)`.
///
/// This reuses the existing "Orchard circuit checks" gate instead of adding a
/// new gate. The gate already has a product constraint,
/// `(v_old + 1 - is_zatoshi_asset) * (root - anchor) = 0`, with exactly the shape needed for
/// `disableCrossAddress * (old_coord - new_coord) = 0` when `is_zatoshi_asset = 1`.
///
/// The ZSA circuit enables that gate on eight extra rows, two per affine coordinate of
/// `(g_d, pk_d)`, with:
///
/// ```text
/// First row:
/// v_old            <- disableCrossAddress
/// v_new            <- 0 (constant)
/// magnitude        <- disableCrossAddress
/// sign             <- 1 (constant)
/// root             <- old coordinate
/// anchor           <- new coordinate
/// enable_spend     <- 1 (constant)
/// enable_output    <- 1 (constant)
/// split_flag       <- 0 (constant)
/// is_zatoshi_asset <- 1 (constant)
///
/// Second row
/// asset_x          <- zatoshi_asset.x()
/// asset_y          <- zatoshi_asset.y()
/// diff_asset_x_inv <- 0 (constant)
/// diff_asset_y_inv <- 0 (constant)
/// psi_old          <- 0 (constant)
/// psi_nf           <- 0 (constant)
/// enable_zsa       <- 1 (constant)
/// disable_cross_address <- disableCrossAddress
/// ```
///
/// With this layout, the gate constraints become:
///
/// ```text
/// split_flag * (1 - split_flag) = 0
///     ->  0 * (1-0) = 0
/// v_old * (1 - split_flag) - v_new = magnitude * sign
///     -> disableCrossAddress * (1-0) - 0 = disableCrossAddress * 1
/// (v_old + 1 - is_zatoshi_asset) * (root - anchor) = 0
///     ->  (disableCrossAddress + 1 - 1) * (old_coord - new_coord) = 0
/// v_old * (1 - enable_spend) = 0
///     ->  disableCrossAddress * (1 - 1) = 0
/// v_new * (1 - enable_output) = 0
///     ->  0 * (1 - 1) = 0
/// is_zatoshi_asset * (1 - is_zatoshi_asset) = 0
///     -> 1 * (1-1) = 0
/// is_zatoshi_asset * diff_asset_x = 0
///     -> 1 * (zatoshi_asset.x() - zatoshi_asset.x()) = 0
/// is_zatoshi_asset * diff_asset_y = 0
///     -> 1 * (zatoshi_asset_y() - zatoshi_asset.y()) = 0
/// (1 - is_zatoshi_asset) * (diff_asset_x * diff_asset_x_inv - 1) * (diff_asset_y * diff_asset_y_inv - 1) = 0
///     -> (1 - 1) * (diff_asset_x * 0 - 1) * (diff_asset_y * 0 - 1) = 0
/// (1 - split_flag)*(psi_old - psi_nf) = 0
///     -> (1 - 0) * (0 - 0) = 0
/// split_flag * is_zatoshi_asset = 0
///     -> 0 * 1 = 0
/// (1 - enable_zsa) * (1 - is_zatoshi_asset) = 0
///     -> (1 - 1) * (1 - 1) = 0
/// disable_cross_address * split_flag = 0
///     -> disableCrossAddress * 0 = 0
/// ```
///
/// The second line is the actual cross-address check. Any nonzero
/// `disableCrossAddress` value forces each old coordinate to equal the
/// corresponding new coordinate. The public API encodes `disableCrossAddress`
/// as 0 or 1, but this algebra does not rely on a boolean constraint.
fn synthesize_cross_address_checks(
    config: &Config<PallasLookupRangeCheck4_5BConfig>,
    layouter: &mut impl Layouter<pallas::Base>,
    addrs: &AddressPoints<PallasLookupRangeCheck4_5BConfig>,
) -> Result<(), plonk::Error> {
    let AddressPoints {
        g_d_old,
        pk_d_old,
        g_d_new,
        pk_d_new,
    } = addrs;

    layouter.assign_region(
        || "ZSA cross-address checks",
        |mut region| {
            let coordinate_checks = [
                ("g_d x", g_d_old.inner().x(), g_d_new.inner().x()),
                ("g_d y", g_d_old.inner().y(), g_d_new.inner().y()),
                ("pk_d x", pk_d_old.inner().x(), pk_d_new.inner().x()),
                ("pk_d y", pk_d_old.inner().y(), pk_d_new.inner().y()),
            ];

            let mut offset = 0;

            for (label, old_coord, new_coord) in coordinate_checks.into_iter() {
                config.q_orchard.enable(&mut region, offset)?;

                // Copy disableCrossAddress from the public input at
                // primary[DISABLE_CROSS_ADDRESS] into advices[0] for this
                // coordinate-check row.
                let cross_address_disabled = region.assign_advice_from_instance(
                    || "v_old <- disableCrossAddress",
                    config.primary,
                    DISABLE_CROSS_ADDRESS,
                    config.advices[0],
                    offset,
                )?;

                // Fill the v_new, magnitude, sign and split_flag cells so the reused
                // value-balance constraint reads:
                // disableCrossAddress * (1-0) - 0 = disableCrossAddress * 1.
                region.assign_advice_from_constant(
                    || "v_new <- zero",
                    config.advices[1],
                    offset,
                    pallas::Base::zero(),
                )?;
                cross_address_disabled.copy_advice(
                    || "magnitude <- disableCrossAddress",
                    &mut region,
                    config.advices[2],
                    offset,
                )?;
                region.assign_advice_from_constant(
                    || "sign <- one",
                    config.advices[3],
                    offset,
                    pallas::Base::one(),
                )?;
                region.assign_advice_from_constant(
                    || "split_flag <- zero",
                    config.advices[8],
                    offset,
                    pallas::Base::zero(),
                )?;

                // Copy the old coordinate into the gate's root cell and the
                // new coordinate into its anchor cell for the equality check.
                old_coord.copy_advice(
                    || format!("root <- {label}"),
                    &mut region,
                    config.advices[4],
                    offset,
                )?;
                new_coord.copy_advice(
                    || format!("anchor <- {label}"),
                    &mut region,
                    config.advices[5],
                    offset,
                )?;

                // Set both enable flags to one so the unrelated enable checks
                // in q_orchard are neutralized on these rows.
                region.assign_advice_from_constant(
                    || "one (neutralize enable_spend check)",
                    config.advices[6],
                    offset,
                    pallas::Base::one(),
                )?;
                region.assign_advice_from_constant(
                    || "one (neutralize enable_output check)",
                    config.advices[7],
                    offset,
                    pallas::Base::one(),
                )?;

                // Set `is_zatoshi_asset` to 1.
                region.assign_advice_from_constant(
                    || "is_zatoshi_asset <- 1",
                    config.advices[9],
                    offset,
                    pallas::Base::one(),
                )?;

                // Second row
                offset += 1;

                // Set `(asset_x, asset_y) = AssetBase::zatoshi()`.
                let zatoshi_asset = AssetBase::zatoshi()
                    .cv_base()
                    .to_affine()
                    .coordinates()
                    .unwrap();
                region.assign_advice_from_constant(
                    || "asset_x <- zatoshi_asset.x()",
                    config.advices[0],
                    offset,
                    *zatoshi_asset.x(),
                )?;
                region.assign_advice_from_constant(
                    || "asset_y <- zatoshi_asset.y()",
                    config.advices[1],
                    offset,
                    *zatoshi_asset.y(),
                )?;

                // Set `diff_asset_x_inv` and `diff_asset_y_inv` to 0.
                region.assign_advice_from_constant(
                    || "diff_asset_x_inv <- 0",
                    config.advices[2],
                    offset,
                    pallas::Base::zero(),
                )?;
                region.assign_advice_from_constant(
                    || "diff_asset_y_inv <- 0",
                    config.advices[3],
                    offset,
                    pallas::Base::zero(),
                )?;

                // Set `psi_old` and `psi_nf` to 0 to satisfy the following constraint (split_flag=0):
                // (1 - split_flag)*(psi_old - psi_nf) = 0
                region.assign_advice_from_constant(
                    || "psi_old <- 0",
                    config.advices[4],
                    offset,
                    pallas::Base::zero(),
                )?;
                region.assign_advice_from_constant(
                    || "psi_nf <- 0",
                    config.advices[5],
                    offset,
                    pallas::Base::zero(),
                )?;

                // Set `enable_zsa` to 1 to satisfy the following constraint:
                // (1 - enable_zsa) * (1 - is_zatoshi_asset) = 0
                region.assign_advice_from_constant(
                    || "enable_zsa <- 1",
                    config.advices[6],
                    offset,
                    pallas::Base::one(),
                )?;

                cross_address_disabled.copy_advice(
                    || "disable_cross_address <- disableCrossAddress",
                    &mut region,
                    config.advices[7],
                    offset,
                )?;

                // Occupy the otherwise-unused rightmost advice columns so the
                // floor planner cannot lay out another region (and enable its
                // gate) on these rows.
                cross_address_disabled.copy_advice(
                    || "disableCrossAddress padding",
                    &mut region,
                    config.advices[8],
                    offset,
                )?;
                cross_address_disabled.copy_advice(
                    || "disableCrossAddress padding",
                    &mut region,
                    config.advices[9],
                    offset,
                )?;

                offset += 1;
            }

            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::iter;
    use ff::Field;
    use group::{Curve, Group, GroupEncoding};
    use halo2_proofs::{circuit::Value, dev::MockProver};
    use pasta_curves::pallas;
    use rand::rngs::OsRng;
    use rand_core::CryptoRngCore;
    use subtle::{Choice, CtOption};

    use crate::{
        builder::SpendInfo,
        bundle::Flags,
        circuit::{Circuit, Instance, Proof, ProvingKey, VerifyingKey, K},
        circuit_version::OrchardCircuitVersion,
        keys::{FullViewingKey, Scope, SpendValidatingKey, SpendingKey},
        note::{
            commitment::NoteCommitTrapdoor, AssetBase, Note, NoteCommitment, NoteVersion,
            Nullifier, RandomSeed, Rho,
        },
        primitives::redpallas::VerificationKey,
        tree::MerklePath,
        value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
    };

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_action_circuit() {
        use plotters::prelude::*;

        let root =
            BitMapBackend::new("action-circuit-layout-zsa.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Orchard Action Circuit", ("sans-serif", 60))
            .unwrap();

        let circuit = crate::circuit::CircuitZsa::empty();
        halo2_proofs::dev::CircuitLayout::default()
            .show_labels(false)
            .view_height(0..(1 << 11))
            .render(K, &circuit, &root)
            .unwrap();
    }

    /// Generates a circuit and instance whose output note is addressed to an expanded
    /// receiver distinct from the spent note's.
    fn generate_circuit_instance<R: CryptoRngCore>(
        is_zatoshi_asset: bool,
        rng: R,
    ) -> (Circuit, Instance) {
        generate_circuit_instance_inner(is_zatoshi_asset, false, false, rng)
    }

    /// Generates a circuit and instance whose output note is addressed to the spent
    /// note's expanded receiver, as the cross-address restriction requires.
    fn generate_self_transfer_circuit_instance<R: CryptoRngCore>(
        is_zatoshi_asset: bool,
        rng: R,
    ) -> (Circuit, Instance) {
        generate_circuit_instance_inner(is_zatoshi_asset, true, false, rng)
    }

    /// Generates a circuit and instance spending a split note.
    ///
    /// The asset is necessarily non-zatoshi: the circuit constrains
    /// `(split_flag = 1) => (is_zatoshi_asset = 0)`.
    fn generate_split_note_circuit_instance<R: CryptoRngCore>(rng: R) -> (Circuit, Instance) {
        generate_circuit_instance_inner(false, false, true, rng)
    }

    /// Extracts the contents of a `Value` that is known.
    fn known<V: Clone>(value: &Value<V>) -> V {
        let mut out = None;
        value.clone().map(|v| out = Some(v));
        out.expect("value is known")
    }

    fn generate_circuit_instance_inner<R: CryptoRngCore>(
        is_zatoshi_asset: bool,
        output_matches_spend: bool,
        split_flag: bool,
        mut rng: R,
    ) -> (Circuit, Instance) {
        // TODO ZSA: note_version should be NoteVersion::ZSA, once that variant exists
        // (`NoteVersion` currently only has `V2`/`V3`).
        let note_version = NoteVersion::V3;

        // Create asset
        let asset_base = if is_zatoshi_asset {
            AssetBase::zatoshi()
        } else {
            AssetBase::random(&mut rng)
        };

        let sk = SpendingKey::random(&mut rng);
        let fvk: FullViewingKey = (&sk).into();
        let sender_address = fvk.address_at(0u32, Scope::External);

        let ak: SpendValidatingKey = fvk.clone().into();
        let alpha = pallas::Scalar::random(&mut rng);
        let rk = ak.randomize(&alpha);

        // A note carrying `asset_base`, resampled until valid; `split` adds a split-note seed.
        let random_note = |recipient, value, rho: Rho, split: bool, rng: &mut R| loop {
            let rseed = RandomSeed::random(rng, &rho);
            let rseed_split_note = if split {
                CtOption::new(RandomSeed::random(rng, &rho), 1u8.into())
            } else {
                CtOption::new(rseed, 0u8.into())
            };
            let note = Note::from_parts_internal(
                recipient,
                value,
                asset_base,
                rho,
                rseed,
                rseed_split_note,
                note_version,
            );
            if note.is_some().into() {
                break note.unwrap();
            }
        };

        let rho_old = Rho::from_nf_old(Nullifier::dummy(&mut rng));
        let spent_note = random_note(
            sender_address,
            NoteValue::from_raw(7),
            rho_old,
            split_flag,
            &mut rng,
        );

        let nf_old = spent_note.nullifier(&fvk);

        let output_address = if output_matches_spend {
            sender_address
        } else {
            loop {
                let other_sk = SpendingKey::random(&mut rng);
                let other_fvk: FullViewingKey = (&other_sk).into();
                let candidate = other_fvk.address_at(0u32, Scope::External);
                if !sender_address.same_expanded_receiver(&candidate) {
                    break candidate;
                }
            }
        };
        let output_note = random_note(
            output_address,
            NoteValue::from_raw(3),
            Rho::from_nf_old(nf_old),
            false,
            &mut rng,
        );
        let cmx = output_note.commitment().into();

        // A split note contributes no value to the action: v_net = -v_new.
        let value = if split_flag {
            NoteValue::ZERO - output_note.value()
        } else {
            spent_note.value() - output_note.value()
        };
        let rcv = ValueCommitTrapdoor::random(&mut rng);
        let cv_net = ValueCommitment::derive_with_asset(value, rcv.clone(), asset_base);

        let path = MerklePath::dummy(&mut rng);
        let anchor = path.root(spent_note.commitment().into());

        let spend_info = SpendInfo {
            dummy_sk: None,
            scope: fvk.scope_for_address(&sender_address).unwrap(),
            fvk,
            note: spent_note,
            merkle_path: Some(path),
            split_flag,
        };

        (
            Circuit::from_action_context_unchecked(
                spend_info,
                output_note,
                alpha,
                rcv,
                OrchardCircuitVersion::ZSA,
            ),
            Instance {
                anchor,
                cv_net,
                nf_old,
                rk,
                cmx,
                enable_spend: true,
                enable_output: true,
                cross_address_disabled: false,
                enable_zsa: true,
            },
        )
    }

    fn random_note_commitment(mut rng: impl CryptoRngCore) -> NoteCommitment {
        NoteCommitment::derive(
            pallas::Point::random(&mut rng).to_affine().to_bytes(),
            pallas::Point::random(&mut rng).to_affine().to_bytes(),
            NoteValue::from_raw(rng.next_u64()),
            AssetBase::random(&mut rng),
            pallas::Base::random(&mut rng),
            pallas::Base::random(&mut rng),
            NoteCommitTrapdoor::new(pallas::Scalar::random(&mut rng)),
        )
        .unwrap()
    }

    /// Runs `MockProver` on the ZSA circuit for `circuit`/`instance`.
    fn zsa_mock_verify(
        circuit: &Circuit,
        instance: &Instance,
    ) -> Result<(), Vec<halo2_proofs::dev::VerifyFailure>> {
        MockProver::run(
            K,
            &circuit.to_zsa().expect("ZSA witnesses are present"),
            instance
                .to_halo2_instance()
                .iter()
                .map(|p| p.to_vec())
                .collect(),
        )
        .unwrap()
        .verify()
    }

    /// Asserts that the statement is rejected, and *only* because of `constraint`.
    fn assert_zsa_rejected_by(circuit: &Circuit, instance: &Instance, constraint: &str) {
        let failures = zsa_mock_verify(circuit, instance).expect_err("statement must be rejected");
        let unrelated: Vec<_> = failures
            .iter()
            .map(|failure| alloc::format!("{failure}"))
            .filter(|failure| !failure.contains(constraint))
            .collect();
        assert!(
            unrelated.is_empty(),
            "expected only `{constraint}` to fail, but also got: {unrelated:#?}"
        );
    }

    // Like `assert_zsa_rejected_by`, for a witness that the circuit binds several ways: a wrong
    // value breaks one constraint and cascades into the copy constraints that share the cell, so
    // `constraint` is only required to be among the failures.
    fn assert_zsa_rejects_with(circuit: &Circuit, instance: &Instance, constraint: &str) {
        let failures = zsa_mock_verify(circuit, instance).expect_err("statement must be rejected");
        assert!(
            failures
                .iter()
                .any(|failure| alloc::format!("{failure}").contains(constraint)),
            "expected `{constraint}` among the failures, but got: {failures:#?}"
        );
    }

    #[test]
    fn zsa_mock_prover_zatoshi_asset() {
        let (circuit, instance) = generate_circuit_instance(true, OsRng);
        assert_eq!(zsa_mock_verify(&circuit, &instance), Ok(()));
    }

    #[test]
    fn zsa_mock_prover_non_zatoshi_asset() {
        let (circuit, instance) = generate_circuit_instance(false, OsRng);
        assert_eq!(zsa_mock_verify(&circuit, &instance), Ok(()));
    }

    #[test]
    fn zsa_cross_address_restriction_is_conditional() {
        // An unrestricted cross-address statement is satisfiable...
        let (circuit, mut instance) = generate_circuit_instance(false, OsRng);
        assert_eq!(zsa_mock_verify(&circuit, &instance), Ok(()));

        // ...but setting `disableCrossAddress` makes it unsatisfiable...
        instance.cross_address_disabled = true;
        assert_zsa_rejected_by(
            &circuit,
            &instance,
            // The coordinate equality reuses the gate's `root = anchor` constraint, with the old and
            // new coordinates copied into those cells and `disableCrossAddress` as the multiplier.
            "(v_old = 0 and is_zatoshi_asset = 1) or (root = anchor)",
        );

        // ...while a restricted self-transfer statement is satisfiable.
        let (circuit, mut instance) = generate_self_transfer_circuit_instance(false, OsRng);
        instance.cross_address_disabled = true;
        assert_eq!(zsa_mock_verify(&circuit, &instance), Ok(()));
    }

    #[test]
    fn zsa_mock_prover_split_note() {
        let (circuit, instance) = generate_split_note_circuit_instance(OsRng);
        assert_eq!(zsa_mock_verify(&circuit, &instance), Ok(()));
    }

    #[test]
    fn zsa_mock_prover_rejects_non_split_nullifier_for_split_note() {
        let (circuit, mut instance) = generate_split_note_circuit_instance(OsRng);

        let psi_nf = known(&circuit.additional_zsa_witnesses.as_ref().unwrap().psi_nf);
        let cm_old = known(&circuit.common_witnesses.cm_old);
        let nk = known(&circuit.common_witnesses.nk);
        let rho_old = known(&circuit.common_witnesses.rho_old);

        // Replace the public nullifier by the one this note would have if it were not a split note.
        instance.nf_old =
            Nullifier::derive(&nk, rho_old.into_inner(), psi_nf, cm_old, Choice::from(0));

        // The nullifier the circuit derives failing to equal the public one.
        assert_zsa_rejected_by(&circuit, &instance, "Equality constraint not satisfied");
    }

    #[test]
    fn zsa_mock_prover_rejects_split_note_with_zatoshi_asset() {
        // The circuit constrains `(split_flag = 1) => (is_zatoshi_asset = 0)`.
        let (circuit, instance) = generate_circuit_instance_inner(true, false, true, OsRng);
        assert_zsa_rejected_by(
            &circuit,
            &instance,
            "(split_flag = 1) => (is_zatoshi_asset = 0)",
        );
    }

    #[test]
    fn zsa_mock_prover_rejects_split_note_with_cross_address_disabled() {
        // The circuit constrains `(disable_cross_address = 1) => (split_flag = 0)`.
        let (circuit, mut instance) = generate_circuit_instance_inner(false, true, true, OsRng);
        instance.cross_address_disabled = true;
        assert_zsa_rejected_by(
            &circuit,
            &instance,
            "(disable_cross_address = 1) => (split_flag = 0)",
        );
    }

    #[test]
    fn zsa_mock_prover_rejects_wrong_psi_nf() {
        let mut rng = OsRng;
        // The circuit constrains `(split_flag = 0) => (psi_old = psi_nf)`.
        let (mut circuit, instance) = generate_circuit_instance(false, &mut rng);
        circuit
            .additional_zsa_witnesses
            .as_mut()
            .expect("ZSA witnesses are present")
            .psi_nf = Value::known(pallas::Base::random(&mut rng));
        // `psi_nf` also feeds the nullifier derivation, whose copy constraint fails alongside.
        assert_zsa_rejects_with(
            &circuit,
            &instance,
            "(split_flag = 0) => (psi_old = psi_nf)",
        );
    }

    #[test]
    fn zsa_mock_prover_rejects_wrong_cm_old() {
        let mut rng = OsRng;
        let (mut circuit, instance) = generate_circuit_instance(false, &mut rng);
        circuit.common_witnesses.cm_old = Value::known(random_note_commitment(&mut rng));
        // `cm_old` is the Merkle leaf, so a wrong witness makes the computed root differ from
        // the public anchor. It also feeds the derived note commitment and `nf_old`, whose copy
        // constraints fail alongside.
        assert_zsa_rejects_with(
            &circuit,
            &instance,
            "(v_old = 0 and is_zatoshi_asset = 1) or (root = anchor)",
        );
    }

    #[test]
    fn zsa_mock_prover_rejects_zero_cv_net() {
        let (circuit, mut instance) = generate_circuit_instance(false, OsRng);
        instance.cv_net = ValueCommitment::from_bytes(&[0u8; 32]).unwrap();
        assert_zsa_rejected_by(&circuit, &instance, "Equality constraint not satisfied");
    }

    #[test]
    fn zsa_mock_prover_rejects_wrong_rk() {
        let (circuit, mut instance) = generate_circuit_instance(false, OsRng);
        instance.rk = VerificationKey::dummy();
        assert_zsa_rejected_by(&circuit, &instance, "Equality constraint not satisfied");
    }

    #[test]
    fn zsa_mock_prover_rejects_wrong_cmx() {
        let mut rng = OsRng;
        let (circuit, mut instance) = generate_circuit_instance(false, &mut rng);
        instance.cmx = random_note_commitment(&mut rng).into();
        assert_zsa_rejected_by(&circuit, &instance, "Equality constraint not satisfied");
    }

    #[test]
    fn zsa_mock_prover_rejects_wrong_nf_old() {
        let mut rng = OsRng;
        let (circuit, mut instance) = generate_circuit_instance(false, &mut rng);
        instance.nf_old = Nullifier::dummy(&mut rng);
        assert_zsa_rejected_by(&circuit, &instance, "Equality constraint not satisfied");
    }

    #[test]
    fn zsa_mock_prover_rejects_asset_mismatch() {
        // The `asset` witness must match the asset baked into `cm_old`/`cv_net`.
        let mut rng = OsRng;
        let (mut circuit, instance) = generate_circuit_instance(false, &mut rng);
        circuit.additional_zsa_witnesses = circuit.additional_zsa_witnesses.map(|mut w| {
            let current_asset = known(&w.asset);
            let another_asset = loop {
                let candidate = AssetBase::random(&mut rng);
                if candidate != current_asset {
                    break candidate;
                }
            };
            w.asset = Value::known(another_asset);
            w
        });
        // The asset is bound into `cm_old`, `cmx` and `cv_net`, so a lie about it is caught by
        // those equalities rather than by a gate constraint.
        assert_zsa_rejected_by(&circuit, &instance, "Equality constraint not satisfied");
    }

    #[test]
    fn zsa_mock_prover_rejects_enable_zsa_false_for_non_zatoshi_asset() {
        // The circuit constrains `(enable_zsa = 0) => (is_zatoshi_asset = 1)`.
        let (circuit, mut instance) = generate_circuit_instance(false, OsRng);
        instance.enable_zsa = false;
        assert_zsa_rejected_by(
            &circuit,
            &instance,
            "(enable_zsa = 0) => (is_zatoshi_asset = 1)",
        );
    }

    #[test]
    fn zsa_restricted_statement_proves_and_verifies() {
        let mut rng = OsRng;
        let (circuit, mut instance) = generate_self_transfer_circuit_instance(false, &mut rng);
        instance.cross_address_disabled = true;

        let pk = ProvingKey::build(OrchardCircuitVersion::ZSA);
        let vk = VerifyingKey::build(OrchardCircuitVersion::ZSA);

        let instances = &[instance.clone()];

        let proof = Proof::create(&pk, &[circuit], instances, &mut rng).unwrap();
        assert!(proof.verify(&vk, instances).is_ok());
    }

    #[test]
    fn zsa_prove_and_verify() {
        let mut rng = OsRng;
        let (circuits, instances): (Vec<_>, Vec<_>) = iter::once(())
            .map(|()| generate_circuit_instance(false, &mut rng))
            .unzip();

        let vk = VerifyingKey::build(OrchardCircuitVersion::ZSA);

        // Test that the proof size is as expected.
        let expected_proof_size = {
            let circuit_cost =
                halo2_proofs::dev::CircuitCost::<pasta_curves::vesta::Point, _>::measure(
                    K,
                    &circuits[0].to_zsa().expect("ZSA witnesses are present"),
                );
            assert_eq!(usize::from(circuit_cost.proof_size(1)), 5120);
            assert_eq!(usize::from(circuit_cost.proof_size(2)), 7392);
            // The constants in `Proof::expected_proof_size` must track the circuit's actual
            // proof size; this guards them against drift if the circuit ever changes.
            assert_eq!(
                Proof::expected_proof_size(OrchardCircuitVersion::ZSA, 1),
                5120
            );
            assert_eq!(
                Proof::expected_proof_size(OrchardCircuitVersion::ZSA, 2),
                7392
            );
            assert_eq!(
                Proof::expected_proof_size(OrchardCircuitVersion::ZSA, instances.len()),
                usize::from(circuit_cost.proof_size(instances.len())),
            );
            usize::from(circuit_cost.proof_size(instances.len()))
        };

        for (circuit, instance) in circuits.iter().zip(instances.iter()) {
            assert_eq!(zsa_mock_verify(circuit, instance), Ok(()));
        }

        let pk = ProvingKey::build(OrchardCircuitVersion::ZSA);
        let proof = Proof::create(&pk, &circuits, &instances, &mut rng).unwrap();
        assert!(proof.verify(&vk, &instances).is_ok());
        assert_eq!(proof.0.len(), expected_proof_size);
    }

    // Proving a ZSA circuit with a proving key for a different circuit version is a misuse: the
    // proving key and circuits must agree (see `Proof::create`). Confirm `create` rejects it with
    // `plonk::Error::Synthesis` rather than emitting an unverifiable proof.
    #[test]
    fn create_rejects_mismatched_proving_key_version() {
        let mut rng = OsRng;

        let (circuit, instance) = generate_circuit_instance(true, &mut rng);

        for pk_version in [
            OrchardCircuitVersion::InsecurePreNu6_2,
            OrchardCircuitVersion::FixedPostNu6_2,
            OrchardCircuitVersion::PostNu6_3,
        ] {
            let mismatched_pk = ProvingKey::build(pk_version);

            assert!(matches!(
                Proof::create(
                    &mismatched_pk,
                    core::slice::from_ref(&circuit),
                    core::slice::from_ref(&instance),
                    &mut rng
                ),
                Err(super::plonk::Error::Synthesis),
            ));
        }
    }

    fn write_test_case<W: std::io::Write>(
        mut w: W,
        instance: &Instance,
        proof: &Proof,
    ) -> std::io::Result<()> {
        w.write_all(&instance.anchor().to_bytes())?;
        w.write_all(&instance.cv_net().to_bytes())?;
        w.write_all(&instance.nf_old().to_bytes())?;
        w.write_all(&<[u8; 32]>::from(instance.rk()))?;
        w.write_all(&instance.cmx().to_bytes())?;
        w.write_all(&[
            u8::from(instance.enable_spend()),
            u8::from(instance.enable_output()),
            u8::from(instance.cross_address_disabled()),
            u8::from(instance.enable_zsa()),
        ])?;
        w.write_all(proof.as_ref())?;
        Ok(())
    }

    fn read_test_case<R: std::io::Read>(mut r: R) -> std::io::Result<(Instance, Proof)> {
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
        let cmx = crate::note::ExtractedNoteCommitment::from_bytes(&read_32_bytes(&mut r)).unwrap();
        let enable_spend = read_bool(&mut r);
        let enable_output = read_bool(&mut r);
        let cross_address_disable = read_bool(&mut r);
        let enable_zsa = read_bool(&mut r);
        let flags = Flags::from_parts(
            enable_spend,
            enable_output,
            !cross_address_disable,
            // TODO ZSA: add enable_zsa once Flags::from_parts has enable_zsa as a parameter
            //enable_zsa,
        );
        let mut instance = Instance::from_parts(anchor, cv_net, nf_old, rk, cmx, flags)
            .expect("test vectors were generated with non-identity rk");
        // TODO ZSA: remove this override once Flags::from_parts has enable_zsa as a parameter
        if enable_zsa {
            instance.enable_zsa = true;
        }
        let mut proof_bytes = vec![];
        r.read_to_end(&mut proof_bytes)?;
        let proof = Proof::new(proof_bytes);
        Ok((instance, proof))
    }

    #[test]
    fn serialized_proof_test_case() {
        let vk = VerifyingKey::build(OrchardCircuitVersion::ZSA);

        if std::env::var_os("ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF").is_some() {
            let create_proof = || -> std::io::Result<()> {
                let mut rng = OsRng;

                let (circuit, instance) = generate_circuit_instance(false, &mut rng);
                let instances = core::slice::from_ref(&instance);

                let pk = ProvingKey::build(OrchardCircuitVersion::ZSA);
                let proof = Proof::create(&pk, &[circuit], instances, &mut rng).unwrap();
                assert!(proof.verify(&vk, instances).is_ok());

                let file =
                    std::fs::File::create("src/circuit_data/circuit_proof_test_case_zsa.bin")?;
                write_test_case(file, &instance, &proof)
            };
            create_proof().expect("should be able to write new proof");
            // Regeneration only writes the fixture; the non-generate run below embeds and
            // verifies it.
            return;
        }

        // Parse the hardcoded proof test case.
        let (instance, proof) = {
            let test_case_bytes = include_bytes!("../circuit_data/circuit_proof_test_case_zsa.bin");
            read_test_case(&test_case_bytes[..]).expect("proof must be valid")
        };
        assert_eq!(proof.0.len(), 5120);

        assert!(proof.verify(&vk, &[instance]).is_ok());
    }

    // Set ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF to regenerate the pinned circuit description.
    #[test]
    fn zsa_pinned_circuit_description() {
        let vk = VerifyingKey::build(OrchardCircuitVersion::ZSA);

        if std::env::var_os("ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF").is_some() {
            std::fs::write(
                "src/circuit_data/circuit_description_zsa",
                format!("{:#?}\n", vk.vk.pinned()),
            )
            .expect("should be able to write new circuit description");
        } else {
            assert_eq!(
                format!("{:#?}\n", vk.vk.pinned()),
                include_str!("../circuit_data/circuit_description_zsa").replace("\r\n", "\n")
            );
        }
    }
}
