---
sidebar_position: 11
title: The Note Commitment Tree
description: Sinsemilla-hashed incremental Merkle tree of every Orchard note commitment.
---

# The Note Commitment Tree

## 1. Why This Chapter Exists

Orchard's anti-double-spend is a single global Merkle tree of all
note commitments. A spender proves, in zero-knowledge, that
their input note is in this tree at a specific position. After
this chapter the reader can produce the empty roots by hand,
trace `combine_inner` to the Sinsemilla call, and explain the
role of `incrementalmerkletree`.

## 2. Definitions

### Definition 2.1 (Tree Parameters)

Let $d = \mathtt{MERKLE\_DEPTH\_ORCHARD} = 32$. The tree has
$2^d$ leaves. The empty-leaf value is
$\mathsf{Uncommitted}_{\mathsf{Orchard}} = 2 \in \mathbb{F}_p$.

### Definition 2.2 (Inner Hash)

For $\ell \in \{0, \dots, d - 1\}$,

$$
\mathsf{MerkleCRH}^{\mathsf{Orchard}}_\ell(L, R) =
\mathsf{Extract}_{\mathbb{P}}\Big(
\mathsf{SinsemillaHashToPoint}_{D}\big(\ell \mathbin{\\|} L \mathbin{\\|} R\big)
\Big),
$$

where $D$ is the personalisation `MERKLE_CRH_PERSONALIZATION` and
$\ell$ is encoded as $\mathtt{L\_ORCHARD\_MERKLE}$ bits.

### Definition 2.3 (Empty Roots)

$$
\mathsf{EmptyRoots}[0] = \mathsf{Uncommitted}_{\mathsf{Orchard}},
\quad
\mathsf{EmptyRoots}[\ell + 1] =
\mathsf{MerkleCRH}^{\mathsf{Orchard}}_\ell\big(\mathsf{EmptyRoots}[\ell],\, \mathsf{EmptyRoots}[\ell]\big).
$$

## 3. The Code

### 3.1 Constants and Empty Roots

```rust reference title="src/tree.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/tree.rs#L25-L42
```

The `lazy_static!` blocks precompute
`UNCOMMITTED_ORCHARD` and the 33-entry `EMPTY_ROOTS` table once
per process.

### 3.2 `MerkleHashOrchard`

A `pallas::Base` wrapper that implements
`incrementalmerkletree::Hashable`. The trait's `combine` is
forwarded to `combine_inner`, which performs the Sinsemilla call
of Definition 2.2.

### 3.3 Frontier Maintenance

The Orchard tree itself is not stored in this crate; the
[`incrementalmerkletree`](https://github.com/zcash/incrementalmerkletree)
crate maintains a *frontier* (the right-most authentication path
plus the right-most leaf at each level), which is enough to
append new leaves and produce paths for live notes.

### 3.4 Inside the Circuit

The Merkle path is hashed inside the Action circuit by the
`MerkleChip` from `halo2_gadgets`; the chip recomputes the chain
of Sinsemilla calls and constrains the final $x$-coordinate to
equal the public anchor.

## 4. Failure Modes

- **Off-by-one depth**. `MERKLE_DEPTH_ORCHARD = 32` means 32
  levels of inner hashes; PRs that introduce `33` or `31` in any
  hash level off-set break consensus.
- **Forgetting $\ell$ in the input**. The depth tag prevents a
  collision between hashes at different levels. Removing it lets
  a path at level 5 be reinterpreted as a path at level 6.
- **`Uncommitted` collision**. The value $2$ was chosen because
  no genuine note commitment is exactly $2$. Adding a new
  "uncommitted" sentinel without checking the collision argument
  is a soundness risk.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.10](https://zips.z.cash/protocol/protocol.pdf):
  Note Commitment Trees.
- [Zcash Protocol Specification, Section 5.4.1.10](https://zips.z.cash/protocol/protocol.pdf):
  Merkle CRH parameters.
- [`incrementalmerkletree`](https://github.com/zcash/incrementalmerkletree):
  the frontier implementation.

## 6. Exercises

1. Compute `EMPTY_ROOTS[0..3]` by hand (running Sinsemilla) and
   cross-check against the lazy-static table by writing a
   one-line unit test.
2. Why is $\ell$ encoded as a fixed-length bit string rather than
   a single byte? Reason about Sinsemilla's chunking.
3. **Code task**. Insert ten leaves into an
   `incrementalmerkletree` frontier (using its API), then ask
   the frontier for the authentication path of the third leaf.
   Recompute the root from the leaf and the path; assert
   equality.

## 7. Further Reading

- [Orchard Book, Note Commitment Tree](https://zcash.github.io/orchard/design/commitments.html):
  the architectural overview maintained by EC Co.
- [Halo 2 Book, Merkle chip](https://zcash.github.io/halo2/design/gadgets/sinsemilla/merkle-crh.html):
  the in-circuit construction.
