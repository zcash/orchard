---
sidebar_position: 1
title: overview and crate map
---

# overview and crate map

## motivation

The `orchard` crate is the reference implementation of the Orchard
shielded protocol that activated on the Zcash network at network
upgrade 5 (NU5). It implements the entire client-side of a shielded
Orchard transaction: key derivation, note creation, note commitment
trees, note encryption, value commitments, the Action circuit (a
Halo 2 zk-SNARK), and the bundle / builder used by wallets. This
chapter is a map: it names the modules, says what each one is
responsible for, and gives an entry point for the rest of the course.

## the math, in one paragraph

Orchard is a UTXO-style shielded protocol over the Pallas curve. A
note $n = (d, \mathsf{pk_d}, v, \rho, \psi, \mathsf{rcm})$ encodes a
spendable output. Notes are committed by Sinsemilla into
$\mathsf{cm} = \mathsf{NoteCommit}(d, \mathsf{pk_d}, v, \rho, \psi;
\mathsf{rcm})$ and inserted into a global Sinsemilla-hashed Merkle
tree. A transaction consists of $N$ Actions; each Action proves, in
zero-knowledge, that an existing note is being spent (revealing only
its nullifier $\mathsf{nf}$) and that a fresh note is being created
(revealing only its commitment $\mathsf{cm}^{\star}$). Value
conservation across all Actions is enforced by a homomorphic Pedersen
commitment $\mathsf{cv^{\mathsf{net}}}$, and spend authority by a
RedPallas signature $\mathsf{spendAuthSig}$ keyed by a re-randomised
spend authorising key $\mathsf{rk}$.

## the implementation

The crate's public surface is declared in
[`src/lib.rs`](https://github.com/zcash/orchard/blob/main/src/lib.rs).
The module graph is roughly:

```text
lib
+-- action            // a single Orchard Action
+-- address           // shielded payment addresses
+-- builder           // transaction-building API for wallets
+-- bundle            // a set of Actions + proof + signatures
|   +-- batch
|   +-- commitments
+-- circuit           // the Halo 2 Action circuit
|   +-- commit_ivk
|   +-- gadget
|   +-- note_commit
+-- constants         // domain separators, fixed bases, generators
|   +-- fixed_bases
|   +-- sinsemilla
+-- keys              // ZIP 32 key tree (sk, ask, nk, ivk, ovk, dk, ...)
+-- note              // Note, NoteCommitment, Nullifier
|   +-- commitment
|   +-- nullifier
+-- note_encryption   // ZIP 212 / ZIP 316 note encryption
+-- pczt              // partially constructed transactions
+-- primitives        // RedPallas, plus re-exports of sinsemilla/poseidon
|   +-- redpallas
+-- spec              // small spec-faithful primitives
+-- tree              // incremental Sinsemilla Merkle tree
+-- value             // NoteValue, ValueSum, ValueCommitment
+-- zip32             // BIP 32 / ZIP 32 derivation for Orchard
```

A direct view of the public re-exports:

```rust reference title="src/lib.rs"
https://github.com/zcash/orchard/blob/main/src/lib.rs#L51-L60
```

How the modules compose to make a transaction:

1. A wallet derives keys from a seed via
   [`zip32`](https://github.com/zcash/orchard/blob/main/src/zip32.rs)
   to produce an
   [`ExtendedSpendingKey`](https://github.com/zcash/orchard/blob/main/src/zip32.rs)
   and the corresponding
   [`FullViewingKey`](https://github.com/zcash/orchard/blob/main/src/keys.rs).
2. The wallet receives notes; each note is an instance of
   [`Note`](https://github.com/zcash/orchard/blob/main/src/note.rs)
   committed to a leaf in the
   [`tree`](https://github.com/zcash/orchard/blob/main/src/tree.rs).
3. To spend, the wallet constructs Actions through the
   [`Builder`](https://github.com/zcash/orchard/blob/main/src/builder.rs);
   the builder generates randomness, derives the nullifier, builds
   the new note commitment, and prepares the Action circuit witness.
4. The Action witnesses are folded into a
   [`Bundle`](https://github.com/zcash/orchard/blob/main/src/bundle.rs)
   together with a single Halo 2 proof
   ([`circuit`](https://github.com/zcash/orchard/blob/main/src/circuit.rs)),
   a binding signature on the net value commitment, and per-Action
   spend authorising signatures
   ([`primitives::redpallas`](https://github.com/zcash/orchard/blob/main/src/primitives/redpallas.rs)).
5. The verifier checks the proof, the signatures, and that all
   anchors and nullifiers are well-formed.

## specification and references

- Zcash Protocol Specification,
  [Section 5.4 (Orchard)](https://zips.z.cash/protocol/protocol.pdf).
- [ZIP 224](https://zips.z.cash/zip-0224): the Orchard shielded
  protocol activation ZIP.
- [Orchard Book](https://zcash.github.io/orchard/): the mdBook
  maintained alongside the crate; this onboarding site is a more
  hands-on, code-linked companion to it.

## exercises

1. Open
   [`src/lib.rs`](https://github.com/zcash/orchard/blob/main/src/lib.rs).
   List every `pub mod` and decide, just from the module name, which
   chapter of this course covers it.
2. The
   [`circuit`](https://github.com/zcash/orchard/blob/main/src/circuit.rs)
   module is gated behind `#[cfg(feature = "circuit")]`. Find the
   declaration in `lib.rs` and explain in one sentence why a downstream
   user might want to disable the `circuit` feature.
3. The crate is `#![no_std]`. Find the corresponding attribute in
   [`src/lib.rs`](https://github.com/zcash/orchard/blob/main/src/lib.rs)
   and list the two `extern crate` declarations that follow it.
