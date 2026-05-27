---
sidebar_position: 13
title: redpallas, the action signature scheme
---

# redpallas, the action signature scheme

## motivation

Orchard uses *RedPallas*, an instantiation of RedDSA over the Pallas
curve, for two distinct signature flows:

- The **binding signature** $\sigma^{\mathsf{bind}}$, which ties the
  net value commitment of a bundle to a known balance.
- The **spend authorising signatures** $\sigma_i^{\mathsf{auth}}$,
  one per Action, each under a re-randomised public key $\mathsf{rk}_i$
  derived from the spender's $\mathsf{ak}$.

The re-randomisable Schnorr signature scheme makes it possible to
hide which $\mathsf{ak}$ corresponds to each Action while still
proving spend authority. This chapter gives the algebra of RedDSA
and links it to the Orchard-specific facade.

## the math

RedDSA over a prime-order group $G = \langle \mathcal{G} \rangle$ of
order $r$, with collision-resistant hash
$H : \{0, 1\}^* \to \mathbb{F}_r$, is:

- **Key generation**: sample
  $\mathsf{sk} \stackrel{\$}{\leftarrow} \mathbb{F}_r$, output
  $\mathsf{pk} = [\mathsf{sk}] \mathcal{G}$.
- **Sign**: to sign message $m$ under $\mathsf{sk}$:
  1. sample a deterministic nonce $r$ from $\mathsf{sk}$ and $m$;
  2. compute $R = [r] \mathcal{G}$;
  3. set $c = H(R \,\|\, \mathsf{pk} \,\|\, m)$;
  4. set $s = r + c \cdot \mathsf{sk} \pmod r$;
  5. output $\sigma = (R, s)$.
- **Verify**: accept iff $[s] \mathcal{G} = R + [c] \mathsf{pk}$ for
  $c = H(R \,\|\, \mathsf{pk} \,\|\, m)$.

RedDSA adds two features on top of the basic Schnorr:

- *Re-randomisation*: $\mathsf{rk} = \mathsf{pk} + [\alpha] \mathcal{G}$
  for a sender-chosen $\alpha \in \mathbb{F}_r$; the signer with
  knowledge of $\mathsf{sk}$ and $\alpha$ can sign under $\mathsf{rk}$
  (a randomised public key). The verifier need not know $\alpha$.
- *Pluggable curves*: the same construction is reused for Sapling
  (Jubjub) and Orchard (Pallas) by parameterising the curve and the
  hash personalisation.

For Orchard, the spend authorisation generator $\mathcal{G}$ is a
fixed Pallas point. The binding signature uses a different generator
$\mathcal{R}$ (the value commitment randomness base from chapter
[value-commitments](./12-value-commitments.md)).

## the implementation

Orchard ships a thin facade in
[`src/primitives/redpallas.rs`](https://github.com/zcash/orchard/blob/main/src/primitives/redpallas.rs):

```rust reference title="src/primitives/redpallas.rs"
https://github.com/zcash/orchard/blob/main/src/primitives/redpallas.rs#L1-L25
```

The facade wraps the upstream
[`reddsa`](https://github.com/ZcashFoundation/reddsa) crate. Two
marker types parametrise the signature scheme:

- `SpendAuth` for spend-authorising signatures (uses
  $\mathcal{G}_{\mathsf{ak}}$ as the base).
- `Binding` for the binding signature (uses $\mathcal{R}$ as the
  base).

`reddsa::orchard::SpendAuth` and `reddsa::orchard::Binding` define
the parameters; `orchard::primitives::redpallas` re-exposes them
under more convenient names and provides the `BatchVerifier`
plumbing used by `Bundle::batch_verify`.

The re-randomisation of $\mathsf{ak}$ to $\mathsf{rk}$ happens at
two points:

- In
  [`src/builder.rs`](https://github.com/zcash/orchard/blob/main/src/builder.rs),
  where the builder samples
  $\alpha_i \stackrel{\$}{\leftarrow} \mathbb{F}_q$ for each Action
  and stores the re-randomised key.
- Inside the Action circuit
  ([`src/circuit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit.rs)),
  where the witness includes $\mathsf{ak}$ and $\alpha$ and the
  circuit constraints
  $\mathsf{rk} = \mathsf{ak} + [\alpha] \mathcal{G}_{\mathsf{ak}}$
  on the public $\mathsf{rk}$ instance columns.

## specification and references

- Zcash Protocol Specification,
  [Section 5.4.7 (RedDSA, RedJubjub, and RedPallas)](https://zips.z.cash/protocol/protocol.pdf).
- [Re-randomisable signatures](https://eprint.iacr.org/2014/758)
  (Pointcheval, Sanders) and the discussion in the Sapling protocol
  paper.
- [`zcashfoundation/reddsa`](https://github.com/ZcashFoundation/reddsa).

## exercises

1. Verify on paper that the re-randomised verification equation
   $[s] \mathcal{G} = R + [c] \mathsf{rk}$ holds when the signer
   knows both $\mathsf{ask}$ and $\alpha$ and computes
   $s = r + c(\mathsf{ask} + \alpha) \bmod r$.
2. Why does the verifier *not* need $\alpha$? What does this imply
   for the unlinkability of two Actions signed by the same wallet?
3. Open
   [`src/primitives/redpallas.rs`](https://github.com/zcash/orchard/blob/main/src/primitives/redpallas.rs).
   List the two newtype wrappers around `reddsa` types and the
   reason each wrapper exists (hint: the `subtle` crate's
   constant-time equality is one such reason).
