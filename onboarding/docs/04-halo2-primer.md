---
sidebar_position: 4
title: Halo 2 Primer
description: PLONKish tables, custom gates, lookups, and the IPA transcript as they appear in src/circuit.rs.
---

# Halo 2 Primer

## 1. Why This Chapter Exists

The Orchard Action circuit is a Halo 2 circuit. A reader who has
only seen R1CS-style systems will not be able to navigate
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)
without first learning what advice columns, selectors, custom
gates, and lookups are. By the end the reader can read a Halo 2
`Config` struct and predict the column layout it will produce.

## 2. Definitions

### Definition 2.1 (PLONKish Table)

A Halo 2 circuit is a rectangular table with $2^K$ rows and a
fixed number of columns of four kinds:

- *advice* columns $A_0, \dots, A_{a-1}$, populated by the prover;
- *fixed* columns $F_0, \dots, F_{f-1}$, populated by the verifier
  during trusted preprocessing;
- *instance* columns $I_0, \dots, I_{i-1}$, the public inputs;
- *selector* columns $S_0, \dots, S_{s-1}$, fixed columns that
  toggle constraints on or off per row.

### Definition 2.2 (Custom Gate)

A custom gate is a multivariate polynomial
$g \in \mathbb{F}_q[X_0, \dots, X_{m-1}]$ together with a selector
$S_g$. For every row $r$,

$$
S_g(r) \cdot g\big(c_0(r), c_1(r), \dots, c_{m-1}(r)\big) = 0,
$$

where each $c_j(r)$ is some column at some row $r + \delta_j$. The
offsets $\delta_j \in \{-1, 0, +1\}$ are *rotations* and let
gates couple adjacent rows.

### Definition 2.3 (Lookup)

A lookup argument requires that for every row $r$, the tuple
$\big(c_0(r), \dots, c_{m-1}(r)\big)$ appears as a row of a fixed
table $T \subset \mathbb{F}_q^m$. Lookups encode range checks
($T = \{(0), (1), \dots, (2^k - 1)\}$) and the Sinsemilla
windowed multiplication table.

### Definition 2.4 (Inner-Product Argument Commitment)

A column $c$ of length $2^K$ is committed as
$\mathrm{Comm}(c) = \sum_{i = 0}^{2^K - 1} c_i \, G_i$ for a fixed
basis $\{G_i\} \subset \mathbb{G}$. The IPA protocol opens
$\mathrm{Comm}(c)$ at a point $\zeta$ in $O(\log 2^K)$ rounds with
no trusted setup. In Orchard, $\mathbb{G}$ is the Vesta curve.

### Definition 2.5 (Transcript)

The Fiat-Shamir transcript is a Blake2b instance personalised with
`"Halo2-Transcript"`. Every commitment and challenge is absorbed
in protocol order. The outer transcript is not recursive in
Orchard.

## 3. The Code

### 3.1 The Halo 2 Imports

```rust reference title="src/circuit.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L5-L15
```

The `plonk` re-exports name the column kinds, the `Constraints`
builder, the `Expression` type used in gates, and the verifier
choices (`SingleVerifier`, `BatchVerifier`). The `transcript`
re-exports give Blake2b read/write halves.

### 3.2 The Chip / Gadget Pattern

A *chip* is a `Config` (column layout, gate definitions) plus the
synthesis code that populates a region of the table. A *gadget* is
a higher-level construction composed of one or more chips. Orchard
pulls the ECC and Sinsemilla chips from
[`halo2_gadgets`](https://github.com/zcash/halo2)
and adds its own Orchard-specific chips:
[`CommitIvkChip`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/commit_ivk.rs)
and
[`NoteCommitChip`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/note_commit.rs).

### 3.3 The `K` Constant

The Action circuit uses
$K$ defined as a constant in
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs).
$2^K$ is the row count of the table. Larger $K$ admits more
constraints but doubles the prover time per increment.

### 3.4 The Lookup Table

The Sinsemilla windowed multiplication uses a fixed lookup table
populated from
[`src/constants/sinsemilla.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/constants/sinsemilla.rs).
The table has $2^{10}$ rows, one per ten-bit window value, and
maps the window to the precomputed generator multiple.

## 4. Failure Modes

- **Region overlap**. Chips synthesise into *regions*. Two chips
  with overlapping selectors silently corrupt each other; the
  reviewer must trace the region offsets.
- **Constraint not gated**. A custom gate that lacks a selector
  multiplier applies to every row, including padding rows whose
  advice columns are zero. The Halo 2 dev-mode prover catches
  this; production proving silently produces a wrong proof.
- **Wrong `K`**. Setting `K` too small causes
  `Error::NotEnoughRows`; too large is silent and wastes prover
  time. The
  [`circuit_description`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)
  snapshot pins the chosen `K`.
- **Transcript divergence between prover and verifier**. If a
  field element is absorbed in a different order or under a
  different personalisation string, verification fails. This is
  the most common cause of a "passes locally, fails in CI" report
  after a Halo 2 dependency bump.

## 5. Spec Pointers

- [PLONK paper](https://eprint.iacr.org/2019/953):
  the underlying argument.
- [Halo paper](https://eprint.iacr.org/2019/1021):
  the IPA-based recursion that motivated Halo 2.
- [Halo 2 Book](https://zcash.github.io/halo2/):
  the canonical reference for the column model, gates, and
  lookups.
- [`zcash/halo2`](https://github.com/zcash/halo2):
  the source of `halo2_proofs` and `halo2_gadgets`.

## 6. Exercises

1. Open
   [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)
   and find the `K` constant. State its value and explain how
   `2^K` relates to the constraint count.
2. List every `meta.advice_column()` and `meta.fixed_column()`
   call in
   [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs).
   Group them by the chip that consumes them.
3. **Code task**. Add a new advice column to the Action circuit
   without consuming it (a dead column). Run `cargo test --lib
   circuit::tests`. Confirm the dev-mode prover rejects the
   change with an "unused column" lint, then revert.

## 7. Further Reading

- [Halo 2 Book, Concepts: Plonkish arithmetisation](https://zcash.github.io/halo2/concepts/arithmetization.html):
  the formal model used by `halo2_proofs`.
- [`halo2_gadgets` source](https://github.com/zcash/halo2/tree/main/halo2_gadgets):
  the chip library most of `src/circuit.rs` builds on.
