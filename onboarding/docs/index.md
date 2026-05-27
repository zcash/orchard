---
sidebar_position: 0
slug: /
title: orchard onboarding course
---

# orchard onboarding course

This site is a graduate-level reading course on the
[`zcash/orchard`](https://github.com/zcash/orchard) crate. Each chapter
combines the math behind a piece of the Orchard shielded protocol with
pointers to the actual Rust code, references to the canonical
specification, and exercises that can be checked against the source.

:::warning Automatically generated content

This site is automatically generated using
[Claude Code](https://www.claude.com/product/claude-code). Errors may
have been introduced. The code is the law: always refer to the
authoritative sources before drawing a conclusion.

The three authoritative references for everything that follows are:

1. The source files in
   [`zcash/orchard`](https://github.com/zcash/orchard/tree/main/src).
2. The
   [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
   (NU5 and later sections cover Orchard).
3. The relevant
   [ZIPs](https://zips.z.cash/), in particular
   [ZIP 32](https://zips.z.cash/zip-0032),
   [ZIP 212](https://zips.z.cash/zip-0212),
   [ZIP 224](https://zips.z.cash/zip-0224) (Orchard activation), and
   [ZIP 316](https://zips.z.cash/zip-0316) (Unified Addresses).

If you spot an error, please open an issue or a pull request on the
onboarding branch of the fork:
[dannywillems/orchard `onboarding`](https://github.com/dannywillems/orchard/tree/onboarding).

:::

## how to read this course

The chapters are numbered. Each one follows the same skeleton:

1. **Motivation**: three to five sentences on what the chapter is about
   and why it matters in Orchard.
2. **The math**: definitions, equations rendered with KaTeX, and the
   security games where they apply.
3. **The implementation**: a walk through the relevant modules, types,
   and functions in `zcash/orchard`, with line-anchored links into the
   source on GitHub.
4. **Specification and references**: pointers to the corresponding
   sections of the Zcash Protocol Specification, the relevant ZIPs, and
   the seminal papers (Halo, Halo 2, Pasta, Sinsemilla, Poseidon).
5. **Exercises**: questions whose answers can be located in the code or
   the spec.

## prerequisites

The reader is assumed to be comfortable with the following:

- Finite field arithmetic over $\mathbb{F}_p$ and the basics of
  elliptic curves over prime fields.
- Discrete-log based cryptography: Pedersen commitments, Schnorr-style
  signatures, key exchange.
- Cryptographic hash functions and the random oracle model.
- The Rust language at the level of reading a non-trivial crate.

Familiarity with PLONK or Halo 2 is helpful but not required; chapter
[Halo 2 primer](./03-halo2-primer.md) introduces the parts of PLONKish
arithmetisation needed to read `src/circuit.rs`.

## notation

This course follows the notation of the Zcash Protocol Specification,
which is itself close to the notation of the original papers:

| Symbol                                | Meaning                              |
| ------------------------------------- | ------------------------------------ |
| $\mathbb{F}_p$                        | The prime field with $p$ elements    |
| $[k]P$                                | Scalar multiplication of $P$ by $k$  |
| $\mathsf{Com}(m; r)$                  | Commitment to $m$ with randomness $r$ |
| $x \stackrel{\$}{\leftarrow} S$       | Sample $x$ uniformly from $S$        |
| $\mathsf{Sinsemilla}_D(m)$            | Sinsemilla hash of $m$ in domain $D$ |
| $\mathsf{Poseidon}(x_1, \dots, x_n)$  | Poseidon permutation output          |
| $q$                                   | Order of the Pallas base field       |
| $r$                                   | Order of the Pallas scalar field     |

The Pallas base field and the Pallas scalar field swap roles in the
Vesta curve; this is the heart of the
[Pasta cycle](./02-pasta-curves.md).

## the seminal references in one place

- [Halo](https://eprint.iacr.org/2019/1021): the recursion technique
  that removes the trusted setup.
- [Halo 2](https://zcash.github.io/halo2/): PLONKish arithmetisation
  with custom gates and lookups.
- [PLONK](https://eprint.iacr.org/2019/953): the underlying proof
  system.
- [Pasta curves](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/):
  the cycle of curves used by Halo 2.
- [Sinsemilla](https://zips.z.cash/protocol/protocol.pdf#concretesinsemillahash):
  the hash function used for the note commitment tree and for
  $\mathsf{Commit}^{\mathsf{ivk}}$.
- [Poseidon](https://eprint.iacr.org/2019/458): the algebraic hash
  used inside the Orchard Action circuit.
- [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf):
  the normative specification of Orchard.

## chapter index

The sidebar on the left lists every chapter in order. The intended
reading path is linear, but each chapter is self-contained enough to
be read on its own once the notation table above has been internalised.
