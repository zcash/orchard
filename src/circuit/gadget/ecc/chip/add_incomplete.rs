use super::{util, CellValue, EccConfig, EccPoint};
use halo2::{
    arithmetic::FieldExt,
    circuit::Region,
    plonk::{ConstraintSystem, Error, Expression},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_add_incomplete: Expression<F>,
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

        q_add_incomplete.clone() * expr1
    });

    // (y_a + y_q)(x_p − x_q) − (y_p − y_q)(x_q − x_a) = 0
    meta.create_gate("point addition expr2", |_| {
        let expr2 = (y_a + y_q.clone()) * (x_p - x_q.clone()) - (y_p - y_q) * (x_q - x_a);

        q_add_incomplete * expr2
    });
}

#[allow(non_snake_case)]
pub(super) fn assign_region<F: FieldExt>(
    a: &EccPoint<F>,
    b: &EccPoint<F>,
    offset: usize,
    region: &mut Region<'_, F>,
    config: EccConfig,
) -> Result<EccPoint<F>, Error> {
    // Compute the sum `a + b`
    let (x_p, y_p) = (a.x.value, a.y.value);
    let (x_q, y_q) = (b.x.value, b.y.value);

    // Handle exceptional cases
    x_p.zip(y_p)
        .zip(x_q)
        .zip(y_q)
        .map(|(((x_p, y_p), x_q), y_q)| {
            // P is point at infinity
            if (x_p == F::zero() && y_p == F::zero())
            // Q is point at infinity
            || (x_q == F::zero() && y_q == F::zero())
            // x_p = x_q
            || (x_p == x_q)
            {
                return Err(Error::SynthesisError);
            }
            Ok(())
        })
        .unwrap_or(Err(Error::SynthesisError))?;

    // Rename columns for `add` context
    let A = (config.extras[0], config.extras[1]);

    // Enable `q_add_incomplete` selector
    config.q_add_incomplete.enable(region, offset)?;

    // Copy point `a` into `x_p`, `y_p` columns
    util::assign_and_constrain(
        region,
        || "x_p",
        config.P.0.into(),
        offset,
        &a.x,
        &config.perm,
    )?;
    util::assign_and_constrain(
        region,
        || "y_p",
        config.P.1.into(),
        offset,
        &a.y,
        &config.perm,
    )?;

    // Copy point `b` into `x_a`, `y_a` columns
    util::assign_and_constrain(region, || "x_q", A.0.into(), offset, &b.x, &config.perm)?;
    util::assign_and_constrain(region, || "y_q", A.1.into(), offset, &b.y, &config.perm)?;

    let r = x_p
        .zip(y_p)
        .zip(x_q)
        .zip(y_q)
        .map(|(((x_p, y_p), x_q), y_q)| {
            let lambda = (y_q - y_p) * (x_q - x_p).invert().unwrap();
            let x_r = lambda * lambda - x_p - x_q;
            let y_r = lambda * (x_p - x_r) - y_p;
            (x_r, y_r)
        });

    // Assign the sum to `x_a`, `y_a` columns in the next row
    let x_r = r.map(|r| r.0);
    let x_r_var = region.assign_advice(
        || "x_r",
        A.0,
        offset + 1,
        || x_r.ok_or(Error::SynthesisError),
    )?;

    let y_r = r.map(|r| r.1);
    let y_r_var = region.assign_advice(
        || "y_r",
        A.1,
        offset + 1,
        || y_r.ok_or(Error::SynthesisError),
    )?;

    Ok(EccPoint {
        x: CellValue::<F>::new(x_r_var, x_r),
        y: CellValue::<F>::new(y_r_var, y_r),
    })
}
