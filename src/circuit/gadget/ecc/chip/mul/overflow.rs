use super::super::{copy, CellValue, EccConfig, EccScalarVar, Var};
use super::Z;
use crate::{
    circuit::gadget::utilities::lookup_range_check::LookupRangeCheckConfig, constants::T_Q,
    primitives::sinsemilla, spec::lebs2ip_field,
};
use halo2::{
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};

use ff::{Field, PrimeFieldBits};
use pasta_curves::{arithmetic::FieldExt, pallas};

use std::{convert::TryInto, iter};

pub struct Config {
    // Selector to check decomposition of lsb
    q_mul_z: Selector,
    // Selector to check z_0 = alpha + t_q (mod p)
    q_mul_overflow: Selector,
    // Selector to constrain s_{120..=125} of s = alpha + k_{254} * 2^127
    q_three_bit: Selector,
    // Selector to constrain s_{126} of s = alpha + k_{254} * 2^127
    q_one_bit: Selector,
    // 10-bit lookup table
    lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    // Advice columns
    advices: [Column<Advice>; 3],
    // Permutation
    perm: Permutation,
}

impl From<&EccConfig> for Config {
    fn from(ecc_config: &EccConfig) -> Self {
        Self {
            q_mul_z: ecc_config.q_mul_z,
            q_mul_overflow: ecc_config.q_mul_overflow,
            q_three_bit: ecc_config.q_three_bit,
            q_one_bit: ecc_config.q_one_bit,
            lookup_config: ecc_config.lookup_config.clone(),
            advices: [
                ecc_config.advices[0],
                ecc_config.advices[1],
                ecc_config.advices[2],
            ],
            perm: ecc_config.perm.clone(),
        }
    }
}

impl Config {
    pub(super) fn create_gate(&self, meta: &mut ConstraintSystem<pallas::Base>) {
        meta.create_gate("overflow checks", |meta| {
            let q_mul_overflow = meta.query_selector(self.q_mul_overflow);

            // Constant expressions
            let one = Expression::Constant(pallas::Base::one());
            let two_pow_127 = Expression::Constant(pallas::Base::from_u128(1 << 127));

            let z_0 = meta.query_advice(self.advices[0], Rotation::prev());
            let z_127 = meta.query_advice(self.advices[0], Rotation::cur());
            let eta = meta.query_advice(self.advices[0], Rotation::next());

            let k_254 = meta.query_advice(self.advices[1], Rotation::prev());
            let alpha = meta.query_advice(self.advices[1], Rotation::cur());

            // s_minus_lo_127 = s - sum_{i = 0}^{126} 2^i ⋅ s_i
            let s_minus_lo_127 = meta.query_advice(self.advices[1], Rotation::next());

            let s = meta.query_advice(self.advices[2], Rotation::cur());
            let s_check = s - (alpha.clone() + k_254.clone() * two_pow_127.clone());

            // q = 2^254 + t_q is the Pallas scalar field modulus.
            // We cast t_q into the base field to check alpha + t_q (mod p).
            let t_q = pallas::Base::from_u128(T_Q);
            let t_q = Expression::Constant(t_q);

            // z_0 - alpha - t_q = 0
            let recovery = z_0 - alpha - t_q;

            // k_254 * (z_{127} - 2^127) = 0
            let lo_zero = k_254.clone() * (z_127.clone() - two_pow_127);

            // k_254 * s_minus_lo_127 = 0
            let s_minus_lo_127_check = k_254.clone() * s_minus_lo_127.clone();

            // (1 - k_254) * (1 - z_{127} * eta) * s_minus_lo_127 = 0
            let canonicity = (one.clone() - k_254) * (one - z_127 * eta) * s_minus_lo_127;

            iter::empty()
                .chain(Some(s_check))
                .chain(Some(recovery))
                .chain(Some(lo_zero))
                .chain(Some(s_minus_lo_127_check))
                .chain(Some(canonicity))
                .map(|poly| q_mul_overflow.clone() * poly)
                .collect::<Vec<_>>()
        });

        meta.create_gate("three-bit range check", |meta| {
            let q_three_bit = meta.query_selector(self.q_three_bit);
            let z_prev = meta.query_advice(self.lookup_config.running_sum, Rotation::prev());
            let z_cur = meta.query_advice(self.lookup_config.running_sum, Rotation::cur());

            // z_cur = (z_prev - word) / (2^3)
            // word = z_prev - z_cur * 2^3
            let three_bit_value = z_prev - z_cur * pallas::Base::from_u64(1 << 3);

            let one = Expression::Constant(pallas::Base::one());

            let mut three_bit_check = one;
            for i in 0..(7 + 1) {
                three_bit_check = three_bit_check
                    * (three_bit_value.clone() - Expression::Constant(pallas::Base::from_u64(i)))
            }

            vec![q_three_bit * three_bit_check]
        });

        meta.create_gate("one-bit range check", |meta| {
            let q_one_bit = meta.query_selector(self.q_one_bit);
            let z_prev = meta.query_advice(self.lookup_config.running_sum, Rotation::prev());
            let z_cur = meta.query_advice(self.lookup_config.running_sum, Rotation::cur());

            // z_cur = (z_prev - word) / (2)
            // word = z_prev - z_cur * 2
            let one_bit_value = z_prev - z_cur * pallas::Base::from_u64(2);

            let one = Expression::Constant(pallas::Base::one());
            vec![q_one_bit * one_bit_value.clone() * (one - one_bit_value)]
        });
    }

    pub(super) fn overflow_check(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        alpha: EccScalarVar,
        zs: &[Z<pallas::Base>], // [z_0, z_1, ..., z_{254}, z_{255}]
    ) -> Result<(), Error> {
        // s = alpha + k_{254} ⋅ 2^{127} is witnessed here, and then copied into
        // the decomposition as well as the overflow check gate.
        // In the overflow check gate, we check that s is properly derived
        // from alpha and k_{254}.
        let s = {
            let k_254 = *zs[254];
            let s_val = alpha
                .value()
                .zip(k_254.value())
                .map(|(alpha, k_254)| alpha + k_254 * pallas::Base::from_u128(1 << 127));

            layouter.assign_region(
                || "s = alpha + k_{254} ⋅ 2^{127}",
                |mut region| {
                    let s_cell = region.assign_advice(
                        || "s = alpha + k_{254} ⋅ 2^{127}",
                        self.advices[0],
                        0,
                        || s_val.ok_or(Error::SynthesisError),
                    )?;
                    Ok(CellValue::new(s_cell, s_val))
                },
            )?
        };

        // Subtract the first 127 low bits of s = alpha + k_{254} ⋅ 2^{127}
        // using:
        // - twelve 10-bit lookups for the first 120 bits, s_{0..=119}
        // - two 3-bit range checks for bits 120..=125, s_{120..=125}
        // - a single boolean constraint for s_126
        let s_minus_lo_127 =
            self.s_minus_lo_127(layouter.namespace(|| "decompose s_{0..=126}"), s)?;

        layouter.assign_region(
            || "overflow check",
            |mut region| {
                let offset = 0;

                // Enable overflow check gate
                self.q_mul_overflow.enable(&mut region, offset + 1)?;

                // Copy `z_0`
                copy(
                    &mut region,
                    || "copy z_0",
                    self.advices[0],
                    offset,
                    &*zs[0],
                    &self.perm,
                )?;

                // Copy `z_127`
                copy(
                    &mut region,
                    || "copy z_127",
                    self.advices[0],
                    offset + 1,
                    &*zs[127],
                    &self.perm,
                )?;

                // Witness η = inv0(z_127), where inv0(x) = 0 if x = 0, 1/x otherwise
                {
                    let eta = zs[127].value().map(|z_127| {
                        if z_127 == pallas::Base::zero() {
                            pallas::Base::zero()
                        } else {
                            z_127.invert().unwrap()
                        }
                    });
                    region.assign_advice(
                        || "η = inv0(z_127)",
                        self.advices[0],
                        offset + 2,
                        || eta.ok_or(Error::SynthesisError),
                    )?;
                }

                // Copy `k_254` = z_254
                copy(
                    &mut region,
                    || "copy k_254",
                    self.advices[1],
                    offset,
                    &*zs[254],
                    &self.perm,
                )?;

                // Copy original alpha
                copy(
                    &mut region,
                    || "copy original alpha",
                    self.advices[1],
                    offset + 1,
                    &*alpha,
                    &self.perm,
                )?;

                // Copy weighted sum of the decomposition of the lower 126 bits
                // of s = alpha + k_{254} ⋅ 2^{127}
                copy(
                    &mut region,
                    || "copy s_minus_lo_127",
                    self.advices[1],
                    offset + 2,
                    &s_minus_lo_127,
                    &self.perm,
                )?;

                // Copy witnessed s to check that it was properly derived from alpha and k_{254}.
                copy(
                    &mut region,
                    || "copy s",
                    self.advices[2],
                    offset + 1,
                    &s,
                    &self.perm,
                )?;

                Ok(())
            },
        )?;

        Ok(())
    }

    fn s_minus_lo_127(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        s: CellValue<pallas::Base>,
    ) -> Result<CellValue<pallas::Base>, Error> {
        // Number of k-bit words we can use in the lookup decomposition.
        let num_words = 127 / sinsemilla::K;
        // 1 /(2^3) = 1 / 8
        let eight_inv = pallas::Base::from_u64(1 << 3).invert().unwrap();

        // Decompose the low 120 bits of `s` using twelve 10-bit lookups.
        let s_minus_lo_120 = {
            let zs = self.lookup_config.lookup_range_check(
                layouter.namespace(|| "Decompose low 120 bits of s"),
                s,
                num_words,
            )?;
            zs[zs.len() - 1]
        };

        layouter.assign_region(
            || "decompose s = alpha + k_{254} ⋅ 2^{127}",
            |mut region| {
                let offset = 0;
                // Decompose bits 120..=126 of `s` using:
                // - a 3-bit range check for s_{120..=122},
                // - a 3-bit range check for s_{122..=125},
                // - a 1-bit range check for s_{126}.
                //
                let (lo_three, hi_three, s_126) = {
                    // `leftover_bits` are s_{120..=126}.
                    let leftover_bits = s_minus_lo_120.value().map(|value| {
                        value
                            .to_le_bits()
                            .iter()
                            .by_val()
                            .take(7)
                            .collect::<Vec<_>>()
                    });

                    if let Some(leftover_bits) = leftover_bits {
                        // s_{120..=122}
                        let lo_three = lebs2ip_field::<pallas::Base, 3>(
                            &leftover_bits[..3].try_into().unwrap(),
                        );

                        // s_{123..=125}
                        let hi_three = lebs2ip_field::<pallas::Base, 3>(
                            &leftover_bits[3..6].try_into().unwrap(),
                        );

                        // s_{126}
                        let s_126 = pallas::Base::from_u64(leftover_bits[6] as u64);

                        (Some(lo_three), Some(hi_three), Some(s_126))
                    } else {
                        (None, None, None)
                    }
                };

                // Subtract s_{120..=122}
                let s_minus_lo_123 = {
                    // Enable three-bit range check
                    self.q_three_bit.enable(&mut region, offset)?;

                    // Compute s_minus_lo_123 = (s_minus_lo_120 - lo_three) / (2^3)
                    let s_minus_lo_123 = s_minus_lo_120
                        .value()
                        .zip(lo_three)
                        .map(|(z_cur, lo)| (z_cur - lo) * eight_inv);

                    // Witness s_minus_lo_123
                    let cell = region.assign_advice(
                        || "s_{120..=122}",
                        self.lookup_config.running_sum,
                        offset,
                        || s_minus_lo_123.ok_or(Error::SynthesisError),
                    )?;

                    CellValue::new(cell, s_minus_lo_123)
                };

                let offset = offset + 1;

                // Subtract s_{123..=125}
                let s_minus_lo_126 = {
                    // Enable three-bit range check
                    self.q_three_bit.enable(&mut region, offset)?;

                    // Compute s_minus_lo_126 = (s_minus_lo_123 - hi_three) / (2^3)
                    let s_minus_lo_126 = s_minus_lo_123
                        .value()
                        .zip(hi_three)
                        .map(|(z_cur, hi)| (z_cur - hi) * eight_inv);

                    // Witness s_minus_lo_126
                    let cell = region.assign_advice(
                        || "s_{123..=125}",
                        self.lookup_config.running_sum,
                        offset,
                        || s_minus_lo_126.ok_or(Error::SynthesisError),
                    )?;

                    CellValue::new(cell, s_minus_lo_126)
                };

                let offset = offset + 1;

                // Subtract s_126
                let s_minus_lo_127 = {
                    // Enable one-bit range check
                    self.q_one_bit.enable(&mut region, offset)?;

                    // Compute s_minus_lo_127 = (s_minus_lo_126 - s_126) / 2
                    let s_minus_lo_127 = s_minus_lo_126
                        .value()
                        .zip(s_126)
                        .map(|(z_cur, s_126)| (z_cur - s_126) * pallas::Base::TWO_INV);

                    // Witness s_minus_lo_127
                    let cell = region.assign_advice(
                        || "s_{126}",
                        self.lookup_config.running_sum,
                        offset,
                        || s_minus_lo_127.ok_or(Error::SynthesisError),
                    )?;
                    CellValue::new(cell, s_minus_lo_127)
                };

                Ok(s_minus_lo_127)
            },
        )
    }
}
