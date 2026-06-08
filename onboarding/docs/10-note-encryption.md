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

## 4. Why Decryption Gives a Spendable Note

Sections 2 and 3 covered the key agreement, the KDF, and where the AEAD
lives in the code. What is missing is the connection to the Action
relation in [chapter 5](./05-action-circuit.md) (Definition 2.2): how
does the data the receiver pulls out of `enc_ciphertext` become enough
to spend the note in a later Action? This section closes the loop.

### 4.1 What the Receiver Learns From One Trial Decryption

For every on-chain Action, the receiver runs
[`try_note_decryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption)
with their incoming viewing key $\mathsf{ivk}$. The trial either fails
(the Action is for someone else) or succeeds and returns a
[`Note`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs#L141-L155)
together with a 512-byte memo.

On success the receiver holds the contents of the v2 plaintext format,
laid out by
[`note_plaintext_bytes`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note_encryption.rs#L170-L178):

| Field            | Plaintext bytes   | Recovered at spend time as                |
| ---------------- | ----------------- | ----------------------------------------- |
| Lead $0x02$      | byte 0            | format-version marker                     |
| $d$              | bytes 1..12       | $g_d = \mathsf{DiversifyHash}(d)$ witness |
| $v$              | bytes 12..20 (LE) | $v$ witness                               |
| $\mathsf{rseed}$ | bytes 20..52      | $\psi$ and $\mathsf{rcm}$ derivation      |
| memo             | bytes 52..564     | not used in the circuit                   |

From $\mathsf{rseed}$ the receiver derives $\psi$ and $\mathsf{rcm}$
deterministically (ZIP 212, exposed as
[`RandomSeed::psi`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs#L107)
and
[`RandomSeed::rcm`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs#L132)).
From $d$ they recompute $g_d$. From their own $\mathsf{ivk}$ they
recompute $\mathsf{pk_d} = [\mathsf{ivk}]\, g_d$ and the recipient
address $(d, \mathsf{pk_d})$. The note's $\rho$ does not ride in the
plaintext: the chain provides it directly, because Orchard ties
$\rho^{\mathsf{new}} = \mathsf{nf}^{\mathsf{old}}$ inside the Action
circuit (see
[`src/circuit.rs#L676`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L676)),
so the receiver reads $\rho$ off the same Action description that
carried the ciphertext. With $(g_d, \mathsf{pk_d}, v, \rho, \psi,
\mathsf{rcm})$ the receiver recomputes the note commitment
$\mathsf{cm}$ and confirms its extracted form $\mathsf{cm}^\star$
appears in their local copy of the note commitment tree, yielding the
authentication path and the leaf position.

### 4.2 The Decrypted Note Is Exactly the Missing Half of the Action Witness

Chapter 5, Definition 2.2 lists the Action circuit's witness. Sort each
component by where it comes from at spend time:

| Witness component                                                                                                                                 | Source                                              |
| ------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| $g_{d,\mathsf{old}}$                                                                                                                              | decrypted $d$, then $\mathsf{DiversifyHash}$        |
| $v_{\mathsf{old}}$                                                                                                                                | decrypted plaintext                                 |
| $\mathsf{rcm}_{\mathsf{old}}$, $\psi_{\mathsf{old}}$                                                                                              | derived from decrypted $\mathsf{rseed}$             |
| $\rho_{\mathsf{old}}$                                                                                                                             | on-chain $\mathsf{nf}$ of the funding Action        |
| $\mathsf{pk_d}_{,\mathsf{old}}$                                                                                                                   | re-derived as $[\mathsf{ivk}]\, g_{d,\mathsf{old}}$ |
| Merkle path and position                                                                                                                          | receiver's local note commitment tree               |
| $\mathsf{ak}$, $\mathsf{nk}$                                                                                                                      | receiver's key tree (via $\mathsf{ask}$)            |
| $\mathsf{rivk}$                                                                                                                                   | receiver's FVK                                      |
| $\alpha$                                                                                                                                          | freshly sampled per Action                          |
| new-note fields ($g_{d,\mathsf{new}}$, $\mathsf{pk_d}_{,\mathsf{new}}$, $v_{\mathsf{new}}$, $\psi_{\mathsf{new}}$, $\mathsf{rcm}_{\mathsf{new}}$) | sender-chosen, for the next recipient               |
| $\mathsf{rcv}$                                                                                                                                    | freshly sampled per Action                          |

The first five rows are precisely what an earlier sender supplied
through the ciphertext and the chain. The next three come from the
receiver's own key tree. $\alpha$ and $\mathsf{rcv}$ are local
randomness. The new-note rows describe whoever the receiver is now
paying and are unrelated to the original decryption. Nothing about
spending requires further coordination with the original sender; the
DH-encrypted plaintext is exactly the in-band channel that delivers the
note-specific half of the Action witness, sized to the bytes the
receiver could not have otherwise reconstructed.

The Action circuit re-derives $\mathsf{pk_d}_{,\mathsf{old}}$ from
$\mathsf{ivk}$ in zero knowledge:
[`src/circuit.rs#L580-L622`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L580-L622)
runs a `CommitIvk` computation, multiplies $g_{d,\mathsf{old}}$ by the
resulting scalar, and constrains the product to equal the witnessed
$\mathsf{pk_d}_{,\mathsf{old}}$. Any inconsistency between the
decrypted note and the receiver's $\mathsf{ivk}$ surfaces as a witness
that fails this equality (and, downstream, the note-commitment check in
constraint group 4 of chapter 5).

### 4.3 Why $\mathsf{pk_d}_{,\mathsf{new}}$ Can Be Witnessed Unchecked

For the new note inside the same Action, the circuit witnesses
$\mathsf{pk_d}_{,\mathsf{new}}$ without checking that it equals
$[\mathsf{ivk}']\, g_{d,\mathsf{new}}$ for any
$\mathsf{ivk}'$ (see
[`src/circuit.rs#L665-L673`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L665-L673)):
the value is wrapped in `NonIdentityPoint::new`, which forces it on
curve and non-identity, and that is the entire check the circuit
performs on the recipient key. The reason is the same as the asymmetry
Sapling encodes across its separate Spend and Output circuits: the
sender alone cannot authenticate the recipient. The sender has only
the public address $(d, \mathsf{pk_d})$, no signature from the
recipient, and no way to prove that $\mathsf{pk_d}$ corresponds to a
real $\mathsf{ivk}$.

The Action circuit therefore constrains only what the sender can prove
unilaterally for the new note:

- $\mathsf{cv}^{\mathsf{net}}$ is a valid value commitment to
  $v_{\mathsf{old}} - v_{\mathsf{new}}$ under trapdoor $\mathsf{rcv}$
  (constraint group 5);
- $\mathsf{cm}^\star_{\mathsf{new}}$ is the extracted Sinsemilla
  commitment to $(g_{d,\mathsf{new}}, \mathsf{pk_d}_{,\mathsf{new}},
  v_{\mathsf{new}}, \rho_{\mathsf{new}}, \psi_{\mathsf{new}},
  \mathsf{rcm}_{\mathsf{new}})$ (constraint group 4);
- $\rho_{\mathsf{new}} = \mathsf{nf}_{\mathsf{old}}$, the bind that
  chains successive notes inside the same Action
  ([`src/circuit.rs#L676`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L676));
- $g_{d,\mathsf{new}}$ is on curve and non-identity.

A malformed $\mathsf{pk_d}_{,\mathsf{new}}$ (off curve, in the
small-order subgroup, or simply not anyone's diversified key) still
produces a valid Action proof and is accepted by consensus. The cost is
borne entirely by the named recipient: no $\mathsf{ivk}$ they hold
lets them derive a matching $\mathsf{pk_d}_{,\mathsf{old}}$ inside a
future Action under the input-side check from
[Section 4.2](#42-the-decrypted-note-is-exactly-the-missing-half-of-the-action-witness),
so the new note becomes unspendable change and the sender has burned
their funds.

Authentication of the recipient is therefore deferred to spend time,
where it appears as

$$
\mathsf{pk_d}_{,\mathsf{old}} = [\mathsf{ivk}]\, g_{d,\mathsf{old}}
$$

inside a future Action. The two checks (off-chain decryption with
$\mathsf{ivk}$, in-circuit re-derivation of
$\mathsf{pk_d}_{,\mathsf{old}}$) lock the same identity at two
different moments, with no need for the original sender to participate
in either.

Unlike Sapling, this asymmetry lives inside a single circuit rather
than across the Spend / Output divide. The Action circuit charges the
spender for the full input-side authentication (Merkle membership,
$\mathsf{pk_d}_{,\mathsf{old}}$ derivation, nullifier integrity, spend
authorisation) and only commits the sender to the fields they can
prove unilaterally on the output side. The unified circuit hides which
half of an Action is real, but the cryptographic load is still split
along the same line.

## 5. Failure Modes

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

## 6. Spec Pointers

- [Zcash Protocol Specification, Section 4.18](https://zips.z.cash/protocol/protocol.pdf):
  Note encryption.
- [ZIP 212](https://zips.z.cash/zip-0212):
  the deterministic randomness derivation that closes the
  malleability gap.
- [`zcash_note_encryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption):
  the shared framework.

## 7. Exercises

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

## 8. Further Reading

- [ZIP 316](https://zips.z.cash/zip-0316):
  Unified Addresses, which encapsulate the diversifier metadata
  the encryption depends on.
- [Bellare-Rogaway's Authenticated Encryption with Associated
  Data](https://web.cs.ucdavis.edu/~rogaway/papers/keywrap.pdf)
  is the formal model that ChaCha20-Poly1305 instantiates.
