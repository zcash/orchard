//! In-band secret distribution for PCZT actions.
//!
//! Which note encryption domain an action's output was encrypted under is determined by
//! the note plaintext version the action itself records: a [`NoteVersion::V2`] output uses
//! [`OrchardDomain`], and a [`NoteVersion::V3`] output uses [`IronwoodDomain`]. Pairing an
//! output with the wrong domain does not fail loudly: trial decryption simply does not
//! match, and the output is then misclassified (for a Signer, the difference between an
//! output it displays for approval and one it treats as change). The helpers here keep
//! that pairing inside this crate, so a PCZT consumer never reproduces it.

use zcash_note_encryption::{
    try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk,
    EphemeralKeyBytes, COMPACT_NOTE_SIZE,
};

use crate::{
    keys::{OutgoingViewingKey, PreparedIncomingViewingKey},
    note_encryption::{CompactAction, IronwoodDomain, OrchardDomain},
    Address, Note, NoteVersion,
};

impl From<&super::Action> for CompactAction {
    fn from(action: &super::Action) -> Self {
        // The compact note ciphertext is the prefix of the full note ciphertext. Both
        // lengths are compile-time constants, and `COMPACT_NOTE_SIZE` is the smaller.
        let mut enc_ciphertext = [0; COMPACT_NOTE_SIZE];
        enc_ciphertext
            .copy_from_slice(&action.output.encrypted_note.enc_ciphertext[..COMPACT_NOTE_SIZE]);

        CompactAction::from_parts(
            action.spend.nullifier,
            action.output.cmx,
            EphemeralKeyBytes(action.output.encrypted_note.epk_bytes),
            enc_ciphertext,
        )
    }
}

impl super::Action {
    /// Trial-decrypts this action's output note with the given incoming viewing key, in
    /// the note encryption domain selected by the output's own [`NoteVersion`].
    ///
    /// Returns the note, the address it was sent to, and its memo, or `None` if this
    /// action's output was not encrypted to `ivk`.
    ///
    /// The incoming viewing key is prepared by the caller ([`PreparedIncomingViewingKey`]),
    /// so a single preparation can be reused across every action in a bundle.
    ///
    /// A successful decryption binds the returned note to this action's `cmx`, but says
    /// nothing about the action's separately-transmitted (and redactable) `recipient` and
    /// `value` fields; use [`Output::verify_note_commitment`] to bind those to the same
    /// `cmx`.
    ///
    /// [`Output::verify_note_commitment`]: super::Output::verify_note_commitment
    pub fn decrypt_output_with_ivk(
        &self,
        ivk: &PreparedIncomingViewingKey,
    ) -> Option<(Note, Address, [u8; 512])> {
        match self.output.note_version {
            NoteVersion::V2 => {
                try_note_decryption(&OrchardDomain::for_pczt_action(self), ivk, self)
            }
            NoteVersion::V3 => {
                try_note_decryption(&IronwoodDomain::for_pczt_action(self), ivk, self)
            }
        }
    }

    /// Trial-decrypts the compact part of this action's output note with the given
    /// incoming viewing key, in the note encryption domain selected by the output's own
    /// [`NoteVersion`].
    ///
    /// Returns the note and the address it was sent to, or `None` if this action's output
    /// was not encrypted to `ivk`. This is the trial decryption a consumer that only ever
    /// sees the compact note ciphertext (a light client, or a hardware wallet that is
    /// streamed one) performs, reproduced here over the full PCZT action.
    ///
    /// Unlike [`Action::decrypt_output_with_ivk`], the note ciphertext is not
    /// authenticated by its AEAD tag; the returned note is instead validated by
    /// re-deriving this action's `cmx`, and the memo is not recovered.
    ///
    /// [`Action::decrypt_output_with_ivk`]: Self::decrypt_output_with_ivk
    pub fn decrypt_compact_output_with_ivk(
        &self,
        ivk: &PreparedIncomingViewingKey,
    ) -> Option<(Note, Address)> {
        let compact = CompactAction::from(self);
        match self.output.note_version {
            NoteVersion::V2 => {
                try_compact_note_decryption(&OrchardDomain::for_pczt_action(self), ivk, &compact)
            }
            NoteVersion::V3 => {
                try_compact_note_decryption(&IronwoodDomain::for_pczt_action(self), ivk, &compact)
            }
        }
    }

    /// Recovers this action's output note with the given outgoing viewing key, in the note
    /// encryption domain selected by the output's own [`NoteVersion`].
    ///
    /// Returns the note, the address it was sent to, and its memo, or `None` if this
    /// action's output was not encrypted with `ovk`. This is the sender-side recovery of
    /// [Zcash Protocol Specification § 4.20.3][decryptovk]; it is the way the party that
    /// created an output reads it back out of a PCZT, and it fails for an output that a
    /// Constructor added under an OVK policy of "None".
    ///
    /// A successful recovery binds the returned note to this action's `cmx`, but says
    /// nothing about the action's separately-transmitted (and redactable) `recipient` and
    /// `value` fields; use [`Output::verify_note_commitment`] to bind those to the same
    /// `cmx`.
    ///
    /// [decryptovk]: https://zips.z.cash/protocol/protocol.pdf#decryptovk
    /// [`Output::verify_note_commitment`]: super::Output::verify_note_commitment
    pub fn recover_output_with_ovk(
        &self,
        ovk: &OutgoingViewingKey,
    ) -> Option<(Note, Address, [u8; 512])> {
        let out_ciphertext = &self.output.encrypted_note.out_ciphertext;
        match self.output.note_version {
            NoteVersion::V2 => try_output_recovery_with_ovk(
                &OrchardDomain::for_pczt_action(self),
                ovk,
                self,
                &self.cv_net,
                out_ciphertext,
            ),
            NoteVersion::V3 => try_output_recovery_with_ovk(
                &IronwoodDomain::for_pczt_action(self),
                ovk,
                self,
                &self.cv_net,
                out_ciphertext,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;
    use zcash_note_encryption::{ShieldedOutput, COMPACT_NOTE_SIZE};

    use crate::{
        builder::{Builder, BundleType},
        bundle::BundleVersion,
        constants::MERKLE_DEPTH_ORCHARD,
        keys::{
            FullViewingKey, OutgoingViewingKey, PreparedIncomingViewingKey, Scope, SpendingKey,
        },
        note::Rho,
        note_encryption::{CompactAction, OrchardDomain},
        tree::EMPTY_ROOTS,
        value::NoteValue,
        Address, NoteVersion,
    };

    /// The value of every fixture output, in zatoshis. Nonzero, so that an output that
    /// failed to decrypt could not be mistaken for a dummy.
    const OUTPUT_VALUE_ZATS: u64 = 5_000;

    /// The memo of every fixture output. Not all-zero, so that a memo recovered from the
    /// wrong part of a note plaintext would not compare equal to it.
    const MEMO: [u8; 512] = [0xab; 512];

    /// Builds a PCZT bundle containing a single output paying `recipient` under
    /// `bundle_version`, and returns it alongside the index of the action carrying that
    /// output.
    fn single_output_bundle(
        bundle_version: BundleVersion,
        ovk: Option<OutgoingViewingKey>,
        recipient: Address,
        mut rng: OsRng,
    ) -> (crate::pczt::Bundle, usize) {
        let mut builder = Builder::new(
            BundleType::DEFAULT,
            bundle_version,
            bundle_version.default_flags(),
            EMPTY_ROOTS[MERKLE_DEPTH_ORCHARD].into(),
        )
        .unwrap();
        builder
            .add_output(ovk, recipient, NoteValue::from_raw(OUTPUT_VALUE_ZATS), MEMO)
            .unwrap();
        let (bundle, bundle_meta) = builder.build_for_pczt(&mut rng).unwrap();
        let action_index = bundle_meta.output_action_index(0).unwrap();
        (bundle, action_index)
    }

    /// The `CompactAction` view of a PCZT action is assembled entirely from that action's
    /// own fields.
    #[test]
    fn compact_action_view_matches_the_action() {
        let mut rng = OsRng;
        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let (bundle, action_index) = single_output_bundle(
            BundleVersion::orchard_v2(),
            None,
            fvk.address_at(0u32, Scope::External),
            rng,
        );

        let action = &bundle.actions()[action_index];
        let compact = CompactAction::from(action);

        assert_eq!(compact.nullifier(), *action.spend().nullifier());
        assert_eq!(compact.cmx(), *action.output().cmx());
        assert_eq!(compact.rho(), Rho::from_nf_old(*action.spend().nullifier()));
        assert_eq!(
            ShieldedOutput::<OrchardDomain, COMPACT_NOTE_SIZE>::ephemeral_key(&compact).0,
            action.output().encrypted_note().epk_bytes,
        );
        assert_eq!(
            ShieldedOutput::<OrchardDomain, COMPACT_NOTE_SIZE>::enc_ciphertext(&compact)[..],
            action.output().encrypted_note().enc_ciphertext[..COMPACT_NOTE_SIZE],
        );
    }

    /// Every trial decryption on a PCZT action selects its domain from the action's own
    /// note version: with the pairing intact each of them opens the output, and with the
    /// note version changed underneath them none of them does.
    #[test]
    fn decryption_domain_follows_the_output_note_version() {
        let mut rng = OsRng;

        for (bundle_version, note_version, other_note_version) in [
            (
                BundleVersion::orchard_v2(),
                NoteVersion::V2,
                NoteVersion::V3,
            ),
            (
                BundleVersion::ironwood_v3(),
                NoteVersion::V3,
                NoteVersion::V2,
            ),
        ] {
            let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
            let recipient = fvk.address_at(0u32, Scope::External);
            let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
            let ovk = fvk.to_ovk(Scope::External);

            let (mut bundle, action_index) =
                single_output_bundle(bundle_version, Some(ovk.clone()), recipient, rng);

            let action = &bundle.actions()[action_index];
            assert_eq!(action.output().note_version(), &note_version);

            let (note, address, memo) = action
                .decrypt_output_with_ivk(&ivk)
                .expect("the recipient's IVK opens the output");
            assert_eq!(address, recipient);
            assert_eq!(note.value(), NoteValue::from_raw(OUTPUT_VALUE_ZATS));
            assert_eq!(memo, MEMO);

            // The compact trial decryption recovers the same note, without the memo.
            assert_eq!(
                action.decrypt_compact_output_with_ivk(&ivk),
                Some((note, address)),
            );

            // The sender recovers the same note and memo under the OVK it used.
            assert_eq!(
                action.recover_output_with_ovk(&ovk),
                Some((note, address, memo)),
            );

            // Another account's keys open nothing.
            let other_fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
            let other_ivk = PreparedIncomingViewingKey::new(&other_fvk.to_ivk(Scope::External));
            assert!(action.decrypt_output_with_ivk(&other_ivk).is_none());
            assert!(action.decrypt_compact_output_with_ivk(&other_ivk).is_none());
            assert!(action
                .recover_output_with_ovk(&other_fvk.to_ovk(Scope::External))
                .is_none());

            // The load-bearing assertion: the note version is what selects the domain, so
            // an action whose note version says otherwise no longer opens under the very
            // keys its output was encrypted to.
            bundle.actions_mut()[action_index].output.note_version = other_note_version;
            let action = &bundle.actions()[action_index];
            assert!(action.decrypt_output_with_ivk(&ivk).is_none());
            assert!(action.decrypt_compact_output_with_ivk(&ivk).is_none());
            assert!(action.recover_output_with_ovk(&ovk).is_none());
        }
    }

    /// An output the Constructor added under an OVK policy of "None" is readable by its
    /// recipient but recoverable by nobody.
    #[test]
    fn output_without_an_ovk_is_not_recoverable() {
        let mut rng = OsRng;
        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let recipient = fvk.address_at(0u32, Scope::External);
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));

        let (bundle, action_index) =
            single_output_bundle(BundleVersion::orchard_v2(), None, recipient, rng);

        let action = &bundle.actions()[action_index];
        assert!(action.decrypt_output_with_ivk(&ivk).is_some());
        for scope in [Scope::External, Scope::Internal] {
            assert!(action.recover_output_with_ovk(&fvk.to_ovk(scope)).is_none());
        }
    }
}
