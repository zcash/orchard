---
sidebar_position: 6
title: Sinsemilla
description: Domain-tagged windowed Pedersen hash on Pallas, used by both Merkle CRH and Commit^ivk.
---

# Sinsemilla

## 1. Why This Chapter Exists

Sinsemilla is the workhorse hash inside the Orchard circuit. It
appears in three places: the note commitment tree, the
$\mathsf{Commit}^{\mathsf{ivk}}$ commitment, and the note
commitment. A contributor who wants to add a new hash domain (a
new commitment, a new Merkle subtree) must understand the
construction, the generator table, and the incomplete-addition
discipline. After this chapter the reader can derive a new
Sinsemilla domain end to end.

## 2. Definitions

### Definition 2.1 (Sinsemilla Inputs)

Fix the window size $k = 10$. The hash operates on bit strings of
length divisible by $k$. Given $m \in \{0, 1\}^{kn}$, partition
into $n$ chunks $m = m_0 \mathbin{\|} m_1 \mathbin{\|} \dots
\mathbin{\|} m_{n-1}$ of $k$ bits each.

### Definition 2.2 (Generators)

For each domain $D$ a point $Q_D \in E_p$ is fixed by
hashing-to-curve a domain string. The shared generator table
$\{S_0, \dots, S_{2^k - 1}\} \subset E_p$ is obtained by
hashing-to-curve indexed strings.

### Definition 2.3 (Sinsemilla Hash to Point)

$$
\mathsf{SinsemillaHashToPoint}_D(m) =
\Big(\ldots
\big((Q_D \mathbin{\,\square\,} S_{m_0}) \mathbin{\,\square\,} S_{m_1}\big)
\mathbin{\,\square\,} \ldots \mathbin{\,\square\,} S_{m_{n-1}}\Big),
$$

where $\square$ is *incomplete addition* on Pallas (the standard
chord-and-tangent formula, undefined when the two inputs are
equal or opposite).

### Definition 2.4 (Sinsemilla Hash and Commit)

$\mathsf{SinsemillaHash}_D(m) =
\mathsf{Extract}_{\mathbb{P}}\big(
\mathsf{SinsemillaHashToPoint}_D(m)\big)$ produces a base-field
element. $\mathsf{SinsemillaCommit}_D(m; r) =
\mathsf{SinsemillaHashToPoint}_D(m) + [r]\, H_D$ produces a
hiding Pedersen commitment using a per-domain randomness
generator $H_D$.

### Invariant 2.5 (Incomplete-Addition Safety)

The incomplete-addition failure case is provably unreachable in
Sinsemilla because the generators $\{S_i\}$ are chosen so that no
running sum coincides with the next addend or its negation. The
canonical argument is reproduced in the Halo 2 Book.

## 3. The Code

### 3.1 The Implementation Crate

The hash is implemented in the external
[`sinsemilla`](https://github.com/zcash/sinsemilla) crate. Orchard
depends on it via Cargo:

```toml reference title="Cargo.toml"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/Cargo.toml#L43-L43
```

### 3.2 Orchard-Specific Constants

Domain separators and the precomputed generator table live in
[`src/constants/sinsemilla.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/constants/sinsemilla.rs).
The Merkle CRH personalisation is `MERKLE_CRH_PERSONALIZATION`;
the note commitment domain string is defined alongside.

### 3.3 In-Tree Use Sites

- The Merkle tree CRH is the Sinsemilla hash:
  [`src/tree.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/tree.rs).
- The note commitment is a Sinsemilla commit:
  [`src/note/commitment.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/commitment.rs).
- The IVK commitment is a Sinsemilla commit:
  [`src/spec.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/spec.rs).

### 3.4 Inside the Circuit

The circuit consumes Sinsemilla via the `halo2_gadgets::sinsemilla`
chip. The lookup table is the precomputed
$\{S_0, \dots, S_{1023}\}$. The Orchard-specific
[`NoteCommitChip`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/note_commit.rs)
and
[`CommitIvkChip`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/commit_ivk.rs)
encode the canonical bit-decomposition of their inputs into the
ten-bit windows that drive the chip.

## 4. Failure Modes

- **Wrong domain separator**. Reusing the personalisation string
  of a different domain collapses two semantically distinct
  hashes; Sinsemilla becomes trivially differentiable.
- **Non-canonical encoding**. The input bits to Sinsemilla must
  uniquely decode the original message. A missing range check in
  the chip lets a prover present two distinct messages with the
  same hash; see
  [`src/circuit/note_commit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/note_commit.rs)
  for the canonicality lookups.
- **Incomplete-addition collision**. If a future contributor adds
  a Sinsemilla domain whose $Q_D$ happens to collide with a
  running sum for some input, the circuit's incomplete-addition
  chip panics. The protection is the audit-time analysis in the
  Halo 2 Book; any new domain needs the same analysis.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 5.4.1.9](https://zips.z.cash/protocol/protocol.pdf):
  Sinsemilla hash function, parameters, and the proof of
  collision resistance.
- [Halo 2 Book, Sinsemilla](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html):
  the circuit-level treatment.
- [`zcash/sinsemilla`](https://github.com/zcash/sinsemilla):
  the implementation crate.

## 6. Exercises

1. Find the precomputed generator table on disk under
   [`src/constants/fixed_bases`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/constants/fixed_bases).
   How many bytes per generator, and how many generators?
2. The window size $k = 10$ is a trade-off between circuit cost
   per chunk and lookup-table size. Read the Halo 2 Book's
   rationale and write one paragraph: what happens to total
   circuit cost if $k$ doubles?
3. **Code task**. Add a new Sinsemilla domain string (e.g.
   `b"OrchardOnboardingTest"`) in a local copy of
   [`src/constants/sinsemilla.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/constants/sinsemilla.rs)
   and compute the corresponding $Q_D$ point. Print the affine
   coordinates and check that the result is on Pallas. Revert.

## 7. Further Reading

- [Halo 2 Book, Incomplete Addition](https://zcash.github.io/halo2/design/gadgets/ecc/addition.html):
  the formula and the analysis that justifies its use here.
- The original Sinsemilla appendix in the Zcash Protocol
  Specification.
