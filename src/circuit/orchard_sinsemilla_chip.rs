//! Defines the `OrchardSinsemillaChip` trait to abstract over `SinsemillaChip` and `SinsemillaChip` types.
//! Used to generalize the `commit_ivk` function.

use pasta_curves::pallas;

use halo2_proofs::circuit::Chip;

use halo2_gadgets::{
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        primitives as sinsemilla, SinsemillaInstructions,
    },
    utilities::lookup_range_check::PallasLookupRangeCheck,
};

use crate::constants::{OrchardCommitDomains, OrchardFixedBases, OrchardHashDomains};

type BaseSinsemillaChip =
    SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases>;

pub(super) trait OrchardSinsemillaChip<Lookup: PallasLookupRangeCheck>:
    SinsemillaInstructions<
        pallas::Affine,
        { sinsemilla::K },
        { sinsemilla::C },
        CellValue = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::CellValue,
        Message = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::Message,
        MessagePiece = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::MessagePiece,
        RunningSum = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::RunningSum,
        X = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::X,
        NonIdentityPoint = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::NonIdentityPoint,
        FixedPoints = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::FixedPoints,
        HashDomains = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::HashDomains,
        CommitDomains = <BaseSinsemillaChip as SinsemillaInstructions<
            pallas::Affine,
            { sinsemilla::K },
            { sinsemilla::C },
        >>::CommitDomains,
    > + Chip<
        pallas::Base,
        Config = SinsemillaConfig<
            OrchardHashDomains,
            OrchardCommitDomains,
            OrchardFixedBases,
            Lookup,
        >,
    > + Clone
    + std::fmt::Debug
    + Eq
{
}

impl<Lookup: PallasLookupRangeCheck> OrchardSinsemillaChip<Lookup>
    for SinsemillaChip<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>
{
}
