---
sidebar_position: 17
title: exercises and study plan
---

# exercises and study plan

## motivation

The previous chapters introduced the Orchard protocol piece by
piece. This chapter proposes a six-week reading plan that ties them
together and ends with a small project. It is meant for a graduate
student new to zero-knowledge protocol implementations who can
allocate roughly eight to ten hours per week.

## the schedule

### week 1: orientation

- Read chapters [overview](./01-overview.md) and
  [pasta-curves](./02-pasta-curves.md).
- Set up a local build of `orchard` and run `cargo test --features
  circuit` end-to-end. Note which tests are slow.
- Open
  [`src/lib.rs`](https://github.com/zcash/orchard/blob/main/src/lib.rs)
  and draw a module-dependency diagram by hand.

### week 2: keys and notes

- Read chapters
  [keys-and-addresses](./07-keys-and-addresses.md) and
  [notes-nullifiers-commitments](./08-notes-nullifiers-commitments.md).
- Write a small Rust binary that derives an `ExtendedSpendingKey`,
  enumerates the first three diversified addresses, and prints them.
- Verify your output against the JSON vectors in
  [`zcash-test-vectors`](https://github.com/zcash-hackworks/zcash-test-vectors).

### week 3: encryption and the tree

- Read chapters [note-encryption](./09-note-encryption.md) and
  [merkle-tree](./10-merkle-tree.md).
- Encrypt a note to one of the addresses from week 2 and decrypt it
  again. Confirm that
  $\mathsf{enc\_ciphertext} \to \mathsf{Note}$ round-trips and that
  the recovered $\mathsf{cm}^\star$ matches.
- Insert a sequence of leaves into an
  [`incrementalmerkletree`](https://github.com/zcash/incrementalmerkletree)
  frontier; compute three Merkle authentication paths and check them
  against the recomputed root.

### week 4: halo 2 and the action circuit

- Read chapters [halo2-primer](./03-halo2-primer.md),
  [sinsemilla](./05-sinsemilla.md), [poseidon](./06-poseidon.md), and
  [action-circuit](./04-action-circuit.md).
- Generate one Orchard Action proof end-to-end (via
  [`tests/builder.rs::bundle_chain`](https://github.com/zcash/orchard/blob/main/tests/builder.rs)).
- Modify a witness to be invalid and confirm that
  `SingleVerifier::verify_proof` rejects with the expected error.

### week 5: bundles, signatures, and values

- Read chapters [bundle-and-builder](./11-bundle-and-builder.md),
  [value-commitments](./12-value-commitments.md), and
  [redpallas](./13-redpallas.md).
- Build a bundle that spends one note and creates two outputs; check
  that `value_balance` matches the difference of input and output
  sums.
- Inspect the proof bytes and the signatures using `hex` and confirm
  their sizes match what the spec predicts.

### week 6: dependencies, audits, and a small project

- Read chapters [dependencies](./14-dependencies.md),
  [test-vectors](./15-test-vectors.md), and
  [audits](./16-audits.md).
- Pick a small project from the list below and complete it.

## small projects

Choose one of the following and write up a 2-3 page note:

1. **Compact circuit visualisation**. Write a tool that consumes
   [`src/circuit_description`](https://github.com/zcash/orchard/tree/main/src/circuit_description)
   and emits a textual summary: column counts, gate counts, lookup
   table sizes, total $2^K$ rows. Compare with the Sapling Spend /
   Output circuits.
2. **Cross-protocol comparison**. Pick one piece of Orchard (e.g.
   the value commitment) and write a side-by-side comparison with
   Sapling's analogue (in
   [`sapling-crypto`](https://github.com/zcash/sapling-crypto)).
   Identify what changed and why.
3. **A new note encryption round-trip test**. Add an integration
   test under
   [`tests/`](https://github.com/zcash/orchard/blob/main/tests/) that
   exercises an edge case not currently covered (e.g. a note with
   a memo at the maximum permitted length). Verify the test runs.
4. **Audit follow-up**. Pick one finding from the
   [audits](./16-audits.md) chapter and write a one-page commentary
   explaining the finding, the fix, and one additional defence in
   depth that the code could plausibly add.

## final self-check

By the end of the course, you should be able to answer:

- What are the public inputs and the private witnesses of an Orchard
  Action, and why is each one needed?
- Why is the Action circuit a single proof per *bundle* rather than
  per *Action*?
- Which generators are used by Sinsemilla, by Poseidon, by
  RedPallas, and by the Pedersen value commitment? Where in the
  source are they defined?
- How does ZIP 32 differ from BIP 32, and why?
- Which dependencies of `orchard` would invalidate consensus if they
  changed semantics, and which would only change performance?

If a question is unclear, return to the corresponding chapter and to
the source files it points to.
