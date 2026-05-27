---
sidebar_position: 5
title: The Orchard Action Circuit
description: Public inputs, witness layout, and constraints of src/circuit.rs.
---

# The Orchard Action Circuit

## 1. Why This Chapter Exists

[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)
is 1,289 lines and concentrates more cryptographic decisions per
line than any other file in the crate. A contributor who touches
it without understanding the public inputs, the witness, or the
constraint groups will produce a soundness bug. After this chapter
the reader can match each clause of the Action statement in the
Zcash Protocol Specification to a `Constraints::with_selector`
block in this file.

## 2. Definitions

### Definition 2.1 (Public Inputs of an Action)

The instance columns of the Action circuit hold

$$
\mathsf{Inst} = (\mathsf{anchor},\, \mathsf{nf},\, \mathsf{rk}_X,\,
\mathsf{rk}_Y,\, \mathsf{cm}^\star_X,\, \mathsf{cv^{\mathsf{net}}}_X,\,
\mathsf{cv^{\mathsf{net}}}_Y,\, \mathsf{enableSpends},\,
\mathsf{enableOutputs}).
$$

### Definition 2.2 (Witness)

The private witness includes the input note path and position, the
input note's full field set, the spending authority $\mathsf{ak}$,
the re-randomiser $\alpha$, the nullifier-deriving key
$\mathsf{nk}$, $\mathsf{rivk}$, the output note's fields, and the
value commitment trapdoor $\mathsf{rcv}$.

### Definition 2.3 (Constraint Groups)

The circuit enforces six groups of constraints:

1. **Merkle membership**:
   $\mathsf{MerkleCRH}^{\mathsf{depth}}_{\mathsf{path},\mathsf{pos}}
   (\mathsf{cm}_{\mathsf{old}}) = \mathsf{anchor}$.
2. **Spend authorisation**:
   $\mathsf{ak} = [\mathsf{ask}] \mathcal{G}_{\mathsf{ak}}$ and
   $\mathsf{rk} = \mathsf{ak} + [\alpha] \mathcal{G}_{\mathsf{ak}}$.
3. **Nullifier**:
   $\mathsf{nf} = \mathsf{Extract}_{\mathbb{P}}\big(
   [\mathsf{PRF}^{\mathsf{nfOrchard}}_{\mathsf{nk}}(\rho)]\,
   \mathcal{K} + \psi + \mathsf{cm}\big)$.
4. **Note commitment integrity** for both old and new notes.
5. **Value commitment**:
   $\mathsf{cv^{\mathsf{net}}} = [v_{\mathsf{old}} - v_{\mathsf{new}}]\,
   \mathcal{V} + [\mathsf{rcv}]\, \mathcal{R}$.
6. **Action enable flags**: when `enableSpends = 0`, the spend
   subcircuit is disabled (dummy spend); when `enableOutputs = 0`,
   the output subcircuit is disabled.

## 3. The Code

### 3.1 The `Config` Struct

```rust reference title="src/circuit.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L17-L40
```

The `Config` bundles all chip configs the Action circuit uses: the
ECC chip, the Poseidon chip, two Sinsemilla configs (one per
domain), the
[`CommitIvkChip`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/commit_ivk.rs)
config, and the
[`NoteCommitChip`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/note_commit.rs)
config.

### 3.2 Synthesise

`Circuit::synthesize` runs top to bottom: load the ECC and
Sinsemilla chips, witness the Merkle path, derive `rk`, derive the
nullifier, build both note commitments, and finally build the
value commitment. Each step ends with a public-input equality
constraint against the appropriate instance column.

### 3.3 The Pinned Snapshot

[`src/circuit_description/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)
holds a textual snapshot of the column layout, gates, and lookup
tables; any change to the circuit shape must update it. The
companion
[`src/circuit_proof_test_case.bin`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_proof_test_case.bin)
is a pinned proof exercised by the unit tests; see
[Chapter 16 (Test Vectors)](./16-test-vectors.md).

## 4. Failure Modes

- **Underconstrained witness**. The most dangerous category of
  bug. The dev-mode prover does not detect it; the only defence
  is the spec-to-constraint review captured in the audit reports
  (Chapter 17). When in doubt, write a malicious witness in a unit
  test and confirm it is rejected.
- **Stale `circuit_proof_test_case.bin`**. If the proof format
  changes upstream (`halo2_proofs` version bump), the pinned
  proof must be regenerated. The reviewer will require a
  paragraph in the PR explaining why the regeneration is safe.
- **Public-input column order**. The verifier wires the instance
  columns to fields of `Instance`; swapping two columns silently
  decouples constraints from their public targets.
- **Enable-flag coverage gap**. A bug in the flag-gating logic
  was triggered in the past by a malformed dummy Action; see
  [#492](https://github.com/zcash/orchard/pull/492) for the
  identity-`rk` consensus rule that resulted.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.19](https://zips.z.cash/protocol/protocol.pdf):
  the Action statement, the authoritative list of constraints.
- [Zcash Protocol Specification, Section 5.4.9.6](https://zips.z.cash/protocol/protocol.pdf):
  the realisation of the Action statement as a Halo 2 circuit.
- [Halo 2 Book, Design](https://zcash.github.io/halo2/design.html):
  the chip patterns that the Action circuit reuses.

## 6. Exercises

1. List every call to `meta.instance_column()` in
   [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs).
   Match each one to a public input from Definition 2.1.
2. Compute (by inspection) how many rows each constraint group
   from Definition 2.3 occupies. Compare with the total $2^K$.
3. **Code task**. Modify
   [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
   to construct an Action with `enableSpends = false` (a dummy
   spend). Run the test and verify that the proof still verifies.
   Then flip `enableSpends = true` while keeping the spend
   witness empty: the prover should reject. Revert.

## 7. Further Reading

- [Orchard Book, Action circuit](https://zcash.github.io/orchard/design/circuit.html):
  the higher-level walkthrough maintained by EC Co. engineers.
- The audit reports cited in
  [Chapter 17 (Audits)](./17-audits.md) include line-by-line
  reviews of the circuit.
