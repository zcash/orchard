---
sidebar_position: 3
title: The Pasta Cycle of Curves
description: Pallas and Vesta, the 2-cycle that underlies the Orchard Halo 2 circuit.
---

# The Pasta Cycle of Curves

## 1. Why This Chapter Exists

Every field operation in Orchard happens in one of two prime fields,
and every curve operation on one of two curves. Picking the wrong
field on a single line silently corrupts the proof; the type system
catches this only because `pasta_curves` distinguishes
`pallas::Base` from `pallas::Scalar` at the type level. After this
chapter the reader can read the type signatures of
[`src/spec.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/spec.rs)
and predict which field each variable lives in.

## 2. Definitions

### Definition 2.1 (Cycle of Curves)

A pair of elliptic curves $E_p / \mathbb{F}_p$ and
$E_q / \mathbb{F}_q$ over prime fields is a 2-cycle if

$$
\#E_p(\mathbb{F}_p) = q \quad \text{and} \quad \#E_q(\mathbb{F}_q) = p.
$$

The base field of one curve is then the scalar field of the other.

### Definition 2.2 (Pasta Parameters)

The Pasta curves are defined by

$$
\begin{aligned}
p &= \mathtt{0x40000000000000000000000000000000\,224698fc094cf91b\,992d30ed00000001}, \\
q &= \mathtt{0x40000000000000000000000000000000\,224698fc0994a8dd\,8c46eb2100000001},
\end{aligned}
$$

both 255-bit primes. Pallas is $E_p : y^2 = x^3 + 5$ with order $q$;
Vesta is $E_q : y^2 = x^3 + 5$ with order $p$. Both have
$j$-invariant $0$, which gives them an efficient endomorphism
$\phi(x, y) = (\zeta x, y)$ with $\zeta$ a primitive cube root of
unity.

### Invariant 2.3 (Field Roles)

In Orchard:

- $\mathbb{F}_p$ is the **base field** of Pallas. Note commitment
  outputs, nullifiers, anchors, and all "field-element-valued"
  hashes live in $\mathbb{F}_p$.
- $\mathbb{F}_q$ is the **scalar field** of Pallas. Spend
  authorising scalars ($\mathsf{ask}$, $\alpha$, $\mathsf{rcv}$,
  $\mathsf{esk}$) live in $\mathbb{F}_q$.
- Inside the Halo 2 circuit, polynomial identities are written
  over the **Vesta scalar field**, which is exactly $\mathbb{F}_p$:
  the circuit and the Pallas base field share their type, which is
  why witness field elements assemble into Pallas points
  natively.

## 3. The Code

### 3.1 Where the Crate Imports Pasta

```rust reference title="src/circuit.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L15-L15
```

`pasta_curves::pallas` exposes `Base`, `Scalar`, `Point`, `Affine`,
and the hash-to-curve helpers. `pasta_curves::vesta` exposes the
mirror types used by the IPA commitment scheme but is otherwise
rarely visible.

### 3.2 Conversion Helpers

The spec-faithful field conversions live in
[`src/spec.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/spec.rs).
`to_base` takes a 64-byte buffer (twice the field size) and
reduces mod $p$; `to_scalar` does the same mod $q$. The 64-byte
input lets the bias from naive `mod p` reduction be negligible
($< 2^{-128}$).

### 3.3 The Endomorphism

The Pasta curves' $j = 0$ form admits the endomorphism
$\phi$ above. `halo2_gadgets` uses GLV decomposition through this
endomorphism in its ECC chip; the savings show up as fewer
incomplete additions per scalar multiplication. The Orchard code
does not invoke GLV directly but inherits the speed-up because
the chip is loaded from
[`halo2_gadgets`](https://github.com/zcash/halo2).

### 3.4 Constant-Time Discipline

`pasta_curves` uses
[`subtle`](https://github.com/dalek-cryptography/subtle) primitives
(`Choice`, `CtOption`, `ConstantTimeEq`) for every value-dependent
branch in scalar multiplication and field inversion. The crate
inherits this discipline transparently; see
[Chapter 15 (Dependencies)](./15-dependencies.md).

## 4. Failure Modes

- **Field confusion**. Computing $[k] \mathcal{G}$ with $k :
  \mathsf{pallas::Base}$ instead of $k : \mathsf{pallas::Scalar}$
  is a type error in `pasta_curves`. PRs that work around this by
  calling `to_scalar` on the byte representation of a
  base-field element risk introducing a subtle reduction bias.
- **Identity points**. Several Orchard operations require
  $\mathsf{rk} \neq \mathcal{O}$ (see
  [#492](https://github.com/zcash/orchard/pull/492)). Failing
  to enforce non-identity in a constructor is a consensus bug:
  the verifier rejects, but a wallet can be tricked into
  building a bundle that no node will accept.
- **Endianness**. `pallas::Base::from_repr` expects a little-endian
  32-byte buffer. Test vectors from external sources sometimes
  arrive big-endian; reverse them before deserialisation.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 5.4.9.5](https://zips.z.cash/protocol/protocol.pdf):
  Pallas and Vesta parameters, with the equations and the
  twisted-Edwards alternate models.
- [Pasta Curves announcement (ECC blog)](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/):
  the design rationale, including why $j = 0$ and the choice of
  a 2-cycle.
- [Halo paper](https://eprint.iacr.org/2019/1021):
  the original motivation for amortised recursion over a cycle.
- [`pasta_curves`](https://github.com/zcash/pasta_curves):
  source for the constant-time field and curve operations.

## 6. Exercises

1. Compute $p \bmod 4$ and $q \bmod 4$ from Definition 2.2 (a
   one-liner in any tool of your choice). State what this implies
   about quadratic residuosity of $-1$ in each field.
2. Open
   [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)
   and search for `vesta::`. Vesta appears in fewer than five
   places. Locate each one and explain in one sentence why Pallas
   dominates the file.
3. **Code task**. Write a five-line Rust program that takes a
   random `pallas::Scalar`, multiplies a fixed `pallas::Point`
   by it, then prints the affine $x$-coordinate. Verify that the
   coordinate type is `pallas::Base`. Commit nothing; this
   confirms the type-level distinction is real.

## 7. Further Reading

- [Halo 2 Book, Background](https://zcash.github.io/halo2/background/curves.html):
  curve choice, GLV, the role of the endomorphism in
  scalar multiplication.
- The
  [`pasta_curves`](https://github.com/zcash/pasta_curves)
  README and the comments in `src/arithmetic/curves.rs` of that
  crate.
