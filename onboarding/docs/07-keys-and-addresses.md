---
sidebar_position: 7
title: keys and addresses
---

# keys and addresses

## motivation

Orchard uses a multi-level key hierarchy derived deterministically
from a single 32-byte spending key. The hierarchy is designed so that
holders of different keys can perform different actions: spend, view
incoming notes, view outgoing notes, or simply receive payments at
many addresses without linkability. This chapter gives the algebraic
relations between the keys, then walks
[`src/keys.rs`](https://github.com/zcash/orchard/blob/main/src/keys.rs)
and
[`src/zip32.rs`](https://github.com/zcash/orchard/blob/main/src/zip32.rs).

## the math

From a spending key
$\mathsf{sk} \in \{0, 1\}^{256}$ the protocol derives:

$$
\begin{aligned}
\mathsf{ask} &= \mathsf{ToScalar}\big(\mathsf{PRF}^{\mathsf{expand}}_{\mathsf{sk}}([0x06])\big),
&& \in \mathbb{F}_q \setminus \{0\}, \\
\mathsf{nk} &= \mathsf{ToBase}\big(\mathsf{PRF}^{\mathsf{expand}}_{\mathsf{sk}}([0x07])\big),
&& \in \mathbb{F}_p, \\
\mathsf{rivk} &= \mathsf{ToScalar}\big(\mathsf{PRF}^{\mathsf{expand}}_{\mathsf{sk}}([0x08])\big),
&& \in \mathbb{F}_q, \\
\mathsf{ak} &= \big[\mathsf{ask}\big]\, \mathcal{G}_{\mathsf{ak}},
&& \in E_p, \\
\mathsf{ivk} &= \mathsf{Commit}^{\mathsf{ivk}}_{\mathsf{rivk}}(\mathsf{ak}, \mathsf{nk}),
&& \in \mathbb{F}_q \setminus \{0\},
\end{aligned}
$$

where $\mathcal{G}_{\mathsf{ak}}$ is the RedPallas spend authorising
generator and $\mathsf{Commit}^{\mathsf{ivk}}$ is a Sinsemilla-based
commitment (the field-element-valued variant). The diversifier key
$\mathsf{dk}$ and outgoing viewing key $\mathsf{ovk}$ are derived
from the *Full Viewing Key*

$$
\mathsf{FVK} = (\mathsf{ak}, \mathsf{nk}, \mathsf{rivk})
$$

via a Blake2b-based PRF; see ZIP 32 for the exact byte mappings.

A diversified payment address is constructed as

$$
\mathsf{addr}_d = (d, \mathsf{pk_d})
\quad \text{with} \quad
\mathsf{pk_d} = [\mathsf{ivk}]\, g_d,
\quad
g_d = \mathsf{DiversifyHash}(d).
$$

The diversifier $d \in \{0, 1\}^{88}$ is generated from an integer
*diversifier index* $j$ by FF1 encryption with key $\mathsf{dk}$, so
different indices produce unlinkable addresses for the same FVK.

## the implementation

The Orchard `keys` module starts by defining `SpendingKey`:

```rust reference title="src/keys.rs"
https://github.com/zcash/orchard/blob/main/src/keys.rs#L1-L30
```

The key types are then defined in order of derivation:

- `SpendingKey` -> `SpendAuthorizingKey` (`ask`)
- `SpendingKey` -> `NullifierDerivingKey` (`nk`)
- `SpendingKey` -> `CommitIvkRandomness` (`rivk`)
- `(ak, nk, rivk)` -> `FullViewingKey`
- `FullViewingKey` -> `IncomingViewingKey` / `OutgoingViewingKey`
- `FullViewingKey` + `DiversifierIndex` -> `Diversifier`
- `(IncomingViewingKey, Diversifier)` -> `Address`

The `KDF_ORCHARD_PERSONALIZATION` constant (`b"Zcash_OrchardKDF"`) is
declared at the top of `keys.rs`; it parametrises the Blake2b PRF
used by the various ZIP 32 expansions.

Address construction is in
[`src/address.rs`](https://github.com/zcash/orchard/blob/main/src/address.rs):

```rust reference title="src/address.rs"
https://github.com/zcash/orchard/blob/main/src/address.rs#L1-L40
```

ZIP 32 hardened derivation is in
[`src/zip32.rs`](https://github.com/zcash/orchard/blob/main/src/zip32.rs).
The `ExtendedSpendingKey` keeps a 32-byte chain code in addition to
the spending key bytes, and exposes `derive_child` for hardened
derivation only (no non-hardened path is defined for Orchard).

## specification and references

- Zcash Protocol Specification,
  [Section 4.2.3 "Orchard Key Components"](https://zips.z.cash/protocol/protocol.pdf).
- [ZIP 32](https://zips.z.cash/zip-0032): the BIP 32 / shielded key
  derivation tree.
- [ZIP 316](https://zips.z.cash/zip-0316): Unified Addresses and
  Unified Viewing Keys.
- [`zcash_spec`](https://github.com/zcash/zcash_spec) for shared key
  derivation primitives.

## exercises

1. Find every call to `PrfExpand` in
   [`src/keys.rs`](https://github.com/zcash/orchard/blob/main/src/keys.rs).
   For each call, identify the one-byte domain tag (e.g. `[0x06]`,
   `[0x07]`) and the key it derives. Cross-check with Section 4.2.3
   of the spec.
2. Look at the implementation of `IncomingViewingKey::diversifier`.
   How does it use FF1 to map an integer index to an 88-bit
   diversifier? What is the round count and what does it imply for
   the linkability guarantee?
3. Explain in one paragraph why ZIP 32 for Orchard only defines
   hardened derivation. (Hint: consider what happens to $\mathsf{ask}$
   under non-hardened derivation if an attacker has $\mathsf{ak}$ and
   one child private key.)
