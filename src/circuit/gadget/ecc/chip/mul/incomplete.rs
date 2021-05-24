use super::super::{CellValue, EccConfig, EccPoint};
use super::{ZValue, X, Y};
use crate::constants;
use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

pub(super) struct IncompleteConfig<C: CurveAffine> {
    // Number of bits covered by this incomplete range.
    num_bits: usize,
    // Selector used to constrain the cells used in incomplete addition.
    pub(super) q_mul: Selector,
    // Cumulative sum used to decompose the scalar.
    pub(super) z: Column<Advice>,
    // x-coordinate of the accumulator in each double-and-add iteration.
    pub(super) x_a: Column<Advice>,
    // x-coordinate of the point being added in each double-and-add iteration.
    pub(super) x_p: Column<Advice>,
    // y-coordinate of the point being added in each double-and-add iteration.
    pub(super) y_p: Column<Advice>,
    // lambda1 in each double-and-add iteration.
    pub(super) lambda1: Column<Advice>,
    // lambda2 in each double-and-add iteration.
    pub(super) lambda2: Column<Advice>,
    // Permutation
    pub(super) perm: Permutation,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> IncompleteConfig<C> {
    // Columns used in processing the `hi` bits of the scalar.
    // `x_p, y_p` are shared across the `hi` and `lo` halves.
    pub(super) fn into_hi_config(ecc_config: &EccConfig) -> Self {
        // Bits k_{254} to k_{4} inclusive are used in incomplete addition.
        // The `hi` half is k_{254} to k_{130} inclusive (length 125 bits).
        let incomplete_hi_len =
            (C::Scalar::NUM_BITS as usize - 1 - constants::NUM_COMPLETE_BITS) / 2;

        Self {
            num_bits: incomplete_hi_len,
            q_mul: ecc_config.q_mul_hi,
            z: ecc_config.bits,
            x_a: ecc_config.extras[0],
            x_p: ecc_config.P.0,
            y_p: ecc_config.P.1,
            lambda1: ecc_config.lambda.0,
            lambda2: ecc_config.lambda.1,
            perm: ecc_config.perm.clone(),
            _marker: PhantomData,
        }
    }

    // Columns used in processing the `lo` bits of the scalar.
    // `x_p, y_p` are shared across the `hi` and `lo` halves.
    pub(super) fn into_lo_config(ecc_config: &EccConfig) -> Self {
        // Bits k_{254} to k_{4} inclusive are used in incomplete addition.
        // The `lo` half is k_{129} to k_{4} inclusive (length 126 bits).
        let incomplete_lo_len = (C::Scalar::NUM_BITS as usize - constants::NUM_COMPLETE_BITS) / 2;

        Self {
            num_bits: incomplete_lo_len,
            q_mul: ecc_config.q_mul_lo,
            z: ecc_config.extras[1],
            x_a: ecc_config.extras[2],
            x_p: ecc_config.P.0,
            y_p: ecc_config.P.1,
            lambda1: ecc_config.extras[3],
            lambda2: ecc_config.extras[4],
            perm: ecc_config.perm.clone(),
            _marker: PhantomData,
        }
    }

    // Gate for incomplete addition part of variable-base scalar multiplication.
    pub(super) fn create_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        let q_mul = meta.query_selector(self.q_mul, Rotation::cur());
        let z_cur = meta.query_advice(self.z, Rotation::cur());
        let z_prev = meta.query_advice(self.z, Rotation::prev());
        let x_a_cur = meta.query_advice(self.x_a, Rotation::cur());
        let x_a_next = meta.query_advice(self.x_a, Rotation::next());
        let x_p_cur = meta.query_advice(self.x_p, Rotation::cur());
        let x_p_next = meta.query_advice(self.x_p, Rotation::next());
        let y_p_cur = meta.query_advice(self.y_p, Rotation::cur());
        let y_p_next = meta.query_advice(self.y_p, Rotation::next());
        let lambda1_cur = meta.query_advice(self.lambda1, Rotation::cur());
        let lambda2_cur = meta.query_advice(self.lambda2, Rotation::cur());
        let lambda1_next = meta.query_advice(self.lambda1, Rotation::next());
        let lambda2_next = meta.query_advice(self.lambda2, Rotation::next());

        // The current bit in the scalar decomposition, k_i = z_i - 2⋅z_{i+1}.
        // Recall that we assigned the cumulative variable `z_i` in descending order,
        // i from n down to 0. So z_{i+1} corresponds to the `z_prev` query.
        let k = z_cur - Expression::Constant(F::from_u64(2)) * z_prev;

        // (k_i) ⋅ (k_i - 1) = 0
        meta.create_gate("Scalar boolean decomposition", |_| {
            let bool_check = k.clone() * (k.clone() + Expression::Constant(-F::one()));
            q_mul.clone() * bool_check
        });

        // The base used in double-and-add remains constant. We check that its
        // x- and y- coordinates are the same throughout.
        meta.create_gate("x_p equality", |_| {
            q_mul.clone() * (x_p_cur.clone() - x_p_next.clone())
        });
        meta.create_gate("y_p equality", |_| {
            q_mul.clone() * (y_p_cur.clone() - y_p_next.clone())
        });

        // y_{A,i} = (λ_{1,i} + λ_{2,i})
        //           * (x_{A,i} - (λ_{1,i}^2 - x_{A,i} - x_{P,i})) / 2
        let y_a_cur = (lambda1_cur.clone() + lambda2_cur.clone())
            * (x_a_cur.clone()
                - (lambda1_cur.clone() * lambda1_cur.clone() - x_a_cur.clone() - x_p_cur.clone()))
            * F::TWO_INV;

        // y_{A,i+1} = (λ_{1,i+1} + λ_{2,i+1})
        //           * (x_{A,i+1} - (λ_{1,i+1}^2 - x_{A,i+1} - x_{P,i+1})) / 2
        let y_a_next = (lambda1_next.clone() + lambda2_next)
            * (x_a_next.clone()
                - (lambda1_next.clone() * lambda1_next - x_a_next.clone() - x_p_next))
            * F::TWO_INV;

        // λ_{1,i}⋅(x_{A,i} − x_{P,i}) − y_{A,i} + (2k_i - 1) y_{P,i} = 0
        meta.create_gate("Double-and-add lambda1", |_| {
            let expr = lambda1_cur.clone() * (x_a_cur.clone() - x_p_cur.clone()) - y_a_cur.clone()
                + (k * F::from_u64(2) + Expression::Constant(-F::one())) * y_p_cur.clone();
            q_mul.clone() * expr
        });

        // (λ_{1,i} + λ_{2,i})⋅(x_{A,i} − (λ_{1,i}^2 − x_{A,i} − x_{P,i})) − 2y_{A,i}) = 0
        meta.create_gate("Double-and-add expr0", |_| {
            let lambda_neg = lambda1_cur.clone() + lambda2_cur.clone();
            let lambda1_expr =
                lambda1_cur.clone() * lambda1_cur.clone() - x_a_cur.clone() - x_p_cur.clone();
            let expr = lambda_neg * (x_a_cur.clone() - lambda1_expr)
                - Expression::Constant(F::from_u64(2)) * y_a_cur.clone();
            q_mul.clone() * expr
        });

        // λ_{2,i}^2 − x_{A,i+1} −(λ_{1,i}^2 − x_{A,i} − x_{P,i}) − x_{A,i} = 0
        meta.create_gate("Double-and-add expr1", |_| {
            let expr1 = lambda2_cur.clone() * lambda2_cur.clone()
                - x_a_next.clone()
                - (lambda1_cur.clone() * lambda1_cur)
                + x_p_cur;

            q_mul.clone() * expr1
        });

        // λ_{2,i}⋅(x_{A,i} − x_{A,i+1}) − y_{A,i} − y_{A,i+1} = 0
        meta.create_gate("Double-and-add expr2", |_| {
            let expr2 = lambda2_cur * (x_a_cur - x_a_next) - y_a_cur - y_a_next;

            q_mul.clone() * expr2
        });
    }

    // We perform incomplete addition on all but the last three bits of the
    // decomposed scalar.
    // We split the bits in the incomplete addition range into "hi" and "lo"
    // halves and process them side by side, using the same rows but with
    // non-overlapping columns.
    // Returns (x, y, z).
    #[allow(clippy::type_complexity)]
    pub(super) fn double_and_add<F: FieldExt>(
        &self,
        region: &mut Region<'_, F>,
        base: &EccPoint<F>,
        offset: usize,
        bits: Option<Vec<bool>>,
        acc: (X<F>, Y<F>, ZValue<F>),
    ) -> Result<(X<F>, Y<F>, ZValue<F>), Error> {
        // Enable `q_mul` on all but the last row of the incomplete range.
        for row in 1..(self.num_bits - 1) {
            self.q_mul.enable(region, offset + row)?;
        }

        // Initialise the running `z` sum for the scalar bits.
        let mut z_val = acc.2.value;
        let mut z_cell = region.assign_advice(
            || "starting z",
            self.z,
            offset,
            || z_val.ok_or(Error::SynthesisError),
        )?;
        region.constrain_equal(&self.perm, z_cell, acc.2.cell)?;

        // Define `x_p`, `y_p`
        let x_p = base.x.value;
        let y_p = base.y.value;

        let offset = offset + 1;

        // Initialise acc
        let mut x_a = acc.0.value;
        let mut x_a_cell = region.assign_advice(
            || "starting x_a",
            self.x_a,
            offset,
            || x_a.ok_or(Error::SynthesisError),
        )?;
        region.constrain_equal(&self.perm, x_a_cell, acc.0.cell)?;
        let mut y_a = *acc.1;

        // Incomplete addition
        bits.map(|bits| -> Result<(), Error> {
            for (row, k) in bits.iter().enumerate() {
                // z_{i} = 2 * z_{i+1} + k_i
                z_val = z_val.map(|z_val| F::from_u64(2) * z_val + F::from_u64(*k as u64));
                z_cell = region.assign_advice(
                    || "z",
                    self.z,
                    row + offset,
                    || z_val.ok_or(Error::SynthesisError),
                )?;

                // Assign `x_p`, `y_p`
                region.assign_advice(
                    || "x_p",
                    self.x_p,
                    row + offset,
                    || x_p.ok_or(Error::SynthesisError),
                )?;
                region.assign_advice(
                    || "y_p",
                    self.y_p,
                    row + offset,
                    || y_p.ok_or(Error::SynthesisError),
                )?;

                // If the bit is set, use `y`; if the bit is not set, use `-y`
                let y_p = y_p.map(|y_p| if !k { -y_p } else { y_p });

                // Compute and assign λ1⋅(x_A − x_P) = y_A − y_P
                let lambda1 = y_a
                    .zip(y_p)
                    .zip(x_a)
                    .zip(x_p)
                    .map(|(((y_a, y_p), x_a), x_p)| (y_a - y_p) * (x_a - x_p).invert().unwrap());
                region.assign_advice(
                    || "lambda1",
                    self.lambda1,
                    row + offset,
                    || lambda1.ok_or(Error::SynthesisError),
                )?;

                // x_R = λ1^2 - x_A - x_P
                let x_r = lambda1
                    .zip(x_a)
                    .zip(x_p)
                    .map(|((lambda1, x_a), x_p)| lambda1 * lambda1 - x_a - x_p);

                // λ2 = (2(y_A) / (x_A - x_R)) - λ1
                let lambda2 =
                    lambda1
                        .zip(y_a)
                        .zip(x_a)
                        .zip(x_r)
                        .map(|(((lambda1, y_a), x_a), x_r)| {
                            F::from_u64(2) * y_a * (x_a - x_r).invert().unwrap() - lambda1
                        });
                region.assign_advice(
                    || "lambda2",
                    self.lambda2,
                    row + offset,
                    || lambda2.ok_or(Error::SynthesisError),
                )?;

                // Compute and assign `x_a` for the next row
                let x_a_new = lambda2
                    .zip(x_a)
                    .zip(x_r)
                    .map(|((lambda2, x_a), x_r)| lambda2 * lambda2 - x_a - x_r);
                y_a = lambda2
                    .zip(x_a)
                    .zip(x_a_new)
                    .zip(y_a)
                    .map(|(((lambda2, x_a), x_a_new), y_a)| lambda2 * (x_a - x_a_new) - y_a);
                x_a = x_a_new;
                x_a_cell = region.assign_advice(
                    || "x_a",
                    self.x_a,
                    row + offset + 1,
                    || x_a.ok_or(Error::SynthesisError),
                )?;
            }
            Ok(())
        });

        Ok((
            X(CellValue::<F>::new(x_a_cell, x_a)),
            Y(y_a),
            ZValue(CellValue::<F>::new(z_cell, z_val)),
        ))
    }
}
