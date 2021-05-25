use super::{add, util, CellValue, EccConfig, EccPoint};
use crate::constants::NUM_COMPLETE_BITS;
use std::ops::{Deref, Range};

use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Fixed, Permutation, Selector},
    poly::Rotation,
};

mod complete;
mod incomplete;

pub struct Config<C: CurveAffine> {
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
    // Configuration used for `hi` bits of the scalar
    hi_config: incomplete::Config<C>,
    // Configuration used for `lo` bits of the scalar
    lo_config: incomplete::Config<C>,
}

impl<C: CurveAffine> From<&EccConfig> for Config<C> {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_mul_complete: ecc_config.q_mul_complete,
            mul_decompose: ecc_config.mul_decompose,
            z_complete: ecc_config.bits,
            perm: ecc_config.perm.clone(),
            add_config: ecc_config.into(),
            hi_config: incomplete::Config::hi_config(ecc_config),
            lo_config: incomplete::Config::lo_config(ecc_config),
        }
    }
}

trait Mul<C: CurveAffine> {
    // Bits used in incomplete addition. k_{254} to k_{4} inclusive
    fn incomplete_range(&self) -> Range<usize> {
        0..(C::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS)
    }

    // Bits k_{254} to k_{4} inclusive are used in incomplete addition.
    // The `lo` half is k_{129} to k_{4} inclusive (length 126 bits).
    fn incomplete_lo_len() -> usize {
        (C::Scalar::NUM_BITS as usize - NUM_COMPLETE_BITS) / 2
    }

    // Bits k_{254} to k_{4} inclusive are used in incomplete addition.
    // The `hi` half is k_{254} to k_{130} inclusive (length 125 bits).
    fn incomplete_hi_len() -> usize {
        (C::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS) / 2
    }

    fn complete_range(&self) -> Range<usize> {
        (C::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS)..(C::Scalar::NUM_BITS as usize - 1)
    }

    fn complete_len(&self) -> usize {
        NUM_COMPLETE_BITS as usize
    }
}

impl<C: CurveAffine> Mul<C> for Config<C> {}

impl<C: CurveAffine> Config<C> {
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        self.hi_config.create_gate(meta);
        self.lo_config.create_gate(meta);

        let complete_config: complete::Config<C> = self.into();
        complete_config.create_gate(meta);

        self.create_final_scalar_gate(meta);
    }

    /// Gate used to check final scalar is recovered.
    fn create_final_scalar_gate(&self, meta: &mut ConstraintSystem<C::Base>) {
        let scalar = meta.query_fixed(self.mul_decompose, Rotation::cur());
        let z_cur = meta.query_advice(self.z_complete, Rotation::cur());

        meta.create_gate("Decompose scalar", |_| {
            // q = 2^254 + t_q is the scalar field of Pallas
            let t_q = C::Base::from_u128(45560315531506369815346746415080538113);

            // Check that `k = scalar + t_q`
            scalar.clone() * (scalar + Expression::Constant(t_q) - z_cur)
        });
    }

    pub(super) fn assign_region(
        &self,
        scalar: &CellValue<C::Base>,
        base: &EccPoint<C::Base>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C::Base>, Error> {
        // Initialize the accumulator `acc = [2]base`
        let acc = self
            .add_config
            .assign_region(&base, &base, offset, region)?;

        // Increase the offset by 1 after complete addition.
        let offset = offset + 1;

        // Decompose the scalar bitwise (big-endian bit order).
        let k_bits = scalar.value.map(|scalar| decompose_scalar::<C>(scalar));

        // Bits used in incomplete addition. k_{254} to k_{4} inclusive
        let k_incomplete = k_bits
            .clone()
            .map(|k_bits| k_bits[self.incomplete_range()].to_vec());

        // Initialize the running sum for scalar decomposition to zero
        let z_val = C::Base::zero();
        let z_cell =
            region.assign_advice(|| "initial z", self.hi_config.z, offset + 1, || Ok(z_val))?;
        let z = CellValue::new(z_cell, Some(z_val));

        // Double-and-add (incomplete addition) for the `hi` half of the scalar decomposition
        let k_incomplete_hi = k_incomplete
            .clone()
            .map(|k_incomplete| k_incomplete[..k_incomplete.len() / 2].to_vec());
        let (x, y_a, z) = self.hi_config.double_and_add(
            region,
            offset + 1,
            &base,
            k_incomplete_hi,
            (X(acc.x.clone()), Y(acc.y.value), Z(z)),
        )?;

        // Double-and-add (incomplete addition) for the `lo` half of the scalar decomposition
        let k_incomplete_lo =
            k_incomplete.map(|k_incomplete| k_incomplete[k_incomplete.len() / 2..].to_vec());
        let (x, y_a, z) = self.lo_config.double_and_add(
            region,
            offset + 1,
            &base,
            k_incomplete_lo,
            (x, y_a, z),
        )?;

        // Move from incomplete addition to complete addition
        let acc = {
            let y_a_col = self.add_config.y_qr;
            let row = Self::incomplete_lo_len() + 2;

            let y_a_cell = region.assign_advice(
                || "y_a",
                y_a_col,
                row + offset,
                || y_a.ok_or(Error::SynthesisError),
            )?;
            util::assign_and_constrain(
                region,
                || "Copy z from incomplete to complete",
                self.z_complete,
                row + offset,
                &z,
                &self.perm,
            )?;
            EccPoint {
                x: x.0,
                y: CellValue::<C::Base>::new(y_a_cell, *y_a),
            }
        };

        let complete_config: complete::Config<C> = self.into();
        // Bits used in complete addition. k_{3} to k_{1} inclusive
        // The LSB k_{0} is handled separately.
        let k_complete = k_bits
            .clone()
            .map(|k_bits| k_bits[self.complete_range()].to_vec());
        let (acc, z_val) =
            complete_config.assign_region(region, offset, k_complete, base, acc, z.value)?;

        // Process the least significant bit
        let k_0 = k_bits.map(|k_bits| k_bits[C::Scalar::NUM_BITS as usize - 1].clone());
        self.process_lsb(region, offset, scalar, base, acc, k_0, z_val)
    }

    fn process_lsb(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        scalar: &CellValue<C::Base>,
        base: &EccPoint<C::Base>,
        acc: EccPoint<C::Base>,
        k_0: Option<bool>,
        mut z_val: Option<C::Base>,
    ) -> Result<EccPoint<C::Base>, Error> {
        // Assign the final `z` value.
        let k_0_row = Self::incomplete_lo_len() + self.complete_range().len() * 2 + 4;
        z_val = z_val
            .zip(k_0)
            .map(|(z_val, k_0)| C::Base::from_u64(2) * z_val + C::Base::from_u64(k_0 as u64));
        region.assign_advice(
            || "final z",
            self.z_complete,
            k_0_row + offset,
            || z_val.ok_or(Error::SynthesisError),
        )?;

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
        let scalar = scalar
            .value
            .map(|scalar| C::Base::from_bytes(&scalar.to_bytes()).unwrap());
        region.assign_fixed(
            || "original k",
            self.mul_decompose,
            k_0_row + offset,
            || scalar.ok_or(Error::SynthesisError),
        )?;

        // If `k_0` is 0, return `Acc - P`. If `k_0` is 1, simply return `Acc`.
        let p = k_0
            .map(|k_0| {
                if !k_0 {
                    // If `k_0` is 0, return `Acc - P`
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
                    Ok(EccPoint {
                        x: CellValue::<C::Base>::new(x_p_cell, x_p),
                        y: CellValue::<C::Base>::new(y_p_cell, y_p),
                    })
                } else {
                    // If `k_0` is 1, simply return `Acc`
                    let x_p_cell = region.assign_advice(
                        || "x_p",
                        self.add_config.x_p,
                        k_0_row + offset,
                        || Ok(C::Base::zero()),
                    )?;

                    let y_p_cell = region.assign_advice(
                        || "y_p",
                        self.add_config.y_p,
                        k_0_row + offset,
                        || Ok(C::Base::zero()),
                    )?;
                    Ok(EccPoint {
                        x: CellValue::<C::Base>::new(x_p_cell, Some(C::Base::zero())),
                        y: CellValue::<C::Base>::new(y_p_cell, Some(C::Base::zero())),
                    })
                }
            })
            .unwrap_or(Err(Error::SynthesisError))?;

        // Return the result of the final complete addition as `[scalar]B`
        self.add_config
            .assign_region(&p, &acc, k_0_row + offset, region)
    }
}

#[derive(Clone, Debug)]
// `x`-coordinate of the accumulator.
struct X<F: FieldExt>(CellValue<F>);
impl<F: FieldExt> Deref for X<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
// `y`-coordinate of the accumulator.
struct Y<F: FieldExt>(Option<F>);
impl<F: FieldExt> Deref for Y<F> {
    type Target = Option<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
// Cumulative sum `z` used to decompose the scalar.
struct Z<F: FieldExt>(CellValue<F>);
impl<F: FieldExt> Deref for Z<F> {
    type Target = CellValue<F>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn decompose_scalar<C: CurveAffine>(scalar: C::Base) -> Vec<bool> {
    // Cast from base field into scalar field.
    let scalar = C::Scalar::from_bytes(&scalar.to_bytes()).unwrap();

    // The scalar field `F_q = 2^254 + t_q`.
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
