use super::super::EccInstructions;
use super::{util, CellValue, EccChip, EccPoint};

use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{Chip, Region},
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<C: CurveAffine>(
    meta: &mut ConstraintSystem<C::Base>,
    q_double: Expression<C::Base>,
    x_a: Expression<C::Base>,
    y_a: Expression<C::Base>,
    x_p: Expression<C::Base>,
    y_p: Expression<C::Base>,
) {
    // 4⋅(y_p)^2⋅(x_a + 2⋅x_p) − 9⋅(x_p)^4 = 0
    meta.create_gate("point doubling expr1", |_| {
        let x_p_4 = x_p.clone() * x_p.clone() * x_p.clone() * x_p.clone();
        let expr1 = y_p.clone()
            * y_p.clone()
            * (x_a.clone() + x_p.clone() * C::Base::from_u64(2))
            * C::Base::from_u64(4)
            - x_p_4 * C::Base::from_u64(9);
        q_double.clone() * expr1
    });

    // 2⋅y_p⋅(y_a + y_p) − 3⋅(x_p)^2⋅(x_p − x_a) = 0
    meta.create_gate("point doubling expr2", |_| {
        let expr2 = y_p.clone() * (y_a + y_p) * C::Base::from_u64(2)
            - x_p.clone() * x_p.clone() * (x_p - x_a) * C::Base::from_u64(3);

        q_double * expr2
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    a: &<EccChip<C> as EccInstructions<C>>::Point,
    offset: usize,
    region: &mut Region<'_, EccChip<C>>,
    config: <EccChip<C> as Chip>::Config,
) -> Result<EccPoint<C::Base>, Error> {
    // Enable `q_double` selector
    config.q_double.enable(region, offset)?;

    // Copy the point into `x_p`, `y_p` columns
    util::assign_and_constrain(region, || "x_p", config.P.0, offset, &a.x, &config.perm_sum)?;
    util::assign_and_constrain(region, || "y_p", config.P.1, offset, &a.y, &config.perm_sum)?;

    // Compute the doubled point
    let (x_p_val, y_p_val) = (a.x.value, a.y.value);
    let doubled = x_p_val.zip(y_p_val).map(|(x, y)| {
        (C::from_xy(x, y).unwrap() * C::Scalar::from_u64(2))
            .to_affine()
            .get_xy()
            .unwrap()
    });
    let x_a_val = doubled.map(|doubled| doubled.0);
    let y_a_val = doubled.map(|doubled| doubled.1);

    // Assign the doubled point to `x_a`, `y_a` columns
    let x_a_var = region.assign_advice(
        || "x_a_val",
        config.A.0,
        offset,
        || x_a_val.ok_or(Error::SynthesisError),
    )?;
    let y_a_var = region.assign_advice(
        || "y_a_val",
        config.A.1,
        offset,
        || y_a_val.ok_or(Error::SynthesisError),
    )?;

    Ok(EccPoint {
        x: CellValue::new(x_a_var, x_a_val),
        y: CellValue::new(y_a_var, y_a_val),
    })
}
