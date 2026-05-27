---
sidebar_position: 4
title: the orchard action circuit
---

# the orchard action circuit

## motivation

The Orchard Action circuit is the heart of the protocol. It is a
Halo 2 circuit that, given a public Anchor, a public nullifier, a
public note commitment, and a public value commitment, proves in
zero-knowledge that an input note was spent and an output note was
created consistently with the spec. Every shielded Orchard
transaction contains exactly one Halo 2 proof asserting these
constraints for all of its Actions in a batch. This chapter walks
through the witness layout, the public inputs, the high-level
constraints, and where each one lives in
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs).

## the math

A single Action has the following public inputs (the *Instance*):

$$
\mathsf{Inst} = \big(\mathsf{anchor}, \mathsf{nf}, \mathsf{rk}_X,
\mathsf{rk}_Y, \mathsf{cm}^\star_X, \mathsf{cv^{\mathsf{net}}}_X,
\mathsf{cv^{\mathsf{net}}}_Y, \mathsf{enableSpends}, \mathsf{enableOutputs}\big)
$$

and the following private witnesses (a subset; see the source for
the full list):

$$
\mathsf{Wit} = \big(\mathsf{path}, \mathsf{pos}, g_d, \mathsf{pk_d},
v_{\mathsf{old}}, \rho_{\mathsf{old}}, \psi_{\mathsf{old}},
\mathsf{rcm}_{\mathsf{old}}, \mathsf{cm}_{\mathsf{old}}, \alpha,
\mathsf{ak}, \mathsf{nk}, \mathsf{rivk}, g_d^\star, \mathsf{pk_d}^\star,
v_{\mathsf{new}}, \psi^\star, \mathsf{rcm}^\star, \mathsf{rcv}\big).
$$

The circuit enforces, among others:

1. **Merkle membership**:
   $\mathsf{MerkleCRH}^{\mathsf{depth}}_{\mathsf{path}, \mathsf{pos}}
   (\mathsf{cm}_{\mathsf{old}}) = \mathsf{anchor}$, where the
   $\mathsf{MerkleCRH}$ is the Sinsemilla-based hash defined in
   the spec.
2. **Spend authorisation**:
   $\mathsf{ak} = \mathsf{SpendAuthSig.DerivePublic}(\mathsf{ask})$
   and
   $\mathsf{rk} = \mathsf{Randomise}(\mathsf{ak}; \alpha) = \mathsf{ak} + [\alpha]\mathcal{G}$,
   where $\mathcal{G}$ is the RedPallas spend authorising generator.
3. **Nullifier derivation**:
   $\mathsf{nf} = \mathsf{Extract}_{\mathbb{P}}\big(
   \mathsf{nk} \otimes \rho_{\mathsf{old}} + \psi_{\mathsf{old}} +
   \mathsf{cm}_{\mathsf{old}}\big)$
   (the $\otimes$ operator is Poseidon-based; see the spec for the
   exact form).
4. **Note commitment integrity**:
   $\mathsf{cm}_{\mathsf{old}} = \mathsf{NoteCommit}(g_d,
   \mathsf{pk_d}, v_{\mathsf{old}}, \rho_{\mathsf{old}},
   \psi_{\mathsf{old}}; \mathsf{rcm}_{\mathsf{old}})$
   and the analogous equation for $\mathsf{cm}^\star$.
5. **Value commitment**:
   $\mathsf{cv^{\mathsf{net}}} = [v_{\mathsf{old}} - v_{\mathsf{new}}]
   \mathcal{V} + [\mathsf{rcv}]\mathcal{R}$.
6. **Spend / output enablement**: the public `enableSpends` and
   `enableOutputs` flags zero out the relevant sub-constraints when
   the Action is a "dummy" Action (used for padding).

## the implementation

The configuration of the Action circuit is the `Config` struct
declared near the top of
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs):

```rust reference title="src/circuit.rs"
https://github.com/zcash/orchard/blob/main/src/circuit.rs#L1-L4
```

The struct lists the chip configurations it bundles: the ECC chip
config (point addition / scalar mul), the Poseidon config (a Poseidon
$P_{128}^{\mathrm{Pasta}}$ sponge), the Sinsemilla configs for both
the Merkle path and the note commitments, and the Orchard-specific
chips
[`CommitIvkChip`](https://github.com/zcash/orchard/blob/main/src/circuit/commit_ivk.rs)
and
[`NoteCommitChip`](https://github.com/zcash/orchard/blob/main/src/circuit/note_commit.rs).

The instance columns (public inputs) and their layout are documented
in the module-level docstring of `circuit.rs`. The witness population
happens in `Circuit::synthesize`, which is large but reads top to
bottom: first the ECC and Sinsemilla chips are loaded, then each
constraint group is synthesised in turn.

A representative slice showing the use of `Constraints` to bind the
nullifier computation:

The high-level dataflow `prove` / `verify` uses `SingleVerifier` for
single proofs and `BatchVerifier` for batched verification:

```rust reference title="src/circuit.rs"
https://github.com/zcash/orchard/blob/main/src/circuit.rs#L8-L13
```

## specification and references

- Zcash Protocol Specification,
  [Section 4.19 (Action statement)](https://zips.z.cash/protocol/protocol.pdf)
  and Section 5.4.9.6 (the Orchard circuit).
- [Halo 2 book](https://zcash.github.io/halo2/), chapter on writing
  a circuit.
- The `circuit_description` file under
  [`src/circuit_description`](https://github.com/zcash/orchard/tree/main/src/circuit_description)
  is the pinned textual description of the circuit shape; consult it
  if you need a deterministic snapshot.

## exercises

1. In
   [`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs),
   find every occurrence of `meta.instance_column()`. Match each
   instance column to one of the public inputs listed above.
2. The circuit has a `K` constant that fixes the number of rows
   $2^K$. Find it and explain how this constant interacts with the
   maximum number of Actions in a bundle.
3. Open
   [`src/circuit/note_commit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit/note_commit.rs).
   Sketch (in prose) how the witness for the canonical encoding of
   $(g_d, \mathsf{pk_d}, v, \rho, \psi)$ is laid out across the
   Sinsemilla input bits.
4. Compare the Action statement on paper (Section 4.19 of the spec)
   with the constraints enforced by
   `Circuit::synthesize`. Identify one constraint that is enforced by
   the spec but does not appear as an explicit custom gate in the
   circuit, and explain how it is enforced indirectly (e.g. by the
   chip preconditions or by the canonical encoding lookups).
