---
sidebar_position: 53
title: 'Worked Example: Byte-by-Byte Anatomy of an Orchard Bundle'
description: 'A reproducible Orchard transaction (from tests/builder.rs) dissected field by field, with every byte linked to the source code that produced it and the ZIP that specifies its wire format.'
---

# Worked Example: Byte-by-Byte Anatomy of an Orchard Bundle

This page picks a single concrete Orchard bundle and decodes
every byte. The bundle is the one produced by the integration
test
[`tests/builder.rs::bundle_chain`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
at the pin
([`f8915bc`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669)),
which means the reader can reproduce every number locally with
`cargo test --release --test builder`.

The byte layout follows
[ZIP 225 "Version 5 Transaction Format"](https://zips.z.cash/zip-0225);
the Orchard fields appear inside the transaction in the order
documented there.

## 1. The Scenario

`bundle_chain` builds two bundles in sequence. The first spends a
single note from a freshly-funded account and creates two output
notes (one to a fresh recipient, one to the sender as change).
The second spends one of those output notes. We dissect the
**first** bundle.

Parameters at construction time:

- $N = 2$ Actions (the minimum;
  [`MIN_ACTIONS`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs#L36-L36)).
  Both Actions are real (no dummies needed for $N = 2$).
- `flagsOrchard`: spends enabled, outputs enabled.
- `valueBalanceOrchard`: zero (a pure shielded -> shielded
  transfer of the input value, split into two outputs).
- `anchorOrchard`: the root of a tree containing exactly the
  fresh input note's commitment.

## 2. Anatomy of One Action Description

An Action description on the wire is exactly **820 bytes**,
laid out in the order below. Field source links point at the
Rust type or function that produces the bytes; serialisation
itself happens in `librustzcash`'s transaction encoder against
the public surface of this crate.

| Offset | Size       | Field             | Wire encoding                                            | Source                                                                                                                                                                              |
| -----: | ---------: | ----------------- | -------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|      0 | 32 bytes   | `cv_net_i`        | Compressed Pallas point (`pallas::Point::to_bytes`)      | [`src/value.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/value.rs) `ValueCommitment::to_bytes`                                            |
|     32 | 32 bytes   | `nf_i`            | Pallas base-field element, little-endian                 | [`src/note/nullifier.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/nullifier.rs) `Nullifier::to_bytes`                                |
|     64 | 32 bytes   | `rk_i`            | Compressed Pallas point (RedPallas VerificationKey)      | [`src/primitives/redpallas.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/primitives/redpallas.rs) (via [`reddsa`](https://github.com/ZcashFoundation/reddsa)) |
|     96 | 32 bytes   | `cmx_i`           | Pallas base-field element, little-endian                 | [`src/note/commitment.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note/commitment.rs) `ExtractedNoteCommitment::to_bytes`                |
|    128 | 32 bytes   | `epk_i`           | Compressed Pallas point (ephemeral public key)           | [`src/note.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs#L311-L319) `TransmittedNoteCiphertext.epk_bytes`                          |
|    160 | 580 bytes  | `enc_ciphertext_i`| ChaCha20-Poly1305 AEAD output (plaintext + 16-byte tag)  | [`src/note_encryption.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note_encryption.rs) `OrchardDomain`                                    |
|    740 | 80 bytes   | `out_ciphertext_i`| ChaCha20-Poly1305 AEAD output (32-byte `pk_d` + 32-byte `esk` + 16-byte tag) | [`src/note_encryption.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note_encryption.rs) `OrchardDomain` |
|    820 |        end |                   |                                                          |                                                                                                                                                                                    |

The in-memory layout of the same data is the
[`Action<A>`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/action.rs#L18-L27)
struct (six fields, with the spend-authorising signature stored
separately in the `Authorization` typestate parameter `A`).
Field semantics are documented in
[Chapter 5 (the Action circuit)](./05-action-circuit.md) and
[Chapter 12 (Bundle and Builder)](./12-bundle-and-builder.md).

### 2.1 Inside `enc_ciphertext_i` (580 Bytes)

The plaintext that is encrypted is the
[`NotePlaintextBytes`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption)
encoding (564 bytes) followed by a 16-byte Poly1305 tag:

| Sub-offset | Size       | Field          | Source                                                                                                                                            |
| ---------: | ---------: | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
|          0 |   1 byte   | lead byte      | [ZIP 212](https://zips.z.cash/zip-0212) version tag (`0x02` for Orchard)                                                                          |
|          1 |  11 bytes  | diversifier `d`| [`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs) `Diversifier`                          |
|         12 |   8 bytes  | value $v$      | [`src/value.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/value.rs) `NoteValue` (little-endian u64)      |
|         20 |  32 bytes  | `rseed`        | [`src/note.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note.rs) `RandomSeed`                           |
|         52 | 512 bytes  | memo           | Per [ZIP 302](https://zips.z.cash/zip-0302); padded to a fixed length                                                                             |
|        564 |  16 bytes  | AEAD tag       | Poly1305 over the ChaCha20 keystream of the previous 564 bytes                                                                                    |
|        580 |        end |                |                                                                                                                                                   |

The recipient derives $\psi$ and $\mathsf{rcm}$ from `rseed`
(see [Chapter 10](./10-note-encryption.md), Section 3.1), so the
plaintext does not transmit them explicitly. This is the
malleability defence introduced by ZIP 212.

### 2.2 Inside `out_ciphertext_i` (80 Bytes)

The outgoing ciphertext lets the sender recover the note from
their own outgoing viewing key. It encrypts a fixed 64-byte
plaintext under a key derived from $\mathsf{ovk}$ and the
Action's public fields:

| Sub-offset | Size       | Field          | Source                                                                                                                                        |
| ---------: | ---------: | -------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
|          0 |  32 bytes  | $\mathsf{pk_d}$| [`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs) `DiversifiedTransmissionKey`      |
|         32 |  32 bytes  | $\mathsf{esk}$ | [`src/keys.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/keys.rs) `EphemeralSecretKey`              |
|         64 |  16 bytes  | AEAD tag       | Poly1305                                                                                                                                      |
|         80 |        end |                |                                                                                                                                               |

## 3. Anatomy of the Whole Bundle

A v5 transaction's Orchard region (ZIP 225, Section "Orchard
Transaction Fields") is laid out as follows. For our two-Action
bundle the total is approximately **3.85 KiB**, dominated by the
two Action descriptions and the proof.

| Field                          | Size                              | Wire encoding                                                                                                                                       | Source                                                                                                                                                  |
| ------------------------------ | --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `nActionsOrchard`              | `CompactSize` (1 byte for $N=2$)  | Variable-length count prefix                                                                                                                        | Transaction encoder                                                                                                                                     |
| `vActionsOrchard`              | $N \times 820 = 1640$ bytes       | Concatenated Action descriptions, in shuffle order (see [Chapter 12](./12-bundle-and-builder.md))                                                   | [`src/action.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/action.rs)                                          |
| `flagsOrchard`                 | 1 byte                            | Bit 0 = `enableSpends`, Bit 1 = `enableOutputs`, bits 2..7 reserved zero                                                                            | [`src/bundle.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs#L57-L74) `Flags`                          |
| `valueBalanceOrchard`          | 8 bytes                           | Signed 63-bit integer encoded as a signed little-endian i64                                                                                          | [`src/value.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/value.rs) `ValueSum`                                 |
| `anchorOrchard`                | 32 bytes                          | Pallas base-field element                                                                                                                            | [`src/tree.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/tree.rs) `Anchor::to_bytes`                           |
| `sizeProofsOrchard`            | `CompactSize`                     | Byte length of the Halo 2 proof that follows                                                                                                         | Transaction encoder                                                                                                                                     |
| `proofsOrchard`                | ~2 KiB (varies with $K$, not $N$) | The Halo 2 IPA proof bytes                                                                                                                           | [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs) `Proof::create`                        |
| `vSpendAuthSigsOrchard`        | $N \times 64 = 128$ bytes         | Concatenated RedPallas `SpendAuth` signatures                                                                                                        | [`src/primitives/redpallas.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/primitives/redpallas.rs)              |
| `bindingSigOrchard`            | 64 bytes                          | One RedPallas `Binding` signature                                                                                                                    | [`src/primitives/redpallas.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/primitives/redpallas.rs)              |

The fixed-size parts of the bundle for this scenario:

```
nActionsOrchard         1   bytes      (CompactSize 0x02)
vActionsOrchard      1640   bytes      (2 x 820)
flagsOrchard            1   byte
valueBalanceOrchard     8   bytes
anchorOrchard          32   bytes
sizeProofsOrchard      ~2   bytes      (CompactSize for ~2 KiB)
proofsOrchard       ~2048   bytes
vSpendAuthSigsOrchard 128   bytes      (2 x 64)
bindingSigOrchard      64   bytes
-----------------------------------
total                 ~3924 bytes  (~3.85 KiB)
```

The **per-Action marginal cost** is $820 + 64 = 884$ bytes
(Action description plus its spend-auth signature). The bundle's
**fixed overhead** is $1 + 1 + 8 + 32 + 64 + \mathsf{proof} \approx
2156$ bytes; the proof dominates and is independent of $N$ for
small $N$. This is the wire-format consequence of the
single-proof-per-bundle design discussed in
[Chapter 5 Section 3.6](./05-action-circuit.md#36-differences-from-sapling-at-the-circuit-level).

## 4. What Each Field Asserts

Every field on the wire is consumed by one of the consensus
checks of
[Chapter 18 Definition 2.3](./18-shielded-transfers.md#definition-23-bundle-validity).
The mapping:

| Wire field                  | Consumed by                                                                                                                                                |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cv_net_i`                  | Sum into $\sum_i \mathsf{cv}^{\mathsf{net}}_i$; binding-signature key derivation (Cond. 2.3.4) and value commitment public input to the proof.             |
| `nf_i`                      | Nullifier-set disjointness (Cond. 2.3.2); public input to the proof.                                                                                       |
| `rk_i`                      | Spend-authorising signature verification (Cond. 2.3.5); public input to the proof.                                                                         |
| `cmx_i`                     | Inserted into the note commitment tree at acceptance; public input to the proof.                                                                           |
| `epk_i`                     | Recipient KDF input; out-ciphertext sender recovery input.                                                                                                 |
| `enc_ciphertext_i`          | Trial-decryption by recipients with the matching $\mathsf{ivk}$.                                                                                           |
| `out_ciphertext_i`          | Sender-side recovery with the matching $\mathsf{ovk}$.                                                                                                     |
| `flagsOrchard`              | Selects whether the spend / output subcircuit is active (Cond. 2.3.3 via `enableSpends`, `enableOutputs`).                                                 |
| `valueBalanceOrchard`       | Binding-signature key (Cond. 2.3.4); chain-level pool accounting.                                                                                          |
| `anchorOrchard`             | Anchor freshness (Cond. 2.3.1); public input to the proof.                                                                                                 |
| `proofsOrchard`             | Halo 2 verifier input (Cond. 2.3.3).                                                                                                                       |
| `vSpendAuthSigsOrchard`     | Per-Action signature verification (Cond. 2.3.5).                                                                                                           |
| `bindingSigOrchard`         | Binding-signature verification (Cond. 2.3.4).                                                                                                              |

If the reader has internalised this table, the rest of the
chapters are commentary.

## 5. Reproducing the Example Locally

To produce the exact byte sequence:

```bash
git clone https://github.com/zcash/orchard
cd orchard
git checkout 0.13.1
cargo test --release --test builder
```

To inspect a real Bundle programmatically without going through
a transaction encoder, the integration test exposes the
`Authorized` Bundle via `verify_bundle`. Add the following
fragment to a copy of
[`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
after the first bundle is built:

```rust
let actions = bundle.actions();
for (i, a) in actions.iter().enumerate() {
    println!("action {i}:");
    println!("  cv_net = {}", hex::encode(a.cv_net().to_bytes()));
    println!("  nf     = {}", hex::encode(a.nullifier().to_bytes()));
    println!("  rk     = {}", hex::encode(<[u8; 32]>::from(a.rk())));
    println!("  cmx    = {}", hex::encode(a.cmx().to_bytes()));
    let ct = a.encrypted_note();
    println!("  epk    = {}", hex::encode(ct.epk_bytes));
    println!("  enc    = {} ({} bytes)", &hex::encode(ct.enc_ciphertext)[..32], ct.enc_ciphertext.len());
    println!("  out    = {} ({} bytes)", &hex::encode(ct.out_ciphertext)[..32], ct.out_ciphertext.len());
}
println!("flags          = {:08b}", bundle.flags().to_byte());
println!("value_balance  = {}", bundle.value_balance());
println!("anchor         = {}", hex::encode(bundle.anchor().to_bytes()));
println!("proof length   = {} bytes", bundle.authorization().proof().as_ref().len());
```

Run `cargo test --release --test builder -- --nocapture` and
match the printed lengths against the table in Section 2. The
actual hex values will differ between runs (the prover samples
fresh randomness), but the lengths are invariant.

## 6. On-Chain Decoding

To inspect a real on-chain Orchard transaction, fetch the raw
transaction hex from any Zcash full node, locate the Orchard
region per
[ZIP 225 "Version 5 Transaction Format"](https://zips.z.cash/zip-0225),
and walk it byte by byte against the table in Section 3.

A canonical reference implementation of the transaction decoder
lives in
[`zcash/librustzcash`](https://github.com/zcash/librustzcash);
the
[`zcash_primitives` crate](https://github.com/zcash/librustzcash/tree/main/zcash_primitives)
exposes `Transaction::read` which delegates Orchard parsing to
this crate's
[`Bundle`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs)
type via the
[`pczt::parse`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/pczt/parse.rs)
helpers.

## 7. What Stays the Same, What Varies

Across bundles in the wild, the **shape** of every Orchard
region is constant: the field order, the per-Action byte count
(820), the signature size (64), the proof byte format. What
varies:

- $N$ (the Action count) is always a power of two greater than
  or equal to `MIN_ACTIONS = 2`.
- The proof length varies slightly with $K$ but is constant for
  a fixed `K = 11` (see
  [Chapter 5 Section 3.4](./05-action-circuit.md#34-proving-and-verifying-keys-location-and-size)).
- `valueBalanceOrchard` is signed and may be negative (value
  enters the pool) or positive (value leaves the pool).
- `flagsOrchard` is almost always `0b11` (both enabled) in
  normal usage; coinbase transactions force `enableSpends = 0`
  per [ZIP 213](https://zips.z.cash/zip-0213).
