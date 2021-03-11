use super::super::EccInstructions;
use super::{add, util, witness_point, EccChip, EccFixedPoints, EccPoint};
use crate::constants;

use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::{Chip, Region},
    plonk::{Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

pub(super) fn create_gate<C: CurveAffine>(
    meta: &mut ConstraintSystem<C::Base>,
    number_base: usize,
    lagrange_coeffs: [Column<Fixed>; 8],
    q_mul_fixed: Expression<C::Base>,
    x_p: Expression<C::Base>,
    y_p: Expression<C::Base>,
    k: Expression<C::Base>,
    u: Expression<C::Base>,
    z: Expression<C::Base>,
) {
    // Check interpolation of x-coordinate
    meta.create_gate("fixed-base scalar mul (x)", |meta| {
        let k_pow: Vec<Expression<C::Base>> = (0..number_base)
            .map(|pow| {
                (0..pow).fold(Expression::Constant(C::Base::one()), |acc, _| {
                    acc * k.clone()
                })
            })
            .collect();

        let interpolated_x = k_pow.iter().zip(lagrange_coeffs.iter()).fold(
            Expression::Constant(C::Base::zero()),
            |acc, (k_pow, coeff)| acc + (k_pow.clone() * meta.query_fixed(*coeff, Rotation::cur())),
        );

        q_mul_fixed.clone() * (interpolated_x - x_p)
    });

    // Check that `y + z = u^2`, where `z` is fixed and `u`, `y` are witnessed
    meta.create_gate("fixed-base scalar mul (y)", |_| {
        q_mul_fixed * (u.clone() * u - y_p - z)
    });
}

pub(super) fn assign_region<C: CurveAffine>(
    scalar: &<EccChip<C> as EccInstructions<C>>::ScalarFixed,
    base: &<EccChip<C> as EccInstructions<C>>::FixedPoint,
    offset: usize,
    region: &mut Region<'_, EccChip<C>>,
    config: <EccChip<C> as Chip>::Config,
    num_windows: usize,
) -> Result<EccPoint<C::Base>, Error> {
    // Assign fixed columns for given fixed base
    for w in 0..num_windows {
        // Enable `q_mul_fixed` selector
        config.q_mul_fixed.enable(region, w + offset)?;

        for k in 0..(1 << config.window_width) {
            // Assign x-coordinate Lagrange interpolation coefficients
            region.assign_fixed(
                || {
                    format!(
                        "Lagrange interpolation coefficient for window: {:?}, k: {:?}",
                        w, k
                    )
                },
                config.lagrange_coeffs[k],
                w + offset,
                || {
                    base.lagrange_coeffs
                        .as_ref()
                        .map(|c| c[w][k])
                        .ok_or(Error::SynthesisError)
                },
            )?;
        }
        // Assign z-values for each window
        region.assign_fixed(
            || format!("z-value for window: {:?}", w),
            config.fixed_z,
            w + offset,
            || {
                base.z
                    .as_ref()
                    .map(|z| C::Base::from_u64(z[w]))
                    .ok_or(Error::SynthesisError)
            },
        )?;
    }

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

    // Get the value of the fixed base
    let b = match base.fixed_point {
        EccFixedPoints::CommitIvkR(inner) => inner.0.value(),
        EccFixedPoints::NoteCommitR(inner) => inner.0.value(),
        EccFixedPoints::NullifierK(inner) => inner.0.value(),
        EccFixedPoints::ValueCommitR(inner) => inner.0.value(),
        EccFixedPoints::ValueCommitV(inner) => inner.0.value(),
    };

    // The scalar decomposition was done in the base field. For computation
    // outside the circuit, we now convert them back into the scalar field.
    let k = scalar
        .k_bits
        .iter()
        .map(|bits| {
            bits.value
                .map(|value| C::Scalar::from_bytes(&value.to_bytes()).unwrap())
        })
        .collect::<Vec<_>>();

    // The scalar decomposition is guaranteed to be in three-bit windows,
    // so we also cast the least significant byte in their serialisation
    // into usize for convenient indexing into `u`-values
    let k_usize = scalar
        .k_bits
        .iter()
        .map(|bits| bits.value.map(|value| value.to_bytes()[0] as usize))
        .collect::<Vec<_>>();

    // This is 2^w, where w is the window width
    let number_base = C::Scalar::from_u64(1u64 << config.window_width);

    // Process the least significant window outside the for loop
    let mul_b = k[0].map(|k_0| b * (k_0 + C::Scalar::one()));
    let mul_b = witness_point::assign_region(
        mul_b.map(|point| point.to_affine()),
        0,
        region,
        config.clone(),
    )?;

    // Assign u = (y_p + z_w).sqrt() for the least significant window
    {
        let u_val = base.u.as_ref().zip(k_usize[0]).map(|(u, k_0)| u[0][k_0]);
        region.assign_advice(
            || "u",
            config.u,
            offset,
            || u_val.ok_or(Error::SynthesisError),
        )?;
    }

    // Initialise the point which will cumulatively sum to [scalar]B.
    // Copy and assign `mul_b` to x_a, y_a columns on the next row
    let x_sum = util::assign_and_constrain(
        region,
        || "initialize sum x",
        config.A.0,
        offset + 1,
        &mul_b.x,
        &config.perm_sum,
    )?;
    let y_sum = util::assign_and_constrain(
        region,
        || "initialize sum y",
        config.A.1,
        offset + 1,
        &mul_b.y,
        &config.perm_sum,
    )?;

    let mut sum = EccPoint { x: x_sum, y: y_sum };

    // Process all windows excluding least and most significant windows
    for (w, k) in k[1..(k.len() - 1)].iter().enumerate() {
        // Offset index by 1 since we already assigned row 0 outside this loop
        let w = w + 1;

        // Compute [(k_w + 1) ⋅ 8^w]B
        let mul_b = k.map(|k| b * (k + C::Scalar::one()) * number_base.pow(&[w as u64, 0, 0, 0]));
        let mul_b = witness_point::assign_region(
            mul_b.map(|point| point.to_affine()),
            offset + w,
            region,
            config.clone(),
        )?;

        // Assign u = (y_p + z_w).sqrt()
        let u_val = base.u.as_ref().zip(k_usize[w]).map(|(u, k)| u[w][k]);
        region.assign_advice(|| "u", config.u, w, || u_val.ok_or(Error::SynthesisError))?;

        // Add to the cumulative sum
        sum = add::assign_region(&mul_b, &sum, offset + w, region, config.clone())?;
    }

    // Process most significant window outside the for loop
    let offset_sum = (0..(num_windows - 1)).fold(C::ScalarExt::zero(), |acc, w| {
        acc + number_base.pow(&[w as u64, 0, 0, 0])
    });

    // `scalar = [k * 8^84 - offset_sum]`, where `offset_sum = \sum_{j = 0}^{83} 8^j`.
    let scalar = k[k.len() - 1]
        .map(|k| k * number_base.pow(&[(num_windows - 1) as u64, 0, 0, 0]) - offset_sum);
    let mul_b = scalar.map(|scalar| b * scalar);
    let mul_b = witness_point::assign_region(
        mul_b.map(|point| point.to_affine()),
        offset + num_windows - 1,
        region,
        config.clone(),
    )?;

    // Assign u = (y_p + z_w).sqrt() for the most significant window
    {
        let u_val = base
            .u
            .as_ref()
            .zip(k_usize[num_windows - 1])
            .map(|(u, k)| u[num_windows - 1][k]);
        region.assign_advice(
            || "u",
            config.u,
            offset + num_windows - 1,
            || u_val.ok_or(Error::SynthesisError),
        )?;
    }

    // Add to the cumulative sum and return the final result as `[scalar]B`.
    add::assign_region(&mul_b, &sum, offset + num_windows - 1, region, config)
}