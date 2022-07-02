use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter},
    plonk::{self, Advice, Column, ConstraintSystem, Constraints, Selector, Expression},
    poly::Rotation,
};
use pasta_curves::pallas;


#[derive(Clone, Debug)]
pub(in crate::circuit) struct MuxConfig {
    q_mux: Selector,
    switch: Column<Advice>,
    left: Column<Advice>,
    right: Column<Advice>,
    out: Column<Advice>,
}

/// A chip implementing a multiplexer on a single row.
///
///     out = if (switch == 0) { left } else { right }
///
/// Switch must be constrained to {0, 1} separately.
pub(in crate::circuit) struct MuxChip {
    config: MuxConfig,
}

impl MuxChip {
    pub(in crate::circuit) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        switch: Column<Advice>,
        left: Column<Advice>,
        right: Column<Advice>,
        out: Column<Advice>,
    ) -> MuxConfig {
        let q_mux = meta.selector();

        meta.create_gate("Field element multiplexer", |meta| {
            let q_mux = meta.query_selector(q_mux);
            let switch = meta.query_advice(switch, Rotation::cur());
            let left = meta.query_advice(left, Rotation::cur());
            let right = meta.query_advice(right, Rotation::cur());
            let out = meta.query_advice(out, Rotation::cur());

            let one = Expression::Constant(pallas::Base::one());
            let not_switch = one - switch.clone();
            let should_be_zero = not_switch * left + switch * right - out;

            Constraints::with_selector(q_mux, Some(should_be_zero))
        });

        MuxConfig { q_mux, switch, left, right, out }
    }

    pub(in crate::circuit) fn construct(config: MuxConfig) -> Self {
        Self { config }
    }
}

impl MuxInstruction<pallas::Base> for MuxChip {
    fn mux(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        switch: &AssignedCell<pallas::Base, pallas::Base>,
        left: &AssignedCell<pallas::Base, pallas::Base>,
        right: &AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<AssignedCell<pallas::Base, pallas::Base>, plonk::Error> {
        layouter.assign_region(
            || "mux",
            |mut region| {
                self.config.q_mux.enable(&mut region, 0)?;

                switch.copy_advice(|| "copy switch", &mut region, self.config.switch, 0)?;
                left.copy_advice(|| "copy left", &mut region, self.config.left, 0)?;
                right.copy_advice(|| "copy right", &mut region, self.config.right, 0)?;

                let out_val = match (switch.value(), left.value(), right.value()) {
                    (Some(switch), Some(left), Some(right)) => {
                        let one = pallas::Base::one();
                        let not_switch = one - switch;
                        let out = not_switch * left + switch * right;
                        Some(out)
                    },
                    _ => None,
                };

                region.assign_advice(
                    || "out",
                    self.config.out,
                    0,
                    || out_val.ok_or(plonk::Error::Synthesis),
                )
            },
        )
    }
}
