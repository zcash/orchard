---
sidebar_position: 20
title: Six-Week Study Plan
description: A week-by-week reading plan that converges on a real first PR.
---

# Six-Week Study Plan

## 1. Why This Chapter Exists

The previous chapters cover the surface of `zcash/orchard`
non-linearly. This chapter weaves them into a six-week plan that
ends with the reader opening a small PR. The plan assumes eight
to ten hours per week.

## 2. Definitions

### Definition 2.1 (Course Output)

The deliverable at the end of week six is a pull request against
[`zcash/orchard`](https://github.com/zcash/orchard) that compiles,
passes CI, and addresses a real issue (one of the candidates in
[Chapter 2](./02-build-test-contribute.md)).

## 3. The Code

### 3.1 Week 1: Orientation

- Read [Chapter 1](./01-crate-map.md) and
  [Chapter 2](./02-build-test-contribute.md).
- Build the crate at the pin
  ([`f8915bc`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669))
  and run `cargo test`.
- Note which tests are slow. Run them in `--release`.
- Open [`src/lib.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/lib.rs)
  and draw the module-dependency graph by hand.

### 3.2 Week 2: Curves and Halo 2

- Read [Chapter 3](./03-pasta-curves.md) and
  [Chapter 4](./04-halo2-primer.md).
- Write a scratch binary that creates a Pallas point and a
  Vesta point, multiplies each by a sampled scalar, and prints
  the result. Type-check the binary.

### 3.3 Week 3: Algebraic Hashes

- Read [Chapter 6](./06-sinsemilla.md) and
  [Chapter 7](./07-poseidon.md).
- Run `cargo bench --bench small` and identify which
  computation dominates the bench.
- Add a unit test in `src/note/nullifier.rs` that exercises an
  edge case (zero `nk`, zero `rho`); commit nothing.

### 3.4 Week 4: Keys, Notes, Tree

- Read [Chapter 8](./08-keys-and-addresses.md),
  [Chapter 9](./09-notes-nullifiers-commitments.md),
  [Chapter 10](./10-note-encryption.md), and
  [Chapter 11](./11-merkle-tree.md).
- Derive five diversified addresses from a seed and confirm
  uniqueness in a scratch test.
- Insert ten leaves into an
  `incrementalmerkletree` frontier and recompute the root.

### 3.5 Week 5: Bundle, Value, and Signature

- Read [Chapter 5](./05-action-circuit.md),
  [Chapter 12](./12-bundle-and-builder.md),
  [Chapter 13](./13-value-commitments.md), and
  [Chapter 14](./14-redpallas.md).
- Run `cargo test --release tests::builder` and trace the test.
- Modify one byte of the witness in
  [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
  and confirm the proof system rejects.

### 3.6 Week 6: Composition, Audits, and the First PR

- Read [Chapter 18](./18-shielded-transfers.md): how Actions
  compose into a single bundle-level state transition; what an
  on-chain observer sees; replay protection.
- Read [Chapter 15](./15-dependencies.md),
  [Chapter 16](./16-test-vectors.md), and
  [Chapter 17](./17-audits.md).
- Pick one open issue from
  [Chapter 2](./02-build-test-contribute.md#37-a-real-first-pr).
- Write a one-paragraph plan, post it as a comment on the issue,
  wait for a maintainer signal, then open the PR.

### 3.7 Optional Week 7: Research Frontier

For readers who want to contribute at the protocol level, not
just at the API level. Skip if the goal is purely engineering
contributions.

- Read [Chapter 19](./19-research-frontier.md) for the open
  questions and improvement vectors.
- Pick one of: a proof-system swap (KZG, Nova, HyperPlonk), a
  circuit shape change (a smaller `K`, a unified
  Sinsemilla/Poseidon chip), or a privacy primitive
  (post-quantum hybrid KEM, padding distribution).
- Sketch a one-page proposal citing the relevant paper(s) from
  Chapter 19 Section 3.7. Post it as an
  [issue on `zcash/zips`](https://github.com/zcash/zips/issues)
  for protocol-level change or
  [`zcash/orchard`](https://github.com/zcash/orchard/issues) for
  crate-level change.

### 3.8 An Alternative: The Researcher Track

A cryptography researcher can read in a different order. The
suggested track is:

1. Start with
   [References Index](./references.md) and
   [Chapter 19](./19-research-frontier.md), Section 3.7 for the
   reading list.
2. Read the [Halo 2 Book](https://zcash.github.io/halo2/) in
   full; come back to
   [Chapter 4](./04-halo2-primer.md) only as a navigation aid.
3. Read
   [Chapter 5 Sections 3.5 to 3.8](./05-action-circuit.md#35-differences-from-sapling-at-the-circuit-level)
   for the design trade-offs and the recursion status.
4. Read
   [Chapter 18](./18-shielded-transfers.md) for the security
   model the proof must support.
5. Use [Chapter 17](./17-audits.md) and
   [Chapter 19](./19-research-frontier.md) as gateways into the
   open audit reports and the open research questions.

## 4. Failure Modes

- **Bypass weeks**. Each week assumes the previous weeks'
  exercises landed. The build test in week 1 catches a stale
  toolchain; skipping it produces hard-to-debug failures later.
- **Pick a too-large issue**. The candidate first PRs were
  chosen because they are scoped to one or two files. A "fix
  all clippy warnings" PR is a different kind of work and
  should be the second PR, not the first.
- **Skip the maintainer signal**. Open issues with old
  threads (`S-waiting-on-review`) may be stalled for reasons
  not visible in the thread. A comment on the issue before
  opening the PR avoids duplicate work.

## 5. Spec Pointers

- [`CHANGELOG.md`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/CHANGELOG.md):
  read it whole before week 6.
- [Issue tracker](https://github.com/zcash/orchard/issues):
  re-read the top of the list each week; new issues land
  regularly.

## 6. Exercises

The end-of-course test. The reader should be able to answer
each in a paragraph.

1. What are the public inputs and the private witness of an
   Orchard Action, and why is each one needed?
2. Why is the Action circuit a single proof per *bundle*, not
   per *Action*?
3. Which generators are used by Sinsemilla, Poseidon,
   RedPallas (per flavour), and the Pedersen value commitment?
   Where in the source are they defined?
4. How does ZIP 32 for Orchard differ from BIP 32, and why?
5. Which dependencies of `orchard` would invalidate consensus
   if they changed semantics, and which would only change
   performance?

If a question is unclear, return to the corresponding chapter
and re-read the file it points at.

## 7. Further Reading

- The
  [Orchard Book](https://zcash.github.io/orchard/) for an
  alternative high-level walkthrough.
- The
  [Halo 2 Book](https://zcash.github.io/halo2/) for the proof
  system at depth.
- The [Zcash blog](https://electriccoin.co/blog/) for the
  ongoing protocol decisions that affect Orchard.
