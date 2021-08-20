//! Utilities used in the constants module.

use ff::Field;
use halo2::arithmetic::{CurveAffine, FieldExt};

/// Evaluate y = f(x) given the coefficients of f(x)
pub fn evaluate<C: CurveAffine>(x: u8, coeffs: &[C::Base]) -> C::Base {
    let x = C::Base::from_u64(x as u64);
    coeffs
        .iter()
        .rev()
        .cloned()
        .reduce(|acc, coeff| acc * x + coeff)
        .unwrap_or_else(C::Base::zero)
}

/// Takes in an FnMut closure and returns a constant-length array with elements of
/// type `Output`.
pub fn gen_const_array<Output: Copy + Default, const LEN: usize>(
    mut closure: impl FnMut(usize) -> Output,
) -> [Output; LEN] {
    let mut ret: [Output; LEN] = [Default::default(); LEN];
    for (bit, val) in ret.iter_mut().zip((0..LEN).map(|idx| closure(idx))) {
        *bit = val;
    }
    ret
}
