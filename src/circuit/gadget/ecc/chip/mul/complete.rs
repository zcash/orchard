use super::super::{add, util, CellValue, EccPoint};
use super::Mul;
use ff::Field;

use halo2::{
    arithmetic::{CurveAffine, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

pub struct Config<C: CurveAffine> {
    // Selector used to constrain the cells used in complete addition.
    q_mul_complete: Selector,
    // Advice column used to decompose scalar in complete addition.
    z_complete: Column<Advice>,
    // Permutation
    perm: Permutation,
    // Configuration used in complete addition
    add_config: add::Config,
    _marker: PhantomData<C>,
}

impl<C: CurveAffine> From<&super::Config<C>> for Config<C> {
    fn from(config: &super::Config<C>) -> Self {
        Self {
            q_mul_complete: config.q_mul_complete,
            z_complete: config.z_complete,
            perm: config.perm.clone(),
            add_config: config.add_config.clone(),
            _marker: PhantomData,
        }
    }
}

impl<C: CurveAffine> Mul<C> for Config<C> {}

impl<C: CurveAffine> Config<C> {
    /// Gate used to check scalar decomposition is correct.
    /// This is used to check the bits used in complete addition, since the incomplete
    /// addition gate (controlled by `q_mul`) already checks scalar decomposition for
    /// the other bits.
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        let q_mul_complete = meta.query_selector(self.q_mul_complete, Rotation::cur());
        let z_cur = meta.query_advice(self.z_complete, Rotation::cur());
        let z_prev = meta.query_advice(self.z_complete, Rotation::prev());

        meta.create_gate("Decompose scalar ", |_| {
            // k_{i} = z_{i} - 2⋅z_{i+1}
            let k = z_cur.clone() - Expression::Constant(C::Base::from_u64(2)) * z_prev;
            // (k_i) ⋅ (k_i - 1) = 0
            let bool_check = k.clone() * (k + Expression::Constant(-C::Base::one()));

            q_mul_complete.clone() * bool_check
        });
    }

    #[allow(clippy::type_complexity)]
    pub(super) fn assign_region(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        bits: &[Option<bool>],
        base: &EccPoint<C>,
        mut acc: EccPoint<C>,
        mut z_val: Option<C::Base>,
    ) -> Result<(EccPoint<C>, Option<C::Base>), Error> {
        // Make sure we have the correct number of bits for the complete addition
        // part of variable-base scalar mul.
        assert_eq!(bits.len(), Self::complete_len());

        // Enable selectors for complete range
        for row in Self::complete_range() {
            self.q_mul_complete.enable(region, row + offset)?;
        }

        // Complete addition
        for (iter, k) in bits.into_iter().enumerate() {
            // Each iteration uses 2 rows (two complete additions)
            let row = Self::incomplete_lo_len() + 2 * iter + 3;

            // Check scalar decomposition here
            region.assign_advice(
                || "z",
                self.z_complete,
                row + offset - 1,
                || z_val.ok_or(Error::SynthesisError),
            )?;
            z_val = z_val
                .zip(k.as_ref())
                .map(|(z_val, k)| C::Base::from_u64(2) * z_val + C::Base::from_u64(*k as u64));
            region.assign_advice(
                || "z",
                self.z_complete,
                row + offset,
                || z_val.ok_or(Error::SynthesisError),
            )?;

            let x_p = base.x.value;
            let x_p_cell = region.assign_advice(
                || "x_p",
                self.add_config.x_p,
                row + offset,
                || x_p.ok_or(Error::SynthesisError),
            )?;

            // If the bit is set, use `y`; if the bit is not set, use `-y`
            let y_p = base.y.value;
            let y_p = y_p
                .zip(k.as_ref())
                .map(|(y_p, k)| if !k { -y_p } else { y_p });

            let y_p_cell = region.assign_advice(
                || "y_p",
                self.add_config.y_p,
                row + offset,
                || y_p.ok_or(Error::SynthesisError),
            )?;
            let p = EccPoint {
                x: CellValue::<C::Base>::new(x_p_cell, x_p),
                y: CellValue::<C::Base>::new(y_p_cell, y_p),
            };

            // Acc + U
            let tmp_acc = self
                .add_config
                .assign_region(&p, &acc, row + offset, region)?;

            // Copy acc from `x_a`, `y_a` over to `x_p`, `y_p` on the next row
            let acc_x = util::assign_and_constrain(
                region,
                || "copy acc x_a",
                self.add_config.x_p,
                row + offset + 1,
                &acc.x,
                &self.perm,
            )?;
            let acc_y = util::assign_and_constrain(
                region,
                || "copy acc y_a",
                self.add_config.y_p,
                row + offset + 1,
                &acc.y,
                &self.perm,
            )?;

            acc = EccPoint { x: acc_x, y: acc_y };

            // Acc + P + Acc
            acc = self
                .add_config
                .assign_region(&acc, &tmp_acc, row + offset + 1, region)?;
        }
        Ok((acc, z_val))
    }
}
