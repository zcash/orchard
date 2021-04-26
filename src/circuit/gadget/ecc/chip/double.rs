use super::{util, EccConfig, EccPoint};

use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{CellValue, Region},
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_double: Expression<F>,
    x_a: Expression<F>,
    y_a: Expression<F>,
    x_p: Expression<F>,
    y_p: Expression<F>,
) {
    let x_p_2 = x_p.clone() * x_p.clone();
    let x_p_4 = x_p_2.clone() * x_p_2.clone();

    // 4⋅(y_p)^2⋅(x_a + 2⋅x_p) − 9⋅(x_p)^4 = 0
    meta.create_gate("point doubling expr1", |_| {
        let expr1 = y_p.clone()
            * y_p.clone()
            * (x_a.clone() + x_p.clone() * F::from_u64(2))
            * F::from_u64(4)
            - x_p_4 * F::from_u64(9);
        q_double.clone() * expr1
    });

    // 2⋅y_p⋅(y_a + y_p) − 3⋅(x_p)^2⋅(x_p − x_a) = 0
    meta.create_gate("point doubling expr2", |_| {
        let expr2 =
            y_p.clone() * (y_a + y_p) * F::from_u64(2) - x_p_2 * (x_p - x_a) * F::from_u64(3);

        q_double * expr2
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    a: &EccPoint<C::Base>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
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
            .coordinates()
            .unwrap()
    });
    let x_a_val = doubled.map(|doubled| *doubled.x());
    let y_a_val = doubled.map(|doubled| *doubled.y());

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
        x: CellValue::<C::Base>::new(x_a_var, x_a_val),
        y: CellValue::<C::Base>::new(y_a_var, y_a_val),
    })
}
