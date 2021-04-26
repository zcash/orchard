use super::{util, EccConfig, EccPoint};

use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, CurveExt, Field, FieldExt},
    circuit::{CellValue, Region},
    plonk::{ConstraintSystem, Error, Expression},
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_add_complete: Expression<F>,
    a: Expression<F>,
    b: Expression<F>,
    c: Expression<F>,
    d: Expression<F>,
    alpha: Expression<F>,
    beta: Expression<F>,
    gamma: Expression<F>,
    delta: Expression<F>,
    lambda: Expression<F>,
    x_p: Expression<F>,
    y_p: Expression<F>,
    x_q: Expression<F>,
    y_q: Expression<F>,
    x_r: Expression<F>,
    y_r: Expression<F>,
) {
    // Boolean checks on A, B, C, D
    {
        meta.create_gate("Check A is boolean", |_| {
            q_add_complete.clone() * a.clone() * (Expression::Constant(F::one()) - a.clone())
        });
        meta.create_gate("Check B is boolean", |_| {
            q_add_complete.clone() * b.clone() * (Expression::Constant(F::one()) - b.clone())
        });
        meta.create_gate("Check C is boolean", |_| {
            q_add_complete.clone() * c.clone() * (Expression::Constant(F::one()) - c.clone())
        });
        meta.create_gate("Check D is boolean", |_| {
            q_add_complete.clone() * d.clone() * (Expression::Constant(F::one()) - d.clone())
        });
    }

    // Logical implications of Boolean flags
    {
        // x_q = x_p ⟹ A
        meta.create_gate("x_q = x_p ⟹ A", |_| {
            let lhs = (x_q.clone() - x_p.clone()) * alpha.clone();
            let rhs = Expression::Constant(F::one()) - a.clone();
            q_add_complete.clone() * (lhs - rhs)
        });

        // x_p = 0 ⟹ B
        meta.create_gate("x_p = 0 ⟹ B", |_| {
            let lhs = x_p.clone() * beta.clone();
            let rhs = Expression::Constant(F::one()) - b.clone();
            q_add_complete.clone() * (lhs - rhs)
        });

        // B ⟹ x_p = 0
        meta.create_gate("B ⟹ x_p = 0", |_| {
            q_add_complete.clone() * b.clone() * x_p.clone()
        });

        // x_q = 0 ⟹ C
        meta.create_gate("x_q = 0 ⟹ C", |_| {
            let lhs = x_q.clone() * gamma.clone();
            let rhs = Expression::Constant(F::one()) - c.clone();
            q_add_complete.clone() * (lhs - rhs)
        });

        // C ⟹ x_q = 0
        meta.create_gate("C ⟹ x_q = 0", |_| {
            q_add_complete.clone() * c.clone() * x_q.clone()
        });

        // y_q = -y_p ⟹ D
        meta.create_gate("y_q = y_p ⟹ D", |_| {
            let lhs = (y_q.clone() + y_p.clone()) * delta.clone();
            let rhs = Expression::Constant(F::one()) - d.clone();
            q_add_complete.clone() * (lhs - rhs)
        });
    }

    // Handle cases in incomplete addition
    {
        // x_q ≠ x_p ⟹ λ = (y_q − y_p)/(x_q − x_p)
        meta.create_gate("x equality", |_| {
            let equal = x_q.clone() - x_p.clone();
            let unequal = equal.clone() * lambda.clone() - (y_q.clone() - y_p.clone());
            q_add_complete.clone() * equal * unequal
        });

        // A ∧ y_p ≠ 0 ⟹ λ = (3 * x_p^2) / 2 * y_p
        meta.create_gate("x equal, y nonzero", |_| {
            let three_x_p_sq = Expression::Constant(F::from_u64(3)) * x_p.clone() * x_p.clone();
            let two_y_p = Expression::Constant(F::from_u64(2)) * y_p.clone();
            let gradient = two_y_p * lambda.clone() - three_x_p_sq;
            q_add_complete.clone() * a.clone() * gradient
        });

        // (¬B ∧ ¬C ⟹ x_r = λ^2 − x_p − x_q) ∧ (B ⟹ x_r = x_q)
        meta.create_gate("x_r check", |_| {
            let not_b = Expression::Constant(F::one()) - b.clone();
            let not_c = Expression::Constant(F::one()) - c.clone();
            let x_r_lambda =
                lambda.clone() * lambda.clone() - x_p.clone() - x_q.clone() - x_r.clone();
            let x_r_x_q = b.clone() * (x_r.clone() - x_q.clone());
            q_add_complete.clone() * (not_b * not_c * x_r_lambda - x_r_x_q)
        });

        // ¬B ∧ ¬C ⟹ y_r = λ⋅(x_p − x_r) − y_p) ∧ (B ⟹ y_r = y_q)
        meta.create_gate("y_r check", |_| {
            let not_b = Expression::Constant(F::one()) - b.clone();
            let not_c = Expression::Constant(F::one()) - c.clone();
            let y_r_lambda =
                lambda.clone() * (x_p.clone() - x_r.clone()) - y_p.clone() - y_r.clone();
            let y_r_y_q = b.clone() * (y_r.clone() - y_q.clone());
            q_add_complete.clone() * (not_b * not_c * y_r_lambda - y_r_y_q)
        });

        // C ⟹ x_r = x_p
        meta.create_gate("x_r = x_p when x_q = 0", |_| {
            q_add_complete.clone() * (c.clone() * (x_r.clone() - x_p.clone()))
        });

        // C ⟹ y_r = y_p
        meta.create_gate("y_r = y_p when x_q = 0", |_| {
            q_add_complete.clone() * (c.clone() * (y_r.clone() - y_p.clone()))
        });

        // D ⟹ x_r = 0
        meta.create_gate("D ⟹ x_r = 0", |_| {
            q_add_complete.clone() * d.clone() * x_r.clone()
        });

        // D ⟹ y_r = 0
        meta.create_gate("D ⟹ y_r = 0", |_| {
            q_add_complete.clone() * d.clone() * y_r.clone()
        });
    }
}

#[allow(clippy::many_single_char_names)]
pub(super) fn assign_region<C: CurveAffine>(
    a: &EccPoint<C::Base>,
    b: &EccPoint<C::Base>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
) -> Result<EccPoint<C::Base>, Error> {
    // Enable `q_add_complete` selector
    config.q_add_complete.enable(region, offset)?;

    // Copy point `a` into `x_p`, `y_p` columns
    util::assign_and_constrain(region, || "x_p", config.P.0, offset, &a.x, &config.perm_sum)?;
    util::assign_and_constrain(region, || "y_p", config.P.1, offset, &a.y, &config.perm_sum)?;

    // Copy point `b` into `x_a`, `y_a` columns
    util::assign_and_constrain(region, || "x_q", config.A.0, offset, &b.x, &config.perm_sum)?;
    util::assign_and_constrain(region, || "y_q", config.A.1, offset, &b.y, &config.perm_sum)?;

    let (x_p, y_p) = (a.x.value, a.y.value);
    let (x_q, y_q) = (b.x.value, b.y.value);

    // Rename columns here to match specification
    let a = config.add_complete_bool[0];
    let b = config.add_complete_bool[1];
    let c = config.add_complete_bool[2];
    let d = config.add_complete_bool[3];
    let alpha = config.add_complete_inv[0];
    let beta = config.add_complete_inv[1];
    let gamma = config.add_complete_inv[2];
    let delta = config.add_complete_inv[3];
    let lambda = config.lambda.0;

    // Assign A, B, C, D, α, β, γ, δ
    {
        x_p.zip(x_q)
            .zip(y_p)
            .zip(y_q)
            .map(|(((x_p, x_q), y_p), y_q)| -> Result<(), Error> {
                if x_q == x_p {
                    // x_q = x_p ⟹ A
                    region.assign_advice(|| "set A", a, offset, || Ok(C::Base::one()))?;

                    // Doubling case, λ = (y_q − y_p) / (x_q − x_p)
                    if y_p != C::Base::zero() {
                        let lambda_val = C::Base::from_u64(3)
                            * x_p
                            * x_p
                            * (C::Base::from_u64(2) * y_p).invert().unwrap();
                        region.assign_advice(|| "set lambda", lambda, offset, || Ok(lambda_val))?;
                    }
                } else {
                    // α = 1 / (x_q - x_p)
                    let alpha_val = (x_q - x_p).invert().unwrap();
                    region.assign_advice(|| "set alpha", alpha, offset, || Ok(alpha_val))?;

                    // Non-doubling case, λ = (y_q - y_p) / (x_q - x_p)
                    let lambda_val = (x_q - x_p).invert().unwrap() * (y_q - y_p);
                    region.assign_advice(|| "set lambda", lambda, offset, || Ok(lambda_val))?;
                }

                if x_p == C::Base::zero() {
                    // x_p = 0 ⟹ B
                    region.assign_advice(|| "set B", b, offset, || Ok(C::Base::one()))?;
                } else {
                    // β = 1 / x_p
                    let beta_val = x_p.invert().unwrap();
                    region.assign_advice(|| "set beta", beta, offset, || Ok(beta_val))?;
                }
                if x_q == C::Base::zero() {
                    // x_q = 0 ⟹ C
                    region.assign_advice(|| "set C", c, offset, || Ok(C::Base::one()))?;
                } else {
                    // γ = 1 / x_q
                    let gamma_val = x_q.invert().unwrap();
                    region.assign_advice(|| "set gamma", gamma, offset, || Ok(gamma_val))?;
                }

                if y_p == -y_q {
                    // y_p = -y_p ⟹ D
                    region.assign_advice(|| "set D", d, offset, || Ok(C::Base::one()))?;
                } else {
                    // δ = 1 / (y_q + y_p)
                    let delta_val = (y_q + y_p).invert().unwrap();
                    region.assign_advice(|| "set delta", delta, offset, || Ok(delta_val))?;
                }
                Ok(())
            });
    }

    // Compute R = P + Q
    let r = x_p
        .zip(y_p)
        .zip(x_q)
        .zip(y_q)
        .map(|(((x_p, y_p), x_q), y_q)| {
            let p = C::from_xy(x_p, y_p).unwrap();
            let q = C::from_xy(x_q, y_q).unwrap();
            p + q
        });

    // `r` can be the point at infinity.
    r.map_or(Err(Error::SynthesisError), |r| {
        if r.is_on_curve().into() {
            // Assign `x_r`
            let x_r_val = *r.to_affine().coordinates().unwrap().x();
            let x_r_cell =
                region.assign_advice(|| "set x_r", config.A.0, offset + 1, || Ok(x_r_val))?;

            // Assign `y_r`
            let y_r_val = *r.to_affine().coordinates().unwrap().y();
            let y_r_cell =
                region.assign_advice(|| "set y_r", config.A.1, offset + 1, || Ok(y_r_val))?;

            Ok(EccPoint {
                x: CellValue::<C::Base>::new(x_r_cell, Some(x_r_val)),
                y: CellValue::<C::Base>::new(y_r_cell, Some(y_r_val)),
            })
        } else {
            Err(Error::SynthesisError)
        }
    })
}
