use super::{util, EccConfig, EccPoint};
use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::{CellValue, Region},
    plonk::{ConstraintSystem, Error, Expression},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_add: Expression<F>,
    x_p: Expression<F>,
    y_p: Expression<F>,
    x_q: Expression<F>,
    y_q: Expression<F>,
    x_a: Expression<F>,
    y_a: Expression<F>,
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
    a: &EccPoint<C::Base>,
    b: &EccPoint<C::Base>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
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
            (p + q).to_affine().coordinates().unwrap()
        });

    // Assign the sum to `x_a`, `y_a` columns in the next row
    let x_a_val = sum.map(|sum| *sum.x());
    let x_a_var = region.assign_advice(
        || "x_a",
        config.A.0,
        offset + 1,
        || x_a_val.ok_or(Error::SynthesisError),
    )?;

    let y_a_val = sum.map(|sum| *sum.y());
    let y_a_var = region.assign_advice(
        || "y_a",
        config.A.1,
        offset + 1,
        || y_a_val.ok_or(Error::SynthesisError),
    )?;

    Ok(EccPoint {
        x: CellValue::<C::Base>::new(x_a_var, x_a_val),
        y: CellValue::<C::Base>::new(y_a_var, y_a_val),
    })
}
