---
sidebar_position: 5
title: sinsemilla
---

# sinsemilla

## motivation

Sinsemilla is the algebraic hash function used in Orchard for two
purposes: hashing into curve points (the note commitment tree and
$\mathsf{Commit}^{\mathsf{ivk}}$) and providing a Pedersen-style
commitment that is cheap inside a Halo 2 circuit. It is much cheaper
inside a PLONKish circuit than a generic hash like SHA-256 because
it is built from 10-bit windowed scalar multiplications and a fixed
table of generators. This chapter gives the construction and points
at the Rust implementation.

## the math

Sinsemilla is parametrised by a *domain separator* $D$ and operates
on bitstrings of length divisible by $k = 10$. Given a bitstring
$m \in \{0, 1\}^{kn}$, partition $m$ into $n$ chunks of $k$ bits:
$m = m_0 \,\|\, m_1 \,\|\, \dots \,\|\, m_{n-1}$.

Two families of curve points are fixed:

- $Q_D$: a per-domain starting point.
- $S_0, S_1, \dots, S_{2^k - 1}$: a table of $2^k = 1024$ generators
  shared across domains, obtained by hashing-to-curve a domain string
  and an index.

The Sinsemilla hash is the incomplete-addition chain

$$
\mathsf{SinsemillaHashToPoint}_D(m) =
\Big(\ldots \big((Q_D \;\;\mathsf{incadd}\;\; S_{m_0}) \;\;\mathsf{incadd}\;\; S_{m_1}\big)
\;\;\mathsf{incadd}\;\; \ldots\;\; S_{m_{n-1}}\Big),
$$

where $\mathsf{incadd}$ is the *incomplete addition* formula on the
Pallas curve (it is undefined only when the two inputs are equal or
opposite, which the protocol is designed to avoid).

To produce a field element, take the $x$-coordinate via
$\mathsf{Extract}_{\mathbb{P}}$:

$$
\mathsf{SinsemillaHash}_D(m) =
\mathsf{Extract}_{\mathbb{P}}\big(\mathsf{SinsemillaHashToPoint}_D(m)\big).
$$

To produce a Pedersen-style commitment, add a blinding term:

$$
\mathsf{SinsemillaCommit}_D(m; r) =
\mathsf{SinsemillaHashToPoint}_D(m) + [r] \, H_D,
$$

where $H_D$ is the per-domain randomness generator.

### why this is circuit-friendly

The map $m_i \in \{0, 1\}^{10} \mapsto S_{m_i}$ is one lookup into a
fixed 1024-row table; the running sum is a chain of *additions of a
known multiple of a known generator*, which is what `halo2_gadgets`'
ECC chip handles natively. The cost per input chunk is dominated by
that single incomplete addition.

## the implementation

The pure-rust implementation of the hash lives in the
[`sinsemilla`](https://github.com/zcash/sinsemilla) crate; Orchard
imports it as a dependency. The Orchard-specific generators and
domain separators are in
[`src/constants/sinsemilla.rs`](https://github.com/zcash/orchard/blob/main/src/constants/sinsemilla.rs)
and
[`src/constants/fixed_bases`](https://github.com/zcash/orchard/blob/main/src/constants/fixed_bases).

The Merkle tree uses Sinsemilla as the inner CRH:

```rust reference title="src/tree.rs"
https://github.com/zcash/orchard/blob/main/src/tree.rs#L1-L20
```

The personalisation string for the Merkle CRH is
`MERKLE_CRH_PERSONALIZATION`, defined in
[`src/constants/sinsemilla.rs`](https://github.com/zcash/orchard/blob/main/src/constants/sinsemilla.rs).

Inside the circuit, Sinsemilla is used through the
`halo2_gadgets::sinsemilla` chip, whose hash table column is
populated from the fixed-base constants. The Orchard-specific
hashing of a note's encoded payload happens in
[`src/circuit/note_commit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit/note_commit.rs)
and the IVK commitment in
[`src/circuit/commit_ivk.rs`](https://github.com/zcash/orchard/blob/main/src/circuit/commit_ivk.rs).

## specification and references

- Zcash Protocol Specification,
  [Section 5.4.1.9 "Sinsemilla Hash Function"](https://zips.z.cash/protocol/protocol.pdf).
- Sinsemilla announcement, in the Zcash Protocol Specification
  appendix.
- [`zcash/sinsemilla`](https://github.com/zcash/sinsemilla) source.
- [Incomplete addition on Pallas](https://zcash.github.io/halo2/design/gadgets/ecc/addition.html#incomplete-addition)
  in the Halo 2 book.

## exercises

1. The fixed table of generators has $2^{10} = 1024$ entries. Find the
   on-disk pre-computed data in
   [`src/constants/fixed_bases`](https://github.com/zcash/orchard/blob/main/src/constants/fixed_bases).
   Where is the file that produces these generators originally
   defined?
2. Incomplete addition fails when the two inputs are equal or
   opposite. Why is it acceptable for Sinsemilla to use incomplete
   addition? Sketch the argument that the failure case never occurs
   for a well-formed input.
3. Implement Sinsemilla over a toy curve in a scratch file, with
   $k = 4$ instead of $k = 10$, and check that hashing the empty
   string gives $Q_D$.
