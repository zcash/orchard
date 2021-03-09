use super::super::EccInstructions;
use super::{util, CellValue, EccChip, EccPoint};
use group::Curve;
use halo2::{
    arithmetic::CurveAffine,
    circuit::{Chip, Region},
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<C: CurveAffine>(
    meta: &mut ConstraintSystem<C::Base>,
    q_add: Expression<C::Base>,
    x_p: Expression<C::Base>,
    y_p: Expression<C::Base>,
    x_q: Expression<C::Base>,
    y_q: Expression<C::Base>,
    x_a: Expression<C::Base>,
    y_a: Expression<C::Base>,
) {
    // (x_a + x_q + x_p)⋅(x_p − x_q)^2 − (y_p − y_q)^2 = 0
    meta.create_gate("point addition expr1", |_| {
        let expr1 = (x_a.clone() + x_q.clone() + x_p.clone())
            * (x_p.clone() - x_q.clone())
            * (x_p.clone() - x_q.clone())
            - (y_p.clone() - y_q.clone()) * (y_p.clone() - y_q.clone());

        q_add.clone() * expr1
    });

    // (y_a + y_q)(x_p − x_q) − (y_p − y_q)(x_q − x_a) = 0
    meta.create_gate("point addition expr2", |_| {
        let expr2 = (y_a + y_q.clone()) * (x_p - x_q.clone()) - (y_p - y_q) * (x_q - x_a);

        q_add * expr2
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    a: &<EccChip<C> as EccInstructions<C>>::Point,
    b: &<EccChip<C> as EccInstructions<C>>::Point,
    offset: usize,
    region: &mut Region<'_, EccChip<C>>,
    config: <EccChip<C> as Chip>::Config,
) -> Result<EccPoint<C::Base>, Error> {
    // Enable `q_add` selector
    config.q_add.enable(region, offset)?;

    // Copy point `a` into `x_p`, `y_p` columns
    util::assign_and_constrain(region, || "x_p", config.P.0, offset, &a.x, &config.perm_sum)?;
    util::assign_and_constrain(region, || "y_p", config.P.1, offset, &a.y, &config.perm_sum)?;

    // Copy point `b` into `x_a`, `y_a` columns
    util::assign_and_constrain(region, || "x_q", config.A.0, offset, &b.x, &config.perm_sum)?;
    util::assign_and_constrain(region, || "y_q", config.A.1, offset, &b.y, &config.perm_sum)?;

    // Compute the sum `a + b`
    let (x_p, y_p) = (a.x.value, a.y.value);
    let (x_q, y_q) = (b.x.value, b.y.value);

    let sum = x_p
        .zip(y_p)
        .zip(x_q)
        .zip(y_q)
        .map(|(((x_p, y_p), x_q), y_q)| {
            let p = C::from_xy(x_p, y_p).unwrap();
            let q = C::from_xy(x_q, y_q).unwrap();
            (p + q).to_affine().get_xy().unwrap()
        });

    // Assign the sum to `x_a`, `y_a` columns in the next row
    let x_a_val = sum.map(|sum| sum.0);
    let x_a_var = region.assign_advice(
        || "x_a",
        config.A.0,
        offset + 1,
        || x_a_val.ok_or(Error::SynthesisError),
    )?;

    let y_a_val = sum.map(|sum| sum.1);
    let y_a_var = region.assign_advice(
        || "y_a",
        config.A.1,
        offset + 1,
        || y_a_val.ok_or(Error::SynthesisError),
    )?;

    Ok(EccPoint {
        x: CellValue::new(x_a_var, x_a_val),
        y: CellValue::new(y_a_var, y_a_val),
    })
}
