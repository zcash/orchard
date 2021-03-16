use super::super::EccInstructions;
use super::{add_complete, double, util, CellValue, EccChip, EccPoint};
use crate::constants;

use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::{Chip, Region},
    plonk::{ConstraintSystem, Error, Expression},
};

pub(super) fn create_gate<C: CurveAffine>(
    meta: &mut ConstraintSystem<C::Base>,
    q_mul: Expression<C::Base>,
    x_a_cur: Expression<C::Base>,
    x_a_next: Expression<C::Base>,
    x_p_cur: Expression<C::Base>,
    x_p_next: Expression<C::Base>,
    lambda1_cur: Expression<C::Base>,
    lambda1_next: Expression<C::Base>,
    lambda2_cur: Expression<C::Base>,
    lambda2_next: Expression<C::Base>,
) {
    let y_a_cur = (lambda1_cur.clone() + lambda2_cur.clone())
        * (x_a_cur.clone()
            - (lambda1_cur.clone() * lambda1_cur.clone() - x_a_cur.clone() - x_p_cur.clone()))
        * C::Base::TWO_INV;

    let y_a_next = (lambda1_next.clone() + lambda2_next)
        * (x_a_next.clone() - (lambda1_next.clone() * lambda1_next - x_a_next.clone() - x_p_next))
        * C::Base::TWO_INV;

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

pub(super) fn assign_region<C: CurveAffine>(
    scalar: &<EccChip<C> as EccInstructions<C>>::ScalarVar,
    base: &<EccChip<C> as EccInstructions<C>>::Point,
    offset: usize,
    region: &mut Region<'_, EccChip<C>>,
    config: <EccChip<C> as Chip>::Config,
) -> Result<EccPoint<C::Base>, Error> {
    // Copy the scalar decomposition
    for (w, k) in scalar.k_bits.iter().enumerate() {
        util::assign_and_constrain(
            region,
            || format!("k[{:?}]", w),
            config.bits,
            w + offset,
            k,
            &config.perm_scalar,
        )?;
    }

    // Initialise acc := [2]B
    let mut acc = double::assign_region(&base, offset, region, config.clone()).unwrap();
    let mut x_a = acc.x.value;
    let mut y_a = acc.y.value;

    // Assign `x_a`. We don't assign `y_a` since that is not used in the efficient
    // double-and-add algorithm.
    let mut x_a_cell = region.assign_advice(
        || "x_a",
        config.A.0,
        offset,
        || x_a.ok_or(Error::SynthesisError),
    )?;

    assert_eq!(scalar.k_bits.len(), C::Scalar::NUM_BITS as usize);

    // Bits used in incomplete addition. k_{254} to k_{4} inclusive
    let incomplete_range = 0..(C::Scalar::NUM_BITS as usize - 1 - constants::NUM_COMPLETE_BITS);
    let k_incomplete = &scalar.k_bits[incomplete_range.clone()];

    // Bits used in complete addition. k_{3} to k_{1} inclusive
    // The LSB k_{0} is handled separately.
    let complete_range = (C::Scalar::NUM_BITS as usize - 1 - constants::NUM_COMPLETE_BITS)
        ..(C::Scalar::NUM_BITS as usize - 1);
    let k_complete = &scalar.k_bits[complete_range.clone()];

    // Enable `q_mul` on all but the last row of the incomplete range.
    for row in 0..(C::Scalar::NUM_BITS as usize - 2 - constants::NUM_COMPLETE_BITS) {
        config.q_mul.enable(region, offset + row)?;
    }

    // Incomplete addition
    for (row, k) in k_incomplete.iter().enumerate() {
        // // Enable `q_mul` selector
        // config.q_mul.enable(region, offset + row)?;

        let x_p = base.x.value;
        region.assign_advice(
            || "x_p",
            config.P.0,
            row + offset,
            || x_p.ok_or(Error::SynthesisError),
        )?;

        // If the bit is set, use `y`; if the bit is not set, use `-y`
        let y_p = base.y.value;
        let y_p = y_p.zip(k.value).map(|(mut y_p, k)| {
            if k == C::Base::zero() {
                y_p = -y_p;
            }
            y_p
        });

        // Compute and assign `lambda1`
        let lambda1 = y_a
            .zip(y_p)
            .zip(x_a)
            .zip(x_p)
            .map(|(((y_a, y_p), x_a), x_p)| (y_a - y_p) * (x_a - x_p).invert().unwrap());
        region.assign_advice(
            || "lambda1",
            config.lambda.0,
            row + offset,
            || lambda1.ok_or(Error::SynthesisError),
        )?;

        // Compute and assign `lambda2`
        let x_r = lambda1
            .zip(x_a)
            .zip(x_p)
            .map(|((lambda1, x_a), x_p)| lambda1 * lambda1 - x_a - x_p);
        let lambda2 = lambda1
            .zip(y_a)
            .zip(x_a)
            .zip(x_r)
            .map(|(((lambda1, y_a), x_a), x_r)| {
                C::Base::from_u64(2) * y_a * (x_a - x_r).invert().unwrap() - lambda1
            });
        region.assign_advice(
            || "lambda2",
            config.lambda.1,
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
            config.A.0,
            row + offset + 1,
            || x_a.ok_or(Error::SynthesisError),
        )?;
    }

    // Move from incomplete addition to complete addition
    {
        let row = C::Scalar::NUM_BITS as usize - 1 - constants::NUM_COMPLETE_BITS;

        let y_a_cell = region.assign_advice(
            || "y_a",
            config.A.1,
            row + offset,
            || y_a.ok_or(Error::SynthesisError),
        )?;
        acc = EccPoint {
            x: CellValue::new(x_a_cell, x_a),
            y: CellValue::new(y_a_cell, y_a),
        };
    }

    // Complete addition
    for ((iter, row), k) in complete_range.clone().enumerate().zip(k_complete.iter()) {
        // Each iteration uses two rows
        let row = row + iter;

        let x_p = base.x.value;
        let x_p_cell = region.assign_advice(
            || "x_p",
            config.P.0,
            row + offset,
            || x_p.ok_or(Error::SynthesisError),
        )?;

        // If the bit is set, use `y`; if the bit is not set, use `-y`
        let y_p = base.y.value;
        let y_p = y_p.zip(k.value).map(|(mut y_p, k)| {
            if k == C::Base::zero() {
                y_p = -y_p;
            }
            y_p
        });
        let y_p_cell = region.assign_advice(
            || "y_p",
            config.P.1,
            row + offset,
            || y_p.ok_or(Error::SynthesisError),
        )?;
        let p = EccPoint {
            x: CellValue::new(x_p_cell, x_p),
            y: CellValue::new(y_p_cell, y_p),
        };

        // Acc + U
        let tmp_acc = add_complete::assign_region(&p, &acc, row + offset, region, config.clone())?;

        // Copy acc from `x_a`, `y_a` over to `x_p`, `y_p` on the next row
        let acc_x = util::assign_and_constrain(
            region,
            || "copy acc x_a",
            config.P.0,
            row + offset + 1,
            &acc.x,
            &config.perm_sum,
        )?;
        let acc_y = util::assign_and_constrain(
            region,
            || "copy acc y_a",
            config.P.1,
            row + offset + 1,
            &acc.y,
            &config.perm_sum,
        )?;

        acc = EccPoint { x: acc_x, y: acc_y };

        // Acc + U + Acc
        acc =
            add_complete::assign_region(&acc, &tmp_acc, row + offset + 1, region, config.clone())?;
    }

    // Process the least significant bit
    let k_0_row = incomplete_range.len() + complete_range.len() * 2;
    let k_0 = &scalar.k_bits[C::Scalar::NUM_BITS as usize - 1].value;

    k_0.map(|k_0| {
        // If `k_0` is 0, return `Acc - P`
        if k_0 == C::Base::zero() {
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
                x: CellValue::new(x_p_cell, x_p),
                y: CellValue::new(y_p_cell, y_p),
            };

            // Return the result of the final complete addition as `[scalar]B`
            add_complete::assign_region(&p, &acc, k_0_row + offset, region, config.clone())
        } else {
            // If `k_0` is 1, simply return `Acc`
            Ok(acc)
        }
    })
    .unwrap_or(Err(Error::SynthesisError))
}
