use super::{add, double, util, CellValue, EccConfig, EccPoint};
use crate::constants::NUM_COMPLETE_BITS;
use std::ops::Deref;

use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
};

#[allow(clippy::too_many_arguments)]
pub(super) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_mul: Expression<F>,
    z_cur: Expression<F>,
    z_prev: Expression<F>,
    x_a_cur: Expression<F>,
    x_a_next: Expression<F>,
    x_p_cur: Expression<F>,
    x_p_next: Expression<F>,
    y_p_cur: Expression<F>,
    y_p_next: Expression<F>,
    lambda1_cur: Expression<F>,
    lambda2_cur: Expression<F>,
    lambda1_next: Expression<F>,
    lambda2_next: Expression<F>,
) {
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
        * (x_a_next.clone() - (lambda1_next.clone() * lambda1_next - x_a_next.clone() - x_p_next))
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

/// Gate used to check scalar decomposition is correct.
/// This is used to check the bits used in complete addition, since the incomplete
/// addition gate (controlled by `q_mul`) already checks scalar decomposition for
/// the other bits.
pub(super) fn create_decompose_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_mul_decompose: Expression<F>,
    z_cur: Expression<F>,
    z_prev: Expression<F>,
) {
    meta.create_gate("Decompose scalar ", |_| {
        // k_{i} = z_{i} - 2⋅z_{i+1}
        let k = z_cur.clone() - Expression::Constant(F::from_u64(2)) * z_prev;
        // (k_i) ⋅ (k_i - 1) = 0
        let bool_check = k.clone() * (k + Expression::Constant(-F::one()));

        q_mul_decompose.clone() * bool_check
    });
}

/// Gate used to check final scalar is recovered.
pub(super) fn create_final_scalar_gate<C: CurveAffine>(
    meta: &mut ConstraintSystem<C::Base>,
    scalar: Expression<C::Base>,
    z_cur: Expression<C::Base>,
) {
    meta.create_gate("Decompose scalar", |_| {
        // q = 2^254 + t_q is the scalar field of Pallas
        let t_q = -(C::Scalar::from_u128(1u128 << 127).square());
        let t_q = C::Base::from_bytes(&t_q.to_bytes()).unwrap();

        // Check that `k = scalar + t_q`
        scalar.clone() * (scalar + Expression::Constant(t_q) - z_cur)
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    scalar: &CellValue<C::Base>,
    base: &EccPoint<C::Base>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
) -> Result<EccPoint<C::Base>, Error> {
    // We bisect the boolean decomposition into `hi` and `lo` halves, and
    // process these halves "in parallel" (i.e. on the same rows, but on
    // non-overlapping columns).
    let hi_columns = IncompleteColumns {
        q_mul: config.q_mul_hi,
        z: config.bits,
        x_a: config.extras[0],
        lambda: config.lambda,
    };
    let lo_columns = IncompleteColumns {
        q_mul: config.q_mul_lo,
        z: config.extras[1],
        x_a: config.extras[2],
        lambda: (config.extras[3], config.extras[4]),
    };

    // Decompose the scalar bitwise (big-endian bit order).
    let k_bits = decompose_scalar::<C>(scalar.value.unwrap());

    // Bits used in incomplete addition. k_{254} to k_{4} inclusive
    let incomplete_range = 0..(C::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS);
    let k_incomplete = &k_bits[incomplete_range];
    let k_incomplete_hi = &k_incomplete[..k_incomplete.len() / 2];
    let k_incomplete_lo = &k_incomplete[k_incomplete.len() / 2..];

    // Bits used in complete addition. k_{3} to k_{1} inclusive
    // The LSB k_{0} is handled separately.
    let complete_range =
        (C::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS)..(C::Scalar::NUM_BITS as usize - 1);
    let k_complete = &k_bits[complete_range.clone()];

    // Initialize the accumulator a [2]base
    let acc = double::assign_region(&base, offset, region, config.clone())?;
    // Initialize the running sum for scalar decomposition to zero
    let z_val = C::Base::zero();
    let z_cell = region.assign_advice(|| "initial z", hi_columns.z, offset + 1, || Ok(z_val))?;
    let z = CellValue::new(z_cell, Some(z_val));

    // Double-and-add (incomplete addition) for the `hi` half of the scalar decomposition
    let (x, y_a, z) = add_incomplete::<C>(
        region,
        &base,
        config.clone(),
        offset + 1,
        hi_columns,
        k_incomplete_hi,
        (X(acc.x.clone()), Y(acc.y.value), ZValue(z)),
    )?;

    // Double-and-add (incomplete addition) for the `lo` half of the scalar decomposition
    let (x, y_a, z) = add_incomplete::<C>(
        region,
        &base,
        config.clone(),
        offset + 1,
        lo_columns,
        k_incomplete_lo,
        (x, y_a, z),
    )?;

    // Move from incomplete addition to complete addition
    let mut acc = {
        let y_a_col = config.extras[1];
        let row = k_incomplete_lo.len() + 2;

        let y_a_cell = region.assign_advice(
            || "y_a",
            y_a_col,
            row + offset,
            || y_a.ok_or(Error::SynthesisError),
        )?;
        util::assign_and_constrain(
            region,
            || "Copy z from incomplete to complete",
            config.bits.into(),
            row + offset,
            &z,
            &config.perm,
        )?;
        EccPoint {
            x: x.0,
            y: CellValue::<C::Base>::new(y_a_cell, *y_a),
        }
    };

    let mut z_val = z.value;
    // Complete addition
    for (iter, k) in k_complete.iter().enumerate() {
        // Each iteration uses 4 rows (two complete additions)
        let row = k_incomplete_lo.len() + 4 * iter + 3;

        // Check scalar decomposition here
        region.assign_advice(
            || "z",
            config.bits,
            row + offset - 1,
            || z_val.ok_or(Error::SynthesisError),
        )?;
        z_val = z_val.map(|z_val| C::Base::from_u64(2) * z_val + C::Base::from_u64(*k as u64));
        region.assign_advice(
            || "z",
            config.bits,
            row + offset,
            || z_val.ok_or(Error::SynthesisError),
        )?;
        config.q_mul_decompose.enable(region, row + offset)?;

        let x_p = base.x.value;
        let x_p_cell = region.assign_advice(
            || "x_p",
            config.P.0,
            row + offset,
            || x_p.ok_or(Error::SynthesisError),
        )?;

        // If the bit is set, use `y`; if the bit is not set, use `-y`
        let y_p = base.y.value;
        let y_p = y_p.map(|y_p| if !k { -y_p } else { y_p });

        let y_p_cell = region.assign_advice(
            || "y_p",
            config.P.1,
            row + offset,
            || y_p.ok_or(Error::SynthesisError),
        )?;
        let p = EccPoint {
            x: CellValue::<C::Base>::new(x_p_cell, x_p),
            y: CellValue::<C::Base>::new(y_p_cell, y_p),
        };

        // Acc + U
        let tmp_acc = add::assign_region::<C>(&p, &acc, row + offset, region, config.clone())?;

        // Copy acc from `x_a`, `y_a` over to `x_p`, `y_p` on the next row
        let acc_x = util::assign_and_constrain(
            region,
            || "copy acc x_a",
            config.P.0.into(),
            row + offset + 2,
            &acc.x,
            &config.perm,
        )?;
        let acc_y = util::assign_and_constrain(
            region,
            || "copy acc y_a",
            config.P.1.into(),
            row + offset + 2,
            &acc.y,
            &config.perm,
        )?;

        acc = EccPoint { x: acc_x, y: acc_y };

        // Acc + P + Acc
        acc = add::assign_region::<C>(&acc, &tmp_acc, row + offset + 2, region, config.clone())?;
    }

    // Process the least significant bit
    let k_0_row = k_incomplete_lo.len() + complete_range.len() * 4 + 4;
    let k_0 = &k_bits[C::Scalar::NUM_BITS as usize - 1];

    // Check that we recover the original scalar.
    //
    // NB: We assume that the scalar fits in the curve's base field. This is not
    // true in general, and in particular for the Pallas curve, whose scalar field
    // `Fq` is larger than its base field `Fp`.
    //
    // However, the only use of variable-base scalar mul in the Orchard protocol
    // is in deriving diversified addresses `[ivk] g_d`,  and `ivk` is guaranteed
    // to be in the base field of the curve. (See non-normative notes in
    // https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents.)

    z_val = z_val.map(|z_val| C::Base::from_u64(2) * z_val + C::Base::from_u64(*k_0 as u64));
    region.assign_advice(
        || "final z",
        config.bits,
        k_0_row + offset,
        || z_val.ok_or(Error::SynthesisError),
    )?;
    region.assign_fixed(
        || "original k",
        config.mul_decompose,
        k_0_row + offset,
        || Ok(C::Base::from_bytes(&scalar.value.unwrap().to_bytes()).unwrap()),
    )?;

    // If `k_0` is 0, return `Acc - P`
    if !k_0 {
        let (x_p, y_p) = (base.x.value, base.y.value.map(|y_p| -y_p));
        let x_p_cell = region.assign_advice(
            || "x_p",
            config.P.0,
            k_0_row + offset,
            || x_p.ok_or(Error::SynthesisError),
        )?;

        let y_p_cell = region.assign_advice(
            || "y_p",
            config.P.1,
            k_0_row + offset,
            || y_p.ok_or(Error::SynthesisError),
        )?;
        let p = EccPoint {
            x: CellValue::<C::Base>::new(x_p_cell, x_p),
            y: CellValue::<C::Base>::new(y_p_cell, y_p),
        };

        // Return the result of the final complete addition as `[scalar]B`
        add::assign_region::<C>(&p, &acc, k_0_row + offset, region, config)
    } else {
        // If `k_0` is 1, simply return `Acc`
        Ok(acc)
    }
}

#[derive(Copy, Clone, Debug)]
struct IncompleteColumns {
    q_mul: Selector,
    z: Column<Advice>,
    x_a: Column<Advice>,
    lambda: (Column<Advice>, Column<Advice>),
}

#[derive(Clone, Debug)]
struct X<F: FieldExt>(CellValue<F>);
impl<F: FieldExt> Deref for X<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
struct Y<F: FieldExt>(Option<F>);
impl<F: FieldExt> Deref for Y<F> {
    type Target = Option<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
struct ZValue<F: FieldExt>(CellValue<F>);
impl<F: FieldExt> Deref for ZValue<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// We perform incomplete addition on all but the last three bits of the
// decomposed scalar.
// We split the bits in the incomplete addition range into "hi" and "lo"
// halves and process them side by side, using the same rows but with
// non-overlapping columns.
// Returns (x, y, z).
#[allow(clippy::type_complexity)]
fn add_incomplete<C: CurveAffine>(
    region: &mut Region<'_, C::Base>,
    base: &EccPoint<C::Base>,
    config: EccConfig,
    offset: usize,
    columns: IncompleteColumns,
    bits: &[bool],
    acc: (X<C::Base>, Y<C::Base>, ZValue<C::Base>),
) -> Result<(X<C::Base>, Y<C::Base>, ZValue<C::Base>), Error> {
    // Initialise the running `z` sum for the scalar bits.
    let mut z_val = acc.2.value;
    let mut z_cell = region.assign_advice(
        || "starting z",
        columns.z,
        offset,
        || z_val.ok_or(Error::SynthesisError),
    )?;
    region.constrain_equal(&config.perm, z_cell, acc.2.cell)?;

    // Define `x_p`, `y_p`
    let x_p = base.x.value;
    let y_p = base.y.value;

    let offset = offset + 1;

    // Initialise acc
    let mut x_a = acc.0.value;
    let mut x_a_cell = region.assign_advice(
        || "starting x_a",
        columns.x_a,
        offset,
        || x_a.ok_or(Error::SynthesisError),
    )?;
    region.constrain_equal(&config.perm, x_a_cell, acc.0.cell)?;
    let mut y_a = *acc.1;

    // Enable `q_mul` on all but the last row of the incomplete range.
    for row in 1..(bits.len() - 1) {
        columns.q_mul.enable(region, offset + row)?;
    }

    // Incomplete addition
    for (row, k) in bits.iter().enumerate() {
        // z_{i} = 2 * z_{i+1} + k_i
        z_val = z_val.map(|z_val| C::Base::from_u64(2) * z_val + C::Base::from_u64(*k as u64));
        z_cell = region.assign_advice(
            || "z",
            columns.z,
            row + offset,
            || z_val.ok_or(Error::SynthesisError),
        )?;

        // Assign `x_p`, `y_p`
        region.assign_advice(
            || "x_p",
            config.P.0,
            row + offset,
            || x_p.ok_or(Error::SynthesisError),
        )?;
        region.assign_advice(
            || "y_p",
            config.P.1,
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
            columns.lambda.0,
            row + offset,
            || lambda1.ok_or(Error::SynthesisError),
        )?;

        // x_R = λ1^2 - x_A - x_P
        let x_r = lambda1
            .zip(x_a)
            .zip(x_p)
            .map(|((lambda1, x_a), x_p)| lambda1 * lambda1 - x_a - x_p);

        // λ2 = (2(y_A) / (x_A - x_R)) - λ1
        let lambda2 = lambda1
            .zip(y_a)
            .zip(x_a)
            .zip(x_r)
            .map(|(((lambda1, y_a), x_a), x_r)| {
                C::Base::from_u64(2) * y_a * (x_a - x_r).invert().unwrap() - lambda1
            });
        region.assign_advice(
            || "lambda2",
            columns.lambda.1,
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
            columns.x_a,
            row + offset + 1,
            || x_a.ok_or(Error::SynthesisError),
        )?;
    }
    Ok((
        X(CellValue::<C::Base>::new(x_a_cell, x_a)),
        Y(y_a),
        ZValue(CellValue::<C::Base>::new(z_cell, z_val)),
    ))
}

fn decompose_scalar<C: CurveAffine>(scalar: C::Base) -> Vec<bool> {
    // Cast into scalar field
    let scalar = C::Scalar::from_bytes(&scalar.to_bytes()).unwrap();

    // The scalar field `F_q = 2^254 + t_q`
    let t_q = -(C::Scalar::from_u128(1u128 << 127).square());

    // We will witness `k = scalar + t_q`
    // `k` is decomposed bitwise in-circuit for our double-and-add algorithm.
    let k = scalar + t_q;

    // `k` is decomposed bitwise (big-endian) into `[k_n, ..., k_0]`, where
    // each `k_i` is a bit and `scalar = k_n * 2^n + ... + k_1 * 2 + k_0`.
    let mut bits: Vec<bool> = k
        .to_le_bits()
        .into_iter()
        .take(C::Scalar::NUM_BITS as usize)
        .collect();
    bits.reverse();
    assert_eq!(bits.len(), C::Scalar::NUM_BITS as usize);

    bits
}
