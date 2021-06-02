use super::{CellValue, EccConfig};
use crate::constants::{self, util};
use arrayvec::ArrayVec;
use ff::Field;
use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

pub mod full_width;
pub mod short;

pub struct Config<C: CurveAffine> {
    q_scalar_fixed: Selector,
    // Decomposition of scalar into `k`-bit windows.
    window: Column<Advice>,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> From<&EccConfig> for Config<C> {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_scalar_fixed: ecc_config.q_scalar_fixed,
            window: ecc_config.advices[0],
            _marker: PhantomData,
        }
    }
}

impl<C: CurveAffine> Config<C> {
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        // Range check gate applies to both full-width and short scalars.
        // Check that `k` is within the allowed window size
        meta.create_gate("witness scalar fixed", |meta| {
            let q_scalar_fixed = meta.query_selector(self.q_scalar_fixed, Rotation::cur());
            let window = meta.query_advice(self.window, Rotation::cur());

            let range_check = (0..constants::H)
                .fold(Expression::Constant(C::Base::one()), |acc, i| {
                    acc * (window.clone() - Expression::Constant(C::Base::from_u64(i as u64)))
                });
            vec![q_scalar_fixed * range_check]
        });
    }

    fn decompose_scalar_fixed<const NUM_WINDOWS: usize, const SCALAR_NUM_BITS: usize>(
        &self,
        scalar: Option<C::Scalar>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<ArrayVec<CellValue<C::Base>, NUM_WINDOWS>, Error> {
        // Enable `q_scalar_fixed` selector
        for idx in 0..NUM_WINDOWS {
            self.q_scalar_fixed.enable(region, offset + idx)?;
        }

        // Decompose scalar into `k-bit` windows
        let scalar_windows: Option<Vec<u8>> = scalar.map(|scalar| {
            util::decompose_scalar_fixed::<C>(
                scalar,
                SCALAR_NUM_BITS,
                constants::FIXED_BASE_WINDOW_SIZE,
            )
        });

        // Store the scalar decomposition
        let mut windows: ArrayVec<CellValue<C::Base>, NUM_WINDOWS> = ArrayVec::new();

        let scalar_windows: Vec<Option<C::Base>> = if let Some(windows) = scalar_windows {
            assert_eq!(windows.len(), NUM_WINDOWS);
            windows
                .into_iter()
                .map(|window| Some(C::Base::from_u64(window as u64)))
                .collect()
        } else {
            vec![None; NUM_WINDOWS]
        };

        for (idx, window) in scalar_windows.into_iter().enumerate() {
            let window_cell = region.assign_advice(
                || format!("k[{:?}]", offset + idx),
                self.window,
                offset + idx,
                || window.ok_or(Error::SynthesisError),
            )?;
            windows.push(CellValue::new(window_cell, window));
        }

        Ok(windows)
    }
}
