use halo2::{
    circuit::{Cell, Chip, Layouter},
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};
use pasta_curves::arithmetic::FieldExt;
use std::marker::PhantomData;

trait CondSwapInstructions<F: FieldExt>: Chip<F> {
    /// Variable representing an (x,y) pair to be conditionally swapped.
    type Pair;

    /// Variable representing a `swap` boolean flag.
    type Swap;

    /// Load a pair (x,y) and a `swap` boolean flag.
    fn load_inputs(
        &self,
        layouter: impl Layouter<F>,
        pair: (Option<F>, Option<F>),
        swap: Option<bool>,
    ) -> Result<(Self::Pair, Self::Swap), Error>;

    /// Given an input pair (x,y) and a `swap` boolean flag, return
    /// (y,x) if `swap` is set, else (x,y) if `swap` is not set.
    fn swap(
        &self,
        layouter: impl Layouter<F>,
        pair: Self::Pair,
        swap: Self::Swap,
    ) -> Result<Self::Pair, Error>;
}

/// A chip implementing a conditional swap.
#[derive(Clone, Debug)]
pub struct CondSwapChip<F> {
    config: CondSwapConfig,
    _marker: PhantomData<F>,
}

#[derive(Clone, Debug)]
pub struct CondSwapConfig {
    q_swap: Selector,
    x: Column<Advice>,
    y: Column<Advice>,
    x_swapped: Column<Advice>,
    y_swapped: Column<Advice>,
    swap: Column<Advice>,
    perm: Permutation,
}

impl<F: FieldExt> CondSwapChip<F> {
    /// Configures this chip for use in a circuit.
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        x: Column<Advice>,
        y: Column<Advice>,
        x_swapped: Column<Advice>,
        y_swapped: Column<Advice>,
        swap: Column<Advice>,
    ) -> CondSwapConfig {
        let q_swap = meta.selector();
        let perm = meta.permutation(&[x.into(), y.into(), swap.into()]);

        let config = CondSwapConfig {
            q_swap,
            x,
            y,
            x_swapped,
            y_swapped,
            swap,
            perm,
        };

        let q_swap = meta.query_selector(q_swap, Rotation::cur());

        let x = meta.query_advice(x, Rotation::cur());
        let y = meta.query_advice(y, Rotation::cur());
        let x_swapped = meta.query_advice(x_swapped, Rotation::cur());
        let y_swapped = meta.query_advice(y_swapped, Rotation::cur());
        let swap = meta.query_advice(swap, Rotation::cur());

        let one = Expression::Constant(F::one());

        // TODO: optimise shape of gate for Merkle path validation

        // x_swapped - y ⋅ swap - x ⋅ (1-swap) = 0
        // This checks that `x_swapped` is equal to `y` when `swap` is set,
        // but remains as `x` when `swap` is not set.
        meta.create_gate("x' = y ⋅ swap + x ⋅ (1-swap)", |_| {
            q_swap.clone()
                * (x_swapped - y.clone() * swap.clone() - x.clone() * (one.clone() - swap.clone()))
        });

        // y_swapped - x ⋅ swap - y ⋅ (1-swap) = 0
        // This checks that `y_swapped` is equal to `x` when `swap` is set,
        // but remains as `y` when `swap` is not set.
        meta.create_gate("y' = x ⋅ swap + y ⋅ (1-swap)", |_| {
            q_swap.clone() * (y_swapped - x * swap.clone() - y * (one.clone() - swap.clone()))
        });

        // Check `swap` is boolean.
        meta.create_gate("boolean check", |_| q_swap * swap.clone() * (one - swap));

        config
    }

    pub fn construct(config: CondSwapConfig) -> Self {
        CondSwapChip {
            config,
            _marker: PhantomData,
        }
    }
}

impl<F: FieldExt> Chip<F> for CondSwapChip<F> {
    type Config = CondSwapConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

/// A pair (x,y) to be conditionally swapped.
#[derive(Copy, Clone)]
struct Pair<F: FieldExt> {
    x: Var<F>,
    y: Var<F>,
}

/// A variable representing a number.
#[derive(Copy, Clone)]
struct Var<F: FieldExt> {
    cell: Cell,
    value: Option<F>,
}

/// A variable representing a `swap` boolean flag.
#[derive(Copy, Clone)]
struct Swap {
    cell: Cell,
    value: Option<bool>,
}

impl<F: FieldExt> CondSwapInstructions<F> for CondSwapChip<F> {
    type Pair = Pair<F>;
    type Swap = Swap;

    fn load_inputs(
        &self,
        mut layouter: impl Layouter<F>,
        pair: (Option<F>, Option<F>),
        swap: Option<bool>,
    ) -> Result<(Self::Pair, Self::Swap), Error> {
        let config = self.config();

        let x = pair.0;
        let y = pair.1;

        layouter.assign_region(
            || "load inputs",
            |mut region| {
                // Witness `x`
                let x_cell =
                    region.assign_advice(|| "x", config.x, 0, || x.ok_or(Error::SynthesisError))?;
                let x = Var {
                    cell: x_cell,
                    value: x,
                };

                // Witness `y`
                let y_cell =
                    region.assign_advice(|| "y", config.y, 0, || y.ok_or(Error::SynthesisError))?;
                let y = Var {
                    cell: y_cell,
                    value: y,
                };

                let swap_cell = region.assign_advice(
                    || "swap",
                    config.swap,
                    0,
                    || {
                        swap.map(|swap| F::from_u64(swap as u64))
                            .ok_or(Error::SynthesisError)
                    },
                )?;

                Ok((
                    Pair { x, y },
                    Swap {
                        cell: swap_cell,
                        value: swap,
                    },
                ))
            },
        )
    }

    fn swap(
        &self,
        mut layouter: impl Layouter<F>,
        pair: Self::Pair,
        swap: Self::Swap,
    ) -> Result<Self::Pair, Error> {
        let config = self.config();

        layouter.assign_region(
            || "swap",
            |mut region| {
                // Enable `q_swap` selector
                config.q_swap.enable(&mut region, 0)?;

                // Copy in `x` value
                let x = pair.x.value;
                let x_cell =
                    region.assign_advice(|| "x", config.x, 0, || x.ok_or(Error::SynthesisError))?;
                region.constrain_equal(&config.perm, x_cell, pair.x.cell)?;

                // Copy in `y` value
                let y = pair.y.value;
                let y_cell =
                    region.assign_advice(|| "y", config.y, 0, || y.ok_or(Error::SynthesisError))?;
                region.constrain_equal(&config.perm, y_cell, pair.y.cell)?;

                // Copy in `swap` value
                let swap_val = swap.value;
                let swap_cell = region.assign_advice(
                    || "swap",
                    config.swap,
                    0,
                    || {
                        swap_val
                            .map(|swap| F::from_u64(swap as u64))
                            .ok_or(Error::SynthesisError)
                    },
                )?;
                region.constrain_equal(&config.perm, swap_cell, swap.cell)?;

                // Conditionally swap x
                let x_swapped = {
                    let x_swapped = x
                        .zip(y)
                        .zip(swap_val)
                        .map(|((x, y), swap)| if swap { y } else { x });
                    let x_swapped_cell = region.assign_advice(
                        || "x_swapped",
                        config.x_swapped,
                        0,
                        || x_swapped.ok_or(Error::SynthesisError),
                    )?;
                    Var {
                        cell: x_swapped_cell,
                        value: x_swapped,
                    }
                };

                // Conditionally swap y
                let y_swapped = {
                    let y_swapped = x
                        .zip(y)
                        .zip(swap_val)
                        .map(|((x, y), swap)| if swap { x } else { y });
                    let y_swapped_cell = region.assign_advice(
                        || "y_swapped",
                        config.y_swapped,
                        0,
                        || y_swapped.ok_or(Error::SynthesisError),
                    )?;
                    Var {
                        cell: y_swapped_cell,
                        value: y_swapped,
                    }
                };

                // Return swapped pair
                Ok(Pair {
                    x: x_swapped,
                    y: y_swapped,
                })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CondSwapChip, CondSwapConfig, CondSwapInstructions};
    use halo2::{
        circuit::{layouter::SingleChipLayouter, Layouter},
        dev::MockProver,
        plonk::{Assignment, Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas::Base};

    #[test]
    fn cond_swap() {
        struct MyCircuit<F: FieldExt> {
            x: Option<F>,
            y: Option<F>,
            swap: Option<bool>,
        }

        impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
            type Config = CondSwapConfig;

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                let x = meta.advice_column();
                let y = meta.advice_column();
                let x_swapped = meta.advice_column();
                let y_swapped = meta.advice_column();
                let swap = meta.advice_column();

                CondSwapChip::<F>::configure(meta, x, y, x_swapped, y_swapped, swap)
            }

            fn synthesize(
                &self,
                cs: &mut impl Assignment<F>,
                config: Self::Config,
            ) -> Result<(), Error> {
                let mut layouter = SingleChipLayouter::new(cs)?;
                let chip = CondSwapChip::<F>::construct(config);

                // Load the pair and the swap flag into the circuit.
                let (pair, swap) = chip.load_inputs(
                    layouter.namespace(|| "load inputs"),
                    (self.x, self.y),
                    self.swap,
                )?;

                // Return the swapped pair.
                let swapped_pair = chip.swap(layouter.namespace(|| "swap"), pair, swap)?;

                if let Some(swap) = self.swap {
                    if swap {
                        // Check that `x` and `y` have been swapped
                        assert_eq!(swapped_pair.x.value.unwrap(), pair.y.value.unwrap());
                        assert_eq!(swapped_pair.y.value.unwrap(), pair.x.value.unwrap());
                    } else {
                        // Check that `x` and `y` have not been swapped
                        assert_eq!(swapped_pair.x.value.unwrap(), pair.x.value.unwrap());
                        assert_eq!(swapped_pair.y.value.unwrap(), pair.y.value.unwrap());
                    }
                }

                Ok(())
            }
        }

        // Test swap case
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                x: Some(Base::rand()),
                y: Some(Base::rand()),
                swap: Some(true),
            };
            let prover = match MockProver::<Base>::run(1, &circuit, vec![]) {
                Ok(prover) => prover,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(prover.verify(), Ok(()));
        }

        // Test non-swap case
        {
            let circuit: MyCircuit<Base> = MyCircuit {
                x: Some(Base::rand()),
                y: Some(Base::rand()),
                swap: Some(false),
            };
            let prover = match MockProver::<Base>::run(1, &circuit, vec![]) {
                Ok(prover) => prover,
                Err(e) => panic!("{:?}", e),
            };
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
