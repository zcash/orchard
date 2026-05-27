---
sidebar_position: 15
title: test vectors and cross-implementation checks
---

# test vectors and cross-implementation checks

## motivation

A specification is only as good as the implementations that agree on
it. The `orchard` crate pins a battery of test vectors that match
those used by the Zcash Foundation's other implementations (zebra,
zcashd, and `librustzcash`). These vectors check key derivation,
note commitments, nullifiers, note encryption round-trips, Merkle
roots, and the circuit-level prover and verifier. This chapter
explains where the vectors live and how they are consumed.

## what is tested

The
[`src/test_vectors.rs`](https://github.com/zcash/orchard/blob/main/src/test_vectors.rs)
module pulls in JSON-encoded test vectors from the
[`zcash-test-vectors`](https://github.com/zcash-hackworks/zcash-test-vectors)
repository. The vectors covered include:

- ZIP 32 key derivation for Orchard: spending key -> extended spending
  key -> child keys.
- Address derivation: $(\mathsf{ivk}, d) \to \mathsf{pk_d} \to
  \mathsf{addr}$.
- Note commitments: canonical encoding -> Sinsemilla output.
- Nullifiers: $(\mathsf{nk}, \rho, \psi, \mathsf{cm}) \to
  \mathsf{nf}$.
- The Merkle tree empty root sequence.
- Note encryption round-trips for both compact and full ciphertexts.
- The circuit proof: a *pinned proof binary*
  [`src/circuit_proof_test_case.bin`](https://github.com/zcash/orchard/blob/main/src/circuit_proof_test_case.bin)
  is checked against the current prover output to detect any
  unintended change to the proof bytes.

The `circuit_description` directory holds a textual snapshot of the
arithmetic circuit shape (column counts, gates, lookups). Any change
to the circuit must update both files.

## the implementation

The pinned proof test sits alongside
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs).
The runner deserialises the proof bytes, verifies with
`SingleVerifier`, and re-proves to compare. A change in
`halo2_proofs` that alters the transcript byte layout will fail this
test, which catches accidental breaking changes in dependencies.

The integration tests in
[`tests/builder.rs`](https://github.com/zcash/orchard/blob/main/tests/builder.rs)
exercise the full happy path: build a bundle, prove, sign, verify,
recover the note from the ciphertext, advance the tree, and chain a
second bundle that spends the new note.

## specification and references

- [`zcash-hackworks/zcash-test-vectors`](https://github.com/zcash-hackworks/zcash-test-vectors)
  for the source of truth.
- [`librustzcash`](https://github.com/zcash/librustzcash) for the
  parallel Rust client; comparing the two test runs catches divergences.
- [`zebra`](https://github.com/ZcashFoundation/zebra) for the
  independent Rust full node.

## exercises

1. Run `cargo test --features circuit` and identify the longest
   running test. Where is it defined?
2. Open the
   [`tests/builder.rs::bundle_chain`](https://github.com/zcash/orchard/blob/main/tests/builder.rs)
   test and explain how it sequences two bundles. Which fields of
   the second `Builder` come from the first bundle's output?
3. Delete one byte of
   [`src/circuit_proof_test_case.bin`](https://github.com/zcash/orchard/blob/main/src/circuit_proof_test_case.bin)
   (don't commit it). What test fails and what is the error message?
   Restore the file when done.
