---
sidebar_position: 51
title: PR Checklist
description: Local steps a contributor runs before pushing a PR to zcash/orchard.
---

# PR Checklist

The crate does not ship a `CONTRIBUTING.md`. The checklist below
is inferred from CI
([`.github/workflows/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows))
and from recent merged PRs. Run every item locally before
opening the PR.

## 1. Before You Code

- Read the issue thread end to end (if there is one). Confirm
  with a comment that you intend to work on it.
- Check the
  [PR queue](https://github.com/zcash/orchard/pulls) for an
  open PR that already addresses the issue. If one exists, leave
  a constructive review rather than duplicating the work.
- For changes that touch the circuit or any consensus path,
  skim the most recent related discussion in the
  [`zcash/halo2` PR queue](https://github.com/zcash/halo2/pulls)
  too; the upstream cadence sometimes pre-empts crate-level work.

## 2. Build and Test

```bash
cargo build --all-features
cargo test --verbose
```

If the change touches `circuit` code:

```bash
cargo test --release --features circuit
```

(The default debug build of the circuit prover is about ten
times slower than release.)

If the change touches anything outside `circuit`, also run:

```bash
rustup target add wasm32-wasip1
cargo build --release --no-default-features --target wasm32-wasip1
```

## 3. Format and Lint

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo fmt` is idempotent; the lints job in CI rejects any diff.

## 4. Documentation

```bash
cargo doc --no-deps --all-features
```

The build will fail if any
`#![deny(rustdoc::broken_intra_doc_links)]` link is broken.

If the change touches a public type or function:

- Update the docstring.
- If the type appears in the Orchard Book under
  [`book/src/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/book/src),
  update the book entry too.

## 5. Changelog

Edit
[`CHANGELOG.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/CHANGELOG.md)
under the `## [Unreleased]` header. Categories follow
[Keep a Changelog](https://keepachangelog.com/):

- `Added`: new public API.
- `Changed`: backward-compatible behaviour change.
- `Removed` / `Deprecated`: API surface removed or marked
  deprecated.
- `Fixed`: bug fix with no behaviour change beyond the fix.
- `Security`: security-relevant fix.

A one-sentence entry is the standard. Cite the PR number when
known: `(#496)`.

## 6. Audits and Consensus

If the change is consensus-relevant (the circuit shape, the
nullifier formula, a key derivation, a serialization format):

- Add a short paragraph in the PR description explaining why
  the change does not require a new audit, or why it does.
- If the pinned proof
  [`src/circuit_proof_test_case.bin`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_proof_test_case.bin)
  must be regenerated, regenerate it locally and include a
  paragraph in the PR explaining how.
- Update
  [`src/circuit_description/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)
  if the circuit shape changes.

## 7. PR Description

Recent merged PRs follow this skeleton:

```
One-line summary.

One or two paragraphs of motivation: what does this change and why.
Cite the issue: "Closes #491."

Then a bulleted list of what changed, by file:

- src/value.rs: added NoteValue::ZERO const for ergonomics.
- src/note.rs: switched call sites from zero() to ZERO.
- CHANGELOG.md: entry under [Unreleased]/Added.
```

Avoid emojis. Avoid marketing language. State the fact, then the
citation.

## 8. After Pushing

- Watch CI. Required checks at the pin:
  `test (ubuntu-latest)`, `test (macos-latest)`,
  `test (windows-latest)`, `build-latest`, `build-nostd`,
  `lints-stable`.
- If a check fails, fix the underlying cause; do not push an
  empty rerun.
- If a maintainer requests changes, respond inline rather than
  force-pushing the whole branch; force-push only at the end of
  a review round.

## 9. Reference Templates

Real merged PRs that are good templates by scope:

- Stylistic:
  [#496 (collapse `OrchardFixedBases`)](https://github.com/zcash/orchard/pull/496).
- Additive API:
  [#495 (`NoteValue::ZERO` const)](https://github.com/zcash/orchard/pull/495).
- Consensus-relevant:
  [#492 (Reject identity rk)](https://github.com/zcash/orchard/pull/492).
