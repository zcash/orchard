//! Note commitment logic for the Orchard circuit.

use crate::{
    constants::{OrchardCommitDomains, OrchardFixedBases, OrchardHashDomains, T_P},
    value::NoteValue,
};
use group::ff::PrimeField;
use halo2_gadgets::{
    ecc::{
        chip::{EccChip, NonIdentityEccPoint},
        {NonIdentityPoint, Point, ScalarFixed},
    },
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        CommitDomain, Message, MessagePiece,
    },
    utilities::{
        bool_check, cond_swap::CondSwapChip, lookup_range_check::PallasLookupRangeCheck,
        FieldValue, RangeConstrained,
    },
};
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Constraints, Error, Expression, Selector},
    poly::Rotation,
};
use pasta_curves::pallas;
use std::iter;

type NoteCommitPiece<Lookup> = MessagePiece<
    pallas::Affine,
    SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
    10,
    253,
>;

/// The values of the running sum at the start and end of the range being used for a
/// canonicity check.
type CanonicityBounds = (
    AssignedCell<pallas::Base, pallas::Base>,
    AssignedCell<pallas::Base, pallas::Base>,
);

/*
    <https://zips.z.cash/protocol/nu5.pdf#concretesinsemillacommit>
    We need to hash g★_d || pk★_d || i2lebsp_{64}(v) || rho || psi,
    where
        - g★_d is the representation of the point g_d, with 255 bits used for the
          x-coordinate and 1 bit used for the y-coordinate;
        - pk★_d is the representation of the point pk_d, with 255 bits used for the
          x-coordinate and 1 bit used for the y-coordinate;
        - v is a 64-bit value;
        - rho is a base field element (255 bits); and
        - psi is a base field element (255 bits).
*/

/// b = b_0 || b_1 || b_2 || b_3
///   = (bits 250..=253 of x(g_d)) || (bit 254 of x(g_d)) || (ỹ bit of g_d) || (bits 0..=3 of pk★_d)
///
/// | A_6 | A_7 | A_8 | q_notecommit_b |
/// ------------------------------------
/// |  b  | b_0 | b_1 |       1        |
/// |     | b_2 | b_3 |       0        |
///
/// <https://p.z.cash/orchard-0.1:note-commit-decomposition-b?partial>
#[derive(Clone, Debug)]
struct DecomposeB<Lookup: PallasLookupRangeCheck> {
    q_notecommit_b: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> DecomposeB<Lookup> {
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        two_pow_4: pallas::Base,
        two_pow_5: pallas::Base,
        two_pow_6: pallas::Base,
    ) -> Self {
        let q_notecommit_b = meta.selector();

        meta.create_gate("NoteCommit MessagePiece b", |meta| {
            let q_notecommit_b = meta.query_selector(q_notecommit_b);

            // b has been constrained to 10 bits by the Sinsemilla hash.
            let b = meta.query_advice(col_l, Rotation::cur());
            // b_0 has been constrained to be 4 bits outside this gate.
            let b_0 = meta.query_advice(col_m, Rotation::cur());
            // This gate constrains b_1 to be boolean.
            let b_1 = meta.query_advice(col_r, Rotation::cur());
            // This gate constrains b_2 to be boolean.
            let b_2 = meta.query_advice(col_m, Rotation::next());
            // b_3 has been constrained to 4 bits outside this gate.
            let b_3 = meta.query_advice(col_r, Rotation::next());

            // b = b_0 + (2^4) b_1 + (2^5) b_2 + (2^6) b_3
            let decomposition_check =
                b - (b_0 + b_1.clone() * two_pow_4 + b_2.clone() * two_pow_5 + b_3 * two_pow_6);

            Constraints::with_selector(
                q_notecommit_b,
                [
                    ("bool_check b_1", bool_check(b_1)),
                    ("bool_check b_2", bool_check(b_2)),
                    ("decomposition", decomposition_check),
                ],
            )
        });

        Self {
            q_notecommit_b,
            col_l,
            col_m,
            col_r,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::type_complexity)]
    fn decompose(
        lookup_config: &Lookup,
        chip: SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
        layouter: &mut impl Layouter<pallas::Base>,
        g_d: &NonIdentityEccPoint,
        pk_d: &NonIdentityEccPoint,
    ) -> Result<
        (
            NoteCommitPiece<Lookup>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        ),
        Error,
    > {
        let (gd_x, gd_y) = (g_d.x(), g_d.y());

        // Constrain b_0 to be 4 bits
        let b_0 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "b_0"),
            gd_x.value(),
            250..254,
        )?;

        // b_1, b_2 will be boolean-constrained in the gate.
        let b_1 = RangeConstrained::bitrange_of(gd_x.value(), 254..255);
        let b_2 = RangeConstrained::bitrange_of(gd_y.value(), 0..1);

        // Constrain b_3 to be 4 bits
        let b_3 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "b_3"),
            pk_d.x().value(),
            0..4,
        )?;

        let b = MessagePiece::from_subpieces(
            chip,
            layouter.namespace(|| "b"),
            [b_0.value(), b_1, b_2, b_3.value()],
        )?;

        Ok((b, b_0, b_1, b_2, b_3))
    }

    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        b: NoteCommitPiece<Lookup>,
        b_0: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        b_1: RangeConstrained<pallas::Base, Value<pallas::Base>>,
        b_2: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        b_3: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
    ) -> Result<AssignedCell<pallas::Base, pallas::Base>, Error> {
        layouter.assign_region(
            || "NoteCommit MessagePiece b",
            |mut region| {
                self.q_notecommit_b.enable(&mut region, 0)?;

                b.inner()
                    .cell_value()
                    .copy_advice(|| "b", &mut region, self.col_l, 0)?;
                b_0.inner()
                    .copy_advice(|| "b_0", &mut region, self.col_m, 0)?;
                let b_1 = region.assign_advice(|| "b_1", self.col_r, 0, || *b_1.inner())?;

                b_2.inner()
                    .copy_advice(|| "b_2", &mut region, self.col_m, 1)?;
                b_3.inner()
                    .copy_advice(|| "b_3", &mut region, self.col_r, 1)?;

                Ok(b_1)
            },
        )
    }
}

/// d = d_0 || d_1 || d_2 || d_3
///   = (bit 254 of x(pk_d)) || (ỹ bit of pk_d) || (bits 0..=7 of v) || (bits 8..=57 of v)
///
/// | A_6 | A_7 | A_8 | q_notecommit_d |
/// ------------------------------------
/// |  d  | d_0 | d_1 |       1        |
/// |     | d_2 | d_3 |       0        |
///
/// <https://p.z.cash/orchard-0.1:note-commit-decomposition-d?partial>
#[derive(Clone, Debug)]
struct DecomposeD<Lookup: PallasLookupRangeCheck> {
    q_notecommit_d: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> DecomposeD<Lookup> {
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        two: pallas::Base,
        two_pow_2: pallas::Base,
        two_pow_10: pallas::Base,
    ) -> Self {
        let q_notecommit_d = meta.selector();

        meta.create_gate("NoteCommit MessagePiece d", |meta| {
            let q_notecommit_d = meta.query_selector(q_notecommit_d);

            // d has been constrained to 60 bits by the Sinsemilla hash.
            let d = meta.query_advice(col_l, Rotation::cur());
            // This gate constrains d_0 to be boolean.
            let d_0 = meta.query_advice(col_m, Rotation::cur());
            // This gate constrains d_1 to be boolean.
            let d_1 = meta.query_advice(col_r, Rotation::cur());
            // d_2 has been constrained to 8 bits outside this gate.
            let d_2 = meta.query_advice(col_m, Rotation::next());
            // d_3 is set to z1_d.
            let d_3 = meta.query_advice(col_r, Rotation::next());

            // d = d_0 + (2) d_1 + (2^2) d_2 + (2^10) d_3
            let decomposition_check =
                d - (d_0.clone() + d_1.clone() * two + d_2 * two_pow_2 + d_3 * two_pow_10);

            Constraints::with_selector(
                q_notecommit_d,
                [
                    ("bool_check d_0", bool_check(d_0)),
                    ("bool_check d_1", bool_check(d_1)),
                    ("decomposition", decomposition_check),
                ],
            )
        });

        Self {
            q_notecommit_d,
            col_l,
            col_m,
            col_r,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::type_complexity)]
    fn decompose(
        lookup_config: &Lookup,
        chip: SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
        layouter: &mut impl Layouter<pallas::Base>,
        pk_d: &NonIdentityEccPoint,
        value: &AssignedCell<NoteValue, pallas::Base>,
    ) -> Result<
        (
            NoteCommitPiece<Lookup>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        ),
        Error,
    > {
        let value_val = value.value().map(|v| pallas::Base::from(v.inner()));

        // d_0, d_1 will be boolean-constrained in the gate.
        let d_0 = RangeConstrained::bitrange_of(pk_d.x().value(), 254..255);
        let d_1 = RangeConstrained::bitrange_of(pk_d.y().value(), 0..1);

        // Constrain d_2 to be 8 bits
        let d_2 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "d_2"),
            value_val.as_ref(),
            0..8,
        )?;

        // d_3 = z1_d from the SinsemillaHash(d) running sum output.
        let d_3 = RangeConstrained::bitrange_of(value_val.as_ref(), 8..58);

        let d = MessagePiece::from_subpieces(
            chip,
            layouter.namespace(|| "d"),
            [d_0, d_1, d_2.value(), d_3],
        )?;

        Ok((d, d_0, d_1, d_2))
    }

    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        d: NoteCommitPiece<Lookup>,
        d_0: RangeConstrained<pallas::Base, Value<pallas::Base>>,
        d_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        d_2: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        z1_d: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<AssignedCell<pallas::Base, pallas::Base>, Error> {
        layouter.assign_region(
            || "NoteCommit MessagePiece d",
            |mut region| {
                self.q_notecommit_d.enable(&mut region, 0)?;

                d.inner()
                    .cell_value()
                    .copy_advice(|| "d", &mut region, self.col_l, 0)?;
                let d_0 = region.assign_advice(|| "d_0", self.col_m, 0, || *d_0.inner())?;
                d_1.inner()
                    .copy_advice(|| "d_1", &mut region, self.col_r, 0)?;

                d_2.inner()
                    .copy_advice(|| "d_2", &mut region, self.col_m, 1)?;
                z1_d.copy_advice(|| "d_3 = z1_d", &mut region, self.col_r, 1)?;

                Ok(d_0)
            },
        )
    }
}

/// e = e_0 || e_1 = (bits 58..=63 of v) || (bits 0..=3 of rho)
///
/// | A_6 | A_7 | A_8 | q_notecommit_e |
/// ------------------------------------
/// |  e  | e_0 | e_1 |       1        |
///
/// <https://p.z.cash/orchard-0.1:note-commit-decomposition-e?partial>
#[derive(Clone, Debug)]
struct DecomposeE<Lookup: PallasLookupRangeCheck> {
    q_notecommit_e: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> DecomposeE<Lookup> {
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        two_pow_6: pallas::Base,
    ) -> Self {
        let q_notecommit_e = meta.selector();

        meta.create_gate("NoteCommit MessagePiece e", |meta| {
            let q_notecommit_e = meta.query_selector(q_notecommit_e);

            // e has been constrained to 10 bits by the Sinsemilla hash.
            let e = meta.query_advice(col_l, Rotation::cur());
            // e_0 has been constrained to 6 bits outside this gate.
            let e_0 = meta.query_advice(col_m, Rotation::cur());
            // e_1 has been constrained to 4 bits outside this gate.
            let e_1 = meta.query_advice(col_r, Rotation::cur());

            // e = e_0 + (2^6) e_1
            let decomposition_check = e - (e_0 + e_1 * two_pow_6);

            Constraints::with_selector(q_notecommit_e, Some(("decomposition", decomposition_check)))
        });

        Self {
            q_notecommit_e,
            col_l,
            col_m,
            col_r,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::type_complexity)]
    fn decompose(
        lookup_config: &Lookup,
        chip: SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
        layouter: &mut impl Layouter<pallas::Base>,
        value: &AssignedCell<NoteValue, pallas::Base>,
        rho: &AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<
        (
            NoteCommitPiece<Lookup>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        ),
        Error,
    > {
        let value_val = value.value().map(|v| pallas::Base::from(v.inner()));

        // Constrain e_0 to be 6 bits.
        let e_0 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "e_0"),
            value_val.as_ref(),
            58..64,
        )?;

        // Constrain e_1 to be 4 bits.
        let e_1 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "e_1"),
            rho.value(),
            0..4,
        )?;

        let e = MessagePiece::from_subpieces(
            chip,
            layouter.namespace(|| "e"),
            [e_0.value(), e_1.value()],
        )?;

        Ok((e, e_0, e_1))
    }

    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        e: NoteCommitPiece<Lookup>,
        e_0: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        e_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "NoteCommit MessagePiece e",
            |mut region| {
                self.q_notecommit_e.enable(&mut region, 0)?;

                e.inner()
                    .cell_value()
                    .copy_advice(|| "e", &mut region, self.col_l, 0)?;
                e_0.inner()
                    .copy_advice(|| "e_0", &mut region, self.col_m, 0)?;
                e_1.inner()
                    .copy_advice(|| "e_1", &mut region, self.col_r, 0)?;

                Ok(())
            },
        )
    }
}

/// g = g_0 || g_1 || g_2
///   = (bit 254 of rho) || (bits 0..=8 of psi) || (bits 9..=248 of psi)
///
/// | A_6 | A_7 | q_notecommit_g |
/// ------------------------------
/// |  g  | g_0 |       1        |
/// | g_1 | g_2 |       0        |
///
/// <https://p.z.cash/orchard-0.1:note-commit-decomposition-g?partial>
#[derive(Clone, Debug)]
struct DecomposeG<Lookup: PallasLookupRangeCheck> {
    q_notecommit_g: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> DecomposeG<Lookup> {
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        two: pallas::Base,
        two_pow_10: pallas::Base,
    ) -> Self {
        let q_notecommit_g = meta.selector();

        meta.create_gate("NoteCommit MessagePiece g", |meta| {
            let q_notecommit_g = meta.query_selector(q_notecommit_g);

            // g has been constrained to 250 bits by the Sinsemilla hash.
            let g = meta.query_advice(col_l, Rotation::cur());
            // This gate constrains g_0 to be boolean.
            let g_0 = meta.query_advice(col_m, Rotation::cur());
            // g_1 has been constrained to 9 bits outside this gate.
            let g_1 = meta.query_advice(col_l, Rotation::next());
            // g_2 is set to z1_g.
            let g_2 = meta.query_advice(col_m, Rotation::next());

            // g = g_0 + (2) g_1 + (2^10) g_2
            let decomposition_check = g - (g_0.clone() + g_1 * two + g_2 * two_pow_10);

            Constraints::with_selector(
                q_notecommit_g,
                [
                    ("bool_check g_0", bool_check(g_0)),
                    ("decomposition", decomposition_check),
                ],
            )
        });

        Self {
            q_notecommit_g,
            col_l,
            col_m,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::type_complexity)]
    fn decompose(
        lookup_config: &Lookup,
        chip: SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
        layouter: &mut impl Layouter<pallas::Base>,
        rho: &AssignedCell<pallas::Base, pallas::Base>,
        psi: &AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<
        (
            NoteCommitPiece<Lookup>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        ),
        Error,
    > {
        // g_0 will be boolean-constrained in the gate.
        let g_0 = RangeConstrained::bitrange_of(rho.value(), 254..255);

        // Constrain g_1 to be 9 bits.
        let g_1 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "g_1"),
            psi.value(),
            0..9,
        )?;

        // g_2 = z1_g from the SinsemillaHash(g) running sum output.
        let g_2 = RangeConstrained::bitrange_of(psi.value(), 9..249);

        let g = MessagePiece::from_subpieces(
            chip,
            layouter.namespace(|| "g"),
            [g_0, g_1.value(), g_2],
        )?;

        Ok((g, g_0, g_1))
    }

    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        g: NoteCommitPiece<Lookup>,
        g_0: RangeConstrained<pallas::Base, Value<pallas::Base>>,
        g_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        z1_g: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<AssignedCell<pallas::Base, pallas::Base>, Error> {
        layouter.assign_region(
            || "NoteCommit MessagePiece g",
            |mut region| {
                self.q_notecommit_g.enable(&mut region, 0)?;

                g.inner()
                    .cell_value()
                    .copy_advice(|| "g", &mut region, self.col_l, 0)?;
                let g_0 = region.assign_advice(|| "g_0", self.col_m, 0, || *g_0.inner())?;

                g_1.inner()
                    .copy_advice(|| "g_1", &mut region, self.col_l, 1)?;
                z1_g.copy_advice(|| "g_2 = z1_g", &mut region, self.col_m, 1)?;

                Ok(g_0)
            },
        )
    }
}

/// For Vanilla Orchard circuit,
/// h = h_0 || h_1 || h_2
///   = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits
///
/// | A_6 | A_7 | A_8 | q_notecommit_h |
/// ------------------------------------
/// |  h  | h_0 | h_1 |       1        |
///
/// <https://p.z.cash/orchard-0.1:note-commit-decomposition-h?partial>
#[derive(Clone, Debug)]
struct DecomposeHVanilla<Lookup: PallasLookupRangeCheck> {
    q_notecommit_h: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> DecomposeHVanilla<Lookup> {
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        two_pow_5: pallas::Base,
    ) -> Self {
        let q_notecommit_h = meta.selector();

        meta.create_gate("NoteCommit MessagePiece h", |meta| {
            let q_notecommit_h = meta.query_selector(q_notecommit_h);

            // h has been constrained to 10 bits by the Sinsemilla hash.
            let h = meta.query_advice(col_l, Rotation::cur());
            // h_0 has been constrained to be 5 bits outside this gate.
            let h_0 = meta.query_advice(col_m, Rotation::cur());
            // This gate constrains h_1 to be boolean.
            let h_1 = meta.query_advice(col_r, Rotation::cur());

            // h = h_0 + (2^5) h_1
            let decomposition_check = h - (h_0 + h_1.clone() * two_pow_5);

            Constraints::with_selector(
                q_notecommit_h,
                [
                    ("bool_check h_1", bool_check(h_1)),
                    ("decomposition", decomposition_check),
                ],
            )
        });

        Self {
            q_notecommit_h,
            col_l,
            col_m,
            col_r,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::type_complexity)]
    fn decompose(
        lookup_config: &Lookup,
        chip: SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
        layouter: &mut impl Layouter<pallas::Base>,
        psi: &AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<
        (
            NoteCommitPiece<Lookup>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
        ),
        Error,
    > {
        // Constrain h_0 to be 5 bits.
        let h_0 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "h_0"),
            psi.value(),
            249..254,
        )?;

        // h_1 will be boolean-constrained in the gate.
        let h_1 = RangeConstrained::bitrange_of(psi.value(), 254..255);

        let h = MessagePiece::from_subpieces(
            chip,
            layouter.namespace(|| "h"),
            [
                h_0.value(),
                h_1,
                RangeConstrained::bitrange_of(Value::known(&pallas::Base::zero()), 0..4),
            ],
        )?;

        Ok((h, h_0, h_1))
    }

    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        h: NoteCommitPiece<Lookup>,
        h_0: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        h_1: RangeConstrained<pallas::Base, Value<pallas::Base>>,
    ) -> Result<AssignedCell<pallas::Base, pallas::Base>, Error> {
        layouter.assign_region(
            || "NoteCommit MessagePiece h",
            |mut region| {
                self.q_notecommit_h.enable(&mut region, 0)?;

                h.inner()
                    .cell_value()
                    .copy_advice(|| "h", &mut region, self.col_l, 0)?;
                h_0.inner()
                    .copy_advice(|| "h_0", &mut region, self.col_m, 0)?;
                let h_1 = region.assign_advice(|| "h_1", self.col_r, 0, || *h_1.inner())?;

                Ok(h_1)
            },
        )
    }
}

/// For ZSA Orchard circuit,
/// h_zec = h_0 || h_1 || h_2_zec
///       = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits
///
/// h_zsa = h_0 || h_1 || h_2_zsa
///       = (bits 249..=253 of psi) || (bit 254 of psi) || (bits 0..=3 of x(asset))
///
/// | A_6     | A_7     | A_8 | q_notecommit_h |
/// --------------------------------------------
/// |  h_zec  | h_0     | h_1 |       1        |
/// |  h_zsa  | h_2_zsa |     |       0        |
///
/// <https://p.z.cash/orchard-0.1:note-commit-decomposition-h?partial>
#[derive(Clone, Debug)]
struct DecomposeHZsa<Lookup: PallasLookupRangeCheck> {
    q_notecommit_h: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> DecomposeHZsa<Lookup> {
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        two_pow_5: pallas::Base,
        two_pow_6: pallas::Base,
    ) -> Self {
        let q_notecommit_h = meta.selector();

        meta.create_gate("NoteCommit MessagePiece h", |meta| {
            let q_notecommit_h = meta.query_selector(q_notecommit_h);

            // h_zec has been constrained to 10 bits by the Sinsemilla hash.
            let h_zec = meta.query_advice(col_l, Rotation::cur());
            // h_0 has been constrained to be 5 bits outside this gate.
            let h_0 = meta.query_advice(col_m, Rotation::cur());
            // This gate constrains h_1 to be boolean.
            let h_1 = meta.query_advice(col_r, Rotation::cur());

            // h_zsa has been constrained to 10 bits by the Sinsemilla hash.
            let h_zsa = meta.query_advice(col_l, Rotation::next());
            // h_2_zsa has been constrained to be 4 bits outside this gate.
            let h_2_zsa = meta.query_advice(col_m, Rotation::next());

            // h_zec = h_0 + (2^5) h_1
            let zec_decomposition_check = h_zec - (h_0.clone() + h_1.clone() * two_pow_5);

            // h_zsa = h_0 + (2^5) h_1 + (2^6) h_2_zsa
            let zsa_decomposition_check =
                h_zsa - (h_0 + h_1.clone() * two_pow_5 + h_2_zsa * two_pow_6);

            Constraints::with_selector(
                q_notecommit_h,
                [
                    ("bool_check h_1", bool_check(h_1)),
                    ("zec_decomposition", zec_decomposition_check),
                    ("zsa_decomposition", zsa_decomposition_check),
                ],
            )
        });

        Self {
            q_notecommit_h,
            col_l,
            col_m,
            col_r,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::type_complexity)]
    fn decompose(
        lookup_config: &Lookup,
        chip: SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
        layouter: &mut impl Layouter<pallas::Base>,
        psi: &AssignedCell<pallas::Base, pallas::Base>,
        asset: &NonIdentityEccPoint,
    ) -> Result<
        (
            NoteCommitPiece<Lookup>,
            NoteCommitPiece<Lookup>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
            RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        ),
        Error,
    > {
        // Constrain h_0 to be 5 bits.
        let h_0 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "h_0"),
            psi.value(),
            249..254,
        )?;

        // h_1 will be boolean-constrained in the gate.
        let h_1 = RangeConstrained::bitrange_of(psi.value(), 254..255);

        // Constrain h_2_zsa to be 4 bits.
        let h_2_zsa = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "h_2_zsa"),
            asset.x().value(),
            0..4,
        )?;

        let h_zec = MessagePiece::from_subpieces(
            chip.clone(),
            layouter.namespace(|| "h_zec"),
            [
                h_0.value(),
                h_1,
                RangeConstrained::bitrange_of(Value::known(&pallas::Base::zero()), 0..4),
            ],
        )?;

        let h_zsa = MessagePiece::from_subpieces(
            chip,
            layouter.namespace(|| "h_zsa"),
            [h_0.value(), h_1, h_2_zsa.value()],
        )?;

        Ok((h_zec, h_zsa, h_0, h_1, h_2_zsa))
    }

    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        h_zec: NoteCommitPiece<Lookup>,
        h_zsa: NoteCommitPiece<Lookup>,
        h_0: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        h_1: RangeConstrained<pallas::Base, Value<pallas::Base>>,
        h_2_zsa: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
    ) -> Result<AssignedCell<pallas::Base, pallas::Base>, Error> {
        layouter.assign_region(
            || "NoteCommit MessagePiece h",
            |mut region| {
                self.q_notecommit_h.enable(&mut region, 0)?;

                h_zec
                    .inner()
                    .cell_value()
                    .copy_advice(|| "h_zec", &mut region, self.col_l, 0)?;
                h_0.inner()
                    .copy_advice(|| "h_0", &mut region, self.col_m, 0)?;
                let h_1 = region.assign_advice(|| "h_1", self.col_r, 0, || *h_1.inner())?;

                h_zsa
                    .inner()
                    .cell_value()
                    .copy_advice(|| "h_zsa", &mut region, self.col_l, 1)?;

                h_2_zsa
                    .inner()
                    .copy_advice(|| "h_2_zsa", &mut region, self.col_m, 1)?;

                Ok(h_1)
            },
        )
    }
}

/// j = j_0 || j_1
///   = (bit 254 of x(asset)) || (ỹ bit of asset)
///
/// | A_6 | A_7 | A_8 | q_notecommit_j |
/// ------------------------------------
/// |  j  | j_0 | j_1 |       1        |
///
/// https://p.z.cash/orchard-0.1:note-commit-decomposition-j?partial
#[derive(Clone, Debug)]
struct DecomposeJ<Lookup: PallasLookupRangeCheck> {
    q_notecommit_j: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> DecomposeJ<Lookup> {
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        two: pallas::Base,
    ) -> Self {
        let q_notecommit_j = meta.selector();

        meta.create_gate("NoteCommit MessagePiece j", |meta| {
            let q_notecommit_j = meta.query_selector(q_notecommit_j);

            // j has been constrained to 10 bits by the Sinsemilla hash.
            let j = meta.query_advice(col_l, Rotation::cur());
            // This gate constrains j_0 to be boolean.
            let j_0 = meta.query_advice(col_m, Rotation::cur());
            // This gate constrains j_1 to be boolean.
            let j_1 = meta.query_advice(col_r, Rotation::cur());

            // j = j_0 + (2) j_1
            let decomposition_check = j - (j_0.clone() + j_1.clone() * two);

            Constraints::with_selector(
                q_notecommit_j,
                [
                    ("bool_check j_0", bool_check(j_0)),
                    ("bool_check j_1", bool_check(j_1)),
                    ("decomposition", decomposition_check),
                ],
            )
        });

        Self {
            q_notecommit_j,
            col_l,
            col_m,
            col_r,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::type_complexity)]
    fn decompose(
        chip: SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
        layouter: &mut impl Layouter<pallas::Base>,
        asset: &NonIdentityEccPoint,
    ) -> Result<
        (
            NoteCommitPiece<Lookup>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
            RangeConstrained<pallas::Base, Value<pallas::Base>>,
        ),
        Error,
    > {
        // j_0, j_1 will be boolean-constrained in the gate.
        let j_0 = RangeConstrained::bitrange_of(asset.x().value(), 254..255);
        let j_1 = RangeConstrained::bitrange_of(asset.y().value(), 0..1);

        let j = MessagePiece::from_subpieces(
            chip,
            layouter.namespace(|| "j"),
            [
                j_0,
                j_1,
                RangeConstrained::bitrange_of(Value::known(&pallas::Base::zero()), 0..8),
            ],
        )?;

        Ok((j, j_0, j_1))
    }

    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        j: NoteCommitPiece<Lookup>,
        j_0: RangeConstrained<pallas::Base, Value<pallas::Base>>,
        j_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
    ) -> Result<AssignedCell<pallas::Base, pallas::Base>, Error> {
        layouter.assign_region(
            || "NoteCommit MessagePiece j",
            |mut region| {
                self.q_notecommit_j.enable(&mut region, 0)?;

                j.inner()
                    .cell_value()
                    .copy_advice(|| "j", &mut region, self.col_l, 0)?;
                let j_0 = region.assign_advice(|| "j_0", self.col_m, 0, || *j_0.inner())?;
                j_1.inner()
                    .copy_advice(|| "j_1", &mut region, self.col_r, 0)?;

                Ok(j_0)
            },
        )
    }
}

/// |  A_6   | A_7 |   A_8   |     A_9     | q_notecommit_g_d |
/// -----------------------------------------------------------
/// | x(g_d) | b_0 | a       | z13_a       |        1         |
/// |        | b_1 | a_prime | z13_a_prime |        0         |
///
/// <https://p.z.cash/orchard-0.1:note-commit-canonicity-g_d?partial>
#[derive(Clone, Debug)]
struct GdCanonicity<Lookup: PallasLookupRangeCheck> {
    q_notecommit_g_d: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    col_z: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> GdCanonicity<Lookup> {
    #[allow(clippy::too_many_arguments)]
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        col_z: Column<Advice>,
        two_pow_130: Expression<pallas::Base>,
        two_pow_250: pallas::Base,
        two_pow_254: pallas::Base,
        t_p: Expression<pallas::Base>,
    ) -> Self {
        let q_notecommit_g_d = meta.selector();

        meta.create_gate("NoteCommit input g_d", |meta| {
            let q_notecommit_g_d = meta.query_selector(q_notecommit_g_d);

            let gd_x = meta.query_advice(col_l, Rotation::cur());

            // b_0 has been constrained to be 4 bits outside this gate.
            let b_0 = meta.query_advice(col_m, Rotation::cur());
            // b_1 has been constrained to be boolean outside this gate.
            let b_1 = meta.query_advice(col_m, Rotation::next());

            // a has been constrained to 250 bits by the Sinsemilla hash.
            let a = meta.query_advice(col_r, Rotation::cur());
            let a_prime = meta.query_advice(col_r, Rotation::next());

            let z13_a = meta.query_advice(col_z, Rotation::cur());
            let z13_a_prime = meta.query_advice(col_z, Rotation::next());

            // x(g_d) = a + (2^250)b_0 + (2^254)b_1
            let decomposition_check = {
                let sum = a.clone() + b_0.clone() * two_pow_250 + b_1.clone() * two_pow_254;
                sum - gd_x
            };

            // a_prime = a + 2^130 - t_P
            let a_prime_check = a + two_pow_130 - t_p - a_prime;

            // The gd_x_canonicity_checks are enforced if and only if `b_1` = 1.
            // x(g_d) = a (250 bits) || b_0 (4 bits) || b_1 (1 bit)
            let canonicity_checks = iter::empty()
                .chain(Some(("b_1 = 1 => b_0", b_0)))
                .chain(Some(("b_1 = 1 => z13_a", z13_a)))
                .chain(Some(("b_1 = 1 => z13_a_prime", z13_a_prime)))
                .map(move |(name, poly)| (name, b_1.clone() * poly));

            Constraints::with_selector(
                q_notecommit_g_d,
                iter::empty()
                    .chain(Some(("decomposition", decomposition_check)))
                    .chain(Some(("a_prime_check", a_prime_check)))
                    .chain(canonicity_checks),
            )
        });

        Self {
            q_notecommit_g_d,
            col_l,
            col_m,
            col_r,
            col_z,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        g_d: &NonIdentityEccPoint,
        a: NoteCommitPiece<Lookup>,
        b_0: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        b_1: AssignedCell<pallas::Base, pallas::Base>,
        a_prime: AssignedCell<pallas::Base, pallas::Base>,
        z13_a: AssignedCell<pallas::Base, pallas::Base>,
        z13_a_prime: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "NoteCommit input g_d",
            |mut region| {
                g_d.x().copy_advice(|| "gd_x", &mut region, self.col_l, 0)?;

                b_0.inner()
                    .copy_advice(|| "b_0", &mut region, self.col_m, 0)?;
                b_1.copy_advice(|| "b_1", &mut region, self.col_m, 1)?;

                a.inner()
                    .cell_value()
                    .copy_advice(|| "a", &mut region, self.col_r, 0)?;
                a_prime.copy_advice(|| "a_prime", &mut region, self.col_r, 1)?;

                z13_a.copy_advice(|| "z13_a", &mut region, self.col_z, 0)?;
                z13_a_prime.copy_advice(|| "z13_a_prime", &mut region, self.col_z, 1)?;

                self.q_notecommit_g_d.enable(&mut region, 0)
            },
        )
    }
}

/// For pk_d
/// |   A_6   | A_7 |    A_8     |      A_9       | q_notecommit_pk_d_asset |
/// -------------------------------------------------------------------------
/// | x(pk_d) | b_3 |    c       | z13_c          |         1               |
/// |         | d_0 | b3_c_prime | z14_b3_c_prime |         0               |
///
/// For asset
/// |   A_6   | A_7      |    A_8     |      A_9       | q_notecommit_pk_d_asset |
/// ------------------------------------------------------------------------------
/// | x(asset) | h_2_zsa |    i       | z13_i          |         1               |
/// |         | j_0      | h2_i_prime | z14_h2_i_prime |         0               |
///
/// <https://p.z.cash/orchard-0.1:note-commit-canonicity-pk_d?partial>
#[derive(Clone, Debug)]
struct PkdAssetCanonicity<Lookup: PallasLookupRangeCheck> {
    q_notecommit_pk_d_asset: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    col_z: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> PkdAssetCanonicity<Lookup> {
    #[allow(clippy::too_many_arguments)]
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        col_z: Column<Advice>,
        two_pow_4: pallas::Base,
        two_pow_140: Expression<pallas::Base>,
        two_pow_254: pallas::Base,
        t_p: Expression<pallas::Base>,
    ) -> Self {
        let q_notecommit_pk_d_asset = meta.selector();

        meta.create_gate("NoteCommit input pk_d or asset", |meta| {
            // The comments and variable names are for `pk_d`
            // This gate is also used with `asset`.
            // We have just to replace `pk_d`, `b_3`, `c`, `d_0` by `asset`, `h_2_zsa`, `i`, `j_0`
            let q_notecommit_pk_d_asset = meta.query_selector(q_notecommit_pk_d_asset);

            let pkd_x = meta.query_advice(col_l, Rotation::cur());

            // `b_3` has been constrained to 4 bits outside this gate.
            let b_3 = meta.query_advice(col_m, Rotation::cur());
            // d_0 has been constrained to be boolean outside this gate.
            let d_0 = meta.query_advice(col_m, Rotation::next());

            // `c` has been constrained to 250 bits by the Sinsemilla hash.
            let c = meta.query_advice(col_r, Rotation::cur());
            let b3_c_prime = meta.query_advice(col_r, Rotation::next());

            let z13_c = meta.query_advice(col_z, Rotation::cur());
            let z14_b3_c_prime = meta.query_advice(col_z, Rotation::next());

            // x(pk_d) = b_3 + (2^4)c + (2^254)d_0
            let decomposition_check = {
                let sum = b_3.clone() + c.clone() * two_pow_4 + d_0.clone() * two_pow_254;
                sum - pkd_x
            };

            // b3_c_prime = b_3 + (2^4)c + 2^140 - t_P
            let b3_c_prime_check = b_3 + (c * two_pow_4) + two_pow_140 - t_p - b3_c_prime;

            // The pkd_x_canonicity_checks are enforced if and only if `d_0` = 1.
            // `x(pk_d)` = `b_3 (4 bits) || c (250 bits) || d_0 (1 bit)`
            let canonicity_checks = iter::empty()
                .chain(Some(("d_0 = 1 => z13_c", z13_c)))
                .chain(Some(("d_0 = 1 => z14_b3_c_prime", z14_b3_c_prime)))
                .map(move |(name, poly)| (name, d_0.clone() * poly));

            Constraints::with_selector(
                q_notecommit_pk_d_asset,
                iter::empty()
                    .chain(Some(("decomposition", decomposition_check)))
                    .chain(Some(("b3_c_prime_check", b3_c_prime_check)))
                    .chain(canonicity_checks),
            )
        });

        Self {
            q_notecommit_pk_d_asset,
            col_l,
            col_m,
            col_r,
            col_z,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn assign(
        // This function is used for `pk_d` and `asset`.
        // For `pk_d`, inputs are `pk_d`, `b_3`, `c`, `d_0`, `b3_c_prime`, `z13_c`, `z14_b3_c_prime`
        // For `asset`, inputs are `asset`, `h_2_zsa`, `i`, `j_0`, `h2_i_prime`, `z13_i`, `z14_h2_i_prime`
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        pk_d: &NonIdentityEccPoint,
        b_3: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        c: NoteCommitPiece<Lookup>,
        d_0: AssignedCell<pallas::Base, pallas::Base>,
        b3_c_prime: AssignedCell<pallas::Base, pallas::Base>,
        z13_c: AssignedCell<pallas::Base, pallas::Base>,
        z14_b3_c_prime: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "NoteCommit input pk_d or asset",
            |mut region| {
                pk_d.x()
                    .copy_advice(|| "pkd_x", &mut region, self.col_l, 0)?;

                b_3.inner()
                    .copy_advice(|| "b_3", &mut region, self.col_m, 0)?;
                d_0.copy_advice(|| "d_0", &mut region, self.col_m, 1)?;

                c.inner()
                    .cell_value()
                    .copy_advice(|| "c", &mut region, self.col_r, 0)?;
                b3_c_prime.copy_advice(|| "b3_c_prime", &mut region, self.col_r, 1)?;

                z13_c.copy_advice(|| "z13_c", &mut region, self.col_z, 0)?;
                z14_b3_c_prime.copy_advice(|| "z14_b3_c_prime", &mut region, self.col_z, 1)?;

                self.q_notecommit_pk_d_asset.enable(&mut region, 0)
            },
        )
    }
}

/// |  A_6  | A_7 | A_8 | A_9 | q_notecommit_value |
/// ------------------------------------------------
/// | value | d_2 | d_3 | e_0 |          1         |
///
/// <https://p.z.cash/orchard-0.1:note-commit-canonicity-v?partial>
#[derive(Clone, Debug)]
struct ValueCanonicity {
    q_notecommit_value: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    col_z: Column<Advice>,
}

impl ValueCanonicity {
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        col_z: Column<Advice>,
        two_pow_8: pallas::Base,
        two_pow_58: pallas::Base,
    ) -> Self {
        let q_notecommit_value = meta.selector();

        meta.create_gate("NoteCommit input value", |meta| {
            let q_notecommit_value = meta.query_selector(q_notecommit_value);

            let value = meta.query_advice(col_l, Rotation::cur());
            // d_2 has been constrained to 8 bits outside this gate.
            let d_2 = meta.query_advice(col_m, Rotation::cur());
            // z1_d has been constrained to 50 bits by the Sinsemilla hash.
            let z1_d = meta.query_advice(col_r, Rotation::cur());
            let d_3 = z1_d;
            // `e_0` has been constrained to 6 bits outside this gate.
            let e_0 = meta.query_advice(col_z, Rotation::cur());

            // value = d_2 + (2^8)d_3 + (2^58)e_0
            let value_check = d_2 + d_3 * two_pow_8 + e_0 * two_pow_58 - value;

            Constraints::with_selector(q_notecommit_value, Some(("value_check", value_check)))
        });

        Self {
            q_notecommit_value,
            col_l,
            col_m,
            col_r,
            col_z,
        }
    }

    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        value: AssignedCell<NoteValue, pallas::Base>,
        d_2: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        z1_d: AssignedCell<pallas::Base, pallas::Base>,
        e_0: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "NoteCommit input value",
            |mut region| {
                value.copy_advice(|| "value", &mut region, self.col_l, 0)?;
                d_2.inner()
                    .copy_advice(|| "d_2", &mut region, self.col_m, 0)?;
                z1_d.copy_advice(|| "d3 = z1_d", &mut region, self.col_r, 0)?;
                e_0.inner()
                    .copy_advice(|| "e_0", &mut region, self.col_z, 0)?;

                self.q_notecommit_value.enable(&mut region, 0)
            },
        )
    }
}

/// | A_6 | A_7 |    A_8     |      A_9       | q_notecommit_rho |
/// --------------------------------------------------------------
/// | rho | e_1 |    f       | z13_f          |        1         |
/// |     | g_0 | e1_f_prime | z14_e1_f_prime |        0         |
///
/// <https://p.z.cash/orchard-0.1:note-commit-canonicity-rho?partial>
#[derive(Clone, Debug)]
struct RhoCanonicity<Lookup: PallasLookupRangeCheck> {
    q_notecommit_rho: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    col_z: Column<Advice>,
    _lookup_marker: std::marker::PhantomData<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> RhoCanonicity<Lookup> {
    #[allow(clippy::too_many_arguments)]
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        col_z: Column<Advice>,
        two_pow_4: pallas::Base,
        two_pow_140: Expression<pallas::Base>,
        two_pow_254: pallas::Base,
        t_p: Expression<pallas::Base>,
    ) -> Self {
        let q_notecommit_rho = meta.selector();

        meta.create_gate("NoteCommit input rho", |meta| {
            let q_notecommit_rho = meta.query_selector(q_notecommit_rho);

            let rho = meta.query_advice(col_l, Rotation::cur());

            // `e_1` has been constrained to 4 bits outside this gate.
            let e_1 = meta.query_advice(col_m, Rotation::cur());
            let g_0 = meta.query_advice(col_m, Rotation::next());

            // `f` has been constrained to 250 bits by the Sinsemilla hash.
            let f = meta.query_advice(col_r, Rotation::cur());
            let e1_f_prime = meta.query_advice(col_r, Rotation::next());

            let z13_f = meta.query_advice(col_z, Rotation::cur());
            let z14_e1_f_prime = meta.query_advice(col_z, Rotation::next());

            // rho = e_1 + (2^4) f + (2^254) g_0
            let decomposition_check = {
                let sum = e_1.clone() + f.clone() * two_pow_4 + g_0.clone() * two_pow_254;
                sum - rho
            };

            // e1_f_prime = e_1 + (2^4)f + 2^140 - t_P
            let e1_f_prime_check = e_1 + (f * two_pow_4) + two_pow_140 - t_p - e1_f_prime;

            // The rho_canonicity_checks are enforced if and only if `g_0` = 1.
            // rho = e_1 (4 bits) || f (250 bits) || g_0 (1 bit)
            let canonicity_checks = iter::empty()
                .chain(Some(("g_0 = 1 => z13_f", z13_f)))
                .chain(Some(("g_0 = 1 => z14_e1_f_prime", z14_e1_f_prime)))
                .map(move |(name, poly)| (name, g_0.clone() * poly));

            Constraints::with_selector(
                q_notecommit_rho,
                iter::empty()
                    .chain(Some(("decomposition", decomposition_check)))
                    .chain(Some(("e1_f_prime_check", e1_f_prime_check)))
                    .chain(canonicity_checks),
            )
        });

        Self {
            q_notecommit_rho,
            col_l,
            col_m,
            col_r,
            col_z,
            _lookup_marker: std::marker::PhantomData,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        rho: AssignedCell<pallas::Base, pallas::Base>,
        e_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        f: NoteCommitPiece<Lookup>,
        g_0: AssignedCell<pallas::Base, pallas::Base>,
        e1_f_prime: AssignedCell<pallas::Base, pallas::Base>,
        z13_f: AssignedCell<pallas::Base, pallas::Base>,
        z14_e1_f_prime: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "NoteCommit input rho",
            |mut region| {
                rho.copy_advice(|| "rho", &mut region, self.col_l, 0)?;

                e_1.inner()
                    .copy_advice(|| "e_1", &mut region, self.col_m, 0)?;
                g_0.copy_advice(|| "g_0", &mut region, self.col_m, 1)?;

                f.inner()
                    .cell_value()
                    .copy_advice(|| "f", &mut region, self.col_r, 0)?;
                e1_f_prime.copy_advice(|| "e1_f_prime", &mut region, self.col_r, 1)?;

                z13_f.copy_advice(|| "z13_f", &mut region, self.col_z, 0)?;
                z14_e1_f_prime.copy_advice(|| "z14_e1_f_prime", &mut region, self.col_z, 1)?;

                self.q_notecommit_rho.enable(&mut region, 0)
            },
        )
    }
}

/// | A_6 | A_7 |     A_8     |       A_9       | q_notecommit_psi |
/// ----------------------------------------------------------------
/// | psi | g_1 |   g_2       | z13_g           |        1         |
/// | h_0 | h_1 | g1_g2_prime | z13_g1_g2_prime |        0         |
///
/// <https://p.z.cash/orchard-0.1:note-commit-canonicity-psi?partial>
#[derive(Clone, Debug)]
struct PsiCanonicity {
    q_notecommit_psi: Selector,
    col_l: Column<Advice>,
    col_m: Column<Advice>,
    col_r: Column<Advice>,
    col_z: Column<Advice>,
}

impl PsiCanonicity {
    #[allow(clippy::too_many_arguments)]
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        col_l: Column<Advice>,
        col_m: Column<Advice>,
        col_r: Column<Advice>,
        col_z: Column<Advice>,
        two_pow_9: pallas::Base,
        two_pow_130: Expression<pallas::Base>,
        two_pow_249: pallas::Base,
        two_pow_254: pallas::Base,
        t_p: Expression<pallas::Base>,
    ) -> Self {
        let q_notecommit_psi = meta.selector();

        meta.create_gate("NoteCommit input psi", |meta| {
            let q_notecommit_psi = meta.query_selector(q_notecommit_psi);

            let psi = meta.query_advice(col_l, Rotation::cur());
            let h_0 = meta.query_advice(col_l, Rotation::next());

            let g_1 = meta.query_advice(col_m, Rotation::cur());
            let h_1 = meta.query_advice(col_m, Rotation::next());

            let z1_g = meta.query_advice(col_r, Rotation::cur());
            let g_2 = z1_g;
            let g1_g2_prime = meta.query_advice(col_r, Rotation::next());

            let z13_g = meta.query_advice(col_z, Rotation::cur());
            let z13_g1_g2_prime = meta.query_advice(col_z, Rotation::next());

            // psi = g_1 + (2^9) g_2 + (2^249) h_0 + (2^254) h_1
            let decomposition_check = {
                let sum = g_1.clone()
                    + g_2.clone() * two_pow_9
                    + h_0.clone() * two_pow_249
                    + h_1.clone() * two_pow_254;
                sum - psi
            };

            // g1_g2_prime = g_1 + (2^9)g_2 + 2^130 - t_P
            let g1_g2_prime_check = g_1 + (g_2 * two_pow_9) + two_pow_130 - t_p - g1_g2_prime;

            // The psi_canonicity_checks are enforced if and only if `h_1` = 1.
            // `psi` = `g_1 (9 bits) || g_2 (240 bits) || h_0 (5 bits) || h_1 (1 bit)`
            let canonicity_checks = iter::empty()
                .chain(Some(("h_1 = 1 => h_0", h_0)))
                .chain(Some(("h_1 = 1 => z13_g", z13_g)))
                .chain(Some(("h_1 = 1 => z13_g1_g2_prime", z13_g1_g2_prime)))
                .map(move |(name, poly)| (name, h_1.clone() * poly));

            Constraints::with_selector(
                q_notecommit_psi,
                iter::empty()
                    .chain(Some(("decomposition", decomposition_check)))
                    .chain(Some(("g1_g2_prime_check", g1_g2_prime_check)))
                    .chain(canonicity_checks),
            )
        });

        Self {
            q_notecommit_psi,
            col_l,
            col_m,
            col_r,
            col_z,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        psi: AssignedCell<pallas::Base, pallas::Base>,
        g_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        z1_g: AssignedCell<pallas::Base, pallas::Base>,
        h_0: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        h_1: AssignedCell<pallas::Base, pallas::Base>,
        g1_g2_prime: AssignedCell<pallas::Base, pallas::Base>,
        z13_g: AssignedCell<pallas::Base, pallas::Base>,
        z13_g1_g2_prime: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "NoteCommit input psi",
            |mut region| {
                psi.copy_advice(|| "psi", &mut region, self.col_l, 0)?;
                h_0.inner()
                    .copy_advice(|| "h_0", &mut region, self.col_l, 1)?;

                g_1.inner()
                    .copy_advice(|| "g_1", &mut region, self.col_m, 0)?;
                h_1.copy_advice(|| "h_1", &mut region, self.col_m, 1)?;

                z1_g.copy_advice(|| "g_2 = z1_g", &mut region, self.col_r, 0)?;
                g1_g2_prime.copy_advice(|| "g1_g2_prime", &mut region, self.col_r, 1)?;

                z13_g.copy_advice(|| "z13_g", &mut region, self.col_z, 0)?;
                z13_g1_g2_prime.copy_advice(|| "z13_g1_g2_prime", &mut region, self.col_z, 1)?;

                self.q_notecommit_psi.enable(&mut region, 0)
            },
        )
    }
}

/// Check decomposition and canonicity of y-coordinates.
/// This is used for both y(g_d) and y(pk_d).
///
/// y = LSB || k_0 || k_1 || k_2 || k_3
///   = (bit 0) || (bits 1..=9) || (bits 10..=249) || (bits 250..=253) || (bit 254)
///
/// These pieces are laid out in the following configuration:
/// | A_5 | A_6 |  A_7  |   A_8   |     A_9     | q_y_canon |
/// ---------------------------------------------------------
/// |  y  | lsb |  k_0  |   k_2   |     k_3     |     1     |
/// |  j  | z1_j| z13_j | j_prime | z13_j_prime |     0     |
/// where z1_j = k_1.
#[derive(Clone, Debug)]
struct YCanonicity {
    q_y_canon: Selector,
    advices: [Column<Advice>; 10],
}

impl YCanonicity {
    #[allow(clippy::too_many_arguments)]
    fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 10],
        two: pallas::Base,
        two_pow_10: pallas::Base,
        two_pow_130: Expression<pallas::Base>,
        two_pow_250: pallas::Base,
        two_pow_254: pallas::Base,
        t_p: Expression<pallas::Base>,
    ) -> Self {
        let q_y_canon = meta.selector();

        meta.create_gate("y coordinate checks", |meta| {
            let q_y_canon = meta.query_selector(q_y_canon);
            let y = meta.query_advice(advices[5], Rotation::cur());
            // LSB has been boolean-constrained outside this gate.
            let lsb = meta.query_advice(advices[6], Rotation::cur());
            // k_0 has been constrained to 9 bits outside this gate.
            let k_0 = meta.query_advice(advices[7], Rotation::cur());
            // k_1 = z1_j (witnessed in the next rotation).
            // k_2 has been constrained to 4 bits outside this gate.
            let k_2 = meta.query_advice(advices[8], Rotation::cur());
            // This gate constrains k_3 to be boolean.
            let k_3 = meta.query_advice(advices[9], Rotation::cur());

            // j = LSB + (2)k_0 + (2^10)k_1
            let j = meta.query_advice(advices[5], Rotation::next());
            let z1_j = meta.query_advice(advices[6], Rotation::next());
            let z13_j = meta.query_advice(advices[7], Rotation::next());

            // j_prime = j + 2^130 - t_P
            let j_prime = meta.query_advice(advices[8], Rotation::next());
            let z13_j_prime = meta.query_advice(advices[9], Rotation::next());

            // Decomposition checks
            // https://p.z.cash/orchard-0.1:note-commit-decomposition-y?partial
            let decomposition_checks = {
                // Check that k_3 is boolean
                let k3_check = bool_check(k_3.clone());
                // Check that j = LSB + (2)k_0 + (2^10)k_1
                let k_1 = z1_j;
                let j_check = j.clone() - (lsb + k_0 * two + k_1 * two_pow_10);
                // Check that y = j + (2^250)k_2 + (2^254)k_3
                let y_check =
                    y - (j.clone() + k_2.clone() * two_pow_250 + k_3.clone() * two_pow_254);
                // Check that j_prime = j + 2^130 - t_P
                let j_prime_check = j + two_pow_130 - t_p - j_prime;

                iter::empty()
                    .chain(Some(("k3_check", k3_check)))
                    .chain(Some(("j_check", j_check)))
                    .chain(Some(("y_check", y_check)))
                    .chain(Some(("j_prime_check", j_prime_check)))
            };

            // Canonicity checks. These are enforced if and only if k_3 = 1.
            // https://p.z.cash/orchard-0.1:note-commit-canonicity-y?partial
            let canonicity_checks = {
                iter::empty()
                    .chain(Some(("k_3 = 1 => k_2 = 0", k_2)))
                    .chain(Some(("k_3 = 1 => z13_j = 0", z13_j)))
                    .chain(Some(("k_3 = 1 => z13_j_prime = 0", z13_j_prime)))
                    .map(move |(name, poly)| (name, k_3.clone() * poly))
            };

            Constraints::with_selector(q_y_canon, decomposition_checks.chain(canonicity_checks))
        });

        Self { q_y_canon, advices }
    }

    #[allow(clippy::too_many_arguments)]
    fn assign(
        &self,
        layouter: &mut impl Layouter<pallas::Base>,
        y: AssignedCell<pallas::Base, pallas::Base>,
        lsb: RangeConstrained<pallas::Base, Value<pallas::Base>>,
        k_0: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        k_2: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        k_3: RangeConstrained<pallas::Base, Value<pallas::Base>>,
        j: AssignedCell<pallas::Base, pallas::Base>,
        z1_j: AssignedCell<pallas::Base, pallas::Base>,
        z13_j: AssignedCell<pallas::Base, pallas::Base>,
        j_prime: AssignedCell<pallas::Base, pallas::Base>,
        z13_j_prime: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>, Error>
    {
        layouter.assign_region(
            || "y canonicity",
            |mut region| {
                self.q_y_canon.enable(&mut region, 0)?;

                // Offset 0
                let lsb = {
                    let offset = 0;

                    // Copy y.
                    y.copy_advice(|| "copy y", &mut region, self.advices[5], offset)?;
                    // Witness LSB.
                    let lsb = region
                        .assign_advice(|| "witness LSB", self.advices[6], offset, || *lsb.inner())
                        // SAFETY: This is sound because we just assigned this cell from a
                        // range-constrained value.
                        .map(|cell| RangeConstrained::unsound_unchecked(cell, lsb.num_bits()))?;
                    // Witness k_0.
                    k_0.inner()
                        .copy_advice(|| "copy k_0", &mut region, self.advices[7], offset)?;
                    // Copy k_2.
                    k_2.inner()
                        .copy_advice(|| "copy k_2", &mut region, self.advices[8], offset)?;
                    // Witness k_3.
                    region.assign_advice(
                        || "witness k_3",
                        self.advices[9],
                        offset,
                        || *k_3.inner(),
                    )?;

                    lsb
                };

                // Offset 1
                {
                    let offset = 1;

                    // Copy j.
                    j.copy_advice(|| "copy j", &mut region, self.advices[5], offset)?;
                    // Copy z1_j.
                    z1_j.copy_advice(|| "copy z1_j", &mut region, self.advices[6], offset)?;
                    // Copy z13_j.
                    z13_j.copy_advice(|| "copy z13_j", &mut region, self.advices[7], offset)?;
                    // Copy j_prime.
                    j_prime.copy_advice(|| "copy j_prime", &mut region, self.advices[8], offset)?;
                    // Copy z13_j_prime.
                    z13_j_prime.copy_advice(
                        || "copy z13_j_prime",
                        &mut region,
                        self.advices[9],
                        offset,
                    )?;
                }

                Ok(lsb)
            },
        )
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug)]
pub struct NoteCommitConfig<Lookup: PallasLookupRangeCheck> {
    b: DecomposeB<Lookup>,
    d: DecomposeD<Lookup>,
    e: DecomposeE<Lookup>,
    g: DecomposeG<Lookup>,
    g_d: GdCanonicity<Lookup>,
    pk_d_asset: PkdAssetCanonicity<Lookup>,
    value: ValueCanonicity,
    rho: RhoCanonicity<Lookup>,
    psi: PsiCanonicity,
    y_canon: YCanonicity,
    advices: [Column<Advice>; 10],
    sinsemilla_config:
        SinsemillaConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
    specific_config_for_circuit: SpecificConfigForCircuit<Lookup>,
}

#[derive(Clone, Debug)]
pub enum SpecificConfigForCircuit<Lookup: PallasLookupRangeCheck> {
    Vanilla(NoteCommitConfigForVanillaCircuit<Lookup>),
    Zsa(NoteCommitConfigForZsaCircuit<Lookup>),
}

#[derive(Clone, Debug)]
pub struct NoteCommitConfigForVanillaCircuit<Lookup: PallasLookupRangeCheck> {
    h_vanilla: DecomposeHVanilla<Lookup>,
}

#[derive(Clone, Debug)]
pub struct NoteCommitConfigForZsaCircuit<Lookup: PallasLookupRangeCheck> {
    h_zsa: DecomposeHZsa<Lookup>,
    j: DecomposeJ<Lookup>,
}

#[derive(Clone, Debug)]
pub struct NoteCommitChip<Lookup: PallasLookupRangeCheck> {
    pub config: NoteCommitConfig<Lookup>,
}

impl<Lookup: PallasLookupRangeCheck> NoteCommitChip<Lookup> {
    #[allow(non_snake_case)]
    #[allow(clippy::many_single_char_names)]
    pub(in crate::circuit) fn configure(
        meta: &mut ConstraintSystem<pallas::Base>,
        advices: [Column<Advice>; 10],
        sinsemilla_config: SinsemillaConfig<
            OrchardHashDomains,
            OrchardCommitDomains,
            OrchardFixedBases,
            Lookup,
        >,
        is_zsa_circuit: bool,
    ) -> NoteCommitConfig<Lookup> {
        // Useful constants
        let two = pallas::Base::from(2);
        let two_pow_2 = pallas::Base::from(1 << 2);
        let two_pow_4 = two_pow_2.square();
        let two_pow_5 = two_pow_4 * two;
        let two_pow_6 = two_pow_5 * two;
        let two_pow_8 = two_pow_4.square();
        let two_pow_9 = two_pow_8 * two;
        let two_pow_10 = two_pow_9 * two;
        let two_pow_58 = pallas::Base::from(1 << 58);
        let two_pow_130 = Expression::Constant(pallas::Base::from_u128(1 << 65).square());
        let two_pow_140 = Expression::Constant(pallas::Base::from_u128(1 << 70).square());
        let two_pow_249 = pallas::Base::from_u128(1 << 124).square() * two;
        let two_pow_250 = two_pow_249 * two;
        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();

        let t_p = Expression::Constant(pallas::Base::from_u128(T_P));

        // Columns used for MessagePiece and message input gates.
        let col_l = advices[6];
        let col_m = advices[7];
        let col_r = advices[8];
        let col_z = advices[9];

        let b = DecomposeB::configure(meta, col_l, col_m, col_r, two_pow_4, two_pow_5, two_pow_6);
        let d = DecomposeD::configure(meta, col_l, col_m, col_r, two, two_pow_2, two_pow_10);
        let e = DecomposeE::configure(meta, col_l, col_m, col_r, two_pow_6);
        let g = DecomposeG::configure(meta, col_l, col_m, two, two_pow_10);
        let specific_config_for_circuit = if is_zsa_circuit {
            let h = DecomposeHZsa::configure(meta, col_l, col_m, col_r, two_pow_5, two_pow_6);
            let j = DecomposeJ::configure(meta, col_l, col_m, col_r, two);
            SpecificConfigForCircuit::Zsa(NoteCommitConfigForZsaCircuit { h_zsa: h, j })
        } else {
            let h = DecomposeHVanilla::configure(meta, col_l, col_m, col_r, two_pow_5);
            SpecificConfigForCircuit::Vanilla(NoteCommitConfigForVanillaCircuit { h_vanilla: h })
        };

        let g_d = GdCanonicity::configure(
            meta,
            col_l,
            col_m,
            col_r,
            col_z,
            two_pow_130.clone(),
            two_pow_250,
            two_pow_254,
            t_p.clone(),
        );

        let pk_d_asset = PkdAssetCanonicity::configure(
            meta,
            col_l,
            col_m,
            col_r,
            col_z,
            two_pow_4,
            two_pow_140.clone(),
            two_pow_254,
            t_p.clone(),
        );

        let value =
            ValueCanonicity::configure(meta, col_l, col_m, col_r, col_z, two_pow_8, two_pow_58);

        let rho = RhoCanonicity::configure(
            meta,
            col_l,
            col_m,
            col_r,
            col_z,
            two_pow_4,
            two_pow_140,
            two_pow_254,
            t_p.clone(),
        );

        let psi = PsiCanonicity::configure(
            meta,
            col_l,
            col_m,
            col_r,
            col_z,
            two_pow_9,
            two_pow_130.clone(),
            two_pow_249,
            two_pow_254,
            t_p.clone(),
        );

        let y_canon = YCanonicity::configure(
            meta,
            advices,
            two,
            two_pow_10,
            two_pow_130,
            two_pow_250,
            two_pow_254,
            t_p,
        );

        NoteCommitConfig {
            b,
            d,
            e,
            g,
            g_d,
            pk_d_asset,
            value,
            rho,
            psi,
            y_canon,
            advices,
            sinsemilla_config,
            specific_config_for_circuit,
        }
    }

    pub(in crate::circuit) fn construct(config: NoteCommitConfig<Lookup>) -> Self {
        Self { config }
    }
}

pub struct ZsaNoteCommitParams {
    pub cond_swap_chip: CondSwapChip<pallas::Base>,
    pub asset: NonIdentityEccPoint,
    pub is_native_asset: AssignedCell<pallas::Base, pallas::Base>,
}
pub struct ZsaFinalDecomposition<Lookup: PallasLookupRangeCheck> {
    pub h_zsa: NoteCommitPiece<Lookup>,
    pub h_2_zsa: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
    pub i: NoteCommitPiece<Lookup>,
    pub j: NoteCommitPiece<Lookup>,
    pub j_0: RangeConstrained<pallas::Base, Value<pallas::Base>>,
    pub j_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
}

pub(in crate::circuit) mod gadgets {
    use super::*;

    #[allow(clippy::many_single_char_names)]
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub(in crate::circuit) fn note_commit<Lookup: PallasLookupRangeCheck>(
        mut layouter: impl Layouter<pallas::Base>,
        chip: SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
        ecc_chip: EccChip<OrchardFixedBases, Lookup>,
        note_commit_chip: NoteCommitChip<Lookup>,
        g_d: &NonIdentityEccPoint,
        pk_d: &NonIdentityEccPoint,
        value: AssignedCell<NoteValue, pallas::Base>,
        rho: AssignedCell<pallas::Base, pallas::Base>,
        psi: AssignedCell<pallas::Base, pallas::Base>,
        rcm: ScalarFixed<pallas::Affine, EccChip<OrchardFixedBases, Lookup>>,
        zsa_params: Option<ZsaNoteCommitParams>,
    ) -> Result<Point<pallas::Affine, EccChip<OrchardFixedBases, Lookup>>, Error> {
        match (&zsa_params, &note_commit_chip.config.specific_config_for_circuit) {
            (Some(_), SpecificConfigForCircuit::Zsa(_)) => {},
            (None, SpecificConfigForCircuit::Vanilla(_)) => {},
            _ => panic!("ZSA parameters must be provided for ZSA circuits, and must not be provided for vanilla circuits"),
        }

        let lookup_config = chip.config().lookup_config();

        // `a` = bits 0..=249 of `x(g_d)`
        let a = MessagePiece::from_subpieces(
            chip.clone(),
            layouter.namespace(|| "a"),
            [RangeConstrained::bitrange_of(g_d.x().value(), 0..250)],
        )?;

        // b = b_0 || b_1 || b_2 || b_3
        //   = (bits 250..=253 of x(g_d)) || (bit 254 of x(g_d)) || (ỹ bit of g_d) || (bits 0..=3 of pk★_d)
        let (b, b_0, b_1, b_2, b_3) =
            DecomposeB::decompose(&lookup_config, chip.clone(), &mut layouter, g_d, pk_d)?;

        // c = bits 4..=253 of pk★_d
        let c = MessagePiece::from_subpieces(
            chip.clone(),
            layouter.namespace(|| "c"),
            [RangeConstrained::bitrange_of(pk_d.x().value(), 4..254)],
        )?;

        // d = d_0 || d_1 || d_2 || d_3
        //   = (bit 254 of x(pk_d)) || (ỹ bit of pk_d) || (bits 0..=7 of v) || (bits 8..=57 of v)
        let (d, d_0, d_1, d_2) =
            DecomposeD::decompose(&lookup_config, chip.clone(), &mut layouter, pk_d, &value)?;

        // e = e_0 || e_1 = (bits 58..=63 of v) || (bits 0..=3 of rho)
        let (e, e_0, e_1) =
            DecomposeE::decompose(&lookup_config, chip.clone(), &mut layouter, &value, &rho)?;

        // f = bits 4..=253 inclusive of rho
        let f = MessagePiece::from_subpieces(
            chip.clone(),
            layouter.namespace(|| "f"),
            [RangeConstrained::bitrange_of(rho.value(), 4..254)],
        )?;

        // g = g_0 || g_1 || g_2
        //   = (bit 254 of rho) || (bits 0..=8 of psi) || (bits 9..=248 of psi)
        let (g, g_0, g_1) =
            DecomposeG::decompose(&lookup_config, chip.clone(), &mut layouter, &rho, &psi)?;

        let (h_zec, h_0, h_1, zsa_decomposition) = match &zsa_params {
            None => {
                // h_zec = h_0 || h_1 || h_2
                //       = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits
                let (h_zec, h_0, h_1) = DecomposeHVanilla::decompose(
                    &lookup_config,
                    chip.clone(),
                    &mut layouter,
                    &psi,
                )?;
                (h_zec, h_0, h_1, None)
            }
            Some(zsa_params) => {
                // h_zec = h_0 || h_1 || h_2_zec
                //   = (bits 249..=253 of psi) || (bit 254 of psi) || 4 zero bits
                // h_zsa = h_0 || h_1 || h_2_zsa
                //   = (bits 249..=253 of psi) || (bit 254 of psi) || (bits 0..=3 of x(asset))
                let (h_zec, h_zsa, h_0, h_1, h_2_zsa) = DecomposeHZsa::decompose(
                    &lookup_config,
                    chip.clone(),
                    &mut layouter,
                    &psi,
                    &zsa_params.asset,
                )?;

                // i = bits 4..=253 of asset
                let i = MessagePiece::from_subpieces(
                    chip.clone(),
                    layouter.namespace(|| "i"),
                    [RangeConstrained::bitrange_of(
                        zsa_params.asset.x().value(),
                        4..254,
                    )],
                )?;

                // j = j_0 || j_1 || j_2 = (bit 254 of x(asset)) || (ỹ bit of asset) || 8 zero bits
                let (j, j_0, j_1) =
                    DecomposeJ::decompose(chip.clone(), &mut layouter, &zsa_params.asset)?;

                // Check decomposition of `y(asset)`.
                let j_1 = y_canonicity(
                    &lookup_config,
                    &note_commit_chip.config.y_canon,
                    layouter.namespace(|| "y(asset) decomposition"),
                    zsa_params.asset.y(),
                    j_1,
                )?;

                (
                    h_zec,
                    h_0,
                    h_1,
                    Some(ZsaFinalDecomposition {
                        h_zsa,
                        h_2_zsa,
                        i,
                        j,
                        j_0,
                        j_1,
                    }),
                )
            }
        };

        // Check decomposition of `y(g_d)`.
        let b_2 = y_canonicity(
            &lookup_config,
            &note_commit_chip.config.y_canon,
            layouter.namespace(|| "y(g_d) decomposition"),
            g_d.y(),
            b_2,
        )?;

        // Check decomposition of `y(pk_d)`.
        let d_1 = y_canonicity(
            &lookup_config,
            &note_commit_chip.config.y_canon,
            layouter.namespace(|| "y(pk_d) decomposition"),
            pk_d.y(),
            d_1,
        )?;

        let (cm, zs_common, zs_zsa_suffix) = match (&zsa_params, &zsa_decomposition) {
            (None, None) => {
                // cm = NoteCommit^Orchard_rcm(g★_d || pk★_d || i2lebsp_{64}(v) || rho || psi)
                //
                // `cm = ⊥` is handled internally to `CommitDomain::commit`: incomplete addition
                // constraints allows ⊥ to occur, and then during synthesis it detects these edge
                // cases and raises an error (aborting proof creation).
                //
                // https://p.z.cash/ZKS:action-cm-old-integrity?partial
                // https://p.z.cash/ZKS:action-cmx-new-integrity?partial
                let message = Message::from_pieces(
                    chip.clone(),
                    vec![
                        a.clone(),
                        b.clone(),
                        c.clone(),
                        d.clone(),
                        e.clone(),
                        f.clone(),
                        g.clone(),
                        h_zec.clone(),
                    ],
                );
                let domain = CommitDomain::new(chip, ecc_chip, &OrchardCommitDomains::NoteCommit);
                let (cm, zs) = domain.commit(
                    layouter.namespace(|| "Process NoteCommit inputs"),
                    message,
                    rcm,
                )?;
                (cm, zs, vec![])
            }
            (Some(zsa_params), Some(zsa_decomposition)) => {
                // cm = NoteCommit^Orchard_rcm(g★_d || pk★_d || i2lebsp_{64}(v) || rho || psi || asset)
                //
                // `cm = ⊥` is handled internally to `CommitDomain::commit`: incomplete addition
                // constraints allows ⊥ to occur, and then during synthesis it detects these edge
                // cases and raises an error (aborting proof creation).
                //
                // https://p.z.cash/ZKS:action-cm-old-integrity?partial
                // https://p.z.cash/ZKS:action-cmx-new-integrity?partial
                let message_common_prefix = Message::from_pieces(
                    chip.clone(),
                    vec![
                        a.clone(),
                        b.clone(),
                        c.clone(),
                        d.clone(),
                        e.clone(),
                        f.clone(),
                        g.clone(),
                    ],
                );

                let message_suffix_zec = Message::from_pieces(chip.clone(), vec![h_zec.clone()]);

                let message_suffix_zsa = Message::from_pieces(
                    chip.clone(),
                    vec![
                        zsa_decomposition.h_zsa.clone(),
                        zsa_decomposition.i.clone(),
                        zsa_decomposition.j.clone(),
                    ],
                );

                // We will evaluate
                // - `hash_point_zec = hash(Q_ZEC, message_common_prefix || message_suffix_zec)`, and
                // - `hash_point_zsa = hash(Q_ZSA, message_common_prefix || message_suffix_zsa)`.
                // by sharing a portion of the hash evaluation process between `hash_point_zec` and
                // `hash_point_zsa`:
                // 1. Q = if (is_native_asset == 0) {Q_ZSA} else {Q_ZEC}
                // 2. common_hash = hash(Q, message_common_prefix) // this part is shared
                // 3. hash_point_zec = hash(common_hash, message_suffix_zec)
                // 4. hash_point_zsa = hash(common_hash, message_suffix_zsa)
                // 5. hash_point = if (is_native_asset == 0) {hash_point_zsa} else {hash_point_zec}
                let zec_domain = CommitDomain::new(
                    chip.clone(),
                    ecc_chip.clone(),
                    &OrchardCommitDomains::NoteCommit,
                );
                let zsa_domain =
                    CommitDomain::new(chip, ecc_chip.clone(), &OrchardCommitDomains::NoteZsaCommit);

                // Perform a MUX to select the desired initial Q point
                // q_init = q_init_zec if is_native_asset is true
                // q_init = q_init_zsa if is_native_asset is false
                let q_init = {
                    let q_init_zec = NonIdentityPoint::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "q_init_zec"),
                        Value::known(zec_domain.q_init()),
                    )?;

                    let q_init_zsa = NonIdentityPoint::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "q_init_zsa"),
                        Value::known(zsa_domain.q_init()),
                    )?;

                    zsa_params.cond_swap_chip.mux_on_non_identity_points(
                        layouter.namespace(|| "mux on hash point"),
                        &zsa_params.is_native_asset,
                        q_init_zsa.inner(),
                        q_init_zec.inner(),
                    )?
                };

                // common_hash = hash(q_init, message_common_prefix)
                //
                // To evaluate the different hash, we could use either zec_domain or zsa_domain
                // because we use a private initial point.
                let (common_hash, zs_common) = zec_domain.hash_with_private_init(
                    layouter.namespace(|| "hash common prefix note"),
                    &q_init,
                    message_common_prefix,
                )?;

                // hash_point_zec = hash(common_hash, message_suffix_zec) = hash(q_init, message_zec)
                let (hash_point_zec, _zs_zec) = zec_domain.hash_with_private_init(
                    layouter.namespace(|| "hash suffix ZEC note"),
                    common_hash.inner(),
                    message_suffix_zec,
                )?;

                // hash_point_zsa = hash(common_hash, message_suffix_zsa) = hash(q_init, message_zsa)
                let (hash_point_zsa, zs_zsa) = zec_domain.hash_with_private_init(
                    layouter.namespace(|| "hash suffix ZSA note"),
                    common_hash.inner(),
                    message_suffix_zsa,
                )?;

                // Perform a MUX to select the desired hash point
                // hash_point = hash_zec if is_native_asset is true
                // hash_point = hash_zsa if is_native_asset is false
                let hash_point = Point::from_inner(
                    ecc_chip,
                    zsa_params.cond_swap_chip.mux_on_points(
                        layouter.namespace(|| "mux on hash point"),
                        &zsa_params.is_native_asset,
                        &(hash_point_zsa.inner().clone().into()),
                        &(hash_point_zec.inner().clone().into()),
                    )?,
                );

                // To evaluate the blinding factor, we could use either zec_domain or zsa_domain
                // because they have both the same `R` constant.
                let blinding_factor =
                    zec_domain.blinding_factor(layouter.namespace(|| "[r] R"), rcm)?;
                let commitment =
                    hash_point.add(layouter.namespace(|| "M + [r] R"), &blinding_factor)?;

                (commitment, zs_common, zs_zsa)
            }
            _ => {
                panic!("Either both zsa_params and zsa_decomposition must be provided, or neither.")
            }
        };

        // `CommitDomain::hash` returns the running sum for each `MessagePiece`. Grab
        // the outputs that we will need for canonicity checks.
        let z13_a = zs_common[0][13].clone();
        let z13_c = zs_common[2][13].clone();
        let z1_d = zs_common[3][1].clone();
        let z13_f = zs_common[5][13].clone();
        let z1_g = zs_common[6][1].clone();
        let g_2 = z1_g.clone();
        let z13_g = zs_common[6][13].clone();

        // Witness and constrain the bounds we need to ensure canonicity.
        let (a_prime, z13_a_prime) = canon_bitshift_130(
            &lookup_config,
            layouter.namespace(|| "x(g_d) canonicity"),
            a.inner().cell_value(),
        )?;

        let (b3_c_prime, z14_b3_c_prime) = pkd_asset_x_canonicity(
            &lookup_config,
            layouter.namespace(|| "x(pk_d) canonicity"),
            b_3.clone(),
            c.inner().cell_value(),
        )?;

        let (e1_f_prime, z14_e1_f_prime) = rho_canonicity(
            &lookup_config,
            layouter.namespace(|| "rho canonicity"),
            e_1.clone(),
            f.inner().cell_value(),
        )?;

        let (g1_g2_prime, z13_g1_g2_prime) = psi_canonicity(
            &lookup_config,
            layouter.namespace(|| "psi canonicity"),
            g_1.clone(),
            g_2,
        )?;

        // Finally, assign values to all of the NoteCommit regions.
        let cfg = note_commit_chip.config.clone();

        let b_1 = cfg
            .b
            .assign(&mut layouter, b, b_0.clone(), b_1, b_2, b_3.clone())?;

        let d_0 = cfg
            .d
            .assign(&mut layouter, d, d_0, d_1, d_2.clone(), z1_d.clone())?;

        cfg.e.assign(&mut layouter, e, e_0.clone(), e_1.clone())?;

        let g_0 = cfg
            .g
            .assign(&mut layouter, g, g_0, g_1.clone(), z1_g.clone())?;

        let h_1 = match (
            &zsa_decomposition,
            &note_commit_chip.config.specific_config_for_circuit,
        ) {
            (None, SpecificConfigForCircuit::Vanilla(specific_config)) => specific_config
                .h_vanilla
                .assign(&mut layouter, h_zec, h_0.clone(), h_1)?,
            (Some(zsa_decomposition), SpecificConfigForCircuit::Zsa(specific_config)) => {
                specific_config.h_zsa.assign(
                    &mut layouter,
                    h_zec,
                    zsa_decomposition.h_zsa.clone(),
                    h_0.clone(),
                    h_1,
                    zsa_decomposition.h_2_zsa.clone(),
                )?
            }
            _ => panic!("zsa_decomposition is required with a ZSA configuration but must be omitted with a Vanilla configuration."),
        };

        cfg.g_d
            .assign(&mut layouter, g_d, a, b_0, b_1, a_prime, z13_a, z13_a_prime)?;

        cfg.pk_d_asset.assign(
            &mut layouter,
            pk_d,
            b_3,
            c,
            d_0,
            b3_c_prime,
            z13_c,
            z14_b3_c_prime,
        )?;

        cfg.value.assign(&mut layouter, value, d_2, z1_d, e_0)?;

        cfg.rho.assign(
            &mut layouter,
            rho,
            e_1,
            f,
            g_0,
            e1_f_prime,
            z13_f,
            z14_e1_f_prime,
        )?;

        cfg.psi.assign(
            &mut layouter,
            psi,
            g_1,
            z1_g,
            h_0,
            h_1,
            g1_g2_prime,
            z13_g,
            z13_g1_g2_prime,
        )?;

        if let (
            Some(zsa_decomposition),
            Some(zsa_params),
            SpecificConfigForCircuit::Zsa(specific_config),
        ) = (
            zsa_decomposition,
            &zsa_params,
            &note_commit_chip.config.specific_config_for_circuit,
        ) {
            let j_0 = specific_config.j.assign(
                &mut layouter,
                zsa_decomposition.j,
                zsa_decomposition.j_0,
                zsa_decomposition.j_1,
            )?;

            let (h2_i_prime, z14_h2_i_prime) = pkd_asset_x_canonicity(
                &lookup_config,
                layouter.namespace(|| "x(asset) canonicity"),
                zsa_decomposition.h_2_zsa.clone(),
                zsa_decomposition.i.inner().cell_value(),
            )?;

            let z13_i = zs_zsa_suffix[1][13].clone();
            cfg.pk_d_asset.assign(
                &mut layouter,
                &zsa_params.asset,
                zsa_decomposition.h_2_zsa,
                zsa_decomposition.i,
                j_0,
                h2_i_prime,
                z13_i,
                z14_h2_i_prime,
            )?;
        }

        Ok(cm)
    }

    /// A canonicity check helper used in checking x(g_d), y(g_d), and y(pk_d).
    ///
    /// Specifications:
    /// - [`g_d` canonicity](https://p.z.cash/orchard-0.1:note-commit-canonicity-g_d?partial)
    /// - [`y` canonicity](https://p.z.cash/orchard-0.1:note-commit-canonicity-y?partial)
    fn canon_bitshift_130<Lookup: PallasLookupRangeCheck>(
        lookup_config: &Lookup,
        mut layouter: impl Layouter<pallas::Base>,
        a: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<CanonicityBounds, Error> {
        // element = `a (250 bits) || b_0 (4 bits) || b_1 (1 bit)`
        // - b_1 = 1 => b_0 = 0
        // - b_1 = 1 => a < t_P
        //     - 0 ≤ a < 2^130 (z_13 of SinsemillaHash(a))
        //     - 0 ≤ a + 2^130 - t_P < 2^130 (thirteen 10-bit lookups)

        // Decompose the low 130 bits of a_prime = a + 2^130 - t_P, and output
        // the running sum at the end of it. If a_prime < 2^130, the running sum
        // will be 0.
        let a_prime = {
            let two_pow_130 = Value::known(pallas::Base::from_u128(1u128 << 65).square());
            let t_p = Value::known(pallas::Base::from_u128(T_P));
            a.value() + two_pow_130 - t_p
        };
        let zs = lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 130 bits of (a + 2^130 - t_P)"),
            a_prime,
            13,
            false,
        )?;
        let a_prime = zs[0].clone();
        assert_eq!(zs.len(), 14); // [z_0, z_1, ..., z_13]

        Ok((a_prime, zs[13].clone()))
    }

    /// Check canonicity of `x(pk_d)` and `x(asset)` encoding.
    ///
    /// [Specification](https://p.z.cash/orchard-0.1:note-commit-canonicity-pk_d?partial).
    fn pkd_asset_x_canonicity<Lookup: PallasLookupRangeCheck>(
        lookup_config: &Lookup,
        mut layouter: impl Layouter<pallas::Base>,
        b_3: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        c: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<CanonicityBounds, Error> {
        // Example for `x(pk_d)`:
        // `x(pk_d)` = `b_3 (4 bits) || c (250 bits) || d_0 (1 bit)`
        // - d_0 = 1 => b_3 + 2^4 c < t_P
        //     - 0 ≤ b_3 + 2^4 c < 2^134
        //         - b_3 is part of the Sinsemilla message piece
        //           b = b_0 (4 bits) || b_1 (1 bit) || b_2 (1 bit) || b_3 (4 bits)
        //         - b_3 is individually constrained to be 4 bits.
        //         - z_13 of SinsemillaHash(c) == 0 constrains bits 4..=253 of pkd_x
        //           to 130 bits. z13_c is directly checked in the gate.
        //     - 0 ≤ b_3 + 2^4 c + 2^140 - t_P < 2^140 (14 ten-bit lookups)
        // For `x(asset)`, we have to replace `pk_d`, `b_3`, `c`, `d_0` by `asset`, `h_2_zsa`, `i`, `j_0`

        // Decompose the low 140 bits of b3_c_prime = b_3 + 2^4 c + 2^140 - t_P,
        // and output the running sum at the end of it.
        // If b3_c_prime < 2^140, the running sum will be 0.
        let b3_c_prime = {
            let two_pow_4 = Value::known(pallas::Base::from(1u64 << 4));
            let two_pow_140 = Value::known(pallas::Base::from_u128(1u128 << 70).square());
            let t_p = Value::known(pallas::Base::from_u128(T_P));
            b_3.inner().value() + (two_pow_4 * c.value()) + two_pow_140 - t_p
        };

        let zs = lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 140 bits of (b_3 + 2^4 c + 2^140 - t_P)"),
            b3_c_prime,
            14,
            false,
        )?;
        let b3_c_prime = zs[0].clone();
        assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_13, z_14]

        Ok((b3_c_prime, zs[14].clone()))
    }

    /// Check canonicity of `rho` encoding.
    ///
    /// [Specification](https://p.z.cash/orchard-0.1:note-commit-canonicity-rho?partial).
    fn rho_canonicity<Lookup: PallasLookupRangeCheck>(
        lookup_config: &Lookup,
        mut layouter: impl Layouter<pallas::Base>,
        e_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        f: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<CanonicityBounds, Error> {
        // `rho` = `e_1 (4 bits) || f (250 bits) || g_0 (1 bit)`
        // - g_0 = 1 => e_1 + 2^4 f < t_P
        // - 0 ≤ e_1 + 2^4 f < 2^134
        //     - e_1 is part of the Sinsemilla message piece
        //       e = e_0 (56 bits) || e_1 (4 bits)
        //     - e_1 is individually constrained to be 4 bits.
        //     - z_13 of SinsemillaHash(f) == 0 constrains bits 4..=253 of rho
        //       to 130 bits. z13_f == 0 is directly checked in the gate.
        // - 0 ≤ e_1 + 2^4 f + 2^140 - t_P < 2^140 (14 ten-bit lookups)

        let e1_f_prime = {
            let two_pow_4 = Value::known(pallas::Base::from(1u64 << 4));
            let two_pow_140 = Value::known(pallas::Base::from_u128(1u128 << 70).square());
            let t_p = Value::known(pallas::Base::from_u128(T_P));
            e_1.inner().value() + (two_pow_4 * f.value()) + two_pow_140 - t_p
        };

        // Decompose the low 140 bits of e1_f_prime = e_1 + 2^4 f + 2^140 - t_P,
        // and output the running sum at the end of it.
        // If e1_f_prime < 2^140, the running sum will be 0.
        let zs = lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 140 bits of (e_1 + 2^4 f + 2^140 - t_P)"),
            e1_f_prime,
            14,
            false,
        )?;
        let e1_f_prime = zs[0].clone();
        assert_eq!(zs.len(), 15); // [z_0, z_1, ..., z_13, z_14]

        Ok((e1_f_prime, zs[14].clone()))
    }

    /// Check canonicity of `psi` encoding.
    ///
    /// [Specification](https://p.z.cash/orchard-0.1:note-commit-canonicity-psi?partial).
    fn psi_canonicity<Lookup: PallasLookupRangeCheck>(
        lookup_config: &Lookup,
        mut layouter: impl Layouter<pallas::Base>,
        g_1: RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>,
        g_2: AssignedCell<pallas::Base, pallas::Base>,
    ) -> Result<CanonicityBounds, Error> {
        // `psi` = `g_1 (9 bits) || g_2 (240 bits) || h_0 (5 bits) || h_1 (1 bit)`
        // - h_1 = 1 => (h_0 = 0) ∧ (g_1 + 2^9 g_2 < t_P)
        // - 0 ≤ g_1 + 2^9 g_2 < 2^130
        //     - g_1 is individually constrained to be 9 bits
        //     - z_13 of SinsemillaHash(g) == 0 constrains bits 0..=248 of psi
        //       to 130 bits. z13_g == 0 is directly checked in the gate.
        // - 0 ≤ g_1 + (2^9)g_2 + 2^130 - t_P < 2^130 (13 ten-bit lookups)

        // Decompose the low 130 bits of g1_g2_prime = g_1 + (2^9)g_2 + 2^130 - t_P,
        // and output the running sum at the end of it.
        // If g1_g2_prime < 2^130, the running sum will be 0.
        let g1_g2_prime = {
            let two_pow_9 = Value::known(pallas::Base::from(1u64 << 9));
            let two_pow_130 = Value::known(pallas::Base::from_u128(1u128 << 65).square());
            let t_p = Value::known(pallas::Base::from_u128(T_P));
            g_1.inner().value() + (two_pow_9 * g_2.value()) + two_pow_130 - t_p
        };

        let zs = lookup_config.witness_check(
            layouter.namespace(|| "Decompose low 130 bits of (g_1 + (2^9)g_2 + 2^130 - t_P)"),
            g1_g2_prime,
            13,
            false,
        )?;
        let g1_g2_prime = zs[0].clone();
        assert_eq!(zs.len(), 14); // [z_0, z_1, ..., z_13]

        Ok((g1_g2_prime, zs[13].clone()))
    }

    /// Check canonicity of y-coordinate given its LSB as a value.
    /// Also, witness the LSB and return the witnessed cell.
    ///
    /// Specifications:
    /// - [`y` decomposition](https://p.z.cash/orchard-0.1:note-commit-decomposition-y?partial)
    /// - [`y` canonicity](https://p.z.cash/orchard-0.1:note-commit-canonicity-y?partial)
    fn y_canonicity<Lookup: PallasLookupRangeCheck>(
        lookup_config: &Lookup,
        y_canon: &YCanonicity,
        mut layouter: impl Layouter<pallas::Base>,
        y: AssignedCell<pallas::Base, pallas::Base>,
        lsb: RangeConstrained<pallas::Base, Value<pallas::Base>>,
    ) -> Result<RangeConstrained<pallas::Base, AssignedCell<pallas::Base, pallas::Base>>, Error>
    {
        // Decompose the field element
        //      y = LSB || k_0 || k_1 || k_2 || k_3
        //        = (bit 0) || (bits 1..=9) || (bits 10..=249) || (bits 250..=253) || (bit 254)

        // Range-constrain k_0 to be 9 bits.
        let k_0 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "k_0"),
            y.value(),
            1..10,
        )?;

        // k_1 will be constrained by the decomposition of j.
        let k_1 = RangeConstrained::bitrange_of(y.value(), 10..250);

        // Range-constrain k_2 to be 4 bits.
        let k_2 = RangeConstrained::witness_short(
            lookup_config,
            layouter.namespace(|| "k_2"),
            y.value(),
            250..254,
        )?;

        // k_3 will be boolean-constrained in the gate.
        let k_3 = RangeConstrained::bitrange_of(y.value(), 254..255);

        // Decompose j = LSB + (2)k_0 + (2^10)k_1 using 25 ten-bit lookups.
        let (j, z1_j, z13_j) = {
            let j = {
                let two = Value::known(pallas::Base::from(2));
                let two_pow_10 = Value::known(pallas::Base::from(1 << 10));
                lsb.inner().value() + two * k_0.inner().value() + two_pow_10 * k_1.inner().value()
            };
            let zs = lookup_config.witness_check(
                layouter.namespace(|| "Decompose j = LSB + (2)k_0 + (2^10)k_1"),
                j,
                25,
                true,
            )?;
            (zs[0].clone(), zs[1].clone(), zs[13].clone())
        };

        // Decompose j_prime = j + 2^130 - t_P using 13 ten-bit lookups.
        // We can reuse the canon_bitshift_130 logic here.
        let (j_prime, z13_j_prime) = canon_bitshift_130(
            lookup_config,
            layouter.namespace(|| "j_prime = j + 2^130 - t_P"),
            j.clone(),
        )?;

        y_canon.assign(
            &mut layouter,
            y,
            lsb,
            k_0,
            k_2,
            k_3,
            j,
            z1_j,
            z13_j,
            j_prime,
            z13_j_prime,
        )
    }
}

#[cfg(test)]
mod tests {
    use core::iter;

    use crate::{
        circuit::gadget::{assign_free_advice, assign_is_native_asset},
        circuit::note_commit::gadgets,
        circuit::note_commit::{NoteCommitChip, NoteCommitConfig},
        constants::{
            fixed_bases::NOTE_COMMITMENT_PERSONALIZATION, OrchardCommitDomains, OrchardFixedBases,
            OrchardHashDomains, L_ORCHARD_BASE, L_VALUE, T_Q,
        },
        note::{commitment::NoteCommitTrapdoor, AssetBase, NoteCommitment},
        value::NoteValue,
    };
    use halo2_gadgets::{
        ecc::{
            chip::{EccChip, EccConfig},
            NonIdentityPoint, ScalarFixed,
        },
        sinsemilla::{chip::SinsemillaChip, primitives::CommitDomain},
        utilities::{
            cond_swap::{CondSwapChip, CondSwapConfig},
            lookup_range_check::{
                LookupRangeCheck, LookupRangeCheckConfig, PallasLookupRangeCheck4_5BConfig,
                PallasLookupRangeCheckConfig,
            },
        },
    };

    use ff::{Field, PrimeField, PrimeFieldBits};
    use group::{Curve, Group, GroupEncoding};
    use halo2_proofs::{
        circuit::{Layouter, SimpleFloorPlanner, Value},
        dev::MockProver,
        plonk::{Circuit, ConstraintSystem, Error},
    };
    use pasta_curves::{arithmetic::CurveAffine, pallas, EpAffine};

    use crate::circuit::note_commit::ZsaNoteCommitParams;
    use rand::{rngs::OsRng, RngCore};

    #[test]
    fn note_commit_vanilla() {
        #[derive(Default)]
        struct MyCircuit {
            gd_x: Value<pallas::Base>,
            gd_y_lsb: Value<pallas::Base>,
            pkd_x: Value<pallas::Base>,
            pkd_y_lsb: Value<pallas::Base>,
            rho: Value<pallas::Base>,
            psi: Value<pallas::Base>,
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = (
                NoteCommitConfig<PallasLookupRangeCheckConfig>,
                EccConfig<OrchardFixedBases>,
            );
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

                // Shared fixed column for loading constants.
                let constants = meta.fixed_column();
                meta.enable_constant(constants);

                for advice in advices.iter() {
                    meta.enable_equality(*advice);
                }

                let table_idx = meta.lookup_table_column();
                let lookup = (
                    table_idx,
                    meta.lookup_table_column(),
                    meta.lookup_table_column(),
                );
                let lagrange_coeffs = [
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                ];

                let range_check = LookupRangeCheckConfig::configure(meta, advices[9], table_idx);
                let sinsemilla_config = SinsemillaChip::<
                    OrchardHashDomains,
                    OrchardCommitDomains,
                    OrchardFixedBases,
                >::configure(
                    meta,
                    advices[..5].try_into().unwrap(),
                    advices[2],
                    lagrange_coeffs[0],
                    lookup,
                    range_check,
                    false,
                );
                let note_commit_config =
                    NoteCommitChip::configure(meta, advices, sinsemilla_config, false);

                let ecc_config = EccChip::<OrchardFixedBases>::configure(
                    meta,
                    advices,
                    lagrange_coeffs,
                    range_check,
                );

                (note_commit_config, ecc_config)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                let (note_commit_config, ecc_config) = config;

                // Load the Sinsemilla generator lookup table used by the whole circuit.
                SinsemillaChip::<
                    OrchardHashDomains,
                    OrchardCommitDomains,
                    OrchardFixedBases,
                >::load(note_commit_config.sinsemilla_config.clone(), &mut layouter)?;

                // Construct a Sinsemilla chip
                let sinsemilla_chip =
                    SinsemillaChip::construct(note_commit_config.sinsemilla_config.clone());

                // Construct an ECC chip
                let ecc_chip = EccChip::construct(ecc_config);

                // Construct a NoteCommit chip
                let note_commit_chip = NoteCommitChip::construct(note_commit_config.clone());

                // Witness g_d
                let g_d = {
                    let g_d = self.gd_x.zip(self.gd_y_lsb).map(|(x, y_lsb)| {
                        // Calculate y = (x^3 + 5).sqrt()
                        let mut y = (x.square() * x + pallas::Affine::b()).sqrt().unwrap();
                        if bool::from(y.is_odd() ^ y_lsb.is_odd()) {
                            y = -y;
                        }
                        pallas::Affine::from_xy(x, y).unwrap()
                    });

                    NonIdentityPoint::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "witness g_d"),
                        g_d,
                    )?
                };

                // Witness pk_d
                let pk_d = {
                    let pk_d = self.pkd_x.zip(self.pkd_y_lsb).map(|(x, y_lsb)| {
                        // Calculate y = (x^3 + 5).sqrt()
                        let mut y = (x.square() * x + pallas::Affine::b()).sqrt().unwrap();
                        if bool::from(y.is_odd() ^ y_lsb.is_odd()) {
                            y = -y;
                        }
                        pallas::Affine::from_xy(x, y).unwrap()
                    });

                    NonIdentityPoint::new(
                        ecc_chip.clone(),
                        layouter.namespace(|| "witness pk_d"),
                        pk_d,
                    )?
                };

                // Witness a random non-negative u64 note value
                // A note value cannot be negative.
                let value = {
                    let mut rng = OsRng;
                    NoteValue::from_raw(rng.next_u64())
                };
                let value_var = {
                    assign_free_advice(
                        layouter.namespace(|| "witness value"),
                        note_commit_config.advices[0],
                        Value::known(value),
                    )?
                };

                // Witness rho
                let rho = assign_free_advice(
                    layouter.namespace(|| "witness rho"),
                    note_commit_config.advices[0],
                    self.rho,
                )?;

                // Witness psi
                let psi = assign_free_advice(
                    layouter.namespace(|| "witness psi"),
                    note_commit_config.advices[0],
                    self.psi,
                )?;

                let rcm = pallas::Scalar::random(OsRng);
                let rcm_gadget = ScalarFixed::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "rcm"),
                    Value::known(rcm),
                )?;

                let cm = gadgets::note_commit(
                    layouter.namespace(|| "Hash NoteCommit pieces"),
                    sinsemilla_chip,
                    ecc_chip.clone(),
                    note_commit_chip,
                    g_d.inner(),
                    pk_d.inner(),
                    value_var,
                    rho,
                    psi,
                    rcm_gadget,
                    None,
                )?;
                let expected_cm = {
                    let domain = CommitDomain::new(NOTE_COMMITMENT_PERSONALIZATION);
                    // Hash g★_d || pk★_d || i2lebsp_{64}(v) || rho || psi
                    let lsb = |y_lsb: pallas::Base| y_lsb == pallas::Base::one();
                    let point = self
                        .gd_x
                        .zip(self.gd_y_lsb)
                        .zip(self.pkd_x.zip(self.pkd_y_lsb))
                        .zip(self.rho.zip(self.psi))
                        .map(|(((gd_x, gd_y_lsb), (pkd_x, pkd_y_lsb)), (rho, psi))| {
                            domain
                                .commit(
                                    iter::empty()
                                        .chain(
                                            gd_x.to_le_bits().iter().by_vals().take(L_ORCHARD_BASE),
                                        )
                                        .chain(Some(lsb(gd_y_lsb)))
                                        .chain(
                                            pkd_x
                                                .to_le_bits()
                                                .iter()
                                                .by_vals()
                                                .take(L_ORCHARD_BASE),
                                        )
                                        .chain(Some(lsb(pkd_y_lsb)))
                                        .chain(value.to_le_bits().iter().by_vals().take(L_VALUE))
                                        .chain(
                                            rho.to_le_bits().iter().by_vals().take(L_ORCHARD_BASE),
                                        )
                                        .chain(
                                            psi.to_le_bits().iter().by_vals().take(L_ORCHARD_BASE),
                                        ),
                                    &rcm,
                                )
                                .unwrap()
                                .to_affine()
                        });
                    NonIdentityPoint::new(ecc_chip, layouter.namespace(|| "witness cm"), point)?
                };
                cm.constrain_equal(layouter.namespace(|| "cm == expected cm"), &expected_cm)
            }
        }

        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        // Test different values of `ak`, `nk`
        let circuits = [
            // `gd_x` = -1, `pkd_x` = -1 (these have to be x-coordinates of curve points)
            // `rho` = 0, `psi` = 0
            MyCircuit {
                gd_x: Value::known(-pallas::Base::one()),
                gd_y_lsb: Value::known(pallas::Base::one()),
                pkd_x: Value::known(-pallas::Base::one()),
                pkd_y_lsb: Value::known(pallas::Base::one()),
                rho: Value::known(pallas::Base::zero()),
                psi: Value::known(pallas::Base::zero()),
            },
            // `rho` = T_Q - 1, `psi` = T_Q - 1
            MyCircuit {
                gd_x: Value::known(-pallas::Base::one()),
                gd_y_lsb: Value::known(pallas::Base::zero()),
                pkd_x: Value::known(-pallas::Base::one()),
                pkd_y_lsb: Value::known(pallas::Base::zero()),
                rho: Value::known(pallas::Base::from_u128(T_Q - 1)),
                psi: Value::known(pallas::Base::from_u128(T_Q - 1)),
            },
            // `rho` = T_Q, `psi` = T_Q
            MyCircuit {
                gd_x: Value::known(-pallas::Base::one()),
                gd_y_lsb: Value::known(pallas::Base::one()),
                pkd_x: Value::known(-pallas::Base::one()),
                pkd_y_lsb: Value::known(pallas::Base::zero()),
                rho: Value::known(pallas::Base::from_u128(T_Q)),
                psi: Value::known(pallas::Base::from_u128(T_Q)),
            },
            // `rho` = 2^127 - 1, `psi` = 2^127 - 1
            MyCircuit {
                gd_x: Value::known(-pallas::Base::one()),
                gd_y_lsb: Value::known(pallas::Base::zero()),
                pkd_x: Value::known(-pallas::Base::one()),
                pkd_y_lsb: Value::known(pallas::Base::one()),
                rho: Value::known(pallas::Base::from_u128((1 << 127) - 1)),
                psi: Value::known(pallas::Base::from_u128((1 << 127) - 1)),
            },
            // `rho` = 2^127, `psi` = 2^127
            MyCircuit {
                gd_x: Value::known(-pallas::Base::one()),
                gd_y_lsb: Value::known(pallas::Base::zero()),
                pkd_x: Value::known(-pallas::Base::one()),
                pkd_y_lsb: Value::known(pallas::Base::zero()),
                rho: Value::known(pallas::Base::from_u128(1 << 127)),
                psi: Value::known(pallas::Base::from_u128(1 << 127)),
            },
            // `rho` = 2^254 - 1, `psi` = 2^254 - 1
            MyCircuit {
                gd_x: Value::known(-pallas::Base::one()),
                gd_y_lsb: Value::known(pallas::Base::one()),
                pkd_x: Value::known(-pallas::Base::one()),
                pkd_y_lsb: Value::known(pallas::Base::one()),
                rho: Value::known(two_pow_254 - pallas::Base::one()),
                psi: Value::known(two_pow_254 - pallas::Base::one()),
            },
            // `rho` = 2^254, `psi` = 2^254
            MyCircuit {
                gd_x: Value::known(-pallas::Base::one()),
                gd_y_lsb: Value::known(pallas::Base::one()),
                pkd_x: Value::known(-pallas::Base::one()),
                pkd_y_lsb: Value::known(pallas::Base::zero()),
                rho: Value::known(two_pow_254),
                psi: Value::known(two_pow_254),
            },
        ];

        for circuit in circuits.iter() {
            let prover = MockProver::<pallas::Base>::run(11, circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }

    #[test]
    fn note_commit_zsa() {
        #[derive(Default)]
        struct MyCircuit {
            g_d: Value<EpAffine>,
            pk_d: Value<EpAffine>,
            rho: Value<pallas::Base>,
            psi: Value<pallas::Base>,
            asset: Value<AssetBase>,
        }

        impl Circuit<pallas::Base> for MyCircuit {
            type Config = (
                NoteCommitConfig<PallasLookupRangeCheck4_5BConfig>,
                EccConfig<OrchardFixedBases, PallasLookupRangeCheck4_5BConfig>,
                CondSwapConfig,
            );
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

                // Shared fixed column for loading constants.
                let constants = meta.fixed_column();
                meta.enable_constant(constants);

                for advice in advices.iter() {
                    meta.enable_equality(*advice);
                }

                let table_idx = meta.lookup_table_column();
                let table_range_check_tag = meta.lookup_table_column();
                let lookup = (
                    table_idx,
                    meta.lookup_table_column(),
                    meta.lookup_table_column(),
                );
                let lagrange_coeffs = [
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                    meta.fixed_column(),
                ];

                let range_check = PallasLookupRangeCheck4_5BConfig::configure_with_tag(
                    meta,
                    advices[9],
                    table_idx,
                    table_range_check_tag,
                );
                let sinsemilla_config = SinsemillaChip::<
                    OrchardHashDomains,
                    OrchardCommitDomains,
                    OrchardFixedBases,
                    PallasLookupRangeCheck4_5BConfig,
                >::configure(
                    meta,
                    advices[..5].try_into().unwrap(),
                    advices[2],
                    lagrange_coeffs[0],
                    lookup,
                    range_check,
                    true,
                );
                let note_commit_config =
                    NoteCommitChip::configure(meta, advices, sinsemilla_config, true);

                let ecc_config =
                    EccChip::<OrchardFixedBases, PallasLookupRangeCheck4_5BConfig>::configure(
                        meta,
                        advices,
                        lagrange_coeffs,
                        range_check,
                    );

                let cond_swap_config =
                    CondSwapChip::configure(meta, advices[0..5].try_into().unwrap());

                (note_commit_config, ecc_config, cond_swap_config)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<pallas::Base>,
            ) -> Result<(), Error> {
                let (note_commit_config, ecc_config, cond_swap_config) = config;

                // Load the Sinsemilla generator lookup table used by the whole circuit.
                SinsemillaChip::<
                    OrchardHashDomains,
                    OrchardCommitDomains,
                    OrchardFixedBases,
                    PallasLookupRangeCheck4_5BConfig,
                >::load(
                    note_commit_config.sinsemilla_config.clone(), &mut layouter
                )?;

                // Construct a Sinsemilla chip
                let sinsemilla_chip =
                    SinsemillaChip::construct(note_commit_config.sinsemilla_config.clone());

                // Construct an ECC chip
                let ecc_chip = EccChip::construct(ecc_config);

                // Construct a NoteCommit chip
                let note_commit_chip = NoteCommitChip::construct(note_commit_config.clone());

                // Construct a CondSwap chip
                let cond_swap_chip = CondSwapChip::construct(cond_swap_config);

                // Witness g_d
                let g_d = NonIdentityPoint::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness g_d"),
                    self.g_d,
                )?;

                // Witness pk_d
                let pk_d = NonIdentityPoint::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness pk_d"),
                    self.pk_d,
                )?;

                // Witness a random non-negative u64 note value
                // A note value cannot be negative.
                let value = {
                    let mut rng = OsRng;
                    NoteValue::from_raw(rng.next_u64())
                };
                let value_var = {
                    assign_free_advice(
                        layouter.namespace(|| "witness value"),
                        note_commit_config.advices[0],
                        Value::known(value),
                    )?
                };

                // Witness rho
                let rho = assign_free_advice(
                    layouter.namespace(|| "witness rho"),
                    note_commit_config.advices[0],
                    self.rho,
                )?;

                // Witness psi
                let psi = assign_free_advice(
                    layouter.namespace(|| "witness psi"),
                    note_commit_config.advices[0],
                    self.psi,
                )?;

                let rcm = pallas::Scalar::random(OsRng);
                let rcm_gadget = ScalarFixed::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "rcm"),
                    Value::known(rcm),
                )?;

                let asset = NonIdentityPoint::new(
                    ecc_chip.clone(),
                    layouter.namespace(|| "witness asset"),
                    self.asset.map(|asset| asset.cv_base().to_affine()),
                )?;

                let is_native_asset = assign_is_native_asset(
                    layouter.namespace(|| "witness is_native_asset"),
                    note_commit_config.advices[0],
                    self.asset,
                )?;
                let cm = gadgets::note_commit(
                    layouter.namespace(|| "Hash NoteCommit pieces"),
                    sinsemilla_chip,
                    ecc_chip.clone(),
                    note_commit_chip,
                    g_d.inner(),
                    pk_d.inner(),
                    value_var,
                    rho,
                    psi,
                    rcm_gadget,
                    Some(ZsaNoteCommitParams {
                        cond_swap_chip,
                        asset: asset.inner().clone(),
                        is_native_asset,
                    }),
                )?;
                let expected_cm = {
                    // Hash g★_d || pk★_d || i2lebsp_{64}(v) || rho || psi
                    let point = self
                        .g_d
                        .zip(self.pk_d)
                        .zip(self.rho.zip(self.psi))
                        .zip(self.asset)
                        .map(|(((g_d, pk_d), (rho, psi)), asset)| {
                            NoteCommitment::derive(
                                g_d.to_bytes(),
                                pk_d.to_bytes(),
                                value,
                                asset,
                                rho,
                                psi,
                                NoteCommitTrapdoor(rcm),
                            )
                            .unwrap()
                            .inner()
                            .to_affine()
                        });
                    NonIdentityPoint::new(ecc_chip, layouter.namespace(|| "witness cm"), point)?
                };
                cm.constrain_equal(layouter.namespace(|| "cm == expected cm"), &expected_cm)
            }
        }

        fn affine_point_from_coordinates(x_coord: pallas::Base, y_lsb: pallas::Base) -> EpAffine {
            // Calculate y = (x^3 + 5).sqrt()
            let mut y = (x_coord.square() * x_coord + pallas::Affine::b())
                .sqrt()
                .unwrap();
            if bool::from(y.is_odd() ^ y_lsb.is_odd()) {
                y = -y;
            }
            pallas::Affine::from_xy(x_coord, y).unwrap()
        }

        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        let mut rng = OsRng;
        let random_asset = AssetBase::random();

        // Test different values of `ak`, `nk`
        let mut circuits = vec![];
        for asset in [random_asset, AssetBase::native()] {
            // `gd_x` = -1, `pkd_x` = -1 (these have to be x-coordinates of curve points)
            // `rho` = 0, `psi` = 0
            circuits.push(MyCircuit {
                g_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::one(),
                )),
                pk_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::one(),
                )),
                rho: Value::known(pallas::Base::zero()),
                psi: Value::known(pallas::Base::zero()),
                asset: Value::known(asset),
            });
            // `rho` = T_Q - 1, `psi` = T_Q - 1
            circuits.push(MyCircuit {
                g_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::zero(),
                )),
                pk_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::zero(),
                )),
                rho: Value::known(pallas::Base::from_u128(T_Q - 1)),
                psi: Value::known(pallas::Base::from_u128(T_Q - 1)),
                asset: Value::known(asset),
            });
            // `rho` = T_Q, `psi` = T_Q
            circuits.push(MyCircuit {
                g_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::one(),
                )),
                pk_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::zero(),
                )),
                rho: Value::known(pallas::Base::from_u128(T_Q)),
                psi: Value::known(pallas::Base::from_u128(T_Q)),
                asset: Value::known(asset),
            });
            // `rho` = 2^127 - 1, `psi` = 2^127 - 1
            circuits.push(MyCircuit {
                g_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::zero(),
                )),
                pk_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::one(),
                )),
                rho: Value::known(pallas::Base::from_u128((1 << 127) - 1)),
                psi: Value::known(pallas::Base::from_u128((1 << 127) - 1)),
                asset: Value::known(asset),
            });
            // `rho` = 2^127, `psi` = 2^127
            circuits.push(MyCircuit {
                g_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::zero(),
                )),
                pk_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::zero(),
                )),
                rho: Value::known(pallas::Base::from_u128(1 << 127)),
                psi: Value::known(pallas::Base::from_u128(1 << 127)),
                asset: Value::known(asset),
            });
            // `rho` = 2^254 - 1, `psi` = 2^254 - 1
            circuits.push(MyCircuit {
                g_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::one(),
                )),
                pk_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::one(),
                )),
                rho: Value::known(two_pow_254 - pallas::Base::one()),
                psi: Value::known(two_pow_254 - pallas::Base::one()),
                asset: Value::known(asset),
            });
            // `rho` = 2^254, `psi` = 2^254
            circuits.push(MyCircuit {
                g_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::one(),
                )),
                pk_d: Value::known(affine_point_from_coordinates(
                    -pallas::Base::one(),
                    pallas::Base::zero(),
                )),
                rho: Value::known(two_pow_254),
                psi: Value::known(two_pow_254),
                asset: Value::known(asset),
            });
            // Random values
            circuits.push(MyCircuit {
                g_d: Value::known(pallas::Point::random(rng).to_affine()),
                pk_d: Value::known(pallas::Point::random(rng).to_affine()),
                rho: Value::known(pallas::Base::random(&mut rng)),
                psi: Value::known(pallas::Base::random(&mut rng)),
                asset: Value::known(asset),
            });
        }

        for circuit in circuits.iter() {
            let prover = MockProver::<pallas::Base>::run(11, circuit, vec![]).unwrap();
            assert_eq!(prover.verify(), Ok(()));
        }
    }
}
