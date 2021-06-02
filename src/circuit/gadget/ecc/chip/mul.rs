use super::{add, util, CellValue, EccConfig, EccPoint};
use crate::constants::NUM_COMPLETE_BITS;
use std::ops::{Deref, Range};

use ff::PrimeField;
use halo2::{
    arithmetic::{CurveAffine, Field, FieldExt},
    circuit::Region,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};

mod complete;
mod incomplete;

pub struct Config<C: CurveAffine> {
    // Selector used to constrain the cells used in complete addition.
    q_mul_complete: Selector,
    // Selector used to check recovery of the original scalar after decomposition.
    q_mul_decompose_var: Selector,
    // Advice column used to decompose scalar in complete addition.
    z_complete: Column<Advice>,
    // Advice column where the scalar is copied for use in the final recovery check.
    scalar: Column<Advice>,
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
        let config = Self {
            q_mul_complete: ecc_config.q_mul_complete,
            q_mul_decompose_var: ecc_config.q_mul_decompose_var,
            z_complete: ecc_config.advices[9],
            scalar: ecc_config.advices[1],
            perm: ecc_config.perm.clone(),
            add_config: ecc_config.into(),
            hi_config: incomplete::Config::hi_config(ecc_config),
            lo_config: incomplete::Config::lo_config(ecc_config),
        };

        assert_eq!(
            config.hi_config.x_p, config.lo_config.x_p,
            "x_p is shared across hi and lo halves."
        );
        assert_eq!(
            config.hi_config.y_p, config.lo_config.y_p,
            "y_p is shared across hi and lo halves."
        );

        let add_config_advices = config.add_config.advice_columns();
        assert!(
            !add_config_advices.contains(&config.z_complete),
            "z_complete cannot overlap with complete addition columns."
        );
        assert!(
            !add_config_advices.contains(&config.hi_config.z),
            "hi_config z cannot overlap with complete addition columns."
        );

        config
    }
}

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
        meta.create_gate("Decompose scalar for variable-base mul", |meta| {
            let q_mul_decompose_var =
                meta.query_selector(self.q_mul_decompose_var, Rotation::cur());
            let scalar = meta.query_advice(self.scalar, Rotation::cur());
            let z_cur = meta.query_advice(self.z_complete, Rotation::cur());

            // The scalar field `F_q = 2^254 + t_q`.
            // -((2^127)^2) = -(2^254) = t_q (mod q)
            let t_q = -(C::Scalar::from_u128(1u128 << 127).square());
            let t_q = C::Base::from_bytes(&t_q.to_bytes()).unwrap();

            // Check that `k = scalar + t_q`
            vec![q_mul_decompose_var * (scalar + Expression::Constant(t_q) - z_cur)]
        });
    }

    pub(super) fn assign_region(
        &self,
        scalar: &CellValue<C::Base>,
        base: &EccPoint<C>,
        offset: usize,
        region: &mut Region<'_, C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        // Initialize the accumulator `acc = [2]base`
        let acc = self
            .add_config
            .assign_region(&base, &base, offset, region)?;

        // Increase the offset by 1 after complete addition.
        let offset = offset + 1;

        // Decompose the scalar bitwise (big-endian bit order).
        let bits = decompose_for_scalar_mul::<C>(scalar.value);

        // Initialize the running sum for scalar decomposition to zero
        let z_val = C::Base::zero();
        let z_cell =
            region.assign_advice(|| "initial z", self.hi_config.z, offset, || Ok(z_val))?;
        let z = CellValue::new(z_cell, Some(z_val));

        // Increase the offset by 1 after initializing `z`.
        let offset = offset + 1;

        // Double-and-add (incomplete addition) for the `hi` half of the scalar decomposition
        let bits_incomplete_hi = &bits[incomplete_hi_range::<C>()];
        let (x, y_a, z) = self.hi_config.double_and_add(
            region,
            offset,
            &base,
            bits_incomplete_hi,
            (X(acc.x.clone()), Y(acc.y.value), Z(z)),
        )?;

        // Double-and-add (incomplete addition) for the `lo` half of the scalar decomposition
        let bits_incomplete_lo = &bits[incomplete_lo_range::<C>()];
        let (x, y_a, z) = self.lo_config.double_and_add(
            region,
            offset,
            &base,
            bits_incomplete_lo,
            (x, y_a, z),
        )?;

        // Move from incomplete addition to complete addition
        let offset = offset + incomplete_lo_len::<C>() + 1;

        // Get value of acc after incomplete addition
        let acc = {
            // Assign final `y_a` output from incomplete addition
            let y_a_cell = region.assign_advice(
                || "y_a",
                self.add_config.y_qr,
                offset,
                || y_a.ok_or(Error::SynthesisError),
            )?;

            EccPoint {
                x: x.0,
                y: CellValue::<C::Base>::new(y_a_cell, *y_a),
            }
        };

        // Initialize `z` running sum for complete addition
        util::assign_and_constrain(
            region,
            || "Initialize `z` running sum for complete addition",
            self.z_complete,
            offset,
            &z,
            &self.perm,
        )?;

        // Increase the offset by 1 after complete addition.
        let offset = offset + 1;

        // Complete addition
        let (acc, z_val) = {
            let complete_config: complete::Config<C> = self.into();
            // Bits used in complete addition. k_{3} to k_{1} inclusive
            // The LSB k_{0} is handled separately.
            let bits_complete = &bits[complete_range::<C>()];
            complete_config.assign_region(region, offset, bits_complete, base, acc, z.value)?
        };

        let offset = offset + complete_len::<C>() * 2;

        // Process the least significant bit
        let lsb = bits[C::Scalar::NUM_BITS as usize - 1];
        let result = self.process_lsb(region, offset, scalar, base, acc, lsb, z_val)?;

        #[cfg(test)]
        // Check that the correct multiple is obtained.
        {
            use group::Curve;

            let base = base.point();
            let scalar = scalar
                .value
                .map(|scalar| C::Scalar::from_bytes(&scalar.to_bytes()).unwrap());
            let real_mul = base.zip(scalar).map(|(base, scalar)| base * scalar);
            let result = result.point();

            assert_eq!(real_mul.unwrap().to_affine(), result.unwrap());
        }

        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    fn process_lsb(
        &self,
        region: &mut Region<'_, C::Base>,
        offset: usize,
        scalar: &CellValue<C::Base>,
        base: &EccPoint<C>,
        acc: EccPoint<C>,
        lsb: Option<bool>,
        mut z_val: Option<C::Base>,
    ) -> Result<EccPoint<C>, Error> {
        // Assign the final `z` value.
        z_val = z_val
            .zip(lsb)
            .map(|(z_val, lsb)| C::Base::from_u64(2) * z_val + C::Base::from_u64(lsb as u64));
        region.assign_advice(
            || "final z",
            self.z_complete,
            offset,
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
        util::assign_and_constrain(
            region,
            || "original scalar",
            self.scalar,
            offset,
            &scalar,
            &self.perm,
        )?;
        self.q_mul_decompose_var.enable(region, offset)?;

        // If `lsb` is 0, return `Acc + (-P)`. If `lsb` is 1, simply return `Acc + 0`.
        let x_p = if let Some(lsb) = lsb {
            if !lsb {
                base.x.value
            } else {
                Some(C::Base::zero())
            }
        } else {
            None
        };
        let y_p = if let Some(lsb) = lsb {
            if !lsb {
                base.y.value.map(|y_p| -y_p)
            } else {
                Some(C::Base::zero())
            }
        } else {
            None
        };

        let x_p_cell = region.assign_advice(
            || "x_p",
            self.add_config.x_p,
            offset + 1,
            || x_p.ok_or(Error::SynthesisError),
        )?;

        let y_p_cell = region.assign_advice(
            || "y_p",
            self.add_config.y_p,
            offset + 1,
            || y_p.ok_or(Error::SynthesisError),
        )?;

        let p = EccPoint {
            x: CellValue::<C::Base>::new(x_p_cell, x_p),
            y: CellValue::<C::Base>::new(y_p_cell, y_p),
        };

        // Return the result of the final complete addition as `[scalar]B`
        self.add_config.assign_region(&p, &acc, offset + 1, region)
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

fn decompose_for_scalar_mul<C: CurveAffine>(scalar: Option<C::Base>) -> Vec<Option<bool>> {
    let bits = scalar.map(|scalar| {
        // Cast from base field into scalar field.
        // Assumptions:
        // - The witnessed scalar field element fits into the base field.
        // - The scalar field order is larger than the base field order.
        let scalar = C::Scalar::from_bytes(&scalar.to_bytes()).unwrap();

        // The scalar field `F_q = 2^254 + t_q`.
        // -((2^127)^2) = -(2^254) = t_q (mod q)
        //
        // Assumptions:
        // - The scalar field can be represented in 255 bits.
        assert_eq!(C::Scalar::NUM_BITS, 255);
        let t_q = -(C::Scalar::from_u128(1u128 << 127).square());

        // We will witness `k = scalar + t_q`
        // `k` is decomposed bitwise in-circuit for our double-and-add algorithm.
        let k = scalar + t_q;

        // `k` is decomposed bitwise (big-endian) into `[k_n, ..., lsb]`, where
        // each `k_i` is a bit and `scalar = k_n * 2^n + ... + k_1 * 2 + lsb`.
        let mut bits: Vec<bool> = k
            .to_le_bits()
            .into_iter()
            .take(C::Scalar::NUM_BITS as usize)
            .collect();
        bits.reverse();
        assert_eq!(bits.len(), C::Scalar::NUM_BITS as usize);

        bits
    });

    if let Some(bits) = bits {
        bits.into_iter().map(Some).collect()
    } else {
        vec![None; C::Scalar::NUM_BITS as usize]
    }
}

// Bits used in incomplete addition. k_{254} to k_{4} inclusive
fn incomplete_len<C: CurveAffine>() -> usize {
    C::Scalar::NUM_BITS as usize - 1 - NUM_COMPLETE_BITS
}

fn incomplete_range<C: CurveAffine>() -> Range<usize> {
    0..incomplete_len::<C>()
}

// Bits used in `lo` half of incomplete addition
fn incomplete_lo_range<C: CurveAffine>() -> Range<usize> {
    (incomplete_len::<C>() / 2)..incomplete_len::<C>()
}

// Bits used in `hi` half of incomplete addition
fn incomplete_hi_range<C: CurveAffine>() -> Range<usize> {
    0..(incomplete_len::<C>() / 2)
}

// Bits k_{254} to k_{4} inclusive are used in incomplete addition.
// The `lo` half is k_{129} to k_{4} inclusive (length 126 bits).
fn incomplete_lo_len<C: CurveAffine>() -> usize {
    (incomplete_len::<C>() + 1) / 2
}

// Bits k_{254} to k_{4} inclusive are used in incomplete addition.
// The `hi` half is k_{254} to k_{130} inclusive (length 125 bits).
fn incomplete_hi_len<C: CurveAffine>() -> usize {
    incomplete_len::<C>() / 2
}

fn complete_range<C: CurveAffine>() -> Range<usize> {
    incomplete_len::<C>()..(C::Scalar::NUM_BITS as usize - 1)
}

fn complete_len<C: CurveAffine>() -> usize {
    NUM_COMPLETE_BITS as usize
}

#[cfg(test)]
pub mod tests {
    use ff::Field;
    use halo2::{
        arithmetic::{CurveAffine, FieldExt},
        circuit::Layouter,
        plonk::Error,
    };

    use crate::circuit::gadget::ecc::{EccInstructions, Point, ScalarVar};

    pub fn test_mul<C: CurveAffine, EccChip: EccInstructions<C> + Clone + Eq + std::fmt::Debug>(
        chip: EccChip,
        mut layouter: impl Layouter<C::Base>,
        zero: &Point<C, EccChip>,
        p: &Point<C, EccChip>,
    ) -> Result<(), Error> {
        let scalar_val = C::Scalar::rand();
        let scalar_val = C::Base::from_bytes(&scalar_val.to_bytes()).unwrap();
        let scalar = ScalarVar::new(
            chip.clone(),
            layouter.namespace(|| "ScalarVar"),
            Some(scalar_val),
        )?;

        // [a]B
        p.mul(layouter.namespace(|| "mul"), &scalar)?;

        // [a]𝒪 should return an error since variable-base scalar multiplication
        // uses incomplete addition at the beginning of its double-and-add.
        zero.mul(layouter.namespace(|| "mul"), &scalar)
            .expect_err("[a]𝒪 should return an error");

        // [0]B should return (0,0) since variable-base scalar multiplication
        // uses complete addition for the final bits of the scalar.
        let scalar_val = C::Base::zero();
        let scalar = ScalarVar::new(chip, layouter.namespace(|| "ScalarVar"), Some(scalar_val))?;
        p.mul(layouter.namespace(|| "mul"), &scalar)?;

        Ok(())
    }
}
