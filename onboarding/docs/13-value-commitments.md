---
sidebar_position: 13
title: Value Commitments and the Value Pool
description: Pedersen value commitments, the binding signature key, and the balance equation.
---

# Value Commitments and the Value Pool

## 1. Why This Chapter Exists

Orchard hides note values inside Sinsemilla note commitments and
reconciles them across a bundle with a Pedersen value commitment.
The balance equation is enforced by a RedPallas binding signature
on the difference of the sum of commitments and the declared
`value_balance`. After this chapter the reader can derive the
binding signature key on paper and locate the bases $\mathcal{V}$
and $\mathcal{R}$ in the code.

## 2. Definitions

### Definition 2.1 (Value Commitment Bases)

Two fixed generators $\mathcal{V}, \mathcal{R} \in E_p \setminus
\{\mathcal{O}\}$, derived by hashing-to-curve under distinct
domain strings.

### Definition 2.2 (Per-Action Net Commitment)

For Action $i$ with input value $v_i^{\mathsf{old}}$, output value
$v_i^{\mathsf{new}}$, and trapdoor $\mathsf{rcv}_i
\stackrel{\$}{\leftarrow} \mathbb{F}_q$:

$$
\mathsf{cv}^{\mathsf{net}}_i =
[v_i^{\mathsf{old}} - v_i^{\mathsf{new}}]\, \mathcal{V} \;+\;
[\mathsf{rcv}_i]\, \mathcal{R}.
$$

### Definition 2.3 (Binding Signature Key)

$$
\mathsf{bvk} =
\sum_i \mathsf{cv}^{\mathsf{net}}_i \;-\;
[\mathsf{value\_balance}]\, \mathcal{V}.
$$

If the prover's value sum matches $\mathsf{value\_balance}$,
$\mathsf{bvk} = \big[\sum_i \mathsf{rcv}_i\big]\, \mathcal{R}$,
which the prover can sign for. Any mismatch breaks knowledge of
the discrete log relative to $\mathcal{R}$.

### Invariant 2.4 (`NoteValue` Range)

`NoteValue` is an unsigned 64-bit integer; `ValueSum` is a signed
64-bit value (in the Rust sense, so range
$[-2^{63}, 2^{63})$). The `valueBalanceOrchard` type parameter on
`Bundle` is user-defined; the Zcash instantiation restricts it to
51 bits.

## 3. The Code

### 3.1 Value Types

```rust reference title="src/value.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/value.rs#L1-L40
```

`NoteValue`, `ValueSum`, `ValueCommitTrapdoor`, and
`ValueCommitment` are the four value-domain types. The module
header documents the i64-vs-63-bit caveat in detail.

### 3.2 Constants

The bases $\mathcal{V}$ and $\mathcal{R}$ are declared as fixed
bases in
[`src/constants/fixed_bases.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/constants/fixed_bases.rs).
Recent refactors collapsed `OrchardFixedBases` to a unit struct
([PR #496](https://github.com/zcash/orchard/pull/496));
contributors should expect the API surface to change here.

### 3.3 Binding Signature Wiring

[`src/bundle/commitments.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle/commitments.rs)
implements the digests that the binding signature signs.
[`src/primitives/redpallas.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/primitives/redpallas.rs)
exposes the
`Binding` marker that selects $\mathcal{R}$ as the base; see
[Chapter 14](./14-redpallas.md).

## 4. Failure Modes

- **Wrong base for the binding signature**. Confusing $\mathcal{V}$
  and $\mathcal{R}$ produces a verifier that accepts an arbitrary
  value imbalance; the type marker `Binding` exists exactly to
  prevent this confusion.
- **Identity `cv^net_i`**. If
  $[v^{\mathsf{old}} - v^{\mathsf{new}}] \mathcal{V} +
  [\mathsf{rcv}] \mathcal{R} = \mathcal{O}$, the prover has no
  discrete-log knowledge of $\mathsf{bvk}$. Sampling fresh
  $\mathsf{rcv}$ per Action keeps the probability negligible.
- **Pseudo-random base reuse**. A future contributor must not
  reuse $\mathcal{V}$ for another commitment scheme; doing so
  would compute a commitment that opens to the wrong subset.
- **NoteValue vs i64 confusion**. The `value.rs` header
  reiterates that `i64` represents only 63 bits in this context.
  PRs that cast a `NoteValue` (`u64`) to `i64` without checking
  the upper bit are broken.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.13](https://zips.z.cash/protocol/protocol.pdf):
  Balance.
- [Zcash Protocol Specification, Section 5.4.8.3](https://zips.z.cash/protocol/protocol.pdf):
  Homomorphic Pedersen commitments.
- [ZIP 224](https://zips.z.cash/zip-0224):
  Orchard-specific value commitment parameters.

## 6. Exercises

1. Take a bundle with two Actions where
   $v_1^{\mathsf{old}} - v_1^{\mathsf{new}} = +1$ and
   $v_2^{\mathsf{old}} - v_2^{\mathsf{new}} = -1$. With
   `value_balance = 0`, compute $\mathsf{bvk}$ symbolically and
   confirm the dependence on the $\mathsf{rcv}_i$'s only.
2. Locate the unit test for `ValueSum::overflow_*`. What edge
   case does it pin down?
3. **Code task**. Patch `tests/builder.rs` so the bundle's
   declared `value_balance` is off by one from the true sum.
   Confirm that `Bundle::verify` returns `Err` and identify the
   `redpallas::Binding` signature failure as the proximate
   cause.

## 7. Further Reading

- [Maller et al., Sonic](https://eprint.iacr.org/2019/099) on the
  Pedersen-commitment style of value reconciliation used here.
- The `src/constants/fixed_bases.rs` commit history
  documents the generator derivation.
