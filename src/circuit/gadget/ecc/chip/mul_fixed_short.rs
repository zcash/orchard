use super::{
    add_incomplete, util, witness_point, EccConfig, EccPoint, EccScalarFixedShort,
    OrchardFixedBaseShort,
};
use crate::constants;

use group::Curve;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{ConstraintSystem, Error, Expression},
};

// We reuse the constraints in the `mul_fixed` gate so exclude them here.
// Here, we add some new constraints specific to the short signed case.
pub(super) fn create_gate<F: FieldExt>(
    meta: &mut ConstraintSystem<F>,
    q_mul_fixed_short: Expression<F>,
    s: Expression<F>,
    y_a: Expression<F>,
    y_p: Expression<F>,
) {
    // `(x_a, y_a)` is the result of `[m]B`, where `m` is the magnitude.
    // We conditionally negate this result using `y_p = y_a * s`, where `s` is the sign.

    // Check that the final `y_p = y_a` or `y_p = -y_a`
    meta.create_gate("check y", |_| {
        q_mul_fixed_short.clone() * (y_p.clone() - y_a.clone()) * (y_p.clone() + y_a.clone())
    });

    // Check that s * y_p = y_a
    meta.create_gate("check negation", |_| q_mul_fixed_short * (s * y_p - y_a));
}

#[allow(non_snake_case)]
pub(super) fn assign_region<C: CurveAffine>(
    scalar: &EccScalarFixedShort<C>,
    base: &OrchardFixedBaseShort<C>,
    offset: usize,
    region: &mut Region<'_, C::Base>,
    config: EccConfig,
) -> Result<EccPoint<C::Base>, Error> {
    // Rename columns for `mul_fixed` context
    let A = (config.extras[0], config.extras[1]);
    let mul_fixed_u = config.extras[2];

    // Assign fixed columns for given fixed base
    for w in 0..constants::NUM_WINDOWS_SHORT {
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
                || Ok(base.lagrange_coeffs_short.0[w].0[k]),
            )?;
        }

        // Assign z-values for each window
        region.assign_fixed(
            || format!("z-value for window: {:?}", w),
            config.fixed_z.into(),
            w + offset,
            || Ok(base.z_short.0[w]),
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

    // Get the value of the fixed base. We only use `ValueCommitV` here.
    let b = base.base.0 .0.value();

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
        let u_val = k_usize[0].map(|k_0| base.u_short.0[0].0[k_0]);
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
        let u_val = k_usize[w].map(|k| base.u_short.0[w].0[k]);
        region.assign_advice(
            || "u",
            mul_fixed_u,
            w,
            || u_val.ok_or(Error::SynthesisError),
        )?;

        // Add to the cumulative sum
        sum = add_incomplete::assign_region(&mul_b, &sum, offset + w, region, config.clone())
            .unwrap();
    }

    // Process most significant window outside the for loop
    let offset_sum = (0..(constants::NUM_WINDOWS_SHORT - 1))
        .fold(C::ScalarExt::zero(), |acc, w| {
            acc + h.pow(&[w as u64, 0, 0, 0])
        });

    // `scalar = [k * 8^21 - offset_sum]`, where `offset_sum = \sum_{j = 0}^{20} 8^j`.
    let last_scalar = k[k.len() - 1]
        .map(|k| k * h.pow(&[(constants::NUM_WINDOWS_SHORT - 1) as u64, 0, 0, 0]) - offset_sum);
    let mul_b = last_scalar.map(|last_scalar| b * last_scalar);
    let mul_b = witness_point::assign_region(
        mul_b.map(|point| point.to_affine()),
        offset + constants::NUM_WINDOWS_SHORT - 1,
        region,
        config.clone(),
    )?;

    // Assign u = (y_p + z_w).sqrt() for the most significant window
    {
        let u_val = k_usize[constants::NUM_WINDOWS_SHORT - 1]
            .map(|k| base.u_short.0[constants::NUM_WINDOWS_SHORT - 1].0[k]);
        region.assign_advice(
            || "u",
            mul_fixed_u,
            offset + constants::NUM_WINDOWS_SHORT - 1,
            || u_val.ok_or(Error::SynthesisError),
        )?;
    }

    // Add to the cumulative sum to get `[magnitude]B`.
    let magnitude_mul = add_incomplete::assign_region(
        &mul_b,
        &sum,
        offset + constants::NUM_WINDOWS_SHORT - 1,
        region,
        config.clone(),
    )?;

    // Assign sign to `bits` column
    let sign = util::assign_and_constrain(
        region,
        || "sign",
        config.bits.into(),
        offset + constants::NUM_WINDOWS_SHORT,
        &scalar.sign,
        &config.perm,
    )?;

    // Conditionally negate `y`-coordinate
    let y_val = match sign.value {
        Some(sign) => {
            if sign == -C::Base::one() {
                magnitude_mul.y.value.map(|y: C::Base| -y)
            } else {
                magnitude_mul.y.value
            }
        }
        None => None,
    };

    // Enable mul_fixed_short selector on final row
    config
        .q_mul_fixed_short
        .enable(region, offset + constants::NUM_WINDOWS_SHORT)?;

    // Assign final `x, y` to `x_p, y_p` columns and return final point
    let x_val = magnitude_mul.x.value;
    let mul = x_val
        .zip(y_val)
        .map(|(x, y)| C::from_xy(x, y).unwrap().to_curve());
    witness_point::assign_region(
        mul.map(|point| point.to_affine()),
        offset + constants::NUM_WINDOWS_SHORT,
        region,
        config,
    )
}
