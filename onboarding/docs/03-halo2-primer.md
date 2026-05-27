---
sidebar_position: 3
title: halo 2 primer
---

# halo 2 primer

## motivation

The Orchard Action circuit is written in
[Halo 2](https://github.com/zcash/halo2). The reader who has only
seen R1CS-based systems (Groth16, Marlin) needs a brief tour of
PLONKish arithmetisation, custom gates, and lookups before opening
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs).
This chapter does not re-prove the soundness of Halo 2; it gives the
notation, the moving parts, and the right place to look up the rest.

## the math

### plonkish arithmetisation

A Halo 2 circuit is a rectangular table with $n$ rows and a fixed
number of columns. Columns are partitioned into:

- *advice* columns $A_0, \dots, A_{a-1}$, populated by the prover;
- *fixed* columns $F_0, \dots, F_{f-1}$, populated by the verifier
  (they are part of the trusted preprocessing);
- *instance* columns $I_0, \dots, I_{i-1}$, public inputs;
- *selector* columns $S_0, \dots, S_{s-1}$, fixed columns that gate
  the application of custom constraints.

A *custom gate* is a multivariate polynomial
$g \in \mathbb{F}_q[X_0, \dots, X_{m-1}]$ together with a selector
$S_g$. The verifier accepts the proof only if, for every row $r$,

$$
S_g(r) \cdot g\big(c_0(r), c_1(r), \dots, c_{m-1}(r)\big) = 0,
$$

where each $c_j$ is one of the advice / fixed columns possibly
offset by a rotation (e.g. $A_0(r-1)$, $A_1(r)$, $A_2(r+1)$).
Rotations let gates couple adjacent rows.

### lookups

A *lookup argument* enforces that, for every row $r$, the tuple
$\big(c_0(r), \dots, c_{m-1}(r)\big)$ appears as a row of a fixed
table $T \subset \mathbb{F}_q^m$. Lookups are how Halo 2 encodes
range checks and large-table operations like Sinsemilla's 10-bit
windowed multiplication.

### polynomial commitments and ipa

Every column is committed using the inner-product argument (IPA)
over Vesta. The proving key, the verification key, and the proof
are all sets of group elements in $E_q$ together with field elements.
Because IPA needs no trusted setup, the entire system is a Halo
proof.

### the transcript

Halo 2 uses a Blake2b-based Fiat-Shamir transcript. Each commitment
and challenge sent during the protocol is absorbed in order. The
choice of `Blake2b` (not Poseidon) for the outer transcript is
deliberate: the outer transcript is *not* recursive in Orchard.

## the implementation

The Orchard crate consumes Halo 2 through the
[`halo2_proofs`](https://github.com/zcash/halo2) crate. The Action
circuit declares its column layout via `Config` structs in
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs)
and its gadgets in
[`src/circuit/gadget.rs`](https://github.com/zcash/orchard/blob/main/src/circuit/gadget.rs).

A representative import block showing how PLONKish primitives surface
in `orchard`:

```rust reference title="src/circuit.rs"
https://github.com/zcash/orchard/blob/main/src/circuit.rs#L5-L15
```

`Advice`, `Selector`, `Constraints`, `Expression`, `Instance`, and
`SingleVerifier` are direct re-exports from `halo2_proofs::plonk`.
`Blake2bRead` / `Blake2bWrite` come from
`halo2_proofs::transcript` and implement the Fiat-Shamir transcript.

### the chip / gadget pattern

Halo 2 reusable components are organised as *chips* (a `Config` plus
the routines that constrain a region of the table) and *gadgets*
(higher-level constructions composed of one or more chips). The
Orchard circuit pulls in the
[`halo2_gadgets`](https://github.com/zcash/halo2) ecc and Sinsemilla
chips, and adds Orchard-specific chips in
[`src/circuit/note_commit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit/note_commit.rs)
and
[`src/circuit/commit_ivk.rs`](https://github.com/zcash/orchard/blob/main/src/circuit/commit_ivk.rs).

## specification and references

- [Halo 2 book](https://zcash.github.io/halo2/): the canonical
  introduction to the protocol and its gadgets.
- [PLONK paper](https://eprint.iacr.org/2019/953).
- [Halo paper](https://eprint.iacr.org/2019/1021).
- [Halo 2 source](https://github.com/zcash/halo2).
- Zcash Protocol Specification, Section 5.4.9.6 "Halo 2".

## exercises

1. In `src/circuit.rs`, list every selector column declared in the
   Action circuit `Config`. How many custom gates are gated by each?
2. A lookup table is declared in
   [`src/constants/sinsemilla.rs`](https://github.com/zcash/orchard/blob/main/src/constants/sinsemilla.rs).
   What does it contain and how is it used?
3. The proof bytes go through `Blake2bWrite::init(...)`. Why is a
   transcript domain separator needed, and what string does Orchard
   use? (Hint: search for `b"Halo2-Transcript"` in
   [`zcash/halo2`](https://github.com/zcash/halo2).)
