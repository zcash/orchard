//! Utilities to compute associated constants for fixed bases.
use super::{FIXED_BASE_WINDOW_SIZE, H};
use arrayvec::ArrayVec;
use ff::Field;
use group::Curve;
use halo2::arithmetic::lagrange_interpolate;
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

/// For each fixed base, we calculate its scalar multiples in three-bit windows.
/// Each window will have $2^3 = 8$ points.
pub fn compute_window_table<C: CurveAffine>(base: C, num_windows: usize) -> Vec<[C; H]> {
    let mut window_table: Vec<[C; H]> = Vec::with_capacity(num_windows);

    // Generate window table entries for all windows but the last.
    // For these first `num_windows - 1` windows, we compute the multiple [(k+2)*(2^3)^w]B.
    // Here, w ranges from [0..`num_windows - 1`)
    for w in 0..(num_windows - 1) {
        window_table.push(
            (0..H)
                .map(|k| {
                    // scalar = (k+2)*(8^w)
                    let scalar = C::ScalarExt::from_u64(k as u64 + 2)
                        * C::ScalarExt::from_u64(H as u64).pow(&[w as u64, 0, 0, 0]);
                    (base * scalar).to_affine()
                })
                .collect::<ArrayVec<C, H>>()
                .into_inner()
                .unwrap(),
        );
    }

    // Generate window table entries for the last window, w = `num_windows - 1`.
    // For the last window, we compute [k * (2^3)^w - sum]B, where sum is defined
    // as sum = \sum_{j = 0}^{`num_windows - 2`} 2^{3j+1}
    let sum = (0..(num_windows - 1)).fold(C::ScalarExt::zero(), |acc, j| {
        acc + C::ScalarExt::from_u64(2).pow(&[
            FIXED_BASE_WINDOW_SIZE as u64 * j as u64 + 1,
            0,
            0,
            0,
        ])
    });
    window_table.push(
        (0..H)
            .map(|k| {
                // scalar = k * (2^3)^w - sum, where w = `num_windows - 1`
                let scalar = C::ScalarExt::from_u64(k as u64)
                    * C::ScalarExt::from_u64(H as u64).pow(&[(num_windows - 1) as u64, 0, 0, 0])
                    - sum;
                (base * scalar).to_affine()
            })
            .collect::<ArrayVec<C, H>>()
            .into_inner()
            .unwrap(),
    );

    window_table
}

/// For each window, we interpolate the $x$-coordinate.
/// Here, we pre-compute and store the coefficients of the interpolation polynomial.
pub fn compute_lagrange_coeffs<C: CurveAffine>(base: C, num_windows: usize) -> Vec<[C::Base; H]> {
    // We are interpolating over the 3-bit window, k \in [0..8)
    let points: Vec<_> = (0..H).map(|i| C::Base::from_u64(i as u64)).collect();

    let window_table = compute_window_table(base, num_windows);

    window_table
        .iter()
        .map(|window_points| {
            let x_window_points: Vec<_> = window_points
                .iter()
                .map(|point| *point.coordinates().unwrap().x())
                .collect();
            lagrange_interpolate(&points, &x_window_points)
                .into_iter()
                .collect::<ArrayVec<C::Base, H>>()
                .into_inner()
                .unwrap()
        })
        .collect()
}

/// For each window, $z$ is a field element such that for each point $(x, y)$ in the window:
/// - $z + y = u^2$ (some square in the field); and
/// - $z - y$ is not a square.
/// If successful, return a vector of `(z: u64, us: [C::Base; H])` for each window.
pub fn find_zs_and_us<C: CurveAffine>(
    base: C,
    num_windows: usize,
) -> Option<Vec<(u64, [[u8; 32]; H])>> {
    // Closure to find z and u's for one window
    let find_z_and_us = |window_points: &[C]| {
        assert_eq!(H, window_points.len());

        let ys: Vec<_> = window_points
            .iter()
            .map(|point| *point.coordinates().unwrap().y())
            .collect();
        (0..(1000 * (1 << (2 * H)))).find_map(|z| {
            ys.iter()
                .map(|&y| {
                    let u = if (-y + C::Base::from_u64(z)).sqrt().is_none().into() {
                        (y + C::Base::from_u64(z)).sqrt().into()
                    } else {
                        None
                    };
                    u.map(|u: C::Base| u.to_bytes())
                })
                .collect::<Option<ArrayVec<[u8; 32], H>>>()
                .map(|us| (z, us.into_inner().unwrap()))
        })
    };

    let window_table = compute_window_table(base, num_windows);
    window_table
        .iter()
        .map(|window_points| find_z_and_us(window_points))
        .collect()
}
