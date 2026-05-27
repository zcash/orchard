---
sidebar_position: 8
title: Keys and Addresses
description: The Orchard key hierarchy in src/keys.rs and src/zip32.rs.
---

# Keys and Addresses

## 1. Why This Chapter Exists

A shielded wallet holds a 32-byte secret and exposes a tree of
derived keys, each with a specific capability (spend, view
incoming, view outgoing, diversify). A contributor changing
anything in
[`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs)
risks breaking the capability separation that protects users.
After this chapter the reader can identify which key is needed
for each operation and prove that no extra capability leaks.

## 2. Definitions

### Definition 2.1 (Spending Key)

$\mathsf{sk} \in \{0, 1\}^{256}$, the root of the key hierarchy.
Hierarchical derivation via ZIP 32 produces an extended spending
key $(\mathsf{sk}, \mathsf{c}) \in \{0, 1\}^{256+256}$ with chain
code $\mathsf{c}$.

### Definition 2.2 (Derived Scalars and Bases)

$$
\begin{aligned}
\mathsf{ask} &= \mathsf{ToScalar}\big(\mathsf{PRF}^{\mathsf{expand}}_{\mathsf{sk}}([\mathtt{0x06}])\big) \in \mathbb{F}_q \setminus \{0\}, \\
\mathsf{nk}  &= \mathsf{ToBase}\big(\mathsf{PRF}^{\mathsf{expand}}_{\mathsf{sk}}([\mathtt{0x07}])\big) \in \mathbb{F}_p, \\
\mathsf{rivk}&= \mathsf{ToScalar}\big(\mathsf{PRF}^{\mathsf{expand}}_{\mathsf{sk}}([\mathtt{0x08}])\big) \in \mathbb{F}_q, \\
\mathsf{ak}  &= [\mathsf{ask}]\, \mathcal{G}_{\mathsf{ak}} \in E_p.
\end{aligned}
$$

### Definition 2.3 (IVK and Full Viewing Key)

$$
\mathsf{ivk} = \mathsf{Commit}^{\mathsf{ivk}}_{\mathsf{rivk}}(\mathsf{ak}, \mathsf{nk}) \in \mathbb{F}_q \setminus \{0\},
$$

and the Full Viewing Key is the triple
$\mathsf{FVK} = (\mathsf{ak}, \mathsf{nk}, \mathsf{rivk})$.

### Definition 2.4 (Diversifier and Address)

A diversifier $d \in \{0, 1\}^{88}$ is produced from an integer
index $j$ by FF1 encryption with key $\mathsf{dk}$.
$g_d = \mathsf{DiversifyHash}(d) \in E_p$ and
$\mathsf{pk_d} = [\mathsf{ivk}]\, g_d$.
A payment address is $\mathsf{addr}_d = (d, \mathsf{pk_d})$.

### Invariant 2.5 (Capability Hierarchy)

The arrows below are deterministic and one-way under standard
assumptions:

$$
\mathsf{sk} \to \mathsf{ask} \to \mathsf{ak} \to \mathsf{FVK} \to (\mathsf{ivk}, \mathsf{ovk}, \mathsf{dk}) \to (g_d, \mathsf{pk_d}).
$$

A holder of $\mathsf{ivk}$ can detect incoming notes but cannot
spend; a holder of $\mathsf{ovk}$ can decrypt the outgoing
ciphertext; a holder of $\mathsf{dk}$ can enumerate the
addresses; only a holder of $\mathsf{ask}$ can sign spend
authorisations.

## 3. The Code

### 3.1 The Spending Key

```rust reference title="src/keys.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs#L1-L30
```

`SpendingKey` is opaque: the bytes are private, and access goes
through derivation methods that return wrapper types
(`SpendAuthorizingKey`, `NullifierDerivingKey`,
`CommitIvkRandomness`).

### 3.2 The Derivation Tree

The methods in
[`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs)
implement the tree in order:

- `SpendingKey -> SpendAuthorizingKey` via
  $\mathsf{PRF}^{\mathsf{expand}}$ with tag `0x06`.
- `SpendingKey -> NullifierDerivingKey` with tag `0x07`.
- `SpendingKey -> CommitIvkRandomness` with tag `0x08`.
- `(ak, nk, rivk) -> FullViewingKey`.
- `FullViewingKey -> IncomingViewingKey | OutgoingViewingKey |
  DiversifierKey`.
- `(IncomingViewingKey, Diversifier) -> Address`.

The KDF personalisation `KDF_ORCHARD_PERSONALIZATION =
b"Zcash_OrchardKDF"` parametrises the Blake2b expansions used by
the various branches.

### 3.3 Addresses

```rust reference title="src/address.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/address.rs#L1-L40
```

`Address::from_parts` is the canonical constructor; it does not
perform any algebraic check beyond well-formedness of the
$g_d$ deserialisation.

### 3.4 ZIP 32

[`src/zip32.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/zip32.rs)
implements hardened-only derivation: $(\mathsf{sk}, \mathsf{c}, i)
\to (\mathsf{sk}', \mathsf{c}')$ with $i$ in the hardened range.
There is no non-hardened path; see Failure Modes.

## 4. Failure Modes

- **Non-hardened derivation reintroduced**. Orchard's ZIP 32 path
  is hardened-only. Adding a non-hardened branch lets a child
  $\mathsf{ak}$ and one child private key together reveal the
  parent $\mathsf{ask}$. This is the same property that broke
  certain BIP 32 wallets and is the reason Sapling/Orchard
  hard-coded hardening.
- **Identity `ak` or `rk`**. A `SpendingKey` whose derived
  $\mathsf{ak}$ is the curve identity should be rejected at
  construction. The corresponding rule for `rk` is enforced by
  [#492](https://github.com/zcash/orchard/pull/492); a similar
  rule for the construction-time check on `ak` lives in
  [`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs).
- **Capability leak via debug printing**. `Debug` impls on key
  types must not leak the raw scalar. The crate uses opaque
  `Debug` formatters; review carefully when adding new key
  types.
- **Deep `ExtendedSpendingKey` derivation panic**. See
  [#464](https://github.com/zcash/orchard/issues/464): chains
  longer than 255 levels panic.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.2.3](https://zips.z.cash/protocol/protocol.pdf):
  Orchard key components.
- [ZIP 32](https://zips.z.cash/zip-0032):
  shielded hardened-only derivation.
- [ZIP 316](https://zips.z.cash/zip-0316):
  Unified Addresses, which bundle Orchard, Sapling, and
  transparent addresses.
- [`zcash_spec`](https://github.com/zcash/zcash_spec):
  shared $\mathsf{PRF}^{\mathsf{expand}}$ primitive used by the
  derivation code.

## 6. Exercises

1. Search
   [`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs)
   for every call to `PrfExpand`. For each call, identify the
   one-byte domain tag and cross-check against Section 4.2.3 of
   the spec.
2. Read the `IncomingViewingKey::address_at` method. How does it
   use FF1 to encrypt an integer index to a diversifier? What
   are the implications for the unlinkability of addresses
   produced by adjacent indices?
3. **Code task**. Write a unit test that derives an
   `ExtendedSpendingKey` from a fixed seed, enumerates the first
   three diversified addresses, and asserts they are distinct.
   Place it in `src/zip32.rs` under the `#[cfg(test)]` module.
   Run `cargo test --lib zip32::tests`.

## 7. Further Reading

- The
  [Orchard Book, Keys and addresses](https://zcash.github.io/orchard/design/keys.html)
  walkthrough of the same hierarchy.
- BIP 32 hardened-derivation discussion in
  [BIP 32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
  for the historical reason behind the hardened-only choice.
