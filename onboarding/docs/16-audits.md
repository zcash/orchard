---
sidebar_position: 16
title: audits and historical issues
---

# audits and historical issues

## motivation

The Orchard implementation, the Halo 2 proof system, and the Pasta
curves have been audited multiple times by independent firms before
their network activation at NU5 in 2022. This chapter collects the
public audit reports and the points each one raised, with the
follow-up commit when a fix is publicly traceable. The list is not
exhaustive but is a useful starting point when assessing the trust
basis of the codebase.

## what to read first

For Orchard specifically, four public reports are particularly
relevant:

- The 2021 [Halo 2 audit by Trail of Bits](https://www.trailofbits.com/reports/Halo2.pdf)
  (covering `halo2_proofs` and `halo2_gadgets`).
- The 2021 [Halo 2 audit by NCC Group](https://research.nccgroup.com/wp-content/uploads/2022/03/NCC_Group_ZcashFoundation_E001950_Report_2021-12-15.pdf).
- The 2022 [Orchard cryptographic audit by Least Authority](https://leastauthority.com/static/publications/LeastAuthority_Zcash_Orchard_Pasta_Final_Audit_Report.pdf)
  (covering Orchard, the Pasta curves, and Sinsemilla).
- The 2022 [Action circuit audit by QEDIT](https://github.com/zcash/orchard/blob/main/book/src/concepts/preliminaries.md)
  referenced from the Orchard book.

The reports are linked from
[zcash.github.io/halo2/audits.html](https://zcash.github.io/halo2/audits.html)
and from the Electric Coin Co.'s
[audit page](https://electriccoin.co/blog/audit-of-zcash-orchard-pallas/).

## recurring themes

Across the reports the categories of finding cluster around:

1. **Incomplete addition edge cases**: Sinsemilla and the ECC chip
   use incomplete addition. Several reports asked for an explicit
   argument that the failure cases (equal or opposite inputs) cannot
   be reached. The argument lives in the Halo 2 book and is encoded
   in the circuit by canonical encoding lookups.
2. **Witness canonicality**: the prover must encode field elements
   into bits in exactly one way. The
   [`src/circuit/note_commit.rs`](https://github.com/zcash/orchard/blob/main/src/circuit/note_commit.rs)
   chip enforces this with range constraints; several findings asked
   for stronger documentation of the lookup tables.
3. **Domain separation**: Sinsemilla, Poseidon, RedDSA, and the
   transcript all use distinct personalisation strings. Audits
   asked for collisions to be ruled out for each pair of domains.
4. **Side-channels in pasta_curves**: constant-time inversion and
   scalar multiplication were reviewed; the fixes are upstream in
   `pasta_curves` and the `subtle` crate.

## tracing a fix

Most fixes are visible in the
[CHANGELOG](https://github.com/zcash/orchard/blob/main/CHANGELOG.md)
of the `orchard` crate and the matching releases of `halo2_proofs`
and `pasta_curves`. The audit-point tag in the git history (search
for `auditpoint` in `git branch -a`) marks the state of the code
submitted to a particular audit; the commits between the audit-point
and the next release are the audit responses.

## specification and references

- [Halo 2 audits page](https://zcash.github.io/halo2/audits.html).
- [Electric Coin Co. blog: audit of Zcash Orchard and Pallas](https://electriccoin.co/blog/audit-of-zcash-orchard-pallas/).
- [Least Authority Orchard / Pasta audit report (PDF)](https://leastauthority.com/static/publications/LeastAuthority_Zcash_Orchard_Pasta_Final_Audit_Report.pdf).
- [Trail of Bits Halo 2 audit report (PDF)](https://www.trailofbits.com/reports/Halo2.pdf).
- [NCC Group Halo 2 audit report (PDF)](https://research.nccgroup.com/wp-content/uploads/2022/03/NCC_Group_ZcashFoundation_E001950_Report_2021-12-15.pdf).

## exercises

1. Pick one finding from the Least Authority report and locate the
   corresponding fix in the `orchard` or `halo2` repository.
2. The "incomplete addition" topic is covered in detail in the
   [Halo 2 book](https://zcash.github.io/halo2/design/gadgets/ecc/addition.html).
   Summarise in three sentences how the circuit avoids the failure
   case for Sinsemilla.
3. Compare two consecutive `CHANGELOG.md` releases of `orchard` near
   an audit date. List the commits attributed to audit feedback.
