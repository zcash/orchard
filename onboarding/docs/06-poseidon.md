---
sidebar_position: 6
title: poseidon
---

# poseidon

## motivation

Poseidon is an algebraic hash function tailored for use inside
arithmetic circuits. Orchard uses it for the nullifier derivation
key combinator (the $\otimes$ operator in the nullifier formula) and
for inner transcript-style mixing where Sinsemilla is too coarse.
Inside a Halo 2 circuit, Poseidon is dramatically cheaper than a
byte-oriented hash like SHA-256 because every round is a polynomial
identity over the field. This chapter introduces the parameters used
in Orchard and the implementing crate.

## the math

Poseidon is a sponge over a permutation
$\pi : \mathbb{F}_q^t \to \mathbb{F}_q^t$, where $t$ is the *width*.
The permutation has three ingredients per round:

1. **AddRoundConstants (ARC)**: add a vector of constants
   $c^{(i)} \in \mathbb{F}_q^t$.
2. **S-box ($\alpha$-power)**: raise each element of the state to the
   power $\alpha$. For Pasta, $\gcd(\alpha, q - 1) = 1$ for
   $\alpha = 5$, so $x \mapsto x^5$ is a bijection.
3. **MixLayer (MDS)**: multiply the state by a fixed $t \times t$
   matrix $M$ over $\mathbb{F}_q$ chosen so that no submatrix is
   singular (a Maximum Distance Separable matrix).

To save constraints, Poseidon distinguishes *full rounds* (S-box
applied to every element) from *partial rounds* (S-box applied only
to the first element). The Orchard parameters are
$P_{128}^{\mathrm{Pasta}}$ with $t = 3$, $\alpha = 5$, $R_F = 8$ full
rounds and $R_P = 56$ partial rounds, targeting 128-bit security.

Used as a hash, the sponge absorbs $t - 1 = 2$ field elements per
permutation call and squeezes one element. For the Orchard
nullifier:

$$
\mathsf{PRF}^{\mathsf{nfOrchard}}(\mathsf{nk}, \rho) =
\mathsf{Poseidon}_{P_{128}^{\mathrm{Pasta}}}(\mathsf{nk}, \rho)_0,
$$

where the subscript $0$ denotes the first output element.

## the implementation

The pure-Rust Poseidon implementation is the
[`halo2_poseidon`](https://crates.io/crates/halo2_poseidon) crate.
Orchard's `Cargo.toml` re-exposes it under the local alias
`poseidon`:

```toml reference title="Cargo.toml"
https://github.com/zcash/orchard/blob/main/Cargo.toml#L41-L41
```

The chip used inside the circuit is `halo2_gadgets::poseidon::Pow5Chip`,
configured for $P_{128}^{\mathrm{Pasta}}$ in
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs).
The parameter trait `P128Pow5T3` lives in `halo2_gadgets::poseidon::primitives`
and exposes the MDS matrix, the round constants, and the round count.

The Orchard-specific use site is the nullifier:

```rust reference title="src/note/nullifier.rs"
https://github.com/zcash/orchard/blob/main/src/note/nullifier.rs#L1-L25
```

Inside the circuit, the nullifier is computed by combining the
output of the Poseidon hash with the in-circuit $\rho + \psi +
\mathsf{cm}$ sum, then extracting the $x$-coordinate of the
resulting point.

## specification and references

- [Poseidon paper](https://eprint.iacr.org/2019/458): Grassi, Khovratovich,
  Rechberger, Roy, Schofnegger.
- [Poseidon $\alpha$ choice for Pasta](https://zcash.github.io/halo2/design/gadgets/poseidon.html)
  in the Halo 2 book.
- Zcash Protocol Specification,
  [Section 5.4.1.10 "Poseidon Hash"](https://zips.z.cash/protocol/protocol.pdf).
- [`halo2_poseidon`](https://crates.io/crates/halo2_poseidon)
  documentation.

## exercises

1. Compute $\gcd(5, q - 1)$ for the Pallas scalar field modulus $q$
   from chapter [pasta-curves](./02-pasta-curves.md). Verify it is
   $1$, then explain why $\alpha = 5$ makes $x \mapsto x^5$ a
   permutation.
2. The Pasta paper claims 128-bit security with $R_F = 8$ and
   $R_P = 56$. Look up the corresponding security argument in the
   Poseidon paper (Section 5) and identify which equation gives the
   minimal $R_F + R_P$.
3. Trace the nullifier computation in
   [`src/note/nullifier.rs`](https://github.com/zcash/orchard/blob/main/src/note/nullifier.rs)
   step by step. List the inputs to the Poseidon call and the field
   in which each one lives.
