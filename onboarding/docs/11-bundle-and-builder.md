---
sidebar_position: 11
title: the bundle and the builder
---

# the bundle and the builder

## motivation

An Orchard transaction can contain many Actions, but the Halo 2 proof
attached to the transaction proves the validity of *all* of them
together. The data structure that bundles Actions, the proof, the
binding signature, and per-Action spend-authorising signatures is the
`Bundle`. The wallet-side construction of a `Bundle` is the
responsibility of the `Builder`, which manages randomness, value
balance, dummy outputs, and the signing flow. This chapter describes
the lifecycle of an Action and the typestate of the `Builder`.

## the math

A `Bundle` is the tuple

$$
B = \big(\{A_i\}_{i=1}^N,\;\; \pi,\;\; \sigma^{\mathsf{bind}},\;\; \{\sigma_i^{\mathsf{auth}}\}_{i=1}^N,\;\; \mathsf{value\_balance}\big),
$$

where:

- $A_i$ is an Action with public inputs
  $(\mathsf{anchor}_i, \mathsf{nf}_i, \mathsf{rk}_i, \mathsf{cm}^\star_i, \mathsf{cv}^{\mathsf{net}}_i, \mathsf{enableSpends}_i, \mathsf{enableOutputs}_i)$,
  plus the encrypted ciphertexts and ephemeral keys.
- $\pi$ is a single Halo 2 proof asserting the conjunction of all
  $N$ Action statements.
- $\sigma^{\mathsf{bind}}$ is a RedPallas signature on the binding
  value commitment $[\mathsf{value\_balance}] \mathcal{V} - \sum_i \mathsf{cv}_i^{\mathsf{net}}$.
- $\sigma_i^{\mathsf{auth}}$ is a RedPallas signature with key
  $\mathsf{rk}_i$ over the SIGHASH of the surrounding transaction.

A `Bundle` is constructed in three phases:
$\texttt{Unauthorized} \to \texttt{Unproven} \to \texttt{Authorized}$,
where each transition adds the relevant signatures or proof. The
`Authorized` form is what is broadcast.

## the implementation

The `Bundle` type is in
[`src/bundle.rs`](https://github.com/zcash/orchard/blob/main/src/bundle.rs):

```rust reference title="src/bundle.rs"
https://github.com/zcash/orchard/blob/main/src/bundle.rs#L1-L20
```

The typestate is encoded via the `Authorization` trait; the three
phases are `InProgress`, `Unauthorized`, and `Authorized`. The
methods `Bundle::create_proof`, `Bundle::prepare`, and
`Bundle::sign` are gated on the appropriate state and produce the
next state.

The `Builder` is in
[`src/builder.rs`](https://github.com/zcash/orchard/blob/main/src/builder.rs):

```rust reference title="src/builder.rs"
https://github.com/zcash/orchard/blob/main/src/builder.rs#L1-L40
```

The builder API is roughly:

1. `Builder::new(flags, anchor)` initialises a builder with the
   Action flags (spends and outputs enablement) and the anchor that
   every spent note must commit under.
2. `Builder::add_spend(fvk, note, merkle_path)` queues an input
   note.
3. `Builder::add_output(ovk, addr, value, memo)` queues an output
   note.
4. `Builder::build(rng)` shuffles the inputs and outputs, pads with
   dummy Actions to a power-of-two count if required, derives the
   per-Action randomisers, and returns an `Unauthorized` `Bundle`.

Dummy Actions are full Actions whose `enableSpends` / `enableOutputs`
flag is zero; the circuit's constraints are dropped on those rows.
Dummies prevent the size of the bundle from leaking the actual
number of spends and outputs.

A representative use site for `BatchVerifier` from
`src/bundle/batch.rs`:

The `commitments` submodule houses the binding signature key
derivation:

```rust reference title="src/bundle/commitments.rs"
https://github.com/zcash/orchard/blob/main/src/bundle/commitments.rs#L1-L20
```

## specification and references

- Zcash Protocol Specification,
  [Section 4.7 (Sending notes)](https://zips.z.cash/protocol/protocol.pdf)
  and Section 4.20 (Action descriptions).
- [ZIP 244](https://zips.z.cash/zip-0244): transaction identifier
  digest, which is the SIGHASH for $\sigma^{\mathsf{auth}}$.
- [`reddsa`](https://github.com/ZcashFoundation/reddsa): the
  external implementation of RedDSA used through the
  [`primitives::redpallas`](https://github.com/zcash/orchard/blob/main/src/primitives/redpallas.rs)
  facade.

## exercises

1. Trace `Builder::build`. List, in order, every randomness sampled
   per Action and which scalar / base field it lives in.
2. The builder pads with dummy Actions to a *power of two*, not just
   a fixed count. Why?
3. The `Bundle` type is parametrised by an `Authorization`. Find the
   place where `Authorized` is the only type that allows access to
   the proof bytes. What invariant does this enforce?
