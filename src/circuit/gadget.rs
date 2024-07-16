//! Common gadgets and functions used in the Orchard circuit.

use ff::Field;

use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter},
    plonk,
};

pub(in crate::circuit) mod add_chip;

/// An instruction set for adding two circuit words (field elements).
pub(in crate::circuit) trait AddInstruction<F: Field>: Chip<F> {
    /// Constraints `a + b` and returns the sum.
    fn add(
        &self,
        layouter: impl Layouter<F>,
        a: &AssignedCell<F, F>,
        b: &AssignedCell<F, F>,
    ) -> Result<AssignedCell<F, F>, plonk::Error>;
}
