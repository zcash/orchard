---
sidebar_position: 7
title: Poseidon
description: The Poseidon permutation parameters used in Orchard and where they live in the code.
---

# Poseidon

## 1. Why This Chapter Exists

Poseidon is the algebraic hash that drives the nullifier
derivation key combinator. It is cheap inside a Halo 2 circuit
because every round is a polynomial identity. A contributor who
needs to add a Poseidon call (a new PRF, a new transcript-style
mixer) must understand the parameter choices for $P_{128}^{\mathrm{Pasta}}$
and the sponge construction. After this chapter the reader can
predict how many rows a new Poseidon call will cost.

## 2. Definitions

### Definition 2.1 (Poseidon Permutation)

A permutation
$\pi : \mathbb{F}_q^t \to \mathbb{F}_q^t$ over a prime field
$\mathbb{F}_q$, parameterised by:

- *width* $t$;
- *S-box exponent* $\alpha$ such that $\gcd(\alpha, q - 1) = 1$;
- *full rounds* $R_F$, in which the S-box is applied to every
  element;
- *partial rounds* $R_P$, in which the S-box is applied only to
  the first element;
- a sequence of round constants $\{c^{(i)}\}$ and an MDS matrix
  $M$.

### Definition 2.2 (Orchard Parameters)

Orchard uses $P_{128}^{\mathrm{Pasta}}$ with $t = 3$, $\alpha = 5$,
$R_F = 8$ full rounds, $R_P = 56$ partial rounds, targeting
128-bit security.

### Definition 2.3 (Sponge as Hash)

The sponge absorbs $t - 1 = 2$ field elements per permutation
call and squeezes one. Used as a length-2 hash:

$$
\mathsf{Poseidon}(x_1, x_2) = \pi(x_1, x_2, 0)_0.
$$

### Definition 2.4 (Orchard Nullifier PRF)

$$
\mathsf{PRF}^{\mathsf{nfOrchard}}(\mathsf{nk}, \rho) =
\mathsf{Poseidon}(\mathsf{nk}, \rho).
$$

## 3. The Code

### 3.1 The Implementation Crate

Orchard re-exports the upstream
[`halo2_poseidon`](https://crates.io/crates/halo2_poseidon) crate
under the local alias `poseidon`:

```toml reference title="Cargo.toml"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/Cargo.toml#L41-L41
```

### 3.2 The Parameter Set

`P128Pow5T3` (the $P_{128}^{\mathrm{Pasta}}$ trait) lives in
`halo2_gadgets::poseidon::primitives`. It exposes the MDS matrix
$M$ and the round-constants table.

### 3.3 Use Site

The nullifier derivation in
[`src/note/nullifier.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/nullifier.rs)
calls the Poseidon sponge with $\mathsf{nk}$ as the first input
and $\rho$ as the second. The output is then combined with the
point $\mathcal{K}$, the trapdoor $\psi$, and $\mathsf{cm}$
before $\mathsf{Extract}_{\mathbb{P}}$; see
[Chapter 9](./09-notes-nullifiers-commitments.md).

### 3.4 Inside the Circuit

`halo2_gadgets::poseidon::Pow5Chip` is the in-circuit
implementation. Its `Config` is bundled into the Action circuit
[`Config`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs).
Each Poseidon permutation costs $R_F + R_P = 64$ rounds; a
two-input hash is a single permutation call.

## 4. Failure Modes

- **Parameter drift**. Hard-coding $R_F$ or $R_P$ in two places
  invites them to drift. Orchard uses a typeclass instead, so
  parameter changes propagate. Watch for "magic" `8` or `56`
  literals in PRs.
- **Field mismatch**. Calling Poseidon with a value that should
  be a `pallas::Scalar` but is provided as a `pallas::Base`
  silently changes the output. The type signature of the
  primitive enforces the field; bypassing it via `to_base` is a
  red flag.
- **Wrong S-box exponent**. $\alpha = 5$ is a permutation only
  because $\gcd(5, q - 1) = 1$. A future field change (a new
  curve choice) breaks Poseidon if the exponent is not
  re-derived.

## 5. Spec Pointers

- [Poseidon paper](https://eprint.iacr.org/2019/458):
  the security analysis and the round-count formula.
- [Zcash Protocol Specification, Section 5.4.1.10](https://zips.z.cash/protocol/protocol.pdf):
  the Orchard parameter choice.
- [Halo 2 Book, Poseidon](https://zcash.github.io/halo2/design/gadgets/poseidon.html):
  the rationale for $\alpha = 5$ over the Pasta scalar field.
- [`halo2_poseidon`](https://crates.io/crates/halo2_poseidon):
  the implementation crate.

## 6. Exercises

1. Compute $\gcd(5, q - 1)$ for the Pallas scalar field modulus
   $q$ from
   [Chapter 3](./03-pasta-curves.md). Show that $x \mapsto x^5$
   is therefore a permutation.
2. The Poseidon paper gives the minimal $R_F + R_P$ that resists
   linear and algebraic attacks at $128$ bits. Look up the
   formula in Section 5 and apply it to $t = 3$, $\alpha = 5$.
   Compare with the Orchard choice $R_F = 8$, $R_P = 56$.
3. **Code task**. Write a five-line program that runs the
   `halo2_poseidon` sponge over two zero scalars and prints the
   output. Cross-check with the value embedded in the unit tests
   of
   [`src/note/nullifier.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/nullifier.rs).

## 7. Further Reading

- The [`halo2_poseidon` source](https://github.com/zcash/halo2/tree/main/halo2_poseidon)
  for the bit-exact constants.
- [Constant-time Poseidon](https://eprint.iacr.org/2023/107)
  for a discussion of side-channels in algebraic hashes.
