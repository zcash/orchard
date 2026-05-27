---
sidebar_position: 9
title: note encryption
---

# note encryption

## motivation

When an Action creates a new shielded note, it must transmit the
plaintext note (and an optional memo) to the recipient. Orchard's
note encryption is a one-shot integrated encryption scheme: it
derives a symmetric key from a sender-chosen ephemeral key and the
recipient's incoming viewing key, encrypts the note plaintext and
memo, and additionally encrypts a "outgoing" payload that lets the
sender's wallet recover what was sent. The construction is
formalised in ZIP 212 and reused, with curve-specific details, from
Sapling. This chapter shows the construction and points at
[`src/note_encryption.rs`](https://github.com/zcash/orchard/blob/main/src/note_encryption.rs).

## the math

For a fresh Action that creates note $n^\star$ to recipient
$(d, \mathsf{pk_d})$:

1. Sender samples an ephemeral secret
   $\mathsf{esk} \stackrel{\$}{\leftarrow} \mathbb{F}_q^*$.
2. Sender computes the ephemeral public key
   $\mathsf{epk} = [\mathsf{esk}]\, g_d \in E_p$.
3. The shared secret is
   $\mathsf{dhsecret} = [\mathsf{esk}]\, \mathsf{pk_d}$.
   (The recipient recovers it as
   $[\mathsf{ivk}]\, \mathsf{epk}$.)
4. The symmetric key is
   $K^{\mathsf{enc}} =
   \mathsf{KDF}^{\mathsf{Orchard}}(\mathsf{dhsecret}, \mathsf{epk})$,
   where $\mathsf{KDF}$ is a Blake2b instance personalised with
   `Zcash_OrchardKDF`.
5. The note plaintext
   $P = \mathsf{NotePlaintextBytes}(d, v, \mathsf{rseed}, \mathsf{memo})$
   is encrypted with $\mathsf{Sym.Encrypt}_{K^{\mathsf{enc}}}(P)$ to
   produce $C^{\mathsf{enc}}$ (AEAD: ChaCha20-Poly1305 with a fixed
   nonce, since $\mathsf{epk}$ already binds the key).
6. The sender additionally encrypts an *outgoing ciphertext*
   $C^{\mathsf{out}}$ under a key derived from $\mathsf{ovk}$ and
   $(\mathsf{cv}, \mathsf{cm}^\star, \mathsf{epk})$, allowing recovery
   from the sender's side.

A recipient with $\mathsf{ivk}$ tries the trial decryption for each
shielded output; success recovers the note. Without $\mathsf{ivk}$
the AEAD tag is indistinguishable from random.

## the implementation

The Orchard note encryption uses
[`zcash_note_encryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption),
a shared crate factored out of Sapling/Orchard. Orchard provides a
domain via the `OrchardDomain` type:

```rust reference title="src/note_encryption.rs"
https://github.com/zcash/orchard/blob/main/src/note_encryption.rs#L1-L30
```

`OrchardDomain` is the Orchard-specific implementation of the
`Domain` trait; it provides the KDF, the ephemeral public key
derivation, the note plaintext encoding, and the AEAD instance.

Compact and non-compact note ciphertexts are both supported via the
`OrchardNoteEncryption` type alias. The compact form omits the memo
and the AEAD tag of the outgoing payload, and is used by light
wallets that scan via a public service.

The KDF is defined as:

The note plaintext encoding follows ZIP 212; the first byte is a
*lead byte* identifying the encoding version, and $\psi$ and
$\mathsf{rcm}$ are recovered deterministically from
$\mathsf{rseed}$ rather than transmitted explicitly. The
`OrchardDomain` implementation enforces this at decode time, which
also defends against malleability: an attacker cannot tweak the
plaintext to land on a different commitment.

## specification and references

- Zcash Protocol Specification,
  [Section 4.18 "Note encryption"](https://zips.z.cash/protocol/protocol.pdf).
- [ZIP 212](https://zips.z.cash/zip-0212): note plaintext encoding
  for Sapling and Orchard.
- [ZIP 316](https://zips.z.cash/zip-0316): Unified Addresses.
- [`zcash_note_encryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption).

## exercises

1. The `OrchardDomain` implementation uses a fixed nonce for
   ChaCha20-Poly1305. Why is this safe?
2. Read the `parse_note_plaintext_without_memo` function in
   [`zcash_note_encryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption)
   and trace how the Orchard-specific
   [`extract_epk`](https://github.com/zcash/orchard/blob/main/src/note_encryption.rs)
   is plugged in.
3. Construct (on paper) a forged $C^{\mathsf{enc}}$ that decrypts
   successfully but to a note whose $\mathsf{cm}^\star$ does not
   match the on-chain Action. Where in the code is this forgery
   detected?
