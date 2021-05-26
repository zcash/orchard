use super::{util, CellValue, EccConfig, EccPoint};
use ff::Field;
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
    // α = inv0(x_q - x_p)
    alpha: Column<Advice>,
    // β = inv0(x_p)
    beta: Column<Advice>,
    // γ = inv0(x_q)
    gamma: Column<Advice>,
    // δ = inv0(y_p + y_q) if x_q = x_p, 0 otherwise
    delta: Column<Advice>,
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
            alpha: ecc_config.extras[2],
            beta: ecc_config.extras[3],
            gamma: ecc_config.extras[4],
            delta: ecc_config.lambda.1,
            perm: ecc_config.perm.clone(),
        }
    }
}

impl Config {
    pub(crate) fn create_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        let q_add = meta.query_selector(self.q_add, Rotation::cur());
        let x_p = meta.query_advice(self.x_p, Rotation::cur());
        let y_p = meta.query_advice(self.y_p, Rotation::cur());
        let x_q = meta.query_advice(self.x_qr, Rotation::cur());
        let y_q = meta.query_advice(self.y_qr, Rotation::cur());
        let x_r = meta.query_advice(self.x_qr, Rotation::next());
        let y_r = meta.query_advice(self.y_qr, Rotation::next());
        let lambda = meta.query_advice(self.lambda, Rotation::cur());

        // inv0(x) is 0 if x = 0, 1/x otherwise.
        // α = inv0(x_q - x_p)
        let alpha = meta.query_advice(self.alpha, Rotation::cur());
        // β = inv0(x_p)
        let beta = meta.query_advice(self.beta, Rotation::cur());
        // γ = inv0(x_q)
        let gamma = meta.query_advice(self.gamma, Rotation::cur());
        // δ = inv0(y_p + y_q) if x_q = x_p, 0 otherwise
        let delta = meta.query_advice(self.delta, Rotation::cur());

        // Useful composite expressions
        // α ⋅(x_q - x_p)
        let if_alpha = (x_q.clone() - x_p.clone()) * alpha;
        // β ⋅ x_p
        let if_beta = x_p.clone() * beta;
        // γ ⋅ x_q
        let if_gamma = x_q.clone() * gamma;
        // δ ⋅(y_p + y_q)
        let if_delta = (y_q.clone() + y_p.clone()) * delta;

        // Useful constants
        let one = Expression::Constant(F::one());
        let two = Expression::Constant(F::from_u64(2));
        let three = Expression::Constant(F::from_u64(3));

        // Handle cases in incomplete addition
        {
            // (x_q − x_p)⋅((x_q − x_p)⋅λ − (y_q−y_p))=0
            meta.create_gate(
                "(x_q − x_p)⋅((x_q − x_p)⋅λ − (y_q−y_p))=0",
                |_| {
                    let x_q_minus_x_p = x_q.clone() - x_p.clone(); // (x_q − x_p)

                    let y_q_minus_y_p = y_q.clone() - y_p.clone(); // (y_q − y_p)
                    let incomplete = x_q_minus_x_p.clone() * lambda.clone() - y_q_minus_y_p; // (x_q − x_p)⋅λ − (y_q−y_p)

                    // q_add ⋅(x_q − x_p)⋅((x_q − x_p)⋅λ − (y_q−y_p))
                    q_add.clone() * x_q_minus_x_p * incomplete
                },
            );

            // (1 - (x_q - x_p)⋅α)⋅(2y_p ⋅λ - 3x_p^2) = 0
            meta.create_gate("(1 - (x_q - x_p)⋅α)⋅(2y_p ⋅λ - 3x_p^2) = 0", |_| {
                let three_x_p_sq = three * x_p.clone() * x_p.clone(); // 3x_p^2
                let two_y_p = two.clone() * y_p.clone(); // 2y_p
                let derivative = two_y_p * lambda.clone() - three_x_p_sq; // (2y_p ⋅λ - 3x_p^2)

                // q_add ⋅(1 - (x_q - x_p)⋅α)⋅(2y_p ⋅λ - 3x_p^2)
                q_add.clone() * (one.clone() - if_alpha.clone()) * derivative
            });

            // x_p⋅x_q⋅(x_q - x_p)⋅(λ^2 - x_p - x_q - x_r) = 0
            meta.create_gate(
                "x_p⋅x_q⋅(x_q - x_p)⋅(λ^2 - x_p - x_q - x_r) = 0",
                |_| {
                    let x_q_minus_x_p = x_q.clone() - x_p.clone(); // (x_q - x_p)
                    let incomplete =
                        lambda.clone() * lambda.clone() - x_p.clone() - x_q.clone() - x_r.clone(); // (λ^2 - x_p - x_q - x_r)

                    // q_add ⋅ x_p⋅x_q⋅(x_q - x_p)⋅(λ^2 - x_p - x_q - x_r)
                    q_add.clone() * x_p.clone() * x_q.clone() * x_q_minus_x_p * incomplete
                },
            );

            // x_p⋅x_q⋅(x_q - x_p)⋅(λ ⋅(x_p - x_r) - y_p - y_r) = 0
            meta.create_gate(
                "x_p⋅x_q⋅(x_q - x_p)⋅(λ ⋅(x_p - x_r) - y_p - y_r) = 0",
                |_| {
                    let x_q_minus_x_p = x_q.clone() - x_p.clone(); // (x_q - x_p)
                    let x_p_minus_x_r = x_p.clone() - x_r.clone(); // (x_p - x_r)

                    // q_add ⋅ x_p⋅x_q⋅(x_q - x_p)⋅(λ ⋅(x_p - x_r) - y_p - y_r)
                    q_add.clone()
                        * x_p.clone()
                        * x_q.clone()
                        * x_q_minus_x_p
                        * (lambda.clone() * x_p_minus_x_r - y_p.clone() - y_r.clone())
                },
            );

            // x_p⋅x_q⋅(y_q + y_p)⋅(λ^2 - x_p - x_q - x_r) = 0
            meta.create_gate("y_r check", |_| {
                let y_q_plus_y_p = y_q.clone() + y_p.clone(); // (y_q + y_p)
                let incomplete =
                    lambda.clone() * lambda.clone() - x_p.clone() - x_q.clone() - x_r.clone(); // (λ^2 - x_p - x_q - x_r)

                // q_add ⋅ x_p⋅x_q⋅(y_q + y_p)⋅(λ^2 - x_p - x_q - x_r)
                q_add.clone() * x_p.clone() * x_q.clone() * y_q_plus_y_p * incomplete
            });

            // x_p⋅x_q⋅(y_q + y_p)⋅(λ ⋅(x_p - x_r) - y_p - y_r) = 0
            meta.create_gate(
                "x_p⋅x_q⋅(y_q + y_p)⋅(λ ⋅(x_p - x_r) - y_p - y_r) = 0",
                |_| {
                    let y_q_plus_y_p = y_q.clone() + y_p.clone(); // (y_q + y_p)
                    let x_p_minus_x_r = x_p.clone() - x_r.clone(); // (x_p - x_r)

                    // q_add ⋅ x_p⋅x_q⋅(y_q + y_p)⋅(λ ⋅(x_p - x_r) - y_p - y_r)
                    q_add.clone()
                        * x_p.clone()
                        * x_q.clone()
                        * y_q_plus_y_p
                        * (lambda.clone() * x_p_minus_x_r - y_p.clone() - y_r.clone())
                },
            );

            meta.create_gate("(1 - x_p * β) * (x_r - x_q) = 0", |_| {
                q_add.clone() * (one.clone() - if_beta.clone()) * (x_r.clone() - x_q.clone())
            });

            meta.create_gate("(1 - x_p * β) * (y_r - y_q) = 0", |_| {
                q_add.clone() * (one.clone() - if_beta) * (y_r.clone() - y_q.clone())
            });

            meta.create_gate("(1 - x_q * γ) * (x_r - x_p) = 0", |_| {
                q_add.clone() * (one.clone() - if_gamma.clone()) * (x_r.clone() - x_p.clone())
            });

            meta.create_gate("(1 - x_q * γ) * (y_r - y_p) = 0", |_| {
                q_add.clone() * (one.clone() - if_gamma) * (y_r.clone() - y_p.clone())
            });

            // ((1 - (x_q - x_p) * α - (y_q + y_p) * δ)) * x_r
            meta.create_gate("((1 - (x_q - x_p) * α - (y_q + y_p) * δ)) * x_r", |_| {
                q_add.clone() * (one.clone() - if_alpha.clone() - if_delta.clone()) * x_r.clone()
            });

            // ((1 - (x_q - x_p) * α - (y_q + y_p) * δ)) * y_r
            meta.create_gate("((1 - (x_q - x_p) * α - (y_q + y_p) * δ)) * y_r", |_| {
                q_add * (one - if_alpha - if_delta) * y_r
            });
        }
    }

    pub(super) fn assign_region<C: CurveAffine>(
        &self,
        a: &EccPoint<C>,
        b: &EccPoint<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        // Enable `q_add` selector
        self.q_add.enable(region, offset)?;

        // Copy point `a` into `x_p`, `y_p` columns
        util::assign_and_constrain(region, || "x_p", self.x_p, offset, &a.x, &self.perm)?;
        util::assign_and_constrain(region, || "y_p", self.y_p, offset, &a.y, &self.perm)?;

        // Copy point `b` into `x_qr`, `y_qr` columns
        util::assign_and_constrain(region, || "x_q", self.x_qr, offset, &b.x, &self.perm)?;
        util::assign_and_constrain(region, || "y_q", self.y_qr, offset, &b.y, &self.perm)?;

        let (x_p, y_p) = (a.x.value, a.y.value);
        let (x_q, y_q) = (b.x.value, b.y.value);

        // inv0(x) evaluates to 0 if x = 0, and 1/x otherwise.

        // Assign α = inv0(x_q - x_p)
        region.assign_advice(
            || "α",
            self.alpha,
            offset,
            || {
                let x_p = x_p.ok_or(Error::SynthesisError)?;
                let x_q = x_q.ok_or(Error::SynthesisError)?;

                let alpha = if x_q != x_p {
                    (x_q - x_p).invert().unwrap()
                } else {
                    C::Base::zero()
                };
                Ok(alpha)
            },
        )?;

        // Assign β = inv0(x_p)
        region.assign_advice(
            || "β",
            self.beta,
            offset,
            || {
                let x_p = x_p.ok_or(Error::SynthesisError)?;

                let beta = if x_p != C::Base::zero() {
                    x_p.invert().unwrap()
                } else {
                    C::Base::zero()
                };
                Ok(beta)
            },
        )?;

        // Assign γ = inv0(x_q)
        region.assign_advice(
            || "γ",
            self.gamma,
            offset,
            || {
                let x_q = x_q.ok_or(Error::SynthesisError)?;

                let gamma = if x_q != C::Base::zero() {
                    x_q.invert().unwrap()
                } else {
                    C::Base::zero()
                };
                Ok(gamma)
            },
        )?;

        // Assign δ = inv0(y_q + y_p) if x_q = x_p, 0 otherwise
        region.assign_advice(
            || "δ",
            self.delta,
            offset,
            || {
                let x_p = x_p.ok_or(Error::SynthesisError)?;
                let x_q = x_q.ok_or(Error::SynthesisError)?;
                let y_p = y_p.ok_or(Error::SynthesisError)?;
                let y_q = y_q.ok_or(Error::SynthesisError)?;

                let delta = if x_q == x_p {
                    if y_q != -y_p {
                        (y_q + y_p).invert().unwrap()
                    } else {
                        C::Base::zero()
                    }
                } else {
                    C::Base::zero()
                };
                Ok(delta)
            },
        )?;

        #[allow(clippy::collapsible_else_if)]
        // Assign lambda
        let lambda = x_p
            .zip(y_p)
            .zip(x_q)
            .zip(y_q)
            .map(|(((x_p, y_p), x_q), y_q)| {
                if x_q != x_p {
                    // λ = (y_q - y_p)/(x_q - x_p)
                    (y_q - y_p) * (x_q - x_p).invert().unwrap()
                } else {
                    if y_p != C::Base::zero() {
                        // 3(x_p)^2
                        let three_x_p_sq = C::Base::from_u64(3) * x_p * x_p;
                        // 2(y_p)
                        let two_y_p = C::Base::from_u64(2) * y_p;
                        // λ = 3(x_p)^2 / 2(y_p)
                        three_x_p_sq * two_y_p.invert().unwrap()
                    } else {
                        C::Base::zero()
                    }
                }
            });
        region.assign_advice(
            || "λ",
            self.lambda,
            offset,
            || lambda.ok_or(Error::SynthesisError),
        )?;

        // Assign x_r
        let x_r =
            x_p.zip(y_p)
                .zip(x_q)
                .zip(y_q)
                .zip(lambda)
                .map(|((((x_p, y_p), x_q), y_q), lambda)| {
                    if x_p == C::Base::zero() {
                        // 0 + Q = Q
                        x_q
                    } else if x_q == C::Base::zero() {
                        // P + 0 = P
                        x_p
                    } else if (x_q == x_p) && (y_q == -y_p) {
                        // P + (-P) maps to (0,0)
                        C::Base::zero()
                    } else {
                        // x_r = λ^2 - x_p - x_q
                        lambda * lambda - x_p - x_q
                    }
                });
        let x_r_cell = region.assign_advice(
            || "x_r",
            self.x_qr,
            offset + 1,
            || x_r.ok_or(Error::SynthesisError),
        )?;

        // Assign y_r
        let y_r = x_p.zip(y_p).zip(x_q).zip(y_q).zip(x_r).zip(lambda).map(
            |(((((x_p, y_p), x_q), y_q), x_r), lambda)| {
                if x_p == C::Base::zero() {
                    // 0 + Q = Q
                    y_q
                } else if x_q == C::Base::zero() {
                    // P + 0 = P
                    y_p
                } else if (x_q == x_p) && (y_q == -y_p) {
                    // P + (-P) maps to (0,0)
                    C::Base::zero()
                } else {
                    // y_r = λ(x_p - x_r) - y_p
                    lambda * (x_p - x_r) - y_p
                }
            },
        );
        let y_r_cell = region.assign_advice(
            || "y_r",
            self.y_qr,
            offset + 1,
            || y_r.ok_or(Error::SynthesisError),
        )?;

        Ok(EccPoint {
            x: CellValue::<C::Base>::new(x_r_cell, x_r),
            y: CellValue::<C::Base>::new(y_r_cell, y_r),
        })
    }
}
