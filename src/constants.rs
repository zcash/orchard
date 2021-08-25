//! Constants used in the Orchard protocol.
pub mod fixed_bases;
pub mod sinsemilla;

pub use self::sinsemilla::{OrchardCommitDomains, OrchardHashDomains};
pub use fixed_bases::OrchardFixedBases;

/// $\mathsf{MerkleDepth^{Orchard}}$
pub(crate) const MERKLE_DEPTH_ORCHARD: usize = 32;

/// The Pallas scalar field modulus is $q = 2^{254} + \mathsf{t_q}$.
/// <https://github.com/zcash/pasta>
pub(crate) const T_Q: u128 = 45560315531506369815346746415080538113;

/// The Pallas base field modulus is $p = 2^{254} + \mathsf{t_p}$.
/// <https://github.com/zcash/pasta>
pub(crate) const T_P: u128 = 45560315531419706090280762371685220353;

/// $\ell^\mathsf{Orchard}_\mathsf{base}$
pub(crate) const L_ORCHARD_BASE: usize = 255;

/// $\ell^\mathsf{Orchard}_\mathsf{scalar}$
pub(crate) const L_ORCHARD_SCALAR: usize = 255;

/// $\ell_\mathsf{value}$
pub(crate) const L_VALUE: usize = 64;

/// SWU hash-to-curve personalization for the group hash for key diversification
pub const KEY_DIVERSIFICATION_PERSONALIZATION: &str = "z.cash:Orchard-gd";

#[cfg(test)]
mod tests {
    use ff::PrimeField;
    use pasta_curves::{arithmetic::FieldExt, pallas};

    #[test]
    // Orchard uses the Pallas base field as its base field.
    fn l_orchard_base() {
        assert_eq!(super::L_ORCHARD_BASE, pallas::Base::NUM_BITS as usize);
    }

    #[test]
    // Orchard uses the Pallas base field as its base field.
    fn l_orchard_scalar() {
        assert_eq!(super::L_ORCHARD_SCALAR, pallas::Scalar::NUM_BITS as usize);
    }

    #[test]
    fn t_q() {
        let t_q = pallas::Scalar::from_u128(super::T_Q);
        let two_pow_254 = pallas::Scalar::from_u128(1 << 127).square();
        assert_eq!(t_q + two_pow_254, pallas::Scalar::zero());
    }

    #[test]
    fn t_p() {
        let t_p = pallas::Base::from_u128(super::T_P);
        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        assert_eq!(t_p + two_pow_254, pallas::Base::zero());
    }
}

#[cfg(test)]
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

#[cfg(test)]
/// Test that Lagrange interpolation coefficients reproduce the correct x-coordinate
/// for each fixed-base multiple in each window.
fn test_lagrange_coeffs<C: CurveAffine>(base: C, num_windows: usize) {
    use ecc::{chip::compute_lagrange_coeffs, gadget::FIXED_BASE_WINDOW_SIZE};
    use ff::Field;
    use group::Curve;

    fn evaluate<C: CurveAffine>(x: u8, coeffs: &[C::Base]) -> C::Base {
        let x = C::Base::from_u64(x as u64);
        coeffs
            .iter()
            .rev()
            .cloned()
            .reduce(|acc, coeff| acc * x + coeff)
            .unwrap_or_else(C::Base::zero)
    }

    let lagrange_coeffs = compute_lagrange_coeffs(base, num_windows);

    // Check first 84 windows, i.e. `k_0, k_1, ..., k_83`
    for (idx, coeffs) in lagrange_coeffs[0..(num_windows - 1)].iter().enumerate() {
        // Test each three-bit chunk in this window.
        for bits in 0..(1 << FIXED_BASE_WINDOW_SIZE) {
            {
                // Interpolate the x-coordinate using this window's coefficients
                let interpolated_x = evaluate::<C>(bits, coeffs);

                // Compute the actual x-coordinate of the multiple [(k+2)*(8^w)]B.
                let point = base
                    * C::Scalar::from_u64(bits as u64 + 2)
                    * C::Scalar::from_u64(fixed_bases::H as u64).pow(&[idx as u64, 0, 0, 0]);
                let x = *point.to_affine().coordinates().unwrap().x();

                // Check that the interpolated x-coordinate matches the actual one.
                assert_eq!(x, interpolated_x);
            }
        }
    }

    // Check last window.
    for bits in 0..(1 << FIXED_BASE_WINDOW_SIZE) {
        // Interpolate the x-coordinate using the last window's coefficients
        let interpolated_x = evaluate::<C>(bits, &lagrange_coeffs[num_windows - 1]);

        // Compute the actual x-coordinate of the multiple [k * (8^84) - offset]B,
        // where offset = \sum_{j = 0}^{83} 2^{3j+1}
        let offset = (0..(num_windows - 1)).fold(C::Scalar::zero(), |acc, w| {
            acc + C::Scalar::from_u64(2).pow(&[
                FIXED_BASE_WINDOW_SIZE as u64 * w as u64 + 1,
                0,
                0,
                0,
            ])
        });
        let scalar = C::Scalar::from_u64(bits as u64)
            * C::Scalar::from_u64(fixed_bases::H as u64).pow(&[(num_windows - 1) as u64, 0, 0, 0])
            - offset;
        let point = base * scalar;
        let x = *point.to_affine().coordinates().unwrap().x();

        // Check that the interpolated x-coordinate matches the actual one.
        assert_eq!(x, interpolated_x);
    }
}

#[cfg(test)]
// Test that the z-values and u-values satisfy the conditions:
//      1. z + y = u^2,
//      2. z - y is not a square
// for the y-coordinate of each fixed-base multiple in each window.
fn test_zs_and_us<C: CurveAffine>(
    base: C,
    z: &[u64],
    u: &[[[u8; 32]; ecc::gadget::H]],
    num_windows: usize,
) {
    use ecc::chip::compute_window_table;
    use ff::Field;

    let window_table = compute_window_table(base, num_windows);

    for ((u, z), window_points) in u.iter().zip(z.iter()).zip(window_table) {
        for (u, point) in u.iter().zip(window_points.iter()) {
            let y = *point.coordinates().unwrap().y();
            let u = C::Base::from_bytes(u).unwrap();
            assert_eq!(C::Base::from_u64(*z) + y, u * u); // allow either square root
            assert!(bool::from((C::Base::from_u64(*z) - y).sqrt().is_none()));
        }
    }
}
