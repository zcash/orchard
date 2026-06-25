# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

All changes in this release support the NU6.3 `enableCrossAddress` bundle flag
and the post-NU 6.3 Orchard Action circuit that enforces the cross-address
restriction. Construction and wire encoding are now selected by a single
`orchard::bundle::BundlePoolRestrictions` (a `(pool, era)` value); existing callers keep
the current behavior by selecting `BundlePoolRestrictions::OrchardNu6_2Only` (and
`OrchardCircuitVersion::FixedPostNu6_2` when building proving/verifying keys).

### Added
- `orchard::bundle::Flags` APIs for the NU6.3 `enableCrossAddress` flag:
  - `Flags::CROSS_ADDRESS_DISABLED`, the restricted flag set. It cannot be
    encoded in pre-NU6.3 formats.
  - `Flags::cross_address_enabled`
- `orchard::bundle::BundlePoolRestrictions`, the `(pool, era)` selector for an Orchard
  bundle. It determines the circuit version (`BundlePoolRestrictions::circuit_version`),
  the note plaintext version (`BundlePoolRestrictions::note_version`),
  the flag-byte interpretation (pre-NU6.3 rules, where bit 2 is reserved and
  cross-address transfers are implicitly enabled, vs NU6.3 rules, where bit 2 is
  `enableCrossAddress`), and whether consensus mandates the cross-address
  restriction (the builder then chooses the value within that constraint).
  Variants: `OrchardPreNu6_2`, `OrchardNu6_2Only`, `OrchardNu6_3Onward`, and
  `IronwoodNu6_3Onward` (which shares the post-NU6.3 circuit and uses V3 note
  plaintexts).
- `orchard::bundle::TxVersion`, the transaction version (`V5` or `V6`) a bundle's
  commitments are computed for. At NU6.3 an Orchard bundle may be encoded in a v5
  or a v6 transaction; the two use different commitment personalizations and place
  the anchor in different digests (v5 in the transaction-ID effects, v6 in the
  authorizing data). It is passed to `Bundle::commitment`,
  `Bundle::authorizing_commitment`, and the `hash_bundle_*_empty` helpers.
- `orchard::note::NoteVersion`, the note plaintext version selector used by
  low-level note constructors and accessors. `NoteVersion::V2` is the ZIP 212
  Orchard note plaintext format, and `NoteVersion::V3` is the quantum-recoverable
  Ironwood note plaintext version defined in ZIP 2005.
- `orchard::Note::version`, returning the note's plaintext version.
- `orchard::circuit::OrchardCircuitVersion::PostNu6_3`, the circuit version
  that enforces the `disableCrossAddress` public input. The post-NU 6.3 circuit
  has its own proving and verifying keys.
- Circuit-version support introspection for the cross-address restriction:
  - `orchard::circuit::OrchardCircuitVersion::supports_cross_address_restriction`
  - `orchard::circuit::ProvingKey::supports_cross_address_restriction`
  - `orchard::circuit::VerifyingKey::circuit_version`
  - `orchard::circuit::VerifyingKey::supports_cross_address_restriction`
- Wallet-controlled change outputs, the only way to retain shielded value in
  a bundle that disables cross-address transfers:
  - `orchard::builder::Builder::add_change_output`
  - `orchard::builder::Builder::changes`
  - `orchard::builder::ChangeInfo` and `orchard::builder::ChangeInfo::new`, the
    change-output counterpart of `OutputInfo`, recording the full viewing key
    that owns the recipient (validated on construction) so the builder can
    fabricate the paired same-expanded-receiver spend.
- `orchard::pczt::Bundle::verify_cross_address_restriction`, so that Signers
  can check the cross-address restriction's same-expanded-receiver structural property
  before signing. It is a no-op for bundles that permit cross-address
  transfers.
- Error variants for the cross-address builder and PCZT checks:
  - `orchard::builder::BuildError::CrossAddressDisabled`
  - `orchard::builder::BuildError::InvalidNoteVersion`
  - `orchard::builder::OutputError::{CrossAddressDisabled, SpendsDisabled, RecipientNotOwned}`
  - `orchard::pczt::ParseError::InvalidNoteVersion`
  - `orchard::pczt::VerifyError::DisallowedCrossAddressTransfer`
  - `orchard::pczt::ProverError::DisallowedCrossAddressTransfer`, wrapping the
    underlying `orchard::pczt::VerifyError`
  - `orchard::pczt::IoFinalizerError::CrossAddressRestriction`
- Note version fields in PCZT spends and outputs, with generated public getters:
  - `orchard::pczt::Spend::note_version`
  - `orchard::pczt::Output::note_version`
- `orchard::note_encryption::IronwoodDomain` and
  `orchard::note_encryption::IronwoodNoteEncryption`, matching
  `OrchardDomain` note-encryption behavior but accepting V3 note plaintexts
  during parsing.
- `orchard::note_encryption::{DomainVersion, NoteEncryptionDomain, OrchardVersion,
  IronwoodVersion}`, the sealed marker-domain API underlying `OrchardDomain`
  and `IronwoodDomain`.
- `orchard::bundle::BatchError`, with its `RestrictionUnsupportedByKey` variant,
  returned by `orchard::bundle::BatchValidator::add_bundle`.
- `orchard::bundle::CommitmentError`, with its `UnrepresentableFlags` variant,
  returned by `orchard::Bundle::commitment`.
- `orchard::bundle::testing::arb_flags_ironwood_post_nu6_3` (under the
  `test-dependencies` feature), a strategy that generates flag sets valid for an
  Ironwood bundle post-NU6.3, including flag sets that disable cross-address
  transfers. `arb_flags` is unchanged and only generates flag sets with
  cross-address transfers enabled, which are representable under every pool
  restriction other than Orchard post-NU6.3.
- `orchard::bundle::testing::arb_bundle_pool_restriction` (under the
  `test-dependencies` feature), a strategy that generates bundle pool
  restrictions.
- `RandomSeed::rcm_v3` (behind the `unstable-voting-circuits` feature), the rcm
  derivation for V3 (ZIP 2005, Ironwood) notes, for low-level construction of
  Ironwood note plaintexts. These APIs are not covered by the crate's semver
  guarantees.

### Changed
- `orchard::bundle::Flags::{from_byte, to_byte}` and
  `orchard::pczt::Bundle::parse` now take a `BundlePoolRestrictions`. Bit 2
  (`enableCrossAddress`) is only representable for the Ironwood pool post-NU6.3;
  it is rejected for pre-NU6.3 (where bit 2 is reserved) and for Orchard
  post-NU6.3 (where consensus mandates the cross-address restriction).
  `Flags::to_byte` now returns `Option<u8>`, yielding `None` when the flag set is
  not representable under the given pool restrictions (cross-address transfers
  disabled under pre-NU6.3, or enabled under Orchard post-NU6.3). A byte with
  bit 2 clear is interpreted differently per epoch: an unrestricted bundle
  before NU6.3, a restricted bundle under NU6.3.
- Key- and circuit-building APIs now take the intended `OrchardCircuitVersion`
  explicitly instead of implicitly selecting `FixedPostNu6_2` — pass
  `FixedPostNu6_2` for the previous behavior, or `PostNu6_3` for restricted
  proofs:
  - `orchard::circuit::ProvingKey::build`
  - `orchard::circuit::VerifyingKey::build`
  - `orchard::circuit::Circuit::from_action_context`
- Bundle construction now takes a `BundlePoolRestrictions` instead of a circuit version:
  - `orchard::builder::Builder::new` takes the `BundlePoolRestrictions`, and
    `orchard::builder::Builder::build` no longer takes a circuit-version argument
    (it derives the circuit version from the protocol).
  - `orchard::builder::bundle` takes the `BundlePoolRestrictions` in place of the
    circuit-version argument.
  - The builder derives new output, dummy, and fabricated note plaintext versions
    from `BundlePoolRestrictions::note_version`.
  - The lower-level `orchard::builder::bundle` helper rejects supplied
    `OutputInfo` and `ChangeInfo` values whose note version does not match the
    `BundlePoolRestrictions`, returning `BuildError::InvalidNoteVersion`.
- Bundle output decryption and recovery helpers now take a `BundlePoolRestrictions`
  and enforce its note plaintext version after decryption:
  - `orchard::Bundle::{decrypt_outputs_with_keys, decrypt_output_with_key}`
  - `orchard::Bundle::{recover_outputs_with_ovks, recover_output_with_ovk}`
  Selecting `BundlePoolRestrictions::IronwoodNu6_3Onward` lets these helpers
  discover V3 Ironwood notes.
- Low-level note and builder-info constructors now take an explicit
  `orchard::note::NoteVersion` so callers choose between V2 Orchard notes and V3
  Ironwood notes. Affected API:
  - `orchard::Note::from_parts`
  - `orchard::Note::new`
  - `orchard::builder::OutputInfo::new`
  - `orchard::builder::ChangeInfo::new`
- `orchard::builder::BundleType::Transactional` no longer embeds a full `Flags`;
  it carries `{ spends_enabled, outputs_enabled, bundle_required }`, and
  `BundleType::flags` and `BundleType::num_actions` now take the
  `BundlePoolRestrictions`. The builder chooses the cross-address bit as a prover-side
  default — the least-restrictive value consensus permits: enabled, except under
  `OrchardNu6_3Onward`, where consensus mandates the restriction. `BundlePoolRestrictions`
  exposes only that consensus constraint; the default lives in builder logic. The
  `Flags` codec still represents NU6.3 `enableCrossAddress = 0` flag sets, so a
  future builder could expose the choice where consensus leaves it free (e.g.
  Ironwood); this branch does not. Coinbase bundles follow the same pool
  restrictions as non-coinbase bundles: post-NU6.3 Orchard coinbase
  transactions cannot contain Orchard actions, so post-NU6.3 coinbase bundle
  construction in this crate is only useful for `IronwoodNu6_3Onward`.
- `orchard::circuit::Instance::from_parts` now takes an
  `orchard::bundle::Flags` argument instead of separate spend/output enable
  booleans, so the cross-address restriction is carried into the public
  instances.
- Proof APIs reject instances that disable cross-address transfers unless the
  key's circuit version supports the cross-address restriction.
  `orchard::Proof::{create, verify}` and `orchard::Bundle::verify_proof`
  return `halo2_proofs::plonk::Error::InvalidInstances`; with pre-NU 6.3
  keys, proving a restricted builder-created bundle returns
  `orchard::builder::BuildError::Proof`, and PCZT proving returns
  `orchard::pczt::ProverError::ProofFailed`. Restricted
  bundles can still be constructed and round-tripped —
  `orchard::Bundle::<Authorized, V>::try_from_parts` and
  `orchard::pczt::Bundle::extract` preserve the flag — with enforcement at
  proving and verification.
- `orchard::bundle::BatchValidator` binds its verifying key at construction:
  `BatchValidator::new` now takes a `&orchard::circuit::VerifyingKey`, and
  `BatchValidator::validate` no longer takes one. `BatchValidator::add_bundle`
  now returns `Result<(), orchard::bundle::BatchError>`, rejecting a bundle that
  disables cross-address transfers — without adding it to the batch — when the
  verifying key's circuit version does not support the cross-address
  restriction. Other proof or signature failures still surface as
  `validate` returning `false`.
- `orchard::builder::Builder` constructs bundles that disable cross-address
  transfers as withdrawal/change bundles in which every action's output is
  addressed to the expanded receiver of the note it spends. The fabricated
  zero-value output paired with each real spend carries a randomized,
  undecryptable note ciphertext rather than one encrypted to the spent note's
  receiver, so neither the owning wallet nor a holder of that receiver's
  incoming viewing key can use it to detect the spend.
  - `Builder::add_output` returns `OutputError::CrossAddressDisabled` for
    these bundles; use `Builder::add_change_output` for retained value, which
    rejects a recipient not owned by the full viewing key
    (`OutputError::RecipientNotOwned`) and, in a restricted bundle, requires
    spends to be enabled (`OutputError::SpendsDisabled`).
  - `orchard::builder::bundle` takes the change outputs as a separate
    `changes: Vec<ChangeInfo>` argument (plain `outputs` and wallet-controlled
    `changes` are distinct), and `orchard::builder::BundleMetadata` numbers
    requested outputs as the plain outputs followed by the change outputs.
  - `orchard::builder::BundleType::num_actions` counts
    `num_spends + num_outputs` requested actions rather than the maximum of
    the two (a requested spend and a requested output never share an action),
    and `orchard::builder::BundleMetadata` maps them to distinct actions.
    Wallets estimating fees (e.g. per ZIP 317) must account for the larger
    action count.
- `orchard::pczt::Bundle::create_proof` now builds the Action circuits for
  the provided `ProvingKey`'s circuit version (previously always
  `FixedPostNu6_2`), and checks the cross-address restriction's same-expanded-receiver
  property, returning `ProverError::DisallowedCrossAddressTransfer` (or
  `ProverError::MissingRecipient` if a `recipient` field is unset).
- `orchard::pczt::{Spend, Output}::parse` now take the note plaintext version
  for the parsed spend or output, and `orchard::pczt::Bundle::parse` rejects
  actions whose output note version does not match the `BundlePoolRestrictions`.
- `orchard::note_encryption::{OrchardNoteEncryption, IronwoodNoteEncryption}`
  documentation now clarifies that encryption uses the note's own
  `NoteVersion`; the aliases differ in which note plaintext versions they
  accept during parsing and decryption.
- `orchard::builder::OutputInfo::dummy` now takes an explicit
  `orchard::note::NoteVersion`; builder-created outputs use the note version
  associated with the selected `BundlePoolRestrictions`.
- `test-dependencies` note and bundle strategies now select note versions from
  their callers or generated bundle pool restrictions:
  - `orchard::note::testing::arb_note` now takes a note version.
  - `orchard::bundle::testing::{arb_action, arb_unauthorized_action}` now take a
    note version through their re-exported path.
  - `orchard::bundle::testing::{arb_action_n, arb_unauthorized_action_n}` now
    take a note version.
  - `orchard::bundle::testing::{arb_bundle, arb_unauthorized_bundle}` now
    generate a bundle pool restriction internally to select the note version.
- `orchard::pczt::Bundle::finalize_io` verifies the cross-address restriction
  before modifying the bundle, returning
  `IoFinalizerError::CrossAddressRestriction` (wrapping the underlying
  `VerifyError`) and leaving the bundle unmodified if the PCZT is missing
  recipient data or violates the restriction.
- `orchard::Bundle::{commitment, authorizing_commitment}` now take a
  `orchard::bundle::BundlePoolRestrictions` (which selects the flag-byte encoding) and
  an `orchard::bundle::TxVersion` (which selects the commitment personalization
  strings and the anchor placement for the transaction the bundle is encoded in). The
  ZIP-244 digest — and therefore the transaction ID and sighash — now depends on both:
  under a NU6.3 protocol an unrestricted bundle's flag byte sets bit 2, and
  `TxVersion::V6` uses the v6 personalization strings and commits the anchor in the
  authorizing commitment instead of the effects commitment. Callers computing
  transaction IDs or sighashes must pass the restrictions and version matching the
  transaction; these APIs do not validate that the selected pool/era is consensus-valid
  for the selected transaction version. `Bundle::commitment` now returns
  `Result<BundleCommitment, CommitmentError>`, returning
  `Err(CommitmentError::UnrepresentableFlags)` if the flags are unrepresentable
  under those restrictions (for example, cross-address transfers disabled under
  pre-NU6.3 restrictions).
- `orchard::bundle::commitments::{hash_bundle_txid_empty, hash_bundle_auth_empty}`
  now take a `BundlePoolRestrictions` and a `TxVersion`.
- Behind the `unstable-voting-circuits` feature, `RandomSeed::rcm` is renamed to
  `RandomSeed::rcm_v2`, marking it as the rcm derivation for V2 (ZIP 212) notes.
  These APIs are not covered by the crate's semver guarantees.

### Removed
- The temporary `_for_version` APIs from `0.14.0`; pass the intended
  `OrchardCircuitVersion` (keys/circuit) or `BundlePoolRestrictions` (construction) to
  the plain APIs listed above instead:
  - `orchard::circuit::ProvingKey::build_for_version`
  - `orchard::circuit::VerifyingKey::build_for_version`
  - `orchard::circuit::Circuit::from_action_context_for_version`
  - `orchard::builder::Builder::new_for_version` (use `Builder::new`, which now
    takes the `BundlePoolRestrictions`)
  - `orchard::builder::bundle_for_version`
- The `Default` impls for `orchard::circuit::Circuit` and
  `orchard::circuit::OrchardCircuitVersion`; callers must choose a circuit
  version explicitly.
- The `Default` impl for `orchard::bundle::BatchValidator`; construct it with
  `BatchValidator::new`, which now requires a verifying key.
- `orchard::Proof::add_to_batch` is no longer public. A raw batch is finalized
  against a caller-supplied verifying key, so this API let a caller batch
  instances that disable cross-address transfers and then finalize them against
  a key whose circuit version does not constrain the `disableCrossAddress`
  public input, bypassing the cross-address restriction. Use
  `orchard::bundle::BatchValidator`, which binds its verifying key at
  construction and enforces the restriction in `BatchValidator::add_bundle`.

### Fixed
- The `Display` output of `orchard::builder::BuildError::OutputsDisabled`
  previously described spends rather than outputs.

## [0.14.0] - 2026-06-02

### Added
- `orchard::action::ActionFromPartsError`
- `orchard::Proof::expected_proof_size`, the canonical byte length of a proof
  for a given number of actions.
- `orchard::bundle::BundleError`
- `impl From<orchard::action::ActionFromPartsError> for orchard::pczt::TxExtractorError`
- `impl From<orchard::bundle::BundleError> for orchard::pczt::TxExtractorError`
- `orchard::bundle::ProofSizeEnforcement`
- `orchard::Bundle::<Authorized, V>::try_from_parts`, which constructs an
  authorized bundle while rejecting a proof whose length is not the canonical
  size for the bundle's number of actions (GHSA-2x4w-pxqw-58v9). This is now the
  only way to construct a `Bundle<Authorized, _>`, so an authorized bundle can
  no longer hold a proof padded with arbitrary data when proof size enforcement
  is strict.
- `orchard::Bundle::<EffectsOnly, V>::from_parts`
- `orchard::circuit::OrchardCircuitVersion`, an enum selecting the Action circuit
  version, with variants `InsecurePreNu6_2` and `FixedPostNu6_2`.
- `orchard::circuit::ProvingKey::build_for_version` and
  `orchard::circuit::VerifyingKey::build_for_version`, which build the key for a
  given `OrchardCircuitVersion`; `build()` continues to build the fixed circuit.
  `ProvingKey::build_for_version` can build the proving key for the pre-NU6.2
  (insecure) circuit.
- `orchard::circuit::ProvingKey::circuit_version`, the version the proving key
  produces proofs for. `Proof::create` now returns an error if a circuit's
  version does not match the proving key's.
- `orchard::circuit::Circuit::from_action_context_for_version`, like
  `from_action_context` but building the circuit for a chosen
  `OrchardCircuitVersion`.
- `orchard::builder::Builder::new_for_version` (requires the `circuit` feature),
  which constructs a builder that produces proofs for a given
  `OrchardCircuitVersion` (`Builder::new` uses `FixedPostNu6_2`).
- `orchard::builder::bundle_for_version` (requires the `circuit` feature), like
  `bundle` but building the Action circuits for a given `OrchardCircuitVersion`.
- `orchard::Bundle::<InProgress<Unproven, S>, V>::circuit_version` (requires the
  `circuit` feature), the `OrchardCircuitVersion` the bundle's actions were built
  for, so a caller can select a matching `ProvingKey` without tracking it
  separately.

### Changed
- Updated to `halo2_gadgets 0.5.0`
- `orchard::action::Action::from_parts` now returns
  `Result<Self, orchard::action::ActionFromPartsError>` instead of `Option<Self>`.
- `orchard::pczt::TxExtractorError` has added variants `InvalidEpk` and
  `NonCanonicalProofSize`. The Transaction Extractor role now rejects a PCZT
  whose `zkproof` is not the canonical size for its number of actions
  (GHSA-2x4w-pxqw-58v9).
- `unstable-voting-circuits`-only:
  - `orchard::constants::OrchardFixedBases` is now a unit struct rather than a
    3-variant enum. It is a trait carrier for the halo2_gadgets `FixedPoints`
    impl and was never constructed as a value; the concrete fixed bases live
    in `OrchardFixedBasesFull`, `OrchardBaseFieldBases`, and
    `OrchardShortScalarBases`, which are unchanged.

### Removed
- `orchard::Bundle::from_parts`. Construct a bundle through the
  authorization-specific constructor instead: `Bundle::<EffectsOnly, V>::from_parts`,
  or `Bundle::<Authorized, V>::try_from_parts` for an authorized bundle.
- `unstable-voting-circuits`-only:
  - The five dead `From<X> for OrchardFixedBases` conversions (from
    `OrchardFixedBasesFull`, `NullifierK`, `ValueCommitV`,
    `OrchardBaseFieldBases`, `OrchardShortScalarBases`). None were reachable;
    in-circuit dispatch goes through the per-slot enums.
  - `impl FixedPoint<pallas::Affine> for NullifierK` and
    `impl FixedPoint<pallas::Affine> for ValueCommitV`. After the 0.13.1
    enum refactor, dispatch routes through `OrchardBaseFieldBases::NullifierK`
    and `OrchardShortScalarBases::ValueCommitV`, leaving the standalone
    unit-struct impls dead.

### Fixed
- The update to `halo2_gadgets 0.5.0` fixes a critical vulnerability related to
  its use in the Orchard circuit. Please see the release notes for
  `halo2_gadgets 0.5.0` for additional details.
- An authorized `Bundle` or a PCZT can no longer carry a `zkproof` padded with
  arbitrary trailing data, and an `Action` can no longer be constructed with an
  `epk` that does not encode a non-identity Pallas point (GHSA-2x4w-pxqw-58v9).
  See the `Bundle::<Authorized, V>::try_from_parts`,
  `Proof::expected_proof_size`, `Action::from_parts`, and `TxExtractorError`
  entries under `Added` and `Changed` above for the API surface of these checks.

## [0.13.1] - 2026-04-27

### Added
- `orchard::{L_ORCHARD_BASE, L_ORCHARD_SCALAR, L_VALUE}`, the bit-length
  parameters of the Orchard base field, scalar field, and value encoding
  as defined in the Zcash protocol specification.
- `orchard::value::NoteValue::ZERO`, a `const NoteValue` equal to zero.
- The following modules and APIs are available behind the
  `unstable-voting-circuits` feature flag to support downstream
  voting-circuit development. These temporary APIs are not covered by the
  crate's semver stability guarantees and may change in any future release:
  - Modules: `orchard::{constants, spec}`,
    `orchard::circuit::{commit_ivk, commit_ivk::gadgets, note_commit,
    note_commit::gadgets, gadget::add_chip}`,
    `orchard::note::{commitment, nullifier}`.
  - Address and circuit helpers: `Address::{g_d, pk_d}`,
    `circuit::gadget::{AddInstruction, assign_free_advice, derive_nullifier,
    commit_ivk, note_commit}`,
    `circuit::gadget::add_chip::{AddConfig, AddChip}` and
    `AddChip::{configure, construct}`,
    `CommitIvkChip::{configure, construct}`,
    `NoteCommitChip::{configure, construct}`.
  - Fixed bases: `orchard::constants::OrchardFixedBases` has three
    variants: `Full(OrchardFixedBasesFull)` for full-width scalar
    multiplication, `Base(OrchardBaseFieldBases)` for base-field
    scalars, and `Short(OrchardShortScalarBases)` for short signed
    scalars. `OrchardBaseFieldBases` covers `NullifierK` and
    `SpendAuthGBase`; `OrchardShortScalarBases` covers `ValueCommitV`
    and `SpendAuthGShort`. `From<NullifierK>`, `From<ValueCommitV>`,
    `From<OrchardFixedBasesFull>`, `From<OrchardBaseFieldBases>`, and
    `From<OrchardShortScalarBases>` conversions to `OrchardFixedBases`
    are provided.
  - Key, note, tree, and value APIs: `SpendingKey::random`,
    `SpendAuthorizingKey::derive_inner`, `NullifierDerivingKey` and
    `CommitIvkRandomness` and their `inner` methods,
    `FullViewingKey::{nk, rivk}`,
    `DiversifiedTransmissionKey::{inner, to_bytes}`,
    `orchard::note::NoteCommitTrapdoor` and `NoteCommitTrapdoor::inner`,
    `Rho::{from_nf_old, into_inner}`, `RandomSeed::{psi, rcm}`,
    `Note::{new, dummy}`, `NoteCommitment::inner`,
    `ExtractedNoteCommitment::inner`, `Nullifier::{from_inner, inner}`,
    `NonIdentityPallasPoint` and `NonIdentityPallasPoint::from_bytes`,
    `MerklePath::dummy`, and `MerkleHashOrchard::inner`.

## [0.13.0] - 2026-04-22

### Added
- `orchard::primitives::redpallas::VerificationKey<T>::is_identity`, which
  returns `true` if the verification key is the identity `pallas::Point`.
- `orchard::primitives::redpallas::testing::arb_valid_spendauth_keypair`
  (under the `test-dependencies` feature): a uniformly-distributed valid
  `(rsk, rk)` key pair with non-identity `rk`.

### Changed
- MSRV is now 1.85.1
- Migrated from yanked `core2` library to `corez`
- `orchard::pczt::Bundle::extract` now takes its `self` argument by
  reference instead of by value.
- `orchard::zip32::Error` has added variant `MaxDerivationDepth`
- `orchard::Action::from_parts` and `orchard::circuit::Instance::from_parts`
  now return `Option<Self>`, yielding `None` when `rk` is the identity
  `pallas::Point`. Callers that previously treated the return as `Self`
  must now handle the `None` case. This aligns the crate with the
  consensus rule introduced in zcashd v6.12.1 and Zebra 4.3.1 (see
  <https://zodl.com/zcashd-zebra-april-2026-disclosure/> and
  <https://zfnd.org/zebra-4-3-1-critical-security-fixes-dockerized-mining-and-ci-hardening/>);
  the Zcash protocol specification will be updated to match.
- `orchard::pczt::TxExtractorError` has added variant `IdentityRk`.
- `orchard::pczt::ProverError` has added variant `IdentityRk`.

## [0.12.0] - 2025-12-05

### Added
- `orchard::pczt::Action::apply_signature`
- `orchard::value::BalanceError`
- `impl std::error::Error` for the following errors:
  - `orchard::pczt`:
    - `IoFinalizerError`
    - `ParseError`
    - `ProverError`
    - `SignerError`
    - `TxExtractorError`
    - `UpdaterError`
    - `VerifyError`
  - `orchard::zip32::Error`

### Changed
- `orchard::builder::BuildError::ValueSum` variant now contains
  `orchard::value::BalanceError`.
- `orchard::pczt::SignerError` has added variants:
  - `InvalidExternalSignature`
- All error enums in this crate are now `#[non_exhaustive]`, to allow future
  error variants to be added without a SemVer break:
  - `orchard::builder`:
    - `BuildError`
    - `SpendError`
  - `orchard::pczt`:
    - `IoFinalizerError`
    - `ParseError`
    - `ProverError`
    - `SignerError`
    - `TxExtractorError`
    - `UpdaterError`
    - `VerifyError`
  - `orchard::zip32::Error`
- `orchard::builder::OutputError` has been changed from a zero-sized struct to
  a `#[non_exhaustive]` enum with (for now) a single variant.

### Removed
- `orchard::value::OverflowError` (use `BalanceError` instead).

## [0.10.2] - 2025-05-08

### Fixed
- Fixes problems in test compilation under `--no-default-features`

## [0.11.0] - 2025-02-20

### Added
- `orchard::pczt::Zip32Derivation::extract_account_index`

### Changed
- MSRV is now 1.70
- Migrated to `nonempty 0.11`, `incrementalmerkletree 0.8`, `shardtree 0.6`, 
  `zcash_spec 0.2`, `zip32 0.2`
- `orchard::builder::Builder::add_output` now takes a `[u8; 512]` for its
  `memo` argument instead of an optional value.

## [0.10.1] - 2024-12-16

### Added
- Support for Partially-Created Zcash Transactions:
  - `orchard::builder::Builder::build_for_pczt`
  - `orchard::note_encryption`:
    - `OrchardDomain::for_pczt_action`
    - `impl ShieldedOutput<OrchardDomain, ENC_CIPHERTEXT_SIZE> for orchard::pczt::Action`
  - `orchard::pczt` module.
- `orchard::bundle::EffectsOnly`
- `orchard::tree::MerklePath::{position, auth_path}`
- `orchard::value`:
  - `Sign`
  - `ValueSum::magnitude_sign`
  - `ValueCommitTrapdoor::to_bytes`
- `impl Clone for orchard::tree::MerklePath`

## [0.10.0] - 2024-10-02

### Changed
- Migrated to `incrementalmerkletree 0.7`.

## [0.9.1] - 2024-08-13

### Changed
- Migrated to `visibility 0.1.1`.

## [0.9.0] - 2024-08-12

### Added
- `orchard::keys::SpendValidatingKey::{from_bytes, to_bytes}` behind the
  `unstable-frost` feature flag. These are temporary APIs exposed for development
  purposes, and will be replaced by type-safe FROST APIs once ZIP 312 key
  generation is specified (https://github.com/zcash/zips/pull/883).

### Changed
- Migrated to `incrementalmerkletree 0.6`.

## [0.8.0] - 2024-03-25

### Added
- `orchard::keys::IncomingViewingKey::prepare`
- `orchard::note::Rho`
- `orchard::action::Action::rho`
- `orchard::note_encryption::CompactAction::rho`
- `orchard::note_encryption::OrchardDomain::for_compact_action`
- Additions under the `test-dependencies` feature flag:
  - `orchard::tree::MerkleHashOrchard::random`
  - `impl Distribution<MerkleHashOrchard> for Standard`

### Changed
- The following methods have their `Nullifier`-typed argument or return value
  now take or return `note::Rho` instead:
  - `orchard::note::RandomSeed::from_bytes`
  - `orchard::note::Note::from_parts`
  - `orchard::note::Note::rho`

### Removed
- `orchard::note_encryption::OrchardDomain::for_nullifier` (use `for_action`
  or `for_compact_action` instead).

## [0.7.1] - 2024-02-29
### Added
- `impl subtle::ConstantTimeEq for orchard::note::Nullifier`
- `orchard::note_encryption`:
  - `CompactAction::cmx`
  - `impl Clone for CompactAction`

## [0.7.0] - 2024-01-26
### Licensing
- The license for this crate is now "MIT OR Apache-2.0". The license
  exception that applied to the Zcash and Zebra projects, other projects
  designed to integrate with Zcash, and certain forks of Zcash, is no longer
  necessary. For clarity, this is intended to be a strict relaxation of the
  previous licensing, i.e. it permits all usage that was previously possible
  with or without use of the license exception.

### Added
- `orchard::builder`:
  - `bundle`
  - `BundleMetadata`
  - `BundleType`
  - `OutputInfo`
- `orchard::bundle::Flags::{ENABLED, SPENDS_DISABLED, OUTPUTS_DISABLED}`
- `orchard::tree::Anchor::empty_tree`

### Changed
- Migrated to the `zip32` crate. The following types have been replaced by the
  equivalent ones in that crate are now re-exported from there:
  - `orchard::keys::{DiversifierIndex, Scope}`
  - `orchard::zip32::ChildIndex`
- `orchard::builder`:
  - `Builder::new` now takes the bundle type to be used in bundle construction,
    instead of taking the flags and anchor separately.
  - `Builder::add_recipient` has been renamed to `add_output` in order to
    clarify than more than one output of a given transaction may be sent to the
    same recipient.
  - `Builder::build` now takes an additional `BundleType` argument that
    specifies how actions should be padded, instead of using hardcoded padding.
    It also now returns a `Result<Option<(Bundle<...>, BundleMetadata)>, ...>`
    instead of a  `Result<Bundle<...>, ...>`.
  - `BuildError` has additional variants:
    - `SpendsDisabled`
    - `OutputsDisabled`
    - `AnchorMismatch`
  - `SpendInfo::new` now returns a `Result<SpendInfo, SpendError>` instead of an
    `Option`.
- `orchard::keys::SpendingKey::from_zip32_seed` now takes a `zip32::AccountId`.

### Removed
- `orchard::bundle::Flags::from_parts`

## [0.6.0] - 2023-09-08
### Changed
- MSRV is now 1.65.0.
- Migrated to `incrementalmerkletree 0.5`.

## [0.5.0] - 2023-06-06
### Changed
- Migrated to `zcash_note_encryption 0.4`, `incrementalmerkletree 0.4`, `bridgetree 0.3`.
  `bridgetree` is now exclusively a test dependency.

## [0.4.0] - 2023-04-11
### Added
- `orchard::builder`:
  - `{SpendInfo::new, InputView, OutputView}`
  - `Builder::{spends, outputs}`
  - `SpendError`
  - `OutputError`
- `orchard::keys`:
  - `PreparedEphemeralPublicKey`
  - `PreparedIncomingViewingKey`
- impls of `memuse::DynamicUsage` for:
  - `orchard::note::Nullifier`
  - `orchard::note_encryption::OrchardDomain`
- impls of `Eq` for:
  - `orchard::zip32::ChildIndex`
  - `orchard::value::ValueSum`

### Changed
- MSRV is now 1.60.0.
- Migrated to `ff 0.13`, `group 0.13`, `pasta_curves 0.5`, `halo2_proofs 0.3`,
  `halo2_gadgets 0.3`, `reddsa 0.5`, `zcash_note_encryption 0.3`.
- `orchard::builder`:
  - `Builder::{add_spend, add_output}` now use concrete error types instead of
    `&'static str`s.
  - `Error` has been renamed to `BuildError` to differentiate from new error
    types.
  - `BuildError` now implements `std::error::Error` and `std::fmt::Display`.

### Fixed
- Several bugs have been fixed that were preventing Orchard bundles from being
  created or verified on 32-bit platforms, or with recent versions of Rust.

## [0.3.0] - 2022-10-19
### Added
- `orchard::Proof::add_to_batch`
- `orchard::address::Address::diversifier`
- `orchard::keys::Diversifier::from_bytes`
- `orchard::note`:
  - `RandomSeed`
  - `Note::{from_parts, rseed}`
- `orchard::circuit::Circuit::from_action_context`

### Changed
- Migrated to `zcash_note_encryption 0.2`.

## [0.2.0] - 2022-06-24
### Added
- `orchard::bundle::BatchValidator`
- `orchard::builder::Builder::value_balance`
- `orchard::note_encryption`:
  - `CompactAction::from_parts`
  - `CompactAction::nullifier`
  - `OrchardDomain::for_nullifier`
- Low-level APIs in `orchard::value` for handling `ValueCommitment`s.
  These are useful in code that constructs proof witnesses itself, but
  note that doing so requires a detailed knowledge of the Zcash protocol
  to avoid privacy and correctness pitfalls.
  - `ValueCommitTrapdoor`
  - `ValueCommitment::derive`

### Changed
- Migrated to `halo2_proofs 0.2`.

## [0.1.0] - 2022-05-10
### Changed
- Migrated to `bitvec 1`, `ff 0.12`, `group 0.12`, `incrementalmerkletree 0.3`,
  `pasta_curves 0.4`, `halo2_proofs 0.1`, `reddsa 0.3`.
- `orchard::bundle`:
  - `Action` has been moved to `orchard::Action`.
  - `Bundle::{try_}authorize` have been renamed to
    `Bundle::{try_}map_authorization`.
  - `Flags::from_byte` now returns `Option<Flags>` instead of
    `io::Result<Flags>`.
- `impl Sub for orchard::value::NoteValue` now returns `ValueSum` instead of
  `Option<ValueSum>`, as the result is guaranteed to be within the valid range
  of `ValueSum`.

## [0.1.0-beta.3] - 2022-04-06
### Added
- `orchard::keys`:
  - `Scope` enum, for distinguishing external and internal scopes for viewing
    keys and addresses.
  - `FullViewingKey::{to_ivk, to_ovk}`, which each take a `Scope` argument.
  - `FullViewingKey::scope_for_address`

### Changed
- Migrated to `halo2_proofs 0.1.0-beta.4`, `incrementalmerkletree 0.3.0-beta.2`.
- `orchard::builder`:
  - `Builder::add_spend` now requires that the `FullViewingKey` matches the
    given `Note`, and handles any scoping itself (instead of requiring the
    caller to pass the `FullViewingKey` for the correct scope).
- `orchard::keys`:
  - `FullViewingKey::{address, address_at}` now each take a `Scope` argument.

### Removed
- `orchard::keys`:
  - `FullViewingKey::derive_internal`
  - `impl From<&FullViewingKey> for IncomingViewingKey` (use
    `FullViewingKey::to_ivk` instead).
  - `impl From<&FullViewingKey> for OutgoingViewingKey` (use
    `FullViewingKey::to_ovk` instead).

## [0.1.0-beta.2] - 2022-03-22
### Added
- `orchard::keys`:
  - `DiversifierIndex::to_bytes`
  - `FullViewingKey::derive_internal`
  - `IncomingViewingKey::diversifier_index`
- `orchard::note`:
  - `impl PartialEq, Eq, PartialOrd, Ord for Nullifier`
- `orchard::primitives::redpallas::VerificationKey::verify`
- `orchard::tree`:
  - `MerklePath::from_parts`
  - `impl PartialEq, Eq, PartialOrd, Ord for MerkleHashOrchard`
- `impl From<orchard::bundle::BundleCommitment> for [u8; 32]`
- `Clone` impls for various structs:
  - `orchard::Bundle::{recover_outputs_with_ovks, recover_output_with_ovk}`
  - `orchard::builder`:
    - `InProgress, SigningMetadata, SigningParts, Unauthorized, Unproven`
  - `orchard::circuit::Circuit`
  - `orchard::keys::SpendAuthorizingKey`
  - `orchard::primitives::redpallas::SigningKey`

### Changed
- MSRV is now 1.56.1.
- Bumped dependencies to `pasta_curves 0.3`, `halo2_proofs 0.1.0-beta.3`.
- The following methods now have an additional `rng: impl RngCore` argument:
  - `orchard::builder::Bundle::create_proof`
  - `orchard::builder::InProgress::create_proof`
  - `orchard::circuit::Proof::create`
- `orchard::Bundle::commitment` now requires the bound `V: Copy + Into<i64>`
  instead of `i64: From<&'a V>`.
- `orchard::Bundle::binding_validating_key` now requires the bound
  `V: Into<i64>` instead of `V: Into<ValueSum>`.
- `orchard::builder::InProgressSignatures` and `orchard::bundle::Authorization`
  now have `Debug` bounds on themselves and their associated types.

### Removed
- `orchard::bundle`:
  - `commitments::hash_bundle_txid_data` (use `Bundle::commitment` instead).
  - `commitments::hash_bundle_auth_data` (use `Bundle::authorizing_commitment`
    instead).
- `orchard::keys`:
  - `FullViewingKey::default_address`
  - `IncomingViewingKey::default_address`
  - `DiversifierKey` (use the APIs on `FullViewingKey` and `IncomingViewingKey`
    instead).
- `impl std::hash::Hash for orchard::tree::MerkleHashOrchard` (use `BTreeMap`
  instead of `HashMap`).
- `orchard::value::ValueSum::from_raw`

## [0.1.0-beta.1] - 2021-12-17
Initial release!
