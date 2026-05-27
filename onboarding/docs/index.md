---
sidebar_position: 0
slug: /
title: Orchard Onboarding Course
description: A graduate-level reading course on the zcash/orchard crate, pinned to tag 0.13.1.
---

# Orchard Onboarding Course

This site is a graduate-level, contribution-oriented reading course
on the [`zcash/orchard`](https://github.com/zcash/orchard) crate.
Each chapter pairs the math behind a piece of the Orchard shielded
protocol with the matching Rust code, the relevant specification,
and exercises that force the reader to touch the source. The
intended outcome is concrete: read the course, then open a real PR
against the crate.

:::warning Automatically Generated Content

This site is automatically generated using
[Claude Code](https://www.claude.com/product/claude-code). Errors
may have been introduced. The code in the repository is the law:
always refer to the authoritative sources before drawing a
conclusion.

The three authoritative references for everything that follows are:

1. The source files in
   [`zcash/orchard`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src),
   pinned to tag
   [`0.13.1`](https://github.com/zcash/orchard/releases/tag/0.13.1).
2. The
   [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
   (Section 4 covers shielded payment components and Section 5.4
   covers Orchard).
3. The relevant
   [ZIPs](https://zips.z.cash/), in particular
   [ZIP 32](https://zips.z.cash/zip-0032),
   [ZIP 212](https://zips.z.cash/zip-0212),
   [ZIP 224](https://zips.z.cash/zip-0224),
   [ZIP 244](https://zips.z.cash/zip-0244), and
   [ZIP 316](https://zips.z.cash/zip-0316).

To file a correction, open an issue or a pull request on the
onboarding branch of the fork:
[dannywillems/orchard `onboarding`](https://github.com/dannywillems/orchard/tree/onboarding).

:::

## How to Read This Course

Chapters are numbered `NN-slug.md` so the sidebar matches the
reading order. Every chapter follows the same seven-section
skeleton:

1. **Why This Chapter Exists**: the question the chapter answers
   and the file or function the reader will touch by the end.
2. **Definitions**: formal definitions (KaTeX for math, grammars
   or state machines otherwise). Downstream chapters cite these by
   name.
3. **The Code**: a walkthrough of the implementation with
   line-anchored live source embeds, annotated with the invariant
   each block enforces.
4. **Failure Modes**: what goes wrong if a contributor changes the
   code without understanding the chapter. Audit findings, CVEs,
   or historical bugs are cited when available.
5. **Spec Pointers**: the relevant ZIPs, sections of the Zcash
   Protocol Specification, and papers, each with the reason it is
   cited.
6. **Exercises**: at least three, of which at least one requires
   the reader to modify code or add a test.
7. **Further Reading**: optional pointers for going deeper.

The supporting pages at the bottom of the sidebar
([Cheat Sheet](./cheat-sheet.md), [PR Checklist](./pr-checklist.md),
[Glossary](./glossary.md), [Discovery Notes](./discovery.md)) are
not chapters; they are reference material returned to during work.

## Prerequisites

The reader is assumed to be comfortable with:

- Finite field arithmetic over $\mathbb{F}_p$ and the basics of
  elliptic curves over prime fields.
- Discrete-log based cryptography: Pedersen commitments,
  Schnorr-style signatures, key exchange.
- Cryptographic hash functions and the random oracle model.
- The Rust language at the level of reading a non-trivial crate
  with `#![no_std]`.

Familiarity with PLONK or Halo 2 is helpful but not required;
[Chapter 4 (Halo 2 Primer)](./04-halo2-primer.md) introduces the
parts of PLONKish arithmetisation needed to read
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs).

## Notation

Used throughout, following the Zcash Protocol Specification.

| Symbol                                | Meaning                                |
| ------------------------------------- | -------------------------------------- |
| $\mathbb{F}_p$                        | The prime field with $p$ elements      |
| $\mathbb{G}$, $E_p$                   | A cyclic group; the curve over $\mathbb{F}_p$ |
| $[k] P$                               | Scalar multiplication of $P$ by $k$    |
| $\mathsf{Com}(m; r)$                  | Commitment to $m$ with randomness $r$  |
| $x \stackrel{\$}{\leftarrow} S$       | Sample $x$ uniformly from $S$          |
| $a \mathbin{\\|} b$                  | Byte-string concatenation              |
| $\mathsf{Sinsemilla}_D(m)$            | Sinsemilla hash of $m$ in domain $D$   |
| $\mathsf{Poseidon}(x_1, \dots, x_n)$  | Poseidon permutation output            |
| $q$, $r$                              | Base / scalar field orders             |
| $\mathsf{Extract}_{\mathbb{P}}(P)$    | $x$-coordinate extraction              |

## Pin

All GitHub source links are pinned to upstream tag
[`0.13.1`](https://github.com/zcash/orchard/releases/tag/0.13.1),
commit
[`f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669).
See [Discovery Notes](./discovery.md) for the full rationale.

## Chapter Index

The sidebar on the left lists every chapter in order. The intended
reading path is linear: Chapter 1 maps the crate, Chapter 2
unlocks the contribution loop, and Chapters 3 to 18 each pick one
subsystem and follow the seven-section skeleton.
