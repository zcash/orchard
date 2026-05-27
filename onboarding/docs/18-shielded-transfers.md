---
sidebar_position: 18
title: From Actions to Shielded Transfers
description: How Action circuits compose into bundles, and how bundles realise a shielded transfer at the consensus layer.
---

# From Actions to Shielded Transfers

## 1. Why This Chapter Exists

The previous chapters dissected one Action: its public inputs,
its witness, its constraints, the keys it derives. A
**shielded transfer**, however, is a chain-level state transition
that consumes a set of input notes and produces a set of output
notes, possibly with transparent ingress or egress. The transfer
is not in any one file: it is the joint effect of the Halo 2
proof, the binding signature, the per-Action spend
authorisations, the nullifier update, and the note-tree update.
After this chapter the reader can trace a payment from "wallet
calls `Builder::add_output`" to "the recipient's wallet sees an
incoming note" and identify every cryptographic check that
licenses that transition.

## 2. Definitions

### Definition 2.1 (Orchard State)

At block height $h$, the Orchard state is the triple

$$
\mathsf{State}_h = \big(\mathsf{CMTree}_h,\, \mathsf{NfSet}_h,\, \mathsf{Pool}_h\big),
$$

where $\mathsf{CMTree}_h$ is the Sinsemilla-hashed Merkle tree of
all note commitments inserted up to height $h$ (see
[Chapter 11](./11-merkle-tree.md)), $\mathsf{NfSet}_h \subset
\mathbb{F}_p$ is the set of revealed nullifiers, and
$\mathsf{Pool}_h \in \mathbb{Z}$ is the running net value held
inside the Orchard pool (sum of all $\mathsf{value\_balance}$
values applied so far).

### Definition 2.2 (Bundle Transfer Semantics)

A valid Bundle $B$ at height $h$ defines a state transition
$\mathsf{State}_h \to \mathsf{State}_{h+1}$:

$$
\begin{aligned}
\mathsf{CMTree}_{h+1} &= \mathsf{CMTree}_h \cup \{\mathsf{cm}^\star_i\}_{i=1}^N, \\
\mathsf{NfSet}_{h+1}  &= \mathsf{NfSet}_h \cup \{\mathsf{nf}_i\}_{i=1}^N, \\
\mathsf{Pool}_{h+1}   &= \mathsf{Pool}_h - \mathsf{value\_balance}(B).
\end{aligned}
$$

The transition is *valid* only if all five conditions below hold;
otherwise the transaction containing $B$ is rejected by every
honest validator.

### Definition 2.3 (Bundle Validity)

A Bundle $B = (\{A_i\}, \pi, \sigma^{\mathsf{bind}},
\{\sigma_i^{\mathsf{auth}}\}, \mathsf{value\_balance})$ is valid
at height $h$ iff:

1. **Anchor freshness**: every Action's anchor $\mathsf{anchor}_i$
   is the root of $\mathsf{CMTree}_{h'}$ for some
   $h' \in \{h - \mathsf{anchorAge}, \dots, h\}$.
2. **Nullifier freshness**: the multiset
   $\{\mathsf{nf}_i\}$ is disjoint from $\mathsf{NfSet}_h$ **and**
   contains no internal duplicates.
3. **Proof validity**:
   $\mathsf{Halo2.Verify}(\mathsf{vk},\, \{\mathsf{Inst}_i\}_{i=1}^N,\, \pi) = \mathsf{accept}$.
4. **Binding signature**:
   $\mathsf{RedPallas.Verify}_{\mathsf{Binding}}\big(\mathsf{bvk}(B),\, \mathsf{SighashBundle}(B),\, \sigma^{\mathsf{bind}}\big) = \mathsf{accept}$,
   where $\mathsf{bvk}(B) = \sum_i \mathsf{cv}^{\mathsf{net}}_i -
   [\mathsf{value\_balance}]\, \mathcal{V}$.
5. **Per-Action authorisation**: for every non-dummy Action,
   $\mathsf{RedPallas.Verify}_{\mathsf{SpendAuth}}\big(\mathsf{rk}_i,\, \mathsf{SighashTx},\, \sigma_i^{\mathsf{auth}}\big) = \mathsf{accept}$.

### Invariant 2.4 (Rho Chaining within a Bundle)

The bundle's Actions are linked by the rho-chain: for each
Action, the *new* note's $\rho$ is bound to the *old* note's
nullifier of the **same Action**. This couples each fresh note
to a specific spend within the same bundle, which is what
prevents two distinct bundles from minting two notes that share a
$\rho$ unless they share a nullifier (and would therefore both be
rejected by Condition 2.3.2).

## 3. The Code

### 3.1 The Mental Model

Read the lifecycle below as a sequence of state transitions.
Every transition is licensed by a cryptographic check; if any
check fails, the transition is reverted before the next one runs.

```
[wallet]                                [chain]
   |
   |  pick spendable notes + paths      anchor = current_root
   |  pick output recipients + values
   |  pad to power-of-two Actions
   |  shuffle pairings
   |  sample alpha_i, rcv_i, esk_i
   |  compute rho_new_i := nf_old_i  (per Action)
   |
   |  Builder::build  ----------------> Unauthorized Bundle
   |  Bundle::create_proof -----------> proof pi
   |  Bundle::prepare ----------------> typestate transition
   |  Bundle::apply_signatures -------> Authorized Bundle
   |
   |---- broadcast(tx with Bundle) --->
   |                                    consensus check:
   |                                      anchor in {recent roots}?
   |                                      nullifiers disjoint?
   |                                      proof verifies?
   |                                      bind sig verifies?
   |                                      auth sigs verify?
   |                                    if accept:
   |                                      insert cm*_i into tree
   |                                      add nf_i to nullifier set
   |                                      apply value_balance
   |
[recipient]
   |  scan every shielded output:
   |    K_enc = KDF(epk, [ivk] epk)
   |    P = AEAD.Decrypt(K_enc, C_enc)
   |    if AEAD tag valid -> recover Note
```

### 3.2 The Code Path of a Transfer

A real run of the integration test in
[`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
exercises every step:

- Spend selection and path construction:
  [`Builder::add_spend`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs)
- Output queueing:
  [`Builder::add_output`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs)
- Padding and Action assembly:
  [`Builder::build`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/builder.rs)
- Proving:
  [`Bundle::create_proof`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs)
- Spend-authorisation signing:
  [`Bundle::apply_signatures`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs)
- Verification:
  [`Bundle::verify_proof`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle.rs)
  plus
  [`bundle::commitments`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle/commitments.rs)
  for SIGHASH.
- Trial decryption:
  [`zcash_note_encryption::try_note_decryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption)
  via
  [`OrchardDomain`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/note_encryption.rs).

### 3.3 What an On-Chain Observer Sees

Per bundle, on the public ledger:

- The list of Actions, each carrying
  $(\mathsf{cv}^{\mathsf{net}}_i,\, \mathsf{nf}_i,\, \mathsf{rk}_i,\, \mathsf{cm}^\star_i,\, \mathsf{epk}_i,\, C^{\mathsf{enc}}_i,\, C^{\mathsf{out}}_i,\, \mathsf{enableSpends},\, \mathsf{enableOutputs})$.
- The Halo 2 proof $\pi$ (one per bundle, not per Action).
- The binding signature $\sigma^{\mathsf{bind}}$.
- The per-Action spend-authorising signatures
  $\{\sigma_i^{\mathsf{auth}}\}$.
- The declared $\mathsf{value\_balance}$.

What the observer does **not** see, even with unlimited
computation, under the protocol's security assumptions:

- The mapping between Actions and the senders / receivers.
- Whether a given Action is a real spend, real output, dummy
  spend with dummy output, or any combination. The Halo 2 proof
  attests that *if* enabled, the spend / output is well-formed,
  but the flags are public and the dummies look identical on the
  wire to real Actions.
- The values of any individual notes; only the net
  $\mathsf{value\_balance}$ of the whole bundle.
- The identity of the diversified address inside any Action.

### 3.4 The Joint Effect: a Transfer

Combining the above:

- Sender's wallet loses access to the **input** notes (their
  nullifiers are now public, the next attempt to spend them is
  rejected by Condition 2.3.2).
- Recipient's wallet, on the next trial-decryption pass, sees the
  **output** notes and adds them to its spendable set.
- The Orchard pool's net value shifts by $-\mathsf{value\_balance}$:
  if $\mathsf{value\_balance} > 0$, that amount leaves the pool
  (typically to a transparent recipient or a Sapling output); if
  $< 0$, that amount enters the pool from a transparent source.
- The state of every full node converges on the same updated
  $\mathsf{State}_{h+1}$.

The transfer is the joint application of these effects. No single
file in `orchard` is the "transfer"; the transfer is what the
verifier's *acceptance* of a bundle licenses on chain.

### 3.5 Replay, Linkability, and Censorship Resistance

- **Replay** is prevented by Condition 2.3.2 (nullifier
  freshness) plus the transaction-level uniqueness enforced by
  the SIGHASH binding to the transaction's other fields
  ([ZIP 244](https://zips.z.cash/zip-0244)).
- **Cross-Action linkability within a bundle** is prevented by
  the shuffle in `Builder::build` and by the per-Action
  randomiser $\alpha_i$ that decouples $\mathsf{rk}_i$ from
  $\mathsf{ak}$.
- **Cross-bundle linkability** between two of the same wallet's
  bundles is prevented by the same $\alpha_i$ argument: every
  bundle's $\mathsf{rk}_i$ is independent under the
  re-randomisation security definition of RedDSA.
- **Censorship resistance at the cryptographic layer**: a node
  can refuse to mine a transaction, but cannot prove it was
  censored. This is a property of the broader chain, not of
  Orchard specifically.

### 3.6 Composition with Other Pools

A Zcash transaction may contain transparent inputs and outputs,
a Sapling bundle, and an Orchard bundle simultaneously. The
SIGHASH ([ZIP 244](https://zips.z.cash/zip-0244)) binds them
together: the per-Action spend-auth signatures sign the
transaction-level digest, so a transaction cannot be split into a
valid sub-transaction. The Orchard pool accounting (`Pool_h`
above) is one of three pool variables that the chain maintains;
the consensus check verifies all three.

## 4. Failure Modes

- **Anchor staleness**. The anchor must be a known recent root.
  A reorg can invalidate it; wallets that build offline and
  broadcast late may produce bundles that are rejected. The
  builder does not detect this; it is a wallet UX issue.
- **Nullifier-set collision under concurrency**. Two transactions
  that spend the same note get mined in the same block; only one
  can win. The losing wallet sees its bundle invalidated. Any
  wallet must serialise spends of the same note.
- **`value_balance` overflow**. The accounting is over a signed
  63-bit quantity; reckless arithmetic in a wallet can wrap.
  The `value.rs` module is documented to be defensive but
  callers must respect the documented range.
- **Memo length leak via padding**. The 512-byte memo padding is
  fixed; a wallet that emits a smaller ciphertext (a buggy
  encoder) leaks the memo length category.
- **Side-channel on trial decryption**. The recipient's wallet
  attempts to decrypt every output; a timing-side-channel that
  distinguishes "AEAD tag failed at byte X" from "AEAD tag failed
  at byte Y" can leak the receiver. The Domain implementation in
  [`zcash_note_encryption`](https://github.com/zcash/librustzcash/tree/main/zcash_note_encryption)
  uses constant-time AEAD via `chacha20poly1305`; a custom
  decryptor must do the same.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.6](https://zips.z.cash/protocol/protocol.pdf):
  Note Tracking by recipients (the trial-decryption side of the
  transfer).
- [Zcash Protocol Specification, Section 4.20](https://zips.z.cash/protocol/protocol.pdf):
  Action descriptions (the on-chain representation).
- [Zcash Protocol Specification, Section 3.3 and 7.3](https://zips.z.cash/protocol/protocol.pdf):
  Block subsidy, transaction structure, and consensus rules.
- [ZIP 224](https://zips.z.cash/zip-0224):
  Orchard activation, including the value-balance and nullifier
  rules.
- [ZIP 244](https://zips.z.cash/zip-0244):
  Transaction Identifier Non-Malleability, the digest the per-
  Action signatures bind to.
- [ZIP 316](https://zips.z.cash/zip-0316):
  Unified Addresses; how a transfer recipient is encoded
  across pools.

## 6. Exercises

1. **Reading task**. Open
   [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs).
   Annotate the file with the Bundle Validity conditions (2.3.1
   to 2.3.5) at each `verify`-equivalent call. Identify which
   line of test code corresponds to each consensus check.
2. **Reading task**. Pick a transaction on a public block
   explorer and list its on-chain Orchard fields. Mark which of
   the Definition 2.3 checks each field feeds into.
3. **Code task**. In a unit test, build a Bundle, then mutate
   one byte of an Action's $\mathsf{cm}^\star$ before calling
   `verify_proof`. Identify the verification step that catches
   the mutation and report its error type.
4. **Code task**. Build a Bundle with two Actions that share the
   same input note (a deliberately invalid configuration). The
   test should fail before consensus; identify whether it fails
   at the builder or at the prover, and why.
5. **Research task**. Read
   [`src/bundle/commitments.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/bundle/commitments.rs)
   alongside [ZIP 244](https://zips.z.cash/zip-0244). Determine
   which transaction fields are bound by the spend-auth
   signature and which are not. Identify one field whose absence
   from the SIGHASH would allow a non-trivial replay.

## 7. Further Reading

- The
  [`librustzcash`](https://github.com/zcash/librustzcash)
  transaction-builder source, which composes Orchard with the
  transparent and Sapling pools in production.
- [The Sapling protocol paper](https://eprint.iacr.org/2018/903),
  Section 3, for a parallel treatment of pool semantics in the
  earlier protocol.
- [Reorgs and finality in Zcash](https://zips.z.cash/protocol/protocol.pdf),
  Section 3.7, for the anchor-staleness window and reorg
  recovery rules.
