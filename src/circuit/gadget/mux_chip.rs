use halo2_gadgets::ecc::chip::EccPoint;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{self, Advice, Column, ConstraintSystem, Constraints, Expression, Selector},
    poly::Rotation,
};
use pasta_curves::pallas;

#[derive(Clone, Debug)]
pub(in crate::circuit) struct MuxConfig {
    choice: Column<Advice>,
    left: Column<Advice>,
    right: Column<Advice>,
    out: Column<Advice>,
    q_mux: Selector,
}

/// A chip implementing a multiplexer on a single row.
///
/// out = if (choice == 0) {left} else {right}
///
/// `choice` must be constrained to {0, 1} separately.
#[derive(Clone, Debug)]
pub(in crate::circuit) struct MuxChip {
    config: MuxConfig,
}

impl Chip<pallas::Base> for MuxChip {
    type Config = MuxConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl MuxChip {
    pub(in crate::circuit) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        choice: Column<Advice>,
        left: Column<Advice>,
        right: Column<Advice>,
        out: Column<Advice>,
    ) -> MuxConfig {
        let q_mux = meta.selector();
        meta.create_gate("Field element multiplexer", |meta| {
            let q_mux = meta.query_selector(q_mux);
            let choice = meta.query_advice(choice, Rotation::cur());
            let left = meta.query_advice(left, Rotation::cur());
            let right = meta.query_advice(right, Rotation::cur());
            let out = meta.query_advice(out, Rotation::cur());

            let one = Expression::Constant(pallas::Base::one());

            let should_be_zero = (one - choice.clone()) * left + choice * right - out;

            Constraints::with_selector(q_mux, Some(should_be_zero))
        });

        MuxConfig {
            choice,
            left,
            right,
            out,
            q_mux,
        }
    }

    pub(in crate::circuit) fn construct(config: MuxConfig) -> Self {
        Self { config }
    }
}

/// An instruction set for multiplexing two points.
pub(crate) trait MuxInstructions {
    /// Constraints `MUX(choice, left, right)` and returns the selected point.
    fn mux(
        &self,
        layouter: impl Layouter<pallas::Base>,
        choice: &AssignedCell<pallas::Base, pallas::Base>,
        left: &EccPoint,
        right: &EccPoint,
    ) -> Result<EccPoint, plonk::Error>;
}

impl MuxInstructions for MuxChip {
    fn mux(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        choice: &AssignedCell<pallas::Base, pallas::Base>,
        left: &EccPoint,
        right: &EccPoint,
    ) -> Result<EccPoint, plonk::Error> {
        let x_cell = layouter.assign_region(
            || "mux x",
            |mut region| {
                self.config.q_mux.enable(&mut region, 0)?;

                choice.copy_advice(|| "copy choice", &mut region, self.config.choice, 0)?;
                left.x()
                    .copy_advice(|| "copy left_x", &mut region, self.config.left, 0)?;
                right
                    .x()
                    .copy_advice(|| "copy right_x", &mut region, self.config.right, 0)?;

                let out_val = (Value::known(pallas::Base::one()) - choice.value())
                    * left.x().value()
                    + choice.value() * right.x().value();

                region.assign_advice(|| "out x", self.config.out, 0, || out_val)
            },
        )?;
        let y_cell = layouter.assign_region(
            || "mux y",
            |mut region| {
                self.config.q_mux.enable(&mut region, 0)?;

                choice.copy_advice(|| "copy choice", &mut region, self.config.choice, 0)?;
                left.y()
                    .copy_advice(|| "copy left_y", &mut region, self.config.left, 0)?;
                right
                    .y()
                    .copy_advice(|| "copy right_y", &mut region, self.config.right, 0)?;

                let out_val = (Value::known(pallas::Base::one()) - choice.value())
                    * left.y().value()
                    + choice.value() * right.y().value();

                region.assign_advice(|| "out y", self.config.out, 0, || out_val)
            },
        )?;

        Ok(EccPoint::from_coordinates_unchecked(
            x_cell.into(),
            y_cell.into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::circuit::gadget::mux_chip::{MuxChip, MuxConfig, MuxInstructions};

    use crate::{circuit::gadget::assign_free_advice, circuit::K, constants::OrchardFixedBases};
    use halo2_gadgets::{
        ecc::{
            chip::{EccChip, EccConfig},
            Point,
        },
        utilities::lookup_range_check::LookupRangeCheckConfig,
    };

    use group::{cofactor::CofactorCurveAffine, Curve, Group};
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        dev::MockProver,
        plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
    };
    use pasta_curves::arithmetic::CurveAffine;
    use pasta_curves::{pallas, EpAffine};

    use rand::rngs::OsRng;

    #[test]
    fn test_mux_chip() {
        #[derive(Clone, Debug)]
        pub struct MyConfig {
            primary: Column<Instance>,
            advice: Column<Advice>,
            mux_config: MuxConfig,
            ecc_config: EccConfig<OrchardFixedBases>,
        }
        #[derive(Default)]
        struct MyCircuit {
            left_point: Value<EpAffine>,
            right_point: Value<EpAffine>,
            choice: Value<pallas::Base>,
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = MyConfig;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self::default()
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

                // Instance column used for public inputs
                let primary = meta.instance_column();
                meta.enable_equality(primary);

                let mux_config =
                    MuxChip::configure(meta, advices[0], advices[1], advices[2], advices[3]);

                let table_idx = meta.lookup_table_column();
                let table_range_check_tag = meta.lookup_table_column();

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

                let range_check = LookupRangeCheckConfig::configure(
                    meta,
                    advices[9],
                    table_idx,
                    table_range_check_tag,
                );

                let ecc_config = EccChip::<OrchardFixedBases>::configure(
                    meta,
                    advices,
                    lagrange_coeffs,
                    range_check,
                );

                MyConfig {
                    primary,
                    advice: advices[0],
                    mux_config,
                    ecc_config,
                }
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                // Construct a MUX chip
                let mux_chip = MuxChip::construct(config.mux_config);

                // Construct an ECC chip
                let ecc_chip = EccChip::construct(config.ecc_config);

                // Assign left point
                let left_point = Point::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "left point"),
                    self.left_point.map(|left_point| left_point),
                )?;

                // Assign right point
                let right_point = Point::new(
                    ecc_chip,
                    layouter.namespace(|| "right point"),
                    self.right_point.map(|right_point| right_point),
                )?;

                // Assign choice
                let choice = assign_free_advice(
                    layouter.namespace(|| "choice"),
                    config.advice,
                    self.choice,
                )?;

                // Apply mux
                let result = mux_chip.mux(
                    layouter.namespace(|| "MUX"),
                    &choice,
                    left_point.inner(),
                    right_point.inner(),
                )?;

                // Check equality with instance
                layouter.constrain_instance(result.x().cell(), config.primary, 0)?;
                layouter.constrain_instance(result.y().cell(), config.primary, 1)
            }
        }

        // Test different circuits
        let mut circuits = vec![];
        let mut instances = vec![];
        for choice in [false, true] {
            let choice_value = if choice {
                pallas::Base::one()
            } else {
                pallas::Base::zero()
            };
            for left_point in [
                pallas::Point::identity().to_affine(),
                pallas::Point::random(OsRng).to_affine(),
            ] {
                for right_point in [
                    pallas::Point::identity().to_affine(),
                    pallas::Point::random(OsRng).to_affine(),
                ] {
                    circuits.push(MyCircuit {
                        left_point: Value::known(left_point),
                        right_point: Value::known(right_point),
                        choice: Value::known(choice_value),
                    });
                    let expected_output = if choice { right_point } else { left_point };
                    let (expected_x, expected_y) = if bool::from(expected_output.is_identity()) {
                        (pallas::Base::zero(), pallas::Base::zero())
                    } else {
                        let coords = expected_output.coordinates().unwrap();
                        (*coords.x(), *coords.y())
                    };
                    instances.push([[expected_x, expected_y]]);
                }
            }
        }

        for (circuit, instance) in circuits.iter().zip(instances.iter()) {
            let prover = MockProver::<pallas::Base>::run(
                K,
                circuit,
                instance.iter().map(|p| p.to_vec()).collect(),
            )
            .unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
