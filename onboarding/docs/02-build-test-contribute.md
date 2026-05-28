---
sidebar_position: 2
title: Build, Test, and Contribute
description: The toolchain, the test loop, and what a real first PR against zcash/orchard looks like.
---

# Build, Test, and Contribute

## 1. Why This Chapter Exists

A reader who finishes this chapter can clone the crate, run the
full and focused test suites, run the lints, identify a real open
issue, and produce a PR that survives CI. Without this chapter the
math of the later chapters has no operational outlet. By the end,
the reader has run at least one local invocation of the Action
circuit prover.

## 2. Definitions

### 2.1 Toolchain

The crate pins the Rust toolchain to `1.85.1` with `clippy` and
`rustfmt` in
[`rust-toolchain.toml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/rust-toolchain.toml).
On macOS / Linux, `rustup` will install the pinned toolchain
automatically the first time `cargo` is invoked.

### 2.2 Feature Matrix

Compilation modes that CI exercises are listed in
[`.github/workflows/ci.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/ci.yml).
At the pin:

- `cargo test` (default features).
- `cargo build --all-features` against latest dependencies (no
  `Cargo.lock`).
- `cargo build --release --no-default-features` for two `no_std`
  targets: `wasm32-wasip1` and `thumbv7em-none-eabihf`.
- Doctests through `cargo test --doc`.

### 2.3 Linting Pipeline

- [`lints-stable.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/lints-stable.yml):
  `cargo fmt --check` and `cargo clippy --all-targets
--all-features -- -D warnings`. The clippy step treats all
  warnings as errors.
- [`lints-beta.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/lints-beta.yml):
  the same against the beta toolchain, advisory only.

## 3. The Code

Every command in this section lives in a shell script under
[`onboarding/scripts/`](https://github.com/dannywillems/orchard/tree/onboarding/onboarding/scripts);
the blocks below are imported live from those files so the page
and the runnable scripts cannot drift.

### 3.1 First Build

```bash reference title="onboarding/scripts/first-build.sh"
https://github.com/dannywillems/orchard/blob/onboarding/onboarding/scripts/first-build.sh#L7-L10
```

The first build downloads `halo2_proofs`, `pasta_curves`, and the
chip libraries; expect about three to five minutes on a warm
machine. Subsequent builds are incremental.

### 3.2 Test Loop

Run the full suite as CI does:

```bash reference title="onboarding/scripts/full-test.sh"
https://github.com/dannywillems/orchard/blob/onboarding/onboarding/scripts/full-test.sh#L7-L7
```

This exercises:

- Unit tests inline in every `src/*.rs` (most numerous in
  [`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs),
  [`src/note.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs),
  and
  [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)).
- The integration test
  [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs),
  which builds and chains real bundles.
- JSON test vectors wired through
  [`src/test_vectors.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/test_vectors.rs).

Run a focused test (faster iteration during development):

```bash reference title="onboarding/scripts/focused-test.sh"
https://github.com/dannywillems/orchard/blob/onboarding/onboarding/scripts/focused-test.sh#L7-L9
```

The slowest test at the pin is the Halo 2 prover round-trip in
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs);
on a 2024 MacBook Pro it takes about 40 seconds. Use
`--release` for proving benchmarks; debug builds are about ten
times slower in the proof system itself.

### 3.3 Format and Lint

```bash reference title="onboarding/scripts/format-and-lint.sh"
https://github.com/dannywillems/orchard/blob/onboarding/onboarding/scripts/format-and-lint.sh#L7-L8
```

The format step is idempotent. Clippy uses the workspace default
configuration; the crate does not ship a `clippy.toml` at the pin.

### 3.4 The `no_std` Build

```bash reference title="onboarding/scripts/no-std-build.sh"
https://github.com/dannywillems/orchard/blob/onboarding/onboarding/scripts/no-std-build.sh#L7-L8
```

A new dependency on `std`-only APIs in any non-test file will
break this; CI does the same for `thumbv7em-none-eabihf`.

### 3.5 Benchmarks

```bash reference title="onboarding/scripts/benchmarks.sh"
https://github.com/dannywillems/orchard/blob/onboarding/onboarding/scripts/benchmarks.sh#L6-L8
```

Source files:
[`benches/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/circuit.rs),
[`benches/note_decryption.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/note_decryption.rs),
[`benches/small.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches/small.rs).
The `circuit` bench is the only one that exercises the proof
system end to end; budget about a minute per run.

### 3.6 Hot Files

The five files most often touched in the last six months on
`main`, with the kind of change that lands there:

| File                                                                                                                                          | Typical change                                             |
| --------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| [`src/value.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/value.rs)                                 | API ergonomics on `NoteValue` (constants, `Default` impls) |
| [`src/constants/fixed_bases.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/constants/fixed_bases.rs) | Fixed-base refactors and visibility tightening             |
| [`src/lib.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs)                                     | Re-exports, feature gating, doc comments                   |
| [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)                             | Visibility, accessor functions, internal renames           |
| [`src/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs)                             | Builder API tweaks, dummy Action generation                |

See [Discovery Notes](./discovery.md) for the rest of the list and
the methodology.

### 3.7 A Real First PR

Read three recent merged PRs end to end before writing your own:

- [#496 (Collapse `OrchardFixedBases` to a unit struct)](https://github.com/zcash/orchard/pull/496):
  a minimal refactor; good template for stylistic-only changes.
- [#495 (`NoteValue::ZERO` const)](https://github.com/zcash/orchard/pull/495):
  one-line API addition with a test; good template for additive
  PRs.
- [#492 (Reject identity rk in constructors)](https://github.com/zcash/orchard/pull/492):
  consensus-relevant change with tests and a CHANGELOG entry;
  good template for a defensive code change.

Candidate first PRs from
[the issue tracker](https://github.com/zcash/orchard/issues):

- [#191 (Test vectors for ZIP 32 derivation)](https://github.com/zcash/orchard/issues/191):
  purely additive, exercises
  [`src/zip32.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/zip32.rs);
  see [ZIP 32](https://zips.z.cash/zip-0032).
- [#467 (Make `NoteCommitment` public)](https://github.com/zcash/orchard/issues/467):
  one-line `pub` change plus a doc comment; needs a deliberate
  argument about what becomes stable.
- [#463 (Update to `rand 0.9`)](https://github.com/zcash/orchard/issues/463):
  mechanical dependency bump, blocked on upstream consensus.

### 3.8 Commit and Changelog Conventions

The repository's
[`CHANGELOG.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/CHANGELOG.md)
uses Keep-a-Changelog categories (`Added`, `Changed`, `Fixed`,
etc.) under the next-release header `## [Unreleased]`. Every PR
that changes behaviour must update this file. The PR template (in
`.github/`) is empty at the pin; PR description style is "one
paragraph of motivation, then a bulleted list of what changed".

## 4. Failure Modes

- **CI green locally, red on the runner**. The `--all-features`
  build with no `Cargo.lock` is the failure mode for new
  dependencies whose patch versions diverge from the locked set.
  Run `rm Cargo.lock && cargo build --all-features` before push.
- **`no_std` regressions**. A `use std::` snuck into a non-test
  module silently passes the default `cargo test` and fails only
  in the `build-nostd` matrix. Search for `use std::` in any
  patch before opening the PR.
- **Format-induced reviews**. The `cargo fmt --check` step in
  [`lints-stable.yml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/lints-stable.yml)
  blocks merge. A `pre-push` hook running `cargo fmt && cargo
clippy --all-targets --all-features -- -D warnings` removes
  the round trip.
- **Stale audit pin**. If a PR touches the circuit shape, the
  pinned
  [`src/circuit_proof_test_case.bin`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_proof_test_case.bin)
  and
  [`src/circuit_description/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)
  must be regenerated and re-reviewed. The reviewer will explicitly
  ask for a regenerated proof; do not be surprised.

## 5. Spec Pointers

- [`README.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/README.md):
  the contribution licence policy.
- [`CHANGELOG.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/CHANGELOG.md):
  the only required edit on most PRs.
- [Halo 2 audit reports](https://zcash.github.io/halo2/audits.html):
  the historical reason for several lint and test rules in CI.

## 6. Exercises

1. **Code task**. Clone the crate at the pin and run
   `cargo test --lib note::nullifier`. Time the run. Then run
   `cargo test --release --lib note::nullifier` and compare. State
   the speed-up in one sentence and explain it.
2. **Code task**. Comment out one line of
   [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
   so that the produced bundle has a wrong value commitment. Run
   the test and read the failure message. Identify which assertion
   inside `Bundle::verify_proof` caught the corruption.
3. **Code task**. Pick one of
   [#191](https://github.com/zcash/orchard/issues/191) or
   [#467](https://github.com/zcash/orchard/issues/467) from the
   issue tracker. Read the existing thread, write a one-paragraph
   plan of attack, and skim the file(s) you would touch. Decide
   whether you would open this PR before reading Chapters 3 to 14.
4. **Reading task**. Read
   [#496](https://github.com/zcash/orchard/pull/496) end to end
   (description, all commits, all review comments). Identify which
   sentence in the review prompted the maintainer to approve.

## 7. Further Reading

- The Rust API
  [guidelines](https://rust-lang.github.io/api-guidelines/) inform
  the review style for additive API changes.
- The
  [`zcash/halo2`](https://github.com/zcash/halo2)
  contribution guide describes the parallel review process for
  changes to the proof system.
