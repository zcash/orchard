use super::CellValue;
use halo2::{
    arithmetic::FieldExt,
    circuit::Region,
    plonk::{Advice, Column, Error, Permutation},
};

/// Assign a cell the same value as another cell and set up a copy constraint between them.
pub(super) fn assign_and_constrain<A, AR, F: FieldExt>(
    region: &mut Region<'_, F>,
    annotation: A,
    column: Column<Advice>,
    row: usize,
    copy: &CellValue<F>,
    perm: &Permutation,
) -> Result<CellValue<F>, Error>
where
    A: Fn() -> AR,
    AR: Into<String>,
{
    let cell = region.assign_advice(annotation, column, row, || {
        copy.value.ok_or(Error::SynthesisError)
    })?;

    region.constrain_equal(perm, cell, copy.cell)?;

    Ok(CellValue::new(cell, copy.value))
}
