---
sidebar_position: 0.5
title: Names, ZIPs, Issues, and PRs
description: Origin of the names Sprout, Sapling, and Orchard, plus a curated index of Orchard-relevant ZIPs, GitHub issues, and merged PRs.
---

# Names, ZIPs, Issues, and PRs

This page is the master index of external references for the
course. The course chapters cite these in context; this page
collects them in one place so a reader can scan the protocol's
public history without searching.

## 1. The Botanical Naming Convention

Zcash's three shielded protocols are named after stages of plant
growth. The metaphor is deliberate: each protocol replaces the
previous one as the "current" shielded pool while the older
pools remain spendable.

### 1.0 Botanical Glossary

The names borrow specific words from arboriculture and
horticulture. Their plain English meanings:

| Name        | Botanical meaning                                                                                                                                                                                                                                                                                                                                              |
| ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Sprout**  | The first shoot a seed pushes above the soil. The earliest visible stage of a plant's life, before it can be identified as a particular species. By extension, any small beginning.                                                                                                                                                                            |
| **Sapling** | A young tree, no longer a seedling but not yet mature. Distinct from a sprout in that it is unambiguously a tree of a specific species, and has grown a recognisable trunk and crown.                                                                                                                                                                          |
| **Pollard** | A mature tree that has been heavily pruned by repeatedly cutting back the upper branches to encourage a dense head of new growth. The technique is called pollarding; the resulting tree is called a pollard. Pollards are common in European urban forestry. (The name was the internal-only name for what later became Orchard; the rename is documented below.) |
| **Orchard** | A deliberate planting of fruit-bearing or nut-bearing trees, managed as a group. An orchard differs from a forest in that the trees are planted for harvest and tended individually.                                                                                                                                                                           |

The metaphor is a progression: a sprout becomes a sapling, a
sapling matures into a productive tree (a pollard is one
horticultural form of such a mature tree), and many mature trees
together form an orchard. Each Zcash protocol generation
inherits the structure of the previous one and adds the next
stage of capability.

The remaining sections document the exact source of each name in
the public record.

### 1.1 "Sprout" (2016)

The original Zcash shielded protocol launched at mainnet
activation in October 2016. It used the BN-128 (alt_bn128)
pairing curve and Groth16-style SNARKs derived from the Zerocash
paper. The name "Sprout" was applied retroactively when its
successor was named; the original codebase referred to it simply
as "the JoinSplit protocol".

The `A-Sprout` label and "Sprout" terminology in
[`zcash/zcash`](https://github.com/zcash/zcash) date back to the
period around the Sapling design discussion (early 2018). See
for example
[zcash/zcash#32](https://github.com/zcash/zcash/issues/32)
(March 2018), which carries the `NU1-sapling` label and
discusses the JoinSplit / Sapling separation. The
[Sprout protocol specification](https://zips.z.cash/protocol/protocol.pdf)
remains in the Zcash Protocol Specification as Section 4.5 and
earlier.

### 1.2 "Sapling" (2018)

Sapling activated at Network Upgrade 1 in October 2018, replacing
Sprout's BN-128 with the BLS12-381 pairing curve and the Jubjub
embedded curve. The name appears in
[`zcash/zcash`](https://github.com/zcash/zcash) issues from early
2018, including the `A-Sapling` and `NU1-sapling` labels. The
relevant ZIPs are 173 ("Sapling Sprout-Compatible Spending"),
117, 200-205, and others under the `Sapling` label in
[`zcash/zips`](https://github.com/zcash/zips).

The naming choice is botanically natural: a sapling is a young
tree, the growth stage that follows a sprout.

### 1.3 "Orchard" (2021), formerly "Pollard"

Orchard activated at Network Upgrade 5 in May 2022, replacing
Sapling's pairing-based Groth16 with Halo 2 over the Pasta cycle
of curves.

The name was originally **Pollard** in the internal design
discussions of 2020. The protocol was renamed to "Orchard"
during ZIP drafting; the explicit citation is the ZIP 224
proposal,
[`zcash/zips#435`](https://github.com/zcash/zips/issues/435),
opened on 2021-03-04, whose body states verbatim:

> Orchard is provisionally the new name for the protocol we
> were calling Pollard.

The provisional name stuck. "Pollard" is a heavily-pruned tree;
"Orchard" is a grove of mature fruit trees. The metaphor
continues the plant-growth theme: sprout (a seedling), sapling
(a young tree), pollard (a pruned mature tree), orchard (a
grove). The Pollard nickname also had a cryptographic resonance
through [Pollard's rho algorithm](https://en.wikipedia.org/wiki/Pollard%27s_rho_algorithm_for_logarithms),
but the public ZIP record gives no evidence that this was the
intended pun.

### 1.4 Beyond Orchard

Future protocol work is tracked under other names. The Zcash
Shielded Assets (ZSA) work
([`zcash/orchard#471`](https://github.com/zcash/orchard/pull/471))
extends Orchard with arbitrary asset issuance; its name follows
a different (non-botanical) convention.

## 2. Authoritative External References

These are the documents the course cites repeatedly.

| Reference                                                                 | Role                                                                                |
| ------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| [Zcash Protocol Specification](https://zips.z.cash/protocol/protocol.pdf) | Normative protocol spec; Section 4 (shielded primitives) and Section 5.4 (Orchard). |
| [ZIPs index](https://zips.z.cash/)                                        | All Zcash Improvement Proposals, indexed by number.                                 |
| [Orchard Book](https://zcash.github.io/orchard/)                          | Upstream mdBook companion to the crate.                                             |
| [Halo 2 Book](https://zcash.github.io/halo2/)                             | Upstream mdBook on the proof system.                                                |
| [Halo paper](https://eprint.iacr.org/2019/1021)                           | The IPA + accumulation result that underlies Halo 2.                                |
| [PLONK paper](https://eprint.iacr.org/2019/953)                           | The arithmetisation Halo 2 extends.                                                 |
| [Poseidon paper](https://eprint.iacr.org/2019/458)                        | The algebraic hash used inside the circuit.                                         |
| [Pasta curves announcement](https://electriccoin.co/blog/the-pasta-curves-for-halo-2-and-beyond/) | The design rationale for the curve cycle.                              |

## 3. Orchard-Relevant ZIPs

ZIPs that the Orchard protocol directly depends on, or that
extend Orchard, drawn from
[`zcash/zips`](https://github.com/zcash/zips) under the `Orchard`
and `Sapling` labels. Marked **(normative)** where consensus-level.

| ZIP                                          | Title                                                              | Status   |
| -------------------------------------------- | ------------------------------------------------------------------ | -------- |
| [ZIP 32](https://zips.z.cash/zip-0032)       | Shielded Hierarchical Deterministic Wallets (normative)            | Final    |
| [ZIP 212](https://zips.z.cash/zip-0212)      | Allow Recipient to Derive Ephemeral Secret from Note Plaintext     | Final    |
| [ZIP 213](https://zips.z.cash/zip-0213)      | Shielded Coinbase Outputs                                          | Final    |
| [ZIP 216](https://zips.z.cash/zip-0216)      | Require Canonical Jubjub Point Encodings                           | Final    |
| [ZIP 224](https://zips.z.cash/zip-0224)      | Orchard Shielded Protocol (normative; the activation ZIP)          | Final    |
| [ZIP 225](https://zips.z.cash/zip-0225)      | Version 5 Transaction Format                                       | Final    |
| [ZIP 226](https://zips.z.cash/zip-0226)      | Reserved (Orchard issuance, draft)                                 | Draft    |
| [ZIP 227](https://zips.z.cash/zip-0227)      | Issuance of Zcash Shielded Assets (ZSA)                            | Draft    |
| [ZIP 244](https://zips.z.cash/zip-0244)      | Transaction Identifier Non-Malleability (normative; SIGHASH)       | Final    |
| [ZIP 252](https://zips.z.cash/zip-0252)      | Network Upgrade 5 (NU5)                                            | Final    |
| [ZIP 308](https://zips.z.cash/zip-0308)      | Migration to Orchard                                               | Draft    |
| [ZIP 316](https://zips.z.cash/zip-0316)      | Unified Addresses and Unified Viewing Keys                         | Final    |
| [ZIP 317](https://zips.z.cash/zip-0317)      | Conventional Transfer Fee Mechanism                                | Final    |
| [ZIP 401](https://zips.z.cash/zip-0401)      | Addressing Mempool Denial-of-Service                               | Final    |

For the full list grouped by label, see
[`zcash/zips` issues with the Orchard label](https://github.com/zcash/zips/issues?q=is%3Aissue+label%3AOrchard).

## 4. Open Issues in `zcash/orchard`

The list below is curated for relevance to contributors; the full
queue is at
[the issues page](https://github.com/zcash/orchard/issues).

### 4.1 Research-Adjacent (Cryptography / Audit)

| Issue                                                                    | Subject                                                                  |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| [#7](https://github.com/zcash/orchard/issues/7)                          | Create key structure / capability diagram                                |
| [#47](https://github.com/zcash/orchard/issues/47)                        | Extract Sapling security analysis into the Orchard Book                  |
| [#84](https://github.com/zcash/orchard/issues/84)                        | Correctness proofs for scalar multiplications and range checks           |
| [#125](https://github.com/zcash/orchard/issues/125)                      | Name all polynomial constraints                                          |
| [#172](https://github.com/zcash/orchard/issues/172)                      | Note Privacy (OOB) depends on PRF-ness of `PRF^expand`                   |
| [#190](https://github.com/zcash/orchard/issues/190)                      | Consider creating a `poseidon::Transcript` primitive                     |

### 4.2 API and Feature Work

| Issue                                                                    | Subject                                                                  |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| [#191](https://github.com/zcash/orchard/issues/191)                      | Add test vectors for ZIP 32 derivation                                   |
| [#216](https://github.com/zcash/orchard/issues/216)                      | Update description of hierarchical addresses in the book                 |
| [#256](https://github.com/zcash/orchard/issues/256)                      | Measure memory-usage benchmarks                                          |
| [#347](https://github.com/zcash/orchard/issues/347)                      | Add a `Circuit` constructor                                              |
| [#430](https://github.com/zcash/orchard/issues/430)                      | API changes required for FROST                                           |
| [#431](https://github.com/zcash/orchard/issues/431)                      | Allow constructing FVK from `SpendValidatingKey`                         |
| [#459](https://github.com/zcash/orchard/issues/459)                      | Allow the `circuit` feature to build under `no_std`                      |
| [#463](https://github.com/zcash/orchard/issues/463)                      | Update to `rand 0.9`                                                     |
| [#464](https://github.com/zcash/orchard/issues/464)                      | Panic on `ExtendedSpendingKey` derivation at depth 256+                  |
| [#467](https://github.com/zcash/orchard/issues/467)                      | Make `NoteCommitment` part of the public API                             |
| [#491](https://github.com/zcash/orchard/issues/491)                      | Document that `cargo test --package orchard` runs no tests               |
| [#497](https://github.com/zcash/orchard/issues/497)                      | Make `BatchValidator::add_bundle` return a `Result`                      |

### 4.3 Recursion (in the upstream `zcash/halo2` crate)

| Issue                                                                    | Subject                                                                  |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| [zcash/halo2#75](https://github.com/zcash/halo2/issues/75)               | Implement support for recursion                                          |
| [zcash/halo2#249](https://github.com/zcash/halo2/issues/249)             | Recursion circuit logic for handling public inputs                       |
| [zcash/halo2#251](https://github.com/zcash/halo2/issues/251)             | User-facing API for recursive proving of IVC                             |

## 5. Notable Merged PRs in `zcash/orchard`

The list below covers PRs that landed in the four releases
preceding the pin
([`f8915bc`](https://github.com/zcash/orchard/tree/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669),
which is the tag `0.13.1`). These are good templates for
contributors writing their first PR.

### 5.1 Consensus-Relevant

| PR                                                                       | Title                                                                    |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| [#492](https://github.com/zcash/orchard/pull/492)                        | Reject identity `rk` in `Action::from_parts` / `Instance::from_parts`    |
| [#479](https://github.com/zcash/orchard/pull/479)                        | Return `DepthOverflow` instead of panicking at depth 255 (fixes #464)    |

### 5.2 Refactor and Cleanup

| PR                                                                       | Title                                                                    |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| [#496](https://github.com/zcash/orchard/pull/496)                        | Collapse `OrchardFixedBases` to a unit struct; drop dead `FixedPoint`    |
| [#495](https://github.com/zcash/orchard/pull/495)                        | Replace `NoteValue::zero()` with `NOTE_VALUE_ZERO` const                 |
| [#493](https://github.com/zcash/orchard/pull/493)                        | Revert the `orchard_internal` crate split                                |
| [#490](https://github.com/zcash/orchard/pull/490)                        | `orchard_internal` split clarifications                                  |
| [#489](https://github.com/zcash/orchard/pull/489)                        | Add `SpendAuthG` fixed-base multiplication support                       |
| [#488](https://github.com/zcash/orchard/pull/488)                        | `unstable-voting-circuits` feature to widen internals                    |
| [#482](https://github.com/zcash/orchard/pull/482)                        | Migrate from yanked `core2` to `corez`                                   |
| [#480](https://github.com/zcash/orchard/pull/480)                        | Split into `orchard_internal` + `orchard` shim (later reverted by #493)  |
| [#478](https://github.com/zcash/orchard/pull/478)                        | CI: use pinned deps for `build-nostd`                                    |

### 5.3 PCZT and ZSA

| PR                                                                       | Title                                                                    |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| [#477](https://github.com/zcash/orchard/pull/477)                        | Make `pczt::Bundle::extract` take `self` by reference                    |
| [#472](https://github.com/zcash/orchard/pull/472)                        | PCZT: support applying external `spendAuthSig` to Spends                 |
| [#471](https://github.com/zcash/orchard/pull/471)                        | Add OrchardZSA                                                           |
| [#470](https://github.com/zcash/orchard/pull/470)                        | Compatibility with latest `halo2` (ZSA features)                         |
| [#499](https://github.com/zcash/orchard/pull/499)                        | Add QR Orchard note version support                                      |

### 5.4 Releases

| PR                                                                       | Tag        |
| ------------------------------------------------------------------------ | ---------- |
| [#498](https://github.com/zcash/orchard/pull/498)                        | 0.13.1     |
| [#494](https://github.com/zcash/orchard/pull/494)                        | 0.13.0     |
| [#474](https://github.com/zcash/orchard/pull/474)                        | 0.12.0     |
| [#465](https://github.com/zcash/orchard/pull/465)                        | 0.10.2     |

## 6. Cross-Repository Anchors

| Repository                                                                              | Role                                                              |
| --------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| [`zcash/orchard`](https://github.com/zcash/orchard)                                     | This crate.                                                       |
| [`zcash/halo2`](https://github.com/zcash/halo2)                                         | The proof system and chip library.                                |
| [`zcash/pasta_curves`](https://github.com/zcash/pasta_curves)                           | Pallas and Vesta curves.                                          |
| [`zcash/sinsemilla`](https://github.com/zcash/sinsemilla)                               | The Sinsemilla hash function (extracted crate).                   |
| [`zcash/zips`](https://github.com/zcash/zips)                                           | The ZIPs index and the Zcash Protocol Specification source.       |
| [`zcash/librustzcash`](https://github.com/zcash/librustzcash)                           | Parallel Rust client, including `zcash_note_encryption`.          |
| [`zcash/incrementalmerkletree`](https://github.com/zcash/incrementalmerkletree)         | The Merkle frontier maintenance.                                  |
| [`zcash/zip32`](https://github.com/zcash/zip32)                                         | Hardened-derivation primitives.                                   |
| [`zcash/zcash_spec`](https://github.com/zcash/zcash_spec)                               | Shared spec primitives.                                           |
| [`zcash-hackworks/zcash-test-vectors`](https://github.com/zcash-hackworks/zcash-test-vectors) | Cross-implementation test vectors.                                |
| [`ZcashFoundation/reddsa`](https://github.com/ZcashFoundation/reddsa)                   | RedDSA over Jubjub and Pallas.                                    |
| [`ZcashFoundation/zebra`](https://github.com/ZcashFoundation/zebra)                     | Independent Rust full node; cross-checks the same consensus.      |
