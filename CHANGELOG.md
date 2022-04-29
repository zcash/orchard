# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- `orchard::bundle`:
  - `Action` has been moved to `orchard::Action`.
  - `Bundle::{try_}authorize` have been renamed to
    `Bundle::{try_}map_authorization`.
  - `Flags::from_byte` now returns `Option<Flags>` instead of
    `io::Result<Flags>`.

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
