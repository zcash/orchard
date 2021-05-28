use std::convert::TryInto;

use crate::constants::{self, FixedBase, H, NUM_WINDOWS, NUM_WINDOWS_SHORT};
use halo2::arithmetic::{CurveAffine, FieldExt};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrchardFixedBases<C: CurveAffine> {
    CommitIvkR(constants::CommitIvkR<C>),
    NoteCommitR(constants::NoteCommitR<C>),
    NullifierK(constants::NullifierK<C>),
    ValueCommitR(constants::ValueCommitR<C>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct OrchardFixedBasesShort<C: CurveAffine>(pub constants::ValueCommitV<C>);

/// A fixed base to be used in scalar multiplication with a full-width scalar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchardFixedBase<C: CurveAffine> {
    pub base: OrchardFixedBases<C>,
    pub lagrange_coeffs: LagrangeCoeffs<C::Base>,
    pub z: Z<C::Base>,
    pub u: U<C::Base>,
}

/// A fixed base to be used in scalar multiplication with a short signed exponent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchardFixedBaseShort<C: CurveAffine> {
    pub base: OrchardFixedBasesShort<C>,
    pub lagrange_coeffs_short: LagrangeCoeffsShort<C::Base>,
    pub z_short: ZShort<C::Base>,
    pub u_short: UShort<C::Base>,
}

pub(super) fn commit_ivk_r<C: CurveAffine>() -> OrchardFixedBase<C> {
    let commit_ivk_r = constants::commit_ivk_r::generator();
    OrchardFixedBase {
        base: OrchardFixedBases::CommitIvkR(commit_ivk_r),
        lagrange_coeffs: commit_ivk_r.0.compute_lagrange_coeffs(NUM_WINDOWS).into(),
        z: constants::commit_ivk_r::Z.into(),
        u: constants::commit_ivk_r::U.into(),
    }
}

pub(super) fn note_commit_r<C: CurveAffine>() -> OrchardFixedBase<C> {
    let note_commit_r = constants::note_commit_r::generator();
    OrchardFixedBase {
        base: OrchardFixedBases::NoteCommitR(note_commit_r),
        lagrange_coeffs: note_commit_r.0.compute_lagrange_coeffs(NUM_WINDOWS).into(),
        z: constants::note_commit_r::Z.into(),
        u: constants::note_commit_r::U.into(),
    }
}

pub(super) fn nullifier_k<C: CurveAffine>() -> OrchardFixedBase<C> {
    let nullifier_k = constants::nullifier_k::generator();
    OrchardFixedBase {
        base: OrchardFixedBases::NullifierK(nullifier_k),
        lagrange_coeffs: nullifier_k.0.compute_lagrange_coeffs(NUM_WINDOWS).into(),
        z: constants::nullifier_k::Z.into(),
        u: constants::nullifier_k::U.into(),
    }
}

pub(super) fn value_commit_r<C: CurveAffine>() -> OrchardFixedBase<C> {
    let value_commit_r = constants::value_commit_r::generator();
    OrchardFixedBase {
        base: OrchardFixedBases::ValueCommitR(value_commit_r),
        lagrange_coeffs: value_commit_r.0.compute_lagrange_coeffs(NUM_WINDOWS).into(),
        z: constants::value_commit_r::Z.into(),
        u: constants::value_commit_r::U.into(),
    }
}

pub(super) fn value_commit_v<C: CurveAffine>() -> OrchardFixedBaseShort<C> {
    let value_commit_v = constants::value_commit_v::generator();
    OrchardFixedBaseShort {
        base: OrchardFixedBasesShort(value_commit_v),
        lagrange_coeffs_short: value_commit_v
            .0
            .compute_lagrange_coeffs(NUM_WINDOWS_SHORT)
            .into(),
        z_short: constants::value_commit_v::Z_SHORT.into(),
        u_short: constants::value_commit_v::U_SHORT.into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 8 coefficients per window
pub struct WindowLagrangeCoeffs<F: FieldExt>(pub Box<[F; H]>);

impl<F: FieldExt> From<&[F; H]> for WindowLagrangeCoeffs<F> {
    fn from(array: &[F; H]) -> Self {
        Self(Box::new(*array))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 85 windows per base (with the exception of ValueCommitV)
pub struct LagrangeCoeffs<F: FieldExt>(pub Box<[WindowLagrangeCoeffs<F>; constants::NUM_WINDOWS]>);

impl<F: FieldExt> From<Vec<WindowLagrangeCoeffs<F>>> for LagrangeCoeffs<F> {
    fn from(windows: Vec<WindowLagrangeCoeffs<F>>) -> Self {
        Self(windows.into_boxed_slice().try_into().unwrap())
    }
}

impl<F: FieldExt> From<Vec<[F; H]>> for LagrangeCoeffs<F> {
    fn from(arrays: Vec<[F; H]>) -> Self {
        let windows: Vec<WindowLagrangeCoeffs<F>> =
            arrays.iter().map(|array| array.into()).collect();
        windows.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 22 windows for ValueCommitV
pub struct LagrangeCoeffsShort<F: FieldExt>(pub Box<[WindowLagrangeCoeffs<F>; NUM_WINDOWS_SHORT]>);

impl<F: FieldExt> From<Vec<WindowLagrangeCoeffs<F>>> for LagrangeCoeffsShort<F> {
    fn from(windows: Vec<WindowLagrangeCoeffs<F>>) -> Self {
        Self(windows.into_boxed_slice().try_into().unwrap())
    }
}

impl<F: FieldExt> From<Vec<[F; H]>> for LagrangeCoeffsShort<F> {
    fn from(arrays: Vec<[F; H]>) -> Self {
        let windows: Vec<WindowLagrangeCoeffs<F>> =
            arrays.iter().map(|array| array.into()).collect();
        windows.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 85 Z's per base (with the exception of ValueCommitV)
pub struct Z<F: FieldExt>(pub Box<[F; NUM_WINDOWS]>);

impl<F: FieldExt> From<[u64; NUM_WINDOWS]> for Z<F> {
    fn from(zs: [u64; NUM_WINDOWS]) -> Self {
        Self(
            zs.iter()
                .map(|z| F::from_u64(*z))
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 22 Z's for ValueCommitV
pub struct ZShort<F: FieldExt>(pub Box<[F; NUM_WINDOWS_SHORT]>);

impl<F: FieldExt> From<[u64; NUM_WINDOWS_SHORT]> for ZShort<F> {
    fn from(zs: [u64; NUM_WINDOWS_SHORT]) -> Self {
        Self(
            zs.iter()
                .map(|z| F::from_u64(*z))
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 8 u's per window
pub struct WindowUs<F: FieldExt>(pub Box<[F; H]>);

impl<F: FieldExt> From<&[[u8; 32]; H]> for WindowUs<F> {
    fn from(window_us: &[[u8; 32]; H]) -> Self {
        Self(
            window_us
                .iter()
                .map(|u| F::from_bytes(&u).unwrap())
                .collect::<Vec<_>>()
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 85 windows per base (with the exception of ValueCommitV)
pub struct U<F: FieldExt>(pub Box<[WindowUs<F>; NUM_WINDOWS]>);

impl<F: FieldExt> From<Vec<WindowUs<F>>> for U<F> {
    fn from(windows: Vec<WindowUs<F>>) -> Self {
        Self(windows.into_boxed_slice().try_into().unwrap())
    }
}

impl<F: FieldExt> From<[[[u8; 32]; H]; NUM_WINDOWS]> for U<F> {
    fn from(window_us: [[[u8; 32]; H]; NUM_WINDOWS]) -> Self {
        let windows: Vec<WindowUs<F>> = window_us.iter().map(|us| us.into()).collect();
        windows.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
// 22 windows for ValueCommitV
pub struct UShort<F: FieldExt>(pub Box<[WindowUs<F>; NUM_WINDOWS_SHORT]>);

impl<F: FieldExt> From<Vec<WindowUs<F>>> for UShort<F> {
    fn from(windows: Vec<WindowUs<F>>) -> Self {
        Self(windows.into_boxed_slice().try_into().unwrap())
    }
}

impl<F: FieldExt> From<[[[u8; 32]; H]; NUM_WINDOWS_SHORT]> for UShort<F> {
    fn from(window_us: [[[u8; 32]; H]; NUM_WINDOWS_SHORT]) -> Self {
        let windows: Vec<WindowUs<F>> = window_us.iter().map(|us| us.into()).collect();
        windows.into()
    }
}
