use super::{CellValue, EccConfig, EccPoint};

use halo2::{
    arithmetic::CurveAffine,
    circuit::Region,
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<C: CurveAffine>(
    meta: &mut ConstraintSystem<C::Base>,
    q_point: Expression<C::Base>,
    x_p: Expression<C::Base>,
    y_p: Expression<C::Base>,
) {
    meta.create_gate("witness point", |_| {
        // Check that y^2 = x^3 + b, where b = 5 in the Pallas equation
        q_point
            * (y_p.clone() * y_p - (x_p.clone() * x_p.clone() * x_p) - Expression::Constant(C::b()))
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    value: Option<C>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
) -> Result<EccPoint<C::Base>, Error> {
    // Enable `q_point` selector
    config.q_point.enable(region, offset)?;

    let value = value.map(|value| value.coordinates().unwrap());

    // Assign `x_p` value
    let x_p_val = value.map(|value| *value.x());
    let x_p_var = region.assign_advice(
        || "x_p",
        config.P.0,
        offset,
        || x_p_val.ok_or(Error::SynthesisError),
    )?;

    // Assign `y_p` value
    let y_p_val = value.map(|value| *value.y());
    let y_p_var = region.assign_advice(
        || "y_p",
        config.P.1,
        offset,
        || y_p_val.ok_or(Error::SynthesisError),
    )?;

    Ok(EccPoint {
        x: CellValue::<C::Base>::new(x_p_var, x_p_val),
        y: CellValue::<C::Base>::new(y_p_var, y_p_val),
    })
}
