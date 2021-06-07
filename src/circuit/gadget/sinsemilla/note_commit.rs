use std::array;

use group::GroupEncoding;
use halo2::{
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Expression, Permutation, Selector},
    poly::Rotation,
};
use pasta_curves::{
    arithmetic::{CurveAffine, FieldExt},
    pallas,
};

use crate::{
    circuit::gadget::{
        ecc::chip::EccPoint,
        utilities::{copy, lookup_range_check::LookupRangeCheckConfig, CellValue, Var},
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
// We need to hash g★_d || pk★_d || i2lebsp_{64}(v) || rho || psi,
// where
// All bit ranges are inclusive.
//
// a = bits 0..=249 of x(g_d)
pub(in crate::circuit) struct A(CellValue<pallas::Base>);

// b = b_0 || b_1 || b_2 || b_3
//   = (bits 250..=253 of x(g_d)) || (bit 254 of x(g_d)) || (ỹ bit of g_d) || (bits 0..=3 of pk★_d)
pub(in crate::circuit) struct B(
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
);

// c = bits 4..=253 of pk★_d
pub(in crate::circuit) struct C(CellValue<pallas::Base>);

// d = d_0 || d_1 || d_2
//   = (bit 254 of x(pk_d)) || (ỹ bit of pk_d) || (bits 0..=7 of v)
pub(in crate::circuit) struct D(
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
);

// e = e_0 || e_1 = (bits 8..=63 of v) || (bits 0..=3 of rho)
pub(in crate::circuit) struct E(CellValue<pallas::Base>, CellValue<pallas::Base>);

// f = bits 4..=253 inclusive of rho
pub(in crate::circuit) struct F(CellValue<pallas::Base>);

// g = g_0 || g_1
//   = (bit 254 of rho) || (bits 0..=8 of psi)
pub(in crate::circuit) struct G(CellValue<pallas::Base>, CellValue<pallas::Base>);

// h = (bits 9..=248 of psi)
pub(in crate::circuit) struct H(CellValue<pallas::Base>);

// i = i_0 || i_1 || i_2
//   = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits
pub(in crate::circuit) struct I(
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
    CellValue<pallas::Base>,
);

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct NoteCommitConfig {
    q_decompose_1: Selector,
    q_decompose_2: Selector,
    q_gd_canon: Selector,
    q_pkd_canon: Selector,
    q_rho_canon: Selector,
    q_psi_canon: Selector,
    advices: [Column<Advice>; 5],
    perm: Permutation,
    sinsemilla_config: SinsemillaConfig,
    gd_lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    pkd_lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    rho_lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
    psi_lookup_config: LookupRangeCheckConfig<pallas::Base, { sinsemilla::K }>,
}

impl NoteCommitConfig {
    #[allow(non_snake_case)]
    #[allow(clippy::many_single_char_names)]
    pub(in crate::circuit) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        sinsemilla_config: SinsemillaConfig,
    ) -> Self {
        let advices = sinsemilla_config.advices();
        let perm = sinsemilla_config.perm.clone();

        let q_decompose_1 = meta.selector();
        let q_decompose_2 = meta.selector();

        let q_gd_canon = meta.selector();
        let q_pkd_canon = meta.selector();
        let q_rho_canon = meta.selector();
        let q_psi_canon = meta.selector();

        let lookup_table = sinsemilla_config.generator_table.table_idx;
        let gd_lookup_config =
            LookupRangeCheckConfig::configure(meta, advices[0], lookup_table, perm.clone());
        let pkd_lookup_config =
            LookupRangeCheckConfig::configure(meta, advices[1], lookup_table, perm.clone());
        let rho_lookup_config =
            LookupRangeCheckConfig::configure(meta, advices[2], lookup_table, perm.clone());
        let psi_lookup_config =
            LookupRangeCheckConfig::configure(meta, advices[3], lookup_table, perm.clone());

        let config = Self {
            q_decompose_1,
            q_decompose_2,
            q_gd_canon,
            q_pkd_canon,
            q_rho_canon,
            q_psi_canon,
            advices,
            perm,
            sinsemilla_config,
            gd_lookup_config,
            pkd_lookup_config,
            rho_lookup_config,
            psi_lookup_config,
        };

        // Useful constants
        let two_pow_4 = pallas::Base::from_u64(1 << 4);
        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        let two_pow_130 = Expression::Constant(pallas::Base::from_u128(1 << 65).square());
        let two_pow_140 = Expression::Constant(pallas::Base::from_u128(1 << 70).square());
        let t_p = Expression::Constant(pallas::Base::from_u128(T_P));

        meta.create_gate("NoteCommit decomposition check 1", |meta| {
            // - q_decompose_1 checks that:
            //     - b = b_0 + (2^4) b_1 + (2^5) b_2 + (2^6) b_3
            //     - d = d_0 + (2^1) d_1 + (2^2) d_2
            //     - value = d_2 + (2^8) e_0
            //     - x(pk_d) = b_3 + (2^4) c + (2^254) d_0
            //     - x(g_d) = a + (2^250) b_0 + (2^254) b_1
            //
            let q_decompose_1 = meta.query_selector(config.q_decompose_1);

            // Cells in the `advices[0]` advice column.
            let a = meta.query_advice(config.advices[0], Rotation::prev());
            let b = meta.query_advice(config.advices[0], Rotation::cur());
            let d = meta.query_advice(config.advices[0], Rotation::next());

            // Cells in the `advices[1]` advice column.
            let b_0 = meta.query_advice(config.advices[1], Rotation::prev());
            let c = meta.query_advice(config.advices[1], Rotation::cur());
            let e_0 = meta.query_advice(config.advices[1], Rotation::next());

            // Cells in the `advices[2]` advice column.
            let b_1 = meta.query_advice(config.advices[2], Rotation::prev());
            let d_0 = meta.query_advice(config.advices[2], Rotation::cur());
            let value = meta.query_advice(config.advices[2], Rotation::next());

            // Cells in the `advices[3]` advice column.
            let b_2 = meta.query_advice(config.advices[3], Rotation::prev());
            let d_1 = meta.query_advice(config.advices[3], Rotation::cur());
            let pkd_x = meta.query_advice(config.advices[3], Rotation::next());

            // Cells in the `advices[4]` advice column.
            let b_3 = meta.query_advice(config.advices[4], Rotation::prev());
            let d_2 = meta.query_advice(config.advices[4], Rotation::cur());
            let gd_x = meta.query_advice(config.advices[4], Rotation::next());

            // b = b_0 + (2^4) b_1 + (2^5) b_2 + (2^6) b_3
            let b_check = {
                let sum = b_0.clone()
                    + b_1.clone() * pallas::Base::from_u64(1 << 4)
                    + b_2 * pallas::Base::from_u64(1 << 5)
                    + b_3.clone() * pallas::Base::from_u64(1 << 6);
                sum - b
            };

            // d = d_0 + (2^1) d_1 + (2^2) d_2
            let d_check = {
                let sum = d_0.clone()
                    + d_1 * pallas::Base::from_u64(1 << 1)
                    + d_2.clone() * pallas::Base::from_u64(1 << 2);
                sum - d
            };

            // value = d_2 + (2^8) e_0
            let value_check = {
                let sum = d_2 + e_0 * pallas::Base::from_u64(1 << 8);
                sum - value
            };

            // x(pk_d) = b_3 + (2^4) c + (2^254) d_0
            let pkd_x_check = {
                let sum = b_3 + c * two_pow_4 + d_0 * two_pow_254;
                sum - pkd_x
            };

            // x(g_d) = a + (2^250) b_0 + (2^254) b_1
            let gd_x_check = {
                let two_pow_250 = pallas::Base::from_u128(1 << 125).square();
                let sum = a + b_0 * two_pow_250 + b_1 * two_pow_254;
                sum - gd_x
            };

            array::IntoIter::new([b_check, d_check, value_check, pkd_x_check, gd_x_check])
                .map(move |poly| q_decompose_1.clone() * poly)
        });

        meta.create_gate("NoteCommit decomposition check 2", |meta| {
            // - q_decompose_2 checks that:
            //     - e = e_0 + (2^56) e_1
            //     - rho = e_1 + (2^4) f + (2^254) g_0
            //     - psi = g_1 + (2^9) h_0 + (2^254) h_1
            //     - g = g_0 + (2) g_1
            //     - i = i_0 + (2^5) i_1 + (2^6) i_2
            //
            let q_decompose_2 = meta.query_selector(config.q_decompose_2);

            // Cells in the `advices[0]` advice column.
            let e_0 = meta.query_advice(config.advices[0], Rotation::prev());
            let g_1 = meta.query_advice(config.advices[0], Rotation::cur());
            let i_2 = meta.query_advice(config.advices[0], Rotation::next());

            // Cells in the `advices[1]` advice column.
            let e_1 = meta.query_advice(config.advices[1], Rotation::prev());
            let g = meta.query_advice(config.advices[1], Rotation::cur());
            let i = meta.query_advice(config.advices[1], Rotation::next());

            // Cells in the `advices[2]` advice column.
            let e = meta.query_advice(config.advices[2], Rotation::prev());
            let h = meta.query_advice(config.advices[2], Rotation::cur());

            // Cells in the `advices[3]` advice column.
            let f = meta.query_advice(config.advices[3], Rotation::prev());
            let i_0 = meta.query_advice(config.advices[3], Rotation::cur());
            let rho = meta.query_advice(config.advices[3], Rotation::next());

            // Cells in the `advices[4]` advice column.
            let g_0 = meta.query_advice(config.advices[4], Rotation::prev());
            let i_1 = meta.query_advice(config.advices[4], Rotation::cur());
            let psi = meta.query_advice(config.advices[4], Rotation::next());

            // e = e_0 + (2^56) e_1
            let e_check = {
                let sum = e_0 + e_1.clone() * pallas::Base::from_u64(1 << 56);
                sum - e
            };

            // rho = e_1 + (2^4) f + (2^254) g_0
            let rho_check = {
                let sum = e_1 + f * two_pow_4 + g_0.clone() * two_pow_254;
                sum - rho
            };

            // psi = g_1 + (2^9) h + (2^249) i_0 + (2^254) i_1
            let psi_check = {
                let two_pow_249 =
                    pallas::Base::from_u128(1 << 124).square() * pallas::Base::from_u128(2);
                let sum = g_1.clone()
                    + h * pallas::Base::from_u64(1 << 9)
                    + i_0.clone() * two_pow_249
                    + i_1.clone() * two_pow_254;
                sum - psi
            };

            // g = g_0 + (2) g_1
            let g_check = {
                let sum = g_0 + g_1 * pallas::Base::from_u64(2);
                sum - g
            };

            // i = i_0 + (2^5) i_1 + (2^6) i_2
            let i_check = {
                let two_pow_5 = pallas::Base::from_u64(1 << 5);
                let two_pow_6 = pallas::Base::from_u64(1 << 6);

                let sum = i_0 + i_1 * two_pow_5 + i_2 * two_pow_6;
                sum - i
            };

            array::IntoIter::new([e_check, rho_check, psi_check, g_check, i_check])
                .map(move |poly| q_decompose_2.clone() * poly)
        });

        meta.create_gate("x(g_d) canonicity check", |meta| {
            // x(g_d) = a (250 bits) || b_0 (4 bits) || b_1 (1 bit)
            let q_gd_canon = meta.query_selector(config.q_gd_canon);
            // The constraints in this gate are enforced if and only if `b_1` = 1.
            let b_1 = meta.query_advice(config.advices[0], Rotation::next());

            // b_1 = 1 => b_0 = 0
            let b_0 = meta.query_advice(config.advices[0], Rotation::cur());

            // b_1 = 1 => z_13 = 0, where z_13 is the 13th running sum
            // output by the 10-bit Sinsemilla decomposition of `a`.
            let z_13 = meta.query_advice(config.advices[1], Rotation::cur());

            // Check that a_prime = a + 2^130 - t_P.
            let a = meta.query_advice(config.advices[2], Rotation::cur());
            let a_prime = meta.query_advice(config.advices[3], Rotation::cur());
            let a_prime_check = a + two_pow_130 - t_p.clone() - a_prime;

            // Check that the running sum output by the 130-bit little-
            // endian decomposition of a_prime is zero.
            let a_prime_decomposition = meta.query_advice(config.advices[4], Rotation::cur());

            array::IntoIter::new([b_0, z_13, a_prime_check, a_prime_decomposition])
                .map(move |poly| q_gd_canon.clone() * b_1.clone() * poly)
        });

        meta.create_gate("x(pk_d) canonicity check", |meta| {
            // `x(pk_d)` = `b_3 (4 bits) || c (250 bits) || d_0 (1 bit)`
            let q_pkd_canon = meta.query_selector(q_pkd_canon);

            // The constraints in this gate are enforced if and only if `d_0` = 1.
            let d_0 = meta.query_advice(config.advices[0], Rotation::cur());

            // d_0 = 1 => lookup_decomposition = 0
            let lookup_decomposition = meta.query_advice(config.advices[0], Rotation::next());

            // d_0 = 1 => z_13c = 0
            let z_13c = meta.query_advice(config.advices[1], Rotation::cur());

            // Check `lookup_element` = b_3 + 2^4 c + 2^140 - t_P was correctly
            // derived from b_3, c.
            let b_3 = meta.query_advice(config.advices[2], Rotation::cur());
            let c = meta.query_advice(config.advices[3], Rotation::cur());
            let witnessed_element = meta.query_advice(config.advices[4], Rotation::cur());
            let expected_element = b_3 + (c * two_pow_4) + two_pow_140.clone() - t_p.clone();
            let lookup_element_check = witnessed_element - expected_element;

            array::IntoIter::new([lookup_decomposition, z_13c, lookup_element_check])
                .map(move |poly| q_pkd_canon.clone() * d_0.clone() * poly)
        });

        meta.create_gate("rho canonicity check", |meta| {
            let q_rho_canon = meta.query_selector(q_rho_canon);

            // The constraints in this gate are enforced if and only if `g_0` = 1.
            let g_0 = meta.query_advice(config.advices[0], Rotation::cur());

            // g_0 = 1 => lookup_decomposition = 0
            let lookup_decomposition = meta.query_advice(config.advices[1], Rotation::cur());

            // g_0 = 1 => z_13f = 0
            let z_13f = meta.query_advice(config.advices[1], Rotation::next());

            // Check `lookup_element` = e_1 + 2^4 f + 2^140 - t_P was correctly
            // derived from e_1, f.
            let e_1 = meta.query_advice(config.advices[2], Rotation::cur());
            let f = meta.query_advice(config.advices[3], Rotation::cur());
            let witnessed_element = meta.query_advice(config.advices[4], Rotation::cur());
            let expected_element = e_1 + (f * two_pow_4) + two_pow_140.clone() - t_p.clone();
            let lookup_element_check = witnessed_element - expected_element;

            array::IntoIter::new([lookup_decomposition, z_13f, lookup_element_check])
                .map(move |poly| q_rho_canon.clone() * g_0.clone() * poly)
        });

        meta.create_gate("psi canonicity check", |meta| {
            // `psi` = `g_1 (9 bits) || h (240 bits) || i_0 (5 bits) || i_1 (1 bit)`
            let q_psi_canon = meta.query_selector(q_psi_canon);

            // The constraints in this gate are enforced if and only if `i_1` = 1.
            let i_1 = meta.query_advice(config.advices[0], Rotation::cur());

            // i_1 = 1 => lookup_decomposition = 0
            let lookup_decomposition = meta.query_advice(config.advices[0], Rotation::next());

            // i_1 = 1 => z_13h = 0
            let z_13h = meta.query_advice(config.advices[1], Rotation::cur());

            // Check `lookup_element` = g_1 + 2^9 h_0 + 2^140 - t_P < 2^140 was correctly
            // derived from g_1, h_0.
            let g_1 = meta.query_advice(config.advices[2], Rotation::cur());
            let h_0 = meta.query_advice(config.advices[3], Rotation::cur());
            let witnessed_element = meta.query_advice(config.advices[4], Rotation::cur());
            let expected_element = g_1 + (h_0 * two_pow_4) + two_pow_140.clone() - t_p.clone();
            let lookup_element_check = witnessed_element - expected_element;

            // i_1 = 1 => i_0 = 0
            let i_0 = meta.query_advice(config.advices[4], Rotation::next());

            array::IntoIter::new([lookup_decomposition, z_13h, lookup_element_check, i_0])
                .map(move |poly| q_psi_canon.clone() * i_1.clone() * poly)
        });

        config
    }

    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub(in crate::circuit) fn decompose(
        &self,
        chip: SinsemillaChip,
        mut layouter: impl Layouter<pallas::Base>,
        g_d: &EccPoint,
        pk_d: &EccPoint,
        value: CellValue<pallas::Base>,
        rho: CellValue<pallas::Base>,
        psi: CellValue<pallas::Base>,
    ) -> Result<
        (
            Message<pallas::Affine, SinsemillaChip, { sinsemilla::K }, { sinsemilla::C }>,
            (A, B, C, D, E, F, G, H, I),
        ),
        Error,
    > {
        /*
            The pieces are witnessed in the below configuration, such that no gate has to query an
            offset greater than +/- 1 from its relative row.

            |  A_0  |  A_1  |  A_2  |  A_3  |   A_4   |       Q_1      |       Q_2       |
            ------------------------------------------------------------------------------
            |   a   |  b_0  |  b_1  |  b_2  |   b_3   |                |                 |
            |   b   |   c   |  d_0  |  d_1  |   d_2   | q_decompose_1  |                 |
            |   d   |  e_0  | value |x(pk_d)|  x(g_d) |                |                 |
            |  e_0  |  e_1  |   e   |   f   |   g_0   |                |                 |
            |  g_1  |   g   |   h   |  i_0  |   i_1   |                |  q_decompose_2  |
            |  i_2  |   i   |       |  rho  |   psi   |                |                 |
        */

        let (gd_x, gd_y) = point_repr(g_d.point());
        let (pkd_x, pkd_y) = point_repr(pk_d.point());
        let value_val = value.value();
        let rho_val = rho.value();
        let psi_val = psi.value();

        // `a` = bits 0..=249 of `x(g_d)`
        let a = {
            let a: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (gd_x, 0..250).into();
            chip.witness_message_piece_field(layouter.namespace(|| "a"), a.field_elem_subset(), 25)?
        };

        // b = b_0 || b_1 || b_2 || b_3
        //   = (bits 250..=253 of x(g_d)) || (bit 254 of x(g_d)) || (ỹ bit of g_d) || (bits 0..=3 of pk★_d)
        let b = {
            let b_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (gd_x, 250..254).into();
            let b_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (gd_x, 254..255).into();
            let b_2: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (gd_y, 0..1).into();
            let b_3: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (pkd_x, 0..4).into();

            chip.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness b = b_0 || b_1 || b_2 || b_3"),
                &[b_0, b_1, b_2, b_3],
            )?
        };

        // c = bits 4..=253 of pk★_d
        let c = {
            let c: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (pkd_x, 4..254).into();
            chip.witness_message_piece_field(layouter.namespace(|| "c"), c.field_elem_subset(), 25)?
        };

        // d = d_0 || d_1 || d_2
        //   = (bit 254 of x(pk_d)) || (ỹ bit of pk_d) || (bits 0..=7 of v)
        let d = {
            let d_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (pkd_x, 254..255).into();
            let d_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (pkd_y, 0..1).into();
            let d_2: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (value_val, 0..8).into();

            chip.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness d = d_0 || d_1 || d_2"),
                &[d_0, d_1, d_2],
            )?
        };

        // e = e_0 || e_1 = (bits 8..=63 of v) || (bits 0..=3 of rho)
        let e = {
            let e_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (value_val, 8..64).into();
            let e_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (rho_val, 0..4).into();

            chip.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness e = e_0 || e_1"),
                &[e_0, e_1],
            )?
        };

        // f = bits 4..=253 inclusive of rho
        let f = {
            let f: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (rho_val, 4..254).into();
            chip.witness_message_piece_field(layouter.namespace(|| "f"), f.field_elem_subset(), 25)?
        };

        // g = g_0 || g_1
        //   = (bit 254 of rho) || (bits 0..=8 of psi)
        let g = {
            let g_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (rho_val, 254..255).into();
            let g_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (psi_val, 0..9).into();

            chip.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness g = g_0 || g_1"),
                &[g_0, g_1],
            )?
        };

        // h = (bits 9..=248 of psi)
        let h = {
            let h: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (psi_val, 9..249).into();
            chip.witness_message_piece_field(layouter.namespace(|| "h"), h.field_elem_subset(), 25)?
        };

        // i = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits
        let i = {
            let i_0: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (psi_val, 249..254).into();
            let i_1: MessageSubPiece<pallas::Base, { sinsemilla::K }> = (psi_val, 254..255).into();
            let i_2: MessageSubPiece<pallas::Base, { sinsemilla::K }> =
                (Some(pallas::Base::zero()), 0..4).into();

            chip.witness_message_piece_subpieces(
                layouter.namespace(|| "Witness i = i_0 || i_1 || i_2"),
                &[i_0, i_1, i_2],
            )?
        };

        /*
            The pieces are witnessed in the below configuration, such that no gate has to query an
            offset greater than +/- 1 from its relative row.

            |  A_0  |  A_1  |  A_2  |  A_3  |   A_4   |       Q_1      |       Q_2       |
            ------------------------------------------------------------------------------
            |   a   |  b_0  |  b_1  |  b_2  |   b_3   |                |                 |
            |   b   |   c   |  d_0  |  d_1  |   d_2   | q_decompose_1  |                 |
            |   d   |  e_0  | value |x(pk_d)|  x(g_d) |                |                 |
            |  e_0  |  e_1  |   e   |   f   |   g_0   |                |                 |
            |  g_1  |   g   |   h   |  i_0  |   i_1   |                |  q_decompose_2  |
            |  i_2  |   i   |       |  rho  |   psi   |                |                 |
        */
        let subpieces = layouter.assign_region(
            || "Decomposition for NoteCommit pieces",
            |mut region| {
                // Enable selectors at the correct positions.
                self.q_decompose_1.enable(&mut region, 1)?;
                self.q_decompose_2.enable(&mut region, 4)?;

                // Assign column advices[0].
                let (e_0, g_1, i_2) = {
                    let column = self.advices[0];

                    // Offset 0
                    copy(
                        &mut region,
                        || "copy a",
                        column,
                        0,
                        &a.cell_value(),
                        &self.perm,
                    )?;

                    // Offset 1
                    copy(
                        &mut region,
                        || "copy b",
                        column,
                        1,
                        &b.cell_value(),
                        &self.perm,
                    )?;

                    // Offset 2
                    copy(
                        &mut region,
                        || "copy d",
                        column,
                        2,
                        &d.cell_value(),
                        &self.perm,
                    )?;

                    // Offset 3
                    let e_0 = copy(
                        &mut region,
                        || "copy e_0",
                        column,
                        3,
                        &e.subpieces()[0].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 4
                    let g_1 = copy(
                        &mut region,
                        || "copy g_1",
                        column,
                        4,
                        &g.subpieces()[1].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 5
                    let i_2 = copy(
                        &mut region,
                        || "copy i_2",
                        column,
                        5,
                        &i.subpieces()[2].cell_value(),
                        &self.perm,
                    )?;

                    (e_0, g_1, i_2)
                };

                // Assign column advices[1].
                let (b_0, e_1) = {
                    let column = self.advices[1];
                    // Offset 0
                    let b_0 = copy(
                        &mut region,
                        || "copy b_0",
                        column,
                        0,
                        &b.subpieces()[0].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 1
                    copy(
                        &mut region,
                        || "copy c",
                        column,
                        1,
                        &c.cell_value(),
                        &self.perm,
                    )?;

                    // Offset 2
                    copy(&mut region, || "copy e_0", column, 2, &e_0, &self.perm)?;

                    // Offset 3
                    let e_1 = copy(
                        &mut region,
                        || "copy e_1",
                        column,
                        3,
                        &e.subpieces()[1].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 4
                    copy(
                        &mut region,
                        || "copy g",
                        column,
                        4,
                        &g.cell_value(),
                        &self.perm,
                    )?;

                    // Offset 5
                    copy(
                        &mut region,
                        || "copy i",
                        column,
                        5,
                        &i.cell_value(),
                        &self.perm,
                    )?;

                    (b_0, e_1)
                };

                // Assign column advices[2].
                let (b_1, d_0) = {
                    let column = self.advices[2];
                    // Offset 0
                    let b_1 = copy(
                        &mut region,
                        || "copy b_1",
                        column,
                        0,
                        &b.subpieces()[1].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 1
                    let d_0 = copy(
                        &mut region,
                        || "copy d_0",
                        column,
                        1,
                        &d.subpieces()[0].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 2
                    copy(&mut region, || "copy value", column, 2, &value, &self.perm)?;

                    // Offset 3
                    copy(
                        &mut region,
                        || "copy e",
                        column,
                        3,
                        &e.cell_value(),
                        &self.perm,
                    )?;

                    // Offset 4
                    copy(
                        &mut region,
                        || "copy h",
                        column,
                        4,
                        &h.cell_value(),
                        &self.perm,
                    )?;

                    // Offset 5 is blank for advices[2]

                    (b_1, d_0)
                };

                // Assign column advices[3].
                let (b_2, d_1, i_0) = {
                    let column = self.advices[3];
                    // Offset 0
                    let b_2 = copy(
                        &mut region,
                        || "copy b_2",
                        column,
                        0,
                        &b.subpieces()[2].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 1
                    let d_1 = copy(
                        &mut region,
                        || "copy d_1",
                        column,
                        1,
                        &d.subpieces()[1].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 2
                    copy(
                        &mut region,
                        || "copy pkd_x",
                        column,
                        2,
                        &pk_d.x(),
                        &self.perm,
                    )?;

                    // Offset 3
                    copy(
                        &mut region,
                        || "copy f",
                        column,
                        3,
                        &f.cell_value(),
                        &self.perm,
                    )?;

                    // Offset 4
                    let i_0 = copy(
                        &mut region,
                        || "copy i_0",
                        column,
                        4,
                        &i.subpieces()[0].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 5
                    copy(&mut region, || "copy rho", column, 5, &rho, &self.perm)?;

                    (b_2, d_1, i_0)
                };

                // Assign column advices[4].
                let (b_3, d_2, g_0, i_1) = {
                    let column = self.advices[4];
                    // Offset 0
                    let b_3 = copy(
                        &mut region,
                        || "copy b_3",
                        column,
                        0,
                        &b.subpieces()[3].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 1
                    let d_2 = copy(
                        &mut region,
                        || "copy d_2",
                        column,
                        1,
                        &d.subpieces()[2].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 2
                    copy(&mut region, || "copy gd_x", column, 2, &g_d.x(), &self.perm)?;

                    // Offset 3
                    let g_0 = copy(
                        &mut region,
                        || "copy g_0",
                        column,
                        3,
                        &g.subpieces()[0].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 4
                    let i_1 = copy(
                        &mut region,
                        || "copy i_1",
                        column,
                        4,
                        &i.subpieces()[1].cell_value(),
                        &self.perm,
                    )?;

                    // Offset 5
                    copy(&mut region, || "copy psi", column, 5, &psi, &self.perm)?;

                    (b_3, d_2, g_0, i_1)
                };

                Ok((
                    A(a.cell_value()),
                    B(b_0, b_1, b_2, b_3),
                    C(c.cell_value()),
                    D(d_0, d_1, d_2),
                    E(e_0, e_1),
                    F(f.cell_value()),
                    G(g_0, g_1),
                    H(h.cell_value()),
                    I(i_0, i_1, i_2),
                ))
            },
        )?;

        Ok((
            Message::from_pieces(chip, vec![a, b, c, d, e, f, g, h, i]),
            subpieces,
        ))
    }

    #[allow(clippy::many_single_char_names)]
    pub(in crate::circuit) fn check_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        subpieces: (A, B, C, D, E, F, G, H, I),
        zs: Vec<Vec<CellValue<pallas::Base>>>,
    ) -> Result<(), Error> {
        let (a, b, c, d, e, f, g, h, i) = subpieces;

        self.gd_x_canonicity(
            layouter.namespace(|| "x(g_d) canonicity"),
            a.0,
            b.0,
            b.1,
            zs[0][13],
        )?;
        self.pkd_x_canonicity(
            layouter.namespace(|| "x(pk_d) canonicity"),
            b.3,
            c.0,
            d.0,
            zs[2][13],
        )?;
        self.rho_canonicity(
            layouter.namespace(|| "rho canonicity"),
            e.1,
            f.0,
            g.0,
            zs[5][13],
        )?;
        self.psi_canonicity(
            layouter.namespace(|| "psi canonicity"),
            g.1,
            h.0,
            i.0,
            i.1,
            zs[7][13],
        )?;

        Ok(())
    }

    // Check canonicity of `x(g_d)` encoding
    fn gd_x_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        a: CellValue<pallas::Base>,
        b_0: CellValue<pallas::Base>,
        b_1: CellValue<pallas::Base>,
        z_13a: CellValue<pallas::Base>,
    ) -> Result<(), Error> {
        // `x(g_d)` = `a (250 bits) || b_0 (4 bits) || b_1 (1 bit)`
        // - b_1 = 1 => b_0 = 0
        // - b_1 = 1 => a < t_P
        //     - 0 ≤ a < 2^130 (z_13 of SinsemillaHash(a))
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
                    self.advices[0],
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
            let zs = self.gd_lookup_config.lookup_range_check(
                layouter.namespace(|| "Decompose low 130 bits of (a + 2^130 - t_P)"),
                a_prime,
                13,
            )?;
            assert_eq!(zs.len(), 14); // [z_0, z_1, ..., z_13]
            zs[13]
        };

        layouter.assign_region(
            || "x(g_d) canonicity",
            |mut region| {
                self.q_gd_canon.enable(&mut region, 0)?;

                // Enforce b_1 = 1 => b_0 = 0 by copying b_0, b_1 to the correct
                // offsets for use in the gate.
                copy(
                    &mut region,
                    || "copy b_0",
                    self.advices[0],
                    0,
                    &b_0,
                    &self.perm,
                )?;
                copy(
                    &mut region,
                    || "copy b_1",
                    self.advices[0],
                    1,
                    &b_1,
                    &self.perm,
                )?;

                // Enforce b_1 = 0 => z_13 of SinsemillaHash(a) == 0 by copying
                // z_13 to the correct offset for use in the gate.
                copy(
                    &mut region,
                    || "copy z_13 from message piece a",
                    self.advices[1],
                    0,
                    &z_13a,
                    &self.perm,
                )?;

                // Check that a_prime = a + 2^130 - t_P was correctly derived
                // from a.
                copy(&mut region, || "copy a", self.advices[2], 0, &a, &self.perm)?;
                copy(
                    &mut region,
                    || "copy a_prime",
                    self.advices[3],
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
                    self.advices[4],
                    0,
                    &a_prime_decomposition,
                    &self.perm,
                )?;
                Ok(())
            },
        )
    }

    // Check canonicity of `x(pk_d)` encoding
    fn pkd_x_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        b_3: CellValue<pallas::Base>,
        c: CellValue<pallas::Base>,
        d_0: CellValue<pallas::Base>,
        z_13c: CellValue<pallas::Base>,
    ) -> Result<(), Error> {
        // `x(pk_d)` = `b_3 (4 bits) || c (250 bits) || d_0 (1 bit)`
        // - d_0 = 1 => b_3 + 2^4 c < t_P
        //     - 0 ≤ b_3 + 2^4 c < 2^134
        //         - b_3 is part of the Sinsemilla message piece
        //           b = b_0 (4 bits) || b_1 (1 bit) || b_2 (1 bit) || b_3 (4 bits)
        //         - b_3 is internally constrained by the Sinsemilla hash to be 4 bits.
        //         - z_13 of SinsemillaHash(c) == 0 constrains bits 4..=133 to 130 bits
        //     - 0 ≤ b_3 + 2^4 c + 2^140 - t_P < 2^140 (14 ten-bit lookups)

        // Witness lookup_element = b_3 + 2^4 c + 2^140 - t_P.
        // We will copy this value into a gate to check that it has been
        // correctly derived from b_3, c.
        let lookup_element = layouter.assign_region(
            || "b_3 + 2^4 c + 2^140 - t_P",
            |mut region| {
                let lookup_element = b_3.value().zip(c.value()).map(|(b_3, c)| {
                    let two_pow_4 = pallas::Base::from_u64(1u64 << 4);
                    let two_pow_140 = pallas::Base::from_u128(1u128 << 70).square();
                    let t_p = pallas::Base::from_u128(T_P);
                    b_3 + (two_pow_4 * c) + two_pow_140 - t_p
                });
                let cell = region.assign_advice(
                    || "b_3 + 2^4 c + 2^140 - t_P",
                    self.advices[0],
                    0,
                    || lookup_element.ok_or(Error::SynthesisError),
                )?;
                Ok(CellValue::new(cell, lookup_element))
            },
        )?;

        // Decompose the low 140 bits of lookup_element = b_3 + 2^4 c + 2^140 - t_P,
        // and output the running sum at the end of it.
        // If lookup_element < 2^140, the running sum will be 0.
        let lookup_decomposition = {
            let zs = self.gd_lookup_config.lookup_range_check(
                layouter.namespace(|| "Decompose low 140 bits of (b_3 + 2^4 c + 2^140 - t_P)"),
                lookup_element,
                14,
            )?;
            assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_13, z_14]
            zs[14]
        };

        layouter.assign_region(
            || "x(pk_d) canonicity",
            |mut region| {
                self.q_pkd_canon.enable(&mut region, 0)?;

                // Copy d_0. The constraints here are enforced if and only if
                // d_0 = 1.
                copy(
                    &mut region,
                    || "copy d_0",
                    self.advices[0],
                    0,
                    &d_0,
                    &self.perm,
                )?;

                // Enforce d_0 = 1 => (0 ≤ lookup_element < 2^130).
                // This is equivalent to enforcing that the running sum returned by the
                // 140-bit decomposition of lookup_element is 0.
                copy(
                    &mut region,
                    || "copy lookup_decomposition",
                    self.advices[0],
                    1,
                    &lookup_decomposition,
                    &self.perm,
                )?;

                // Enforce d_0 = 1 => z_13 of SinsemillaHash(c) == 0 by copying
                // z_13c to the correct offset for use in the gate.
                copy(
                    &mut region,
                    || "copy z_13 from message piece c",
                    self.advices[1],
                    0,
                    &z_13c,
                    &self.perm,
                )?;

                // Check `lookup_element` = b_3 + 2^4 c + 2^140 - t_P was correctly
                // derived from b_3, c.
                copy(
                    &mut region,
                    || "copy b_3",
                    self.advices[2],
                    0,
                    &b_3,
                    &self.perm,
                )?;
                copy(&mut region, || "copy c", self.advices[3], 0, &c, &self.perm)?;
                copy(
                    &mut region,
                    || "copy lookup_element",
                    self.advices[4],
                    0,
                    &lookup_element,
                    &self.perm,
                )?;

                Ok(())
            },
        )
    }

    // Check canonicity of `rho` encoding
    fn rho_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        e_1: CellValue<pallas::Base>,
        f: CellValue<pallas::Base>,
        g_0: CellValue<pallas::Base>,
        z_13f: CellValue<pallas::Base>,
    ) -> Result<(), Error> {
        // `rho` = `e_1 (4 bits) || f (250 bits) || g_0 (1 bit)`
        // - g_0 = 1 => e_1 + 2^4 f < t_P
        // - 0 ≤ e_1 + 2^4 f < 2^134
        //     - e_1 is part of the Sinsemilla message piece
        //       e = e_0 (56 bits) || e_1 (4 bits)
        //     - e_1 is internally constrained by the Sinsemilla hash to be 4 bits.
        //     - z_13 of SinsemillaHash(f) constrains bits 4..=133 to 130 bits
        // - 0 ≤ e_1 + 2^4 f + 2^140 - t_P < 2^140 (14 ten-bit lookups)

        // Witness lookup_element = e_1 + 2^4 f + 2^140 - t_P.
        // We will copy this value into a gate to check that it has been
        // correctly derived from e_1, f.
        let lookup_element = layouter.assign_region(
            || "e_1 + 2^4 f + 2^140 - t_P",
            |mut region| {
                let lookup_element = e_1.value().zip(f.value()).map(|(e_1, f)| {
                    let two_pow_4 = pallas::Base::from_u64(1u64 << 4);
                    let two_pow_140 = pallas::Base::from_u128(1u128 << 70).square();
                    let t_p = pallas::Base::from_u128(T_P);
                    e_1 + (two_pow_4 * f) + two_pow_140 - t_p
                });
                let cell = region.assign_advice(
                    || "e_1 + 2^4 f + 2^140 - t_P",
                    self.advices[0],
                    0,
                    || lookup_element.ok_or(Error::SynthesisError),
                )?;
                Ok(CellValue::new(cell, lookup_element))
            },
        )?;

        // Decompose the low 140 bits of lookup_element = e_1 + 2^4 f + 2^140 - t_P,
        // and output the running sum at the end of it.
        // If lookup_element < 2^140, the running sum will be 0.
        let lookup_decomposition = {
            let zs = self.rho_lookup_config.lookup_range_check(
                layouter.namespace(|| "Decompose low 140 bits of (e_1 + 2^4 f + 2^140 - t_P)"),
                lookup_element,
                14,
            )?;
            assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_13, z_14]
            zs[14]
        };

        layouter.assign_region(
            || "rho canonicity",
            |mut region| {
                self.q_rho_canon.enable(&mut region, 0)?;

                // Copy g_0. The constraints here are enforced if and only if
                // g_0 = 1.
                copy(
                    &mut region,
                    || "copy g_0",
                    self.advices[0],
                    0,
                    &g_0,
                    &self.perm,
                )?;

                // Enforce g_0 = 1 => (0 ≤ lookup_element < 2^130).
                // This is equivalent to enforcing that the running sum returned by the
                // 140-bit decomposition of lookup_element is 0.
                copy(
                    &mut region,
                    || "copy lookup_decomposition",
                    self.advices[1],
                    0,
                    &lookup_decomposition,
                    &self.perm,
                )?;

                // Enforce g_0 = 1 => z_13 of SinsemillaHash(f) == 0 by copying
                // z_13f to the correct offset for use in the gate.
                copy(
                    &mut region,
                    || "copy z_13 from message piece f",
                    self.advices[1],
                    1,
                    &z_13f,
                    &self.perm,
                )?;

                // Check `lookup_element` = e_1 + 2^4 f + 2^140 - t_P was correctly
                // derived from e_1, f.
                copy(
                    &mut region,
                    || "copy e_1",
                    self.advices[2],
                    0,
                    &e_1,
                    &self.perm,
                )?;
                copy(&mut region, || "copy f", self.advices[3], 0, &f, &self.perm)?;
                copy(
                    &mut region,
                    || "copy lookup_element",
                    self.advices[4],
                    0,
                    &lookup_element,
                    &self.perm,
                )?;

                Ok(())
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    // Check canonicity of `psi` encoding
    fn psi_canonicity(
        &self,
        mut layouter: impl Layouter<pallas::Base>,
        g_1: CellValue<pallas::Base>,
        h: CellValue<pallas::Base>,
        i_0: CellValue<pallas::Base>,
        i_1: CellValue<pallas::Base>,
        z_13h: CellValue<pallas::Base>,
    ) -> Result<(), Error> {
        // `psi` = `g_1 (9 bits) || h (240 bits) || i_0 (5 bits) || i_1 (1 bit)`
        // - i_1 = 1 => (i_0 = 0) ∧ (g_1 + 2^9 h < t_P)
        // - 0 ≤ g_1 + 2^9 h < 2^139
        //     - g_1 is part of the Sinsemilla message piece
        //       g = g_0 (1 bit) || g_1 (9 bits)
        //     - g_1 is internally constrained by the SinsemillaHash to be 9 bits.
        //     - z_13 of SinsemillaHash(h) constrains bits 9..=138 to 130 bits
        // - 0 ≤ g_1 + 2^9 h + 2^140 - t_P < 2^140 (14 ten-bit lookups)

        // Witness lookup_element = g_1 + 2^9 h + 2^140 - t_P.
        // We will copy this value into a gate to check that it has been
        // correctly derived from g_1, f.
        let lookup_element = layouter.assign_region(
            || "g_1 + 2^9 h + 2^140 - t_P",
            |mut region| {
                let lookup_element = g_1.value().zip(h.value()).map(|(g_1, h)| {
                    let two_pow_9 = pallas::Base::from_u64(1u64 << 9);
                    let two_pow_140 = pallas::Base::from_u128(1u128 << 70).square();
                    let t_p = pallas::Base::from_u128(T_P);
                    g_1 + (two_pow_9 * h) + two_pow_140 - t_p
                });
                let cell = region.assign_advice(
                    || "g_1 + 2^9 h + 2^140 - t_P",
                    self.advices[0],
                    0,
                    || lookup_element.ok_or(Error::SynthesisError),
                )?;
                Ok(CellValue::new(cell, lookup_element))
            },
        )?;

        // Decompose the low 140 bits of lookup_element = g_1 + 2^9 h + 2^140 - t_P,
        // and output the running sum at the end of it.
        // If lookup_element < 2^140, the running sum will be 0.
        let lookup_decomposition = {
            let zs = self.rho_lookup_config.lookup_range_check(
                layouter.namespace(|| "Decompose low 140 bits of (g_1 + 2^9 h + 2^140 - t_P)"),
                lookup_element,
                14,
            )?;
            assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_13, z_14]
            zs[14]
        };

        layouter.assign_region(
            || "psi canonicity",
            |mut region| {
                self.q_psi_canon.enable(&mut region, 0)?;

                // Copy i_1. The constraints here are enforced if and only if
                // i_1 = 1.
                copy(
                    &mut region,
                    || "copy i_1",
                    self.advices[0],
                    0,
                    &i_1,
                    &self.perm,
                )?;

                // Enforce h_1 = 1 => (0 ≤ lookup_element < 2^130).
                // This is equivalent to enforcing that the running sum returned by the
                // 140-bit decomposition of lookup_element is 0.
                copy(
                    &mut region,
                    || "copy lookup_decomposition",
                    self.advices[0],
                    1,
                    &lookup_decomposition,
                    &self.perm,
                )?;

                // Enforce h_1 = 1 => z_13 of SinsemillaHash(h) == 0 by copying
                // z_13h to the correct offset for use in the gate.
                copy(
                    &mut region,
                    || "copy z_13 from message piece h",
                    self.advices[1],
                    0,
                    &z_13h,
                    &self.perm,
                )?;

                // Check `lookup_element` = g_1 + 2^9 h + 2^140 - t_P was correctly
                // derived from g_1, h.
                copy(
                    &mut region,
                    || "copy g_1",
                    self.advices[2],
                    0,
                    &g_1,
                    &self.perm,
                )?;
                copy(&mut region, || "copy h", self.advices[3], 0, &h, &self.perm)?;
                copy(
                    &mut region,
                    || "copy lookup_element",
                    self.advices[4],
                    0,
                    &lookup_element,
                    &self.perm,
                )?;

                // Enforce i_1 = 1 => i_0 = 0
                copy(
                    &mut region,
                    || "copy i_0",
                    self.advices[4],
                    1,
                    &i_0,
                    &self.perm,
                )?;

                Ok(())
            },
        )
    }
}

fn point_repr(point: Option<pallas::Affine>) -> (Option<pallas::Base>, Option<pallas::Base>) {
    let x: Option<pallas::Base> = point.map(|point| *point.coordinates().unwrap().x());
    let y: Option<pallas::Base> = point.map(|point| {
        let last_byte: u8 = point.to_bytes().as_ref()[31];
        let last_bit = (last_byte >> 7) % 2;
        pallas::Base::from_u64(last_bit as u64)
    });

    (x, y)
}
