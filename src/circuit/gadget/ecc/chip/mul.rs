use super::{add, double, util, CellValue, EccConfig, EccPoint};
use crate::constants::NUM_COMPLETE_BITS;
use std::ops::Deref;

use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Permutation, Selector},
    poly::Rotation,
};

mod incomplete;
use incomplete::IncompleteConfig;

pub struct Config {
    // Selector used to constrain the cells used in complete addition.
    q_mul_complete: Selector,
    // Fixed column used to check recovery of the original scalar after decomposition.
    mul_decompose: Column<Fixed>,
    // Advice column used to decompose scalar in complete addition.
    z_complete: Column<Advice>,
    // Permutation
    perm: Permutation,
    // Configuration used in complete addition
    add_config: add::Config,
    // Configuration used in point doubling
    double_config: double::Config,
    // Configuration used for `hi` bits of the scalar
    hi_config: IncompleteConfig,
    // Configuration used for `lo` bits of the scalar
    lo_config: IncompleteConfig,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_mul_complete: ecc_config.q_mul_complete,
            mul_decompose: ecc_config.mul_decompose,
            z_complete: ecc_config.bits,
            perm: ecc_config.perm.clone(),
            add_config: ecc_config.into(),
            double_config: ecc_config.into(),
            hi_config: IncompleteConfig::into_hi_config(ecc_config),
            lo_config: IncompleteConfig::into_lo_config(ecc_config),
        }
    }
}

impl Config {
    pub(super) fn create_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        self.hi_config.create_gate(meta);
        self.lo_config.create_gate(meta);
        self.create_decompose_gate(meta);
        self.create_final_scalar_gate(meta);
    }

    pub(super) fn assign_region<C: CurveAffine>(
        &self,
        scalar: &CellValue<C::Base>,
        base: &EccPoint<C::Base>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C::Base>, Error> {
        // Decompose the scalar bitwise (big-endian bit order).
        let k_bits = decompose_scalar::<C>(scalar.value.unwrap());

        // Bits used in incomplete addition. k_{254} to k_{4} inclusive
        let incomplete_range = 0..(C::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS);
        let k_incomplete = &k_bits[incomplete_range];
        let k_incomplete_hi = &k_incomplete[..k_incomplete.len() / 2];
        let k_incomplete_lo = &k_incomplete[k_incomplete.len() / 2..];

        // Bits used in complete addition. k_{3} to k_{1} inclusive
        // The LSB k_{0} is handled separately.
        let complete_range = (C::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS)
            ..(C::Scalar::NUM_BITS as usize - 1);
        let k_complete = &k_bits[complete_range.clone()];

        // Initialize the accumulator a [2]base
        let acc = self.double_config.assign_region(&base, offset, region)?;

        // Initialize the running sum for scalar decomposition to zero
        let z_val = C::Base::zero();
        let z_cell =
            region.assign_advice(|| "initial z", self.hi_config.z, offset + 1, || Ok(z_val))?;
        let z = CellValue::new(z_cell, Some(z_val));

        // Double-and-add (incomplete addition) for the `hi` half of the scalar decomposition
        let (x, y_a, z) = self.hi_config.double_and_add(
            region,
            &base,
            offset + 1,
            k_incomplete_hi,
            (X(acc.x.clone()), Y(acc.y.value), ZValue(z)),
        )?;

        // Double-and-add (incomplete addition) for the `lo` half of the scalar decomposition
        let (x, y_a, z) = self.lo_config.double_and_add(
            region,
            &base,
            offset + 1,
            k_incomplete_lo,
            (x, y_a, z),
        )?;

        // Move from incomplete addition to complete addition
        let mut acc = {
            let y_a_col = self.add_config.y_qr;
            let row = k_incomplete_lo.len() + 2;

            let y_a_cell = region.assign_advice(
                || "y_a",
                y_a_col,
                row + offset,
                || y_a.ok_or(Error::SynthesisError),
            )?;
            util::assign_and_constrain(
                region,
                || "Copy z from incomplete to complete",
                self.z_complete.into(),
                row + offset,
                &z,
                &self.perm,
            )?;
            EccPoint {
                x: x.0,
                y: CellValue::<C::Base>::new(y_a_cell, *y_a),
            }
        };

        let mut z_val = z.value;
        // Complete addition
        for (iter, k) in k_complete.iter().enumerate() {
            // Each iteration uses 4 rows (two complete additions)
            let row = k_incomplete_lo.len() + 4 * iter + 3;

            // Check scalar decomposition here
            region.assign_advice(
                || "z",
                self.z_complete,
                row + offset - 1,
                || z_val.ok_or(Error::SynthesisError),
            )?;
            z_val = z_val.map(|z_val| C::Base::from_u64(2) * z_val + C::Base::from_u64(*k as u64));
            region.assign_advice(
                || "z",
                self.z_complete,
                row + offset,
                || z_val.ok_or(Error::SynthesisError),
            )?;
            self.q_mul_complete.enable(region, row + offset)?;

            let x_p = base.x.value;
            let x_p_cell = region.assign_advice(
                || "x_p",
                self.add_config.x_p,
                row + offset,
                || x_p.ok_or(Error::SynthesisError),
            )?;

            // If the bit is set, use `y`; if the bit is not set, use `-y`
            let y_p = base.y.value;
            let y_p = y_p.map(|y_p| if !k { -y_p } else { y_p });

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
                .assign_region::<C>(&p, &acc, row + offset, region)?;

            // Copy acc from `x_a`, `y_a` over to `x_p`, `y_p` on the next row
            let acc_x = util::assign_and_constrain(
                region,
                || "copy acc x_a",
                self.add_config.x_p.into(),
                row + offset + 2,
                &acc.x,
                &self.perm,
            )?;
            let acc_y = util::assign_and_constrain(
                region,
                || "copy acc y_a",
                self.add_config.y_p.into(),
                row + offset + 2,
                &acc.y,
                &self.perm,
            )?;

            acc = EccPoint { x: acc_x, y: acc_y };

            // Acc + P + Acc
            acc = self
                .add_config
                .assign_region::<C>(&acc, &tmp_acc, row + offset + 2, region)?;
        }

        // Process the least significant bit
        let k_0_row = k_incomplete_lo.len() + complete_range.len() * 4 + 4;
        let k_0 = &k_bits[C::Scalar::NUM_BITS as usize - 1];

        // Check that we recover the original scalar.
        //
        // NB: We assume that the scalar fits in the curve's base field. This is not
        // true in general, and in particular for the Pallas curve, whose scalar field
        // `Fq` is larger than its base field `Fp`.
        //
        // However, the only use of variable-base scalar mul in the Orchard protocol
        // is in deriving diversified addresses `[ivk] g_d`,  and `ivk` is guaranteed
        // to be in the base field of the curve. (See non-normative notes in
        // https://zips.z.cash/protocol/nu5.pdf#orchardkeycomponents.)

        z_val = z_val.map(|z_val| C::Base::from_u64(2) * z_val + C::Base::from_u64(*k_0 as u64));
        region.assign_advice(
            || "final z",
            self.z_complete,
            k_0_row + offset,
            || z_val.ok_or(Error::SynthesisError),
        )?;
        region.assign_fixed(
            || "original k",
            self.mul_decompose,
            k_0_row + offset,
            || Ok(C::Base::from_bytes(&scalar.value.unwrap().to_bytes()).unwrap()),
        )?;

        // If `k_0` is 0, return `Acc - P`
        if !k_0 {
            let (x_p, y_p) = (base.x.value, base.y.value.map(|y_p| -y_p));
            let x_p_cell = region.assign_advice(
                || "x_p",
                self.add_config.x_p,
                k_0_row + offset,
                || x_p.ok_or(Error::SynthesisError),
            )?;

            let y_p_cell = region.assign_advice(
                || "y_p",
                self.add_config.y_p,
                k_0_row + offset,
                || y_p.ok_or(Error::SynthesisError),
            )?;
            let p = EccPoint {
                x: CellValue::<C::Base>::new(x_p_cell, x_p),
                y: CellValue::<C::Base>::new(y_p_cell, y_p),
            };

            // Return the result of the final complete addition as `[scalar]B`
            self.add_config
                .assign_region::<C>(&p, &acc, k_0_row + offset, region)
        } else {
            // If `k_0` is 1, simply return `Acc`
            Ok(acc)
        }
    }

    /// Gate used to check scalar decomposition is correct.
    /// This is used to check the bits used in complete addition, since the incomplete
    /// addition gate (controlled by `q_mul`) already checks scalar decomposition for
    /// the other bits.
    fn create_decompose_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        let q_mul_complete = meta.query_selector(self.q_mul_complete, Rotation::cur());
        let z_cur = meta.query_advice(self.z_complete, Rotation::cur());
        let z_prev = meta.query_advice(self.z_complete, Rotation::prev());

        meta.create_gate("Decompose scalar ", |_| {
            // k_{i} = z_{i} - 2⋅z_{i+1}
            let k = z_cur.clone() - Expression::Constant(F::from_u64(2)) * z_prev;
            // (k_i) ⋅ (k_i - 1) = 0
            let bool_check = k.clone() * (k + Expression::Constant(-F::one()));

            q_mul_complete.clone() * bool_check
        });
    }

    /// Gate used to check final scalar is recovered.
    pub(super) fn create_final_scalar_gate<F: FieldExt>(&self, meta: &mut ConstraintSystem<F>) {
        let scalar = meta.query_fixed(self.mul_decompose, Rotation::cur());
        let z_cur = meta.query_advice(self.z_complete, Rotation::cur());

        meta.create_gate("Decompose scalar", |_| {
            // q = 2^254 + t_q is the scalar field of Pallas
            let t_q = F::from_u128(45560315531506369815346746415080538113);

            // Check that `k = scalar + t_q`
            scalar.clone() * (scalar + Expression::Constant(t_q) - z_cur)
        });
    }
}

#[derive(Clone, Debug)]
struct X<F: FieldExt>(CellValue<F>);
impl<F: FieldExt> Deref for X<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
struct Y<F: FieldExt>(Option<F>);
impl<F: FieldExt> Deref for Y<F> {
    type Target = Option<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
struct ZValue<F: FieldExt>(CellValue<F>);
impl<F: FieldExt> Deref for ZValue<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn decompose_scalar<C: CurveAffine>(scalar: C::Base) -> Vec<bool> {
    // Cast into scalar field
    let scalar = C::Scalar::from_bytes(&scalar.to_bytes()).unwrap();

    // The scalar field `F_q = 2^254 + t_q`
    let t_q = -(C::Scalar::from_u128(1u128 << 127).square());

    // We will witness `k = scalar + t_q`
    // `k` is decomposed bitwise in-circuit for our double-and-add algorithm.
    let k = scalar + t_q;

    // `k` is decomposed bitwise (big-endian) into `[k_n, ..., k_0]`, where
    // each `k_i` is a bit and `scalar = k_n * 2^n + ... + k_1 * 2 + k_0`.
    let mut bits: Vec<bool> = k
        .to_le_bits()
        .into_iter()
        .take(C::Scalar::NUM_BITS as usize)
        .collect();
    bits.reverse();
    assert_eq!(bits.len(), C::Scalar::NUM_BITS as usize);

    bits
}
