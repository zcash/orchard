use super::{util, CellValue, EccConfig, EccPoint};
use ff::Field;
use group::{Curve, Group};
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};

#[derive(Clone, Debug)]
pub struct Config {
    q_add: Selector,
    // lambda
    lambda: Column<Advice>,
    // x-coordinate of P in P + Q = R
    pub x_p: Column<Advice>,
    // y-coordinate of P in P + Q = R
    pub y_p: Column<Advice>,
    // x-coordinate of Q or R in P + Q = R
    pub x_qr: Column<Advice>,
    // y-coordinate of Q or R in P + Q = R
    pub y_qr: Column<Advice>,
    // a or alpha
    a_alpha: Column<Advice>,
    // b or beta
    b_beta: Column<Advice>,
    // c or gamma
    c_gamma: Column<Advice>,
    // d or delta
    d_delta: Column<Advice>,
    // Permutation
    perm: Permutation,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_add: ecc_config.q_add,
            lambda: ecc_config.lambda.0,
            x_p: ecc_config.P.0,
            y_p: ecc_config.P.1,
            x_qr: ecc_config.extras[0],
            y_qr: ecc_config.extras[1],
            a_alpha: ecc_config.extras[2],
            b_beta: ecc_config.extras[3],
            c_gamma: ecc_config.extras[4],
            d_delta: ecc_config.lambda.1,
            perm: ecc_config.perm.clone(),
        }
    }
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        let q_add = meta.query_selector(self.q_add, Rotation::cur());
        let x_p = meta.query_advice(self.x_p, Rotation::cur());
        let y_p = meta.query_advice(self.y_p, Rotation::cur());
        let x_q = meta.query_advice(self.x_qr, Rotation::cur());
        let y_q = meta.query_advice(self.y_qr, Rotation::cur());
        let x_r = meta.query_advice(self.x_qr, Rotation::next());
        let y_r = meta.query_advice(self.y_qr, Rotation::next());
        let lambda = meta.query_advice(self.lambda, Rotation::cur());

        let a = meta.query_advice(self.a_alpha, Rotation::cur());
        let b = meta.query_advice(self.b_beta, Rotation::cur());
        let c = meta.query_advice(self.c_gamma, Rotation::cur());
        let d = meta.query_advice(self.d_delta, Rotation::cur());

        // \alpha = (x_q - x_p)^{-1}
        let alpha = meta.query_advice(self.a_alpha, Rotation::next());
        // \beta = x_p^{-1}
        let beta = meta.query_advice(self.b_beta, Rotation::next());
        // \gamma = x_q^{-1}
        let gamma = meta.query_advice(self.c_gamma, Rotation::next());
        // \delta = (y_p + y_q)^{-1}
        let delta = meta.query_advice(self.d_delta, Rotation::next());

        // Boolean checks on A, B, C, D
        {
            meta.create_gate("Check A is boolean", |_| {
                q_add.clone() * a.clone() * (Expression::Constant(F::one()) - a.clone())
            });
            meta.create_gate("Check B is boolean", |_| {
                q_add.clone() * b.clone() * (Expression::Constant(F::one()) - b.clone())
            });
            meta.create_gate("Check C is boolean", |_| {
                q_add.clone() * c.clone() * (Expression::Constant(F::one()) - c.clone())
            });
            meta.create_gate("Check D is boolean", |_| {
                q_add.clone() * d.clone() * (Expression::Constant(F::one()) - d.clone())
            });
        }

        // Logical implications of Boolean flags
        {
            // (x_q − x_p)⋅α = 1 − A
            meta.create_gate("x_q = x_p ⟹ A", |_| {
                let lhs = (x_q.clone() - x_p.clone()) * alpha.clone();
                let rhs = Expression::Constant(F::one()) - a.clone();
                q_add.clone() * (lhs - rhs)
            });

            // x_p⋅β = 1 − B
            meta.create_gate("x_p = 0 ⟹ B", |_| {
                let lhs = x_p.clone() * beta.clone();
                let rhs = Expression::Constant(F::one()) - b.clone();
                q_add.clone() * (lhs - rhs)
            });

            // B⋅x_p = 0
            meta.create_gate("B ⟹ x_p = 0", |_| q_add.clone() * b.clone() * x_p.clone());

            // x_q⋅γ = 1 − C
            meta.create_gate("x_q = 0 ⟹ C", |_| {
                let lhs = x_q.clone() * gamma.clone();
                let rhs = Expression::Constant(F::one()) - c.clone();
                q_add.clone() * (lhs - rhs)
            });

            // C⋅x_q = 0
            meta.create_gate("C ⟹ x_q = 0", |_| q_add.clone() * c.clone() * x_q.clone());

            // (y_q + y_p)⋅δ = 1 − D
            meta.create_gate("y_q = y_p ⟹ D", |_| {
                let lhs = (y_q.clone() + y_p.clone()) * delta.clone();
                let rhs = Expression::Constant(F::one()) - d.clone();
                q_add.clone() * (lhs - rhs)
            });
        }

        // Handle cases in incomplete addition
        {
            // (x_q − x_p)⋅((x_q − x_p)⋅λ − (y_q−y_p))=0
            meta.create_gate("x equality", |_| {
                let equal = x_q.clone() - x_p.clone();
                let unequal = equal.clone() * lambda.clone() - (y_q.clone() - y_p.clone());
                q_add.clone() * equal * unequal
            });

            // A⋅(2y_p⋅λ − 3x_p^2) = 0
            meta.create_gate("x equal, y nonzero", |_| {
                let three_x_p_sq = Expression::Constant(F::from_u64(3)) * x_p.clone() * x_p.clone();
                let two_y_p = Expression::Constant(F::from_u64(2)) * y_p.clone();
                let gradient = two_y_p * lambda.clone() - three_x_p_sq;
                q_add.clone() * a.clone() * gradient
            });

            // (1 − B)⋅(1 − C)⋅(1 − D)⋅(λ^2 − x_p − x_q − x_r) + B⋅(x_r − x_q) = 0
            meta.create_gate("x_r check", |_| {
                let not_b = Expression::Constant(F::one()) - b.clone();
                let not_c = Expression::Constant(F::one()) - c.clone();
                let not_d = Expression::Constant(F::one()) - d.clone();
                let x_r_lambda =
                    lambda.clone() * lambda.clone() - x_p.clone() - x_q.clone() - x_r.clone();
                let x_r_x_q = b.clone() * (x_r.clone() - x_q.clone());
                q_add.clone() * (not_b * not_c * not_d * x_r_lambda + x_r_x_q)
            });

            // (1 − B)⋅(1 − C)⋅(1 − D)⋅(λ⋅(x_p − x_r) − y_p − y_r) + B⋅(y_r − y_q) = 0
            meta.create_gate("y_r check", |_| {
                let not_b = Expression::Constant(F::one()) - b.clone();
                let not_c = Expression::Constant(F::one()) - c.clone();
                let not_d = Expression::Constant(F::one()) - d.clone();
                let y_r_lambda =
                    lambda.clone() * (x_p.clone() - x_r.clone()) - y_p.clone() - y_r.clone();
                let y_r_y_q = b.clone() * (y_r.clone() - y_q.clone());
                q_add.clone() * (not_b * not_c * not_d * y_r_lambda + y_r_y_q)
            });

            // C⋅(x_r − x_p) = 0
            meta.create_gate("x_r = x_p when x_q = 0", |_| {
                q_add.clone() * (c.clone() * (x_r.clone() - x_p.clone()))
            });

            // C⋅(y_r − y_p) = 0
            meta.create_gate("y_r = y_p when x_q = 0", |_| {
                q_add.clone() * (c.clone() * (y_r.clone() - y_p.clone()))
            });

            // D⋅x_r = 0
            meta.create_gate("D ⟹ x_r = 0", |_| q_add.clone() * d.clone() * x_r.clone());

            // D⋅y_r = 0
            meta.create_gate("D ⟹ y_r = 0", |_| q_add.clone() * d.clone() * y_r.clone());
        }
    }

    #[allow(clippy::many_single_char_names)]
    #[allow(non_snake_case)]
    pub(super) fn assign_region<C: CurveAffine>(
        &self,
        a: &EccPoint<C::Base>,
        b: &EccPoint<C::Base>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C::Base>, Error> {
        // Enable `q_add` selector
        self.q_add.enable(region, offset)?;

        // Copy point `a` into `x_p`, `y_p` columns
        util::assign_and_constrain(region, || "x_p", self.x_p.into(), offset, &a.x, &self.perm)?;
        util::assign_and_constrain(region, || "y_p", self.y_p.into(), offset, &a.y, &self.perm)?;

        // Copy point `b` into `x_qr`, `y_qr` columns
        util::assign_and_constrain(region, || "x_q", self.x_qr.into(), offset, &b.x, &self.perm)?;
        util::assign_and_constrain(region, || "y_q", self.y_qr.into(), offset, &b.y, &self.perm)?;

        let (x_p, y_p) = (a.x.value, a.y.value);
        let (x_q, y_q) = (b.x.value, b.y.value);

        // Assign A, B, C, D, α, β, γ, δ
        {
            x_p.zip(x_q)
                .zip(y_p)
                .zip(y_q)
                .map(|(((x_p, x_q), y_p), y_q)| -> Result<(), Error> {
                    if x_q == x_p {
                        // x_q = x_p ⟹ A
                        region.assign_advice(
                            || "set A",
                            self.a_alpha,
                            offset,
                            || Ok(C::Base::one()),
                        )?;

                        // Doubling case, λ = 3(x_p)^2 / (2 * y_p)
                        if y_p != C::Base::zero() {
                            let lambda_val = C::Base::from_u64(3)
                                * x_p
                                * x_p
                                * (C::Base::from_u64(2) * y_p).invert().unwrap();
                            region.assign_advice(
                                || "set lambda",
                                self.lambda,
                                offset,
                                || Ok(lambda_val),
                            )?;
                        }
                    } else {
                        // α = 1 / (x_q - x_p)
                        let alpha_val = (x_q - x_p).invert().unwrap();
                        region.assign_advice(
                            || "set alpha",
                            self.a_alpha,
                            offset + 1,
                            || Ok(alpha_val),
                        )?;

                        // Non-doubling case, λ = (y_q - y_p) / (x_q - x_p)
                        let lambda_val = (x_q - x_p).invert().unwrap() * (y_q - y_p);
                        region.assign_advice(
                            || "set lambda",
                            self.lambda,
                            offset,
                            || Ok(lambda_val),
                        )?;
                    }

                    if x_p == C::Base::zero() {
                        // x_p = 0 ⟹ B
                        region.assign_advice(
                            || "set B",
                            self.b_beta,
                            offset,
                            || Ok(C::Base::one()),
                        )?;
                    } else {
                        // β = 1 / x_p
                        let beta_val = x_p.invert().unwrap();
                        region.assign_advice(
                            || "set beta",
                            self.b_beta,
                            offset + 1,
                            || Ok(beta_val),
                        )?;
                    }
                    if x_q == C::Base::zero() {
                        // x_q = 0 ⟹ C
                        region.assign_advice(
                            || "set C",
                            self.c_gamma,
                            offset,
                            || Ok(C::Base::one()),
                        )?;
                    } else {
                        // γ = 1 / x_q
                        let gamma_val = x_q.invert().unwrap();
                        region.assign_advice(
                            || "set gamma",
                            self.c_gamma,
                            offset + 1,
                            || Ok(gamma_val),
                        )?;
                    }

                    if y_p == -y_q {
                        // y_p = -y_p ⟹ D
                        region.assign_advice(
                            || "set D",
                            self.d_delta,
                            offset,
                            || Ok(C::Base::one()),
                        )?;
                    } else {
                        // δ = 1 / (y_q + y_p)
                        let delta_val = (y_q + y_p).invert().unwrap();
                        region.assign_advice(
                            || "set delta",
                            self.d_delta,
                            offset + 1,
                            || Ok(delta_val),
                        )?;
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
                // If either `p` or `q` are (0,0) represent them as C::zero()
                let p = if x_p == C::Base::zero() && y_p == C::Base::zero() {
                    C::identity()
                } else {
                    C::from_xy(x_p, y_p).unwrap()
                };
                let q = if x_q == C::Base::zero() && y_q == C::Base::zero() {
                    C::identity()
                } else {
                    C::from_xy(x_q, y_q).unwrap()
                };
                p + q
            });

        let x_r_val = r.map(|r| {
            if r.is_identity().into() {
                C::Base::zero()
            } else {
                *r.to_affine().coordinates().unwrap().x()
            }
        });

        let y_r_val = r.map(|r| {
            if r.is_identity().into() {
                C::Base::zero()
            } else {
                *r.to_affine().coordinates().unwrap().y()
            }
        });

        // Assign `x_r`
        let x_r_cell = region.assign_advice(
            || "set x_r",
            self.x_qr,
            offset + 1,
            || x_r_val.ok_or(Error::SynthesisError),
        )?;

        // Assign `y_r`
        let y_r_cell = region.assign_advice(
            || "set y_r",
            self.y_qr,
            offset + 1,
            || y_r_val.ok_or(Error::SynthesisError),
        )?;

        Ok(EccPoint {
            x: CellValue::<C::Base>::new(x_r_cell, x_r_val),
            y: CellValue::<C::Base>::new(y_r_cell, y_r_val),
        })
    }
}
