use super::{
    add_incomplete, util, witness_point, EccConfig, EccPoint, EccScalarFixed, OrchardFixedBase,
    OrchardFixedBases,
};
use crate::constants;

use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{Column, ConstraintSystem, Error, Expression, Fixed},
    poly::Rotation,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    lagrange_coeffs: [Column<Fixed>; constants::H],
    q_mul_fixed: Expression<F>,
    x_p: Expression<F>,
    y_p: Expression<F>,
    k: Expression<F>,
    u: Expression<F>,
    z: Expression<F>,
) {
    // Check interpolation of x-coordinate
    meta.create_gate("fixed-base scalar mul (x)", |meta| {
        let k_pow: Vec<Expression<F>> = (0..constants::H)
            .map(|pow| (0..pow).fold(Expression::Constant(F::one()), |acc, _| acc * k.clone()))
            .collect();

        let interpolated_x = k_pow
            .iter()
            .zip(lagrange_coeffs.iter())
            .fold(Expression::Constant(F::zero()), |acc, (k_pow, coeff)| {
                acc + (k_pow.clone() * meta.query_fixed(*coeff, Rotation::cur()))
            });

        q_mul_fixed.clone() * (interpolated_x - x_p)
    });

    // Check that `y + z = u^2`, where `z` is fixed and `u`, `y` are witnessed
    meta.create_gate("fixed-base scalar mul (y)", |_| {
        q_mul_fixed * (u.clone() * u - y_p - z)
    });
}

#[allow(non_snake_case)]
pub(super) fn assign_region<C: CurveAffine>(
    scalar: &EccScalarFixed<C>,
    base: &OrchardFixedBase<C>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
) -> Result<EccPoint<C::Base>, Error> {
    // Rename columns for `mul_fixed` context
    let A = (config.extras[0], config.extras[1]);
    let mul_fixed_u = config.extras[2];

    // Assign fixed columns for given fixed base
    for w in 0..constants::NUM_WINDOWS {
        // Enable `q_mul_fixed` selector
        config.q_mul_fixed.enable(region, w + offset)?;

        // Assign x-coordinate Lagrange interpolation coefficients
        for k in 0..(constants::H) {
            region.assign_fixed(
                || {
                    format!(
                        "Lagrange interpolation coeff for window: {:?}, k: {:?}",
                        w, k
                    )
                },
                config.lagrange_coeffs[k],
                w + offset,
                || Ok(base.lagrange_coeffs.0[w].0[k]),
            )?;
        }

        // Assign z-values for each window
        region.assign_fixed(
            || format!("z-value for window: {:?}", w),
            config.fixed_z.into(),
            w + offset,
            || Ok(base.z.0[w]),
        )?;
    }

    // Copy the scalar decomposition
    for (w, k) in scalar.k_bits.iter().enumerate() {
        util::assign_and_constrain(
            region,
            || format!("k[{:?}]", w),
            config.bits.into(),
            w + offset,
            k,
            &config.perm,
        )?;
    }

    // Get the value of the fixed base. `ValueCommitV` is excluded here
    // since it is only used in multiplication with a short signed exponent.
    let b = match base.base {
        OrchardFixedBases::CommitIvkR(inner) => inner.0.value(),
        OrchardFixedBases::NoteCommitR(inner) => inner.0.value(),
        OrchardFixedBases::NullifierK(inner) => inner.0.value(),
        OrchardFixedBases::ValueCommitR(inner) => inner.0.value(),
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
    let h = C::Scalar::from_u64(constants::H as u64);

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
        let u_val = k_usize[0].map(|k_0| base.u.0[0].0[k_0]);
        region.assign_advice(
            || "u",
            mul_fixed_u,
            offset,
            || u_val.ok_or(Error::SynthesisError),
        )?;
    }

    // Initialise the point which will cumulatively sum to [scalar]B.
    // Copy and assign `mul_b` to x_a, y_a columns on the next row
    let x_sum = util::assign_and_constrain(
        region,
        || "initialize sum x",
        A.0.into(),
        offset + 1,
        &mul_b.x,
        &config.perm,
    )?;
    let y_sum = util::assign_and_constrain(
        region,
        || "initialize sum y",
        A.1.into(),
        offset + 1,
        &mul_b.y,
        &config.perm,
    )?;

    let mut sum = EccPoint { x: x_sum, y: y_sum };

    // Process all windows excluding least and most significant windows
    for (w, k) in k[1..(k.len() - 1)].iter().enumerate() {
        // Offset index by 1 since we already assigned row 0 outside this loop
        let w = w + 1;

        // Compute [(k_w + 1) ⋅ 8^w]B
        let mul_b = k.map(|k| b * (k + C::Scalar::one()) * h.pow(&[w as u64, 0, 0, 0]));
        let mul_b = witness_point::assign_region(
            mul_b.map(|point| point.to_affine()),
            offset + w,
            region,
            config.clone(),
        )?;

        // Assign u = (y_p + z_w).sqrt()
        let u_val = k_usize[w].map(|k| base.u.0[w].0[k]);
        region.assign_advice(
            || "u",
            mul_fixed_u,
            w,
            || u_val.ok_or(Error::SynthesisError),
        )?;

        // Add to the cumulative sum
        sum = add_incomplete::assign_region(&mul_b, &sum, offset + w, region, config.clone())?;
    }

    // Process most significant window outside the for loop
    let offset_sum = (0..(constants::NUM_WINDOWS - 1)).fold(C::ScalarExt::zero(), |acc, w| {
        acc + h.pow(&[w as u64, 0, 0, 0])
    });

    // `scalar = [k * 8^84 - offset_sum]`, where `offset_sum = \sum_{j = 0}^{83} 8^j`.
    let scalar = k[k.len() - 1]
        .map(|k| k * h.pow(&[(constants::NUM_WINDOWS - 1) as u64, 0, 0, 0]) - offset_sum);
    let mul_b = scalar.map(|scalar| b * scalar);
    let mul_b = witness_point::assign_region(
        mul_b.map(|point| point.to_affine()),
        offset + constants::NUM_WINDOWS - 1,
        region,
        config.clone(),
    )?;

    // Assign u = (y_p + z_w).sqrt() for the most significant window
    {
        let u_val =
            k_usize[constants::NUM_WINDOWS - 1].map(|k| base.u.0[constants::NUM_WINDOWS - 1].0[k]);
        region.assign_advice(
            || "u",
            mul_fixed_u,
            offset + constants::NUM_WINDOWS - 1,
            || u_val.ok_or(Error::SynthesisError),
        )?;
    }

    // Add to the cumulative sum and return the final result as `[scalar]B`.
    add_incomplete::assign_region(
        &mul_b,
        &sum,
        offset + constants::NUM_WINDOWS - 1,
        region,
        config,
    )
}
