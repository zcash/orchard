//! The Orchard Action circuit implementation.
//!
//! This module defines the common structures, traits and implementations for the
//! Orchard Action circuit, supporting both the standard ("Vanilla") and ZSA variations.

use alloc::vec::Vec;

use group::{Curve, GroupEncoding};
use halo2_proofs::{
    circuit::{floor_planner, Layouter, Value},
    plonk::{
        self, Advice, BatchVerifier, Column, Instance as InstanceColumn, Selector, SingleVerifier,
    },
    transcript::{Blake2bRead, Blake2bWrite},
};
use pasta_curves::{arithmetic::CurveAffine, pallas, vesta};
use rand::RngCore;
use std::marker::PhantomData;

use crate::{
    builder::SpendInfo,
    bundle::Flags,
    circuit::{
        commit_ivk::CommitIvkConfig, gadget::add_chip::AddConfig, note_commit::NoteCommitConfig,
    },
    constants::{
        OrchardCommitDomains, OrchardFixedBases, OrchardHashDomains, MERKLE_DEPTH_ORCHARD,
    },
    keys::{
        CommitIvkRandomness, DiversifiedTransmissionKey, NullifierDerivingKey, SpendValidatingKey,
    },
    note::{
        commitment::{NoteCommitTrapdoor, NoteCommitment},
        nullifier::Nullifier,
        AssetBase, ExtractedNoteCommitment, Note, Rho,
    },
    primitives::redpallas::{SpendAuth, VerificationKey},
    spec::NonIdentityPallasPoint,
    tree::{Anchor, MerkleHashOrchard},
    value::{NoteValue, ValueCommitTrapdoor, ValueCommitment},
};
use halo2_gadgets::{
    ecc::{chip::EccConfig, CircuitVersion},
    poseidon::Pow5Config as PoseidonConfig,
    sinsemilla::{chip::SinsemillaConfig, merkle::chip::MerkleConfig},
    utilities::lookup_range_check::PallasLookupRangeCheck,
};

mod circuit_vanilla;
mod circuit_zsa;

#[cfg(not(feature = "unstable-voting-circuits"))]
pub(in crate::circuit) mod commit_ivk;
#[cfg(feature = "unstable-voting-circuits")]
pub mod commit_ivk;
pub(in crate::circuit) mod derive_nullifier;
pub mod gadget;
#[cfg(not(feature = "unstable-voting-circuits"))]
pub(in crate::circuit) mod note_commit;
#[cfg(feature = "unstable-voting-circuits")]
pub mod note_commit;
pub(in crate::circuit) mod orchard_sinsemilla_chip;
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

/// The `OrchardCircuit` trait defines an interface for different implementations of the PLONK circuit
/// for the different Orchard protocol flavors (Vanilla and ZSA). It serves as a bridge between
/// plonk::Circuit interfaces and specific requirements of the Orchard protocol's variations.
pub trait OrchardCircuit: Sized + Default {
    /// Substitution for Config type of plonk::Circuit trait
    type Config: Clone;

    /// Wrapper for configure function of plonk::Circuit trait
    fn configure(meta: &mut plonk::ConstraintSystem<pallas::Base>) -> Self::Config;

    /// Wrapper for configure function of plonk::Circuit trait
    fn synthesize(
        circuit: &Witnesses,
        config: Self::Config,
        layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), plonk::Error>;

    /// Builds the ZSA-specific witnesses for the circuit.
    /// For OrchardVanilla circuits, it should return `Value::unknown()`.
    fn build_additional_zsa_witnesses(
        psi_nf: pallas::Base,
        asset: AssetBase,
        split_flag: bool,
    ) -> Value<AdditionalZsaWitnesses>;
}

impl<C: OrchardCircuit> plonk::Circuit<pallas::Base> for Circuit<C> {
    type Config = C::Config;
    type FloorPlanner = floor_planner::V1;

    fn without_witnesses(&self) -> Self {
        Self::empty(self.witnesses.circuit_version)
    }

    fn configure(meta: &mut plonk::ConstraintSystem<pallas::Base>) -> Self::Config {
        C::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        layouter: impl Layouter<pallas::Base>,
    ) -> Result<(), plonk::Error> {
        C::synthesize(&self.witnesses, config, layouter)
    }
}

/// Selects which version of the Orchard Action circuit to build.
///
/// [`FixedPostNu6_2`] and [`InsecurePreNu6_2`] produce different verifying keys: the fixed
/// circuit anchors the variable-base scalar-multiplication base (see `halo2_gadgets`), while
/// the pre-NU6.2 one does not. [`PostNu6_3`] extends the fixed circuit by enforcing the
/// same-address check, i.e. `(g_d^old, pk_d^old) = (g_d^new, pk_d^new)`, when the
/// boolean `disableCrossAddress` public input is set.
///
/// This is a runtime value rather than a type parameter: it is carried in [`Circuit`] and
/// chosen when building a [`ProvingKey`] or [`VerifyingKey`], so the circuit version can be
/// threaded dynamically (e.g. across an FFI boundary).
///
/// Please note that the public exposure of APIs using `InsecurePreNu6_2` is intentional,
/// and is strictly necessary for verifying the block chain from NU5 activation and for
/// creating proofs needed by tests that operate at past epochs. These APIs cannot be
/// used accidentally without passing an `OrchardCircuitVersion` that is clearly labelled
/// "insecure". This is not a security vulnerability.
///
/// [`FixedPostNu6_2`]: OrchardCircuitVersion::FixedPostNu6_2
/// [`InsecurePreNu6_2`]: OrchardCircuitVersion::InsecurePreNu6_2
/// [`PostNu6_3`]: OrchardCircuitVersion::PostNu6_3
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchardCircuitVersion {
    /// The insecure pre-NU6.2 circuit, in which the variable-base scalar-multiplication base
    /// is not anchored to the real base. For reconstructing the historical (NU5..NU6.2)
    /// verifying key only — never for proving or current verification.
    InsecurePreNu6_2,
    /// The fixed circuit, active from NU6.2 onward. Used for all current network proving and
    /// verification.
    FixedPostNu6_2,
    /// The post-NU 6.3 circuit. This uses the fixed circuit with additional constraints
    /// enforcing the `disableCrossAddress` public input.
    PostNu6_3,
    /// ZSA
    ZSA,
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
            OrchardCircuitVersion::InsecurePreNu6_2
            | OrchardCircuitVersion::FixedPostNu6_2
            | OrchardCircuitVersion::ZSA => false,
            OrchardCircuitVersion::PostNu6_3 => true,
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
}

/// The Orchard Action circuit.
#[derive(Clone, Debug)]
pub struct Circuit<C: OrchardCircuit> {
    pub(crate) witnesses: Witnesses,
    pub(crate) phantom: core::marker::PhantomData<C>,
}

impl<C: OrchardCircuit> Circuit<C> {
    /// Returns an empty circuit with all private witnesses unknown.
    ///
    /// This is used for circuit shape-dependent operations, such as generating keys
    /// or rendering the circuit layout, where witness values are not required but the
    /// selected circuit version still determines the configured constraints.
    fn empty(circuit_version: OrchardCircuitVersion) -> Self {
        Circuit::<C> {
            witnesses: Witnesses::empty(circuit_version),
            phantom: PhantomData,
        }
    }
}

/// The ZSA-specific witnesses.
#[derive(Clone, Debug)]
pub struct AdditionalZsaWitnesses {
    pub(crate) psi_nf: pallas::Base,
    pub(crate) asset: AssetBase,
    pub(crate) split_flag: bool,
}

fn unpack(
    zsa_values: Value<AdditionalZsaWitnesses>,
) -> (Value<pallas::Base>, Value<AssetBase>, Value<bool>) {
    (
        zsa_values.clone().map(|values| values.psi_nf),
        zsa_values.clone().map(|values| values.asset),
        zsa_values.map(|values| values.split_flag),
    )
}

/// The Orchard Action witnesses
///
/// The `circuit_version` field selects which circuit to build. Callers must choose it
/// explicitly instead of relying on a default.
#[derive(Clone, Debug)]
pub struct Witnesses {
    pub(crate) path: Value<[MerkleHashOrchard; MERKLE_DEPTH_ORCHARD]>,
    pub(crate) pos: Value<u32>,
    pub(crate) g_d_old: Value<NonIdentityPallasPoint>,
    pub(crate) pk_d_old: Value<DiversifiedTransmissionKey>,
    pub(crate) v_old: Value<NoteValue>,
    pub(crate) rho_old: Value<Rho>,
    pub(crate) psi_old: Value<pallas::Base>,
    pub(crate) rcm_old: Value<NoteCommitTrapdoor>,
    pub(crate) cm_old: Value<NoteCommitment>,
    pub(crate) alpha: Value<pallas::Scalar>,
    pub(crate) ak: Value<SpendValidatingKey>,
    pub(crate) nk: Value<NullifierDerivingKey>,
    pub(crate) rivk: Value<CommitIvkRandomness>,
    pub(crate) g_d_new: Value<NonIdentityPallasPoint>,
    pub(crate) pk_d_new: Value<DiversifiedTransmissionKey>,
    pub(crate) v_new: Value<NoteValue>,
    pub(crate) psi_new: Value<pallas::Base>,
    pub(crate) rcm_new: Value<NoteCommitTrapdoor>,
    pub(crate) rcv: Value<ValueCommitTrapdoor>,

    // The ZSA-specific witnesses.
    // For OrchardVanilla circuits, this field should be initialized to `Value::unknown()`.
    pub(crate) additional_zsa_witnesses: Value<AdditionalZsaWitnesses>,
    pub(crate) circuit_version: OrchardCircuitVersion,
}

impl Witnesses {
    /// Returns an empty circuit with all private witnesses unknown.
    ///
    /// This is used for circuit shape-dependent operations, such as generating keys
    /// or rendering the circuit layout, where witness values are not required but the
    /// selected circuit version still determines the configured constraints.
    fn empty(circuit_version: OrchardCircuitVersion) -> Self {
        Witnesses {
            path: Value::unknown(),
            pos: Value::unknown(),
            g_d_old: Value::unknown(),
            pk_d_old: Value::unknown(),
            v_old: Value::unknown(),
            rho_old: Value::unknown(),
            psi_old: Value::unknown(),
            rcm_old: Value::unknown(),
            cm_old: Value::unknown(),
            alpha: Value::unknown(),
            ak: Value::unknown(),
            nk: Value::unknown(),
            rivk: Value::unknown(),
            g_d_new: Value::unknown(),
            pk_d_new: Value::unknown(),
            v_new: Value::unknown(),
            psi_new: Value::unknown(),
            rcm_new: Value::unknown(),
            rcv: Value::unknown(),
            additional_zsa_witnesses: Value::unknown(),
            circuit_version,
        }
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
    pub fn from_action_context<C: OrchardCircuit>(
        spend: SpendInfo,
        output_note: Note,
        alpha: pallas::Scalar,
        rcv: ValueCommitTrapdoor,
        circuit_version: OrchardCircuitVersion,
    ) -> Option<Self> {
        (Rho::from_nf_old(spend.note.nullifier(&spend.fvk)) == output_note.rho()).then(|| {
            Self::from_action_context_unchecked::<C>(
                spend,
                output_note,
                alpha,
                rcv,
                circuit_version,
            )
        })
    }

    pub(crate) fn from_action_context_unchecked<C: OrchardCircuit>(
        spend: SpendInfo,
        output_note: Note,
        alpha: pallas::Scalar,
        rcv: ValueCommitTrapdoor,
        circuit_version: OrchardCircuitVersion,
    ) -> Self {
        let sender_address = spend.note.recipient();
        let rho_old = spend.note.rho();
        let psi_old = spend.note.psi();
        let rcm_old = spend.note.rcm();

        let psi_new = output_note.psi();
        let rcm_new = output_note.rcm();

        let nf_rseed = spend.note.rseed_split_note().unwrap_or(*spend.note.rseed());
        let psi_nf = nf_rseed.psi(&rho_old);
        let additional_zsa_witnesses =
            C::build_additional_zsa_witnesses(psi_nf, spend.note.asset(), spend.split_flag);

        Witnesses {
            path: Value::known(spend.merkle_path.auth_path()),
            pos: Value::known(spend.merkle_path.position()),
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

            additional_zsa_witnesses,
            circuit_version,
        }
    }
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
    pub fn build<C: OrchardCircuit>(circuit_version: OrchardCircuitVersion) -> Self {
        let params = halo2_proofs::poly::commitment::Params::new(K);
        let circuit = Circuit::<C>::empty(circuit_version);

        let vk = plonk::keygen_vk(&params, &circuit).unwrap();

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
    pub fn build<C: OrchardCircuit>(circuit_version: OrchardCircuitVersion) -> Self {
        let params = halo2_proofs::poly::commitment::Params::new(K);
        let circuit = Circuit::<C>::empty(circuit_version);

        let vk = plonk::keygen_vk(&params, &circuit).unwrap();
        let pk = plonk::keygen_pk(&params, vk, &circuit).unwrap();

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
/// The `enable_zsa` field was introduced with the ZSA feature; it did not exist before.
/// In vanilla Orchard, `enable_zsa` is always false, so this method always appends a zero to the
/// instance vector. Since halo2_proofs pads instance values with zero, old proofs (without this
/// extra entry) and new proofs behave identically.
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

    /// Note: Before the ZSA feature was introduced, this method returned a 9-element instance slice.
    /// With ZSA, it now returns 10 elements, the last one corresponding to `enable_zsa`.
    /// In vanilla Orchard, `enable_zsa` is always false, so this extra element is always zero.
    /// Since halo2_proofs pads instance values with zero, old proofs (without this element)
    /// and new proofs behave identically.
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
        // Instance columns are zero-padded over the evaluation domain, so for statements
        // where this flag is false, this encoding is commitment-identical to the historical
        // nine-row encoding. Pre-NU 6.3 circuits leave this row unconstrained, which is why
        // restricted statements must never reach those keys (see `Proof::create` and
        // `Proof::verify`).
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
    /// [`OrchardCircuitVersion::PostNu6_3`] proving key.
    ///
    /// All instances of a bundle carry the same `disableCrossAddress` value; that uniformity
    /// is the bundle layer's invariant, and is not checked here.
    pub fn create<C: OrchardCircuit>(
        pk: &ProvingKey,
        circuits: &[Circuit<C>],
        instances: &[Instance],
        mut rng: impl RngCore,
    ) -> Result<Self, plonk::Error> {
        if circuits
            .iter()
            .any(|c| c.witnesses.circuit_version != pk.circuit_version)
        {
            return Err(plonk::Error::Synthesis);
        }
        if instances.iter().any(Instance::cross_address_disabled)
            && !pk.supports_cross_address_restriction()
        {
            return Err(plonk::Error::InvalidInstances);
        }

        let instances: Vec<_> = instances.iter().map(|i| i.to_halo2_instance()).collect();
        let instances: Vec<Vec<_>> = instances
            .iter()
            .map(|i| i.iter().map(|c| &c[..]).collect())
            .collect();
        let instances: Vec<_> = instances.iter().map(|i| &i[..]).collect();

        let mut transcript = Blake2bWrite::<_, vesta::Affine, _>::init(vec![]);
        plonk::create_proof(
            &pk.params,
            &pk.pk,
            circuits,
            &instances,
            &mut rng,
            &mut transcript,
        )?;
        Ok(Proof(transcript.finalize()))
    }

    /// Verifies this proof with the given instances.
    ///
    /// # Errors
    ///
    /// Returns [`plonk::Error::InvalidInstances`] if any instance has
    /// `disableCrossAddress = 1` and `vk` is not an
    /// [`OrchardCircuitVersion::PostNu6_3`] verifying key.
    ///
    /// Also returns an error if proof verification fails.
    pub fn verify(&self, vk: &VerifyingKey, instances: &[Instance]) -> Result<(), plonk::Error> {
        if instances.iter().any(Instance::cross_address_disabled)
            && !vk.supports_cross_address_restriction()
        {
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

#[cfg(test)]
mod tests {

    
    

    mod from_parts_rk_identity {
        use ff::{Field as _, PrimeField as _};
        use pasta_curves::pallas;

        use super::super::Instance;
        use crate::note::AssetBase;
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
            let cv_net = ValueCommitment::derive(
                ValueSum::from_raw_inner(42),
                ValueCommitTrapdoor::zero(),
                AssetBase::zatoshi(),
            );
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
}
