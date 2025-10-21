//! PCZT support for Orchard.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use getset::Getters;
use pasta_curves::pallas;
use zcash_note_encryption::{note_bytes::NoteBytes, OutgoingCipherKey};
use zip32::ChildIndex;

use crate::{
    bundle::Flags,
    keys::{FullViewingKey, SpendingKey},
    note::{
        AssetBase, ExtractedNoteCommitment, Nullifier, RandomSeed, Rho, TransmittedNoteCiphertext,
    },
    orchard_sighash_versioning::VerSpendAuthSig,
    primitives::{
        redpallas::{self, Binding, SpendAuth},
        OrchardPrimitives,
    },
    tree::MerklePath,
    value::{NoteValue, ValueCommitTrapdoor, ValueCommitment, ValueSum},
    Address, Anchor, Proof,
};

mod parse;
pub use parse::ParseError;

mod verify;
pub use verify::VerifyError;

mod io_finalizer;
pub use io_finalizer::IoFinalizerError;

mod updater;
pub use updater::{ActionUpdater, Updater, UpdaterError};

#[cfg(feature = "circuit")]
mod prover;
#[cfg(feature = "circuit")]
pub use prover::ProverError;

mod signer;
pub use signer::SignerError;

mod tx_extractor;
pub use tx_extractor::{TxExtractorError, Unbound};

/// PCZT fields that are specific to producing the transaction's Orchard bundle (if any).
///
/// This struct is for representing Orchard in a partially-created transaction. If you
/// have a fully-created transaction, use [the regular `Bundle` struct].
///
/// [the regular `Bundle` struct]: crate::Bundle
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct Bundle {
    /// The Orchard actions in this bundle.
    ///
    /// Entries are added by the Constructor, and modified by an Updater, IO Finalizer,
    /// Signer, Combiner, or Spend Finalizer.
    pub(crate) actions: Vec<Action>,

    /// The flags for the Orchard bundle.
    ///
    /// This is set by the Creator. The Constructor MUST only add spends and outputs that
    /// are consistent with these flags (i.e. are dummies as appropriate).
    pub(crate) flags: Flags,

    /// The sum of the values of all `actions`.
    ///
    /// This is initialized by the Creator, and updated by the Constructor as spends or
    /// outputs are added to the PCZT. It enables per-spend and per-output values to be
    /// redacted from the PCZT after they are no longer necessary.
    pub(crate) value_sum: ValueSum,

    /// Assets intended for burning
    ///
    /// Set by the Constructor.
    pub(crate) burn: Vec<(AssetBase, NoteValue)>,

    /// The Orchard anchor for this transaction.
    ///
    /// Set by the Creator.
    pub(crate) anchor: Anchor,

    /// Block height after which this Bundle's Actions are invalid by consensus.
    ///
    /// For the OrchardZSA protocol, `expiry_height` is set to 0, indicating no expiry.
    /// This field is reserved for future use.
    ///
    /// Set by the Constructor.
    pub(crate) expiry_height: u32,

    /// The Orchard bundle proof.
    ///
    /// This is `None` until it is set by the Prover.
    pub(crate) zkproof: Option<Proof>,

    /// The Orchard binding signature signing key.
    ///
    /// - This is `None` until it is set by the IO Finalizer.
    /// - The Transaction Extractor uses this to produce the binding signature.
    pub(crate) bsk: Option<redpallas::SigningKey<Binding>>,
}

impl Bundle {
    /// Returns a mutable reference to the actions in this bundle.
    ///
    /// This is used by Signers to apply signatures with [`Action::sign`].
    ///
    /// Note: updating the `Action`s via the returned slice will not update other
    /// fields of the bundle dependent on them, such as `value_sum` and `bsk`.
    pub fn actions_mut(&mut self) -> &mut [Action] {
        &mut self.actions
    }
}

/// PCZT fields that are specific to producing an Orchard action within a transaction.
///
/// This struct is for representing Orchard actions in a partially-created transaction.
/// If you have a fully-created transaction, use [the regular `Action` struct].
///
/// [the regular `Action` struct]: crate::Action
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct Action {
    /// A commitment to the net value created or consumed by this action.
    pub(crate) cv_net: ValueCommitment,

    /// The spend half of this action.
    pub(crate) spend: Spend,

    /// The output half of this action.
    pub(crate) output: Output,

    /// The value commitment randomness.
    ///
    /// - This is set by the Constructor.
    /// - The IO Finalizer compresses it into the bsk.
    /// - This is required by the Prover.
    /// - This may be used by Signers to verify that the value correctly matches `cv`.
    ///
    /// This opens `cv` for all participants. For Signers who don't need this information,
    /// or after proofs / signatures have been applied, this can be redacted.
    pub(crate) rcv: Option<ValueCommitTrapdoor>,
}

/// Information about an Orchard spend within a transaction.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct Spend {
    /// The nullifier of the note being spent.
    pub(crate) nullifier: Nullifier,

    /// The randomized verification key for the note being spent.
    pub(crate) rk: redpallas::VerificationKey<SpendAuth>,

    /// The versioned spend authorization signature.
    ///
    /// This is set by the Signer.
    pub(crate) spend_auth_sig: Option<VerSpendAuthSig>,

    /// The address that received the note being spent.
    ///
    /// - This is set by the Constructor (or Updater?).
    /// - This is required by the Prover.
    pub(crate) recipient: Option<Address>,

    /// The value of the input being spent.
    ///
    /// - This is required by the Prover.
    /// - This may be used by Signers to verify that the value matches `cv`, and to
    ///   confirm the values and change involved in the transaction.
    ///
    /// This exposes the input value to all participants. For Signers who don't need this
    /// information, or after signatures have been applied, this can be redacted.
    pub(crate) value: Option<NoteValue>,

    /// The asset of this Action.
    ///
    /// - This is set by the Constructor.
    /// - This is required by the Prover.
    pub(crate) asset: Option<AssetBase>,

    /// The rho value for the note being spent.
    ///
    /// - This is set by the Constructor.
    /// - This is required by the Prover.
    //
    // TODO: This could be merged with `rseed` into a tuple. `recipient` and `value` are
    // separate because they might need to be independently redacted. (For which role?)
    pub(crate) rho: Option<Rho>,

    /// The seed randomness for the note being spent.
    ///
    /// - This is set by the Constructor.
    /// - This is required by the Prover.
    pub(crate) rseed: Option<RandomSeed>,

    /// The seed randomness for split notes.
    ///
    /// - This is set by the Constructor.
    /// - This is required by the Prover.
    pub(crate) rseed_split_note: Option<RandomSeed>,

    /// The full viewing key that received the note being spent.
    ///
    /// - This is set by the Updater.
    /// - This is required by the Prover.
    pub(crate) fvk: Option<FullViewingKey>,

    /// A witness from the note to the bundle's anchor.
    ///
    /// - This is set by the Updater.
    /// - This is required by the Prover.
    pub(crate) witness: Option<MerklePath>,

    /// The spend authorization randomizer.
    ///
    /// - This is chosen by the Constructor.
    /// - This is required by the Signer for creating `spend_auth_sig`, and may be used to
    ///   validate `rk`.
    /// - After`zkproof` / `spend_auth_sig` has been set, this can be redacted.
    pub(crate) alpha: Option<pallas::Scalar>,

    /// A flag to indicate whether the value of the SpendInfo will be counted in the `ValueSum` of the action.
    ///
    /// - This is chosen by the Constructor.
    /// - This is required by the Prover.
    pub(crate) split_flag: Option<bool>,

    /// The ZIP 32 derivation path at which the spending key can be found for the note
    /// being spent.
    pub(crate) zip32_derivation: Option<Zip32Derivation>,

    /// The spending key for this spent note, if it is a dummy note.
    ///
    /// - This is chosen by the Constructor.
    /// - This is required by the IO Finalizer, and is cleared by it once used.
    /// - Signers MUST reject PCZTs that contain `dummy_sk` values.
    pub(crate) dummy_sk: Option<SpendingKey>,

    /// Proprietary fields related to the note being spent.
    pub(crate) proprietary: BTreeMap<String, Vec<u8>>,
}

/// Information about an Orchard output within a transaction.
#[derive(Getters)]
#[getset(get = "pub")]
pub struct Output {
    /// A commitment to the new note being created.
    pub(crate) cmx: ExtractedNoteCommitment,

    /// The transmitted note ciphertext.
    ///
    /// This contains the following PCZT fields:
    /// - `ephemeral_key`
    /// - `enc_ciphertext`
    /// - `out_ciphertext`
    pub(crate) encrypted_note: PcztTransmittedNoteCiphertext,

    /// The address that will receive the output.
    ///
    /// - This is set by the Constructor.
    /// - This is required by the Prover.
    /// - The Signer can use `recipient` and `rseed` (if present) to verify that
    ///   `enc_ciphertext` is correctly encrypted (and contains a note plaintext matching
    ///   the public commitments), and to confirm the value of the memo.
    pub(crate) recipient: Option<Address>,

    /// The value of the output.
    ///
    /// This may be used by Signers to verify that the value matches `cv`, and to confirm
    /// the values and change involved in the transaction.
    ///
    /// This exposes the value to all participants. For Signers who don't need this
    /// information, we can drop the values and compress the rcvs into the bsk global.
    pub(crate) value: Option<NoteValue>,

    /// The seed randomness for the output.
    ///
    /// - This is set by the Constructor.
    /// - This is required by the Prover.
    /// - The Signer can use `recipient` and `rseed` (if present) to verify that
    ///   `enc_ciphertext` is correctly encrypted (and contains a note plaintext matching
    ///   the public commitments), and to confirm the value of the memo.
    pub(crate) rseed: Option<RandomSeed>,

    /// The `ock` value used to encrypt `out_ciphertext`.
    ///
    /// This enables Signers to verify that `out_ciphertext` is correctly encrypted.
    ///
    /// This may be `None` if the Constructor added the output using an OVK policy of
    /// "None", to make the output unrecoverable from the chain by the sender.
    pub(crate) ock: Option<OutgoingCipherKey>,

    /// The ZIP 32 derivation path at which the spending key can be found for the output.
    pub(crate) zip32_derivation: Option<Zip32Derivation>,

    /// The user-facing address to which this output is being sent, if any.
    ///
    /// - This is set by an Updater.
    /// - Signers must parse this address (if present) and confirm that it contains
    ///   `recipient` (either directly, or e.g. as a receiver within a Unified Address).
    pub(crate) user_address: Option<String>,

    /// Proprietary fields related to the note being created.
    pub(crate) proprietary: BTreeMap<String, Vec<u8>>,
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Output")
            .field("cmx", &self.cmx)
            .field("encrypted_note", &self.encrypted_note)
            .field("recipient", &self.recipient)
            .field("value", &self.value)
            .field("rseed", &self.rseed)
            .field("zip32_derivation", &self.zip32_derivation)
            .field("user_address", &self.user_address)
            .field("proprietary", &self.proprietary)
            .finish_non_exhaustive()
    }
}

/// The ZIP 32 derivation path at which a key can be found.
#[derive(Debug, Getters, PartialEq, Eq)]
#[getset(get = "pub")]
pub struct Zip32Derivation {
    /// The [ZIP 32 seed fingerprint](https://zips.z.cash/zip-0032#seed-fingerprints).
    seed_fingerprint: [u8; 32],

    /// The sequence of indices corresponding to the shielded HD path.
    derivation_path: Vec<ChildIndex>,
}

impl Zip32Derivation {
    /// Extracts the ZIP 32 account index from this derivation path.
    ///
    /// Returns `None` if the seed fingerprints don't match, or if this is a non-standard
    /// derivation path.
    pub fn extract_account_index(
        &self,
        seed_fp: &zip32::fingerprint::SeedFingerprint,
        expected_coin_type: zip32::ChildIndex,
    ) -> Option<zip32::AccountId> {
        if self.seed_fingerprint == seed_fp.to_bytes() {
            match &self.derivation_path[..] {
                [purpose, coin_type, account_index]
                    if purpose == &zip32::ChildIndex::hardened(32)
                        && coin_type == &expected_coin_type =>
                {
                    Some(
                        zip32::AccountId::try_from(account_index.index() - (1 << 31))
                            .expect("zip32::ChildIndex only supports hardened"),
                    )
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

/// An encrypted note.
#[derive(Clone, Debug)]
pub struct PcztTransmittedNoteCiphertext {
    /// The serialization of the ephemeral public key
    pub epk_bytes: [u8; 32],
    /// The encrypted note ciphertext
    pub enc_ciphertext: Vec<u8>,
    /// An encrypted value that allows the holder of the outgoing cipher
    /// key for the note to recover the note plaintext.
    pub out_ciphertext: [u8; 80],
}

impl<P: OrchardPrimitives> From<TransmittedNoteCiphertext<P>> for PcztTransmittedNoteCiphertext {
    fn from(transmitted: TransmittedNoteCiphertext<P>) -> Self {
        PcztTransmittedNoteCiphertext {
            epk_bytes: transmitted.epk_bytes,
            enc_ciphertext: transmitted.enc_ciphertext.as_ref().to_vec(),
            out_ciphertext: transmitted.out_ciphertext,
        }
    }
}

impl<P: OrchardPrimitives> From<PcztTransmittedNoteCiphertext> for TransmittedNoteCiphertext<P> {
    fn from(ciphertext: PcztTransmittedNoteCiphertext) -> Self {
        TransmittedNoteCiphertext {
            epk_bytes: ciphertext.epk_bytes,
            enc_ciphertext: P::NoteCiphertextBytes::from_slice(&ciphertext.enc_ciphertext)
                .expect("Failed to parse enc_ciphertext: data may be corrupt or incorrect size"),
            out_ciphertext: ciphertext.out_ciphertext,
        }
    }
}

#[cfg(test)]
mod tests {
    use blake2b_simd::Hash as Blake2bHash;
    use ff::{Field, PrimeField};
    use incrementalmerkletree::{Marking, Retention};
    use pasta_curves::pallas;
    use rand::{rngs::StdRng, SeedableRng};
    use shardtree::{store::memory::MemoryShardStore, ShardTree};

    use crate::{
        builder::{Builder, BundleType},
        bundle::commitments::hash_bundle_txid_data,
        circuit::ProvingKey,
        constants::MERKLE_DEPTH_ORCHARD,
        keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
        note::{AssetBase, ExtractedNoteCommitment, RandomSeed, Rho},
        orchard_flavor::{OrchardFlavor, OrchardVanilla, OrchardZSA},
        pczt::Zip32Derivation,
        tree::{MerkleHashOrchard, EMPTY_ROOTS},
        value::NoteValue,
        Note,
    };

    fn shielding_bundle<FL: OrchardFlavor>(bundle_type: BundleType) -> Blake2bHash {
        let pk = ProvingKey::build::<FL>();
        let mut rng = StdRng::seed_from_u64(1u64);

        let sk = SpendingKey::random(&mut rng);
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        // Run the Creator and Constructor roles.
        let mut builder = Builder::new(bundle_type, EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into());
        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(5000),
                AssetBase::native(),
                [0u8; 512],
            )
            .unwrap();
        let balance: i64 = builder.value_balance().unwrap();
        assert_eq!(balance, -5000);
        let mut pczt_bundle = builder.build_for_pczt::<FL>(&mut rng).unwrap().0;

        // Run the IO Finalizer role.
        let sighash = [0; 32];
        pczt_bundle.finalize_io(sighash, rng.clone()).unwrap();

        // Run the Prover role.
        pczt_bundle.create_proof::<FL, _>(&pk, rng.clone()).unwrap();

        // Run the Transaction Extractor role.
        let bundle = pczt_bundle.extract::<i64, FL>().unwrap().unwrap();

        let orchard_digest = hash_bundle_txid_data(&bundle);

        assert_eq!(bundle.value_balance(), &(-5000));
        // We can successfully bind the bundle.
        bundle.apply_binding_signature(sighash, rng).unwrap();

        orchard_digest
    }

    #[test]
    fn shielding_bundle_orchard_zsa() {
        let orchard_digest = shielding_bundle::<OrchardZSA>(BundleType::DEFAULT_ZSA);
        assert_eq!(
            orchard_digest.as_bytes(),
            // Locks the `orchard_digest` for OrchardZSA
            &[
                164, 99, 147, 6, 19, 86, 66, 138, 59, 232, 52, 111, 235, 197, 66, 151, 86, 53, 65,
                250, 3, 129, 217, 11, 97, 160, 68, 75, 195, 188, 76, 161
            ],
        );
        let orchard_digest = shielding_bundle::<OrchardZSA>(BundleType::DEFAULT_VANILLA);
        assert_eq!(
            orchard_digest.as_bytes(),
            // Locks the `orchard_digest` for OrchardZSA
            &[
                208, 245, 141, 8, 182, 182, 91, 48, 64, 24, 222, 106, 63, 72, 11, 99, 216, 111,
                114, 94, 36, 153, 68, 253, 137, 240, 20, 184, 160, 192, 10, 155
            ],
        );
    }

    #[test]
    fn shielding_bundle_orchard_vanilla() {
        let orchard_digest = shielding_bundle::<OrchardVanilla>(BundleType::DEFAULT_VANILLA);
        assert_eq!(
            orchard_digest.as_bytes(),
            // `orchard_digest` taken from the `zcash/orchard` repository at commit `4ac248d0` (v0.11.0)
            // This ensures backward compatibility.
            &[
                35, 70, 47, 40, 125, 172, 163, 93, 38, 0, 111, 90, 137, 81, 41, 181, 216, 25, 190,
                222, 210, 45, 44, 127, 182, 44, 228, 229, 132, 64, 53, 7,
            ],
        );
    }

    fn shielded_bundle<FL: OrchardFlavor>(bundle_type: BundleType) -> Blake2bHash {
        let pk = ProvingKey::build::<FL>();
        let mut rng = StdRng::seed_from_u64(1u64);

        // Pretend we derived the spending key via ZIP 32.
        let zip32_derivation = Zip32Derivation::parse([1; 32], vec![]).unwrap();
        let sk = SpendingKey::random(&mut rng);
        let ask = SpendAuthorizingKey::from(&sk);
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        // Pretend we already received a note.
        let value = NoteValue::from_raw(15_000);
        let note = {
            let rho = Rho::from_bytes(&pallas::Base::random(&mut rng).to_repr()).unwrap();
            loop {
                if let Some(note) = Note::from_parts(
                    recipient,
                    value,
                    AssetBase::native(),
                    rho,
                    RandomSeed::random(&mut rng, &rho),
                )
                .into_option()
                {
                    break note;
                }
            }
        };

        // Use the tree with a single leaf.
        let (anchor, merkle_path) = {
            let cmx: ExtractedNoteCommitment = note.commitment().into();
            let leaf = MerkleHashOrchard::from_cmx(&cmx);
            let mut tree: ShardTree<MemoryShardStore<MerkleHashOrchard, u32>, 32, 16> =
                ShardTree::new(MemoryShardStore::empty(), 100);
            tree.append(
                leaf,
                Retention::Checkpoint {
                    id: 0,
                    marking: Marking::Marked,
                },
            )
            .unwrap();
            let root = tree.root_at_checkpoint_id(&0).unwrap().unwrap();
            let position = tree.max_leaf_position(None).unwrap().unwrap();
            let merkle_path = tree
                .witness_at_checkpoint_id(position, &0)
                .unwrap()
                .unwrap();
            assert_eq!(root, merkle_path.root(MerkleHashOrchard::from_cmx(&cmx)));
            (root.into(), merkle_path)
        };

        // Run the Creator and Constructor roles.
        let mut builder = Builder::new(bundle_type, anchor);
        builder
            .add_spend(fvk.clone(), note, merkle_path.into())
            .unwrap();
        builder
            .add_output(
                None,
                recipient,
                NoteValue::from_raw(10_000),
                AssetBase::native(),
                [0u8; 512],
            )
            .unwrap();
        builder
            .add_output(
                Some(fvk.to_ovk(Scope::Internal)),
                fvk.address_at(0u32, Scope::Internal),
                NoteValue::from_raw(5_000),
                AssetBase::native(),
                [0u8; 512],
            )
            .unwrap();
        let balance: i64 = builder.value_balance().unwrap();
        assert_eq!(balance, 0);
        let mut pczt_bundle = builder.build_for_pczt::<FL>(&mut rng).unwrap().0;

        // Run the IO Finalizer role.
        let sighash = [0; 32];
        pczt_bundle.finalize_io(sighash, rng.clone()).unwrap();

        // Run the Updater role.
        for action in pczt_bundle.actions_mut() {
            if action.spend.value() == &Some(value) {
                action.spend.zip32_derivation = Some(Zip32Derivation {
                    seed_fingerprint: zip32_derivation.seed_fingerprint,
                    derivation_path: zip32_derivation.derivation_path.clone(),
                });
            }
        }

        // Run the Prover role.
        pczt_bundle.create_proof::<FL, _>(&pk, rng.clone()).unwrap();

        // TODO: Verify that the PCZT contains sufficient information to decrypt and check
        // `enc_ciphertext`.

        // Run the Signer role.
        for action in pczt_bundle.actions_mut() {
            if action.spend.zip32_derivation.as_ref() == Some(&zip32_derivation) {
                action.sign(sighash, &ask, rng.clone()).unwrap();
            }
        }

        // Run the Transaction Extractor role.
        let bundle = pczt_bundle.extract::<i64, FL>().unwrap().unwrap();

        let orchard_digest = hash_bundle_txid_data(&bundle);

        assert_eq!(bundle.value_balance(), &0);
        // We can successfully bind the bundle.
        bundle.apply_binding_signature(sighash, rng).unwrap();

        orchard_digest
    }

    #[test]
    fn shielded_bundle_orchard_zsa() {
        let orchard_digest = shielded_bundle::<OrchardZSA>(BundleType::DEFAULT_ZSA);
        assert_eq!(
            orchard_digest.as_bytes(),
            // Locks the `orchard_digest` for OrchardZSA
            &[
                202, 244, 46, 236, 35, 213, 200, 114, 164, 148, 125, 157, 39, 224, 254, 144, 240,
                88, 23, 228, 37, 184, 222, 224, 48, 207, 191, 180, 197, 241, 61, 185
            ],
        );
        let orchard_digest = shielded_bundle::<OrchardZSA>(BundleType::DEFAULT_VANILLA);
        assert_eq!(
            orchard_digest.as_bytes(),
            // Locks the `orchard_digest` for OrchardZSA
            &[
                34, 179, 215, 77, 244, 97, 43, 180, 112, 169, 87, 174, 23, 157, 98, 43, 72, 248,
                198, 6, 24, 193, 245, 116, 123, 100, 230, 178, 196, 32, 5, 103
            ],
        );
    }

    #[test]
    fn shielded_bundle_orchard_vanilla() {
        let orchard_digest = shielded_bundle::<OrchardVanilla>(BundleType::DEFAULT_VANILLA);
        assert_eq!(
            orchard_digest.as_bytes(),
            // `orchard_digest` taken from the `zcash/orchard` repository at commit `4ac248d0` (v0.11.0)
            // This ensures backward compatibility.
            &[
                231, 253, 96, 110, 94, 252, 150, 231, 45, 160, 60, 178, 10, 219, 95, 29, 113, 154,
                223, 96, 50, 247, 213, 119, 211, 232, 186, 59, 24, 93, 177, 225
            ],
        );
    }
}
