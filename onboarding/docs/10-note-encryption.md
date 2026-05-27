---
sidebar_position: 10
title: Note Encryption
description: ZIP 212 note plaintext encoding and the Orchard KEM, ChaCha20-Poly1305 AEAD, and outgoing ciphertext.
---

# Note Encryption

## 1. Why This Chapter Exists

Every shielded output ships an encrypted note plaintext alongside
its commitment. The recipient trial-decrypts every ciphertext on
chain; the sender additionally encrypts an outgoing ciphertext
for their own recovery. After this chapter the reader can map
each byte of an Orchard note ciphertext to its source in
[`src/note_encryption.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note_encryption.rs).

## 2. Definitions

### Definition 2.1 (Ephemeral Key)

Sender samples $\mathsf{esk} \xleftarrow{R} \mathbb{F}_q^*$ and
publishes $\mathsf{epk} = [\mathsf{esk}]\, g_d \in E_p$.

### Definition 2.2 (Shared Secret and KDF)

$$
\mathsf{dhsecret} = [\mathsf{esk}]\, \mathsf{pk_d} \in E_p, \quad
K^{\mathsf{enc}} = \mathsf{KDF}^{\mathsf{Orchard}}(\mathsf{dhsecret}, \mathsf{epk}),
$$

where $\mathsf{KDF}^{\mathsf{Orchard}}$ is Blake2b personalised
with `b"Zcash_OrchardKDF"`.

### Definition 2.3 (AEAD)

$$
C^{\mathsf{enc}} = \mathsf{ChaCha20Poly1305}_{K^{\mathsf{enc}}}(P),
$$

with fixed nonce $0^{96}$. The nonce reuse is safe because
$K^{\mathsf{enc}}$ depends on the unique $\mathsf{epk}$.

### Definition 2.4 (Outgoing Ciphertext)

A separate AEAD on
$(\mathsf{cv}, \mathsf{cm}^\star, \mathsf{epk}, P_{\mathrm{ovk}})$
under a key derived from $\mathsf{ovk}$. Allows the sender to
recover the note from their own viewing key.

### Definition 2.5 (Note Plaintext Encoding)

The 580-byte note plaintext encodes the lead byte, $d$, $v$,
$\mathsf{rseed}$, and the 512-byte memo.
[ZIP 212](https://zips.z.cash/zip-0212) deterministically derives
$\psi$ and $\mathsf{rcm}$ from $\mathsf{rseed}$ at decode time,
which prevents a malleability attack.

## 3. The Code

### 3.1 The Domain

```rust reference title="src/note_encryption.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note_encryption.rs#L82-L86
```

`OrchardDomain` carries just a single field, `rho`, which binds
the trial-decryption attempt to the Action's nullifier seed. The
implementation of the
[`zcash_note_encryption::Domain`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption)
trait further down the same file supplies the KDF, the
ephemeral-key derivation, the note plaintext encoder, and the
AEAD instance.

### 3.2 Trial Decryption

The `zcash_note_encryption` crate exposes a `try_note_decryption`
function that the Orchard wallet calls per output. Success
returns the note and the memo; failure is indistinguishable from
random under the IK-CCA security definition.

### 3.3 Outgoing Encryption

`OrchardDomain` also supplies the outgoing-ciphertext fields. The
binding to the value commitment $\mathsf{cv}$ and the extracted
commitment $\mathsf{cm}^\star$ prevents a sender from later
claiming the wrong note was sent.

### 3.4 Compact vs Full Ciphertexts

`OrchardDomain::COMPACT_NOTE_SIZE` and the related `NOTE_SIZE`
constants distinguish the two variants. Light wallets receive
only the compact prefix from a public-server scan.

## 4. Failure Modes

- **Skipping the $\mathsf{rseed}$ check**. If a decoder accepts
  the explicit $\psi$ / $\mathsf{rcm}$ fields instead of
  re-deriving them, an adversary can re-encrypt a malicious
  payload that decrypts to a wrong note.
- **Fixed-nonce abuse**. The fixed nonce is safe only because
  each $K^{\mathsf{enc}}$ is single-use. Reusing $\mathsf{esk}$
  across two outputs breaks confidentiality (key + nonce reuse).
- **Memo length leak**. The memo is padded to a fixed 512 bytes;
  any code path that emits a variable-length memo leaks length.
- **Ciphertext padding regression**. If a future contributor
  truncates the ciphertext to save bytes, the AEAD tag becomes
  recoverable but the integrity guarantee is broken. The
  `OrchardDomain` constants prevent this if used consistently.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.18](https://zips.z.cash/protocol/protocol.pdf):
  Note encryption.
- [ZIP 212](https://zips.z.cash/zip-0212):
  the deterministic randomness derivation that closes the
  malleability gap.
- [`zcash_note_encryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption):
  the shared framework.

## 6. Exercises

1. Read the implementation of `OrchardDomain::derive_esk`. Where
   is $\mathsf{esk}$ stored after it is sampled, and how does
   the trial-decryption side recover $\mathsf{epk}$ from the
   ciphertext metadata?
2. Trace a single trial-decryption call from
   [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
   into `zcash_note_encryption` and back. Identify the point at
   which the AEAD tag is verified.
3. **Code task**. Modify
   [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
   to flip one byte of the encrypted ciphertext between encryption
   and decryption. Run the test and verify the AEAD tag check
   rejects. Revert.

## 7. Further Reading

- [ZIP 316](https://zips.z.cash/zip-0316):
  Unified Addresses, which encapsulate the diversifier metadata
  the encryption depends on.
- [Bellare-Rogaway's Authenticated Encryption with Associated
  Data](https://web.cs.ucdavis.edu/~rogaway/papers/keywrap.pdf)
  is the formal model that ChaCha20-Poly1305 instantiates.
