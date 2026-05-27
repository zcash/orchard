---
sidebar_position: 8
title: notes, nullifiers, and commitments
---

# notes, nullifiers, and commitments

## motivation

A *note* is the Orchard analogue of a UTXO: a record describing a
single spendable output. Two cryptographic objects make a note
usable in shielded transactions:

- the *note commitment* $\mathsf{cm}$, an opaque commitment to the
  note's contents that goes into the global note commitment tree;
- the *nullifier* $\mathsf{nf}$, a deterministic identifier that is
  revealed when the note is spent, preventing double-spending.

The same note is committed once and nullified at most once. This
chapter walks the data structures and the deterministic derivation
of $\mathsf{cm}$ and $\mathsf{nf}$.

## the math

A note is the tuple

$$
n = (d, \mathsf{pk_d}, v, \rho, \psi, \mathsf{rcm}),
$$

with

- $d \in \{0, 1\}^{88}$: the diversifier;
- $\mathsf{pk_d} \in E_p$: the diversified transmission key;
- $v \in [0, 2^{64})$: the note value;
- $\rho \in \mathbb{F}_p$: a "nonce" tying the note to a unique
  predecessor;
- $\psi \in \mathbb{F}_p$: an auxiliary randomness derived from
  $\rho$ for technical reasons (the "$\psi$ trick");
- $\mathsf{rcm} \in \mathbb{F}_q$: the note commitment trapdoor.

Both $\psi$ and $\mathsf{rcm}$ are derived from the sender's randomness
$\mathsf{rseed} \in \{0, 1\}^{256}$ via the ZIP 212 PRFs.

The note commitment is

$$
\mathsf{cm} = \mathsf{NoteCommit}_{\mathsf{rcm}}\big(g_d, \mathsf{pk_d}, v, \rho, \psi\big),
$$

a Sinsemilla commitment over the canonical encoding of the note's
algebraic fields. The *extracted* commitment

$$
\mathsf{cm}^\star = \mathsf{Extract}_{\mathbb{P}}(\mathsf{cm})
$$

is the value that goes into the Merkle tree.

The nullifier is

$$
\mathsf{nf} = \mathsf{Extract}_{\mathbb{P}}\Big(
\big[\mathsf{PRF}^{\mathsf{nfOrchard}}_{\mathsf{nk}}(\rho)\big]\, \mathcal{K} \;+\; \psi
\;+\; \mathsf{cm}\Big),
$$

where $\mathcal{K}$ is the *nullifier base* generator (a fixed point
of $E_p$ defined by hashing-to-curve with a domain string).
$\mathsf{PRF}^{\mathsf{nfOrchard}}$ is Poseidon-based (chapter
[poseidon](./06-poseidon.md)).

## the implementation

The `Note` type and its constructors are in
[`src/note.rs`](https://github.com/zcash/orchard/blob/main/src/note.rs):

```rust reference title="src/note.rs"
https://github.com/zcash/orchard/blob/main/src/note.rs#L1-L40
```

The submodules
[`src/note/commitment.rs`](https://github.com/zcash/orchard/blob/main/src/note/commitment.rs)
and
[`src/note/nullifier.rs`](https://github.com/zcash/orchard/blob/main/src/note/nullifier.rs)
implement $\mathsf{cm}$ and $\mathsf{nf}$ respectively.

`NoteCommitment::derive` encodes the note's fields canonically:
$g_d$ and $\mathsf{pk_d}$ are encoded as 256-bit compressed points,
$v$ as 64 bits little-endian, $\rho$ and $\psi$ as $\ell_{\mathsf{Orchard}\,\mathsf{base}} = 255$ bits.
The encoding is then hashed with `SinsemillaCommit` under the domain
`Z.Orchard-NoteCommit` parameter, see
[`src/constants/sinsemilla.rs`](https://github.com/zcash/orchard/blob/main/src/constants/sinsemilla.rs).

`Nullifier::derive` is the orchard version of the Poseidon-plus-point
formula above; the `nk` (a base-field element) is used as the
Poseidon key, $\rho$ as the message, and the resulting field element
is multiplied by the nullifier base and combined with $\psi$ and
$\mathsf{cm}$ before x-extraction.

The `Rho` newtype guards $\rho$:

```rust reference title="src/note.rs"
https://github.com/zcash/orchard/blob/main/src/note.rs#L37-L42
```

It is constructed from the nullifier of the previous note in the
spend chain; this is what makes nullifiers unforgeable even for an
attacker who can mint arbitrary notes off-chain.

## specification and references

- Zcash Protocol Specification,
  [Section 4.1.6 (Note plaintexts and memo fields)](https://zips.z.cash/protocol/protocol.pdf)
  and Section 4.16 (Note commitments).
- [ZIP 212](https://zips.z.cash/zip-0212): standardised note
  randomness derivation; see "Orchard note plaintexts".
- Zcash Protocol Specification, Section 4.17 (Nullifier derivation
  for Orchard).

## exercises

1. Read
   [`src/note/commitment.rs`](https://github.com/zcash/orchard/blob/main/src/note/commitment.rs)
   and write down, in pseudocode, the canonical byte encoding of a
   Note that is fed into `SinsemillaCommit`.
2. The protocol uses both $\rho$ and the nullifier base $\mathcal{K}$
   to derive $\mathsf{nf}$. What attack does each one prevent? (Hint:
   consider an attacker who knows $\mathsf{nk}$ but not $\rho$, and
   one who knows $\rho$ but not $\mathsf{nk}$.)
3. Trace `Rho::from_bytes` to its caller. Where in the codebase is
   the previous note's nullifier turned into the next $\rho$?
