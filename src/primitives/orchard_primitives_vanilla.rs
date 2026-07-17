//! This module implements the note encryption and commitment logic specific for the
//! `OrchardVanilla` flavor.

use alloc::vec::Vec;
use blake2b_simd::Hash as Blake2bHash;
use zcash_note_encryption::note_bytes::NoteBytesData;

use crate::{
    bundle::{
        commitments::{hasher, BundleCommitmentFormat},
        Authorization, Authorized, BundleVersion, CommitmentError, TxVersion,
    },
    flavor::OrchardVanilla,
    note::{AssetBase, Note},
    note_encryption::{
        build_base_note_plaintext_bytes, Memo, COMPACT_NOTE_SIZE_VANILLA, MEMO_SIZE,
    },
    primitives::orchard_primitives::OrchardPrimitives,
    sighash_kind::OrchardSighashKind,
    Bundle,
};

impl OrchardPrimitives for OrchardVanilla {
    const COMPACT_NOTE_SIZE: usize = COMPACT_NOTE_SIZE_VANILLA;
    const BASE_PROOF_SIZE: usize = 2720;
    const PER_ACTION_PROOF_SIZE: usize = 2272;

    type NotePlaintextBytes = NoteBytesData<{ Self::NOTE_PLAINTEXT_SIZE }>;
    type NoteCiphertextBytes = NoteBytesData<{ Self::ENC_CIPHERTEXT_SIZE }>;
    type CompactNotePlaintextBytes = NoteBytesData<{ Self::COMPACT_NOTE_SIZE }>;
    type CompactNoteCiphertextBytes = NoteBytesData<{ Self::COMPACT_NOTE_SIZE }>;

    fn build_note_plaintext_bytes(note: &Note, memo: &Memo) -> Self::NotePlaintextBytes {
        let mut np = build_base_note_plaintext_bytes(note);

        np[COMPACT_NOTE_SIZE_VANILLA..].copy_from_slice(memo);

        NoteBytesData(np)
    }

    fn extract_asset(_plaintext: &Self::CompactNotePlaintextBytes) -> Option<AssetBase> {
        Some(AssetBase::zatoshi())
    }
    /// Write disjoint parts of each bundle action as 3 separate hashes
    /// as defined in [ZIP-244: Transaction Identifier Non-Malleability][zip244]:
    /// * \[(nullifier, cmx, ephemeral_key, enc_ciphertext\[..52\])*\] personalized
    ///   with the format's compact-action personalization string
    /// * \[enc_ciphertext\[52..564\]*\] (memo ciphertexts) personalized
    ///   with the format's action-memos personalization string
    /// * \[(cv, rk, enc_ciphertext\[564..\], out_ciphertext)*\] personalized
    ///   with the format's non-compact-action personalization string
    ///
    /// Then, hash these together along with (flags, value_balance_orchard, and — for the v5
    /// transaction format only — anchor_orchard), personalized with the format's bundle
    /// personalization string. In the v6 format the anchor is included by
    /// `hash_bundle_auth_data` instead.
    ///
    /// Returns [`CommitmentError::InvalidTransactionVersion`] if `tx_version` is not valid for the
    /// bundle's [`BundleVersion`].
    ///
    /// [zip244]: https://zips.z.cash/zip-0244
    /// [`BundleVersion`]: crate::bundle::BundleVersion
    fn hash_bundle_txid_data<A: Authorization, V: Copy + Into<i64>>(
        bundle: &Bundle<A, V, OrchardVanilla>,
        tx_version: TxVersion,
    ) -> Result<Blake2bHash, CommitmentError> {
        let format = bundle
            .bundle_version()
            .value_pool()
            .commitment_format(tx_version)?;

        if format == BundleCommitmentFormat::ZSA {
            return Err(CommitmentError::InvalidTransactionVersion);
        }

        let personalizations = format.personalizations();
        let mut h = hasher(personalizations.bundle);
        let mut ch = hasher(personalizations.actions_compact);
        let mut mh = hasher(personalizations.actions_memos);
        let mut nh = hasher(personalizations.actions_noncompact);

        for action in bundle.actions().iter() {
            ch.update(&action.nullifier().to_bytes());
            ch.update(&action.cmx().to_bytes());
            ch.update(&action.encrypted_note().epk_bytes);
            ch.update(&action.encrypted_note().enc_ciphertext.as_ref()[..Self::COMPACT_NOTE_SIZE]);

            mh.update(
                &action.encrypted_note().enc_ciphertext.as_ref()
                    [Self::COMPACT_NOTE_SIZE..Self::COMPACT_NOTE_SIZE + MEMO_SIZE],
            );

            nh.update(&action.cv_net().to_bytes());
            nh.update(&<[u8; 32]>::from(action.rk()));
            nh.update(
                &action.encrypted_note().enc_ciphertext.as_ref()
                    [Self::COMPACT_NOTE_SIZE + MEMO_SIZE..],
            );
            nh.update(&action.encrypted_note().out_ciphertext);
        }

        h.update(ch.finalize().as_bytes());
        h.update(mh.finalize().as_bytes());
        h.update(nh.finalize().as_bytes());
        h.update(&[bundle.flag_byte()]);
        h.update(&(*bundle.value_balance()).into().to_le_bytes());
        if format.includes_anchor_in_txid_digest() {
            h.update(&bundle.anchor().to_bytes());
        }
        Ok(h.finalize())
    }

    /// Construct the commitment to the authorizing data of an
    /// authorized bundle as defined in [ZIP-244: Transaction
    /// Identifier Non-Malleability][zip244]
    ///
    /// [zip244]: https://zips.z.cash/zip-0244
    fn hash_bundle_auth_data<V>(
        bundle: &Bundle<Authorized, V, OrchardVanilla>,
        tx_version: TxVersion,
        _sighash_info_for_kind: impl Fn(&OrchardSighashKind) -> Vec<u8>,
    ) -> Result<Blake2bHash, CommitmentError> {
        let format = bundle
            .bundle_version()
            .value_pool()
            .commitment_format(tx_version)?;

        if format == BundleCommitmentFormat::ZSA {
            return Err(CommitmentError::InvalidTransactionVersion);
        }
        let mut h = hasher(format.personalizations().auth);
        h.update(bundle.authorization().proof().as_ref());
        for action in bundle.actions().iter() {
            assert_eq!(
                *action.authorization().sighash_kind(),
                OrchardSighashKind::AllEffecting
            );
            h.update(&<[u8; 64]>::from(action.authorization().sig()));
        }
        assert_eq!(
            *bundle.authorization().binding_signature().sighash_kind(),
            OrchardSighashKind::AllEffecting
        );
        h.update(&<[u8; 64]>::from(
            bundle.authorization().binding_signature().sig(),
        ));
        if format.includes_anchor_in_authorizing_digest() {
            h.update(&bundle.anchor().to_bytes());
        }
        Ok(h.finalize())
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
        flavor::OrchardVanilla,
        keys::{
            DiversifiedTransmissionKey, Diversifier, EphemeralSecretKey, IncomingViewingKey,
            OutgoingViewingKey, PreparedIncomingViewingKey,
        },
        note::{
            testing::arb_zatoshi_note, AssetBase, ExtractedNoteCommitment, Note, Nullifier,
            RandomSeed, Rho, TransmittedNoteCiphertext,
        },
        note_encryption::{
            parse_note_plaintext_without_memo, prf_ock_orchard, CompactAction, OrchardDomain,
        },
        primitives::redpallas,
        value::{NoteValue, ValueCommitment},
        NoteVersion,
    };

    proptest! {
        #[test]
        fn encoding_roundtrip(
            note in arb_zatoshi_note(NoteVersion::V3),
        ) {
            let memo = &crate::test_vectors::note_encryption_vanilla::TEST_VECTORS[0].memo;
            let rho = note.rho();

            // Encode.
            let plaintext = OrchardDomain::note_plaintext_bytes(&note, memo);

            // Decode.
            let domain = OrchardDomain::from_rho(rho);
            let (compact, parsed_memo) = domain.split_plaintext_at_memo(&plaintext).unwrap();

            let (parsed_note, parsed_recipient) = parse_note_plaintext_without_memo::<OrchardVanilla, _>(rho, &compact, NoteVersion::V3,
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
        let test_vectors = crate::test_vectors::note_encryption_vanilla::TEST_VECTORS;

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

            let asset = AssetBase::zatoshi();

            let note =
                Note::from_parts(recipient, value, asset, rho, rseed, NoteVersion::V2).unwrap();
            assert_eq!(ExtractedNoteCommitment::from(note.commitment()), cmx);

            let action = Action::from_parts(
                // nf_old is the nullifier revealed by the receiving Action.
                nf_old,
                // We don't need a real rk for this test.
                redpallas::VerificationKey::dummy(),
                cmx,
                TransmittedNoteCiphertext::<OrchardVanilla> {
                    epk_bytes: ephemeral_key.0,
                    enc_ciphertext: NoteBytesData(tv.c_enc),
                    out_ciphertext: tv.c_out,
                },
                cv_net.clone(),
                (),
            ).expect("a key returned by VerificationKey::dummy() is vanishingly unlikely to be the identity");

            //
            // Test decryption
            // (Tested first because it only requires immutable references.)
            //

            let domain = OrchardDomain::from_rho(rho);

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

            let ne = zcash_note_encryption::NoteEncryption::<OrchardDomain>::new_with_esk(
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
