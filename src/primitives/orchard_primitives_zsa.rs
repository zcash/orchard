//! This module implements the note encryption and commitment logic specific for the `OrchardZSA`
//! flavor.

use alloc::{collections::BTreeMap, vec::Vec};
use blake2b_simd::Hash as Blake2bHash;
use zcash_note_encryption::note_bytes::NoteBytesData;

use crate::{
    bundle::{
        commitments::{
            get_compact_size, hasher, ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION_V6,
            ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION,
            ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION_V6,
            ZCASH_ORCHARD_ACTION_GROUPS_HASH_PERSONALIZATION,
            ZCASH_ORCHARD_ACTION_GROUPS_SIGS_HASH_PERSONALIZATION,
            ZCASH_ORCHARD_HASH_PERSONALIZATION, ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION,
            ZCASH_ORCHARD_SPEND_AUTH_SIGS_HASH_PERSONALIZATION,
            ZCASH_ORCHARD_ZSA_BURN_HASH_PERSONALIZATION,
        },
        Authorization, Authorized,
    },
    flavor::OrchardZSA,
    note::{AssetBase, Note},
    primitives::{
        orchard_primitives::OrchardPrimitives,
        zcash_note_encryption_domain::{
            build_base_note_plaintext_bytes, Memo, COMPACT_NOTE_SIZE_VANILLA,
            COMPACT_NOTE_SIZE_ZSA, MEMO_SIZE, NOTE_VERSION_BYTE_V3,
        },
    },
    sighash_versioning::OrchardSighashVersion,
    Bundle,
};

impl OrchardPrimitives for OrchardZSA {
    const COMPACT_NOTE_SIZE: usize = COMPACT_NOTE_SIZE_ZSA;

    type NotePlaintextBytes = NoteBytesData<{ Self::NOTE_PLAINTEXT_SIZE }>;
    type NoteCiphertextBytes = NoteBytesData<{ Self::ENC_CIPHERTEXT_SIZE }>;
    type CompactNotePlaintextBytes = NoteBytesData<{ Self::COMPACT_NOTE_SIZE }>;
    type CompactNoteCiphertextBytes = NoteBytesData<{ Self::COMPACT_NOTE_SIZE }>;

    fn build_note_plaintext_bytes(note: &Note, memo: &Memo) -> Self::NotePlaintextBytes {
        let mut np = build_base_note_plaintext_bytes(NOTE_VERSION_BYTE_V3, note);

        np[COMPACT_NOTE_SIZE_VANILLA..COMPACT_NOTE_SIZE_ZSA]
            .copy_from_slice(&note.asset().to_bytes());
        np[COMPACT_NOTE_SIZE_ZSA..].copy_from_slice(memo);

        NoteBytesData(np)
    }

    fn extract_asset(plaintext: &Self::CompactNotePlaintextBytes) -> Option<AssetBase> {
        let bytes = plaintext.as_ref()[COMPACT_NOTE_SIZE_VANILLA..COMPACT_NOTE_SIZE_ZSA]
            .try_into()
            .unwrap();

        AssetBase::from_bytes(bytes).into()
    }

    /// Evaluate `orchard_digest` for the bundle as defined in
    /// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
    ///
    /// [zip246]: https://zips.z.cash/zip-0246
    fn hash_bundle_txid_data<A: Authorization, V: Copy + Into<i64>>(
        bundle: &Bundle<A, V, OrchardZSA>,
    ) -> Blake2bHash {
        let mut h = hasher(ZCASH_ORCHARD_HASH_PERSONALIZATION);
        let mut agh = hasher(ZCASH_ORCHARD_ACTION_GROUPS_HASH_PERSONALIZATION);

        let mut ch = hasher(ZCASH_ORCHARD_ACTIONS_COMPACT_HASH_PERSONALIZATION_V6);
        // TODO Remove mh once new Memo Bundles are implemented (ZIP-231).
        let mut mh = hasher(ZCASH_ORCHARD_ACTIONS_MEMOS_HASH_PERSONALIZATION);
        let mut nh = hasher(ZCASH_ORCHARD_ACTIONS_NONCOMPACT_HASH_PERSONALIZATION_V6);

        for action in bundle.actions().iter() {
            ch.update(&action.nullifier().to_bytes());
            ch.update(&action.cmx().to_bytes());
            ch.update(&action.encrypted_note().epk_bytes);
            // TODO Remove once new Memo Bundles are implemented (ZIP-231).
            ch.update(&action.encrypted_note().enc_ciphertext.as_ref()[..Self::COMPACT_NOTE_SIZE]);
            // TODO Uncomment once new Memo Bundles are implemented (ZIP-231).
            // ch.update(&action.encrypted_note().enc_ciphertext.as_ref());

            // TODO Remove once new Memo Bundles are implemented (ZIP-231).
            mh.update(
                &action.encrypted_note().enc_ciphertext.as_ref()
                    [Self::COMPACT_NOTE_SIZE..Self::COMPACT_NOTE_SIZE + MEMO_SIZE],
            );

            nh.update(&action.cv_net().to_bytes());
            nh.update(&<[u8; 32]>::from(action.rk()));
            // TODO Remove once new Memo Bundles are implemented (ZIP-231).
            nh.update(
                &action.encrypted_note().enc_ciphertext.as_ref()
                    [Self::COMPACT_NOTE_SIZE + MEMO_SIZE..],
            );
            nh.update(&action.encrypted_note().out_ciphertext);
        }

        agh.update(ch.finalize().as_bytes());
        // TODO Remove once new Memo Bundles are implemented (ZIP-231).
        agh.update(mh.finalize().as_bytes());
        agh.update(nh.finalize().as_bytes());

        agh.update(&[bundle.flags().to_byte()]);
        agh.update(&bundle.anchor().to_bytes());
        // For the OrchardZSA protocol, `expiry_height` is set to 0, indicating no expiry.
        agh.update(&0u32.to_le_bytes());

        let mut burn_hasher = hasher(ZCASH_ORCHARD_ZSA_BURN_HASH_PERSONALIZATION);
        for burn_item in bundle.burn() {
            burn_hasher.update(&burn_item.0.to_bytes());
            burn_hasher.update(&burn_item.1.to_bytes());
        }
        agh.update(burn_hasher.finalize().as_bytes());
        h.update(agh.finalize().as_bytes());

        h.update(&(*bundle.value_balance()).into().to_le_bytes());
        h.finalize()
    }

    /// Evaluate `orchard_auth_digest` for the bundle as defined in
    /// [ZIP-246: Digests for the Version 6 Transaction Format][zip246]
    ///
    /// The `sighash_version_map` provides the mapping from each
    /// `OrchardSighashVersion` to the corresponding `SighashInfo`
    /// encoding.
    ///
    /// [zip246]: https://zips.z.cash/zip-0246
    fn hash_bundle_auth_data<V>(
        bundle: &Bundle<Authorized, V, OrchardZSA>,
        sighash_version_map: &BTreeMap<OrchardSighashVersion, Vec<u8>>,
    ) -> Blake2bHash {
        let mut h = hasher(ZCASH_ORCHARD_SIGS_HASH_PERSONALIZATION);
        let mut agh = hasher(ZCASH_ORCHARD_ACTION_GROUPS_SIGS_HASH_PERSONALIZATION);
        agh.update(bundle.authorization().proof().as_ref());
        let mut sash = hasher(ZCASH_ORCHARD_SPEND_AUTH_SIGS_HASH_PERSONALIZATION);
        for action in bundle.actions().iter() {
            let version_bytes = sighash_version_map
                .get(action.authorization().version())
                .expect("Unknown Orchard sighash version.");
            sash.update(&get_compact_size(version_bytes.len()));
            sash.update(version_bytes);
            sash.update(&<[u8; 64]>::from(action.authorization().sig()));
        }
        agh.update(sash.finalize().as_bytes());
        h.update(agh.finalize().as_bytes());

        let version_bytes = sighash_version_map
            .get(bundle.authorization().binding_signature().version())
            .unwrap();
        h.update(&get_compact_size(version_bytes.len()));
        h.update(version_bytes);
        h.update(&<[u8; 64]>::from(
            bundle.authorization().binding_signature().sig(),
        ));
        h.finalize()
    }

    fn default_sighash_version() -> OrchardSighashVersion {
        OrchardSighashVersion::V0
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use rand::rngs::OsRng;

    use zcash_note_encryption::{
        note_bytes::NoteBytesData, try_compact_note_decryption, try_note_decryption,
        try_output_recovery_with_ovk, Domain, EphemeralKeyBytes,
    };

    use crate::{
        action::Action,
        address::Address,
        flavor::OrchardZSA,
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, IncomingViewingKey,
            OutgoingViewingKey, PreparedIncomingViewingKey,
        },
        note::{
            testing::arb_note, AssetBase, ExtractedNoteCommitment, Note, Nullifier, RandomSeed,
            Rho, TransmittedNoteCiphertext,
        },
        primitives::{
            compact_action::CompactAction,
            orchard_domain::OrchardDomain,
            redpallas,
            zcash_note_encryption_domain::{
                parse_note_plaintext_without_memo, parse_note_version, prf_ock_orchard,
            },
        },
        value::{NoteValue, ValueCommitment},
    };

    type OrchardDomainZSA = OrchardDomain<OrchardZSA>;

    proptest! {
        #[test]
        fn encoding_roundtrip(
            note in arb_note(NoteValue::from_raw(100)),
        ) {
            let memo = &crate::test_vectors::note_encryption_zsa::TEST_VECTORS[0].memo;
            let rho = note.rho();

            // Encode.
            let plaintext = OrchardDomainZSA::note_plaintext_bytes(&note, memo);

            // Decode.
            let domain = OrchardDomainZSA::for_rho(rho);
            let (compact, parsed_memo) = domain.split_plaintext_at_memo(&plaintext).unwrap();

            assert!(parse_note_version(compact.as_ref()).is_some());

            let (parsed_note, parsed_recipient) = parse_note_plaintext_without_memo::<OrchardZSA, _>(rho, &compact,
                |diversifier| {
                    assert_eq!(diversifier, &note.recipient().diversifier());
                    Some(*note.recipient().pk_d())
                }
            ).expect("Plaintext parsing failed");

            // Check.
            assert_eq!(parsed_note, note);
            assert_eq!(parsed_recipient, note.recipient());
            assert_eq!(&parsed_memo, memo);
        }
    }

    #[test]
    fn test_vectors() {
        let test_vectors = crate::test_vectors::note_encryption_zsa::TEST_VECTORS;

        for tv in test_vectors {
            //
            // Load the test vector components
            //

            // Recipient key material
            let ivk = PreparedIncomingViewingKey::new(
                &IncomingViewingKey::from_bytes(&tv.incoming_viewing_key).unwrap(),
            );
            let ovk = OutgoingViewingKey::from(tv.ovk);
            let d = Diversifier::from_bytes(tv.default_d);
            let pk_d = DiversifiedTransmissionKey::from_bytes(&tv.default_pk_d).unwrap();

            // Received Action
            let cv_net = ValueCommitment::from_bytes(&tv.cv_net).unwrap();
            let nf_old = Nullifier::from_bytes(&tv.nf_old).unwrap();
            let rho = Rho::from_nf_old(nf_old);
            let cmx = ExtractedNoteCommitment::from_bytes(&tv.cmx).unwrap();

            let esk = EphemeralSecretKey::from_bytes(&tv.esk).unwrap();
            let ephemeral_key = EphemeralKeyBytes(tv.ephemeral_key);

            // Details about the expected note
            let value = NoteValue::from_raw(tv.v);
            let rseed = RandomSeed::from_bytes(tv.rseed, &rho).unwrap();

            //
            // Test the individual components
            //

            let shared_secret = esk.agree(&pk_d);
            assert_eq!(shared_secret.to_bytes(), tv.shared_secret);

            let k_enc = shared_secret.kdf_orchard(&ephemeral_key);
            assert_eq!(k_enc.as_bytes(), tv.k_enc);

            let ock = prf_ock_orchard(&ovk, &cv_net, &cmx.to_bytes(), &ephemeral_key);
            assert_eq!(ock.as_ref(), tv.ock);

            let recipient = Address::from_parts(d, pk_d);

            let asset = AssetBase::from_bytes(&tv.asset).unwrap();

            let note = Note::from_parts(recipient, value, asset, rho, rseed).unwrap();
            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);

            let action = Action::from_parts(
                // nf_old is the nullifier revealed by the receiving Action.
                nf_old,
                // We don't need a valid rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext::<OrchardZSA> {
                    epk_bytes: ephemeral_key.0,
                    enc_ciphertext: NoteBytesData(tv.c_enc),
                    out_ciphertext: tv.c_out,
                },
                cv_net.clone(),
                (),
            );

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            let domain = OrchardDomain::for_rho(rho);

            match try_note_decryption(&domain, &ivk, &action) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Note decryption failed"),
            }

            match try_compact_note_decryption(&domain, &ivk, &CompactAction::from(&action)) {
                Some((decrypted_note, decrypted_to)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                }
                None => panic!("Compact note decryption failed"),
            }

            match try_output_recovery_with_ovk(&domain, &ovk, &action, &cv_net, &tv.c_out) {
                Some((decrypted_note, decrypted_to, decrypted_memo)) => {
                    assert_eq!(decrypted_note, note);
                    assert_eq!(decrypted_to, recipient);
                    assert_eq!(&decrypted_memo[..], &tv.memo[..]);
                }
                None => panic!("Output recovery failed"),
            }

            //
            // Test encryption
            //

            let ne = zcash_note_encryption::NoteEncryption::<OrchardDomainZSA>::new_with_esk(
                esk,
                Some(ovk),
                note,
                tv.memo,
            );

            assert_eq!(ne.encrypt_note_plaintext().as_ref(), &tv.c_enc[..]);
            assert_eq!(
                &ne.encrypt_outgoing_plaintext(&cv_net, &cmx, &mut OsRng)[..],
                &tv.c_out[..]
            );
        }
    }
}
