use super::CellValue;
use halo2::{
    arithmetic::FieldExt,
    circuit::Region,
    plonk::{Advice, Any, Column, Error, Fixed, Permutation},
};
use std::convert::TryFrom;

/// Assign a cell the same value as another cell and set up a copy constraint between them.
pub(super) fn assign_and_constrain<A, AR, F: FieldExt>(
    region: &mut Region<'_, F>,
    annotation: A,
    column: Column<Any>,
    row: usize,
    copy: &CellValue<F>,
    perm: &Permutation,
) -> Result<CellValue<F>, Error>
where
    A: Fn() -> AR,
    AR: Into<String>,
{
    let cell = if let Ok(column) = Column::<Advice>::try_from(column) {
        region.assign_advice(annotation, column, row, || {
            copy.value.ok_or(Error::SynthesisError)
        })?
    } else if let Ok(column) = Column::<Fixed>::try_from(column) {
        region.assign_fixed(annotation, column, row, || {
            copy.value.ok_or(Error::SynthesisError)
        })?
    } else {
        return Err(Error::SynthesisError);
    };

    region.constrain_equal(perm, cell, copy.cell)?;

    Ok(CellValue::new(cell, copy.value))
}
