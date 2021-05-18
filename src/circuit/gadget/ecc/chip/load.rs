use std::convert::TryInto;

use crate::constants::{self, FixedBase, H, NUM_WINDOWS, NUM_WINDOWS_SHORT};
use halo2::arithmetic::{CurveAffine, FieldExt};

#[derive(Copy, Clone, Debug)]
pub enum OrchardFixedBases<C: CurveAffine> {
    CommitIvkR(constants::CommitIvkR<C>),
    NoteCommitR(constants::NoteCommitR<C>),
    NullifierK(constants::NullifierK<C>),
    ValueCommitR(constants::ValueCommitR<C>),
}

#[derive(Copy, Clone, Debug)]
pub struct OrchardFixedBasesShort<C: CurveAffine>(pub constants::ValueCommitV<C>);

/// A fixed base to be used in scalar multiplication with a full-width scalar.
#[derive(Clone, Debug)]
pub struct OrchardFixedBase<C: CurveAffine> {
    pub base: OrchardFixedBases<C>,
    pub lagrange_coeffs: LagrangeCoeffs<C::Base>,
    pub z: Z<C::Base>,
    pub u: U<C::Base>,
}

/// A fixed base to be used in scalar multiplication with a short signed exponent.
#[derive(Clone, Debug)]
pub struct OrchardFixedBaseShort<C: CurveAffine> {
    pub base: OrchardFixedBasesShort<C>,
    pub lagrange_coeffs_short: LagrangeCoeffsShort<C::Base>,
    pub z_short: ZShort<C::Base>,
    pub u_short: UShort<C::Base>,
}

#[derive(Clone, Debug)]
// 8 coefficients per window
pub struct WindowLagrangeCoeffs<F: FieldExt>(pub Box<[F; H]>);

#[derive(Clone, Debug)]
// 85 windows per base (with the exception of ValueCommitV)
pub struct LagrangeCoeffs<F: FieldExt>(pub Box<[WindowLagrangeCoeffs<F>; constants::NUM_WINDOWS]>);

#[derive(Clone, Debug)]
// 22 windows for ValueCommitV
pub struct LagrangeCoeffsShort<F: FieldExt>(pub Box<[WindowLagrangeCoeffs<F>; NUM_WINDOWS_SHORT]>);

#[derive(Clone, Debug)]
// 85 Z's per base (with the exception of ValueCommitV)
pub struct Z<F: FieldExt>(pub Box<[F; NUM_WINDOWS]>);

#[derive(Clone, Debug)]
// 22 Z's for ValueCommitV
pub struct ZShort<F: FieldExt>(pub Box<[F; NUM_WINDOWS_SHORT]>);

#[derive(Clone, Debug)]
// 8 u's per window
pub struct WindowUs<F: FieldExt>(pub Box<[F; H]>);

#[derive(Clone, Debug)]
// 85 windows per base (with the exception of ValueCommitV)
pub struct U<F: FieldExt>(pub Box<[WindowUs<F>; NUM_WINDOWS]>);

#[derive(Clone, Debug)]
// 22 windows for ValueCommitV
pub struct UShort<F: FieldExt>(pub Box<[WindowUs<F>; NUM_WINDOWS_SHORT]>);

pub(super) fn commit_ivk_r<C: CurveAffine>() -> OrchardFixedBase<C> {
    let commit_ivk_r = constants::commit_ivk_r::generator();
    OrchardFixedBase {
        base: OrchardFixedBases::CommitIvkR(commit_ivk_r),
        lagrange_coeffs: load_lagrange_coeffs(commit_ivk_r.0.compute_lagrange_coeffs(NUM_WINDOWS)),
        z: load_z(&constants::commit_ivk_r::Z),
        u: process_u(&constants::commit_ivk_r::U),
    }
}

pub(super) fn note_commit_r<C: CurveAffine>() -> OrchardFixedBase<C> {
    let note_commit_r = constants::note_commit_r::generator();
    OrchardFixedBase {
        base: OrchardFixedBases::NoteCommitR(note_commit_r),
        lagrange_coeffs: load_lagrange_coeffs(note_commit_r.0.compute_lagrange_coeffs(NUM_WINDOWS)),
        z: load_z(&constants::note_commit_r::Z),
        u: process_u(&constants::note_commit_r::U),
    }
}

pub(super) fn nullifier_k<C: CurveAffine>() -> OrchardFixedBase<C> {
    let nullifier_k = constants::nullifier_k::generator();
    OrchardFixedBase {
        base: OrchardFixedBases::NullifierK(nullifier_k),
        lagrange_coeffs: load_lagrange_coeffs(nullifier_k.0.compute_lagrange_coeffs(NUM_WINDOWS)),
        z: load_z(&constants::nullifier_k::Z),
        u: process_u(&constants::nullifier_k::U),
    }
}

pub(super) fn value_commit_r<C: CurveAffine>() -> OrchardFixedBase<C> {
    let value_commit_r = constants::value_commit_r::generator();
    OrchardFixedBase {
        base: OrchardFixedBases::ValueCommitR(value_commit_r),
        lagrange_coeffs: load_lagrange_coeffs(
            value_commit_r.0.compute_lagrange_coeffs(NUM_WINDOWS),
        ),
        z: load_z(&constants::value_commit_r::Z),
        u: process_u(&constants::value_commit_r::U),
    }
}

pub(super) fn value_commit_v<C: CurveAffine>() -> OrchardFixedBaseShort<C> {
    let value_commit_v = constants::value_commit_v::generator();
    OrchardFixedBaseShort {
        base: OrchardFixedBasesShort(value_commit_v),
        lagrange_coeffs_short: load_lagrange_coeffs_short(
            value_commit_v.0.compute_lagrange_coeffs(NUM_WINDOWS_SHORT),
        ),
        z_short: load_z_short(&constants::value_commit_v::Z_SHORT),
        u_short: process_u_short(&constants::value_commit_v::U_SHORT),
    }
}

fn load_lagrange_coeffs<F: FieldExt>(coeffs: Vec<[F; H]>) -> LagrangeCoeffs<F> {
    LagrangeCoeffs(
        coeffs
            .iter()
            .map(|window| {
                WindowLagrangeCoeffs(
                    window
                        .iter()
                        .map(|&coeff| coeff)
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                        .try_into()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .unwrap(),
    )
}

fn load_lagrange_coeffs_short<F: FieldExt>(coeffs: Vec<[F; H]>) -> LagrangeCoeffsShort<F> {
    LagrangeCoeffsShort(
        coeffs
            .iter()
            .map(|window| {
                WindowLagrangeCoeffs(
                    window
                        .iter()
                        .map(|&coeff| coeff)
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                        .try_into()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .unwrap(),
    )
}

fn load_z<F: FieldExt>(zs: &[u64]) -> Z<F> {
    Z(zs.iter()
        .map(|z| F::from_u64(*z))
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .try_into()
        .unwrap())
}

fn load_z_short<F: FieldExt>(zs: &[u64]) -> ZShort<F> {
    ZShort(
        zs.iter()
            .map(|z| F::from_u64(*z))
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .unwrap(),
    )
}

fn process_u<F: FieldExt>(us: &[[[u8; 32]; H]]) -> U<F> {
    U(us.iter()
        .map(|window_us| {
            WindowUs(
                window_us
                    .iter()
                    .map(|u| F::from_bytes(&u).unwrap())
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
                    .try_into()
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .try_into()
        .unwrap())
}

fn process_u_short<F: FieldExt>(us: &[[[u8; 32]; H]]) -> UShort<F> {
    UShort(
        us.iter()
            .map(|window_us| {
                WindowUs(
                    window_us
                        .iter()
                        .map(|u| F::from_bytes(&u).unwrap())
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                        .try_into()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
            .try_into()
            .unwrap(),
    )
}
