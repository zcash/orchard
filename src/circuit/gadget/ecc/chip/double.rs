use super::{util, CellValue, EccConfig, EccPoint};

use halo2::{
    arithmetic::FieldExt,
    circuit::Region,
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

#[allow(non_snake_case)]
pub(super) fn assign_region<F: FieldExt>(
    a: &EccPoint<F>,
    offset: usize,
    region: &mut Region<'_, F>,
    config: EccConfig,
) -> Result<EccPoint<F>, Error> {
    // Rename columns
    let A = (config.extras[0], config.extras[1]);

    // Enable `q_double` selector
    config.q_double.enable(region, offset)?;

    // Copy the point into `x_p`, `y_p` columns
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

    // Compute the doubled point
    let (x_p, y_p) = (a.x.value, a.y.value);
    let r = x_p.zip(y_p).map(|(x_p, y_p)| {
        let lambda = F::from_u64(3) * x_p * x_p * F::TWO_INV * y_p.invert().unwrap();
        let x_r = lambda * lambda - x_p - x_p;
        let y_r = lambda * (x_p - x_r) - y_p;
        (x_r, y_r)
    });
    let x_r = r.map(|r| r.0);
    let y_r = r.map(|r| r.1);

    // Assign the doubled point to `x_a`, `y_a` columns
    let x_r_var =
        region.assign_advice(|| "x_r", A.0, offset, || x_r.ok_or(Error::SynthesisError))?;
    let y_r_var =
        region.assign_advice(|| "y_r", A.1, offset, || y_r.ok_or(Error::SynthesisError))?;

    Ok(EccPoint {
        x: CellValue::<F>::new(x_r_var, x_r),
        y: CellValue::<F>::new(y_r_var, y_r),
    })
}
