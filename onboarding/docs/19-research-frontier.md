---
sidebar_position: 19
title: Research Frontier and Improvement Vectors
description: Open questions, comparison with newer proof systems, and concrete avenues to improve the Orchard protocol.
---

# Research Frontier and Improvement Vectors

## 1. Why This Chapter Exists

The reader who has worked through the previous chapters now sees
Orchard as a settled design. A cryptography researcher should
not. Several decisions in Orchard are points on a Pareto frontier
that has moved since 2022, and several open questions in the
public issue trackers are research problems disguised as
engineering tickets. After this chapter the reader can identify
where to put research effort to plausibly improve the protocol,
which directions have already been studied, and which carry the
biggest open questions.

## 2. Definitions

### Definition 2.1 (Improvement Vector)

A change to the protocol or its implementation that, at fixed
security level, strictly improves at least one of the following
axes without regressing another: proof size, prover time,
verifier time, memory footprint, anonymity-set size, recipient
scanning time, security-margin tightness, or feature surface.

### Definition 2.2 (Hardness Assumptions of Orchard)

The standalone security of an Orchard bundle reduces, modulo
modelling assumptions, to a conjunction of:

1. **Halo 2 knowledge soundness** in the algebraic group model,
   reducing to the discrete-log assumption on
   $E_p, E_q$ (Pallas, Vesta).
2. **Collision and pre-image resistance** of Sinsemilla under
   the random oracle modelling of its generator derivation.
3. **Collision and pre-image resistance** of Poseidon
   $P_{128}^{\mathrm{Pasta}}$, which is itself conjectural and
   not reducible to a more standard assumption.
4. **Strong unforgeability of RedDSA / RedPallas** in the
   random oracle model under the discrete-log assumption on
   $E_p$.
5. **IK-CCA security of the Orchard KEM**, which decomposes
   into the gap-DH assumption on $E_p$ plus IND-CCA of
   ChaCha20-Poly1305.
6. **PRF security of `Blake2b`** for the key-derivation paths
   ($\mathsf{KDF}^{\mathsf{Orchard}}$,
   $\mathsf{PRF}^{\mathsf{expand}}$).

Tightening any one of these reductions or replacing the
underlying primitive is an Improvement Vector.

## 3. The Code

### 3.1 The Proof System Frontier

Halo 2 in `zcash/halo2` is a 2020-era PLONKish system with IPA
commitments. Since then several proof systems have appeared that
sit on different points of the trade-off curve. Concretely:

- **KZG-based PLONK variants** (PLONK, fflonk, HyperPlonk):
  much smaller proofs (single-digit kilobytes), constant
  verifier time, but require a trusted setup (Universal SRS).
  Switching Orchard to a KZG-based system would shrink the
  proof but reintroduce a ceremony.
  Reference: [Gabizon et al. 2019](https://eprint.iacr.org/2019/953).
- **Folding schemes / accumulation**:
  [Nova](https://eprint.iacr.org/2021/370),
  [HyperNova](https://eprint.iacr.org/2023/573),
  [Sangria](https://github.com/microsoft/Sangria-paper),
  [ProtoStar](https://eprint.iacr.org/2023/620). These are the
  direct successors of Halo's accumulation argument and the
  natural path to recursion in Orchard. The open issues
  [zcash/halo2#75](https://github.com/zcash/halo2/issues/75),
  [#249](https://github.com/zcash/halo2/issues/249), and
  [#251](https://github.com/zcash/halo2/issues/251) track the
  upstream work; a contribution that ports any of the modern
  folding schemes to the Pasta cycle would be high-impact.
- **Sumcheck-based protocols**:
  [Spartan](https://eprint.iacr.org/2019/550),
  [HyperPlonk](https://eprint.iacr.org/2022/1355),
  [Brakedown](https://eprint.iacr.org/2021/1043). Sub-linear
  verifier with no FFT during proving; competitive against IPA
  for the prover at the Orchard scale, with sub-linear verifier
  if combined with a polynomial commitment scheme that supports
  it.
- **STARK-style hash-only commitments** (FRI, ligero):
  transparent like IPA, smaller verifier than IPA, but proofs
  are an order of magnitude larger. Probably not a fit unless
  combined with recursion.

A meta-question: should the proof system be **field-agnostic**
so that Orchard can swap the inner system without touching the
circuit? Halo 2's `halo2_proofs` is currently entangled with
its IPA backend; abstracting the polynomial commitment scheme is
a non-trivial engineering project that would unlock side-by-side
benchmarks.

### 3.2 The Circuit Frontier

The Action circuit at the pin ($K = 11$, $2^{11} = 2048$ rows)
makes specific choices that future work can re-examine.

- **Sinsemilla window size**.
  $k = 10$ gives a 1024-row lookup table; changing $k$ shifts
  rows-per-input-bit against lookup-table cost. A column-budget
  study at $k \in \{8, 12, 16\}$ would clarify whether the
  current point is optimal.
  Reference:
  [Halo 2 Book, Sinsemilla](https://zcash.github.io/halo2/design/gadgets/sinsemilla.html).
- **Poseidon rate**. The sponge uses width $t = 3$, rate 2. Per
  [Grassi et al. 2019](https://eprint.iacr.org/2019/458) the
  number of partial rounds drops sharply with $t$; a wider
  Poseidon ($t = 4$ or $t = 5$) might reduce the per-permutation
  row cost. The cost / security trade-off is sensitive to the
  exact security target.
- **Joint Sinsemilla / Poseidon chip**. Both hashes share the
  ECC chip and the lookup machinery. A unified chip that
  pipelines Sinsemilla and Poseidon could reduce wasted rows
  during transitions between the two.
- **Reduced `K`**.
  [`src/circuit.rs:L74`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit.rs#L74-L74)
  fixes $K = 11$. The 2,048-row table is heavily utilised, but
  not every row is binding; an audit-quality compression study
  could attempt $K = 10$ and report which constraint group breaks
  first. A 50% prover-time reduction would follow.
- **Named constraints**. Open issue
  [zcash/orchard#125 "Name all polynomial constraints"](https://github.com/zcash/orchard/issues/125)
  is a documentation task with a research consequence: named
  constraints make formal verification of the circuit feasible
  and lower the audit cost of every future change.
- **Formal correctness proofs**. Open issue
  [zcash/orchard#84 "Correctness proofs for scalar
  multiplications and scalar range checks"](https://github.com/zcash/orchard/issues/84)
  is an explicit invitation to write a mechanised proof of the
  in-circuit primitives. Candidate frameworks: Lean / Coq /
  EasyCrypt.

### 3.3 The Curve Frontier

The Pasta cycle (Pallas, Vesta) was deliberately chosen for
Halo 2. Three lines of research are open.

- **Bigger fields**. Both Pasta primes are ~255 bits. A 320-bit
  cycle would buy security margin for the long-tail post-quantum
  transition window at the cost of ~2.5x in field arithmetic.
- **More structure**. Both curves have $j = 0$, giving them an
  efficient endomorphism. A cycle whose endomorphism extends
  further (e.g. a CM-by-Eisenstein-integers structure) could
  speed up scalar multiplication and possibly enable lattice-
  based attack mitigations against side-channels.
- **Pairing-friendly cycle**. None of the Pasta-style cycles
  admit a pairing. A cycle of pairing-friendly curves would
  unlock KZG-based PLONK variants and reach proof sizes below
  the current Halo 2 floor. The trade-off is the engineering
  burden of two pairing curves and a substantial trusted-setup
  ceremony.

### 3.4 The Privacy Frontier

Privacy guarantees are encoded by the protocol but not always
optimally so. Candidate research questions:

- **Padding distribution**. Power-of-two padding (see
  [Chapter 12](./12-bundle-and-builder.md)) leaks the bucket
  the spend count fits into. A more uniform pad distribution
  (e.g. always exactly 8 Actions, or a randomised Poisson cap)
  could harden against statistical attacks at modest extra cost.
- **Forward secrecy of `esk`**. The ephemeral secret
  $\mathsf{esk}$ is sampled per Action and discarded by the
  sender after constructing the ciphertext. If a sender wallet
  is later compromised, the per-Action $\mathsf{esk}$ is gone,
  but the binding to $\mathsf{epk}$ is on chain. A formal forward-
  secrecy treatment would clarify the exact threat model.
- **Post-quantum threat**. The Orchard KEM is DH-based and
  therefore would not be secure against a sufficiently capable
  quantum adversary, in line with every other Diffie-Hellman
  scheme deployed today. Research direction: a hybrid KEM that
  ships both a classical Orchard ciphertext and a post-quantum
  KEM encapsulation (e.g. ML-KEM 768) for the same plaintext.
  This is non-trivial because the recipient's IVK lives in
  $\mathbb{F}_q$ but a PQ KEM is byte-oriented; the encoding
  has to be done carefully to avoid linkability between the
  classical and PQ ciphertexts.
- **Cross-pool linkability**. A transaction that touches both
  the Sapling pool and the Orchard pool is observable as such.
  A unified shielded pool would close this gap;
  [ZIP 316](https://zips.z.cash/zip-0316) is the
  current architectural answer at the address layer, but the
  on-chain pool separation remains.
- **Memo confidentiality vs compressibility**. The 512-byte
  memo is encrypted but its plaintext is unbounded in entropy;
  empirical studies on memo content could quantify how much
  information leaks via memo length-class buckets.

### 3.5 The Feature Frontier

These are tracked as concrete work items in the issue tracker;
the research consequence is what they unlock.

- **ZSA (Zcash Shielded Assets)** ([PR #471](https://github.com/zcash/orchard/pull/471)):
  issuance and transfer of arbitrary asset types within the
  Orchard pool. Research questions: anonymity-set fragmentation
  per asset, denial-of-service vectors from spam issuance, and
  the security definition for "asset binding" in the value
  commitment.
- **FROST integration** ([issue #430](https://github.com/zcash/orchard/issues/430)):
  threshold spend authorisation via FROST over Pallas. Research
  questions: round complexity of FROST in the re-randomisation
  setting, side-channels in the share generation, formal
  treatment of malicious-majority security.
- **PCZT (Partially Constructed Zcash Transaction)** support is
  already in the crate ([`src/pczt.rs`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/pczt.rs));
  open questions remain on offline-signing flows for hardware
  wallets and on the interaction with FROST.
- **QR-Orchard note versions** ([PR #499](https://github.com/zcash/orchard/pull/499)):
  a versioning scheme for note metadata. Research consequence:
  forward-compatible note formats without breaking the trial-
  decryption side.

### 3.6 The Implementation Frontier

These are not research per se, but they sit on the boundary.

- **GPU-friendly prover**. The Halo 2 prover spends most of its
  time in NTTs and multi-scalar multiplications. Both are
  GPU-amenable; the current
  [`halo2_proofs`](https://github.com/zcash/halo2) crate has no
  GPU backend. A correct, side-channel-respectful GPU port is
  an open engineering question.
- **Parallel multi-Action proving**. Each bundle proves all
  Actions jointly; the prover could pipeline distinct chips
  across cores. Profiling at the pin shows roughly linear
  scaling to 4 cores, then diminishing returns. A research
  question: what is the asymptotic parallelism budget of the
  Halo 2 prover for this circuit shape?
- **Constant-time guarantees under realistic deployments**. The
  `pasta_curves` crate uses `subtle` primitives, but the
  surrounding wallet code is not under the same discipline. A
  measurement study (cache, timing, power) against a real Zcash
  wallet would clarify the threat model.

### 3.7 What a Researcher Should Actually Read First

Pick one of the following and treat it as the entry to the
literature:

- For the proof system:
  [Halo](https://eprint.iacr.org/2019/1021),
  [PLONK](https://eprint.iacr.org/2019/953),
  [Halo Infinite](https://eprint.iacr.org/2020/1536),
  and the [Halo 2 Book](https://zcash.github.io/halo2/).
- For Sinsemilla:
  the Sinsemilla appendix of the
  [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
  plus
  [Bowe-Hopwood 2020](https://web.archive.org/web/20221230120000/https://github.com/zcash/zcash/issues/2234)
  for the design discussion.
- For Poseidon:
  [Grassi et al. 2019](https://eprint.iacr.org/2019/458) plus
  the
  [Pasta-specific parameter discussion](https://zcash.github.io/halo2/design/gadgets/poseidon.html).
- For the protocol as a whole:
  Sections 4 and 5 of the
  [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf)
  in full.
- For audits as a starting point on the attack surface:
  [Trail of Bits 2021](https://www.trailofbits.com/reports/Halo2.pdf),
  [NCC Group 2021](https://research.nccgroup.com/wp-content/uploads/2022/03/NCC_Group_ZcashFoundation_E001950_Report_2021-12-15.pdf),
  [Least Authority 2022](https://leastauthority.com/static/publications/LeastAuthority_Zcash_Orchard_Pasta_Final_Audit_Report.pdf).

## 4. Failure Modes

For researchers proposing protocol changes, the recurring
mistakes are:

- **Local optimisation that breaks global properties**. Tuning a
  chip for fewer rows can introduce subtle witness ambiguity
  (see [Chapter 17](./17-audits.md), category "canonicality"). A
  change must come with the recomputed
  [`src/circuit_description/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)
  snapshot and an argument that no new ambiguity is introduced.
- **Asymptotic argument with no benchmark**. A proof-system
  change that "should be 2x faster" is meaningless without a
  benchmark on the Orchard Action circuit specifically. The
  benches under
  [`benches/`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/benches)
  are the canonical baseline.
- **New assumption without reduction**. Replacing Poseidon with
  a new hash is acceptable; doing so without an explicit
  security claim is not. The audit boundary (Chapter 17)
  enforces this.
- **Recursion-first work that ignores Pasta concretely**. A
  generic accumulation scheme is only useful here if it fits
  the Pasta cycle. Proposals must come with a worked-out
  in-circuit verifier shape.
- **Privacy "improvements" measured only at the cryptographic
  layer**. The actual anonymity set is a network-layer property;
  any privacy claim must include an explicit threat model that
  covers the gossip layer.

## 5. Spec Pointers

- [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf):
  Section 5 enumerates the cryptographic primitives; Section 4
  enumerates the protocol invariants the new primitives must
  preserve.
- [ZIP 224](https://zips.z.cash/zip-0224):
  the activation rules a backward-compatible change must respect.
- [`zcash/halo2` README](https://github.com/zcash/halo2):
  status of the proof system, including unaudited areas.
- The
  [zcash/zips repo](https://github.com/zcash/zips):
  the venue for any protocol-level proposal. New ZIPs go
  through public discussion before any code change lands.

## 6. Exercises

1. **Research task**. Pick one issue from
   [`zcash/halo2`](https://github.com/zcash/halo2/issues)
   labelled `A-recursion`. Sketch a one-page design proposal
   for closing it, citing the cited paper(s) in this chapter.
2. **Research task**. Read
   [PR #471 (ZSA)](https://github.com/zcash/orchard/pull/471).
   Identify the _security claim_ the PR makes and the
   _primitive_ on which the claim rests. Determine whether the
   claim is reducible to one of Definition 2.2's assumptions.
3. **Code task**. Run
   `cargo bench --bench circuit -- --quick` at the pin. Record
   the prover and verifier times. Propose one change from
   [Section 3.2 (Circuit Frontier)](#32-the-circuit-frontier)
   that would, if implemented, plausibly reduce the prover time
   by 10% or more; justify with a back-of-the-envelope estimate.
4. **Research task**. The Pasta cycle has a particular size and
   structure (Section 3.3 above). Sketch a hypothetical cycle
   that would dominate Pasta on one of the listed axes; identify
   the obstacle that has so far prevented its use.
5. **Privacy task**. Construct (on paper) an observer-side
   attack that distinguishes between two Bundles built with
   power-of-two padding versus a Bundle padded to a fixed size.
   Quantify the information leakage in bits as a function of
   the spend / output counts.

## 7. Further Reading

- [The Zcash Foundation's research blog](https://zfnd.org/category/research/)
  for the cadence of open questions discussed in public.
- [Anoma research notes](https://research.anoma.net/) for
  parallel privacy-protocol design choices.
- [Aleo / Aleo SDK](https://github.com/AleoHQ/snarkVM) for a
  competing PLONK-style protocol with a different circuit
  choice.
- [Penumbra](https://github.com/penumbra-zone/penumbra) for a
  Cosmos-ecosystem privacy protocol that also uses Pasta + Halo 2;
  divergences in design choices are instructive.
- Cryptography conferences with relevant tracks:
  [CRYPTO](https://www.iacr.org/conferences/crypto/),
  [EUROCRYPT](https://www.iacr.org/conferences/eurocrypt/),
  [TCC](https://www.iacr.org/conferences/tcc/),
  [PKC](https://www.iacr.org/conferences/pkc/),
  [USENIX Security](https://www.usenix.org/conferences),
  [CCS](https://www.sigsac.org/ccs.html), and the IACR
  [ePrint Archive](https://eprint.iacr.org/) for preprints.
