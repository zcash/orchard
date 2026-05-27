---
sidebar_position: 10
title: the merkle tree
---

# the merkle tree

## motivation

Orchard maintains a single global Merkle tree of every note
commitment ever created. A spender proves, in zero-knowledge, that
the note they are spending was once committed, by providing a Merkle
authentication path from the note commitment leaf to the tree's
*anchor* (root). The hash used at each Merkle node is a domain-tagged
Sinsemilla. This chapter describes the structure of the tree, the
empty-leaf convention, and the incremental implementation that
clients use to maintain a state-pruned view of the tree.

## the math

Fix $d = $ `MERKLE_DEPTH_ORCHARD` (currently 32). The Merkle tree
has $2^d$ leaves. The "empty" leaf is the field element
$\mathsf{Uncommitted}_{\mathsf{Orchard}} = 2 \in \mathbb{F}_p$. The
inner hash at level $\ell \in \{0, \dots, d - 1\}$ takes two child
hashes and returns

$$
\mathsf{MerkleCRH}^{\mathsf{Orchard}}_\ell(\mathsf{left}, \mathsf{right}) =
\mathsf{Extract}_{\mathbb{P}}\Big(
\mathsf{SinsemillaHashToPoint}_{D}(\ell \,\|\, \mathsf{left} \,\|\, \mathsf{right})
\Big),
$$

where $D = $ `Zcash_OrchardCRH_M`. Level $\ell$ is encoded as an
unsigned 10-bit value (`L_ORCHARD_MERKLE`). The anchor is the result
of hashing the leaves up to the root.

The choice of $\mathsf{Uncommitted}_{\mathsf{Orchard}} = 2$ avoids
collisions with any genuine note commitment: a genuine
$\mathsf{cm}^\star$ is the $x$-coordinate of a curve point and is
extremely unlikely to be exactly $2$, but the spec fixes the value
explicitly to remove ambiguity.

The set of *empty roots* $\mathsf{EmptyRoots}[\ell]$ for
$\ell \in \{0, \dots, d\}$ is precomputed: $\mathsf{EmptyRoots}[0] =
\mathsf{Uncommitted}_{\mathsf{Orchard}}$, and
$\mathsf{EmptyRoots}[\ell + 1] = \mathsf{MerkleCRH}^{\mathsf{Orchard}}_\ell
(\mathsf{EmptyRoots}[\ell], \mathsf{EmptyRoots}[\ell])$.

## the implementation

The Orchard tree is in
[`src/tree.rs`](https://github.com/zcash/orchard/blob/main/src/tree.rs):

```rust reference title="src/tree.rs"
https://github.com/zcash/orchard/blob/main/src/tree.rs#L25-L42
```

The empty leaf and the precomputed empty roots are declared as
`lazy_static` values. `MerkleHashOrchard` is the newtype that
implements `incrementalmerkletree::Hashable`, which is the trait the
external
[`incrementalmerkletree`](https://github.com/zcash/incrementalmerkletree)
crate uses to maintain a frontier of the global tree.

The personalisation constant `MERKLE_CRH_PERSONALIZATION` is in
[`src/constants/sinsemilla.rs`](https://github.com/zcash/orchard/blob/main/src/constants/sinsemilla.rs).
The level encoding helper `i2lebsp_k` (little-endian bit string of
fixed length) is in the same file.

The depth constant lives in
[`src/constants.rs`](https://github.com/zcash/orchard/blob/main/src/constants.rs)
as `MERKLE_DEPTH_ORCHARD`.

`MerkleHashOrchard::combine_inner` applies Sinsemilla with the level
prefix; `combine` is `combine_inner` wrapped so callers can pass a
`Level` from `incrementalmerkletree` directly.

Inside the Action circuit, the Merkle path is hashed by the
`MerkleChip` from `halo2_gadgets`; the path bits are part of the
witness and the chip computes the chained Sinsemilla hash inside
the circuit, terminating at the public anchor instance column.

## specification and references

- Zcash Protocol Specification,
  [Section 4.10 "Note Commitment Trees"](https://zips.z.cash/protocol/protocol.pdf)
  and Section 5.4.1.10 (MerkleCRH).
- [`incrementalmerkletree`](https://github.com/zcash/incrementalmerkletree).
- [`zcash/sinsemilla`](https://github.com/zcash/sinsemilla).

## exercises

1. Compute the first three empty roots in `Hex` and cross-check with
   the values produced by `EMPTY_ROOTS[0..3]` in
   [`src/tree.rs`](https://github.com/zcash/orchard/blob/main/src/tree.rs)
   (you can verify by writing a unit test).
2. Why is the level $\ell$ included in the input to
   $\mathsf{MerkleCRH}$? What attack does it prevent? (Hint:
   length-extension and depth-confusion attacks.)
3. The crate's external dependency
   [`incrementalmerkletree`](https://github.com/zcash/incrementalmerkletree)
   maintains a frontier rather than the full tree. Explain in two
   sentences what the frontier is and why it is enough to insert new
   leaves and produce proofs.
