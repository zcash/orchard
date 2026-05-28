---
sidebar_position: 15
title: Dependencies and Constant-Time Discipline
description: External crates that ship cryptographic guarantees on behalf of orchard.
---

# Dependencies and Constant-Time Discipline

## 1. Why This Chapter Exists

The `orchard` crate is small because it delegates. Every
cryptographic primitive lives in an external crate, and changing
those crates changes Orchard. A contributor must know which crate
is responsible for which guarantee, and which dependencies are
load-bearing under `no_std`. After this chapter the reader can
read `Cargo.toml` and assess the impact of any version bump.

## 2. Definitions

### Definition 2.1 (Constant-Time Operation)

An operation `f(x)` is constant-time if its execution time does
not depend on the value of `x` (only on its type). For an elliptic
curve point $P$ and scalar $k$, $[k] P$ is constant-time if the
scalar multiplication runs through the same sequence of curve
operations regardless of $k$.

### Definition 2.2 (Side-Channel-Free)

A higher-level guarantee that adds branch-free comparisons (`subtle`
`ConstantTimeEq`) and conditional selects (`ConditionallySelectable`)
to the constant-time operations.

### Definition 2.3 (`no_std` Compatibility)

A crate is `no_std` if it can compile without the Rust standard
library, relying only on `core` (and optionally `alloc`).
`orchard` is `#![no_std]` and turns off the default features of
every dependency that pulls in `std`.

## 3. The Code

### 3.1 The Dependency Block

```toml reference title="Cargo.toml"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/Cargo.toml#L25-L52
```

The cryptographically critical entries:

- [`pasta_curves`](https://github.com/zcash/pasta_curves) 0.5: the
  Pallas and Vesta curve types
  ([Chapter 3](./03-pasta-curves.md)).
- `halo2_proofs` (via `halo2_gadgets`) from
  [`zcash/halo2`](https://github.com/zcash/halo2): the proof
  system and chip library, behind feature `circuit`.
- [`sinsemilla`](https://github.com/zcash/sinsemilla) 0.1: the
  hash function ([Chapter 6](./06-sinsemilla.md)).
- `halo2_poseidon` 0.1 (re-exported as `poseidon`): the
  permutation ([Chapter 7](./07-poseidon.md)).
- [`reddsa`](https://github.com/ZcashFoundation/reddsa) 0.5: the
  RedDSA implementation ([Chapter 14](./14-redpallas.md)).
- [`zcash_note_encryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption) 0.4:
  the note encryption framework
  ([Chapter 10](./10-note-encryption.md)).
- [`zip32`](https://github.com/zcash/zip32) 0.2.0 and
  [`zcash_spec`](https://github.com/zcash/zcash_spec) 0.2.1:
  shared key-derivation primitives.
- [`incrementalmerkletree`](https://github.com/zcash/incrementalmerkletree)
  0.8.1: the Merkle frontier maintenance.

Side-channel hygiene:

- [`subtle`](https://github.com/dalek-cryptography/subtle) 2.6:
  constant-time `Choice`, `CtOption`, comparisons.
- The crate uses `#![forbid(unsafe_code)]` (see
  [Chapter 1](./01-crate-map.md)).

Symmetric primitives:

- [`aes`](https://github.com/RustCrypto/block-ciphers/tree/master/aes)
  0.8: AES-256 used by FF1 for diversifier index encryption.
- [`blake2b_simd`](https://github.com/oconnor663/blake2b_simd)
  1.x: Blake2b for KDFs.
- [`fpe`](https://github.com/str4d/fpe) 0.6: FF1 over AES-256.

### 3.2 Cargo Features

The crate ships several feature flags. The non-trivial ones at
the pin:

- **`circuit`** (off by default): pulls in `halo2_proofs` and
  `halo2_gadgets` and compiles the Action circuit. A consumer
  that only needs to verify Bundles, parse notes, or work with
  keys can omit this feature and avoid the proof-system
  dependency entirely.
- **`multicore`**: forwards to
  `halo2_proofs/multicore`; turns on parallel proving for the
  Action circuit.
- **`dev-graph`**: pulls in
  `plotters` and `image` to render the circuit layout as a
  graph; useful when reviewing a chip layout change. Not part
  of release builds.
- **`unstable-voting-circuits`** (off by default): re-exports a
  set of otherwise-internal modules and types so that downstream
  voting-circuit projects can re-use Orchard's gadgets. **These
  APIs are explicitly outside the crate's semver guarantees and
  may change in any release.**

  Concretely, enabling the feature widens the visibility of
  `orchard::{constants, spec}`,
  `orchard::circuit::{commit_ivk, commit_ivk::gadgets,`
  `note_commit, note_commit::gadgets, gadget::add_chip}`, and
  `orchard::note::{commitment, nullifier}`, plus a number of
  `Address`, `Note`, `CommitIvkChip`, `NoteCommitChip`, and
  `AddChip` accessors. The
  [CHANGELOG entry](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/CHANGELOG.md)
  ("`unstable-voting-circuits` feature flag") lists the full
  surface at the pin. The feature exists to let projects such as
  voting-circuit research build on top of Orchard's audited
  gadgets without forking the crate; a consumer that ships in
  production and depends on these symbols accepts the risk that
  a future Orchard point release moves or removes them.

  Mechanically the feature uses the
  [`visibility`](https://crates.io/crates/visibility)
  proc-macro: each gated item carries
  `#[cfg_attr(feature = "unstable-voting-circuits", visibility::make(pub))]`,
  which promotes the item from its default `pub(crate)` or
  module-private visibility to `pub` only when the feature is on.

### 3.3 `no_std` Posture

The crate declares `#![no_std]` in
[`src/lib.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs)
and turns off default features of every dependency that would
otherwise pull in `std`. CI runs `build-nostd` for
`wasm32-wasip1` and `thumbv7em-none-eabihf` (see
[Chapter 2](./02-build-test-contribute.md)).

### 3.4 Locking and the Latest-Deps Build

The default CI runs against the committed `Cargo.lock`. A
separate job removes the lock and re-builds with the latest
patch versions of every dependency. PRs that pin a non-trivial
new dependency must check both.

## 4. Failure Modes

- **Latent `std` import**. A dependency adds a `std`-only
  feature default; the `--no-default-features` build breaks on
  the next bump. CI catches this only in the `build-nostd`
  matrix.
- **Subtle version mismatch**. Two transitive dependencies pull
  in different `subtle` major versions; the `Choice` type
  becomes ambiguous. `cargo tree -d` surfaces these.
- **Halo 2 version bump that changes the transcript**. Any
  semver-breaking change in `halo2_proofs` invalidates the
  pinned proof bytes
  [`src/circuit_proof_test_case.bin`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_proof_test_case.bin).
- **Audit unaware dependency replacement**. Replacing a crate
  with a fork (e.g. a faster Sinsemilla) silently shifts the
  audit boundary. The PR description must call this out.

## 5. Spec Pointers

- [RUSTSEC advisory database](https://rustsec.org/) and
  `cargo audit` for known vulnerabilities.
- [Halo 2 audit reports](https://zcash.github.io/halo2/audits.html)
  for the audited boundary of the upstream crates.
- [Constant-time discipline in `dalek-cryptography/subtle`](https://github.com/dalek-cryptography/subtle):
  the canonical reference.

## 6. Exercises

1. Run `cargo tree -e features` against the crate at the pin and
   identify any transitive dependency that pulls in `std`
   indirectly. State the chain.
2. Search `Cargo.toml` for every `default-features = false` and
   explain in one sentence why each disable is necessary.
3. **Code task**. Run `cargo audit` against the workspace.
   Triage any advisory: classify it as
   load-bearing-for-Orchard, transitive-but-test-only, or
   already-fixed-upstream. Open an issue or a PR for one finding
   if appropriate.

## 7. Further Reading

- The
  [`pasta_curves`](https://github.com/zcash/pasta_curves) README,
  which documents the constant-time guarantees.
- [`halo2_proofs` release notes](https://github.com/zcash/halo2/releases)
  for the cadence of breaking changes downstream.
