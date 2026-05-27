---
sidebar_position: 999
title: Discovery Notes
description: Working notes from the discovery phase that produced this course.
---

# Discovery Notes

These notes are the ground truth for chapter selection and the
commit pin. They are kept in the published site for transparency:
if a chapter contains a claim that contradicts these notes, this
file wins until the chapter is corrected.

## 1. Pin

All GitHub links in this course are pinned to the upstream tag
`0.13.1`, commit
[`f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669).

When a chapter says "see `src/foo.rs:L10-L20`", interpret it as
`https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/foo.rs#L10-L20`.

The pin is the most recent crates.io release at the time of writing.
The `main` branch has two post-release refactors on
`OrchardFixedBases` that are not yet released; chapters note them
where relevant but do not link to them.

## 2. Top-Level Files

- [`README.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/README.md):
  one paragraph and a link to the Orchard Book.
- [`CHANGELOG.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/CHANGELOG.md):
  per-release breaking and non-breaking changes.
- [`COPYING.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/COPYING.md):
  dual MIT / Apache-2.0 licensing.
- No `CONTRIBUTING.md`, no `AGENTS.md`, no `CLAUDE.md`. Contribution
  workflow must be inferred from CI and recent PRs.
- [`rust-toolchain.toml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/rust-toolchain.toml):
  pinned to Rust `1.85.1` with `clippy` and `rustfmt` components.
- [`katex-header.html`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/katex-header.html):
  the KaTeX stylesheet header injected into `cargo doc` output.

## 3. Build Manifest

- [`Cargo.toml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/Cargo.toml):
  one crate (no workspace). MSRV `1.85.1`. License `MIT OR Apache-2.0`.
- Default features: `circuit`, `multicore`, `std`.
- Other features: `unstable-frost`, `unstable-voting-circuits`,
  `dev-graph` (pulls in `image` and `plotters`), `test-dependencies`
  (enables `proptest` and `rand/std`).
- Critical dependencies (versions at pin):
  `pasta_curves 0.5`, `halo2_proofs` via `halo2_gadgets` (both from
  `zcash/halo2`), `sinsemilla 0.1`, `halo2_poseidon 0.1`,
  `reddsa 0.5`, `incrementalmerkletree 0.8.1`,
  `zcash_note_encryption 0.4`, `zcash_spec 0.2.1`, `zip32 0.2.0`.

## 4. Crate Map

Top-level modules under `src/`, with one-line purpose. Curly brace
listed entries are submodules.

```
lib.rs               // public re-exports, no_std declarations
action.rs            // a single Orchard Action description
address.rs           // shielded payment addresses
builder.rs           // wallet API for assembling bundles
bundle.rs            // Bundle type, typestate Unauthorized -> Authorized
bundle/batch.rs      // batched proof + signature verification
bundle/commitments.rs// SIGHASH / commitments for the bundle layer
circuit.rs           // the Halo 2 Action circuit (feature `circuit`)
circuit/gadget.rs    // shared chips: addition, free advice
circuit/commit_ivk.rs// Commit^ivk chip
circuit/note_commit.rs// NoteCommit chip
circuit/gadget/      // additional chips reused by the above
constants.rs         // hash personalisations, depths, bit lengths
constants/sinsemilla.rs    // Sinsemilla constants and helpers
constants/fixed_bases.rs   // fixed-base scalar multiplication tables
constants/fixed_bases/     // per-base precomputed window tables
constants/load.rs    // deserialisation of the precomputed tables
constants/util.rs    // small helpers (i2lebsp, etc.)
keys.rs              // ZIP 32 key tree: ask, nk, ivk, ovk, dk, ...
note.rs              // Note, Rho, RandomSeed
note/commitment.rs   // NoteCommitment, ExtractedNoteCommitment
note/nullifier.rs    // Nullifier derivation
note_encryption.rs   // ZIP 212 / ZIP 316 note encryption
pczt.rs              // partially constructed transactions (PCZT)
pczt/parse.rs        // parse PCZT bytes
pczt/prover.rs       // PCZT role: prover
pczt/signer.rs       // PCZT role: spend authorisation signer
pczt/updater.rs      // PCZT role: updater
pczt/io_finalizer.rs // PCZT IO finalizer role
pczt/tx_extractor.rs // PCZT bundle extractor
pczt/verify.rs       // PCZT verification helpers
primitives.rs        // exposes redpallas only
primitives/redpallas.rs // RedDSA over Pallas: signing API
spec.rs              // ToBase / ToScalar / DiversifyHash / KdfOrchard
tree.rs              // Sinsemilla-based incremental Merkle tree
value.rs             // NoteValue, ValueSum, ValueCommitment
zip32.rs             // ZIP 32 hardened derivation for Orchard
test_vectors.rs      // (cfg test) JSON test-vector wiring
```

The `circuit_description` directory holds a pinned textual snapshot
of the circuit shape; the `circuit_proof_test_case.bin` file is a
pinned proof used by the circuit tests.

## 5. Public API Entry Points

The crate's surface is declared in
[`src/lib.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs#L51-L60):
`Action`, `Address`, `Bundle`, `MERKLE_DEPTH_ORCHARD` (re-exported
as `NOTE_COMMITMENT_TREE_DEPTH`), `L_ORCHARD_BASE`,
`L_ORCHARD_SCALAR`, `L_VALUE`, `Note`, `Anchor`, and the `Proof`
opaque newtype.

Public modules: `builder`, `bundle`, `circuit` (feature-gated),
`keys`, `note`, `note_encryption`, `pczt`, `primitives`, `tree`,
`value`, `zip32`. With `unstable-voting-circuits`, the private
`spec` and `constants` modules become public.

No binaries are produced.

## 6. Tests, Benchmarks, Fuzz

- Integration tests: a single file
  [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
  that exercises the full happy path (build, prove, sign, verify,
  recover, chain).
- Unit tests inline in every `src/` file.
- JSON test vectors loaded via
  [`src/test_vectors.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/test_vectors.rs)
  from the
  [`zcash-test-vectors`](https://github.com/zcash-hackworks/zcash-test-vectors)
  repository.
- Benchmarks under `benches/`:
  [`circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/circuit.rs)
  (proving / verifying the Action circuit),
  [`note_decryption.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/note_decryption.rs),
  [`small.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/small.rs).
- Property-based regressions kept under `proptest-regressions/`.
- No `cargo-fuzz` harness in-tree.

## 7. CI Graph

GitHub Actions workflows at the pin:

- [`ci.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/ci.yml):
  cargo test on ubuntu/windows/macos, latest-deps build,
  `no_std` build for `wasm32-wasip1` and `thumbv7em-none-eabihf`,
  doctests, MSRV check.
- [`lints-stable.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/lints-stable.yml):
  `cargo fmt --check` + `cargo clippy -- -D warnings`.
- [`lints-beta.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/lints-beta.yml):
  clippy on beta toolchain (non-blocking).
- [`bench.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/bench.yml):
  build benchmarks (no continuous tracking).
- [`book.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/book.yml):
  publishes the Orchard mdBook to
  [zcash.github.io/orchard](https://zcash.github.io/orchard/).

Required status checks for a green PR: `cargo test` on all three
operating systems, the `no_std` builds, the stable lints, the
codegen check.

## 8. Release / Versioning

- Tag-based releases. Recent tags: `0.13.1`, `0.13.0`, `0.12.0`,
  `0.11.0`, ... in `git tag --sort=-creatordate`.
- Releases are cut by Electric Coin Co. maintainers from `main`.
- [`CHANGELOG.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/CHANGELOG.md)
  uses [Keep a Changelog](https://keepachangelog.com/) categories.

## 9. Hot Files (Last 6 Months)

Top files by commit count on `main` since the pin minus six months:

| Count | File                                                                         |
| ----- | ---------------------------------------------------------------------------- |
| 19    | `CHANGELOG.md`                                                               |
| 13    | `Cargo.toml`                                                                 |
|  9    | [`src/value.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/value.rs) |
|  9    | `Cargo.lock`                                                                 |
|  6    | [`src/constants/fixed_bases.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/constants/fixed_bases.rs) |
|  5    | [`src/lib.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs) |
|  5    | [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs) |
|  4    | [`src/pczt/tx_extractor.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/pczt/tx_extractor.rs) |
|  4    | [`src/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs) |
|  4    | [`src/address.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/address.rs) |
|  3    | [`src/zip32.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/zip32.rs) |

Recent themes: `valargroup` fork landing PCZT and FROST scaffolding,
EC Co. refactors on `OrchardFixedBases` and `NoteValue` constants,
release administrivia.

## 10. Open Issues Worth Knowing About

Sampled from `gh issue list --state open --limit 20`:

- [#497](https://github.com/zcash/orchard/issues/497): refactor
  `BatchValidator::add_bundle` to return a `Result<(), Error>`.
- [#491](https://github.com/zcash/orchard/issues/491): documentation
  request about `cargo test --package orchard` (superseded by the
  reverted crate split).
- [#467](https://github.com/zcash/orchard/issues/467): make
  `NoteCommitment` part of the public API.
- [#464](https://github.com/zcash/orchard/issues/464): fix panic on
  `ExtendedSpendingKey` derivation at depth 256+.
- [#463](https://github.com/zcash/orchard/issues/463): upgrade to
  `rand 0.9`.
- [#459](https://github.com/zcash/orchard/issues/459): make the
  `circuit` feature build under `no_std`.
- [#431](https://github.com/zcash/orchard/issues/431): construct a
  `FullViewingKey` from a `SpendValidatingKey`.
- [#347](https://github.com/zcash/orchard/issues/347): add a public
  `Circuit` constructor.
- [#191](https://github.com/zcash/orchard/issues/191): test vectors
  for [ZIP 32](https://zips.z.cash/zip-0032) derivation in
  `src/zip32.rs`.
- [#125](https://github.com/zcash/orchard/issues/125) and
  [#84](https://github.com/zcash/orchard/issues/84): name and prove
  polynomial constraints; correctness proofs for scalar
  multiplication and range checks.

No labelled "good first issue" exists at the pin. The two safest
beginner targets are issues #463 (mechanical dependency bump,
constrained by upstream rand release) and #191 (purely additive
test vectors). Both are referenced in chapter
[Build, Test, and Contribute](./02-build-test-contribute.md).

## 11. External Canonical References

These are cited repeatedly across chapters.

- The
  [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
  (sections 4 and 5).
- The [ZIPs index](https://zips.z.cash/), in particular
  [ZIP 32](https://zips.z.cash/zip-0032),
  [ZIP 212](https://zips.z.cash/zip-0212),
  [ZIP 224](https://zips.z.cash/zip-0224) (Orchard activation),
  [ZIP 244](https://zips.z.cash/zip-0244) (transaction identifiers),
  [ZIP 316](https://zips.z.cash/zip-0316) (Unified Addresses).
- [Halo paper](https://eprint.iacr.org/2019/1021).
- [Halo 2 Book](https://zcash.github.io/halo2/) and
  [`zcash/halo2`](https://github.com/zcash/halo2) source.
- [Pasta curves blog post](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/).
- [Poseidon paper](https://eprint.iacr.org/2019/458).
- Sinsemilla appendix in the Zcash Protocol Specification.
- Public Orchard / Halo 2 audit reports linked from
  [zcash.github.io/halo2/audits.html](https://zcash.github.io/halo2/audits.html).
