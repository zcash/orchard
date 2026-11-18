//! Derive nullifier logic for the Orchard circuit.

use crate::constants::nullifier_l::nullifier_l;
use halo2_gadgets::utilities::cond_swap::CondSwapChip;
use halo2_proofs::circuit::AssignedCell;
use pasta_curves::pallas;

/// Parameters needed to derive a nullifier for a split note.
#[derive(Debug)]
#[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
pub(super) struct ZsaNullifierParams {
    pub(super) cond_swap_chip: CondSwapChip<pallas::Base>,
    pub(super) split_flag: AssignedCell<pallas::Base, pallas::Base>,
}

/// Gadget functions for `DeriveNullifier` operations.
#[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
pub(in crate::circuit) mod gadgets {
    use super::*;

    use crate::{
        circuit::gadget::AddInstruction,
        constants::{NullifierK, OrchardFixedBases},
    };
    use halo2_gadgets::{
        ecc::{chip::EccPoint, EccInstructions, FixedPointBaseField, Point, X},
        poseidon::{
            primitives::{self as poseidon, ConstantLength},
            Hash as PoseidonHash, PoseidonSpongeInstructions,
        },
    };
    use halo2_proofs::{circuit::Layouter, plonk};

    /// `DeriveNullifier` from [Section 4.16: Note Commitments and Nullifiers].
    ///
    /// [Section 4.16: Note Commitments and Nullifiers]: https://zips.z.cash/protocol/protocol.pdf#commitmentsandnullifiers
    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]
    pub(in crate::circuit) fn derive_nullifier<
        PoseidonChip: PoseidonSpongeInstructions<pallas::Base, poseidon::P128Pow5T3, ConstantLength<2>, 3, 2>,
        AddChip: AddInstruction<pallas::Base>,
        EccChip: EccInstructions<
            pallas::Affine,
            FixedPoints = OrchardFixedBases,
            Point = EccPoint,
            Var = AssignedCell<pallas::Base, pallas::Base>,
        >,
    >(
        mut layouter: impl Layouter<pallas::Base>,
        poseidon_chip: PoseidonChip,
        add_chip: AddChip,
        ecc_chip: EccChip,
        rho: AssignedCell<pallas::Base, pallas::Base>,
        psi: &AssignedCell<pallas::Base, pallas::Base>,
        cm: &Point<pallas::Affine, EccChip>,
        nk: AssignedCell<pallas::Base, pallas::Base>,
        zsa_params: Option<ZsaNullifierParams>,
    ) -> Result<X<pallas::Affine, EccChip>, plonk::Error> {
        // hash = poseidon_hash(nk, rho)
        let hash = {
            let poseidon_message = [nk, rho];
            let poseidon_hasher =
                PoseidonHash::init(poseidon_chip, layouter.namespace(|| "Poseidon init"))?;
            poseidon_hasher.hash(
                layouter.namespace(|| "Poseidon hash (nk, rho)"),
                poseidon_message,
            )?
        };

        // Add hash output to psi.
        // `scalar` = poseidon_hash(nk, rho) + psi.
        let scalar = add_chip.add(
            layouter.namespace(|| "scalar = poseidon_hash(nk, rho) + psi"),
            &hash,
            psi,
        )?;

        // Multiply scalar by NullifierK
        // `product` = [poseidon_hash(nk, rho) + psi] NullifierK.
        let product = {
            let nullifier_k = FixedPointBaseField::from_inner(ecc_chip.clone(), NullifierK.into());
            nullifier_k.mul(
                layouter.namespace(|| "[poseidon_output + psi] NullifierK"),
                scalar,
            )?
        };

        // Add cm to multiplied fixed base
        // nf = cm + [poseidon_output + psi] NullifierK
        let nf = cm.add(layouter.namespace(|| "nf"), &product)?;

        match zsa_params {
            None => Ok(nf.extract_p()),
            Some(zsa_params) => {
                // Add NullifierL to nf
                // split_note_nf = NullifierL + nf
                let nullifier_l = Point::new_from_constant(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness NullifierL constant"),
                    nullifier_l(),
                )?;
                let split_note_nf = nullifier_l.add(layouter.namespace(|| "split_note_nf"), &nf)?;

                // Select the desired nullifier according to split_flag
                Ok(Point::from_inner(
                    ecc_chip,
                    zsa_params.cond_swap_chip.mux_on_points(
                        layouter.namespace(|| "mux on nf"),
                        &zsa_params.split_flag,
                        nf.inner(),
                        split_note_nf.inner(),
                    )?,
                )
                .extract_p())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        circuit::{
            derive_nullifier::{gadgets::derive_nullifier, ZsaNullifierParams},
            gadget::{
                add_chip::{AddChip, AddConfig},
                assign_free_advice, assign_split_flag,
            },
            K,
        },
        constants::{OrchardCommitDomains, OrchardFixedBases, OrchardHashDomains},
        keys::NullifierDerivingKey,
        note::{commitment::NoteCommitment, Note, NoteVersion, Nullifier},
    };
    use halo2_gadgets::{
        ecc::{
            chip::{CircuitVersion, EccChip, EccConfig},
            Point,
        },
        poseidon::{primitives as poseidon, Pow5Chip, Pow5Config},
        sinsemilla::chip::{SinsemillaChip, SinsemillaConfig},
        utilities::{
            cond_swap::{CondSwapChip, CondSwapConfig},
            lookup_range_check::{LookupRangeCheck4_5BConfig, PallasLookupRangeCheck4_5BConfig},
        },
    };

    use group::Curve;
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        dev::MockProver,
        plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
    };
    use pasta_curves::pallas;
    use rand::rngs::OsRng;
    use subtle::Choice;

    /// Checks that the `derive_nullifier` gadget agrees with `Nullifier::derive`.
    #[test]
    fn test_derive_nullifier() {
        #[derive(Clone, Debug)]
        struct MyConfig {
            primary: Column<Instance>,
            advices: [Column<Advice>; 10],
            add_config: AddConfig,
            ecc_config: EccConfig<OrchardFixedBases, PallasLookupRangeCheck4_5BConfig>,
            poseidon_config: Pow5Config<pallas::Base, 3, 2>,
            cond_swap_config: CondSwapConfig,
            // The Sinsemilla config is only used to initialize the table_idx lookup table in
            // the same way as in the Orchard circuit.
            sinsemilla_config: SinsemillaConfig<
                OrchardHashDomains,
                OrchardCommitDomains,
                OrchardFixedBases,
                PallasLookupRangeCheck4_5BConfig,
            >,
        }

        #[derive(Default)]
        struct MyCircuit {
            nk: Value<NullifierDerivingKey>,
            rho: Value<pallas::Base>,
            psi: Value<pallas::Base>,
            cm: Value<NoteCommitment>,
            // When `Some`, call the gadget with `ZsaNullifierParams` carrying this split_flag.
            split_flag: Option<bool>,
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = MyConfig;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self {
                    split_flag: self.split_flag,
                    ..Default::default()
                }
            }

            fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
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

                for advice in advices.iter() {
                    meta.enable_equality(*advice);
                }

                let primary = meta.instance_column();
                meta.enable_equality(primary);

                let table_idx = meta.lookup_table_column();
                let table_range_check_tag = meta.lookup_table_column();
                let lookup = (
                    table_idx,
                    meta.lookup_table_column(),
                    meta.lookup_table_column(),
                );

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
                meta.enable_constant(lagrange_coeffs[0]);

                let range_check = LookupRangeCheck4_5BConfig::configure_with_tag(
                    meta,
                    advices[9],
                    table_idx,
                    table_range_check_tag,
                );

                let sinsemilla_config = SinsemillaChip::configure(
                    meta,
                    advices[..5].try_into().unwrap(),
                    advices[6],
                    lagrange_coeffs[0],
                    lookup,
                    range_check,
                    true,
                );

                // Laid out exactly as in the Orchard circuit.
                let add_config = AddChip::configure(meta, advices[7], advices[8], advices[6]);
                let poseidon_config = Pow5Chip::configure::<poseidon::P128Pow5T3>(
                    meta,
                    advices[6..9].try_into().unwrap(),
                    advices[5],
                    lagrange_coeffs[2..5].try_into().unwrap(),
                    lagrange_coeffs[5..8].try_into().unwrap(),
                );
                let cond_swap_config =
                    CondSwapChip::configure(meta, advices[..5].try_into().unwrap());

                MyConfig {
                    primary,
                    advices,
                    add_config,
                    ecc_config:
                        EccChip::<OrchardFixedBases, PallasLookupRangeCheck4_5BConfig>::configure(
                            meta,
                            advices,
                            lagrange_coeffs,
                            range_check,
                        ),
                    poseidon_config,
                    cond_swap_config,
                    sinsemilla_config,
                }
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                // Load the Sinsemilla generator lookup table.
                SinsemillaChip::load(config.sinsemilla_config.clone(), &mut layouter)?;

                let ecc_chip = EccChip::construct(config.ecc_config, CircuitVersion::AnchoredBase);

                let nk = assign_free_advice(
                    layouter.namespace(|| "witness nk"),
                    config.advices[0],
                    self.nk.map(|nk| nk.inner()),
                )?;
                let rho = assign_free_advice(
                    layouter.namespace(|| "witness rho"),
                    config.advices[0],
                    self.rho,
                )?;
                let psi = assign_free_advice(
                    layouter.namespace(|| "witness psi"),
                    config.advices[0],
                    self.psi,
                )?;
                let cm = Point::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness cm"),
                    self.cm.as_ref().map(|cm| cm.inner().to_affine()),
                )?;

                // Build the gadget's `zsa_params` from `self.split_flag`:
                // - `None`        -> `zsa_params = None`
                // - `Some(false)` -> `zsa_params = Some(..)` with `split_flag = 0`
                // - `Some(true)`  -> `zsa_params = Some(..)` with `split_flag = 1`
                let zsa_params = self
                    .split_flag
                    .map(|split_flag| {
                        let split_flag = assign_split_flag(
                            layouter.namespace(|| "witness split_flag"),
                            config.advices[0],
                            Value::known(split_flag),
                        )?;
                        Ok::<_, Error>(ZsaNullifierParams {
                            cond_swap_chip: CondSwapChip::construct(
                                config.cond_swap_config.clone(),
                            ),
                            split_flag,
                        })
                    })
                    .transpose()?;

                let nf = derive_nullifier(
                    layouter.namespace(|| "nf = DeriveNullifier_nk(rho, psi, cm)"),
                    Pow5Chip::construct(config.poseidon_config.clone()),
                    AddChip::construct(config.add_config.clone()),
                    ecc_chip,
                    rho,
                    &psi,
                    &cm,
                    nk,
                    zsa_params,
                )?;

                layouter.constrain_instance(nf.inner().cell(), config.primary, 0)
            }
        }

        let mut rng = OsRng;
        let (_, fvk, note) = Note::dummy(&mut rng, None, NoteVersion::V3);

        for split_flag in [None, Some(false), Some(true)] {
            // `split_flag` drives both the circuit and the expected value:
            // - `None`        -> `zsa_params = None`
            // - `Some(false)` -> `zsa_params = Some(..)` with `split_flag = 0`
            // - `Some(true)`  -> `zsa_params = Some(..)` with `split_flag = 1`
            let expected_nf = Nullifier::derive(
                fvk.nk(),
                note.rho().into_inner(),
                note.psi(),
                note.commitment(),
                Choice::from(u8::from(split_flag.unwrap_or(false))),
            );

            let circuit = MyCircuit {
                nk: Value::known(*fvk.nk()),
                rho: Value::known(note.rho().into_inner()),
                psi: Value::known(note.psi()),
                cm: Value::known(note.commitment()),
                split_flag,
            };

            let prover =
                MockProver::<pallas::Base>::run(K, &circuit, vec![vec![expected_nf.inner()]])
                    .unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
