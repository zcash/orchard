//! The Orchard Action circuit implementation.
//!
//! This module defines the common structures, traits and implementations for the
//! Orchard Action circuit, supporting both the standard ("Vanilla") and ZSA variations.

use alloc::vec::Vec;

use group::{Curve, GroupEncoding};
use halo2_proofs::{
    circuit::Value,
    plonk::{
        self, Advice, BatchVerifier, Column, Instance as InstanceColumn, Selector, SingleVerifier,
    },
    transcript::{Blake2bRead, Blake2bWrite},
};
use pasta_curves::{arithmetic::CurveAffine, pallas, vesta};
use rand::RngCore;

use crate::{
    builder::SpendInfo,
    bundle::Flags,
    circuit::{
        commit_ivk::{CommitIvkChip, CommitIvkConfig},
        gadget::add_chip::{AddChip, AddConfig},
        note_commit::{NoteCommitChip, NoteCommitConfig},
    },
    circuit_version::OrchardCircuitVersion,
    constants::{OrchardCommitDomains, OrchardFixedBases, OrchardHashDomains},
    note::{nullifier::Nullifier, ExtractedNoteCommitment, Note, Rho},
    primitives::redpallas::{SpendAuth, VerificationKey},
    tree::Anchor,
    value::{ValueCommitTrapdoor, ValueCommitment},
};
use halo2_gadgets::{
    ecc::{
        chip::{EccChip, EccConfig},
        CircuitVersion, NonIdentityPoint,
    },
    poseidon::{primitives as poseidon, Pow5Chip as PoseidonChip, Pow5Config as PoseidonConfig},
    sinsemilla::{
        chip::{SinsemillaChip, SinsemillaConfig},
        merkle::chip::{MerkleChip, MerkleConfig},
    },
    utilities::lookup_range_check::PallasLookupRangeCheck,
};

mod circuit_vanilla;
mod circuit_zsa;

use circuit_vanilla::CircuitVanilla;
use circuit_zsa::{AdditionalZsaWitnesses, CircuitZsa};

#[cfg(not(feature = "unstable-voting-circuits"))]
pub(in crate::circuit) mod commit_ivk;
#[cfg(feature = "unstable-voting-circuits")]
pub mod commit_ivk;
#[cfg(not(feature = "unstable-voting-circuits"))]
pub(in crate::circuit) mod derive_nullifier;
#[cfg(feature = "unstable-voting-circuits")]
pub mod derive_nullifier;
pub mod gadget;
#[cfg(not(feature = "unstable-voting-circuits"))]
pub(in crate::circuit) mod note_commit;
#[cfg(feature = "unstable-voting-circuits")]
pub mod note_commit;
pub(in crate::circuit) mod value_commit_orchard;

pub use crate::Proof;

/// Size of the Orchard circuit.
const K: u32 = 11;

// Absolute offsets for public inputs.
const ANCHOR: usize = 0;
const CV_NET_X: usize = 1;
const CV_NET_Y: usize = 2;
const NF_OLD: usize = 3;
const RK_X: usize = 4;
const RK_Y: usize = 5;
const CMX: usize = 6;
const ENABLE_SPEND: usize = 7;
const ENABLE_OUTPUT: usize = 8;
const DISABLE_CROSS_ADDRESS: usize = 9;
const ENABLE_ZSA: usize = 10;

/// Configuration needed to use the Orchard Action circuit.
#[derive(Clone, Debug)]
pub struct Config<Lookup: PallasLookupRangeCheck> {
    primary: Column<InstanceColumn>,
    q_orchard: Selector,
    advices: [Column<Advice>; 10],
    add_config: AddConfig,
    ecc_config: EccConfig<OrchardFixedBases, Lookup>,
    poseidon_config: PoseidonConfig<pallas::Base, 3, 2>,
    merkle_config_1:
        MerkleConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
    merkle_config_2:
        MerkleConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
    sinsemilla_config_1:
        SinsemillaConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
    sinsemilla_config_2:
        SinsemillaConfig<OrchardHashDomains, OrchardCommitDomains, OrchardFixedBases, Lookup>,
    commit_ivk_config: CommitIvkConfig,
    old_note_commit_config: NoteCommitConfig<Lookup>,
    new_note_commit_config: NoteCommitConfig<Lookup>,
}

impl OrchardCircuitVersion {
    /// Whether this circuit version enforces the `disableCrossAddress` public input.
    ///
    /// Statements with `disableCrossAddress = 1` can be proven and verified only with
    /// keys for a circuit version that constrains the flag. [`PostNu6_3`] constrains it;
    /// older circuit versions leave it unconstrained, so they cannot enforce — and must
    /// not be asked to attest to — the restriction.
    ///
    /// [`PostNu6_3`]: OrchardCircuitVersion::PostNu6_3
    pub fn supports_cross_address_restriction(self) -> bool {
        match self {
            OrchardCircuitVersion::InsecurePreNu6_2 | OrchardCircuitVersion::FixedPostNu6_2 => {
                false
            }
            OrchardCircuitVersion::PostNu6_3 | OrchardCircuitVersion::ZSA => true,
        }
    }

    /// The corresponding `halo2_gadgets` variable-base scalar-mul circuit version.
    fn halo2_version(self) -> CircuitVersion {
        match self {
            OrchardCircuitVersion::InsecurePreNu6_2 => CircuitVersion::InsecureUnanchoredBase,
            OrchardCircuitVersion::FixedPostNu6_2
            | OrchardCircuitVersion::PostNu6_3
            | OrchardCircuitVersion::ZSA => CircuitVersion::AnchoredBase,
        }
    }

    pub(crate) fn is_zsa(self) -> bool {
        self == OrchardCircuitVersion::ZSA
    }
}

/// The Orchard Action circuit.
///
/// Carries the private witnesses of a single action for any [`OrchardCircuitVersion`].
/// The ZSA-specific witnesses are present only for [`OrchardCircuitVersion::ZSA`];
/// the Vanilla circuit versions leave them absent and do not prove them.
#[derive(Clone, Debug)]
pub struct Circuit {
    pub(crate) common_witnesses: CircuitVanilla,
    pub(crate) additional_zsa_witnesses: Option<AdditionalZsaWitnesses>,
}

impl Circuit {
    /// Returns the [`OrchardCircuitVersion`] this circuit instance is configured for.
    fn circuit_version(&self) -> OrchardCircuitVersion {
        self.common_witnesses.circuit_version
    }

    /// Returns the witnesses proved by the Vanilla circuit versions.
    ///
    /// # Errors
    ///
    /// Returns [`plonk::Error::Synthesis`] if `additional_zsa_witnesses` is present: the
    /// Vanilla circuit versions must never be asked to prove ZSA-specific witnesses.
    fn to_vanilla(&self) -> Result<CircuitVanilla, plonk::Error> {
        if self.additional_zsa_witnesses.is_some() {
            return Err(plonk::Error::Synthesis);
        }
        Ok(self.common_witnesses.clone())
    }

    /// Returns the witnesses proved by the ZSA circuit version.
    ///
    /// # Errors
    ///
    /// Returns [`plonk::Error::Synthesis`] if `additional_zsa_witnesses` is absent: the ZSA
    /// circuit version cannot be proved without the ZSA-specific witnesses.
    fn to_zsa(&self) -> Result<CircuitZsa, plonk::Error> {
        Ok(CircuitZsa {
            common_witnesses: self.common_witnesses.clone(),
            additional_zsa_witnesses: self
                .additional_zsa_witnesses
                .clone()
                .ok_or(plonk::Error::Synthesis)?,
        })
    }

    /// This constructor is public to enable creation of custom builders.
    /// If you are not creating a custom builder, use [`Builder`] to compose
    /// and authorize a transaction.
    ///
    /// Constructs a `Circuit` for the given `circuit_version` from the following components:
    /// - `spend`: [`SpendInfo`] of the note spent in scope of the action
    /// - `output_note`: a note created in scope of the action
    /// - `alpha`: a scalar used for randomization of the action spend validating key
    /// - `rcv`: trapdoor for the action value commitment
    ///
    /// Returns `None` if the `rho` of the `output_note` is not equal
    /// to the nullifier of the spent note.
    ///
    /// [`SpendInfo`]: crate::builder::SpendInfo
    /// [`Builder`]: crate::builder::Builder
    pub fn from_action_context(
        spend: SpendInfo,
        output_note: Note,
        alpha: pallas::Scalar,
        rcv: ValueCommitTrapdoor,
        circuit_version: OrchardCircuitVersion,
    ) -> Option<Circuit> {
        (Rho::from_nf_old(spend.note.nullifier(&spend.fvk)) == output_note.rho()).then(|| {
            Self::from_action_context_unchecked(spend, output_note, alpha, rcv, circuit_version)
        })
    }

    /// # Panics
    ///
    /// Panics for a Vanilla `circuit_version` if the spent note's asset is not zatoshi,
    /// or if `spend.split_flag` is true: those statements exist only in the ZSA circuit,
    /// so a Vanilla version cannot attest to them.
    ///
    /// Panics for any `circuit_version` if `spend.split_flag` disagrees with whether
    /// `spend.note` carries a split seed: the note-derived public nullifier and the
    /// `split_flag` witness would then describe different actions.
    pub(crate) fn from_action_context_unchecked(
        spend: SpendInfo,
        output_note: Note,
        alpha: pallas::Scalar,
        rcv: ValueCommitTrapdoor,
        circuit_version: OrchardCircuitVersion,
    ) -> Circuit {
        if !circuit_version.is_zsa() {
            assert!(
                bool::from(spend.note.asset().is_zatoshi()),
                "asset must be zatoshi in OrchardVanilla circuit"
            );
            assert!(
                !spend.split_flag,
                "split_flag must be false in OrchardVanilla circuit"
            );
        }

        assert_eq!(
            spend.split_flag,
            bool::from(spend.note.rseed_split_note().is_some()),
            "split_flag must match the presence of the note's split seed"
        );

        let sender_address = spend.note.recipient();
        let rho_old = spend.note.rho();
        let psi_old = spend.note.psi();
        let rcm_old = spend.note.rcm();
        // Unwitnessed spends (a deferred-anchor bundle, ZIP 374) exist only in bundles
        // that refuse in-memory building, and no public constructor produces one, so a
        // spend that reaches circuit construction always carries its Merkle path.
        let merkle_path = spend
            .merkle_path
            .expect("a spend used as a circuit witness carries a Merkle path");

        let psi_new = output_note.psi();
        let rcm_new = output_note.rcm();

        let common_witnesses = CircuitVanilla {
            path: Value::known(merkle_path.auth_path()),
            pos: Value::known(merkle_path.position()),
            g_d_old: Value::known(sender_address.g_d()),
            pk_d_old: Value::known(*sender_address.pk_d()),
            v_old: Value::known(spend.note.value()),
            rho_old: Value::known(rho_old),
            psi_old: Value::known(psi_old),
            rcm_old: Value::known(rcm_old),
            cm_old: Value::known(spend.note.commitment()),
            alpha: Value::known(alpha),
            ak: Value::known(spend.fvk.clone().into()),
            nk: Value::known(*spend.fvk.nk()),
            rivk: Value::known(spend.fvk.rivk(spend.scope)),
            g_d_new: Value::known(output_note.recipient().g_d()),
            pk_d_new: Value::known(*output_note.recipient().pk_d()),
            v_new: Value::known(output_note.value()),
            psi_new: Value::known(psi_new),
            rcm_new: Value::known(rcm_new),
            rcv: Value::known(rcv),
            circuit_version,
        };

        let additional_zsa_witnesses = if circuit_version.is_zsa() {
            Some(AdditionalZsaWitnesses {
                psi_nf: Value::known(spend.note.psi_nf()),
                asset: Value::known(spend.note.asset()),
                split_flag: Value::known(spend.split_flag),
            })
        } else {
            None
        };

        Circuit {
            common_witnesses,
            additional_zsa_witnesses,
        }
    }
}

/// Configures the Orchard Action circuit in the given constraint system.
///
/// Shared by both circuit variations; `is_zsa` selects both the `q_orchard` gate
/// ([`circuit_vanilla::configure_vanilla_orchard_gate`] or
/// [`circuit_zsa::configure_zsa_orchard_gate`]) and the ZSA-specific configuration
/// of the Sinsemilla and NoteCommit chips.
fn configure_circuit<Lookup: PallasLookupRangeCheck>(
    meta: &mut plonk::ConstraintSystem<pallas::Base>,
    is_zsa: bool,
) -> Config<Lookup> {
    // Advice columns used in the Orchard circuit.
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

    let q_orchard = meta.selector();
    if is_zsa {
        circuit_zsa::configure_zsa_orchard_gate(meta, advices, q_orchard);
    } else {
        circuit_vanilla::configure_vanilla_orchard_gate(meta, advices, q_orchard);
    }

    // Addition of two field elements.
    let add_config = AddChip::configure(meta, advices[7], advices[8], advices[6]);

    // Fixed columns for the Sinsemilla generator lookup table
    let table_idx = meta.lookup_table_column();
    let lookup = (
        table_idx,
        meta.lookup_table_column(),
        meta.lookup_table_column(),
    );

    // Instance column used for public inputs
    let primary = meta.instance_column();
    meta.enable_equality(primary);

    // Permutation over all advice columns.
    for advice in advices.iter() {
        meta.enable_equality(*advice);
    }

    // Poseidon requires four advice columns, while ECC incomplete addition requires
    // six, so we could choose to configure them in parallel. However, we only use a
    // single Poseidon invocation, and we have the rows to accommodate it serially.
    // Instead, we reduce the proof size by sharing fixed columns between the ECC and
    // Poseidon chips.
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
    let rc_a = lagrange_coeffs[2..5].try_into().unwrap();
    let rc_b = lagrange_coeffs[5..8].try_into().unwrap();

    // Also use the first Lagrange coefficient column for loading global constants.
    // It's free real estate :)
    meta.enable_constant(lagrange_coeffs[0]);

    // We have a lot of free space in the right-most advice columns; use one of them
    // for all of our range checks.
    // The ZSA lookup argument also allocates its extra tag table column here.
    let range_check = Lookup::configure(meta, advices[9], table_idx);

    // Configuration for curve point operations.
    // This uses 10 advice columns and spans the whole circuit.
    let ecc_config = EccChip::<OrchardFixedBases, Lookup>::configure(
        meta,
        advices,
        lagrange_coeffs,
        range_check,
    );

    // Configuration for the Poseidon hash.
    let poseidon_config = PoseidonChip::configure::<poseidon::P128Pow5T3>(
        meta,
        // We place the state columns after the partial_sbox column so that the
        // pad-and-add region can be laid out more efficiently.
        advices[6..9].try_into().unwrap(),
        advices[5],
        rc_a,
        rc_b,
    );

    // Configuration for a Sinsemilla hash instantiation and a
    // Merkle hash instantiation using this Sinsemilla instance.
    // Since the Sinsemilla config uses only 5 advice columns,
    // we can fit two instances side-by-side.
    let (sinsemilla_config_1, merkle_config_1) = {
        let sinsemilla_config_1 = SinsemillaChip::configure(
            meta,
            advices[..5].try_into().unwrap(),
            advices[6],
            lagrange_coeffs[0],
            lookup,
            range_check,
            is_zsa,
        );
        let merkle_config_1 = MerkleChip::configure(meta, sinsemilla_config_1.clone());

        (sinsemilla_config_1, merkle_config_1)
    };

    // Configuration for a Sinsemilla hash instantiation and a
    // Merkle hash instantiation using this Sinsemilla instance.
    // Since the Sinsemilla config uses only 5 advice columns,
    // we can fit two instances side-by-side.
    let (sinsemilla_config_2, merkle_config_2) = {
        let sinsemilla_config_2 = SinsemillaChip::configure(
            meta,
            advices[5..].try_into().unwrap(),
            advices[7],
            lagrange_coeffs[1],
            lookup,
            range_check,
            is_zsa,
        );
        let merkle_config_2 = MerkleChip::configure(meta, sinsemilla_config_2.clone());

        (sinsemilla_config_2, merkle_config_2)
    };

    // Configuration to handle decomposition and canonicity checking
    // for CommitIvk.
    let commit_ivk_config = CommitIvkChip::configure(meta, advices);

    // Configuration to handle decomposition and canonicity checking
    // for NoteCommit_old.
    let old_note_commit_config =
        NoteCommitChip::configure(meta, advices, sinsemilla_config_1.clone(), is_zsa);

    // Configuration to handle decomposition and canonicity checking
    // for NoteCommit_new.
    let new_note_commit_config =
        NoteCommitChip::configure(meta, advices, sinsemilla_config_2.clone(), is_zsa);

    Config {
        primary,
        q_orchard,
        advices,
        add_config,
        ecc_config,
        poseidon_config,
        merkle_config_1,
        merkle_config_2,
        sinsemilla_config_1,
        sinsemilla_config_2,
        commit_ivk_config,
        old_note_commit_config,
        new_note_commit_config,
    }
}

/// Cells carrying the addresses of an action's spent and newly created notes, returned
/// from the shared synthesis logic so that circuit versions can impose additional
/// constraints on them.
struct AddressPoints<Lookup: PallasLookupRangeCheck> {
    g_d_old: NonIdentityPoint<pallas::Affine, EccChip<OrchardFixedBases, Lookup>>,
    pk_d_old: NonIdentityPoint<pallas::Affine, EccChip<OrchardFixedBases, Lookup>>,
    g_d_new: NonIdentityPoint<pallas::Affine, EccChip<OrchardFixedBases, Lookup>>,
    pk_d_new: NonIdentityPoint<pallas::Affine, EccChip<OrchardFixedBases, Lookup>>,
}

/// The verifying key for the Orchard Action circuit.
///
/// Build with [`VerifyingKey::build`] for an explicit circuit version.
#[derive(Debug)]
pub struct VerifyingKey {
    pub(crate) params: halo2_proofs::poly::commitment::Params<vesta::Affine>,
    pub(crate) vk: plonk::VerifyingKey<vesta::Affine>,
    circuit_version: OrchardCircuitVersion,
}

impl VerifyingKey {
    /// Builds the verifying key for the given circuit version.
    ///
    /// See [`OrchardCircuitVersion`] for which version to use.
    pub fn build(circuit_version: OrchardCircuitVersion) -> Self {
        let params = halo2_proofs::poly::commitment::Params::new(K);
        let vk = if circuit_version.is_zsa() {
            plonk::keygen_vk(&params, &CircuitZsa::empty()).unwrap()
        } else {
            plonk::keygen_vk(&params, &CircuitVanilla::empty(circuit_version)).unwrap()
        };

        VerifyingKey {
            params,
            vk,
            circuit_version,
        }
    }

    /// The circuit version this verifying key was built for.
    pub fn circuit_version(&self) -> OrchardCircuitVersion {
        self.circuit_version
    }

    /// Returns whether this verifying key supports the cross-address restriction.
    pub fn supports_cross_address_restriction(&self) -> bool {
        self.circuit_version.supports_cross_address_restriction()
    }
}

/// The proving key for the Orchard Action circuit.
///
/// Build with [`ProvingKey::build`] for an explicit circuit version.
/// The resulting proofs verify only under a compatible [`VerifyingKey`].
#[derive(Debug)]
pub struct ProvingKey {
    params: halo2_proofs::poly::commitment::Params<vesta::Affine>,
    pk: plonk::ProvingKey<vesta::Affine>,
    circuit_version: OrchardCircuitVersion,
}

impl ProvingKey {
    /// Builds the proving key for the given circuit version.
    ///
    /// See [`OrchardCircuitVersion`] for which version to use.
    pub fn build(circuit_version: OrchardCircuitVersion) -> Self {
        let params = halo2_proofs::poly::commitment::Params::new(K);
        let pk = if circuit_version.is_zsa() {
            let circuit = CircuitZsa::empty();
            let vk = plonk::keygen_vk(&params, &circuit).unwrap();
            plonk::keygen_pk(&params, vk, &circuit).unwrap()
        } else {
            let circuit = CircuitVanilla::empty(circuit_version);
            let vk = plonk::keygen_vk(&params, &circuit).unwrap();
            plonk::keygen_pk(&params, vk, &circuit).unwrap()
        };

        ProvingKey {
            params,
            pk,
            circuit_version,
        }
    }

    /// The circuit version this proving key produces proofs for.
    pub fn circuit_version(&self) -> OrchardCircuitVersion {
        self.circuit_version
    }

    /// Returns whether this proving key supports the cross-address restriction.
    pub fn supports_cross_address_restriction(&self) -> bool {
        self.circuit_version.supports_cross_address_restriction()
    }
}

/// Public inputs to the Orchard Action circuit.
///
/// Adding a public input whose default value is zero is backwards-compatible: halo2_proofs
/// zero-pads instance values, so a statement that leaves the public input zero encodes exactly
/// as it did before the public input existed.
///
/// # Invariants
///
/// Every `Instance` has a non-identity `rk`.
#[derive(Clone, Debug)]
pub struct Instance {
    anchor: Anchor,
    cv_net: ValueCommitment,
    nf_old: Nullifier,
    rk: VerificationKey<SpendAuth>,
    cmx: ExtractedNoteCommitment,
    enable_spend: bool,
    enable_output: bool,
    cross_address_disabled: bool,
    enable_zsa: bool,
}

impl Instance {
    /// Constructs an [`Instance`] from its constituent parts.
    ///
    /// This API can be used in combination with [`Proof::verify`] to build verification
    /// pipelines for many proofs, where you don't want to pass around the full bundle.
    /// Use [`Bundle::verify_proof`] instead if you have the full bundle.
    ///
    /// The provided [`Flags`] are encoded into the spend/output enable public inputs and
    /// the `disableCrossAddress` public input, which is set to the negation of
    /// [`Flags::cross_address_enabled`]. If cross-address transfers are disabled,
    /// callers must use a proving or verifying key whose circuit version supports the
    /// cross-address restriction; [`Proof::create`], [`Proof::verify`], and
    /// [`crate::bundle::BatchValidator`] enforce this.
    ///
    /// Returns `None` if `rk` is the identity [`pasta_curves::pallas::Point`].
    /// zcashd v6.12.1 and Zebra 4.3.1 both added a consensus rule rejecting
    /// transactions whose Orchard actions have an identity `rk`; the Zcash
    /// protocol specification will be updated to match, and this crate
    /// aligns with that rule.
    ///
    /// See:
    /// - <https://zodl.com/zcashd-zebra-april-2026-disclosure/>
    /// - <https://zfnd.org/zebra-4-3-1-critical-security-fixes-dockerized-mining-and-ci-hardening/>
    ///
    /// [`Bundle::verify_proof`]: crate::Bundle::verify_proof
    pub fn from_parts(
        anchor: Anchor,
        cv_net: ValueCommitment,
        nf_old: Nullifier,
        rk: VerificationKey<SpendAuth>,
        cmx: ExtractedNoteCommitment,
        flags: Flags,
    ) -> Option<Self> {
        (!rk.is_identity()).then_some(Instance {
            anchor,
            cv_net,
            nf_old,
            rk,
            cmx,
            enable_spend: flags.spends_enabled(),
            enable_output: flags.outputs_enabled(),
            cross_address_disabled: !flags.cross_address_enabled(),
            enable_zsa: flags.zsa_enabled(),
        })
    }

    /// Returns the Merkle tree anchor of this instance.
    pub(crate) fn anchor(&self) -> &Anchor {
        &self.anchor
    }

    /// Returns the commitment to the net value of this instance.
    pub(crate) fn cv_net(&self) -> &ValueCommitment {
        &self.cv_net
    }

    /// Returns the nullifier of the note being spent by this instance.
    pub(crate) fn nf_old(&self) -> &Nullifier {
        &self.nf_old
    }

    /// Returns the randomized verification key of this instance.
    pub(crate) fn rk(&self) -> &VerificationKey<SpendAuth> {
        &self.rk
    }

    /// Returns the commitment to the new note being created by this instance.
    pub(crate) fn cmx(&self) -> &ExtractedNoteCommitment {
        &self.cmx
    }

    /// Returns whether the spend is enabled for this instance.
    pub(crate) fn enable_spend(&self) -> bool {
        self.enable_spend
    }

    /// Returns whether the output is enabled for this instance.
    pub(crate) fn enable_output(&self) -> bool {
        self.enable_output
    }

    /// Returns whether cross-address transfers are disabled for this instance.
    pub(crate) fn cross_address_disabled(&self) -> bool {
        self.cross_address_disabled
    }

    /// Returns whether zsa are enabled for this instance.
    pub(crate) fn enable_zsa(&self) -> bool {
        self.enable_zsa
    }

    /// Adding a public input whose default value is zero is backwards-compatible:
    /// halo2_proofs zero-pads instance values, so a statement that leaves the public input
    /// zero encodes exactly as it did before the public input existed.
    fn to_halo2_instance(&self) -> [[vesta::Scalar; 11]; 1] {
        let mut instance = [vesta::Scalar::zero(); 11];

        instance[ANCHOR] = self.anchor.inner();
        instance[CV_NET_X] = self.cv_net.x();
        instance[CV_NET_Y] = self.cv_net.y();
        instance[NF_OLD] = self.nf_old.inner();

        let rk = pallas::Point::from_bytes(&self.rk.clone().into())
            .expect("the cached byte encoding of a VerificationKey<_> is canonical")
            .to_affine()
            .coordinates()
            .expect("rk is non-identity by construction");

        instance[RK_X] = *rk.x();
        instance[RK_Y] = *rk.y();
        instance[CMX] = self.cmx.inner();
        instance[ENABLE_SPEND] = vesta::Scalar::from(u64::from(self.enable_spend));
        instance[ENABLE_OUTPUT] = vesta::Scalar::from(u64::from(self.enable_output));
        // Pre-NU 6.3 circuits leave this row unconstrained, which is why restricted statements
        // must never reach those keys (see `Proof::create` and `Proof::verify`).
        instance[DISABLE_CROSS_ADDRESS] =
            vesta::Scalar::from(u64::from(self.cross_address_disabled));
        instance[ENABLE_ZSA] = vesta::Scalar::from(u64::from(self.enable_zsa));

        [instance]
    }
}

impl Proof {
    /// Creates a proof for the given circuits and instances.
    ///
    /// The resulting proof verifies only under a compatible [`VerifyingKey`] (see
    /// [`OrchardCircuitVersion`]).
    ///
    /// Returns [`plonk::Error::Synthesis`] if any circuit's version does not match `pk`'s
    /// version, since `pk` could not produce a valid proof for it.
    ///
    /// Returns [`plonk::Error::InvalidInstances`] if any instance has
    /// `disableCrossAddress = 1` and `pk` is not an
    /// [`OrchardCircuitVersion::PostNu6_3`] or an [`OrchardCircuitVersion::ZSA`] proving key.
    ///
    /// Returns [`plonk::Error::InvalidInstances`] if any instance has
    /// `enable_zsa = 1` and `pk` is not an
    /// [`OrchardCircuitVersion::ZSA`] proving key.
    ///
    /// All instances of a bundle carry the same `disableCrossAddress` value; that uniformity
    /// is the bundle layer's invariant, and is not checked here.
    pub fn create(
        pk: &ProvingKey,
        circuits: &[Circuit],
        instances: &[Instance],
        mut rng: impl RngCore,
    ) -> Result<Self, plonk::Error> {
        if circuits
            .iter()
            .any(|c| c.circuit_version() != pk.circuit_version)
        {
            return Err(plonk::Error::Synthesis);
        }

        if instances.iter().any(Instance::cross_address_disabled)
            && !pk.supports_cross_address_restriction()
        {
            return Err(plonk::Error::InvalidInstances);
        }

        if instances.iter().any(Instance::enable_zsa) && !pk.circuit_version.is_zsa() {
            return Err(plonk::Error::InvalidInstances);
        }

        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance()).collect();
        let instances: Vec<Vec<_>> = instances
            .iter()
            .map(|i| i.iter().map(|c| &c[..]).collect())
            .collect();
        let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

        let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);

        if pk.circuit_version.is_zsa() {
            let circuits: Vec<_> = circuits
                .iter()
                .map(Circuit::to_zsa)
                .collect::<Result<_, _>>()?;
            plonk::create_proof(
                &pk.params,
                &pk.pk,
                &circuits,
                &instances,
                &mut rng,
                &mut transcript,
            )?;
        } else {
            let circuits: Vec<_> = circuits
                .iter()
                .map(Circuit::to_vanilla)
                .collect::<Result<_, _>>()?;
            plonk::create_proof(
                &pk.params,
                &pk.pk,
                &circuits,
                &instances,
                &mut rng,
                &mut transcript,
            )?;
        };

        Ok(Proof(transcript.finalize()))
    }

    /// Verifies this proof with the given instances.
    ///
    /// # Errors
    ///
    /// Returns [`plonk::Error::InvalidInstances`] if any instance has
    /// `disableCrossAddress = 1` and `vk` is not an
    /// [`OrchardCircuitVersion::PostNu6_3`] or an [`OrchardCircuitVersion::ZSA`] verifying key.
    ///
    /// Returns [`plonk::Error::InvalidInstances`] if any instance has
    /// `enable_zsa = 1` and `vk` is not an
    /// [`OrchardCircuitVersion::ZSA`] verifying key.
    ///
    /// Also returns an error if proof verification fails.
    pub fn verify(&self, vk: &VerifyingKey, instances: &[Instance]) -> Result<(), plonk::Error> {
        if instances.iter().any(Instance::cross_address_disabled)
            && !vk.supports_cross_address_restriction()
        {
            return Err(plonk::Error::InvalidInstances);
        }

        if instances.iter().any(Instance::enable_zsa) && !vk.circuit_version.is_zsa() {
            return Err(plonk::Error::InvalidInstances);
        }

        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance()).collect();
        let instances: Vec<Vec<_>> = instances
            .iter()
            .map(|i| i.iter().map(|c| &c[..]).collect())
            .collect();
        let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

        let strategy = SingleVerifier::new(&vk.params);
        let mut transcript = Blake2bRead::init(&self.0[..]);
        plonk::verify_proof(&vk.params, &vk.vk, strategy, &instances, &mut transcript)
    }

    /// Adds this proof to the given batch for verification with the given instances.
    ///
    /// Internal to [`BatchValidator`], which is the only public batch path. A raw batch
    /// does not know which [`VerifyingKey`] it will be finalized with, so it cannot enforce
    /// that instances disabling cross-address transfers are only finalized with a key whose
    /// circuit version constrains the `disableCrossAddress` public input (see
    /// [`OrchardCircuitVersion::supports_cross_address_restriction`]). [`BatchValidator`]
    /// binds its key at construction and rejects such bundles in [`add_bundle`] before they
    /// reach this method; exposing this directly would let a caller sidestep that check by
    /// finalizing the batch against an unsupported key.
    ///
    /// [`BatchValidator`]: crate::bundle::BatchValidator
    /// [`add_bundle`]: crate::bundle::BatchValidator::add_bundle
    pub(crate) fn add_to_batch(
        &self,
        batch: &mut BatchVerifier<vesta::Affine>,
        instances: Vec<Instance>,
    ) {
        let instances = instances
            .iter()
            .map(|i| {
                i.to_halo2_instance()
                    .into_iter()
                    .map(|c| c.into_iter().collect())
                    .collect()
            })
            .collect();

        batch.add_proof(instances, self.0.clone());
    }
}

#[cfg(all(test, feature = "verifier-fingerprint"))]
mod fingerprint;

#[cfg(test)]
mod tests {

    mod from_parts_rk_identity {
        use ff::{Field as _, PrimeField as _};
        use pasta_curves::pallas;

        use super::super::Instance;
        use crate::{
            bundle::Flags,
            note::{ExtractedNoteCommitment, Nullifier},
            primitives::redpallas::{self, SpendAuth},
            tree::Anchor,
            value::{ValueCommitTrapdoor, ValueCommitment, ValueSum},
        };

        /// Non-rk fields for `Instance`. Distinct non-zero patterns avoid
        /// accidental overlap with sentinel values. See the analogous helper
        /// in `src/action.rs` for notes on which of these fields have
        /// consensus-level validity checks elsewhere in the pipeline.
        fn dummy_other_fields() -> (Anchor, ValueCommitment, Nullifier, ExtractedNoteCommitment) {
            let anchor = Anchor::from_bytes([6u8; 32]).unwrap();
            let cv_net =
                ValueCommitment::derive(ValueSum::from_raw(42), ValueCommitTrapdoor::zero());
            let nf_old = Nullifier::from_bytes(&[1u8; 32]).unwrap();
            let cmx = ExtractedNoteCommitment::from_bytes(&[2u8; 32]).unwrap();
            (anchor, cv_net, nf_old, cmx)
        }

        fn identity_rk() -> redpallas::VerificationKey<SpendAuth> {
            redpallas::VerificationKey::<SpendAuth>::try_from([0u8; 32])
                .expect("plain redpallas accepts the identity encoding")
        }

        fn non_identity_rk() -> redpallas::VerificationKey<SpendAuth> {
            let ask_bytes: [u8; 32] = pallas::Scalar::ONE.to_repr();
            let ask = redpallas::SigningKey::<SpendAuth>::try_from(ask_bytes)
                .expect("1 is a valid scalar");
            (&ask).into()
        }

        #[test]
        fn rejects_identity_rk() {
            let (anchor, cv_net, nf_old, cmx) = dummy_other_fields();
            let result =
                Instance::from_parts(anchor, cv_net, nf_old, identity_rk(), cmx, Flags::ENABLED);
            assert!(result.is_none());
        }

        #[test]
        fn accepts_non_identity_rk() {
            let (anchor, cv_net, nf_old, cmx) = dummy_other_fields();
            let rk = non_identity_rk();
            let instance =
                Instance::from_parts(anchor, cv_net, nf_old, rk.clone(), cmx, Flags::ENABLED)
                    .expect("non-identity rk must be accepted");
            assert_eq!(instance.rk(), &rk);
        }
    }

    mod zsa_witnesses_invariant {
        use ff::Field;
        use halo2_proofs::circuit::Value;
        use pasta_curves::pallas;

        use super::super::{
            plonk, AdditionalZsaWitnesses, Circuit, CircuitVanilla, OrchardCircuitVersion,
        };
        use crate::note::AssetBase;

        /// `Circuit::to_vanilla` must reject any `Circuit` that carries ZSA-specific
        /// witnesses for a non-ZSA `circuit_version`: the Vanilla circuit versions have no
        /// way to prove them, so their presence indicates a construction bug rather than a
        /// provable statement.
        #[test]
        fn to_vanilla_errors_if_zsa_witnesses_are_present() {
            let circuit = Circuit {
                common_witnesses: CircuitVanilla::empty(OrchardCircuitVersion::FixedPostNu6_2),
                additional_zsa_witnesses: Some(AdditionalZsaWitnesses {
                    psi_nf: Value::known(pallas::Base::ZERO),
                    asset: Value::known(AssetBase::zatoshi()),
                    split_flag: Value::known(false),
                }),
            };

            assert!(matches!(circuit.to_vanilla(), Err(plonk::Error::Synthesis)));
        }

        #[test]
        fn to_vanilla_accepts_absent_zsa_witnesses() {
            let circuit = Circuit {
                common_witnesses: CircuitVanilla::empty(OrchardCircuitVersion::FixedPostNu6_2),
                additional_zsa_witnesses: None,
            };

            assert!(circuit.to_vanilla().is_ok());
        }

        /// `Circuit::to_zsa` must reject a `Circuit` that carries no ZSA-specific witnesses,
        /// since the ZSA circuit version cannot prove its statement without them.
        #[test]
        fn to_zsa_errors_if_zsa_witnesses_are_absent() {
            let circuit = Circuit {
                common_witnesses: CircuitVanilla::empty(OrchardCircuitVersion::ZSA),
                additional_zsa_witnesses: None,
            };

            assert!(matches!(circuit.to_zsa(), Err(plonk::Error::Synthesis)));
        }

        #[test]
        fn to_zsa_accepts_present_zsa_witnesses() {
            let circuit = Circuit {
                common_witnesses: CircuitVanilla::empty(OrchardCircuitVersion::ZSA),
                additional_zsa_witnesses: Some(AdditionalZsaWitnesses {
                    psi_nf: Value::unknown(),
                    asset: Value::unknown(),
                    split_flag: Value::unknown(),
                }),
            };

            assert!(circuit.to_zsa().is_ok());
        }
    }
}
