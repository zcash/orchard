---
sidebar_position: 12
title: The Bundle and the Builder
description: Assembly of Actions into a Bundle and the Unauthorized -> Authorized typestate.
---

# The Bundle and the Builder

## 1. Why This Chapter Exists

[`src/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs)
and
[`src/bundle.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs)
are the most likely files for a contributor to touch when adding
wallet-side behaviour. Together they own the randomness, the
dummy-action padding, the typestate transitions, and the batch
verification entry point. After this chapter the reader can
predict in what order each randomiser is sampled and at which
typestate each signature is attached.

## 2. Definitions

### Definition 2.1 (Action)

An Action $A$ contains the public Action statement of
[Chapter 5](./05-action-circuit.md) plus the encrypted ciphertext
data
$(\mathsf{cv^{\mathsf{net}}}, \mathsf{nf}, \mathsf{rk}, \mathsf{cm}^\star,
\mathsf{epk}, C^{\mathsf{enc}}, C^{\mathsf{out}},
\mathsf{enableSpends}, \mathsf{enableOutputs})$.

### Definition 2.2 (Bundle)

$$
B = \big(\{A_i\}_{i=1}^N,\, \pi,\, \sigma^{\mathsf{bind}},\, \{\sigma_i^{\mathsf{auth}}\}_{i=1}^N,\, \mathsf{value\_balance}\big),
$$

where $\pi$ is the single Halo 2 proof asserting the conjunction
of all $A_i$.

### Definition 2.3 (Typestate)

The bundle progresses
$\texttt{InProgress} \to \texttt{Unauthorized} \to \texttt{Authorized}$
through the `Authorization` trait. Each transition adds the
proof or the signatures.

### Definition 2.4 (Padding)

The builder pads the Action list with dummy Actions until $N$
matches a target. Dummies prevent the size of the bundle from
leaking the count of real spends and outputs.

## 3. The Code

### 3.1 The `Bundle` Type

```rust reference title="src/bundle.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs#L161-L176
```

`Bundle<T, V>` is generic over the authorisation typestate `T`
(which implements the `Authorization` trait declared just above
this block) and the user-defined `valueBalanceOrchard` type `V`.
The public methods are gated on `T`; only the `Authorized`
typestate exposes the proof bytes and the per-Action signatures.

### 3.2 The `Builder`

```rust reference title="src/builder.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs#L541-L562
```

`Builder::new(bundle_type, anchor)` opens an empty builder
parameterised by a `BundleType` (transactional or coinbase, with
spends/outputs enablement flags) and the Merkle root every spend
must commit under. `add_spend` and `add_output` queue inputs and
outputs; `build(rng)` shuffles, pads with dummies, and produces
an `Unauthorized` `Bundle` ready for proving.

### 3.3 The Signing Flow

1. `Bundle::<InProgress, V>::create_proof(rng, &pk)` runs the
   prover and returns `Bundle<Unauthorized<...>, V>`.
2. `Bundle::<Unauthorized<...>, V>::prepare(rng, sighash)` derives
   the SIGHASH-bound signing material.
3. Per-Action `apply_signatures(&[ask])` produces
   `Authorized<V>`.

### 3.4 Batch Verification

[`src/bundle/batch.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle/batch.rs)
batches Halo 2 proof checks and RedPallas signature checks across
many bundles. Open issue
[#497](https://github.com/zcash/orchard/issues/497) tracks
returning a structured error from `add_bundle`.

### 3.5 SIGHASH

[`src/bundle/commitments.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle/commitments.rs)
implements the per-Bundle digests used by the binding signature
and the spend authorising signatures (per
[ZIP 244](https://zips.z.cash/zip-0244)).

## 4. Failure Modes

- **Order-dependent randomness**. The builder shuffles the
  Action list before sampling per-Action randomisers. A change
  that samples before shuffling exposes the order of inputs.
- **Mis-paired flags**. `enableSpends` and `enableOutputs` must
  cover both the witness and the public flag values. Setting
  only one side produces a valid-looking proof that is rejected.
- **Premature signature access**. The typestate prevents
  reading signatures from an `Unauthorized` bundle. PRs that
  introduce a public accessor must enforce the same gate.
- **Power-of-two regression**. The padding count must be a
  power of two (see code comments). A change that bumps to a
  non-power-of-two breaks the assumption used by some
  verifiers' batch sizing.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.7 and 4.20](https://zips.z.cash/protocol/protocol.pdf):
  Sending notes; Action descriptions.
- [ZIP 244](https://zips.z.cash/zip-0244):
  transaction identifier digest.
- [`reddsa`](https://github.com/ZcashFoundation/reddsa):
  the upstream RedDSA implementation; see
  [Chapter 14](./14-redpallas.md).

## 6. Exercises

1. Read `Builder::build`. List, in order, every randomness
   sampled per Action and which field (base or scalar) it
   inhabits.
2. The padding count is a power of two. Find the constant and
   the calling site. Why a power of two specifically?
3. **Code task**. In a unit test, build a `Bundle` with one
   spend and zero outputs, then assert that the produced bundle
   has exactly two Actions (one real, one dummy). Run
   `cargo test --lib builder::`.

## 7. Further Reading

- [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs):
  the canonical integration scenario.
- The
  [Orchard Book, Sending and receiving](https://zcash.github.io/orchard/user/keys.html)
  high-level walkthrough.
