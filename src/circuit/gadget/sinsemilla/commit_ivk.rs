use std::array;

use halo2::{
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};
use pasta_curves::{arithmetic::FieldExt, pallas};

use crate::{
    circuit::gadget::utilities::{
        copy, lookup_range_check::LookupRangeCheckConfig, CellValue, Var,
    },
    constants::T_P,
    primitives::sinsemilla,
};

use super::{
    chip::{SinsemillaChip, SinsemillaConfig},
    message::MessageSubPiece,
    Message, SinsemillaInstructions,
};

// <https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit>
// We need to hash `ak || nk` where each of `ak`, `nk` is a field element (255 bits).
//
// `a` = bits 0..=249 of `ak`
pub(in crate::circuit) struct A(CellValue<pallas::Base>);

// `b = b_0||b_1||b_2` = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=244 of  `nk`)
pub(in crate::circuit) struct B(
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
);

// `c = c_0||c_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)
pub(in crate::circuit) struct C(CellValue<pallas::Base>, CellValue<pallas::Base>);

#[derive(Clone, Debug)]
pub struct CommitIvkConfig {
    q_decompose: Selector,
    q_ak_canon: Selector,
    q_nk_canon: Selector,
    ak_nk: Column<Advice>,
    a_b: Column<Advice>,
    b0_b1: Column<Advice>,
    b2_c: Column<Advice>,
    c0_c1: Column<Advice>,
    perm: Permutation,
    sinsemilla_config: SinsemillaConfig,
    ak_lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    nk_lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
}

impl CommitIvkConfig {
    pub(in crate::circuit) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        sinsemilla_config: SinsemillaConfig,
    ) -> Self {
        let advices = sinsemilla_config.advices();
        let perm = sinsemilla_config.perm.clone();

        let ak_nk = advices[0];
        let a_b = advices[1];
        let b0_b1 = advices[2];
        let b2_c = advices[3];
        let c0_c1 = advices[4];

        let q_decompose = meta.selector();
        let q_ak_canon = meta.selector();
        let q_nk_canon = meta.selector();

        let lookup_table = sinsemilla_config.generator_table.table_idx;
        let ak_lookup_config =
            LookupRangeCheckConfig::configure(meta, advices[0], lookup_table, perm.clone());
        let nk_lookup_config =
            LookupRangeCheckConfig::configure(meta, advices[1], lookup_table, perm.clone());

        let config = Self {
            q_decompose,
            q_ak_canon,
            q_nk_canon,
            ak_nk,
            a_b,
            b0_b1,
            b2_c,
            c0_c1,
            perm,
            sinsemilla_config,
            ak_lookup_config,
            nk_lookup_config,
        };

        // <https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit>
        // We need to hash `ak || nk` where each of `ak`, `nk` is a field element (255 bits).
        //
        // `a` = bits 0..=249 of `ak`
        // `b = b_0||b_1||b_2` = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=244 of  `nk`)
        // `c = c_0||c_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)
        meta.create_gate("CommitIvk decomposition check", |meta| {
            let q_decompose = meta.query_selector(config.q_decompose);

            let a = meta.query_advice(config.a_b, Rotation::cur());
            let b_whole = meta.query_advice(config.a_b, Rotation::next());
            let c_whole = meta.query_advice(config.b2_c, Rotation::next());
            let ak = meta.query_advice(config.ak_nk, Rotation::cur());
            let nk = meta.query_advice(config.ak_nk, Rotation::next());

            let b_0 = meta.query_advice(config.b0_b1, Rotation::cur());
            let b_1 = meta.query_advice(config.b0_b1, Rotation::next());
            let b_2 = meta.query_advice(config.b2_c, Rotation::cur());
            let c_0 = meta.query_advice(config.c0_c1, Rotation::cur());
            let c_1 = meta.query_advice(config.c0_c1, Rotation::next());

            // `b = b_0||b_1||b_2` = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=244 of  `nk`)
            // Check that b = b_0 (4 bits) || b_1 (1 bit) || b_2 (245 bits)
            let b_check = b_0.clone()
                + b_1.clone() * pallas::Base::from_u64(1 << 4)
                + b_2.clone() * pallas::Base::from_u64(1 << 5)
                - b_whole;

            // `c = c_0||c_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)
            // Check that c = c_0 (9 bits) || c_1 (1 bit)
            let c_check = c_0.clone() + c_1.clone() * pallas::Base::from_u64(1 << 9) - c_whole;

            // Check that ak = a (250 bits) || b_0 (4 bits) || b_1 (1 bit)
            let two_pow_250 = pallas::Base::from_u128(1 << 125).square();
            let two_pow_254 = two_pow_250 * pallas::Base::from_u64(1 << 4);
            let ak_check = a + b_0 * two_pow_250 + b_1 * two_pow_254 - ak;

            // Check that nk = b_2 (245 bits) || c_0 (9 bits) || c_1 (1 bit)
            let two_pow_245 = pallas::Base::from_u64(1 << 49).pow(&[5, 0, 0, 0]);
            let nk_check = b_2 + c_0 * two_pow_245 + c_1 * two_pow_254 - nk;

            array::IntoIter::new([b_check, c_check, ak_check, nk_check])
                .map(move |poly| q_decompose.clone() * poly)
        });

        meta.create_gate("ak canonicity", |meta| {
            // ak = a (250 bits) || b_0 (4 bits) || b_1 (1 bit)
            let q_ak_canon = meta.query_selector(config.q_ak_canon);
            // The constraints in this gate are enforced if and only if `b_1` = 1.
            let b_1 = meta.query_advice(config.b0_b1, Rotation::next());

            // b_1 = 1 => b_0 = 0
            let b_0 = meta.query_advice(config.b0_b1, Rotation::cur());

            // b_1 = 1 => z_13 = 0, where z_13 is the 13th running sum
            // output by the 10-bit Sinsemilla decomposition of `a`.
            let z_13 = meta.query_advice(config.ak_nk, Rotation::cur());

            // Check that a_prime = a + 2^130 - t_P.
            let two_pow_130 = Expression::Constant(pallas::Base::from_u128(1 << 65).square());
            let t_p = Expression::Constant(pallas::Base::from_u128(T_P));
            let a = meta.query_advice(config.a_b, Rotation::cur());
            let a_prime = meta.query_advice(config.b2_c, Rotation::cur());
            let a_prime_check = a + two_pow_130 - t_p - a_prime;

            // Check that the running sum output by the 130-bit little-
            // endian decomposition of a_prime is zero.
            let a_prime_decomposition = meta.query_advice(config.c0_c1, Rotation::cur());

            array::IntoIter::new([b_0, z_13, a_prime_check, a_prime_decomposition])
                .map(move |poly| q_ak_canon.clone() * b_1.clone() * poly)
        });

        meta.create_gate("nk canonicity", |meta| {
            // nk = b_2 (245 bits) || c_0 (9 bits) || c_1 (1 bit)
            let q_nk_canon = meta.query_selector(config.q_nk_canon);
            // The constraints in this gate are enforced if and only if `b_1` = 1.
            let c_1 = meta.query_advice(config.c0_c1, Rotation::next());

            // c_1 = 1 => c_0 = 0
            let c_0 = meta.query_advice(config.c0_c1, Rotation::cur());

            // c_1 = 1 => z_14 = 0, where z_14 is the 14th running sum
            // output by the 10-bit Sinsemilla decomposition of `b`.
            let z_14 = meta.query_advice(config.ak_nk, Rotation::cur());

            // Check that b2_prime = b_2 + 2^140 - t_P.
            let two_pow_140 = Expression::Constant(pallas::Base::from_u128(1 << 70).square());
            let t_p = Expression::Constant(pallas::Base::from_u128(T_P));
            let b_2 = meta.query_advice(config.b2_c, Rotation::cur());
            let b2_prime = meta.query_advice(config.b2_c, Rotation::next());
            let b2_prime_check = b_2 + two_pow_140 - t_p - b2_prime;

            // Check that the running sum output by the 140-bit little-
            // endian decomposition of b2_prime is zero.
            let b2_prime_decomposition = meta.query_advice(config.b0_b1, Rotation::cur());

            array::IntoIter::new([c_0, z_14, b2_prime_check, b2_prime_decomposition])
                .map(move |poly| q_nk_canon.clone() * c_1.clone() * poly)
        });

        config
    }

    #[allow(non_snake_case)]
    #[allow(clippy::type_complexity)]
    pub(in crate::circuit) fn decompose(
        &self,
        chip: SinsemillaChip,
        mut layouter: impl Layouter<pallas::Base>,
        ak: CellValue<pallas::Base>,
        nk: CellValue<pallas::Base>,
    ) -> Result<
        (
            Message<pallas::Affine, SinsemillaChip, { sinsemilla::K }, { sinsemilla::C }>,
            (A, B, C),
        ),
        Error,
    > {
        // <https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit>
        // We need to hash `ak || nk` where each of `ak`, `nk` is a field element (255 bits).
        //
        // `a` = bits 0..=249 of `ak`
        // `b = b_0||b_1||b_2` = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=244 of  `nk`)
        // `c = c_0||c_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)

        // `a` = bits 0..=249 of `ak`
        let a = {
            let a: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (ak.value(), 0..250).into();

            chip.witness_message_piece_field(layouter.namespace(|| "a"), a.field_elem_subset(), 25)?
        };

        // `b = b_0||b_1||b_2` = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=244 of  `nk`)
        let b = {
            let b_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (ak.value(), 250..254).into();
            let b_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (ak.value(), 254..255).into();
            let b_2: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (nk.value(), 0..245).into();

            chip.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness b = b_0 || b_1 || b_2"),
                &[b_0, b_1, b_2],
            )?
        };

        // `c = c_0||c_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)
        let c = {
            let c_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (nk.value(), 245..254).into();
            let c_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (nk.value(), 254..255).into();

            chip.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness c = c_0 || c_1"),
                &[c_0, c_1],
            )?
        };

        // Check that the pieces have been decomposed properly.
        let (a_subpieces, b_subpieces, c_subpieces) = {
            layouter.assign_region(
                || "Check piece decomposition",
                |mut region| {
                    // Enable `q_decompose` which checks the decomposition.
                    self.q_decompose.enable(&mut region, 0)?;

                    // Copy `ak` at the correct position.
                    copy(&mut region, || "Copy ak", self.ak_nk, 0, &ak, &self.perm)?;

                    // Copy `nk` at the correct position.
                    copy(&mut region, || "Copy nk", self.ak_nk, 1, &nk, &self.perm)?;

                    // Copy and assign `a`.
                    let a_subpieces = copy(
                        &mut region,
                        || "copy a",
                        self.a_b,
                        0,
                        &a.cell_value(),
                        &self.perm,
                    )?;

                    // Copy and assign `b`.
                    copy(
                        &mut region,
                        || "copy b",
                        self.a_b,
                        1,
                        &b.cell_value(),
                        &self.perm,
                    )?;

                    // Copy and assign `c`.
                    copy(
                        &mut region,
                        || "copy c",
                        self.b2_c,
                        1,
                        &c.cell_value(),
                        &self.perm,
                    )?;

                    let b_subpieces = {
                        // Copy and assign `b_0`.
                        let b_0 = copy(
                            &mut region,
                            || "Copy and assign b_0",
                            self.b0_b1,
                            0,
                            &b.subpieces()[0].cell_value(),
                            &self.perm,
                        )?;

                        // Copy and assign `b_1`.
                        let b_1 = copy(
                            &mut region,
                            || "Copy and assign b_1",
                            self.b0_b1,
                            1,
                            &b.subpieces()[1].cell_value(),
                            &self.perm,
                        )?;

                        // Copy and assign `b_2`.
                        let b_2 = copy(
                            &mut region,
                            || "Copy and assign b_2",
                            self.b2_c,
                            0,
                            &b.subpieces()[2].cell_value(),
                            &self.perm,
                        )?;

                        B(b_0, b_1, b_2)
                    };

                    let c_subpieces = {
                        // Copy and assign `c_0`.
                        let c_0 = copy(
                            &mut region,
                            || "Copy and assign c_0",
                            self.c0_c1,
                            0,
                            &c.subpieces()[0].cell_value(),
                            &self.perm,
                        )?;

                        // Copy and assign `c_1`.
                        let c_1 = copy(
                            &mut region,
                            || "Copy and assign c_1",
                            self.c0_c1,
                            1,
                            &c.subpieces()[1].cell_value(),
                            &self.perm,
                        )?;

                        C(c_0, c_1)
                    };

                    Ok((A(a_subpieces), b_subpieces, c_subpieces))
                },
            )?
        };

        Ok((
            Message::from_pieces(chip, vec![a, b, c]),
            (a_subpieces, b_subpieces, c_subpieces),
        ))
    }

    pub(in crate::circuit) fn check_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        subpieces: (A, B, C),
        zs: Vec<Vec<CellValue<pallas::Base>>>,
    ) -> Result<(), Error> {
        /*
            We hashed `ak || nk` where each of `ak`, `nk` is a field element
            (255 bits).
            `a` = bits 0..=249 of `ak`
            `b = b_0||b_1||b_2` = (bits 250..=253 of `ak`) || (bit 254 of  `ak`) || (bits 0..=244 of  `nk`)
            `c = c_0||c_1` = (bits 245..=253 of `nk`) || (bit 254 of `nk`)

            zs = [
                [z_0, ..., z_25 for a],
                [z_0, ..., z_25 for b],
                [z_0, z_1 for c]
            ]
        */

        let (a, b, c) = subpieces;

        self.ak_canonicity(
            layouter.namespace(|| "ak canonicity"),
            a.0,
            b.0,
            b.1,
            zs[0][13],
        )?;

        self.nk_canonicity(
            layouter.namespace(|| "nk canonicity"),
            b.2,
            c.0,
            c.1,
            zs[1][14],
        )?;

        Ok(())
    }

    // Check canonicity of `ak` encoding
    fn ak_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        a: CellValue<pallas::Base>,
        b_0: CellValue<pallas::Base>,
        b_1: CellValue<pallas::Base>,
        z_13: CellValue<pallas::Base>,
    ) -> Result<(), Error> {
        // `ak` = `a (250 bits) || b_0 (4 bits) || b_1 (1 bit)`
        // - b_1 = 1 => b_0 = 0
        // - b_1 = 1 => a < t_P
        //     - (0 ≤ a < 2^130) => z_13 of SinsemillaHash(a) == 0
        //     - 0 ≤ a + 2^130 - t_P < 2^130 (thirteen 10-bit lookups)

        // Witness a_prime = a + 2^130 - t_P.
        // We will copy this value into a gate to check that it has been
        // correctly derived from a.
        let a_prime = layouter.assign_region(
            || "a + 2^130 - t_P",
            |mut region| {
                let a_prime = a.value().map(|a| {
                    let two_pow_130 = pallas::Base::from_u128(1u128 << 65).square();
                    let t_p = pallas::Base::from_u128(T_P);
                    a + two_pow_130 - t_p
                });
                let cell = region.assign_advice(
                    || "a + 2^130 - t_P",
                    self.ak_nk,
                    0,
                    || a_prime.ok_or(Error::SynthesisError),
                )?;
                Ok(CellValue::new(cell, a_prime))
            },
        )?;

        // Decompose the low 130 bits of a_prime = a + 2^130 - t_P, and output
        // the running sum at the end of it. If a_prime < 2^130, the running sum
        // will be 0.
        let a_prime_decomposition = {
            let zs = self.ak_lookup_config.lookup_range_check(
                layouter.namespace(|| "Decompose low 130 bits of (a + 2^130 - t_P)"),
                a_prime,
                13,
            )?;
            assert_eq!(zs.len(), 14); // [z_0, z_1, ..., z_13]
            zs[13]
        };

        layouter.assign_region(
            || "ak canonicity",
            |mut region| {
                self.q_ak_canon.enable(&mut region, 0)?;

                // Enforce b_1 = 1 => b_0 = 0 by copying b_0, b_1 to the correct
                // offsets for use in the gate.
                copy(&mut region, || "copy b_0", self.b0_b1, 0, &b_0, &self.perm)?;
                copy(&mut region, || "copy b_1", self.b0_b1, 1, &b_1, &self.perm)?;

                // Enforce b_1 = 0 => z_13 of SinsemillaHash(a) == 0 by copying
                // z_13 to the correct offset for use in the gate.
                copy(
                    &mut region,
                    || "copy z_13",
                    self.ak_nk,
                    0,
                    &z_13,
                    &self.perm,
                )?;

                // Check that a_prime = a + 2^130 - t_P was correctly derived
                // from a.
                copy(&mut region, || "copy a", self.a_b, 0, &a, &self.perm)?;
                copy(
                    &mut region,
                    || "copy a_prime",
                    self.b2_c,
                    0,
                    &a_prime,
                    &self.perm,
                )?;

                // Enforce b_1 = 0 => (0 ≤ a_prime < 2^130).
                // This is equivalent to enforcing that the running sum returned by the
                // 130-bit decomposition of a_prime is 0.
                copy(
                    &mut region,
                    || "copy a_prime_decomposition",
                    self.c0_c1,
                    0,
                    &a_prime_decomposition,
                    &self.perm,
                )?;
                Ok(())
            },
        )
    }

    // Check canonicity of `nk` encoding
    fn nk_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        b_2: CellValue<pallas::Base>,
        c_0: CellValue<pallas::Base>,
        c_1: CellValue<pallas::Base>,
        z_14: CellValue<pallas::Base>,
    ) -> Result<(), Error> {
        // `nk` = `b_2 (245 bits) || c_0 (9 bits) || c_1 (1 bit)`
        // - c_1 = 0 => c_0 = 0
        // - c_1 = 0 => b_2 < t_P
        //     - 0 ≤ b_2 < 2^135
        //         - b_2 is part of the Sinsemilla message piece
        //           b = b_0 (4 bits) || b_1 (1 bit) || b_2 (245 bits)
        //         - z_14 of SinsemillaHash(b) == 0 constrains the low 135 bits of b_2
        //     - 0 ≤ b_2 + 2^140 - t_P < 2^140 (fourteen 10-bit lookups)

        // Witness b2_prime = b_2 + 2^140 - t_P.
        // We will copy this value into a gate to check that it has been
        // correctly derived from b.
        let b2_prime = layouter.assign_region(
            || "a + 2^130 - t_P",
            |mut region| {
                let b2_prime = b_2.value().map(|b_2| {
                    let two_pow_140 = pallas::Base::from_u128(1u128 << 70).square();
                    let t_p = pallas::Base::from_u128(T_P);
                    b_2 + two_pow_140 - t_p
                });
                let cell = region.assign_advice(
                    || "b_2 + 2^140 - t_P",
                    self.ak_nk,
                    0,
                    || b2_prime.ok_or(Error::SynthesisError),
                )?;
                Ok(CellValue::new(cell, b2_prime))
            },
        )?;

        // Decompose the low 140 bits of b2_prime = b_2 + 2^140 - t_P, and output
        // the running sum at the end of it. If b2_prime < 2^140, the running sum
        // will be 0.
        let b2_prime_decomposition = {
            let zs = self.ak_lookup_config.lookup_range_check(
                layouter.namespace(|| "Decompose low 140 bits of (b_2 + 2^140 - t_P)"),
                b2_prime,
                14,
            )?;
            assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_14]
            zs[14]
        };

        layouter.assign_region(
            || "nk canonicity",
            |mut region| {
                self.q_nk_canon.enable(&mut region, 0)?;

                // Enforce c_1 = 1 => c_0 = 0 by copying c_0, c_1 to the correct
                // offsets for use in the gate.
                copy(&mut region, || "copy c_0", self.c0_c1, 0, &c_0, &self.perm)?;
                copy(&mut region, || "copy c_1", self.c0_c1, 1, &c_1, &self.perm)?;

                // Enforce c_1 = 0 => z_14 of SinsemillaHash(b) == 0 by copying
                // z_14 to the correct offset for use in the gate.
                copy(
                    &mut region,
                    || "copy z_14",
                    self.ak_nk,
                    0,
                    &z_14,
                    &self.perm,
                )?;

                // Check that b2_prime = a + 2^130 - t_P was correctly derived
                // from b_2
                copy(&mut region, || "copy b_2", self.b2_c, 0, &b_2, &self.perm)?;
                copy(
                    &mut region,
                    || "copy b2_prime",
                    self.b2_c,
                    1,
                    &b2_prime,
                    &self.perm,
                )?;

                // Enforce c_1 = 0 => (0 ≤ b2_prime < 2^140).
                // This is equivalent to enforcing that the running sum returned by the
                // 140-bit decomposition of b2_prime is 0.
                copy(
                    &mut region,
                    || "copy b2_prime_decomposition",
                    self.b0_b1,
                    0,
                    &b2_prime_decomposition,
                    &self.perm,
                )?;
                Ok(())
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::CommitIvkConfig;
    use crate::{
        circuit::gadget::{
            ecc::{
                chip::{EccChip, EccConfig},
                ScalarFixed,
            },
            sinsemilla::{
                chip::{SinsemillaChip, SinsemillaCommitDomains},
                CommitDomain,
            },
            utilities::{CellValue, UtilitiesInstructions},
        },
        constants::T_Q,
    };
    use halo2::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::FieldExt, pallas};

    use std::convert::TryInto;

    #[test]
    fn commit_ivk_canonicity_check() {
        #[derive(Default)]
        struct MyCircuit {
            ak: Option<pallas::Base>,
            nk: Option<pallas::Base>,
        }

        impl UtilitiesInstructions<pallas::Base> for MyCircuit {
            type Var = CellValue<pallas::Base>;
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = (CommitIvkConfig, EccConfig);
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self::default()
            }

            fn configure(meta: &mut ConstraintSystem<pallas::Base>) -> Self::Config {
                let advices = [
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                    meta.advice_column(),
                ];

                // Shared fixed column for loading constants
                let constants = meta.fixed_column();

                // Permutation over all advice columns
                let perm = meta.permutation(
                    &advices
                        .iter()
                        .map(|advice| (*advice).into())
                        .chain(Some(constants.into()))
                        .collect::<Vec<_>>(),
                );
                let table_idx = meta.fixed_column();
                let lookup = (table_idx, meta.fixed_column(), meta.fixed_column());

                let sinsemilla_config = SinsemillaChip::configure(
                    meta,
                    advices[..5].try_into().unwrap(),
                    lookup,
                    constants,
                    perm.clone(),
                );
                let commit_ivk_config = CommitIvkConfig::configure(meta, sinsemilla_config);

                let ecc_config = EccChip::configure(meta, advices, table_idx, constants, perm);

                (commit_ivk_config, ecc_config)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                let (commit_ivk_config, ecc_config) = config;

                // Load the Sinsemilla generator lookup table used by the whole circuit.
                SinsemillaChip::load(commit_ivk_config.sinsemilla_config.clone(), &mut layouter)?;

                // Construct a Sinsemilla chip
                let sinsemilla_chip =
                    SinsemillaChip::construct(commit_ivk_config.sinsemilla_config.clone());

                // Construct an ECC chip
                let ecc_chip = EccChip::construct(ecc_config);

                // Witness ak
                let ak = self.load_private(
                    layouter.namespace(|| "load ak"),
                    commit_ivk_config.ak_nk,
                    self.ak,
                )?;

                // Witness nk
                let nk = self.load_private(
                    layouter.namespace(|| "load nk"),
                    commit_ivk_config.ak_nk,
                    self.nk,
                )?;

                let (ak_nk, subpieces) = commit_ivk_config.decompose(
                    sinsemilla_chip.clone(),
                    layouter.namespace(|| "decompose ak, nk"),
                    ak,
                    nk,
                )?;

                let domain = CommitDomain::new(
                    sinsemilla_chip,
                    ecc_chip.clone(),
                    &SinsemillaCommitDomains::CommitIvk,
                );

                // Use a random scalar for rivk
                let rivk = ScalarFixed::new(
                    ecc_chip,
                    layouter.namespace(|| "rivk"),
                    Some(pallas::Scalar::rand()),
                )?;

                let (_ivk, zs) =
                    domain.short_commit(layouter.namespace(|| "CommitIvk"), ak_nk, rivk)?;

                commit_ivk_config.check_canonicity(
                    layouter.namespace(|| "Check canonicity of CommitIvk inputs"),
                    subpieces,
                    zs,
                )
            }
        }

        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        // Test different values of `ak`, `nk`
        let circuits = [
            // `ak` = 0, `nk` = 0
            MyCircuit {
                ak: Some(pallas::Base::zero()),
                nk: Some(pallas::Base::zero()),
            },
            // `ak` = T_Q - 1, `nk` = T_Q - 1
            MyCircuit {
                ak: Some(pallas::Base::from_u128(T_Q - 1)),
                nk: Some(pallas::Base::from_u128(T_Q - 1)),
            },
            // `ak` = T_Q, `nk` = T_Q
            MyCircuit {
                ak: Some(pallas::Base::from_u128(T_Q)),
                nk: Some(pallas::Base::from_u128(T_Q)),
            },
            // `ak` = 2^127 - 1, `nk` = 2^127 - 1
            MyCircuit {
                ak: Some(pallas::Base::from_u128((1 << 127) - 1)),
                nk: Some(pallas::Base::from_u128((1 << 127) - 1)),
            },
            // `ak` = 2^127, `nk` = 2^127
            MyCircuit {
                ak: Some(pallas::Base::from_u128(1 << 127)),
                nk: Some(pallas::Base::from_u128(1 << 127)),
            },
            // `ak` = 2^254 - 1, `nk` = 2^254 - 1
            MyCircuit {
                ak: Some(two_pow_254 - pallas::Base::one()),
                nk: Some(two_pow_254 - pallas::Base::one()),
            },
            // `ak` = 2^254, `nk` = 2^254
            MyCircuit {
                ak: Some(two_pow_254),
                nk: Some(two_pow_254),
            },
        ];

        for circuit in circuits.iter() {
            let prover = MockProver::<pallas::Base>::run(11, circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
