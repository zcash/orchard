---
sidebar_position: 2
title: the pasta cycle of curves
---

# the pasta cycle of curves

## motivation

Orchard's proof system is Halo 2, which uses inner product arguments
without a trusted setup. The recursion technique behind Halo
([Bowe, Grigg, Hopwood 2019](https://eprint.iacr.org/2019/1021))
needs two curves $E_1 / \mathbb{F}_p$ and $E_2 / \mathbb{F}_q$ such
that the base field of one is the scalar field of the other and vice
versa: a "cycle". The Pasta curves
[Pallas and Vesta](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/)
are exactly such a cycle, designed for the Halo 2 protocol. Orchard
uses Pallas for the in-circuit arithmetic and Vesta as the commitment
curve.

## the math

Let $p$ and $q$ be two distinct primes. A pair of curves
$E_p / \mathbb{F}_p$ and $E_q / \mathbb{F}_q$ is a 2-cycle if

$$
\#E_p(\mathbb{F}_p) = q
\quad \text{and} \quad
\#E_q(\mathbb{F}_q) = p.
$$

The base field of $E_p$ is then the scalar field of $E_q$ and vice
versa. In Pasta:

$$
\begin{aligned}
p &= 0x40000000\;00000000\;00000000\;00000000\;224698fc\;094cf91b\;992d30ed\;00000001, \\
q &= 0x40000000\;00000000\;00000000\;00000000\;224698fc\;0994a8dd\;8c46eb21\;00000001.
\end{aligned}
$$

Both are roughly 255 bits. Pallas is the curve over $\mathbb{F}_p$
with order $q$; Vesta is over $\mathbb{F}_q$ with order $p$. Both
have $j$-invariant $0$ and equation $y^2 = x^3 + 5$.

Useful identities for cycle hopping:

- A scalar in $\mathbb{F}_q$ acts as a coordinate (a base-field
  element) on the other curve, and conversely. When a Pallas point is
  hashed into a Pallas base-field element via the
  $x$-coordinate-extraction map $\mathsf{Extract}$, the result is a
  scalar of Vesta if we choose to view it that way.
- The "embedded curve" used inside the Action circuit is Pallas: the
  circuit's arithmetic happens in $\mathbb{F}_q$ (the Vesta scalar
  field, equivalently the Pallas base field), and the witnesses
  include points of $E_p$ whose coordinates fit natively into circuit
  variables.

The choice of $j = 0$ is deliberate: it gives an efficient
endomorphism $\phi$ with $\phi(P) = \lambda P$ for a cube root of
unity $\lambda \in \mathbb{F}_q$ on Pallas, which speeds up scalar
multiplication and is exploited by the GLV decomposition used by
[`halo2_gadgets`](https://github.com/zcash/halo2).

## the implementation

The cycle is provided by the
[`pasta_curves`](https://github.com/zcash/pasta_curves) crate. The
`orchard` crate imports it as:

```rust reference title="src/circuit.rs"
https://github.com/zcash/orchard/blob/main/src/circuit.rs#L16-L16
```

The two field types and the curve groups are then used pervasively:

- `pallas::Base` is the Pallas base field $\mathbb{F}_p$. It is the
  field in which the Action circuit polynomials live, because Pallas
  is the embedded curve and Vesta is the commitment curve.
- `pallas::Scalar` is the Pallas scalar field $\mathbb{F}_q$. Spend
  authorising scalars (e.g. $\alpha$, the re-randomiser) and value
  commitment scalars (e.g. $\mathsf{rcv}$) live here.
- `pallas::Point` and `pallas::Affine` are the projective and affine
  representations of $E_p$.

A representative use of the field hierarchy from
[`src/spec.rs`](https://github.com/zcash/orchard/blob/main/src/spec.rs)
is the conversion `to_base`, which decodes an array of 64 bytes into
a `pallas::Base` modulo $p$:

The `pasta_curves` crate exposes a base-field hash function family
$\mathsf{H}$ used to derive nullifier-deriving keys; the curve crate
also provides the constant-time `CtOption<T>` decoding used in
deserialisation.

## specification and references

- Pasta curves announcement:
  [Pasta curves for Halo 2 and beyond](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/).
- [Halo paper](https://eprint.iacr.org/2019/1021) for the cycle
  motivation.
- [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf),
  Section 5.4.9.5 "The Pallas and Vesta curves".
- [`pasta_curves`](https://github.com/zcash/pasta_curves) source.

## exercises

1. Compute $p \bmod 4$ and $q \bmod 4$. What does the result tell you
   about the existence of a square root of $-1$ in each field?
2. Open
   [`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs)
   and find every occurrence of `vesta::`. Where is Vesta actually
   used in the Orchard implementation, and why does it appear so much
   less often than Pallas?
3. In
   [`src/spec.rs`](https://github.com/zcash/orchard/blob/main/src/spec.rs),
   locate the `to_base` and `to_scalar` helpers. Explain in one
   sentence why the helpers take a `[u8; 64]` rather than `[u8; 32]`.
