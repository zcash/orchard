---
sidebar_position: 12
title: value commitments and the value pool
---

# value commitments and the value pool

## motivation

Orchard hides per-note values inside note commitments. To check that
a transaction does not create or destroy value, the protocol cannot
just sum the cleartext values; it sums *commitments* and verifies a
balance equation on the resulting curve point. The commitment used
is a standard Pedersen commitment over Pallas. This chapter walks
through the value types in
[`src/value.rs`](https://github.com/zcash/orchard/blob/main/src/value.rs)
and the balance equation that ties together all Actions in a bundle.

## the math

Fix two generators
$\mathcal{V}, \mathcal{R} \in E_p \setminus \{\mathcal{O}\}$,
the *value commitment value base* and the *value commitment
randomness base*. For each Action $i$ with old value
$v_i^{\mathsf{old}}$ and new value $v_i^{\mathsf{new}}$, the prover
samples $\mathsf{rcv}_i \stackrel{\$}{\leftarrow} \mathbb{F}_q$ and
forms the *net value commitment*

$$
\mathsf{cv}^{\mathsf{net}}_i = [v_i^{\mathsf{old}} - v_i^{\mathsf{new}}] \, \mathcal{V}
\;+\; [\mathsf{rcv}_i] \, \mathcal{R}.
$$

The bundle as a whole declares a public scalar
$\mathsf{value\_balance}$ (the net amount entering or leaving the
shielded pool). Pedersen-homomorphism gives:

$$
\sum_i \mathsf{cv}^{\mathsf{net}}_i \;=\; \Big[\sum_i (v_i^{\mathsf{old}} - v_i^{\mathsf{new}})\Big] \mathcal{V}
\;+\; \Big[\sum_i \mathsf{rcv}_i\Big] \mathcal{R}.
$$

The protocol requires
$\sum_i (v_i^{\mathsf{old}} - v_i^{\mathsf{new}}) = \mathsf{value\_balance}$.
The binding signature key is then the curve point

$$
\mathsf{bvk} = \sum_i \mathsf{cv}^{\mathsf{net}}_i \;-\; [\mathsf{value\_balance}] \mathcal{V}
\;=\; \Big[\sum_i \mathsf{rcv}_i\Big] \mathcal{R},
$$

which is a public key whose discrete log to base $\mathcal{R}$ is
exactly $\sum_i \mathsf{rcv}_i$. The prover holds that discrete log
and signs the SIGHASH; the verifier checks the signature against
$\mathsf{bvk}$. If $\sum_i \mathsf{cv}^{\mathsf{net}}_i$ ever differs
from $[\mathsf{value\_balance}] \mathcal{V}$ by anything other than
a multiple of $\mathcal{R}$, the signature cannot verify (it is
"binding" to a correct value sum).

## the implementation

Value-related types live in
[`src/value.rs`](https://github.com/zcash/orchard/blob/main/src/value.rs):

```rust reference title="src/value.rs"
https://github.com/zcash/orchard/blob/main/src/value.rs#L1-L40
```

The three types are:

- `NoteValue` is an unsigned 64-bit integer with maximum
  $\mathsf{MAX\_NOTE\_VALUE} = 2^{64} - 1$.
- `ValueSum` is a signed 64-bit value (range
  $\mathsf{VALUE\_SUM\_RANGE}$). The docstring at the top of
  `value.rs` warns that an `i64` is *not* a signed 64-bit integer:
  it can represent values in $[-(2^{63}), 2^{63})$, which is one
  bit shy of a true signed 64-bit type.
- The user-defined `valueBalanceOrchard` type parameter on `Bundle`
  must be convertible from `i64` and is bounded by 63 bits in the
  Zcash instantiation (because Zcash's `MAX_MONEY` fits in 51 bits).

The `ValueCommitment` newtype wraps a `pallas::Point`. The
`ValueCommitTrapdoor` newtype wraps $\mathsf{rcv}$ (a
`pallas::Scalar`). The binding signature key is computed by the
`commit_v` helper plus a final subtraction of
$[\mathsf{value\_balance}] \mathcal{V}$.

The bases $\mathcal{V}$ and $\mathcal{R}$ are defined as fixed bases
in
[`src/constants/fixed_bases.rs`](https://github.com/zcash/orchard/blob/main/src/constants/fixed_bases.rs)
under the identifiers `VALUE_COMMITMENT_V_BYTES` and
`VALUE_COMMITMENT_R_BYTES`.

## specification and references

- Zcash Protocol Specification,
  [Section 5.4.8.3 (Homomorphic Pedersen commitments)](https://zips.z.cash/protocol/protocol.pdf)
  and Section 4.13 (Balance).
- [ZIP 224](https://zips.z.cash/zip-0224) for Orchard-specific
  parameters.

## exercises

1. Suppose $\mathsf{value\_balance} = 0$ but
   $v^{\mathsf{old}}_1 - v^{\mathsf{new}}_1 = 1$ and
   $v^{\mathsf{old}}_2 - v^{\mathsf{new}}_2 = -1$. Compute $\mathsf{bvk}$
   symbolically and confirm that the equation still holds.
2. Why does the protocol use *two* generators rather than one? Show
   that a single-generator Pedersen commitment would be perfectly
   binding but not hiding.
3. Find the test
   [`tests/builder.rs::bundle_chain`](https://github.com/zcash/orchard/blob/main/tests/builder.rs)
   and read the assertions on `bundle.value_balance()`. What invariant
   does it verify across a chain of bundles?
