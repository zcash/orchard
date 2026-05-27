---
sidebar_position: 1
title: Crate and Module Map
description: The module graph of zcash/orchard, what each module owns, and how a transaction flows through them.
---

# Crate and Module Map

## 1. Why This Chapter Exists

The `orchard` crate is small (about eight thousand lines of Rust),
but it touches every layer of a shielded protocol: curves,
proof system, key tree, encryption, Merkle tree, builder API, and
PCZT. A contributor must know which file owns which concern before
they can change anything; otherwise they will either duplicate
work in the wrong module or break a cross-module invariant. By the
end of the chapter the reader will be able to point at a feature
("a new flag in the Action description", "a different note
plaintext encoding") and name the file they would touch.

## 2. Definitions

The crate is a single Rust library (no workspace). Its public
surface is declared by `pub use` and `pub mod` in
[`src/lib.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs).
The build is controlled by Cargo features in
[`Cargo.toml`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/Cargo.toml):

- `default = ["circuit", "multicore", "std"]`. The default build
  includes the Halo 2 Action circuit, multicore IPA, and `std`.
- `circuit` activates the Halo 2 prover and verifier and adds
  dependencies on `halo2_proofs` and `halo2_gadgets`.
- `unstable-voting-circuits` widens visibility of internal
  modules (`spec`, `constants`) for the voting-circuit work.
- `unstable-frost` reserves API surface for FROST multi-party
  spend authorisation.
- `dev-graph` enables circuit visualisation via the `plotters`
  crate.

The crate is `#![no_std]` (with `alloc`) and `#![forbid(unsafe_code)]`.

### Module Tiers

The modules form four tiers. Lower tiers are not allowed to depend
on higher tiers.

- **Tier 0 (primitives)**: `spec`, `constants`, `primitives`.
- **Tier 1 (data structures)**: `address`, `note`, `value`, `tree`.
- **Tier 2 (composition)**: `keys`, `zip32`, `note_encryption`,
  `action`, `circuit`.
- **Tier 3 (transaction)**: `builder`, `bundle`, `pczt`.

## 3. The Code

### 3.1 Public Re-Exports

```rust reference title="src/lib.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs#L54-L60
```

Seven names land at the crate root in this `pub use` block:
`Action`, `Address`, `Bundle`, the re-exported Merkle depth
constant, three bit-length constants, `Note`, and `Anchor`. The
`Proof` opaque newtype is defined just below this block in the
same file.

### 3.2 Tier 0: Primitives

- [`src/spec.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/spec.rs)
  hosts the small spec-faithful helpers: `to_base`, `to_scalar`,
  `diversify_hash`, `extract_p`, `prf_expand`, `commit_ivk`,
  `ka_orchard`. They are pure functions of bytes.
- [`src/constants.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/constants.rs)
  re-exports submodule constants and declares the canonical
  bit-length and depth constants.
- [`src/primitives.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/primitives.rs)
  exposes only `redpallas`; see
  [Chapter 14 (RedPallas)](./14-redpallas.md).

### 3.3 Tier 1: Data Structures

- [`src/address.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/address.rs):
  the `(d, pk_d)` payment address type.
- [`src/note.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs)
  with `commitment.rs` and `nullifier.rs`: the `Note` type, its
  commitment, and its nullifier.
- [`src/value.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/value.rs):
  `NoteValue`, `ValueSum`, `ValueCommitment`.
- [`src/tree.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/tree.rs):
  the Sinsemilla-based incremental Merkle tree wrapper.

### 3.4 Tier 2: Composition

- [`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs)
  derives every key in the Orchard key hierarchy.
- [`src/zip32.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/zip32.rs)
  implements ZIP 32 hardened derivation on top.
- [`src/note_encryption.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note_encryption.rs)
  plugs the Orchard-specific KDF and encoding into
  `zcash_note_encryption`.
- [`src/action.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/action.rs)
  is the in-memory representation of one Action description.
- [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)
  is the Halo 2 Action circuit, with three Orchard-specific chips
  under `circuit/`.

### 3.5 Tier 3: Transaction

- [`src/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs):
  the wallet-side API that assembles a list of inputs and outputs
  into a list of Actions and a witness.
- [`src/bundle.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs)
  with `bundle/batch.rs` and `bundle/commitments.rs`: the
  `Bundle` typestate and batch verification.
- [`src/pczt.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/pczt.rs)
  with submodules `parse`, `prover`, `signer`, `updater`,
  `io_finalizer`, `tx_extractor`, `verify`: the partially
  constructed transaction support for split-role signing.

### 3.6 The Transaction Lifecycle

A wallet building one Action follows the tiers from the bottom up:

1. Derive an
   [`ExtendedSpendingKey`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/zip32.rs)
   from a seed (Tier 2).
2. Locate a spendable
   [`Note`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs)
   and its
   [`MerklePath`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/tree.rs)
   (Tier 1).
3. Open a
   [`Builder`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs)
   and call `add_spend` / `add_output` (Tier 3).
4. `Builder::build` constructs an
   `Unauthorized` [`Bundle`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs)
   containing all
   [`Action`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/action.rs)
   descriptions and the witness.
5. `Bundle::create_proof` runs the
   [`circuit`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)
   prover.
6. `Bundle::apply_signatures` produces the per-Action
   [`redpallas`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/primitives/redpallas.rs)
   spend-authorising signatures and the binding signature.

## 4. Failure Modes

- **Cross-tier leakage**. The `spec` module is supposed to be pure
  functions of bytes; adding a dependency on `keys` or `note`
  turns small spec-faithful primitives into a circular import.
  Watch for new `use crate::keys` lines in
  [`src/spec.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/spec.rs).
- **Feature gating on `circuit`**. The Halo 2 prover and verifier
  pull in `halo2_proofs`, which doubles the compilation time and
  the binary size. A PR that unconditionally references
  `halo2_proofs` from a non-`circuit` file breaks the
  `--no-default-features` build that downstream `no_std` users
  rely on. The CI matrix runs
  [`build-nostd`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/.github/workflows/ci.yml)
  for two targets to catch this.
- **Forgetting the `unstable-voting-circuits` widening**. The
  module re-exports in
  [`src/lib.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs)
  are double-declared with `#[cfg(...)]` to expose private
  submodules under the voting feature. New modules must follow
  the same pattern or they will be unreachable in voting builds.

## 5. Spec Pointers

- [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf):
  Section 5.4 is the normative description of Orchard. Read
  Section 5.4 alongside `src/lib.rs` to confirm which spec
  concept lives in which module.
- [ZIP 224](https://zips.z.cash/zip-0224): the Orchard activation
  ZIP. It defines what a "bundle" is from the consensus perspective.
- [Orchard Book](https://zcash.github.io/orchard/): the mdBook
  shipped alongside the crate.

## 6. Exercises

1. Open
   [`src/lib.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs)
   and list every `#[cfg(...)]`-gated `pub mod`. For each, identify
   the feature that toggles it and the spec or audit reason for
   the gate.
2. Run `cargo doc --no-deps --open` against the crate at the pin.
   In the rendered docs, click through to a public type in each
   tier (e.g. `Bundle`, `Note`, `Address`). Confirm that the type
   only references types of the same or lower tier.
3. **Code task**. Add a one-line `tracing` log to a constructor in
   `src/note.rs` (use the `log` crate already used by
   `halo2_proofs`). Run `cargo test note::tests::` and confirm
   the log appears. Revert the change. This forces the reader to
   build the crate end to end.

## 7. Further Reading

- The previous chapter on the upstream mdBook
  ([Orchard Book](https://zcash.github.io/orchard/concepts/preliminaries.html))
  covers the same architecture at a higher level.
- Compare with
  [`zcash/sapling-crypto`](https://github.com/zcash/sapling-crypto)
  to see how the same architecture predates Orchard.
