---
sidebar_position: 14
title: RedPallas, the Action Signature Scheme
description: RedDSA over Pallas as it appears in src/primitives/redpallas.rs.
---

# RedPallas, the Action Signature Scheme

## 1. Why This Chapter Exists

Orchard signs two things per bundle: the binding commitment with
the **binding** signature, and each Action's `rk` with a
**spend-authorising** signature. Both use RedDSA over Pallas
(RedPallas). The re-randomisable property lets each Action sign
under a one-time public key without revealing the wallet's
$\mathsf{ak}$. After this chapter the reader understands the
difference between the two flavours and can derive their bases.

## 2. Definitions

### Definition 2.1 (RedDSA Sign)

Over a prime-order group with base $\mathcal{G}$ and order $r$,
with hash $H : \{0, 1\}^* \to \mathbb{F}_r$, to sign $m$ under
secret $\mathsf{sk}$ with public $\mathsf{pk} = [\mathsf{sk}]
\mathcal{G}$:

1. Sample nonce $r \in \mathbb{F}_r$ deterministically from
   $\mathsf{sk}$ and $m$.
2. $R = [r]\, \mathcal{G}$.
3. $c = H(R \mathbin{\|} \mathsf{pk} \mathbin{\|} m)$.
4. $s = r + c \cdot \mathsf{sk} \pmod r$.
5. Output $\sigma = (R, s)$.

### Definition 2.2 (Re-randomisation)

For randomiser $\alpha \in \mathbb{F}_r$, the randomised key is
$\mathsf{rk} = \mathsf{pk} + [\alpha]\, \mathcal{G}$. The
randomised secret is $\mathsf{sk} + \alpha$, and a signer with
both can sign messages under $\mathsf{rk}$.

### Definition 2.3 (Orchard Flavours)

- **`SpendAuth`**: base $\mathcal{G}_{\mathsf{ak}}$, signs the
  per-Action SIGHASH; key is $\mathsf{rk}$.
- **`Binding`**: base $\mathcal{R}$ (the value commitment
  randomness base), signs the bundle-level SIGHASH; key is
  $\mathsf{bvk}$ from
  [Chapter 13](./13-value-commitments.md).

## 3. The Code

### 3.1 The Facade

```rust reference title="src/primitives/redpallas.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/primitives/redpallas.rs#L1-L25
```

Orchard wraps the upstream
[`reddsa`](https://github.com/ZcashFoundation/reddsa) crate. The
two marker types `SpendAuth` and `Binding` parametrise the same
implementation over the two bases.

### 3.2 Use Sites

- Per-Action `rk` derivation in
  [`src/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs)
  (sampling $\alpha$ and constructing $\mathsf{rk} = \mathsf{ak}
  + [\alpha] \mathcal{G}_{\mathsf{ak}}$).
- In-circuit constraint $\mathsf{rk} = \mathsf{ak} + [\alpha]
  \mathcal{G}_{\mathsf{ak}}$ in
  [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs).
- Binding signature key derivation in
  [`src/bundle.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs).

## 4. Failure Modes

- **Identity `rk`**.
  [#492](https://github.com/zcash/orchard/pull/492) added an
  explicit rejection of identity `rk` in `Action::from_parts`.
  The defence catches a malformed bundle that would otherwise
  pass verification under a special case of the Schnorr
  equation.
- **Nonce reuse**. RedDSA derives the nonce deterministically
  from $\mathsf{sk}$ and the message; a buggy override that
  resamples randomly opens the door to a discrete-log recovery
  attack on $\mathsf{sk}$.
- **Wrong base**. Signing a `SpendAuth` message with the
  `Binding` base or vice versa produces a verifier that accepts
  unrelated values. The marker types prevent this at compile
  time; do not bypass them.
- **`alpha = 0`**. If $\alpha = 0$, $\mathsf{rk} = \mathsf{ak}$
  and unlinkability is lost. The builder must sample
  $\alpha \neq 0$; an override that allows zero is a privacy
  bug.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 5.4.7](https://zips.z.cash/protocol/protocol.pdf):
  RedDSA, RedJubjub, and RedPallas instantiation.
- [Pointcheval-Sanders re-randomisable signatures](https://eprint.iacr.org/2014/758):
  the underlying construction.
- [`reddsa`](https://github.com/ZcashFoundation/reddsa):
  the upstream Rust implementation.

## 6. Exercises

1. Verify on paper that
   $[s] \mathcal{G} = R + [c] \mathsf{rk}$ holds when
   $s = r + c(\mathsf{sk} + \alpha) \bmod r$. Show that the
   verifier does not need $\alpha$.
2. The two marker types `SpendAuth` and `Binding` cannot be
   confused at compile time. Read
   [`src/primitives/redpallas.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/primitives/redpallas.rs)
   and identify the trait bound that enforces the distinction.
3. **Code task**. Add a unit test in
   `src/primitives/redpallas.rs` that constructs an
   identity-valued `VerificationKey<SpendAuth>` and asserts that
   parsing returns an error. Run `cargo test --lib redpallas::`.

## 7. Further Reading

- [Sapling protocol paper](https://eprint.iacr.org/2018/903)
  for the historical RedJubjub design that RedPallas mirrors.
- The audit reports linked from
  [Chapter 17](./17-audits.md), which review the binding
  signature derivation in detail.
