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
    ecc::chip::EccConfig,
    poseidon::Pow5Config as PoseidonConfig,
    sinsemilla::{chip::SinsemillaConfig, merkle::chip::MerkleConfig},
    utilities::lookup_range_check::PallasLookupRangeCheck,
};

mod circuit_vanilla;
mod circuit_zsa;

pub(in crate::circuit) mod commit_ivk;
pub(in crate::circuit) mod derive_nullifier;
pub(in crate::circuit) mod gadget;
pub(in crate::circuit) mod note_commit;
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
const ENABLE_ZSA: usize = 9;

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
        Self::default()
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

/// The Orchard Action circuit.
#[derive(Clone, Debug, Default)]
pub struct Circuit<C: OrchardCircuit> {
    pub(crate) witnesses: Witnesses,
    pub(crate) phantom: core::marker::PhantomData<C>,
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
#[derive(Clone, Debug, Default)]
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
}

impl Witnesses {
    /// This constructor is public to enable creation of custom builders.
    /// If you are not creating a custom builder, use [`Builder`] to compose
    /// and authorize a transaction.
    ///
    /// Constructs a `Circuit` from the following components:
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
    ) -> Option<Self> {
        (Rho::from_nf_old(spend.note.nullifier(&spend.fvk)) == output_note.rho())
            .then(|| Self::from_action_context_unchecked::<C>(spend, output_note, alpha, rcv))
    }

    pub(crate) fn from_action_context_unchecked<C: OrchardCircuit>(
        spend: SpendInfo,
        output_note: Note,
        alpha: pallas::Scalar,
        rcv: ValueCommitTrapdoor,
    ) -> Self {
        let sender_address = spend.note.recipient();
        let rho_old = spend.note.rho();
        let psi_old = spend.note.rseed().psi(&rho_old);
        let rcm_old = spend.note.rseed().rcm(&rho_old);

        let rho_new = output_note.rho();
        let psi_new = output_note.rseed().psi(&rho_new);
        let rcm_new = output_note.rseed().rcm(&rho_new);

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
        }
    }
}

/// The verifying key for the Orchard Action circuit.
///
/// In the current type system, this could be a verifying key for either
/// the original Orchard Action circuit, or the OrchardZSA circuit.
#[derive(Debug)]
pub struct VerifyingKey {
    pub(crate) params: halo2_proofs::poly::commitment::Params<vesta::Affine>,
    pub(crate) vk: plonk::VerifyingKey<vesta::Affine>,
}

impl VerifyingKey {
    /// Builds the verifying key.
    pub fn build<C: OrchardCircuit>() -> Self {
        let params = halo2_proofs::poly::commitment::Params::new(K);
        let circuit: Circuit<C> = Default::default();

        let vk = plonk::keygen_vk(&params, &circuit).unwrap();

        VerifyingKey { params, vk }
    }
}

/// The proving key for the Orchard Action circuit.
///
/// In the current type system, this could be a proving key for either
/// the original Orchard Action circuit, or the OrchardZSA circuit.
#[derive(Debug)]
pub struct ProvingKey {
    params: halo2_proofs::poly::commitment::Params<vesta::Affine>,
    pk: plonk::ProvingKey<vesta::Affine>,
}

impl ProvingKey {
    /// Builds the proving key.
    pub fn build<C: OrchardCircuit>() -> Self {
        let params = halo2_proofs::poly::commitment::Params::new(K);
        let circuit: Circuit<C> = Default::default();

        let vk = plonk::keygen_vk(&params, &circuit).unwrap();
        let pk = plonk::keygen_pk(&params, vk, &circuit).unwrap();

        ProvingKey { params, pk }
    }
}

/// Public inputs to the Orchard Action circuit.
///
/// The `enable_zsa` field was introduced with the ZSA feature; it did not exist before.
/// In vanilla Orchard, `enable_zsa` is always false, so this method always appends a zero to the
/// instance vector. Since halo2_proofs pads instance values with zero, old proofs (without this
/// extra entry) and new proofs behave identically.
#[derive(Clone, Debug)]
pub struct Instance {
    pub(crate) anchor: Anchor,
    pub(crate) cv_net: ValueCommitment,
    pub(crate) nf_old: Nullifier,
    pub(crate) rk: VerificationKey<SpendAuth>,
    pub(crate) cmx: ExtractedNoteCommitment,
    pub(crate) enable_spend: bool,
    pub(crate) enable_output: bool,
    pub(crate) enable_zsa: bool,
}

impl Instance {
    /// Constructs an [`Instance`] from its constituent parts.
    ///
    /// This API can be used in combination with [`Proof::verify`] to build verification
    /// pipelines for many proofs, where you don't want to pass around the full bundle.
    /// Use [`Bundle::verify_proof`] instead if you have the full bundle.
    ///
    /// [`Bundle::verify_proof`]: crate::Bundle::verify_proof
    pub fn from_parts(
        anchor: Anchor,
        cv_net: ValueCommitment,
        nf_old: Nullifier,
        rk: VerificationKey<SpendAuth>,
        cmx: ExtractedNoteCommitment,
        flags: Flags,
    ) -> Self {
        Instance {
            anchor,
            cv_net,
            nf_old,
            rk,
            cmx,
            enable_spend: flags.spends_enabled(),
            enable_output: flags.outputs_enabled(),
            enable_zsa: flags.zsa_enabled(),
        }
    }

    /// Note: Before the ZSA feature was introduced, this method returned a 9-element instance slice.
    /// With ZSA, it now returns 10 elements, the last one corresponding to `enable_zsa`.
    /// In vanilla Orchard, `enable_zsa` is always false, so this extra element is always zero.
    /// Since halo2_proofs pads instance values with zero, old proofs (without this element)
    /// and new proofs behave identically.
    fn to_halo2_instance(&self) -> [[vesta::Scalar; 10]; 1] {
        let mut instance = [vesta::Scalar::zero(); 10];

        instance[ANCHOR] = self.anchor.inner();
        instance[CV_NET_X] = self.cv_net.x();
        instance[CV_NET_Y] = self.cv_net.y();
        instance[NF_OLD] = self.nf_old.0;

        let rk = pallas::Point::from_bytes(&self.rk.clone().into())
            .unwrap()
            .to_affine()
            .coordinates()
            .unwrap();

        instance[RK_X] = *rk.x();
        instance[RK_Y] = *rk.y();
        instance[CMX] = self.cmx.inner();
        instance[ENABLE_SPEND] = vesta::Scalar::from(u64::from(self.enable_spend));
        instance[ENABLE_OUTPUT] = vesta::Scalar::from(u64::from(self.enable_output));
        instance[ENABLE_ZSA] = vesta::Scalar::from(u64::from(self.enable_zsa));

        [instance]
    }
}

impl Proof {
    /// Creates a proof for the given circuits and instances.
    pub fn create<C: OrchardCircuit>(
        pk: &ProvingKey,
        circuits: &[Circuit<C>],
        instances: &[Instance],
        mut rng: impl RngCore,
    ) -> Result<Self, plonk::Error> {
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
    pub fn verify(&self, vk: &VerifyingKey, instances: &[Instance]) -> Result<(), plonk::Error> {
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
    /// Use this API if you want more control over how proof batches are processed. If you
    /// just want to batch-validate Orchard bundles, use [`bundle::BatchValidator`].
    ///
    /// [`bundle::BatchValidator`]: crate::bundle::BatchValidator
    pub fn add_to_batch(&self, batch: &mut BatchVerifier<vesta::Affine>, instances: Vec<Instance>) {
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
