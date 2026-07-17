//! The Orchard Action circuit implementation for the Vanilla variation of the Orchard protocol.
//!
//! Includes the configuration, synthesis, and proof verification logic.

use group::Curve;

use pasta_curves::pallas;

use halo2_gadgets::{
    ecc::{chip::EccChip, FixedPoint, NonIdentityPoint, Point, ScalarFixed, ScalarVar},
    poseidon::{primitives as poseidon, Pow5Chip as PoseidonChip},
    sinsemilla::{
        chip::SinsemillaChip,
        merkle::{chip::MerkleChip, MerklePath},
    },
    utilities::lookup_range_check::{
        LookupRangeCheck, LookupRangeCheckConfig, PallasLookupRangeCheckConfig,
    },
};
use halo2_proofs::{
    circuit::{Layouter, Value},
    plonk::{self, Constraints, Expression},
    poly::Rotation,
};

use crate::{
    circuit::{
        commit_ivk::{gadgets::commit_ivk, CommitIvkChip},
        derive_nullifier::gadgets::derive_nullifier,
        gadget::{add_chip::AddChip, assign_free_advice},
        note_commit::{gadgets::note_commit, NoteCommitChip},
        value_commit_orchard::gadgets::value_commit_orchard,
        AdditionalZsaWitnesses, Config, OrchardCircuit, OrchardCircuitVersion, Witnesses, ANCHOR,
        CMX, CV_NET_X, CV_NET_Y, DISABLE_CROSS_ADDRESS, ENABLE_OUTPUT, ENABLE_SPEND, NF_OLD, RK_X,
        RK_Y,
    },
    constants::{OrchardFixedBases, OrchardFixedBasesFull, OrchardHashDomains},
    flavor::OrchardVanilla,
    note::AssetBase,
};

/// Cells carrying the addresses of an action's spent and newly created notes, returned
/// from the shared synthesis logic so that circuit versions can impose additional
/// constraints on them.
struct AddressPoints {
    g_d_old: NonIdentityPoint<pallas::Affine, EccChip<OrchardFixedBases>>,
    pk_d_old: NonIdentityPoint<pallas::Affine, EccChip<OrchardFixedBases>>,
    g_d_new: NonIdentityPoint<pallas::Affine, EccChip<OrchardFixedBases>>,
    pk_d_new: NonIdentityPoint<pallas::Affine, EccChip<OrchardFixedBases>>,
}

impl Witnesses {
    /// Synthesizes the Orchard Action checks common to every circuit version,
    /// parameterized by `self.circuit_version`, returning the cells carrying the old
    /// and new note addresses so that circuit versions can impose additional
    /// constraints on them.
    #[allow(non_snake_case)]
    fn synthesize_base(
        &self,
        config: &Config<PallasLookupRangeCheckConfig>,
        layouter: &mut impl Layouter<pallas::Base>,
    ) -> Result<AddressPoints, plonk::Error> {
        // Load the Sinsemilla generator lookup table used by the whole circuit.
        SinsemillaChip::load(config.sinsemilla_config_1.clone(), layouter)?;

        // Construct the ECC chip.
        let ecc_chip = config.ecc_chip(self.circuit_version.halo2_version());

        // Witness private inputs that are used across multiple checks.
        let (psi_old, rho_old, cm_old, g_d_old, ak_P, nk, v_old, v_new) = {
            // Witness psi_old
            let psi_old = assign_free_advice(
                layouter.namespace(|| "witness psi_old"),
                config.advices[0],
                self.psi_old,
            )?;

            // Witness rho_old
            let rho_old = assign_free_advice(
                layouter.namespace(|| "witness rho_old"),
                config.advices[0],
                self.rho_old.map(|rho| rho.into_inner()),
            )?;

            // Witness cm_old
            let cm_old = Point::new(
                ecc_chip.clone(),
                layouter.namespace(|| "cm_old"),
                self.cm_old.as_ref().map(|cm| cm.inner().to_affine()),
            )?;

            // Witness g_d_old
            let g_d_old = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "gd_old"),
                self.g_d_old.as_ref().map(|gd| gd.to_affine()),
            )?;

            // Witness ak_P.
            let ak_P: Value<pallas::Point> = self.ak.as_ref().map(|ak| ak.into());
            let ak_P = NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness ak_P"),
                ak_P.map(|ak_P| ak_P.to_affine()),
            )?;

            // Witness nk.
            let nk = assign_free_advice(
                layouter.namespace(|| "witness nk"),
                config.advices[0],
                self.nk.map(|nk| nk.inner()),
            )?;

            // Witness v_old.
            let v_old = assign_free_advice(
                layouter.namespace(|| "witness v_old"),
                config.advices[0],
                self.v_old,
            )?;

            // Witness v_new.
            let v_new = assign_free_advice(
                layouter.namespace(|| "witness v_new"),
                config.advices[0],
                self.v_new,
            )?;

            (psi_old, rho_old, cm_old, g_d_old, ak_P, nk, v_old, v_new)
        };

        // Merkle path validity check (https://p.z.cash/ZKS:action-merkle-path-validity?partial).
        let root = {
            let path = self
                .path
                .map(|typed_path| typed_path.map(|node| node.inner()));
            let merkle_inputs = MerklePath::construct(
                [config.merkle_chip_1(), config.merkle_chip_2()],
                OrchardHashDomains::MerkleCrh,
                self.pos,
                path,
            );
            let leaf = cm_old.extract_p().inner().clone();
            merkle_inputs.calculate_root(layouter.namespace(|| "Merkle path"), leaf)?
        };

        // Value commitment integrity (https://p.z.cash/ZKS:action-cv-net-integrity?partial).
        let v_net_magnitude_sign = {
            // Witness the magnitude and sign of v_net = v_old - v_new
            let v_net_magnitude_sign = {
                let v_net = self.v_old - self.v_new;
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
                self.rcv.as_ref().map(|rcv| rcv.inner()),
            )?;

            let cv_net = value_commit_orchard(
                layouter.namespace(|| "cv_net = ValueCommit^Orchard_rcv(v_net)"),
                ecc_chip.clone(),
                v_net_magnitude_sign.clone(),
                rcv,
                None,
            )?;

            // Constrain cv_net to equal public input
            layouter.constrain_instance(cv_net.inner().x().cell(), config.primary, CV_NET_X)?;
            layouter.constrain_instance(cv_net.inner().y().cell(), config.primary, CV_NET_Y)?;

            // Return the magnitude and sign so we can use them in the Orchard gate.
            v_net_magnitude_sign
        };

        // Nullifier integrity (https://p.z.cash/ZKS:action-nullifier-integrity).
        let nf_old = {
            let nf_old = derive_nullifier(
                layouter.namespace(|| "nf_old = DeriveNullifier_nk(rho_old, psi_old, cm_old)"),
                config.poseidon_chip(),
                config.add_chip(),
                ecc_chip.clone(),
                rho_old.clone(),
                &psi_old,
                &cm_old,
                nk.clone(),
                None,
            )?;

            // Constrain nf_old to equal public input
            layouter.constrain_instance(nf_old.inner().cell(), config.primary, NF_OLD)?;

            nf_old
        };

        // Spend authority (https://p.z.cash/ZKS:action-spend-authority)
        {
            let alpha =
                ScalarFixed::new(ecc_chip.clone(), layouter.namespace(|| "alpha"), self.alpha)?;

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
                    self.rivk.map(|rivk| rivk.inner()),
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
                self.pk_d_old.map(|pk_d_old| pk_d_old.inner().to_affine()),
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
                self.rcm_old.as_ref().map(|rcm_old| rcm_old.inner()),
            )?;

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let derived_cm_old = note_commit(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_1(),
                config.ecc_chip(self.circuit_version.halo2_version()),
                config.note_commit_chip_old(),
                g_d_old.inner(),
                pk_d_old.inner(),
                v_old.clone(),
                rho_old,
                psi_old,
                rcm_old,
                None,
            )?;

            // Constrain derived cm_old to equal witnessed cm_old
            derived_cm_old.constrain_equal(layouter.namespace(|| "cm_old equality"), &cm_old)?;
        }

        // Witness g_d_new, used in the new note commitment integrity check below and,
        // for the post-NU 6.3 circuit, in the cross-address checks.
        let g_d_new = {
            let g_d_new = self.g_d_new.map(|g_d_new| g_d_new.to_affine());
            NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness g_d_new_star"),
                g_d_new,
            )?
        };

        // Witness pk_d_new, used in the new note commitment integrity check below and,
        // for the post-NU 6.3 circuit, in the cross-address checks.
        let pk_d_new = {
            let pk_d_new = self.pk_d_new.map(|pk_d_new| pk_d_new.inner().to_affine());
            NonIdentityPoint::new(
                ecc_chip.clone(),
                layouter.namespace(|| "witness pk_d_new"),
                pk_d_new,
            )?
        };

        // New note commitment integrity (https://p.z.cash/ZKS:action-cmx-new-integrity?partial).
        {
            // ρ^new = nf^old
            let rho_new = nf_old.inner().clone();

            // Witness psi_new
            let psi_new = assign_free_advice(
                layouter.namespace(|| "witness psi_new"),
                config.advices[0],
                self.psi_new,
            )?;

            let rcm_new = ScalarFixed::new(
                ecc_chip,
                layouter.namespace(|| "rcm_new"),
                self.rcm_new.as_ref().map(|rcm_new| rcm_new.inner()),
            )?;

            // g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)
            let cm_new = note_commit(
                layouter.namespace(|| {
                    "g★_d || pk★_d || i2lebsp_{64}(v) || i2lebsp_{255}(rho) || i2lebsp_{255}(psi)"
                }),
                config.sinsemilla_chip_2(),
                config.ecc_chip(self.circuit_version.halo2_version()),
                config.note_commit_chip_new(),
                g_d_new.inner(),
                pk_d_new.inner(),
                v_new.clone(),
                rho_new,
                psi_new,
                rcm_new,
                None,
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

                config.q_orchard.enable(&mut region, 0)
            },
        )?;

        Ok(AddressPoints {
            g_d_old,
            pk_d_old,
            g_d_new,
            pk_d_new,
        })
    }

    /// Enforces the post-NU 6.3 cross-address restriction for one action: when
    /// `disableCrossAddress` is nonzero, the spent note and output note must be
    /// addressed to the same expanded receiver, meaning equal `(g_d, pk_d)`.
    ///
    /// This reuses the existing "Orchard circuit checks" gate instead of adding a
    /// new gate. The gate already has a product constraint,
    /// `v_old * (root - anchor) = 0`, with exactly the shape needed for
    /// `disableCrossAddress * (old_coord - new_coord) = 0`.
    ///
    /// The post-NU 6.3 circuit enables that gate on four extra rows, one per affine coordinate of
    /// `(g_d, pk_d)`, with each row wired as:
    ///
    /// ```text
    /// v_old          <- disableCrossAddress
    /// v_new          <- 0 (constant)
    /// magnitude      <- disableCrossAddress
    /// sign           <- 1 (constant)
    /// root           <- old coordinate
    /// anchor         <- new coordinate
    /// enable_spend   <- 1 (constant)
    /// enable_output  <- 1 (constant)
    /// ```
    ///
    /// With this layout, the gate constraints become:
    ///
    /// ```text
    /// v_old - v_new = magnitude * sign  ->  disableCrossAddress - 0 = disableCrossAddress * 1
    /// v_old * (root - anchor) = 0       ->  disableCrossAddress * (old_coord - new_coord) = 0
    /// v_old * (1 - enable_spend) = 0    ->  disableCrossAddress * (1 - 1) = 0
    /// v_new * (1 - enable_output) = 0   ->  0 * (1 - 1) = 0
    /// ```
    ///
    /// The second line is the actual cross-address check. Any nonzero
    /// `disableCrossAddress` value forces each old coordinate to equal the
    /// corresponding new coordinate. The public API encodes `disableCrossAddress`
    /// as 0 or 1, but this algebra does not rely on a boolean constraint.
    ///
    /// The two otherwise-unused advice columns are also filled with copies of
    /// `disableCrossAddress` so these rows occupy every advice column; that prevents
    /// the floor planner from overlapping another selector-enabled region with the
    /// check rows.
    fn synthesize_cross_address_checks(
        config: &Config<PallasLookupRangeCheckConfig>,
        layouter: &mut impl Layouter<pallas::Base>,
        addrs: &AddressPoints,
    ) -> Result<(), plonk::Error> {
        let AddressPoints {
            g_d_old,
            pk_d_old,
            g_d_new,
            pk_d_new,
        } = addrs;

        layouter.assign_region(
            || "post-NU 6.3 cross-address checks",
            |mut region| {
                let coordinate_checks = [
                    ("g_d x", g_d_old.inner().x(), g_d_new.inner().x()),
                    ("g_d y", g_d_old.inner().y(), g_d_new.inner().y()),
                    ("pk_d x", pk_d_old.inner().x(), pk_d_new.inner().x()),
                    ("pk_d y", pk_d_old.inner().y(), pk_d_new.inner().y()),
                ];

                for (offset, (label, old_coord, new_coord)) in
                    coordinate_checks.into_iter().enumerate()
                {
                    // Copy disableCrossAddress from the public input at
                    // primary[DISABLE_CROSS_ADDRESS] into advices[0] for this
                    // coordinate-check row.
                    let cross_address_disabled = region.assign_advice_from_instance(
                        || "disableCrossAddress",
                        config.primary,
                        DISABLE_CROSS_ADDRESS,
                        config.advices[0],
                        offset,
                    )?;

                    // Fill the v_new, magnitude, and sign cells so the reused
                    // value-balance constraint reads:
                    // disableCrossAddress - 0 = disableCrossAddress * 1.
                    region.assign_advice_from_constant(
                        || "zero",
                        config.advices[1],
                        offset,
                        pallas::Base::zero(),
                    )?;
                    cross_address_disabled.copy_advice(
                        || "disableCrossAddress magnitude",
                        &mut region,
                        config.advices[2],
                        offset,
                    )?;
                    region.assign_advice_from_constant(
                        || "positive sign",
                        config.advices[3],
                        offset,
                        pallas::Base::one(),
                    )?;

                    // Copy the old coordinate into the gate's root cell and the
                    // new coordinate into its anchor cell for the equality check.
                    old_coord.copy_advice(
                        || format!("old {label}"),
                        &mut region,
                        config.advices[4],
                        offset,
                    )?;
                    new_coord.copy_advice(
                        || format!("new {label}"),
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

                    config.q_orchard.enable(&mut region, offset)?;
                }

                Ok(())
            },
        )
    }
}

impl OrchardCircuit for OrchardVanilla {
    type Config = Config<PallasLookupRangeCheckConfig>;

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

        // Constrain v_old - v_new = magnitude * sign    (https://p.z.cash/ZKS:action-cv-net-integrity?partial).
        // Either v_old = 0, or calculated root = anchor (https://p.z.cash/ZKS:action-merkle-path-validity?partial).
        // Constrain v_old = 0 or enable_spend = 1       (https://p.z.cash/ZKS:action-enable-spend).
        // Constrain v_new = 0 or enable_output = 1      (https://p.z.cash/ZKS:action-enable-output).
        //
        // This gate is also reused for the same-address check; see
        // [`Circuit::synthesize_cross_address_checks`].
        let q_orchard = meta.selector();
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

            let one = Expression::Constant(pallas::Base::one());

            Constraints::with_selector(
                q_orchard,
                [
                    (
                        "v_old - v_new = magnitude * sign",
                        v_old.clone() - v_new.clone() - magnitude * sign,
                    ),
                    (
                        "Either v_old = 0, or root = anchor",
                        v_old.clone() * (root - anchor),
                    ),
                    (
                        "v_old = 0 or enable_spend = 1",
                        v_old * (one.clone() - enable_spend),
                    ),
                    (
                        "v_new = 0 or enable_output = 1",
                        v_new * (one - enable_output),
                    ),
                ],
            )
        });

        // Addition of two field elements.
        let add_config = AddChip::configure(meta, advices[7], advices[8], advices[6]);

        // Fixed columns for the Sinsemilla generator lookup table
        let table_idx = meta.lookup_table_column();
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
        let range_check = LookupRangeCheckConfig::configure(meta, advices[9], table_idx);

        // Configuration for curve point operations.
        // This uses 10 advice columns and spans the whole circuit.
        let ecc_config =
            EccChip::<OrchardFixedBases>::configure(meta, advices, lagrange_coeffs, range_check);

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
                false,
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
                false,
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
            NoteCommitChip::configure(meta, advices, sinsemilla_config_1.clone(), false);

        // Configuration to handle decomposition and canonicity checking
        // for NoteCommit_new.
        let new_note_commit_config =
            NoteCommitChip::configure(meta, advices, sinsemilla_config_2.clone(), false);

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
        }
    }

    #[allow(non_snake_case)]
    fn synthesize(
        circuit: &Witnesses,
        config: Self::Config,
        mut layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), plonk::Error> {
        assert_ne!(circuit.circuit_version, OrchardCircuitVersion::ZSA);

        let addrs = circuit.synthesize_base(&config, &mut layouter)?;

        if circuit.circuit_version.supports_cross_address_restriction() {
            Witnesses::synthesize_cross_address_checks(&config, &mut layouter, &addrs)?;
        }

        Ok(())
    }

    /// For OrchardVanilla circuits, `build_additional_zsa_witnesses` returns `Value::unknown()`.
    ///
    /// # Panics
    /// Panics if the asset is not zatoshi or if `split_flag` is true.
    fn build_additional_zsa_witnesses(
        _: pallas::Base,
        asset: AssetBase,
        split_flag: bool,
    ) -> Value<AdditionalZsaWitnesses> {
        if !(bool::from(asset.is_zatoshi())) {
            panic!("asset must be zatoshi in OrchardVanilla circuit");
        }
        if split_flag {
            panic!("split_flag must be false in OrchardVanilla circuit");
        }
        Value::unknown()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::iter;

    use ff::Field;
    use halo2_proofs::{circuit::Value, dev::MockProver};
    use pasta_curves::{pallas, vesta};
    use rand::{rngs::OsRng, RngCore};

    use crate::{
        bundle::{BundleVersion, Flags},
        circuit::{
            Circuit, Instance, OrchardCircuitVersion, Proof, ProvingKey, SingleVerifier,
            VerifyingKey, Witnesses, K,
        },
        flavor::OrchardVanilla,
        keys::SpendValidatingKey,
        note::{AssetBase, Note, NoteVersion, Rho},
        tree::MerklePath,
        value::{ValueCommitTrapdoor, ValueCommitment},
    };

    /// Generates a circuit and instance whose output note is addressed to an expanded
    /// receiver distinct from the spent note's.
    fn generate_circuit_instance<R: RngCore>(
        rng: R,
        circuit_version: OrchardCircuitVersion,
    ) -> (Circuit<OrchardVanilla>, Instance) {
        generate_circuit_instance_inner(rng, circuit_version, false)
    }

    /// Generates a circuit and instance whose output note is addressed to the spent
    /// note's expanded receiver, as the cross-address restriction requires.
    fn generate_self_transfer_circuit_instance<R: RngCore>(
        rng: R,
        circuit_version: OrchardCircuitVersion,
    ) -> (Circuit<OrchardVanilla>, Instance) {
        generate_circuit_instance_inner(rng, circuit_version, true)
    }

    fn generate_circuit_instance_inner<R: RngCore>(
        mut rng: R,
        circuit_version: OrchardCircuitVersion,
        output_matches_spend: bool,
    ) -> (Circuit<OrchardVanilla>, Instance) {
        // Note Version does not matter for this
        let note_version = NoteVersion::V2;
        let (_, fvk, spent_note) = Note::dummy(&mut rng, None, note_version);

        let sender_address = spent_note.recipient();
        let nk = *fvk.nk();
        let rivk = fvk.rivk(fvk.scope_for_address(&spent_note.recipient()).unwrap());
        let nf_old = spent_note.nullifier(&fvk);
        let rho = Rho::from_nf_old(nf_old);
        let ak: SpendValidatingKey = fvk.into();
        let alpha = pallas::Scalar::random(&mut rng);
        let rk = ak.randomize(&alpha);

        let output_note = if output_matches_spend {
            Note::new(
                sender_address,
                spent_note.value(),
                AssetBase::zatoshi(),
                rho,
                note_version,
                &mut rng,
            )
        } else {
            loop {
                let (_, _, output_note) = Note::dummy(&mut rng, Some(rho), note_version);
                if !sender_address.same_expanded_receiver(&output_note.recipient()) {
                    break output_note;
                }
            }
        };
        let cmx = output_note.commitment().into();

        let value = spent_note.value() - output_note.value();
        let rcv = ValueCommitTrapdoor::random(&mut rng);
        let cv_net = ValueCommitment::derive(value, rcv.clone(), AssetBase::zatoshi());

        let path = MerklePath::dummy(&mut rng);
        let anchor = path.root(spent_note.commitment().into());

        (
            Circuit {
                witnesses: Witnesses {
                    circuit_version,
                    path: Value::known(path.auth_path()),
                    pos: Value::known(path.position()),
                    g_d_old: Value::known(sender_address.g_d()),
                    pk_d_old: Value::known(*sender_address.pk_d()),
                    v_old: Value::known(spent_note.value()),
                    rho_old: Value::known(spent_note.rho()),
                    psi_old: Value::known(spent_note.psi()),
                    rcm_old: Value::known(spent_note.rcm()),
                    cm_old: Value::known(spent_note.commitment()),
                    alpha: Value::known(alpha),
                    ak: Value::known(ak),
                    nk: Value::known(nk),
                    rivk: Value::known(rivk),
                    g_d_new: Value::known(output_note.recipient().g_d()),
                    pk_d_new: Value::known(*output_note.recipient().pk_d()),
                    v_new: Value::known(output_note.value()),
                    psi_new: Value::known(output_note.psi()),
                    rcm_new: Value::known(output_note.rcm()),
                    rcv: Value::known(rcv),
                    additional_zsa_witnesses: Value::unknown(),
                },
                phantom: core::marker::PhantomData,
            },
            Instance {
                anchor,
                cv_net,
                nf_old,
                rk,
                cmx,
                enable_spend: true,
                enable_output: true,
                cross_address_disabled: false,
                enable_zsa: false,
            },
        )
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ProofFixtureEncoding {
        LegacyTwoFlags,
        PostNu6_3ThreeFlags,
    }

    fn write_test_case<W: std::io::Write>(
        mut w: W,
        instance: &Instance,
        proof: &Proof,
        encoding: ProofFixtureEncoding,
    ) -> std::io::Result<()> {
        w.write_all(&instance.anchor().to_bytes())?;
        w.write_all(&instance.cv_net().to_bytes())?;
        w.write_all(&instance.nf_old().to_bytes())?;
        w.write_all(&<[u8; 32]>::from(instance.rk()))?;
        w.write_all(&instance.cmx().to_bytes())?;
        match encoding {
            ProofFixtureEncoding::LegacyTwoFlags => {
                w.write_all(&[
                    u8::from(instance.enable_spend()),
                    u8::from(instance.enable_output()),
                ])?;
            }
            ProofFixtureEncoding::PostNu6_3ThreeFlags => {
                w.write_all(&[
                    u8::from(instance.enable_spend()),
                    u8::from(instance.enable_output()),
                    u8::from(instance.cross_address_disabled()),
                ])?;
            }
        }
        w.write_all(proof.as_ref())?;
        Ok(())
    }

    fn read_test_case<R: std::io::Read>(
        mut r: R,
        encoding: ProofFixtureEncoding,
    ) -> std::io::Result<(Instance, Proof)> {
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
        let (cross_address_bit, bundle_version) = match encoding {
            ProofFixtureEncoding::LegacyTwoFlags => (0, BundleVersion::orchard_v2()),
            ProofFixtureEncoding::PostNu6_3ThreeFlags => {
                // The fixture stores the instance-level *disable* bit; the NU6.3 flag
                // byte carries the *enable* bit, so invert when reconstructing.
                //
                // The circuit is pool-agnostic; decode under Ironwood, the pool whose flag
                // byte can represent `enableCrossAddress` either way (Orchard post-NU6.3
                // rejects bit 2).
                let cross_address_disabled = read_bool(&mut r);
                (
                    u8::from(!cross_address_disabled) << 2,
                    BundleVersion::ironwood_v3(),
                )
            }
        };
        let flags = Flags::from_byte(
            u8::from(enable_spend) | (u8::from(enable_output) << 1) | cross_address_bit,
            bundle_version,
        )
        .expect("test vectors use canonical flag encodings");

        let instance = Instance::from_parts(anchor, cv_net, nf_old, rk, cmx, flags)
            .expect("test vectors were generated with non-identity rk");

        let mut proof_bytes = vec![];
        r.read_to_end(&mut proof_bytes)?;
        let proof = Proof::new(proof_bytes);

        Ok((instance, proof))
    }

    #[test]
    fn halo2_instance_includes_cross_address_disabled_flag() {
        let (_, mut instance) =
            generate_circuit_instance(OsRng, OrchardCircuitVersion::FixedPostNu6_2);

        let halo2_instance = instance.to_halo2_instance();
        assert_eq!(halo2_instance[0].len(), 11);
        assert_eq!(
            halo2_instance[0][super::DISABLE_CROSS_ADDRESS],
            vesta::Scalar::zero()
        );

        instance.cross_address_disabled = true;
        assert_eq!(
            instance.to_halo2_instance()[0][super::DISABLE_CROSS_ADDRESS],
            vesta::Scalar::one()
        );
    }

    #[test]
    fn cross_address_support_matches_circuit_version() {
        assert!(!OrchardCircuitVersion::InsecurePreNu6_2.supports_cross_address_restriction());
        assert!(!OrchardCircuitVersion::FixedPostNu6_2.supports_cross_address_restriction());
        assert!(OrchardCircuitVersion::PostNu6_3.supports_cross_address_restriction());
    }

    #[test]
    fn post_nu6_3_cross_address_restriction_is_conditional() {
        let mock_verify = |circuit: &Circuit<OrchardVanilla>, instance: &Instance| {
            MockProver::run(
                K,
                circuit,
                instance
                    .to_halo2_instance()
                    .iter()
                    .map(|p| p.to_vec())
                    .collect(),
            )
            .unwrap()
            .verify()
        };

        // An unrestricted cross-address statement is satisfiable...
        let (circuit, mut instance) =
            generate_circuit_instance(OsRng, OrchardCircuitVersion::PostNu6_3);
        assert_eq!(mock_verify(&circuit, &instance), Ok(()));

        // ...but setting `disableCrossAddress` makes it unsatisfiable...
        instance.cross_address_disabled = true;
        assert!(mock_verify(&circuit, &instance).is_err());

        // ...while a restricted self-transfer statement is satisfiable.
        let (circuit, mut instance) =
            generate_self_transfer_circuit_instance(OsRng, OrchardCircuitVersion::PostNu6_3);
        instance.cross_address_disabled = true;
        assert_eq!(mock_verify(&circuit, &instance), Ok(()));
    }

    #[test]
    fn post_nu6_3_restricted_statement_proves_and_verifies() {
        let mut rng = OsRng;
        let (circuit, mut instance) =
            generate_self_transfer_circuit_instance(&mut rng, OrchardCircuitVersion::PostNu6_3);
        instance.cross_address_disabled = true;

        let pk = ProvingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);
        let vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::PostNu6_3);

        let proof = Proof::create(
            &pk,
            core::slice::from_ref(&circuit),
            core::slice::from_ref(&instance),
            &mut rng,
        )
        .unwrap();
        assert!(proof.verify(&vk, core::slice::from_ref(&instance)).is_ok());
    }

    // FixedPostNu6_2 leaves instance row 9 (`disableCrossAddress`) unconstrained, so a
    // freshly created proof can satisfy a restricted statement at the raw halo2 level
    // without enforcing anything about addresses. This test documents that hazard and
    // pins the API checks that close it.
    #[test]
    fn restricted_statement_requires_supporting_key() {
        use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite};

        let mut rng = OsRng;
        let (circuit, mut instance) =
            generate_circuit_instance(&mut rng, OrchardCircuitVersion::FixedPostNu6_2);
        instance.cross_address_disabled = true;

        let pk = ProvingKey::build::<OrchardVanilla>(OrchardCircuitVersion::FixedPostNu6_2);
        let vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::FixedPostNu6_2);

        let raw_instances = instance.to_halo2_instance();
        let raw_instances: Vec<_> = raw_instances.iter().map(|i| &i[..]).collect();
        let raw_instances = [&raw_instances[..]];

        let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);
        super::plonk::create_proof(
            &pk.params,
            &pk.pk,
            core::slice::from_ref(&circuit),
            &raw_instances,
            &mut rng,
            &mut transcript,
        )
        .unwrap();
        let proof_bytes = transcript.finalize();

        let strategy = SingleVerifier::new(&vk.params);
        let mut transcript = Blake2bRead::init(&proof_bytes[..]);
        assert!(super::plonk::verify_proof(
            &vk.params,
            &vk.vk,
            strategy,
            &raw_instances,
            &mut transcript,
        )
        .is_ok());

        assert!(matches!(
            Proof::create(
                &pk,
                core::slice::from_ref(&circuit),
                core::slice::from_ref(&instance),
                &mut rng,
            ),
            Err(super::plonk::Error::InvalidInstances),
        ));

        let proof = Proof::new(proof_bytes);
        assert!(matches!(
            proof.verify(&vk, core::slice::from_ref(&instance)),
            Err(super::plonk::Error::InvalidInstances),
        ));
    }

    // Set ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF to regenerate the pinned circuit description
    // for this version.
    fn pinned_circuit_description(
        circuit_version: OrchardCircuitVersion,
        path: &str,
        expected: &str,
    ) -> VerifyingKey {
        let vk = VerifyingKey::build::<OrchardVanilla>(circuit_version);

        if std::env::var_os("ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF").is_some() {
            std::fs::write(path, format!("{:#?}\n", vk.vk.pinned()))
                .expect("should be able to write new circuit description");
        } else {
            assert_eq!(
                format!("{:#?}\n", vk.vk.pinned()),
                expected.replace("\r\n", "\n")
            );
        }

        vk
    }

    // TODO: recast as a proptest
    fn round_trip_for_version(circuit_version: OrchardCircuitVersion, vk: &VerifyingKey) {
        let mut rng = OsRng;

        let (circuits, instances): (Vec<_>, Vec<_>) = iter::once(())
            .map(|()| generate_circuit_instance(&mut rng, circuit_version))
            .unzip();

        // Test that the proof size is as expected.
        let expected_proof_size = {
            let circuit_cost =
                halo2_proofs::dev::CircuitCost::<pasta_curves::vesta::Point, _>::measure(
                    K,
                    &circuits[0],
                );
            // These sizes are identical for every circuit version: the post-NU 6.3 circuit reuses the
            // existing Orchard checks gate on spare rows and adds no columns or
            // commitments, leaving the proof shape unchanged.
            assert_eq!(usize::from(circuit_cost.proof_size(1)), 4992);
            assert_eq!(usize::from(circuit_cost.proof_size(2)), 7264);
            // The constants in `Proof::expected_proof_size` must track the circuit's actual
            // proof size; this guards them against drift if the circuit ever changes.
            assert_eq!(Proof::expected_proof_size::<OrchardVanilla>(1), 4992);
            assert_eq!(Proof::expected_proof_size::<OrchardVanilla>(2), 7264);
            assert_eq!(
                Proof::expected_proof_size::<OrchardVanilla>(instances.len()),
                usize::from(circuit_cost.proof_size(instances.len())),
            );
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

        let pk = ProvingKey::build::<OrchardVanilla>(circuit_version);
        let proof = Proof::create(&pk, &circuits, &instances, &mut rng).unwrap();
        assert!(proof.verify(vk, &instances).is_ok());
        assert_eq!(proof.0.len(), expected_proof_size);
    }

    #[test]
    fn round_trip_fixed() {
        let vk = pinned_circuit_description(
            OrchardCircuitVersion::FixedPostNu6_2,
            "src/circuit_data/circuit_description_fixed_vanilla",
            include_str!("../circuit_data/circuit_description_fixed_vanilla"),
        );
        round_trip_for_version(OrchardCircuitVersion::FixedPostNu6_2, &vk);
    }

    #[test]
    fn round_trip_post_nu6_3() {
        let vk = pinned_circuit_description(
            OrchardCircuitVersion::PostNu6_3,
            "src/circuit_data/circuit_description_post_nu6_3",
            include_str!("../circuit_data/circuit_description_post_nu6_3"),
        );
        round_trip_for_version(OrchardCircuitVersion::PostNu6_3, &vk);
    }

    // Proves with the proving key for `proving_version` and checks that the proof verifies under
    // the verifying key for the same version, but not under a version with a different verifying
    // key.
    fn proof_is_rejected_by_other_circuit_version(
        proving_version: OrchardCircuitVersion,
        other_version: OrchardCircuitVersion,
    ) {
        let mut rng = OsRng;

        let (circuit, instance) = generate_circuit_instance(&mut rng, proving_version);
        let instances = core::slice::from_ref(&instance);

        let pk = ProvingKey::build::<OrchardVanilla>(proving_version);
        let proof = Proof::create(&pk, &[circuit], instances, &mut rng).unwrap();

        // Verifies under the matching version's verifying key.
        let vk_matching = VerifyingKey::build::<OrchardVanilla>(proving_version);
        assert!(proof.verify(&vk_matching, instances).is_ok());

        // Does not verify under the other version's verifying key.
        let vk_other = VerifyingKey::build::<OrchardVanilla>(other_version);
        assert!(proof.verify(&vk_other, instances).is_err());
    }

    #[test]
    fn proof_verifies_against_matching_circuit_version() {
        // Insecure proofs are rejected by the anchored circuit versions, and anchored proofs are
        // rejected by the insecure verifying key.
        proof_is_rejected_by_other_circuit_version(
            OrchardCircuitVersion::FixedPostNu6_2,
            OrchardCircuitVersion::InsecurePreNu6_2,
        );
        proof_is_rejected_by_other_circuit_version(
            OrchardCircuitVersion::PostNu6_3,
            OrchardCircuitVersion::InsecurePreNu6_2,
        );
        proof_is_rejected_by_other_circuit_version(
            OrchardCircuitVersion::InsecurePreNu6_2,
            OrchardCircuitVersion::FixedPostNu6_2,
        );
        proof_is_rejected_by_other_circuit_version(
            OrchardCircuitVersion::InsecurePreNu6_2,
            OrchardCircuitVersion::PostNu6_3,
        );
    }

    #[test]
    fn fixed_and_post_nu6_3_have_distinct_verifying_keys() {
        proof_is_rejected_by_other_circuit_version(
            OrchardCircuitVersion::FixedPostNu6_2,
            OrchardCircuitVersion::PostNu6_3,
        );
        proof_is_rejected_by_other_circuit_version(
            OrchardCircuitVersion::PostNu6_3,
            OrchardCircuitVersion::FixedPostNu6_2,
        );
    }

    // Proving a circuit with a proving key for a different circuit version is a misuse: the
    // proving key and circuits must agree (see `Proof::create`). Confirm `create` rejects it
    // with `plonk::Error::Synthesis` rather than emitting an unverifiable proof.
    #[test]
    fn create_rejects_mismatched_proving_key_version() {
        let mut rng = OsRng;

        for (circuit_version, pk_version) in [
            (
                OrchardCircuitVersion::InsecurePreNu6_2,
                OrchardCircuitVersion::FixedPostNu6_2,
            ),
            (
                OrchardCircuitVersion::FixedPostNu6_2,
                OrchardCircuitVersion::PostNu6_3,
            ),
            (
                OrchardCircuitVersion::PostNu6_3,
                OrchardCircuitVersion::FixedPostNu6_2,
            ),
        ] {
            let (circuit, instance) = generate_circuit_instance(&mut rng, circuit_version);
            let instances = core::slice::from_ref(&instance);

            let mismatched_pk = ProvingKey::build::<OrchardVanilla>(pk_version);

            assert!(matches!(
                Proof::create(&mismatched_pk, &[circuit], instances, &mut rng),
                Err(super::plonk::Error::Synthesis),
            ));
        }
    }

    fn serialized_proof_test_case_for_version(
        circuit_version: OrchardCircuitVersion,
        proof_path: &str,
        test_case_bytes: &[u8],
        encoding: ProofFixtureEncoding,
        expected_proof_size: usize,
        restricted: bool,
    ) {
        let vk = VerifyingKey::build::<OrchardVanilla>(circuit_version);
        // Set ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF to regenerate this serialized proof
        // fixture. The non-regeneration path embeds and verifies the checked-in fixture.
        if std::env::var_os("ORCHARD_CIRCUIT_TEST_GENERATE_NEW_PROOF").is_some() {
            let create_proof = || -> std::io::Result<()> {
                let mut rng = OsRng;

                let (circuit, mut instance) = if restricted {
                    generate_self_transfer_circuit_instance(&mut rng, circuit_version)
                } else {
                    generate_circuit_instance(&mut rng, circuit_version)
                };
                instance.cross_address_disabled = restricted;
                let instances = core::slice::from_ref(&instance);

                let pk = ProvingKey::build::<OrchardVanilla>(circuit_version);
                let proof = Proof::create(&pk, &[circuit], instances, &mut rng).unwrap();
                assert!(proof.verify(&vk, instances).is_ok());

                let file = std::fs::File::create(proof_path)?;
                write_test_case(file, &instance, &proof, encoding)
            };
            create_proof().expect("should be able to write new proof");
            // Regeneration only writes the fixture; the non-generate run below embeds and
            // verifies it.
            return;
        }

        // Parse the hardcoded proof test case.
        let (instance, proof) =
            read_test_case(test_case_bytes, encoding).expect("proof must be valid");
        assert_eq!(instance.cross_address_disabled(), restricted);
        assert_eq!(proof.0.len(), expected_proof_size);

        assert!(proof.verify(&vk, &[instance]).is_ok());
    }

    #[test]
    fn serialized_fixed_proof_test_case() {
        serialized_proof_test_case_for_version(
            OrchardCircuitVersion::FixedPostNu6_2,
            "src/circuit_data/circuit_proof_test_case_fixed_vanilla.bin",
            include_bytes!("../circuit_data/circuit_proof_test_case_fixed_vanilla.bin"),
            ProofFixtureEncoding::LegacyTwoFlags,
            4992,
            false,
        );
    }

    #[test]
    fn serialized_post_nu6_3_proof_test_case() {
        serialized_proof_test_case_for_version(
            OrchardCircuitVersion::PostNu6_3,
            "src/circuit_data/circuit_proof_test_case_post_nu6_3.bin",
            include_bytes!("../circuit_data/circuit_proof_test_case_post_nu6_3.bin"),
            ProofFixtureEncoding::PostNu6_3ThreeFlags,
            4992,
            false,
        );
    }

    #[test]
    fn serialized_post_nu6_3_restricted_proof_test_case() {
        serialized_proof_test_case_for_version(
            OrchardCircuitVersion::PostNu6_3,
            "src/circuit_data/circuit_proof_test_case_post_nu6_3_restricted.bin",
            include_bytes!("../circuit_data/circuit_proof_test_case_post_nu6_3_restricted.bin"),
            ProofFixtureEncoding::PostNu6_3ThreeFlags,
            4992,
            true,
        );
    }

    // The deployed (NU5..NU6.2) verifying key and a pre-fix proof. `InsecurePreNu6_2`
    // reconstructs the historical circuit, so this checks that the deployed verifying key is
    // reproduced exactly and that the old proof still verifies under it — the guarantee that
    // lets a node sync from before the fix. These fixtures are frozen as the canonical
    // pre-NU6.2 verifying key and a sample proof, so they are never regenerated.
    #[test]
    fn insecure_against_stored_circuit() {
        let vk = VerifyingKey::build::<OrchardVanilla>(OrchardCircuitVersion::InsecurePreNu6_2);
        assert_eq!(
            format!("{:#?}\n", vk.vk.pinned()),
            include_str!("../circuit_data/circuit_description_insecure_vanilla").replace("\r\n", "\n")
        );

        let (instance, proof) = {
            let test_case_bytes =
                include_bytes!("../circuit_data/circuit_proof_test_case_insecure_vanilla.bin");
            read_test_case(&test_case_bytes[..], ProofFixtureEncoding::LegacyTwoFlags)
                .expect("proof must be valid")
        };
        assert_eq!(proof.0.len(), 4992);
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

        let circuit = Circuit::<OrchardVanilla>::empty(OrchardCircuitVersion::FixedPostNu6_2);
        halo2_proofs::dev::CircuitLayout::default()
            .show_labels(false)
            .view_height(0..(1 << 11))
            .render(K, &circuit, &root)
            .unwrap();
    }
}
