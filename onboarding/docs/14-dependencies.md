---
sidebar_position: 14
title: cryptographic dependencies and constant-time guarantees
---

# cryptographic dependencies and constant-time guarantees

## motivation

The `orchard` crate is a small core that delegates most of its
algebra to other crates. Auditing Orchard therefore means knowing
which crate is responsible for which primitive and what guarantees
that crate provides, especially around side-channels. This chapter
enumerates the cryptographic dependencies, the role each plays, and
the side-channel posture inherited by `orchard`.

## the math, in this context

For a curve-based protocol, two side-channel guarantees matter:

- **Constant-time scalar multiplication**: $[k] P$ runs in time
  independent of $k$.
- **Constant-time field operations**: $a \cdot b$, $a + b$, $a^{-1}$
  run in time independent of $a, b$.

Both are properties of the underlying field / group library, not of
the protocol. The `subtle` crate provides constant-time primitives
(`Choice`, `CtOption`, `ConstantTimeEq`) that the curve libraries
use to construct constant-time algorithms.

## the dependencies

The full list is in
[`Cargo.toml`](https://github.com/zcash/orchard/blob/main/Cargo.toml):

```toml reference title="Cargo.toml"
https://github.com/zcash/orchard/blob/main/Cargo.toml#L25-L52
```

The cryptographically critical ones:

- [`pasta_curves`](https://github.com/zcash/pasta_curves): the
  Pallas and Vesta curve implementations. Provides
  `pallas::Base`, `pallas::Scalar`, `pallas::Point`, and the
  hash-to-curve and field-from-uniform-bytes helpers. Scalar
  multiplication and field inversion are constant-time.
- `halo2_proofs` and `halo2_gadgets` (via the `circuit` feature):
  the proof system and the standard chip library, both maintained
  in
  [`zcash/halo2`](https://github.com/zcash/halo2).
- [`sinsemilla`](https://github.com/zcash/sinsemilla): the
  Sinsemilla hash function implementation, used both inside and
  outside the circuit.
- `halo2_poseidon` (re-exported as `poseidon`): the Poseidon
  permutation; see chapter [poseidon](./06-poseidon.md).
- [`reddsa`](https://github.com/ZcashFoundation/reddsa): RedDSA over
  Jubjub and Pallas; see chapter [redpallas](./13-redpallas.md).
- [`zcash_note_encryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption):
  the shared note encryption framework.
- [`zip32`](https://github.com/zcash/zip32) and
  [`zcash_spec`](https://github.com/zcash/zcash_spec): shared
  primitives for ZIP 32 / ZIP 316 across Sapling and Orchard.

Side-channel hygiene primitives:

- [`subtle`](https://github.com/dalek-cryptography/subtle): the
  shared constant-time helper crate. `pasta_curves`, `reddsa`,
  `aes`, `blake2b_simd`, and Orchard itself all use it.
- The crate does not currently use `zeroize` directly; the secret
  scalar wrappers are protected by `subtle::ConditionallySelectable`
  and the `#[forbid(unsafe_code)]` attribute in
  [`src/lib.rs`](https://github.com/zcash/orchard/blob/main/src/lib.rs).

Symmetric primitives used by note encryption:

- [`aes`](https://github.com/RustCrypto/block-ciphers/tree/master/aes):
  AES-256 used by FF1 (`fpe`) for diversifier index encryption.
- [`blake2b_simd`](https://github.com/oconnor663/blake2b_simd): the
  Blake2b instance used for KDFs and PRFs.
- [`fpe`](https://github.com/str4d/fpe): format-preserving encryption
  (FF1) used to map diversifier indices to diversifiers.

## the no-std posture

`orchard` is `#![no_std]` (see chapter
[overview](./01-overview.md)). All cryptographic dependencies are
configured with `default-features = false` to keep them no-std
compatible. This makes the crate suitable for embedded usage but
implies that any new dependency must come without an implicit
`std` requirement.

## specification and references

- The `RUSTSEC` advisory database:
  [rustsec.org](https://rustsec.org/).
- The `cargo audit` tool for checking advisories against
  `Cargo.lock`.
- [Halo 2 audit reports](https://zcash.github.io/halo2/audits.html).
- [`pasta_curves`](https://github.com/zcash/pasta_curves) README,
  which documents the constant-time guarantees.

## exercises

1. Run `cargo tree -e features` from the `orchard` crate root. Pick
   one dependency you have never heard of and write a one-paragraph
   summary of its role in Orchard.
2. Open
   [`Cargo.toml`](https://github.com/zcash/orchard/blob/main/Cargo.toml).
   Identify every dependency with `default-features = false` and
   explain why this matters for `no_std` builds.
3. Pick a constant-time-critical function in
   [`pasta_curves`](https://github.com/zcash/pasta_curves) (e.g.
   `Fp::invert`) and verify by code reading that no early returns
   are present that could leak the operand.
