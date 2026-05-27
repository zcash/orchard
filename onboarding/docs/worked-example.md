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

The snippet lives at
[`onboarding/data/print_bundle.rs`](https://github.com/dannywillems/orchard/blob/onboarding/onboarding/data/print_bundle.rs)
and is embedded below from the file (it is not maintained
in this Markdown):

```rust reference title="onboarding/data/print_bundle.rs"
https://github.com/dannywillems/orchard/blob/onboarding/onboarding/data/print_bundle.rs#L1-L42
```

Run `cargo test --release --test builder -- --nocapture` and
match the printed lengths against the table in Section 2. The
actual hex values will differ between runs (the prover samples
fresh randomness), but the lengths are invariant.

## 6. A Real Test-Vector Walkthrough

For a fully-deterministic, byte-exact example that anyone can
reproduce without an explorer API, use the official ZIP 244 test
vectors. Two files in this repository support this section:

- [`onboarding/data/zip_0244.json`](https://github.com/dannywillems/orchard/blob/onboarding/onboarding/data/zip_0244.json):
  a local mirror of
  [`test-vectors/zcash/zip_0244.json`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/test-vectors/zcash/zip_0244.json)
  from
  [`zcash-hackworks/zcash-test-vectors`](https://github.com/zcash-hackworks/zcash-test-vectors).
  The file holds ten complete v5 transactions in hex form, with
  their txid and SIGHASH digests already computed. These vectors
  are part of the official cross-implementation test corpus that
  `zcashd`, `zebra`, `librustzcash`, and `zcash/orchard` are all
  required to agree on. The mirror exists so the example does
  not break if the upstream repository is reorganised or taken
  down.
- [`onboarding/data/parse_v5.py`](https://github.com/dannywillems/orchard/blob/onboarding/onboarding/data/parse_v5.py):
  a self-contained Python 3 parser that consumes the JSON file
  and produces the byte dump shown below. ~150 lines, zero
  third-party dependencies. Used in
  [Section 6.7](#67-decoding-this-yourself).

The Python *generators* that produced the JSON (the reference
implementations of Sinsemilla, Poseidon, the Orchard note model,
and the v5 transaction encoder) live upstream in
[`zcash-hackworks/zcash-test-vectors`](https://github.com/zcash-hackworks/zcash-test-vectors)
under
[`zcash_test_vectors/orchard/`](https://github.com/zcash-hackworks/zcash-test-vectors/tree/master/zcash_test_vectors/orchard).
A cryptographer wanting a second reference implementation of the
Orchard primitives should read these alongside the Rust crate.
The relevant files:

- [`commitments.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/commitments.py),
  [`note.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/note.py):
  note structure and `NoteCommit^Orchard`.
- [`key_components.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/key_components.py),
  [`zip32.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/zip32.py):
  the key tree.
- [`sinsemilla.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/sinsemilla.py),
  [`poseidon.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/poseidon.py):
  the algebraic hashes.
- [`merkle_tree.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/merkle_tree.py):
  the Merkle CRH and the empty roots.
- [`note_encryption.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/note_encryption.py):
  the KDF, AEAD, and outgoing ciphertext.
- [`pallas.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/pallas.py),
  [`iso_pallas.py`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/zcash_test_vectors/orchard/iso_pallas.py):
  Pallas curve arithmetic and the iso-Pallas hash-to-curve.

These are pure Python; they do not depend on SageMath. (Earlier
Zcash work, e.g. some of the Sapling spec arithmetic, did ship
SageMath scripts; the Orchard generators are Python-only.)

### 6.1 The Transaction

We dissect the test vector at index 2 of the file. It is a v5
transaction with **3 Orchard Actions** (the cleanest illustration
of the per-Action structure short of a real mainnet bundle):

- **txid**:
  `f5e34d3ef76b6dd5f0d536997d10c989b24ba0e4f6e10f731ac35edd375269ae`.
  Upstream source URL of this exact vector:
  [`zcash-hackworks/zcash-test-vectors/blob/master/test-vectors/zcash/zip_0244.json`](https://github.com/zcash-hackworks/zcash-test-vectors/blob/master/test-vectors/zcash/zip_0244.json)
  (entry index 4 in the JSON, which is vector 2 after skipping
  the two metadata rows).
- **Total size**: 3,102 bytes.
- **Header**: `0x05000080` (v5 with the overwinter bit set), per
  [ZIP 225](https://zips.z.cash/zip-0225).
- **Transparent**: 1 input, 0 outputs.
- **Sapling**: no spends, no outputs.
- **Orchard region**: 3,031 bytes.

To reproduce the byte dump locally:

```bash
git clone https://github.com/dannywillems/orchard
cd orchard
git checkout onboarding
python3 onboarding/data/parse_v5.py 2
```

The full breakdown follows.

### 6.2 Action[0] (820 Bytes)

Starts at offset 72 in the transaction.

```
+  0 cv_net  faa19283702811bca8fa9c52c128785d
              5d3ddc1da409b44a033001fc1543133f
+ 32 nf      6a9d49dd9f47085b1f3e8f977ce5f7a6
              f6605223d5ba7ae0ab9025b73bc03f3f
+ 64 rk      1ac884e9473ecf636030919525dbaea7
              1e7274d1c2ccbb4a2b740a35aa3a5c3d
+ 96 cmx     5d06a6241bc05bbccdf9fef59a95589c
              1a336203594094f82833d7445fe2d011
+128 epk     5d7d8cb349e2f9c24b5f7e77f2e1f15e
              da49ed2155106329d7e215e1741f373f
+160 enc[:16]  f7c0d2324847cce1405def7c469b0e27
+724 enc tag d18f7b855992632e2c76c0fbf1ef963e
+740 out[:16]  a80e3223de3277bc559251725829ec03
+804 out tag cb4f90ba83a9e49601b194042f2900d9
```

### 6.3 Action[1] (820 Bytes)

Starts at offset 892.

```
cv_net  6d1856dc27ed57b5c7e2491953ac43ae15887d94ad572827d90ea6c9f9da2200
nf      4396b3be1b409da4bd69063faa7b6e79de45885649bae36de34def8fcec85303
rk      64024749d3053475a2c2d1d8f695a07a1a2487d5397cee8483dd8f3e96338d91
cmx     01ae9d8ad3070c2b1a91573af5e0c5e4cbbf4acdc6b54c9272200d9970250c17
epk     4211a8b71a7d8e8cf1bbea0f674b6e97e60e0c330321972ccf916ecc8a70d981
enc[:16]  22db70e6669080b9816b2232c81a4c66
enc tag  692598a6047c23c4c01400f1ab5730ea
out[:16]  c0ae8d5843d5051c376240172af218d7
out tag  fb9e5d2744ea8848b2623ac07f8ef61a
```

### 6.4 Action[2] (820 Bytes)

Starts at offset 1712.

```
cv_net  805d4d4f644d91712c0a1c222d0549fdbeacf21a6dc40e5a00cf1e05234dba19
nf      2d51938d28b89f60eca8ed2ace91caa5a8af4ee6d00540657fe32914103b5d18
rk      0be5dcce5d3ff7d6e950061dab9aeab28105916beb318d7b82a129a40a2f0396
cmx     139ae350764ef26b3494223135962304c73c0018ca5b69411297732a4e1aa91a
epk     2240513058dc334b4b744ad923818a2fee7c263b0d1e4b79d90ed3a8f2491018
enc[:16]  14f3d8be2b9823d342f46213e942a7e1
enc tag  42ff69d9b2f180be12ed75344a395aa1
out[:16]  0f852f083ad64ef40e9c0309e9bba54b
out tag  32b8505775108dc85e2ade2eac1e636e
```

### 6.5 Bundle-Level Fields

After the three Actions (offset 2532):

```
flagsOrchard          0x02
                      bit 0 (enableSpends)  = 0
                      bit 1 (enableOutputs) = 1
                      A pure receive (this bundle creates real
                      output notes but every spend is a dummy).

valueBalanceOrchard   2815d656d0db0200
                      = 804637809972520 zatoshi (synthetic value
                      for the test corpus; mainnet bundles
                      typically have |value_balance| < 1e16).

anchorOrchard         feb73271500be1722f737da9db24e9dca6cf8445
                      589653262020c33bf7803138

sizeProofsOrchard     CompactSize 270  (one byte: 0xfd is not
                      needed because the value fits)

proofsOrchard         270 bytes
                      0707de072068c170570327e6d9f5c6dd...
                      ...ff36695e802cbcb6b58c1ba7ed5eacfa

spendAuthSig[0]       64 bytes  a6544dd0634ea1aa5900b515150d12e2...
spendAuthSig[1]       64 bytes  772ecd148bc8567439e75332cc281e78...
spendAuthSig[2]       64 bytes  485e9720aaaedbb2e010ebd667bd832c...

bindingSigOrchard     64 bytes  367886f2269544d59860df33b51100bd...
```

### 6.6 Cross-Checks the Reader Can Run

Each of these is one line in a Python REPL given the raw
transaction hex.

- **Field sizes match Section 2**. Each Action is exactly 820
  bytes (32 + 32 + 32 + 32 + 32 + 580 + 80). Each signature is
  exactly 64 bytes. The bundle has $1 + 3 \cdot 820 + 1 + 8 + 32
  + 2 + 270 + 3 \cdot 64 + 64 = 2868 + 270 = 3138$ ... actually
  $1 + 2460 + 1 + 8 + 32 + 2 + 270 + 192 + 64 = 3030$ Orchard
  bytes plus the 1-byte CompactSize for `nActionsOrchard` = 3031
  total. Matches the parser's "Orchard region total: 3031".
- **Distinct nullifiers**. The three nf values above are
  pairwise distinct. If they were not, the bundle would violate
  the internal-disjointness clause of
  [Chapter 18 Definition 2.3](./18-shielded-transfers.md#definition-23-bundle-validity).
- **Distinct rk**. The three rk values are pairwise distinct, as
  required by the per-Action re-randomisation argument in
  [Chapter 14](./14-redpallas.md).
- **Distinct epk**. The three epk values are pairwise distinct,
  which is what allows the fixed-nonce ChaCha20-Poly1305 to be
  safe (one key per output).

### 6.7 Decoding This Yourself

The full parser is committed at
[`onboarding/data/parse_v5.py`](https://github.com/dannywillems/orchard/blob/onboarding/onboarding/data/parse_v5.py).
It is ~150 lines of pure Python 3 with no third-party
dependencies; it follows the v5 layout from
[ZIP 225](https://zips.z.cash/zip-0225) byte for byte. Usage:

```bash
python3 onboarding/data/parse_v5.py 2  # vector indices with
                                       # Orchard data: 2, 4, 5, 6
```

The script reads the bundled
[`onboarding/data/zip_0244.json`](https://github.com/dannywillems/orchard/blob/onboarding/onboarding/data/zip_0244.json)
by path, selects the requested vector, and prints the field
table reproduced above. Rather than inline the parser body here
(which would inevitably drift from the source), the doc cites
the file directly:

```rust reference title="onboarding/data/parse_v5.py"
https://github.com/dannywillems/orchard/blob/onboarding/onboarding/data/parse_v5.py#L1-L20
```

(The header comment shows the usage; the rest of the file is at
the link above.) The onboarding CI workflow runs the script and
its linters on every push (see
[Section 6.8](#68-ci-coverage-for-the-python-tooling)) so the
example cannot silently rot.

### 6.8 CI Coverage for the Python Tooling

The
[`.github/workflows/onboarding-docs.yml`](https://github.com/dannywillems/orchard/blob/onboarding/.github/workflows/onboarding-docs.yml)
workflow runs three checks against
[`onboarding/data/parse_v5.py`](https://github.com/dannywillems/orchard/blob/onboarding/onboarding/data/parse_v5.py)
on every push to the `onboarding` branch, before the Docusaurus
build:

1. `ruff format --check onboarding/data` to ensure the parser is
   formatted with `ruff format` (Black-compatible).
2. `ruff check onboarding/data` to lint with a curated rule set
   (pycodestyle, pyflakes, isort, etc.).
3. `python3 onboarding/data/parse_v5.py 2` to actually execute
   the parser against the bundled JSON and assert it emits the
   expected `nActionsOrchard (CS)  3` line and exits with code 0.

If any of these fail, the deploy is aborted, so the doc never
ships with a broken parser or a divergent JSON mirror.

### 6.9 What These Vectors Are, and What They Are Not

These are **synthetic** v5 transactions generated by the
[`zcash-test-vectors`](https://github.com/zcash-hackworks/zcash-test-vectors)
generator. They:

- exercise the wire format exhaustively, including edge cases;
- are bit-exact reference outputs that every implementation
  must reproduce;
- are not on chain.

To dissect an actual on-chain transaction, use the same parser
against raw hex from a mainnet block. See Section 7 below.

## 7. On-Chain Decoding

To inspect a real on-chain Orchard transaction, fetch the raw
transaction hex from any Zcash full node, locate the Orchard
region per
[ZIP 225 "Version 5 Transaction Format"](https://zips.z.cash/zip-0225),
and walk it byte by byte against the table in Section 3 or with
the parser in Section 6.7.

Most public Zcash explorers
([blockchair.com/zcash](https://blockchair.com/zcash),
[mainnet.zcashexplorer.app](https://mainnet.zcashexplorer.app),
[blockexplorer.com](https://blockexplorer.com/),
[explorer.zcha.in](https://explorer.zcha.in))
expose the raw transaction hex under a "Raw" or "API" tab.
Anonymous API access is rate-limited; an API key is typically
required for sustained access. Alternatively, run a local
`zebrad` or `zcashd` node and query its JSON-RPC
(`getrawtransaction <txid> 0`).

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

## 8. What Stays the Same, What Varies

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
