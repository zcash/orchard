//! In-band secret distribution for PCZT actions.
//!
//! Which note encryption domain an action's output was encrypted under is a property of the
//! *bundle*, not of the action: it is determined by the bundle's [`BundleVersion`] — its
//! [`ValuePool`] and [`ProtocolVersion`] together — and every action in a bundle shares it.
//! The dependency runs in that direction throughout this crate: [`BundleVersion::note_version`]
//! derives the note plaintext version from the bundle version, so the note version is a
//! consequence of the pool rather than an independent input, and it is not guaranteed to
//! identify the pool (and hence the domain) on its own as further pools are defined.
//!
//! Pairing an output with the wrong domain does not fail loudly: trial decryption simply does
//! not match, and the output is then misclassified (for a Signer, the difference between an
//! output it displays for approval and one it treats as change). The helpers here keep that
//! pairing inside this crate, so a PCZT consumer never reproduces it; each takes the bundle
//! version, so the consumer supplies the pool and protocol version and this module maps them
//! to a domain.
//!
//! That mapping is partial, and every helper is therefore fallible as well as optional. The
//! two outcomes are kept apart on purpose:
//!
//! * `Ok(None)` is the ordinary negative result of trial decryption — this output was not
//!   encrypted to the key supplied, i.e. it is not yours.
//! * [`Err(UnsupportedBundleVersion)`] means this crate defines no note encryption domain for
//!   the bundle version supplied, so no trial decryption was attempted at all. Nothing has
//!   been learned about the output.
//!
//! Collapsing the second into the first would recreate the misclassification this module
//! exists to prevent, one level up: a consumer that reads "no defined domain" as "not mine"
//! would silently treat every output of such a bundle as somebody else's.
//!
//! [`Err(UnsupportedBundleVersion)`]: UnsupportedBundleVersion
//! [`BundleVersion`]: crate::bundle::BundleVersion
//! [`BundleVersion::note_version`]: crate::bundle::BundleVersion::note_version
//! [`ValuePool`]: crate::ValuePool
//! [`ProtocolVersion`]: crate::ProtocolVersion

use core::fmt;

use zcash_note_encryption::{
    try_compact_note_decryption, try_note_decryption, try_output_recovery_with_ovk,
    EphemeralKeyBytes,
};

use crate::{
    bundle::BundleVersion,
    keys::{OutgoingViewingKey, PreparedIncomingViewingKey},
    note_encryption::{CompactAction, IronwoodDomain, OrchardDomain, COMPACT_NOTE_SIZE},
    Address, Note, ProtocolVersion, ValuePool,
};

impl From<&super::Action> for CompactAction {
    fn from(action: &super::Action) -> Self {
        // The compact note ciphertext is the prefix of the full note ciphertext. Both
        // lengths are compile-time constants, and `COMPACT_NOTE_SIZE` is the smaller.
        let mut enc_ciphertext = [0; COMPACT_NOTE_SIZE];
        enc_ciphertext
            .copy_from_slice(&action.output.encrypted_note.enc_ciphertext.0[..COMPACT_NOTE_SIZE]);

        CompactAction::from_parts(
            action.spend.nullifier,
            action.output.cmx,
            EphemeralKeyBytes(action.output.encrypted_note.epk_bytes),
            enc_ciphertext,
        )
    }
}

/// This crate defines no note encryption domain for a [`BundleVersion`].
///
/// This is returned by the trial decryption and output recovery helpers on
/// [`crate::pczt::Action`], and is *not* a decryption failure: no trial decryption was
/// attempted, and nothing has been learned about the action's output. Distinguishing it from
/// `Ok(None)` ("this output was not encrypted to the key supplied") is the caller's
/// responsibility, and the point of the distinction — a consumer that treats it as `Ok(None)`
/// silently misclassifies every output of such a bundle as somebody else's.
///
/// A [`BundleVersion`] is a [`ValuePool`] paired with a [`ProtocolVersion`], and no
/// constructor of [`BundleVersion`] produces a pairing this crate has no domain for today, so
/// this error is unreachable as the API stands. It exists because such a pairing would not be
/// caught by the exhaustive match that selects the domain: that match only fails to compile
/// when a *variant* is added to either enum, and [`BundleVersion`] is `#[non_exhaustive]` with
/// private fields, so a new constructor pairing variants this crate already has changes no
/// enum and breaks no match.
///
/// [`BundleVersion`]: crate::bundle::BundleVersion
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsupportedBundleVersion {
    bundle_version: BundleVersion,
}

impl UnsupportedBundleVersion {
    /// The bundle version for which this crate defines no note encryption domain.
    ///
    /// Its [`value_pool`] and [`protocol_version`] together are what has no domain; neither
    /// is unsupported on its own.
    ///
    /// [`value_pool`]: crate::bundle::BundleVersion::value_pool
    /// [`protocol_version`]: crate::bundle::BundleVersion::protocol_version
    pub fn bundle_version(&self) -> BundleVersion {
        self.bundle_version
    }
}

impl fmt::Display for UnsupportedBundleVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "no note encryption domain is defined for the {:?} value pool under protocol version {:?}",
            self.bundle_version.value_pool(),
            self.bundle_version.protocol_version(),
        )
    }
}

impl core::error::Error for UnsupportedBundleVersion {}

/// The note encryption domain a PCZT action's output was encrypted under.
///
/// [`ActionDomain::for_action`] is the single place in this module that decides which domain
/// that is; the helpers below only dispatch over the result.
enum ActionDomain {
    /// The domain for an action in a bundle in the [`ValuePool::Orchard`] pool.
    Orchard(OrchardDomain),
    /// The domain for an action in a bundle in the [`ValuePool::Ironwood`] pool.
    Ironwood(IronwoodDomain),
}

impl ActionDomain {
    /// The note encryption domain under which `action`'s output was encrypted, for an action
    /// in a bundle of version `bundle_version`.
    ///
    /// The domain is a function of the bundle's [`ValuePool`] and [`ProtocolVersion`] together,
    /// never of a note plaintext version on its own. That function is partial, and two
    /// separate guards keep it honest, because a pairing this crate has no domain for can
    /// arrive in two unrelated ways:
    ///
    /// * A *new variant* of [`ValuePool`] or [`ProtocolVersion`]. The match below is written
    ///   out over both enums with no wildcard arm, so adding one fails to compile here rather
    ///   than silently reusing one of today's domains for it.
    ///
    /// * A *new pairing of existing variants*, which is how such a version is most likely to
    ///   arrive, and which no exhaustiveness check can catch. [`BundleVersion`] is
    ///   `#[non_exhaustive]` with private fields, so every value of it comes from one of its
    ///   constructors; a fifth constructor pairing variants this crate already has changes no
    ///   enum and breaks no match. The error arm below is what such a pairing reaches, instead
    ///   of being handed a domain it was never specified to use.
    ///
    /// Both guards are load-bearing, so neither replaces the other.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedBundleVersion`] if this crate defines no note encryption domain
    /// for `bundle_version`. No constructor of [`BundleVersion`] produces such a version
    /// today, so this is unreachable through the public API as it stands.
    ///
    /// [`BundleVersion`]: crate::bundle::BundleVersion
    fn for_action(
        action: &super::Action,
        bundle_version: BundleVersion,
    ) -> Result<Self, UnsupportedBundleVersion> {
        match (
            bundle_version.value_pool(),
            bundle_version.protocol_version(),
        ) {
            // Every protocol version instantiated for the Orchard pool uses `OrchardDomain`.
            (
                ValuePool::Orchard,
                ProtocolVersion::InsecureV1 | ProtocolVersion::V2 | ProtocolVersion::V3,
            ) => Ok(Self::Orchard(OrchardDomain::for_pczt_action(action))),
            // The Ironwood pool uses `IronwoodDomain`, which is defined from
            // `ProtocolVersion::V3` onward.
            (ValuePool::Ironwood, ProtocolVersion::V3) => {
                Ok(Self::Ironwood(IronwoodDomain::for_pczt_action(action)))
            }
            // The Ironwood pool has no domain before `ProtocolVersion::V3`, so it gets none
            // here either. These pairings are named rather than wildcarded so that the
            // exhaustiveness check above still holds; `BundleVersion` has no constructor for
            // them, so they are unconstructable today.
            (ValuePool::Ironwood, ProtocolVersion::InsecureV1 | ProtocolVersion::V2) => {
                Err(UnsupportedBundleVersion { bundle_version })
            }
        }
    }
}

impl super::Action {
    /// Trial-decrypts this action's output note with the given incoming viewing key, in the
    /// note encryption domain selected by `bundle_version`.
    ///
    /// Returns the note, the address it was sent to, and its memo, or `Ok(None)` if this
    /// action's output was not encrypted to `ivk`.
    ///
    /// `bundle_version` must be the version of the bundle this action belongs to
    /// ([`Bundle::bundle_version`]); it is the bundle's value pool and protocol version that
    /// select the domain. This action's own `note_version` is deliberately not consulted: it
    /// is a restatement of [`BundleVersion::note_version`] that [`Bundle::parse`] rejects a
    /// bundle for disagreeing with, so treating a disagreement as a reason to decline would
    /// turn a tampered field into exactly the silent misclassification these helpers exist to
    /// prevent. Bind it to the output instead with [`Output::verify_note_commitment`], which
    /// fails loudly.
    ///
    /// The incoming viewing key is prepared by the caller ([`PreparedIncomingViewingKey`]),
    /// so a single preparation can be reused across every action in a bundle.
    ///
    /// A successful decryption binds the returned note to this action's `cmx`, but says
    /// nothing about the action's separately-transmitted (and redactable) `recipient` and
    /// `value` fields; use [`Output::verify_note_commitment`] to bind those to the same
    /// `cmx`.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedBundleVersion`] if this crate defines no note encryption domain
    /// for `bundle_version`, in which case no trial decryption was attempted and nothing has
    /// been learned about this action's output. This is distinct from `Ok(None)`, which is
    /// the ordinary negative result of trial decryption; treating it as `Ok(None)` would
    /// misclassify the output as somebody else's.
    ///
    /// [`Bundle::bundle_version`]: super::Bundle::bundle_version
    /// [`Bundle::parse`]: super::Bundle::parse
    /// [`BundleVersion::note_version`]: crate::bundle::BundleVersion::note_version
    /// [`Output::verify_note_commitment`]: super::Output::verify_note_commitment
    pub fn decrypt_output_with_ivk(
        &self,
        ivk: &PreparedIncomingViewingKey,
        bundle_version: BundleVersion,
    ) -> Result<Option<(Note, Address, [u8; 512])>, UnsupportedBundleVersion> {
        Ok(match ActionDomain::for_action(self, bundle_version)? {
            ActionDomain::Orchard(domain) => try_note_decryption(&domain, ivk, self),
            ActionDomain::Ironwood(domain) => try_note_decryption(&domain, ivk, self),
        })
    }

    /// Trial-decrypts the compact part of this action's output note with the given incoming
    /// viewing key, in the note encryption domain selected by `bundle_version`.
    ///
    /// Returns the note and the address it was sent to, or `Ok(None)` if this action's output
    /// was not encrypted to `ivk`. This is the trial decryption a consumer that only ever
    /// sees the compact note ciphertext (a light client, or a hardware wallet that is
    /// streamed one) performs, reproduced here over the full PCZT action.
    ///
    /// `bundle_version` selects the domain exactly as it does for
    /// [`Action::decrypt_output_with_ivk`].
    ///
    /// Unlike [`Action::decrypt_output_with_ivk`], the note ciphertext is not
    /// authenticated by its AEAD tag; the returned note is instead validated by
    /// re-deriving this action's `cmx`, and the memo is not recovered.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedBundleVersion`] on the same terms as
    /// [`Action::decrypt_output_with_ivk`]: this crate defines no note encryption domain for
    /// `bundle_version`, so no trial decryption was attempted, and the result must not be
    /// read as `Ok(None)`.
    ///
    /// [`Action::decrypt_output_with_ivk`]: Self::decrypt_output_with_ivk
    pub fn decrypt_compact_output_with_ivk(
        &self,
        ivk: &PreparedIncomingViewingKey,
        bundle_version: BundleVersion,
    ) -> Result<Option<(Note, Address)>, UnsupportedBundleVersion> {
        let compact = CompactAction::from(self);
        Ok(match ActionDomain::for_action(self, bundle_version)? {
            ActionDomain::Orchard(domain) => try_compact_note_decryption(&domain, ivk, &compact),
            ActionDomain::Ironwood(domain) => try_compact_note_decryption(&domain, ivk, &compact),
        })
    }

    /// Recovers this action's output note with the given outgoing viewing key, in the note
    /// encryption domain selected by `bundle_version`.
    ///
    /// Returns the note, the address it was sent to, and its memo, or `Ok(None)` if this
    /// action's output was not encrypted with `ovk`. This is the sender-side recovery of
    /// [Zcash Protocol Specification § 4.20.3][decryptovk]; it is the way the party that
    /// created an output reads it back out of a PCZT, and it fails for an output that a
    /// Constructor added under an OVK policy of "None".
    ///
    /// `bundle_version` selects the domain exactly as it does for
    /// [`Action::decrypt_output_with_ivk`].
    ///
    /// A successful recovery binds the returned note to this action's `cmx`, but says
    /// nothing about the action's separately-transmitted (and redactable) `recipient` and
    /// `value` fields; use [`Output::verify_note_commitment`] to bind those to the same
    /// `cmx`.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedBundleVersion`] on the same terms as
    /// [`Action::decrypt_output_with_ivk`]: this crate defines no note encryption domain for
    /// `bundle_version`, so no recovery was attempted, and the result must not be read as
    /// `Ok(None)` — which here would mean "I did not create this output".
    ///
    /// [decryptovk]: https://zips.z.cash/protocol/protocol.pdf#decryptovk
    /// [`Action::decrypt_output_with_ivk`]: Self::decrypt_output_with_ivk
    /// [`Output::verify_note_commitment`]: super::Output::verify_note_commitment
    pub fn recover_output_with_ovk(
        &self,
        ovk: &OutgoingViewingKey,
        bundle_version: BundleVersion,
    ) -> Result<Option<(Note, Address, [u8; 512])>, UnsupportedBundleVersion> {
        let out_ciphertext = &self.output.encrypted_note.out_ciphertext;
        Ok(match ActionDomain::for_action(self, bundle_version)? {
            ActionDomain::Orchard(domain) => {
                try_output_recovery_with_ovk(&domain, ovk, self, &self.cv_net, out_ciphertext)
            }
            ActionDomain::Ironwood(domain) => {
                try_output_recovery_with_ovk(&domain, ovk, self, &self.cv_net, out_ciphertext)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::{rand_core::UnwrapErr, rngs::SysRng};
    use zcash_note_encryption::ShieldedOutput;

    use crate::{
        builder::{Builder, BundleType},
        bundle::BundleVersion,
        constants::MERKLE_DEPTH_ORCHARD,
        keys::{
            FullViewingKey, OutgoingViewingKey, PreparedIncomingViewingKey, Scope, SpendingKey,
        },
        note::Rho,
        note_encryption::{CompactAction, OrchardDomain, COMPACT_NOTE_SIZE},
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

    /// Every [`BundleVersion`] instantiated for the [`crate::ValuePool::Orchard`] pool.
    ///
    /// Together with [`IRONWOOD_POOL_VERSIONS`], every [`BundleVersion`] this crate can
    /// construct; add a new constructor to whichever of the two it belongs to.
    const ORCHARD_POOL_VERSIONS: &[BundleVersion] = &[
        BundleVersion::orchard_insecure_v1(),
        BundleVersion::orchard_v2(),
        BundleVersion::orchard_v3(),
    ];

    /// Every [`BundleVersion`] instantiated for the [`crate::ValuePool::Ironwood`] pool.
    ///
    /// See [`ORCHARD_POOL_VERSIONS`].
    const IRONWOOD_POOL_VERSIONS: &[BundleVersion] = &[BundleVersion::ironwood_v3()];

    /// Every [`BundleVersion`] this crate can construct.
    fn all_bundle_versions() -> impl Iterator<Item = BundleVersion> {
        ORCHARD_POOL_VERSIONS
            .iter()
            .chain(IRONWOOD_POOL_VERSIONS)
            .copied()
    }

    /// Builds a PCZT bundle containing a single output paying `recipient` under
    /// `bundle_version`, and returns it alongside the index of the action carrying that
    /// output.
    fn single_output_bundle(
        bundle_version: BundleVersion,
        ovk: Option<OutgoingViewingKey>,
        recipient: Address,
        mut rng: UnwrapErr<SysRng>,
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
        let mut rng = UnwrapErr(SysRng);
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
            ShieldedOutput::<OrchardDomain>::ephemeral_key(&compact).0,
            action.output().encrypted_note().epk_bytes,
        );
        assert_eq!(
            ShieldedOutput::<OrchardDomain>::enc_ciphertext_compact(&compact).0[..],
            action.output().encrypted_note().enc_ciphertext.0[..COMPACT_NOTE_SIZE],
        );
    }

    /// Every trial decryption on a PCZT action selects its domain from the value pool of the
    /// [`BundleVersion`] it is passed, for both of the pools defined today:
    ///
    /// - every protocol version instantiated for the output's own pool opens the output,
    /// - the other pool's bundle version opens nothing, and
    /// - neither outcome moves when the action's own `note_version` is rewritten, because it
    ///   is not an input to the choice of domain.
    #[test]
    fn decryption_domain_follows_the_bundle_value_pool() {
        let mut rng = UnwrapErr(SysRng);

        for (bundle_version, same_pool_versions, other_pool_version, other_note_version) in [
            (
                BundleVersion::orchard_v2(),
                ORCHARD_POOL_VERSIONS,
                BundleVersion::ironwood_v3(),
                NoteVersion::V3,
            ),
            (
                BundleVersion::ironwood_v3(),
                IRONWOOD_POOL_VERSIONS,
                BundleVersion::orchard_v2(),
                NoteVersion::V2,
            ),
        ] {
            let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
            let recipient = fvk.address_at(0u32, Scope::External);
            let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
            let ovk = fvk.to_ovk(Scope::External);
            let other_fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
            let other_ivk = PreparedIncomingViewingKey::new(&other_fvk.to_ivk(Scope::External));
            let other_ovk = other_fvk.to_ovk(Scope::External);

            let (mut bundle, action_index) =
                single_output_bundle(bundle_version, Some(ovk.clone()), recipient, rng);

            // The action's own note version is the bundle version's, by construction. The
            // assertions below hold both with that agreement intact and with it broken.
            assert_eq!(
                bundle.actions()[action_index].output().note_version(),
                &bundle_version.note_version(),
            );

            for note_version in [bundle_version.note_version(), other_note_version] {
                bundle.actions_mut()[action_index].output.note_version = note_version;
                let action = &bundle.actions()[action_index];

                // Every protocol version instantiated for this output's pool selects the
                // domain the output was encrypted under, so all of them open it.
                for version in same_pool_versions {
                    let (note, address, memo) = action
                        .decrypt_output_with_ivk(&ivk, *version)
                        .expect("a domain is defined for every constructible bundle version")
                        .expect("the recipient's IVK opens the output under its own pool");
                    assert_eq!(address, recipient);
                    assert_eq!(note.value(), NoteValue::from_raw(OUTPUT_VALUE_ZATS));
                    assert_eq!(memo, MEMO);

                    // The compact trial decryption recovers the same note, without the memo.
                    assert_eq!(
                        action.decrypt_compact_output_with_ivk(&ivk, *version),
                        Ok(Some((note, address))),
                    );

                    // The sender recovers the same note and memo under the OVK it used.
                    assert_eq!(
                        action.recover_output_with_ovk(&ovk, *version),
                        Ok(Some((note, address, memo))),
                    );

                    // Another account's keys open nothing. That is `Ok(None)`, not an error:
                    // the domain was defined, the trial decryption simply did not match.
                    assert_eq!(
                        action.decrypt_output_with_ivk(&other_ivk, *version),
                        Ok(None),
                    );
                    assert_eq!(
                        action.decrypt_compact_output_with_ivk(&other_ivk, *version),
                        Ok(None),
                    );
                    assert_eq!(
                        action.recover_output_with_ovk(&other_ovk, *version),
                        Ok(None),
                    );
                }

                // The load-bearing assertion: the bundle's value pool is what selects the
                // domain, so under the other pool's bundle version the output no longer
                // opens under the very keys it was encrypted to. Both pools have a domain
                // defined, so this too is `Ok(None)` rather than an error.
                assert_eq!(
                    action.decrypt_output_with_ivk(&ivk, other_pool_version),
                    Ok(None),
                );
                assert_eq!(
                    action.decrypt_compact_output_with_ivk(&ivk, other_pool_version),
                    Ok(None),
                );
                assert_eq!(
                    action.recover_output_with_ovk(&ovk, other_pool_version),
                    Ok(None),
                );
            }
        }
    }

    /// An output the Constructor added under an OVK policy of "None" is readable by its
    /// recipient but recoverable by nobody.
    #[test]
    fn output_without_an_ovk_is_not_recoverable() {
        let mut rng = UnwrapErr(SysRng);
        let bundle_version = BundleVersion::orchard_v2();
        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let recipient = fvk.address_at(0u32, Scope::External);
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));

        let (bundle, action_index) = single_output_bundle(bundle_version, None, recipient, rng);

        let action = &bundle.actions()[action_index];
        assert!(action
            .decrypt_output_with_ivk(&ivk, bundle_version)
            .expect("a domain is defined for every constructible bundle version")
            .is_some());
        for scope in [Scope::External, Scope::Internal] {
            assert_eq!(
                action.recover_output_with_ovk(&fvk.to_ovk(scope), bundle_version),
                Ok(None),
            );
        }
    }

    /// Every [`BundleVersion`] this crate can construct has a note encryption domain defined
    /// for it, so [`UnsupportedBundleVersion`] is unreachable through the public API as it
    /// stands: `BundleVersion` is `#[non_exhaustive]` with private fields, and none of its
    /// four constructors pairs the Ironwood pool with a protocol version before
    /// `ProtocolVersion::V3`.
    ///
    /// The error is therefore not pinned by a test that provokes it — there is no way to
    /// build its input — but by this test, which says what must stay true of the versions
    /// that *can* be built. A future constructor that pairs existing variants would change no
    /// enum and break no match; if it is added to [`ORCHARD_POOL_VERSIONS`] or
    /// [`IRONWOOD_POOL_VERSIONS`] without a domain arm in [`super::ActionDomain::for_action`],
    /// this test is what fails.
    ///
    /// [`UnsupportedBundleVersion`]: super::UnsupportedBundleVersion
    #[test]
    fn every_constructible_bundle_version_has_a_domain() {
        let mut rng = UnwrapErr(SysRng);
        let fvk = FullViewingKey::from(&SpendingKey::random(&mut rng));
        let recipient = fvk.address_at(0u32, Scope::External);
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
        let ovk = fvk.to_ovk(Scope::External);

        let (bundle, action_index) = single_output_bundle(
            BundleVersion::orchard_v2(),
            Some(ovk.clone()),
            recipient,
            rng,
        );
        let action = &bundle.actions()[action_index];

        for version in all_bundle_versions() {
            assert!(
                action.decrypt_output_with_ivk(&ivk, version).is_ok(),
                "no domain defined for {version:?}",
            );
            assert!(
                action
                    .decrypt_compact_output_with_ivk(&ivk, version)
                    .is_ok(),
                "no domain defined for {version:?}",
            );
            assert!(
                action.recover_output_with_ovk(&ovk, version).is_ok(),
                "no domain defined for {version:?}",
            );
        }
    }
}
