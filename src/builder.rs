//! Logic for building Orchard components of transactions.

use core::fmt;
use core::iter;

use ff::Field;
use halo2_proofs::circuit::Value;
use nonempty::NonEmpty;
use pasta_curves::pallas;
use rand::{prelude::SliceRandom, CryptoRng, RngCore};

use crate::{
    action::Action,
    address::Address,
    bundle::{Authorization, Authorized, Bundle, Flags},
    circuit::{Circuit, Instance, Proof, ProvingKey},
    keys::{
        FullViewingKey, OutgoingViewingKey, Scope, SpendAuthorizingKey, SpendValidatingKey,
        SpendingKey,
    },
    note::{Note, TransmittedNoteCiphertext},
    note_encryption::OrchardNoteEncryption,
    primitives::redpallas::{self, Binding, SpendAuth},
    tree::{Anchor, MerklePath},
    value::{self, NoteValue, OverflowError, ValueCommitTrapdoor, ValueCommitment, ValueSum},
};

const MIN_ACTIONS: usize = 2;

/// An error type for the kinds of errors that can occur during bundle construction.
#[derive(Debug)]
pub enum Error {
    /// A bundle could not be built because required signatures were missing.
    MissingSignatures,
    /// An error occurred in the process of producing a proof for a bundle.
    Proof(halo2_proofs::plonk::Error),
    /// An overflow error occurred while attempting to construct the value
    /// for a bundle.
    ValueSum(value::OverflowError),
    /// External signature is not valid.
    InvalidExternalSignature,
    /// A signature is valid for more than one input. This should never happen if `alpha`
    /// is sampled correctly, and indicates a critical failure in randomness generation.
    DuplicateSignature,
}

impl From<halo2_proofs::plonk::Error> for Error {
    fn from(e: halo2_proofs::plonk::Error) -> Self {
        Error::Proof(e)
    }
}

impl From<value::OverflowError> for Error {
    fn from(e: value::OverflowError) -> Self {
        Error::ValueSum(e)
    }
}

/// Information about a specific note to be spent in an [`Action`].
#[derive(Debug)]
struct SpendInfo {
    dummy_sk: Option<SpendingKey>,
    fvk: FullViewingKey,
    scope: Scope,
    note: Note,
    merkle_path: MerklePath,
}

impl SpendInfo {
    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    fn dummy(rng: &mut impl RngCore) -> Self {
        let (sk, fvk, note) = Note::dummy(rng, None);
        let merkle_path = MerklePath::dummy(rng);

        SpendInfo {
            dummy_sk: Some(sk),
            fvk,
            // We use external scope to avoid unnecessary derivations, because the dummy
            // note's spending key is random and thus scoping is irrelevant.
            scope: Scope::External,
            note,
            merkle_path,
        }
    }
}

/// Information about a specific recipient to receive funds in an [`Action`].
#[derive(Debug)]
struct RecipientInfo {
    ovk: Option<OutgoingViewingKey>,
    recipient: Address,
    value: NoteValue,
    memo: Option<[u8; 512]>,
}

impl RecipientInfo {
    /// Defined in [Zcash Protocol Spec § 4.8.3: Dummy Notes (Orchard)][orcharddummynotes].
    ///
    /// [orcharddummynotes]: https://zips.z.cash/protocol/nu5.pdf#orcharddummynotes
    fn dummy(rng: &mut impl RngCore) -> Self {
        let fvk: FullViewingKey = (&SpendingKey::random(rng)).into();
        let recipient = fvk.address_at(0u32, Scope::External);

        RecipientInfo {
            ovk: None,
            recipient,
            value: NoteValue::zero(),
            memo: None,
        }
    }
}

/// Information about a specific [`Action`] we plan to build.
#[derive(Debug)]
struct ActionInfo {
    spend: SpendInfo,
    output: RecipientInfo,
    rcv: ValueCommitTrapdoor,
}

impl ActionInfo {
    fn new(spend: SpendInfo, output: RecipientInfo, rng: impl RngCore) -> Self {
        ActionInfo {
            spend,
            output,
            rcv: ValueCommitTrapdoor::random(rng),
        }
    }

    /// Returns the value sum for this action.
    fn value_sum(&self) -> ValueSum {
        self.spend.note.value() - self.output.value
    }

    /// Builds the action.
    ///
    /// Defined in [Zcash Protocol Spec § 4.7.3: Sending Notes (Orchard)][orchardsend].
    ///
    /// [orchardsend]: https://zips.z.cash/protocol/nu5.pdf#orchardsend
    fn build(self, mut rng: impl RngCore) -> (Action<SigningMetadata>, Circuit) {
        let v_net = self.value_sum();
        let cv_net = ValueCommitment::derive(v_net, self.rcv.clone());

        let nf_old = self.spend.note.nullifier(&self.spend.fvk);
        let sender_address = self.spend.note.recipient();
        let rho_old = self.spend.note.rho();
        let psi_old = self.spend.note.rseed().psi(&rho_old);
        let rcm_old = self.spend.note.rseed().rcm(&rho_old);
        let ak: SpendValidatingKey = self.spend.fvk.clone().into();
        let alpha = pallas::Scalar::random(&mut rng);
        let rk = ak.randomize(&alpha);

        let note = Note::new(self.output.recipient, self.output.value, nf_old, &mut rng);
        let cm_new = note.commitment();
        let cmx = cm_new.into();

        let encryptor = OrchardNoteEncryption::new(
            self.output.ovk,
            note,
            self.output.recipient,
            self.output.memo.unwrap_or_else(|| {
                let mut memo = [0; 512];
                memo[0] = 0xf6;
                memo
            }),
        );

        let encrypted_note = TransmittedNoteCiphertext {
            epk_bytes: encryptor.epk().to_bytes().0,
            enc_ciphertext: encryptor.encrypt_note_plaintext(),
            out_ciphertext: encryptor.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut rng),
        };

        (
            Action::from_parts(
                nf_old,
                rk,
                cmx,
                encrypted_note,
                cv_net,
                SigningMetadata {
                    dummy_ask: self.spend.dummy_sk.as_ref().map(SpendAuthorizingKey::from),
                    parts: SigningParts {
                        ak: ak.clone(),
                        alpha,
                    },
                },
            ),
            Circuit {
                path: Value::known(self.spend.merkle_path.auth_path()),
                pos: Value::known(self.spend.merkle_path.position()),
                g_d_old: Value::known(sender_address.g_d()),
                pk_d_old: Value::known(*sender_address.pk_d()),
                v_old: Value::known(self.spend.note.value()),
                rho_old: Value::known(rho_old),
                psi_old: Value::known(psi_old),
                rcm_old: Value::known(rcm_old),
                cm_old: Value::known(self.spend.note.commitment()),
                alpha: Value::known(alpha),
                ak: Value::known(ak),
                nk: Value::known(*self.spend.fvk.nk()),
                rivk: Value::known(self.spend.fvk.rivk(self.spend.scope)),
                g_d_new: Value::known(note.recipient().g_d()),
                pk_d_new: Value::known(*note.recipient().pk_d()),
                v_new: Value::known(note.value()),
                psi_new: Value::known(note.rseed().psi(&note.rho())),
                rcm_new: Value::known(note.rseed().rcm(&note.rho())),
                rcv: Value::known(self.rcv),
            },
        )
    }
}

/// A builder that constructs a [`Bundle`] from a set of notes to be spent, and recipients
/// to receive funds.
#[derive(Debug)]
pub struct Builder {
    spends: Vec<SpendInfo>,
    recipients: Vec<RecipientInfo>,
    flags: Flags,
    anchor: Anchor,
}

impl Builder {
    /// Constructs a new empty builder for an Orchard bundle.
    pub fn new(flags: Flags, anchor: Anchor) -> Self {
        Builder {
            spends: vec![],
            recipients: vec![],
            flags,
            anchor,
        }
    }

    /// Adds a note to be spent in this transaction.
    ///
    /// - `note` is a spendable note, obtained by trial-decrypting an [`Action`] using the
    ///   [`zcash_note_encryption`] crate instantiated with [`OrchardDomain`].
    /// - `merkle_path` can be obtained using the [`incrementalmerkletree`] crate
    ///   instantiated with [`MerkleHashOrchard`].
    ///
    /// Returns an error if the given Merkle path does not have the required anchor for
    /// the given note.
    ///
    /// [`OrchardDomain`]: crate::note_encryption::OrchardDomain
    /// [`MerkleHashOrchard`]: crate::tree::MerkleHashOrchard
    pub fn add_spend(
        &mut self,
        fvk: FullViewingKey,
        note: Note,
        merkle_path: MerklePath,
    ) -> Result<(), &'static str> {
        if !self.flags.spends_enabled() {
            return Err("Spends are not enabled for this builder");
        }

        // Consistency check: all anchors must be equal.
        let cm = note.commitment();
        let path_root: Anchor =
            <Option<_>>::from(merkle_path.root(cm.into())).ok_or("Derived the bottom anchor")?;
        if path_root != self.anchor {
            return Err("All anchors must be equal.");
        }

        // Check if note is internal or external.
        let scope = fvk
            .scope_for_address(&note.recipient())
            .ok_or("FullViewingKey does not correspond to the given note")?;

        self.spends.push(SpendInfo {
            dummy_sk: None,
            fvk,
            scope,
            note,
            merkle_path,
        });

        Ok(())
    }

    /// Adds an address which will receive funds in this transaction.
    pub fn add_recipient(
        &mut self,
        ovk: Option<OutgoingViewingKey>,
        recipient: Address,
        value: NoteValue,
        memo: Option<[u8; 512]>,
    ) -> Result<(), &'static str> {
        if !self.flags.outputs_enabled() {
            return Err("Outputs are not enabled for this builder");
        }

        self.recipients.push(RecipientInfo {
            ovk,
            recipient,
            value,
            memo,
        });

        Ok(())
    }

    /// The net value of the bundle to be built. The value of all spends,
    /// minus the value of all outputs.
    ///
    /// Useful for balancing a transaction, as the value balance of an individual bundle
    /// can be non-zero. Each bundle's value balance is [added] to the transparent
    /// transaction value pool, which [must not have a negative value]. (If it were
    /// negative, the transaction would output more value than it receives in inputs.)
    ///
    /// [added]: https://zips.z.cash/protocol/protocol.pdf#orchardbalance
    /// [must not have a negative value]: https://zips.z.cash/protocol/protocol.pdf#transactions
    pub fn value_balance<V: TryFrom<i64>>(&self) -> Result<V, value::OverflowError> {
        let value_balance = self
            .spends
            .iter()
            .map(|spend| spend.note.value() - NoteValue::zero())
            .chain(
                self.recipients
                    .iter()
                    .map(|recipient| NoteValue::zero() - recipient.value),
            )
            .fold(Some(ValueSum::zero()), |acc, note_value| acc? + note_value)
            .ok_or(OverflowError)?;
        i64::try_from(value_balance).and_then(|i| V::try_from(i).map_err(|_| value::OverflowError))
    }

    /// Builds a bundle containing the given spent notes and recipients.
    ///
    /// The returned bundle will have no proof or signatures; these can be applied with
    /// [`Bundle::create_proof`] and [`Bundle::apply_signatures`] respectively.
    pub fn build<V: TryFrom<i64>>(
        mut self,
        mut rng: impl RngCore,
    ) -> Result<Bundle<InProgress<Unproven, Unauthorized>, V>, Error> {
        // Pair up the spends and recipients, extending with dummy values as necessary.
        let pre_actions: Vec<_> = {
            let num_spends = self.spends.len();
            let num_recipients = self.recipients.len();
            let num_actions = [num_spends, num_recipients, MIN_ACTIONS]
                .iter()
                .max()
                .cloned()
                .unwrap();

            self.spends.extend(
                iter::repeat_with(|| SpendInfo::dummy(&mut rng)).take(num_actions - num_spends),
            );
            self.recipients.extend(
                iter::repeat_with(|| RecipientInfo::dummy(&mut rng))
                    .take(num_actions - num_recipients),
            );

            // Shuffle the spends and recipients, so that learning the position of a
            // specific spent note or output note doesn't reveal anything on its own about
            // the meaning of that note in the transaction context.
            self.spends.shuffle(&mut rng);
            self.recipients.shuffle(&mut rng);

            self.spends
                .into_iter()
                .zip(self.recipients.into_iter())
                .map(|(spend, recipient)| ActionInfo::new(spend, recipient, &mut rng))
                .collect()
        };

        // Move some things out of self that we will need.
        let flags = self.flags;
        let anchor = self.anchor;

        // Determine the value balance for this bundle, ensuring it is valid.
        let value_balance = pre_actions
            .iter()
            .fold(Some(ValueSum::zero()), |acc, action| {
                acc? + action.value_sum()
            })
            .ok_or(OverflowError)?;

        let result_value_balance: V = i64::try_from(value_balance)
            .map_err(Error::ValueSum)
            .and_then(|i| V::try_from(i).map_err(|_| Error::ValueSum(value::OverflowError)))?;

        // Compute the transaction binding signing key.
        let bsk = pre_actions
            .iter()
            .map(|a| &a.rcv)
            .sum::<ValueCommitTrapdoor>()
            .into_bsk();

        // Create the actions.
        let (actions, circuits): (Vec<_>, Vec<_>) =
            pre_actions.into_iter().map(|a| a.build(&mut rng)).unzip();

        // Verify that bsk and bvk are consistent.
        let bvk = (actions.iter().map(|a| a.cv_net()).sum::<ValueCommitment>()
            - ValueCommitment::derive(value_balance, ValueCommitTrapdoor::zero()))
        .into_bvk();
        assert_eq!(redpallas::VerificationKey::from(&bsk), bvk);

        Ok(Bundle::from_parts(
            NonEmpty::from_vec(actions).unwrap(),
            flags,
            result_value_balance,
            anchor,
            InProgress {
                proof: Unproven { circuits },
                sigs: Unauthorized { bsk },
            },
        ))
    }
}

/// Marker trait representing bundle signatures in the process of being created.
pub trait InProgressSignatures: fmt::Debug {
    /// The authorization type of an Orchard action in the process of being authorized.
    type SpendAuth: fmt::Debug;
}

/// Marker for a bundle in the process of being built.
#[derive(Clone, Debug)]
pub struct InProgress<P, S: InProgressSignatures> {
    proof: P,
    sigs: S,
}

impl<P: fmt::Debug, S: InProgressSignatures> Authorization for InProgress<P, S> {
    type SpendAuth = S::SpendAuth;
}

/// Marker for a bundle without a proof.
///
/// This struct contains the private data needed to create a [`Proof`] for a [`Bundle`].
#[derive(Clone, Debug)]
pub struct Unproven {
    circuits: Vec<Circuit>,
}

impl<S: InProgressSignatures> InProgress<Unproven, S> {
    /// Creates the proof for this bundle.
    pub fn create_proof(
        &self,
        pk: &ProvingKey,
        instances: &[Instance],
        rng: impl RngCore,
    ) -> Result<Proof, halo2_proofs::plonk::Error> {
        Proof::create(pk, &self.proof.circuits, instances, rng)
    }
}

impl<S: InProgressSignatures, V> Bundle<InProgress<Unproven, S>, V> {
    /// Creates the proof for this bundle.
    pub fn create_proof(
        self,
        pk: &ProvingKey,
        mut rng: impl RngCore,
    ) -> Result<Bundle<InProgress<Proof, S>, V>, Error> {
        let instances: Vec<_> = self
            .actions()
            .iter()
            .map(|a| a.to_instance(*self.flags(), *self.anchor()))
            .collect();
        self.try_map_authorization(
            &mut (),
            |_, _, a| Ok(a),
            |_, auth| {
                let proof = auth.create_proof(pk, &instances, &mut rng)?;
                Ok(InProgress {
                    proof,
                    sigs: auth.sigs,
                })
            },
        )
    }
}

/// The parts needed to sign an [`Action`].
#[derive(Clone, Debug)]
pub struct SigningParts {
    /// The spend validating key for this action. Used to match spend authorizing keys to
    /// actions they can create signatures for.
    ak: SpendValidatingKey,
    /// The randomization needed to derive the actual signing key for this note.
    alpha: pallas::Scalar,
}

/// Marker for an unauthorized bundle with no signatures.
#[derive(Clone, Debug)]
pub struct Unauthorized {
    bsk: redpallas::SigningKey<Binding>,
}

impl InProgressSignatures for Unauthorized {
    type SpendAuth = SigningMetadata;
}

/// Container for metadata needed to sign an [`Action`].
#[derive(Clone, Debug)]
pub struct SigningMetadata {
    /// If this action is spending a dummy note, this field holds that note's spend
    /// authorizing key.
    ///
    /// These keys are used automatically in [`Bundle<Unauthorized>::prepare`] or
    /// [`Bundle<Unauthorized>::apply_signatures`] to sign dummy spends.
    dummy_ask: Option<SpendAuthorizingKey>,
    parts: SigningParts,
}

/// Marker for a partially-authorized bundle, in the process of being signed.
#[derive(Debug)]
pub struct PartiallyAuthorized {
    binding_signature: redpallas::Signature<Binding>,
    sighash: [u8; 32],
}

impl InProgressSignatures for PartiallyAuthorized {
    type SpendAuth = MaybeSigned;
}

/// A heisen[`Signature`] for a particular [`Action`].
///
/// [`Signature`]: redpallas::Signature
#[derive(Debug)]
pub enum MaybeSigned {
    /// The information needed to sign this [`Action`].
    SigningMetadata(SigningParts),
    /// The signature for this [`Action`].
    Signature(redpallas::Signature<SpendAuth>),
}

impl MaybeSigned {
    fn finalize(self) -> Result<redpallas::Signature<SpendAuth>, Error> {
        match self {
            Self::Signature(sig) => Ok(sig),
            _ => Err(Error::MissingSignatures),
        }
    }
}

impl<P: fmt::Debug, V> Bundle<InProgress<P, Unauthorized>, V> {
    /// Loads the sighash into this bundle, preparing it for signing.
    ///
    /// This API ensures that all signatures are created over the same sighash.
    pub fn prepare<R: RngCore + CryptoRng>(
        self,
        mut rng: R,
        sighash: [u8; 32],
    ) -> Bundle<InProgress<P, PartiallyAuthorized>, V> {
        self.map_authorization(
            &mut rng,
            |rng, _, SigningMetadata { dummy_ask, parts }| {
                // We can create signatures for dummy spends immediately.
                dummy_ask
                    .map(|ask| ask.randomize(&parts.alpha).sign(rng, &sighash))
                    .map(MaybeSigned::Signature)
                    .unwrap_or(MaybeSigned::SigningMetadata(parts))
            },
            |rng, auth| InProgress {
                proof: auth.proof,
                sigs: PartiallyAuthorized {
                    binding_signature: auth.sigs.bsk.sign(rng, &sighash),
                    sighash,
                },
            },
        )
    }
}

impl<V> Bundle<InProgress<Proof, Unauthorized>, V> {
    /// Applies signatures to this bundle, in order to authorize it.
    ///
    /// This is a helper method that wraps [`Bundle::prepare`], [`Bundle::sign`], and
    /// [`Bundle::finalize`].
    pub fn apply_signatures<R: RngCore + CryptoRng>(
        self,
        mut rng: R,
        sighash: [u8; 32],
        signing_keys: &[SpendAuthorizingKey],
    ) -> Result<Bundle<Authorized, V>, Error> {
        signing_keys
            .iter()
            .fold(self.prepare(&mut rng, sighash), |partial, ask| {
                partial.sign(&mut rng, ask)
            })
            .finalize()
    }
}

impl<P: fmt::Debug, V> Bundle<InProgress<P, PartiallyAuthorized>, V> {
    /// Signs this bundle with the given [`SpendAuthorizingKey`].
    ///
    /// This will apply signatures for all notes controlled by this spending key.
    pub fn sign<R: RngCore + CryptoRng>(self, mut rng: R, ask: &SpendAuthorizingKey) -> Self {
        let expected_ak = ask.into();
        self.map_authorization(
            &mut rng,
            |rng, partial, maybe| match maybe {
                MaybeSigned::SigningMetadata(parts) if parts.ak == expected_ak => {
                    MaybeSigned::Signature(
                        ask.randomize(&parts.alpha).sign(rng, &partial.sigs.sighash),
                    )
                }
                s => s,
            },
            |_, partial| partial,
        )
    }
    /// Appends externally computed [`Signature`]s.
    ///
    /// Each signature will be applied to the one input for which it is valid. An error
    /// will be returned if the signature is not valid for any inputs, or if it is valid
    /// for more than one input.
    ///
    /// [`Signature`]: redpallas::Signature
    pub fn append_signatures(
        self,
        signatures: &[redpallas::Signature<SpendAuth>],
    ) -> Result<Self, Error> {
        signatures.iter().try_fold(self, Self::append_signature)
    }

    fn append_signature(self, signature: &redpallas::Signature<SpendAuth>) -> Result<Self, Error> {
        let mut signature_valid_for = 0usize;
        let bundle = self.map_authorization(
            &mut signature_valid_for,
            |valid_for, partial, maybe| match maybe {
                MaybeSigned::SigningMetadata(parts) => {
                    let rk = parts.ak.randomize(&parts.alpha);
                    if rk.verify(&partial.sigs.sighash[..], signature).is_ok() {
                        *valid_for += 1;
                        MaybeSigned::Signature(signature.clone())
                    } else {
                        // Signature isn't for this input.
                        MaybeSigned::SigningMetadata(parts)
                    }
                }
                s => s,
            },
            |_, partial| partial,
        );
        match signature_valid_for {
            0 => Err(Error::InvalidExternalSignature),
            1 => Ok(bundle),
            _ => Err(Error::DuplicateSignature),
        }
    }
}

impl<V> Bundle<InProgress<Proof, PartiallyAuthorized>, V> {
    /// Finalizes this bundle, enabling it to be included in a transaction.
    ///
    /// Returns an error if any signatures are missing.
    pub fn finalize(self) -> Result<Bundle<Authorized, V>, Error> {
        self.try_map_authorization(
            &mut (),
            |_, _, maybe| maybe.finalize(),
            |_, partial| {
                Ok(Authorized::from_parts(
                    partial.proof,
                    partial.sigs.binding_signature,
                ))
            },
        )
    }
}

/// Generators for property testing.
#[cfg(any(test, feature = "test-dependencies"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-dependencies")))]
pub mod testing {
    use core::fmt::Debug;
    use incrementalmerkletree::{bridgetree::BridgeTree, Tree};
    use rand::{rngs::StdRng, CryptoRng, SeedableRng};

    use proptest::collection::vec;
    use proptest::prelude::*;

    use crate::{
        address::testing::arb_address,
        bundle::{Authorized, Bundle, Flags},
        circuit::ProvingKey,
        keys::{testing::arb_spending_key, FullViewingKey, SpendAuthorizingKey, SpendingKey},
        note::testing::arb_note,
        tree::{Anchor, MerkleHashOrchard, MerklePath},
        value::{testing::arb_positive_note_value, NoteValue, MAX_NOTE_VALUE},
        Address, Note,
    };

    use super::Builder;

    /// An intermediate type used for construction of arbitrary
    /// bundle values. This type is required because of a limitation
    /// of the proptest prop_compose! macro which does not correctly
    /// handle polymorphic generator functions. Instead of generating
    /// a bundle directly, we generate the bundle inputs, and then
    /// are able to use the `build` function to construct the bundle
    /// from these inputs, but using a `ValueBalance` implementation that
    /// is defined by the end user.
    #[derive(Debug)]
    struct ArbitraryBundleInputs<R> {
        rng: R,
        sk: SpendingKey,
        anchor: Anchor,
        notes: Vec<(Note, MerklePath)>,
        recipient_amounts: Vec<(Address, NoteValue)>,
    }

    impl<R: RngCore + CryptoRng> ArbitraryBundleInputs<R> {
        /// Create a bundle from the set of arbitrary bundle inputs.
        fn into_bundle<V: TryFrom<i64>>(mut self) -> Bundle<Authorized, V> {
            let fvk = FullViewingKey::from(&self.sk);
            let flags = Flags::from_parts(true, true);
            let mut builder = Builder::new(flags, self.anchor);

            for (note, path) in self.notes.into_iter() {
                builder.add_spend(fvk.clone(), note, path).unwrap();
            }

            for (addr, value) in self.recipient_amounts.into_iter() {
                let scope = fvk.scope_for_address(&addr).unwrap();
                let ovk = fvk.to_ovk(scope);

                builder
                    .add_recipient(Some(ovk.clone()), addr, value, None)
                    .unwrap();
            }

            let pk = ProvingKey::build();
            builder
                .build(&mut self.rng)
                .unwrap()
                .create_proof(&pk, &mut self.rng)
                .unwrap()
                .prepare(&mut self.rng, [0; 32])
                .sign(&mut self.rng, &SpendAuthorizingKey::from(&self.sk))
                .finalize()
                .unwrap()
        }
    }

    prop_compose! {
        /// Produce a random valid Orchard bundle.
        fn arb_bundle_inputs(sk: SpendingKey)
        (
            n_notes in 1usize..30,
            n_recipients in 1..30,

        )
        (
            // generate note values that we're certain won't exceed MAX_NOTE_VALUE in total
            notes in vec(
                arb_positive_note_value(MAX_NOTE_VALUE / n_notes as u64).prop_flat_map(arb_note),
                n_notes
            ),
            recipient_amounts in vec(
                arb_address().prop_flat_map(move |a| {
                    arb_positive_note_value(MAX_NOTE_VALUE / n_recipients as u64)
                        .prop_map(move |v| (a, v))
                }),
                n_recipients as usize
            ),
            rng_seed in prop::array::uniform32(prop::num::u8::ANY)
        ) -> ArbitraryBundleInputs<StdRng> {
            const MERKLE_DEPTH_ORCHARD: u8 = crate::constants::MERKLE_DEPTH_ORCHARD as u8;
            let mut tree = BridgeTree::<MerkleHashOrchard, MERKLE_DEPTH_ORCHARD>::new(100);
            let mut notes_and_auth_paths: Vec<(Note, MerklePath)> = Vec::new();

            for note in notes.iter() {
                let leaf = MerkleHashOrchard::from_cmx(&note.commitment().into());
                tree.append(&leaf);
                let position = tree.witness().expect("tree is not empty");

                let root = tree.root(0).unwrap();
                let path = MerklePath::from((position, tree.authentication_path(position, &root).expect("we just witnessed the path")));
                notes_and_auth_paths.push((*note, path));
            }

            ArbitraryBundleInputs {
                rng: StdRng::from_seed(rng_seed),
                sk,
                anchor: tree.root(0).unwrap().into(),
                notes: notes_and_auth_paths,
                recipient_amounts
            }
        }
    }

    /// Produce an arbitrary valid Orchard bundle using a random spending key.
    pub fn arb_bundle<V: TryFrom<i64> + Debug>() -> impl Strategy<Value = Bundle<Authorized, V>> {
        arb_spending_key()
            .prop_flat_map(arb_bundle_inputs)
            .prop_map(|inputs| inputs.into_bundle::<V>())
    }

    /// Produce an arbitrary valid Orchard bundle using a specified spending key.
    pub fn arb_bundle_with_key<V: TryFrom<i64> + Debug>(
        k: SpendingKey,
    ) -> impl Strategy<Value = Bundle<Authorized, V>> {
        arb_bundle_inputs(k).prop_map(|inputs| inputs.into_bundle::<V>())
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;

    use super::Builder;
    use crate::{
        bundle::{Authorized, Bundle, Flags},
        circuit::ProvingKey,
        constants::MERKLE_DEPTH_ORCHARD,
        keys::{FullViewingKey, Scope, SpendingKey},
        tree::EMPTY_ROOTS,
        value::NoteValue,
    };

    #[test]
    fn shielding_bundle() {
        let pk = ProvingKey::build();
        let mut rng = OsRng;

        let sk = SpendingKey::random(&mut rng);
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        let mut builder = Builder::new(
            Flags::from_parts(true, true),
            EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
        );

        builder
            .add_recipient(None, recipient, NoteValue::from_raw(5000), None)
            .unwrap();
        let balance: i64 = builder.value_balance().unwrap();
        assert_eq!(balance, -5000);

        let bundle: Bundle<Authorized, i64> = builder
            .build(&mut rng)
            .unwrap()
            .create_proof(&pk, &mut rng)
            .unwrap()
            .prepare(&mut rng, [0; 32])
            .finalize()
            .unwrap();
        assert_eq!(bundle.value_balance(), &(-5000))
    }
}
