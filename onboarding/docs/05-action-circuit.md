---
sidebar_position: 5
title: The Orchard Action Circuit
description: Public inputs, witness layout, and constraints of src/circuit.rs.
---

# The Orchard Action Circuit

## 1. Why This Chapter Exists

[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs)
is 1,289 lines and concentrates more cryptographic decisions per
line than any other file in the crate. A contributor who touches
it without understanding the public inputs, the witness, or the
constraint groups will produce a soundness bug. After this chapter
the reader can match each clause of the Action statement in the
Zcash Protocol Specification to a `Constraints::with_selector`
block in this file.

### A Note on the Term "Action"

Sapling has two separate description types per shielded operation,
`Spend` and `Output`, that an external observer can count and
distinguish. Orchard collapses both into a single unified
description called an **Action**, with `enableSpends` /
`enableOutputs` flags that toggle the spend or output subcircuit
on or off. The privacy gain is that an external observer cannot
tell, from the shape of the bundle, whether an Action is a real
spend with a real output, a real spend with a dummy output, a
dummy spend with a real output, or two dummies; everything looks
the same on the wire. The term was introduced in the original
Orchard proposal,
[zcash/zips#435 "Orchard (provisional name) Shielded Protocol"](https://github.com/zcash/zips/issues/435),
and normalised in
[ZIP 224](https://zips.z.cash/zip-0224) and Section 4.20 of the
Zcash Protocol Specification.

## 2. Definitions

### Vocabulary Recap (Quick Reminder)

The Action circuit is the SNARK that asserts an entire shielded
state transition in zero knowledge. Before reading the formal
definitions, the reader should have the five Zerocash-inherited
terms in scope. Each is treated in full on the
[lineage page](./protocol-lineage.md#4-the-shielded-pool-vocabulary)
and in its dedicated chapter; this is just a refresher.

- **Note** $n = (d, \mathsf{pk_d}, v, \rho, \psi, \mathsf{rcm})$:
  a shielded UTXO. Diversifier, recipient key, value, nullifier
  seed, auxiliary randomness, commitment trapdoor.
  ([Chapter 9](./09-notes-nullifiers-commitments.md))
- **Note commitment** $\mathsf{cm}$: a binding, hiding
  Sinsemilla commitment to a note. Its extracted form
  $\mathsf{cm}^\star = \mathsf{Extract}_{\mathbb{P}}(\mathsf{cm})$
  is what is inserted into the tree.
  ([Chapter 6](./06-sinsemilla.md),
  [Chapter 9](./09-notes-nullifiers-commitments.md))
- **Anchor**: the root of the global note commitment tree at
  some recent block height. A spend proves membership of its
  input note's commitment under this anchor.
  ([Chapter 11](./11-merkle-tree.md))
- **Nullifier** $\mathsf{nf}$: the deterministic, unique
  identifier of a spent note. Published on chain so the chain
  can reject double-spends without learning which note was
  spent.
  ([Chapter 9](./09-notes-nullifiers-commitments.md))
- **Value commitment** $\mathsf{cv^{\mathsf{net}}}$: a Pedersen
  commitment that hides individual note values while letting
  consensus check value conservation across the bundle via the
  binding signature.
  ([Chapter 13](./13-value-commitments.md))

If any of these are unfamiliar, read
[Background: From Zerocoin to Orchard](./protocol-lineage.md)
first; the formal definitions below assume them.

### Definition 2.1 (Public Inputs of an Action)

The instance columns of the Action circuit hold

$$
\mathsf{Inst} = (\mathsf{anchor},\, \mathsf{nf},\, \mathsf{rk}_X,\,
\mathsf{rk}_Y,\, \mathsf{cm}^\star_X,\, \mathsf{cv^{\mathsf{net}}}_X,\,
\mathsf{cv^{\mathsf{net}}}_Y,\, \mathsf{enableSpends},\,
\mathsf{enableOutputs}).
$$

### Definition 2.2 (Witness)

The private witness includes the input note path and position, the
input note's full field set, the spending authority $\mathsf{ak}$,
the re-randomiser $\alpha$, the nullifier-deriving key
$\mathsf{nk}$, $\mathsf{rivk}$, the output note's fields, and the
value commitment trapdoor $\mathsf{rcv}$.

### Definition 2.3 (Circuit Size at the Pin)

The shape of the Action circuit, as recorded in the pinned
[`src/circuit_description`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)
snapshot:

| Quantity                     | Value                            |
| ---------------------------- | -------------------------------- |
| $K$ (table height exponent)  | $11$                             |
| Rows ($2^K$)                 | $2048$                           |
| Extended $K$ (FFT domain)    | $14$                             |
| Public inputs                | $9$ cells in $1$ instance column |
| Advice columns (witness)     | $10$                             |
| Fixed columns (preprocessed) | $29$                             |
| Selectors                    | $56$                             |
| Polynomial constraints       | $193$ (see Section 3.5)          |

The nine public inputs are the cells of the single instance
column, indexed at the top of `src/circuit.rs` (lines 77 to 85):
$\mathsf{ANCHOR}$, $\mathsf{CV\_NET\_X}$, $\mathsf{CV\_NET\_Y}$,
$\mathsf{NF\_OLD}$, $\mathsf{RK\_X}$, $\mathsf{RK\_Y}$,
$\mathsf{CMX}$, $\mathsf{ENABLE\_SPEND}$,
$\mathsf{ENABLE\_OUTPUT}$.

The "witness" in PLONKish does not split into "private inputs"
and "intermediate wires": every per-row witness cell is an
advice-column entry. The total witness cell count is therefore
$10 \times 2048 = 20480$. Selector and fixed cells are
preprocessed once during verifier-key generation and do not
move per proof.

### Definition 2.4 (Constraint Groups)

The circuit enforces six groups of constraints:

1. **Merkle membership**:
   $\mathsf{MerkleCRH}^{\mathsf{depth}}_{\mathsf{path},\mathsf{pos}}
   (\mathsf{cm}_{\mathsf{old}}) = \mathsf{anchor}$.
2. **Spend authorisation**:
   $\mathsf{ak} = [\mathsf{ask}] \mathcal{G}_{\mathsf{ak}}$ and
   $\mathsf{rk} = \mathsf{ak} + [\alpha] \mathcal{G}_{\mathsf{ak}}$.
3. **Nullifier**:
   $\mathsf{nf} = \mathsf{Extract}_{\mathbb{P}}\big(
   [\mathsf{PRF}^{\mathsf{nfOrchard}}_{\mathsf{nk}}(\rho)]\,
   \mathcal{K} + \psi + \mathsf{cm}\big)$.
4. **Note commitment integrity** for both old and new notes.
5. **Value commitment**:
   $\mathsf{cv^{\mathsf{net}}} = [v_{\mathsf{old}} - v_{\mathsf{new}}]\,
   \mathcal{V} + [\mathsf{rcv}]\, \mathcal{R}$.
6. **Action enable flags**: when `enableSpends = 0`, the spend
   subcircuit is disabled (dummy spend); when `enableOutputs = 0`,
   the output subcircuit is disabled.

## 3. The Code

### 3.1 What the Circuit Proves, in Plain Terms

Before reading the source, fix the high-level picture: one
invocation of the Action circuit produces a single zk-SNARK that
convinces a verifier of all of the following at once, **without
revealing the secrets that make them true**.

1. **The spender owns a real, unspent note**. There exists a note
   $n_{\mathsf{old}}$ whose commitment $\mathsf{cm}_{\mathsf{old}}$
   is part of the global note commitment tree at the public
   anchor. The Merkle authentication path is in the witness; the
   anchor is public. The verifier learns nothing about the
   position of the note in the tree or its contents.
2. **The spender is authorised to spend it**. The wallet knows the
   spending key whose authorising public key is $\mathsf{ak}$.
   The circuit re-randomises $\mathsf{ak}$ with a fresh scalar
   $\alpha$ into $\mathsf{rk}$, which is public; the verifier
   checks the signature against $\mathsf{rk}$ outside the circuit,
   but the circuit forces $\mathsf{rk}$ to be a re-randomisation
   of a key that controls a real note.
3. **The nullifier was computed honestly**. The public nullifier
   $\mathsf{nf}$ is the deterministic output of the spec's
   nullifier formula on $\mathsf{nk}$ and the old note's
   $\rho$. Two distinct spends of the same note necessarily
   produce the same nullifier, so the chain rejects the double
   spend without ever seeing the spender's identity.
4. **A new note was committed to**. There exists a fresh note
   $n_{\mathsf{new}}$ (recipient, value, randomness) whose
   commitment $\mathsf{cm}^\star$ is public and inserted into the
   tree on chain. The recipient and value are hidden; the
   commitment is opaque.
5. **Value is conserved up to a declared imbalance**. The public
   net value commitment $\mathsf{cv}^{\mathsf{net}}$ equals
   $[v_{\mathsf{old}} - v_{\mathsf{new}}] \mathcal{V} +
   [\mathsf{rcv}] \mathcal{R}$ inside the circuit; summed across
   all Actions in the bundle, the differences add up to the
   public `value_balance` (verified outside the circuit by the
   binding signature; see [Chapter 13](./13-value-commitments.md)).
6. **Optional spend / output suppression**. The public
   `enableSpends` and `enableOutputs` flags switch off the
   corresponding subcircuit when the Action is a dummy. This is
   how Orchard pads to a power-of-two Action count without
   leaking the real spend / output count.

The circuit asserts these jointly. Every shielded Orchard
transaction's proof is one such SNARK over the whole bundle: each
Action is one execution trace through this circuit, and the
single proof attests to all of them together.

A useful mental model: think of the Action circuit as a single
small program that takes the witness as input, runs through the
checks above in order, and aborts on the first failure. The
prover convinces the verifier that an accepting execution exists,
without revealing the input.

### 3.2 The `Config` Struct

```rust reference title="src/circuit.rs"
https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L87-L105
```

`Config` bundles every chip configuration the Action circuit uses:
the primary instance column, the `q_orchard` selector, ten advice
columns, the addition chip, the ECC chip, a Poseidon
$P_{128}^{\mathrm{Pasta}}$ sponge, two Merkle configs (one per
hash domain), two Sinsemilla configs, the
[`CommitIvkChip`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/commit_ivk.rs)
config, and two
[`NoteCommitChip`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit/note_commit.rs)
configs (old and new note commitments share the chip but use
distinct configurations).

### 3.3 Synthesise

`Circuit::synthesize` runs top to bottom: load the ECC and
Sinsemilla chips, witness the Merkle path, derive `rk`, derive the
nullifier, build both note commitments, and finally build the
value commitment. Each step ends with a public-input equality
constraint against the appropriate instance column.

### 3.4 Proving and Verifying Keys (Location and Size)

Halo 2 needs two derived keys per circuit:

- A **`VerifyingKey`**: the structured reference string (SRS) plus
  the commitments to the fixed columns (selectors, lookup tables,
  precomputed bases). Sufficient to verify a proof.
- A **`ProvingKey`**: the `VerifyingKey` plus the prover-side
  precomputed data (the Lagrange evaluation of every fixed
  column, the permutation argument data). Sufficient to produce a
  proof.

Both types are declared in
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs):

- [`VerifyingKey`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L765-L782)
  (lines 765 to 782).
- [`ProvingKey`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L784-L802)
  (lines 784 to 802).

Neither key is stored as a file in the repository. Both are
**deterministically (re)derived** at process start by calling
`VerifyingKey::build()` or `ProvingKey::build()`. Both functions
take no parameters: the SRS is built from
`halo2_proofs::poly::commitment::Params::new(K)` and the
verifying key from `plonk::keygen_vk(&params, &circuit)` against
the default `Circuit`. The construction is reproducible bit for
bit because it has no random input; the SRS is the IPA-style
public parameters, with no trusted setup.

#### Size

The circuit constant is `const K: u32 = 11;` (line 74 of
[`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L74-L74)),
so the PLONKish table has $2^{K} = 2048$ rows. The keys' sizes
scale with $2^K$:

- **SRS (`Params<vesta::Affine>`)**: a vector of $2^K$ Vesta
  curve points. Each compressed point is 32 bytes, so the SRS
  is on the order of $2^{11} \times 32 \approx 64$ KiB. The
  uncompressed (in-memory) form roughly doubles that.
- **`VerifyingKey`**: SRS plus one Vesta-point commitment per
  fixed column. With a few tens of fixed columns the additional
  data is on the order of a kilobyte; the verifying key is
  dominated by the SRS.
- **`ProvingKey`**: `VerifyingKey` plus the Lagrange basis
  evaluation of every fixed column (one `[Fp; 2^K]` array per
  column), plus the permutation polynomial data. Each scalar is
  32 bytes; with a few tens of fixed and advice columns at
  $2^K = 2048$ rows, the proving key sits in the low single-digit
  megabytes.
- **Action proof itself** (one per bundle, not per Action): a
  fixed-size sequence of group elements and scalars that grows
  with $\log(2^K) = K$ rounds of IPA. At the pin, an Action proof
  is in the kilobytes range.

A contributor must remember three operational consequences:

1. The keys are **not persisted to disk** by the crate. A
   long-running prover usually calls `ProvingKey::build()` once
   at startup and reuses the result; rebuilding for every proof
   wastes seconds of CPU.
2. **Increasing `K`** (because a new gate ran out of rows)
   doubles the SRS, the verifying key, and roughly doubles the
   prover time and proof size. Any PR that bumps `K` must
   justify the cost.
3. The keys depend on the **circuit shape**, not on a witness.
   Any change to a chip configuration that affects column
   counts, gate definitions, or lookup tables invalidates the
   keys derived by previous binaries. The pinned proof
   [`src/circuit_proof_test_case.bin`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_proof_test_case.bin)
   catches such drift in CI.

### 3.5 The Pinned Snapshot

[`src/circuit_description/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)
holds a textual snapshot of the column layout, gates, and lookup
tables; any change to the circuit shape must update it. The
companion
[`src/circuit_proof_test_case.bin`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_proof_test_case.bin)
is a pinned proof exercised by the unit tests; see
[Chapter 16 (Test Vectors)](./16-test-vectors.md).

### 3.6 Differences From Sapling at the Circuit Level

Sapling and Orchard solve the same problem (a shielded UTXO with
a zk proof per output and per spend), but the circuit-level
implementations differ on every axis. A reader who has read the
Sapling Spend or Output circuit should expect almost no
structural overlap with `src/circuit.rs`.

| Axis                   | Sapling                                                            | Orchard                                                                                                                                                                                                                                        |
| ---------------------- | ------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Proof system           | Groth16                                                            | Halo 2 (IPA, no trusted setup)                                                                                                                                                                                                                 |
| Setup                  | Per-circuit MPC trusted setup ("Powers of Tau" + Sapling-specific) | Universal, transparent (only the SRS lives in code)                                                                                                                                                                                            |
| Arithmetisation        | R1CS                                                               | PLONKish: advice + fixed + selector + instance columns, custom gates, lookups                                                                                                                                                                  |
| Pairing curve          | BLS12-381                                                          | None (no pairing; commitments are IPA on Vesta)                                                                                                                                                                                                |
| Embedded curve         | Jubjub (twisted Edwards)                                           | Pallas (short Weierstrass with $j = 0$ endomorphism)                                                                                                                                                                                           |
| Distinct circuits      | Two: `Spend` and `Output`                                          | One: `Action` (combines a spend and an output, with enable flags)                                                                                                                                                                              |
| Proofs per transaction | One per Spend, one per Output                                      | One per `Bundle` (all Actions in a single proof)                                                                                                                                                                                               |
| Merkle CRH             | Pedersen hash on Jubjub                                            | Sinsemilla on Pallas (windowed Pedersen with a 1024-entry lookup table)                                                                                                                                                                        |
| Note commitment        | Windowed Pedersen commitment                                       | `SinsemillaCommit` (same windowed structure as the CRH)                                                                                                                                                                                        |
| Nullifier PRF          | $\mathsf{Blake2s}$ in-circuit                                      | Poseidon $P_{128}^{\mathrm{Pasta}}$ in-circuit                                                                                                                                                                                                 |
| Spend authorisation    | RedJubjub re-randomisable Schnorr                                  | RedPallas re-randomisable Schnorr                                                                                                                                                                                                              |
| Value commitment       | Pedersen on Jubjub                                                 | Pedersen on Pallas                                                                                                                                                                                                                             |
| Circuit shape pin      | None in-tree (compiled snapshots only)                             | [`src/circuit_description/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description) text + pinned proof                                                                                       |
| Dummy padding          | Not needed (separate Spend/Output proofs)                          | Pads Action list to a power of two with dummies gated by enable flags                                                                                                                                                                          |
| Recursion-friendly     | No (Groth16 + BLS12-381 does not recurse cleanly)                  | Designed for it (Halo 2 + Pasta cycle); recursion not yet shipped in `zcash/halo2` (see [#75](https://github.com/zcash/halo2/issues/75), [#249](https://github.com/zcash/halo2/issues/249), [#251](https://github.com/zcash/halo2/issues/251)) |

Consequences a contributor should keep in mind:

- A Sapling-style "byte-oriented hash inside the circuit" (Blake2s,
  SHA-256) is dramatically more expensive in Halo 2 than in
  Groth16 because PLONKish gates are field-aligned. Orchard's
  switch to Sinsemilla and Poseidon was driven by this asymmetry.
- The single-proof-per-bundle design means every Action shares
  the prover's expensive setup costs; bundle-level fixed costs
  dominate over per-Action marginal costs. This is the opposite
  trade-off to Sapling's per-description proofs.
- Halo 2's custom gates and lookups make the Action circuit
  cheaper at the cost of a much harder review surface. A reviewer
  comfortable with R1CS soundness arguments still needs to learn
  Halo 2's PLONKish-specific failure modes
  ([Chapter 4](./04-halo2-primer.md)).
- The Pasta cycle exists precisely because Halo 2 wants future
  recursion. Sapling's pairing-based setup could not do that
  without a separate cycle-of-pairings construction.

### 3.7 What Halo 2 Is Used For Today

Orchard ships Halo 2 as a **transparent, non-recursive zk-SNARK
for a single fixed circuit**. Concretely, the production usage
consists of three layers and nothing more:

1. **Custom-gate PLONKish arithmetisation**. The Action circuit
   is one PLONKish table of $2^K$ rows with the column layout
   described in `Config` ([Section 3.2](#32-the-config-struct)).
   Custom gates and lookups encode the bit-decomposition checks,
   the Sinsemilla windowed multiplication, and the Poseidon
   permutation. Source:
   [`zcash/halo2` `halo2_proofs`](https://github.com/zcash/halo2/tree/main/halo2_proofs).
2. **Inner-product-argument (IPA) polynomial commitments over
   Vesta**. Column commitments are folded with the IPA protocol
   from the
   [Halo paper](https://eprint.iacr.org/2019/1021).
   This is what makes the setup transparent: there is no trusted
   setup ceremony, only a public structured reference string
   derived from a published seed.
3. **Blake2b Fiat-Shamir transcript**. The outer transcript that
   binds commitments and challenges is a plain Blake2b instance;
   no in-circuit verifier exists yet. See
   [Chapter 4](./04-halo2-primer.md).

Production Orchard **does not**, at the pin, do any of the
following:

- **Recursive proof composition**. The Halo 2 verifier is not
  encoded as a circuit. Each Action bundle's proof is verified
  classically and standalone. The recursion machinery is tracked
  by the open issues
  [zcash/halo2#75 "Implement support for recursion"](https://github.com/zcash/halo2/issues/75),
  [zcash/halo2#249 "Implement recursion circuit logic for
  handling public inputs"](https://github.com/zcash/halo2/issues/249),
  and
  [zcash/halo2#251 "Implement user-facing API for recursive
  proving of IVC"](https://github.com/zcash/halo2/issues/251),
  all open at the time the course was pinned. These issues carry
  the `A-recursion` and `C-target` labels: target functionality
  that is part of the long-term roadmap but not yet shipped.
- **Accumulation / folding**. Halo's original accumulation
  argument is the building block that recursion would need; the
  upstream crate does not expose an accumulator API yet.
- **A circuit-level verifier**. There is no in-circuit
  verification gadget in `halo2_gadgets`. Adding one is a
  prerequisite to any recursion work.

In short, Halo 2 in Orchard today is the **plain non-recursive
SNARK** layer of the protocol. The Pasta cycle is the structural
prerequisite that _would let_ recursion happen without further
breaking changes; the work of actually shipping that recursion is
upstream and unfinished.

### 3.8 Why Recursion Matters

Recursion here means: a proof verifier is itself encoded as a
circuit, so a proof can attest to the verification of another
proof. The Orchard Action circuit does not use recursion at the
pin (see [Section 3.7](#37-what-halo-2-is-used-for-today) for the
tracking issues). The design keeps the door open. Three reasons
this matters.

1. **Cost amortisation across many transactions**. Without
   recursion, a verifier validates each transaction's proof
   independently in $O(\log n)$ time, where $n$ is the circuit
   size. With recursion, many proofs collapse into one
   "rollup" proof whose verification time is independent of the
   batch size. For a chain that handles millions of shielded
   transactions, this is the difference between a node syncing
   in days versus in minutes.
2. **Light-client succinctness**. A light client that cannot
   afford to store the chain can verify a single recursive proof
   that the chain's entire history is valid. Without recursion,
   the light client must trust a third party for the chain's
   state. Recursive Halo proofs make trust-minimised light
   clients feasible.
3. **Composable cryptography**. Application-layer protocols
   (private bridges, private rollups, recursive zk-VMs) need to
   nest proofs cleanly. Sapling's Groth16 cannot recurse without
   a second pairing-friendly curve cycle, which roughly doubles
   the prover cost and re-introduces a trusted setup. Halo 2
   over Pasta recurses with no extra ceremony: the verifier
   circuit and the prover live in the same field hierarchy.

The cycle requirement (see [Chapter 3](./03-pasta-curves.md)) is
the structural prerequisite: the field a circuit operates over
must equal the scalar field of the curve that the inner proof
uses, so the inner proof's group elements can be witnessed as
in-circuit values. The Pasta pair (Pallas, Vesta) is that
prerequisite met with no trusted setup.

In short: even when Orchard does not invoke recursion today, the
choice of Halo 2 + Pasta keeps the future migration to recursive
proofs an engineering exercise rather than a protocol break.

For the parallel implementation, see
[`zcash/sapling-crypto`](https://github.com/zcash/sapling-crypto).

## 4. Failure Modes

- **Underconstrained witness**. The most dangerous category of
  bug. The dev-mode prover does not detect it; the only defence
  is the spec-to-constraint review captured in the audit reports
  (Chapter 17). When in doubt, write a malicious witness in a unit
  test and confirm it is rejected.
- **Stale `circuit_proof_test_case.bin`**. If the proof format
  changes upstream (`halo2_proofs` version bump), the pinned
  proof must be regenerated. The reviewer will require a
  paragraph in the PR explaining why the regeneration is safe.
- **Public-input column order**. The verifier wires the instance
  columns to fields of `Instance`; swapping two columns silently
  decouples constraints from their public targets.
- **Enable-flag coverage gap**. A bug in the flag-gating logic
  was triggered in the past by a malformed dummy Action; see
  [#492](https://github.com/zcash/orchard/pull/492) for the
  identity-`rk` consensus rule that resulted.

## 5. Spec Pointers

- [Zcash Protocol Specification, Section 4.19](https://zips.z.cash/protocol/protocol.pdf):
  the Action statement, the authoritative list of constraints.
- [Zcash Protocol Specification, Section 5.4.9.6](https://zips.z.cash/protocol/protocol.pdf):
  the realisation of the Action statement as a Halo 2 circuit.
- [Halo 2 Book, Design](https://zcash.github.io/halo2/design.html):
  the chip patterns that the Action circuit reuses.

## 6. Exercises

1. List every call to `meta.instance_column()` in
   [`src/circuit.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs).
   Match each one to a public input from Definition 2.1.
2. Compute (by inspection) how many rows each constraint group
   from Definition 2.3 occupies. Compare with the total $2^K$.
3. **Code task**. Modify
   [`tests/builder.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/tests/builder.rs)
   to construct an Action with `enableSpends = false` (a dummy
   spend). Run the test and verify that the proof still verifies.
   Then flip `enableSpends = true` while keeping the spend
   witness empty: the prover should reject. Revert.

## 7. Further Reading

- [Orchard Book, Action circuit](https://zcash.github.io/orchard/design/circuit.html):
  the higher-level walkthrough maintained by EC Co. engineers.
- The audit reports cited in
  [Chapter 17 (Audits)](./17-audits.md) include line-by-line
  reviews of the circuit.
