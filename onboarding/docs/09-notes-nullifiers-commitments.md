---
sidebar_position: 9
title: Notes, Nullifiers, and Commitments
description: The Note type, its commitment, and its nullifier derivation.
---

# Notes, Nullifiers, and Commitments

## 1. Why This Chapter Exists

A note is the Orchard analogue of a UTXO. Two cryptographic
objects make it usable: the note commitment (deposited into the
tree) and the nullifier (revealed on spend). Both must be
deterministic from the note and the spender's nullifier-deriving
key, and both must be tightly bound to prevent double-spend and
forgery. After this chapter the reader can locate every
contributor-relevant constant of the note encoding and trace
$\rho$ from its origin in the previous spend.

## 2. Definitions

### Definition 2.1 (Note)

A note is the tuple

$$
n = (d,\, \mathsf{pk_d},\, v,\, \rho,\, \psi,\, \mathsf{rcm}),
$$

with

- $d \in \{0, 1\}^{88}$: diversifier;
- $\mathsf{pk_d} \in E_p$: diversified transmission key;
- $v \in [0, 2^{64})$: value;
- $\rho \in \mathbb{F}_p$: nullifier seed (derived from the
  previous note's nullifier);
- $\psi \in \mathbb{F}_p$: deterministic randomness derived from
  $\mathsf{rseed}$;
- $\mathsf{rcm} \in \mathbb{F}_q$: commitment trapdoor (also
  derived from $\mathsf{rseed}$).

### Definition 2.2 (Note Commitment)

$$
\mathsf{cm} = \mathsf{NoteCommit}_{\mathsf{rcm}}\big(g_d,\, \mathsf{pk_d},\, v,\, \rho,\, \psi\big),
$$

a Sinsemilla commit over the canonical encoding. The
$x$-coordinate extraction
$\mathsf{cm}^\star = \mathsf{Extract}_{\mathbb{P}}(\mathsf{cm})$
is the value inserted into the Merkle tree.

### Definition 2.3 (Nullifier)

$$
\mathsf{nf} = \mathsf{Extract}_{\mathbb{P}}\Big(
\big[\mathsf{PRF}^{\mathsf{nfOrchard}}_{\mathsf{nk}}(\rho)\big]\, \mathcal{K} + \psi + \mathsf{cm}\Big),
$$

where $\mathcal{K} \in E_p$ is the nullifier base.

### Invariant 2.4 (Rho Chaining)

A note's $\rho$ is the nullifier of the input note that funded it.
The first note in a transaction chain seeds $\rho$ from the spent
note's nullifier; subsequent notes do the same. This binds note
creation to a specific spend and prevents two distinct notes from
sharing a nullifier without one of them being a forgery.

## 3. The Code

### 3.1 The `Note` Type

```rust reference title="src/note.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs#L1-L40
```

Fields are private; constructors are
`Note::new` (from random) and `Note::from_parts` (with explicit
trapdoor). Both return `CtOption<Note>` because deserialisation
of $g_d$ may fail.

### 3.2 Note Commitment

[`src/note/commitment.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/commitment.rs)
encodes the note canonically into a bit string and calls
`SinsemillaCommit` under the Orchard note commitment domain.

### 3.3 Nullifier

[`src/note/nullifier.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/nullifier.rs)
implements the formula of Definition 2.3 step by step. The
in-circuit version is in
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs).

### 3.4 The `Rho` Newtype

```rust reference title="src/note.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs#L37-L42
```

`Rho` wraps a `pallas::Base` and exposes `from_nf` so that
$\rho$ for a new note is always created from a previous
nullifier; the constructor that takes raw bytes is
`pub(crate)` only.

## 4. Failure Modes

- **Forged $\rho$**. Allowing $\rho$ to be sampled freely instead
  of being chained to a spent nullifier breaks Invariant 2.4 and
  enables double-spends of distinct notes.
- **Non-canonical encoding**. The fixed bit-length encoding of
  $v$, $\rho$, $\psi$ is enforced by range checks in
  [`src/circuit/note_commit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/note_commit.rs).
  Missing one range check is the textbook unintended ambiguity
  bug.
- **PRF identity collision**. If
  $\mathsf{PRF}^{\mathsf{nfOrchard}}_{\mathsf{nk}}(\rho)$ ever
  returns zero, the resulting nullifier degenerates to
  $\mathsf{Extract}_{\mathbb{P}}(\psi + \mathsf{cm})$, leaking
  more structure. The probability is negligible but tests assert
  the property.
- **`rseed` reuse**. Reusing $\mathsf{rseed}$ across two notes
  with the same recipient produces the same $\psi$ and
  $\mathsf{rcm}$, which leaks information. The encoding rules in
  ZIP 212 forbid reuse.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.16](https://zips.z.cash/protocol/protocol.pdf):
  Note commitments.
- [Zcash Protocol Specification, Section 4.17](https://zips.z.cash/protocol/protocol.pdf):
  Nullifier derivation for Orchard.
- [ZIP 212](https://zips.z.cash/zip-0212):
  standardised note randomness derivation.
- [`src/test_vectors.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/test_vectors.rs):
  the wiring of JSON test vectors that exercises notes and
  nullifiers.

## 6. Exercises

1. Read
   [`src/note/commitment.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/commitment.rs)
   and write down, in pseudocode, the byte encoding fed into
   `SinsemillaCommit`. Cite the bit-length constants
   (`L_ORCHARD_BASE`, `L_VALUE`, ...) you use.
2. Identify the unit test in
   [`src/note/nullifier.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/nullifier.rs)
   that checks against an external test vector. What format do
   the test vectors arrive in?
3. **Code task**. Add a unit test in `src/note.rs` that builds
   two notes with the same $\rho$ and confirms their
   nullifiers are distinct when $\mathsf{nk}$ or the other
   fields differ. Place the test under `#[cfg(test)]` and run
   `cargo test --lib note::`.

## 7. Further Reading

- [Zcash Protocol Specification, Section 4.1](https://zips.z.cash/protocol/protocol.pdf):
  the higher-level structure that
  notes / commitments / nullifiers fit into.
- The
  [Orchard Book, Notes](https://zcash.github.io/orchard/design/notes.html)
  walkthrough.
