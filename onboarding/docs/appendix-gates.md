---
sidebar_position: 21
title: "Appendix: Action Circuit Polynomial Constraints"
description: Auto-generated KaTeX rendering of every gate polynomial in the Orchard 0.13.1 Action circuit verifier key.
---

# Appendix: Action Circuit Polynomial Constraints

This appendix lists the 193 polynomial constraints of the
Orchard Action circuit at orchard 0.13.1. Each polynomial $P$
vanishes on every valid assignment: $P = 0$.

**Provenance.** The polynomials are extracted from the
[`src/circuit_description`](https://github.com/zcash/orchard/blob/f8915bc5c8d1c9fa3124ad28bcf73ce232ef3669/src/circuit_description)
dump (a serialisation of
`halo2_proofs::plonk::PinnedVerificationKey`) by the
[`gates-to-latex`](https://github.com/dannywillems/orchard/tree/onboarding/onboarding/tools/gates-to-latex)
tool that ships in this repo. To regenerate after an upstream
change, run `make appendix-gates` from the `onboarding/`
directory; the tool re-reads the vendored copy at
`onboarding/data/orchard-0.13.1-circuit_description.txt`.

**Notation.** The advice, fixed, and instance columns are
indexed by their `column_index` in the constraint system:

- $A_c$, $A_c^{(+r)}$, $A_c^{(-r)}$: advice column $c$ at the
  current row, rotated by $+r$ or $-r$.
- $F_c$, $F_c^{(+r)}$, $F_c^{(-r)}$: fixed column $c$ at the
  current row or a rotation. The pinned circuit uses 29 fixed
  columns; the lowest indices are the selector-promotion
  columns produced by Halo 2's `compress_selectors` pass, and
  the higher indices carry the chip-level constants used by
  the ECC, Sinsemilla, and Poseidon chips.
- Constants are rendered in hex. Values below `0xffff` are
  shown in full; larger values are truncated to a six-hex-digit
  head followed by `\ldots` to keep KaTeX expressions readable.

**Grouping.** Halo 2's `compress_selectors` pass packs every
`meta.create_gate(...)` group into a single shared fixed column
by giving the column a small integer value per gate member.
That value selects the member through an envelope of the form
$F_c \cdot (k_1 - F_c) \cdot \dots \cdot (k_n - F_c)$. Two
polynomials that share the same envelope column $c$ therefore
come from the same source-level `create_gate` call. We use $c$
as the group key and list polynomials per group; the
source-level chip that owns each group can be identified by
opening `src/circuit.rs` and reading the chip-configuration
calls in `Circuit::configure` in order. Polynomials that do
not match the envelope pattern are listed under "Ungrouped".

**Scope.** This is the raw polynomial form, not yet annotated
with chip-level meaning. Phase 2 of this work would attach a
source-level chip name to each group (ECC, Sinsemilla,
Poseidon, Merkle, CommitIvk, NoteCommit). Doing so cleanly
requires upstream changes in `halo2_proofs` to expose gate
names; the pinned dump deliberately strips them.

## Summary

| Envelope column $c$ | Polynomials in group | Original indices                                                  |
| ------------------- | -------------------- | ----------------------------------------------------------------- |
| $F_{16}$            | 2                    | 87, 88                                                            |
| $F_{17}$            | 2                    | 97, 98                                                            |
| $F_{18}$            | 17                   | 1, 2, 3, 4, 5, 6, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34      |
| $F_{19}$            | 17                   | 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,\,... |
| $F_{20}$            | 18                   | 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, \,... |
| $F_{21}$            | 18                   | 53, 54, 55, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, \,... |
| $F_{22}$            | 4                    | 56, 57, 58, 59                                                    |
| $F_{23}$            | 4                    | 60, 61, 62, 63                                                    |
| $F_{24}$            | 11                   | 79, 80, 81, 82, 83, 84, 85, 86, 89, 90, 91                        |
| $F_{25}$            | 26                   | 92, 93, 94, 95, 96, 106, 107, 108, 109, 110, 111, 112, 113, \,... |
| $F_{26}$            | 21                   | 99, 100, 101, 102, 103, 104, 105, 127, 128, 129, 130, 131, 1\,... |
| $F_{27}$            | 25                   | 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, \,... |
| $F_{28}$            | 28                   | 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, \,... |

## Group 1 (envelope column $F_{16}$, 2 polynomials)

### Polynomial 87 (original index 87)

$$
\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{0}^{(+1)} + \left(A_{3}\right) \cdot \left(A_{3}\right) + -\left(A_{0}\right) + -\left(A_{1}\right) + A_{0}\right) = 0
$$

### Polynomial 88 (original index 88)

$$
\left(\mathtt{0x4} \cdot \left(A_{4}\right)\right) \cdot \left(A_{0} + -\left(A_{0}^{(+1)}\right)\right) + -\left(\mathtt{0x2} \cdot \left(\left(A_{3} + A_{4}\right) \cdot \left(A_{0} + -\left(\left(A_{3}\right) \cdot \left(A_{3}\right) + -\left(A_{0}\right) + -\left(A_{1}\right)\right)\right)\right) + \left(\mathtt{0x2} + -\left(\left(F_{12}\right) \cdot \left(F_{12} + -\left(\mathtt{0x1}\right)\right)\right)\right) \cdot \left(\left(A_{3}^{(+1)} + A_{4}^{(+1)}\right) \cdot \left(A_{0}^{(+1)} + -\left(\left(A_{3}^{(+1)}\right) \cdot \left(A_{3}^{(+1)}\right) + -\left(A_{0}^{(+1)}\right) + -\left(A_{1}^{(+1)}\right)\right)\right)\right) + \left(\mathtt{0x2} \cdot \left(\left(F_{12}\right) \cdot \left(F_{12} + -\left(\mathtt{0x1}\right)\right)\right)\right) \cdot \left(A_{3}^{(+1)}\right)\right) = 0
$$

## Group 2 (envelope column $F_{17}$, 2 polynomials)

### Polynomial 97 (original index 97)

$$
\left(A_{9}\right) \cdot \left(A_{9}\right) + -\left(A_{5}^{(+1)} + \left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{5}\right) + -\left(A_{6}\right) + A_{5}\right) = 0
$$

### Polynomial 98 (original index 98)

$$
\left(\mathtt{0x4} \cdot \left(A_{9}\right)\right) \cdot \left(A_{5} + -\left(A_{5}^{(+1)}\right)\right) + -\left(\mathtt{0x2} \cdot \left(\left(A_{8} + A_{9}\right) \cdot \left(A_{5} + -\left(\left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{5}\right) + -\left(A_{6}\right)\right)\right)\right) + \left(\mathtt{0x2} + -\left(\left(F_{13}\right) \cdot \left(F_{13} + -\left(\mathtt{0x1}\right)\right)\right)\right) \cdot \left(\left(A_{8}^{(+1)} + A_{9}^{(+1)}\right) \cdot \left(A_{5}^{(+1)} + -\left(\left(A_{8}^{(+1)}\right) \cdot \left(A_{8}^{(+1)}\right) + -\left(A_{5}^{(+1)}\right) + -\left(A_{6}^{(+1)}\right)\right)\right)\right) + \left(\mathtt{0x2} \cdot \left(\left(F_{13}\right) \cdot \left(F_{13} + -\left(\mathtt{0x1}\right)\right)\right)\right) \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

## Group 3 (envelope column $F_{18}$, 17 polynomials)

### Polynomial 1 (original index 1)

$$
A_{0} + -\left(A_{1}\right) + -\left(\left(A_{2}\right) \cdot \left(A_{3}\right)\right) = 0
$$

### Polynomial 2 (original index 2)

$$
\left(A_{0}\right) \cdot \left(A_{4} + -\left(A_{5}\right)\right) = 0
$$

### Polynomial 3 (original index 3)

$$
\left(A_{0}\right) \cdot \left(\mathtt{0x1} + -\left(A_{6}\right)\right) = 0
$$

### Polynomial 4 (original index 4)

$$
\left(A_{1}\right) \cdot \left(\mathtt{0x1} + -\left(A_{7}\right)\right) = 0
$$

### Polynomial 5 (original index 5)

$$
A_{7} + A_{8} + -\left(A_{6}\right) = 0
$$

### Polynomial 6 (original index 6)

$$
\left(\mathtt{0x400} \cdot \left(A_{9}^{(-1)}\right)\right) \cdot \left(A_{9}^{(+1)}\right) + -\left(A_{9}\right) = 0
$$

### Polynomial 24 (original index 24)

$$
A_{4} + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{4}^{(+1)} + A_{5}^{(+1)}\right) \cdot \left(A_{3}^{(+1)} + -\left(\left(A_{4}^{(+1)}\right) \cdot \left(A_{4}^{(+1)}\right) + -\left(A_{3}^{(+1)}\right) + -\left(A_{0}^{(+1)}\right)\right)\right)\right)\right) = 0
$$

### Polynomial 25 (original index 25)

$$
A_{0} + -\left(A_{0}^{(+1)}\right) = 0
$$

### Polynomial 26 (original index 26)

$$
A_{1} + -\left(A_{1}^{(+1)}\right) = 0
$$

### Polynomial 27 (original index 27)

$$
\left(A_{9} + -\left(\mathtt{0x2} \cdot \left(A_{9}^{(-1)}\right)\right)\right) \cdot \left(\mathtt{0x1} + -\left(A_{9} + -\left(\mathtt{0x2} \cdot \left(A_{9}^{(-1)}\right)\right)\right)\right) = 0
$$

### Polynomial 28 (original index 28)

$$
\left(A_{4}\right) \cdot \left(A_{3} + -\left(A_{0}\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{4} + A_{5}\right) \cdot \left(A_{3} + -\left(\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{3}\right) + -\left(A_{0}\right)\right)\right)\right)\right) + \left(\mathtt{0x2} \cdot \left(A_{9} + -\left(\mathtt{0x2} \cdot \left(A_{9}^{(-1)}\right)\right)\right) + -\left(\mathtt{0x1}\right)\right) \cdot \left(A_{1}\right) = 0
$$

### Polynomial 29 (original index 29)

$$
\left(A_{5}\right) \cdot \left(A_{5}\right) + -\left(A_{3}^{(+1)}\right) + -\left(\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{3}\right) + -\left(A_{0}\right)\right) + -\left(A_{3}\right) = 0
$$

### Polynomial 30 (original index 30)

$$
\left(A_{5}\right) \cdot \left(A_{3} + -\left(A_{3}^{(+1)}\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{4} + A_{5}\right) \cdot \left(A_{3} + -\left(\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{3}\right) + -\left(A_{0}\right)\right)\right)\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{4}^{(+1)} + A_{5}^{(+1)}\right) \cdot \left(A_{3}^{(+1)} + -\left(\left(A_{4}^{(+1)}\right) \cdot \left(A_{4}^{(+1)}\right) + -\left(A_{3}^{(+1)}\right) + -\left(A_{0}^{(+1)}\right)\right)\right)\right)\right) = 0
$$

### Polynomial 31 (original index 31)

$$
\left(A_{9} + -\left(\mathtt{0x2} \cdot \left(A_{9}^{(-1)}\right)\right)\right) \cdot \left(\mathtt{0x1} + -\left(A_{9} + -\left(\mathtt{0x2} \cdot \left(A_{9}^{(-1)}\right)\right)\right)\right) = 0
$$

### Polynomial 32 (original index 32)

$$
\left(A_{4}\right) \cdot \left(A_{3} + -\left(A_{0}\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{4} + A_{5}\right) \cdot \left(A_{3} + -\left(\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{3}\right) + -\left(A_{0}\right)\right)\right)\right)\right) + \left(\mathtt{0x2} \cdot \left(A_{9} + -\left(\mathtt{0x2} \cdot \left(A_{9}^{(-1)}\right)\right)\right) + -\left(\mathtt{0x1}\right)\right) \cdot \left(A_{1}\right) = 0
$$

### Polynomial 33 (original index 33)

$$
\left(A_{5}\right) \cdot \left(A_{5}\right) + -\left(A_{3}^{(+1)}\right) + -\left(\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{3}\right) + -\left(A_{0}\right)\right) + -\left(A_{3}\right) = 0
$$

### Polynomial 34 (original index 34)

$$
\left(A_{5}\right) \cdot \left(A_{3} + -\left(A_{3}^{(+1)}\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{4} + A_{5}\right) \cdot \left(A_{3} + -\left(\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{3}\right) + -\left(A_{0}\right)\right)\right)\right)\right) + -\left(A_{4}^{(+1)}\right) = 0
$$

## Group 4 (envelope column $F_{19}$, 17 polynomials)

### Polynomial 7 (original index 7)

$$
\left(A_{1}\right) \cdot \left(A_{1}\right) + -\left(\left(\left(A_{0}\right) \cdot \left(A_{0}\right)\right) \cdot \left(A_{0}\right)\right) + -\left(\mathtt{0x5}\right) = 0
$$

### Polynomial 8 (original index 8)

$$
\left(A_{1}\right) \cdot \left(A_{1}\right) + -\left(\left(\left(A_{0}\right) \cdot \left(A_{0}\right)\right) \cdot \left(A_{0}\right)\right) + -\left(\mathtt{0x5}\right) = 0
$$

### Polynomial 9 (original index 9)

$$
\left(A_{1}\right) \cdot \left(A_{1}\right) + -\left(\left(\left(A_{0}\right) \cdot \left(A_{0}\right)\right) \cdot \left(A_{0}\right)\right) + -\left(\mathtt{0x5}\right) = 0
$$

### Polynomial 10 (original index 10)

$$
\left(\left(A_{2}^{(+1)} + A_{2} + A_{0}\right) \cdot \left(A_{0} + -\left(A_{2}\right)\right)\right) \cdot \left(A_{0} + -\left(A_{2}\right)\right) + -\left(\left(A_{1} + -\left(A_{3}\right)\right) \cdot \left(A_{1} + -\left(A_{3}\right)\right)\right) = 0
$$

### Polynomial 11 (original index 11)

$$
\left(A_{3}^{(+1)} + A_{3}\right) \cdot \left(A_{0} + -\left(A_{2}\right)\right) + -\left(\left(A_{1} + -\left(A_{3}\right)\right) \cdot \left(A_{2} + -\left(A_{2}^{(+1)}\right)\right)\right) = 0
$$

### Polynomial 12 (original index 12)

$$
\left(A_{2} + -\left(A_{0}\right)\right) \cdot \left(\left(A_{2} + -\left(A_{0}\right)\right) \cdot \left(A_{4}\right) + -\left(A_{3} + -\left(A_{1}\right)\right)\right) = 0
$$

### Polynomial 13 (original index 13)

$$
\left(\mathtt{0x1} + -\left(\left(A_{2} + -\left(A_{0}\right)\right) \cdot \left(A_{5}\right)\right)\right) \cdot \left(\left(\left(\mathtt{0x2}\right) \cdot \left(A_{1}\right)\right) \cdot \left(A_{4}\right) + -\left(\left(\mathtt{0x3}\right) \cdot \left(\left(A_{0}\right) \cdot \left(A_{0}\right)\right)\right)\right) = 0
$$

### Polynomial 14 (original index 14)

$$
\left(\left(\left(A_{0}\right) \cdot \left(A_{2}\right)\right) \cdot \left(A_{2} + -\left(A_{0}\right)\right)\right) \cdot \left(\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{0}\right) + -\left(A_{2}\right) + -\left(A_{2}^{(+1)}\right)\right) = 0
$$

### Polynomial 15 (original index 15)

$$
\left(\left(\left(A_{0}\right) \cdot \left(A_{2}\right)\right) \cdot \left(A_{2} + -\left(A_{0}\right)\right)\right) \cdot \left(\left(A_{4}\right) \cdot \left(A_{0} + -\left(A_{2}^{(+1)}\right)\right) + -\left(A_{1}\right) + -\left(A_{3}^{(+1)}\right)\right) = 0
$$

### Polynomial 16 (original index 16)

$$
\left(\left(\left(A_{0}\right) \cdot \left(A_{2}\right)\right) \cdot \left(A_{3} + A_{1}\right)\right) \cdot \left(\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(A_{0}\right) + -\left(A_{2}\right) + -\left(A_{2}^{(+1)}\right)\right) = 0
$$

### Polynomial 17 (original index 17)

$$
\left(\left(\left(A_{0}\right) \cdot \left(A_{2}\right)\right) \cdot \left(A_{3} + A_{1}\right)\right) \cdot \left(\left(A_{4}\right) \cdot \left(A_{0} + -\left(A_{2}^{(+1)}\right)\right) + -\left(A_{1}\right) + -\left(A_{3}^{(+1)}\right)\right) = 0
$$

### Polynomial 18 (original index 18)

$$
\left(\mathtt{0x1} + -\left(\left(A_{0}\right) \cdot \left(A_{6}\right)\right)\right) \cdot \left(A_{2}^{(+1)} + -\left(A_{2}\right)\right) = 0
$$

### Polynomial 19 (original index 19)

$$
\left(\mathtt{0x1} + -\left(\left(A_{0}\right) \cdot \left(A_{6}\right)\right)\right) \cdot \left(A_{3}^{(+1)} + -\left(A_{3}\right)\right) = 0
$$

### Polynomial 20 (original index 20)

$$
\left(\mathtt{0x1} + -\left(\left(A_{2}\right) \cdot \left(A_{7}\right)\right)\right) \cdot \left(A_{2}^{(+1)} + -\left(A_{0}\right)\right) = 0
$$

### Polynomial 21 (original index 21)

$$
\left(\mathtt{0x1} + -\left(\left(A_{2}\right) \cdot \left(A_{7}\right)\right)\right) \cdot \left(A_{3}^{(+1)} + -\left(A_{1}\right)\right) = 0
$$

### Polynomial 22 (original index 22)

$$
\left(\mathtt{0x1} + -\left(\left(A_{2} + -\left(A_{0}\right)\right) \cdot \left(A_{5}\right)\right) + -\left(\left(A_{3} + A_{1}\right) \cdot \left(A_{8}\right)\right)\right) \cdot \left(A_{2}^{(+1)}\right) = 0
$$

### Polynomial 23 (original index 23)

$$
\left(\mathtt{0x1} + -\left(\left(A_{2} + -\left(A_{0}\right)\right) \cdot \left(A_{5}\right)\right) + -\left(\left(A_{3} + A_{1}\right) \cdot \left(A_{8}\right)\right)\right) \cdot \left(A_{3}^{(+1)}\right) = 0
$$

## Group 5 (envelope column $F_{20}$, 18 polynomials)

### Polynomial 35 (original index 35)

$$
A_{8} + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{8}^{(+1)} + A_{2}^{(+1)}\right) \cdot \left(A_{7}^{(+1)} + -\left(\left(A_{8}^{(+1)}\right) \cdot \left(A_{8}^{(+1)}\right) + -\left(A_{7}^{(+1)}\right) + -\left(A_{0}^{(+1)}\right)\right)\right)\right)\right) = 0
$$

### Polynomial 36 (original index 36)

$$
A_{0} + -\left(A_{0}^{(+1)}\right) = 0
$$

### Polynomial 37 (original index 37)

$$
A_{1} + -\left(A_{1}^{(+1)}\right) = 0
$$

### Polynomial 38 (original index 38)

$$
\left(A_{6} + -\left(\mathtt{0x2} \cdot \left(A_{6}^{(-1)}\right)\right)\right) \cdot \left(\mathtt{0x1} + -\left(A_{6} + -\left(\mathtt{0x2} \cdot \left(A_{6}^{(-1)}\right)\right)\right)\right) = 0
$$

### Polynomial 39 (original index 39)

$$
\left(A_{8}\right) \cdot \left(A_{7} + -\left(A_{0}\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{8} + A_{2}\right) \cdot \left(A_{7} + -\left(\left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{7}\right) + -\left(A_{0}\right)\right)\right)\right)\right) + \left(\mathtt{0x2} \cdot \left(A_{6} + -\left(\mathtt{0x2} \cdot \left(A_{6}^{(-1)}\right)\right)\right) + -\left(\mathtt{0x1}\right)\right) \cdot \left(A_{1}\right) = 0
$$

### Polynomial 40 (original index 40)

$$
\left(A_{2}\right) \cdot \left(A_{2}\right) + -\left(A_{7}^{(+1)}\right) + -\left(\left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{7}\right) + -\left(A_{0}\right)\right) + -\left(A_{7}\right) = 0
$$

### Polynomial 41 (original index 41)

$$
\left(A_{2}\right) \cdot \left(A_{7} + -\left(A_{7}^{(+1)}\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{8} + A_{2}\right) \cdot \left(A_{7} + -\left(\left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{7}\right) + -\left(A_{0}\right)\right)\right)\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{8}^{(+1)} + A_{2}^{(+1)}\right) \cdot \left(A_{7}^{(+1)} + -\left(\left(A_{8}^{(+1)}\right) \cdot \left(A_{8}^{(+1)}\right) + -\left(A_{7}^{(+1)}\right) + -\left(A_{0}^{(+1)}\right)\right)\right)\right)\right) = 0
$$

### Polynomial 42 (original index 42)

$$
\left(A_{6} + -\left(\mathtt{0x2} \cdot \left(A_{6}^{(-1)}\right)\right)\right) \cdot \left(\mathtt{0x1} + -\left(A_{6} + -\left(\mathtt{0x2} \cdot \left(A_{6}^{(-1)}\right)\right)\right)\right) = 0
$$

### Polynomial 43 (original index 43)

$$
\left(A_{8}\right) \cdot \left(A_{7} + -\left(A_{0}\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{8} + A_{2}\right) \cdot \left(A_{7} + -\left(\left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{7}\right) + -\left(A_{0}\right)\right)\right)\right)\right) + \left(\mathtt{0x2} \cdot \left(A_{6} + -\left(\mathtt{0x2} \cdot \left(A_{6}^{(-1)}\right)\right)\right) + -\left(\mathtt{0x1}\right)\right) \cdot \left(A_{1}\right) = 0
$$

### Polynomial 44 (original index 44)

$$
\left(A_{2}\right) \cdot \left(A_{2}\right) + -\left(A_{7}^{(+1)}\right) + -\left(\left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{7}\right) + -\left(A_{0}\right)\right) + -\left(A_{7}\right) = 0
$$

### Polynomial 45 (original index 45)

$$
\left(A_{2}\right) \cdot \left(A_{7} + -\left(A_{7}^{(+1)}\right)\right) + -\left(\mathtt{0x200000\ldots} \cdot \left(\left(A_{8} + A_{2}\right) \cdot \left(A_{7} + -\left(\left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{7}\right) + -\left(A_{0}\right)\right)\right)\right)\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 46 (original index 46)

$$
\left(A_{9}^{(+1)} + -\left(\left(\mathtt{0x2}\right) \cdot \left(A_{9}^{(-1)}\right)\right)\right) \cdot \left(\mathtt{0x1} + -\left(A_{9}^{(+1)} + -\left(\left(\mathtt{0x2}\right) \cdot \left(A_{9}^{(-1)}\right)\right)\right)\right) = 0
$$

### Polynomial 47 (original index 47)

$$
\left(A_{9}^{(+1)} + -\left(\left(\mathtt{0x2}\right) \cdot \left(A_{9}^{(-1)}\right)\right)\right) \cdot \left(A_{9} + -\left(A_{1}^{(-1)}\right)\right) + \left(\mathtt{0x1} + -\left(A_{9}^{(+1)} + -\left(\left(\mathtt{0x2}\right) \cdot \left(A_{9}^{(-1)}\right)\right)\right)\right) \cdot \left(A_{9} + A_{1}^{(-1)}\right) = 0
$$

### Polynomial 48 (original index 48)

$$
A_{8} + -\left(A_{7} + \left(A_{7}^{(-1)}\right) \cdot \left(\left(\mathtt{0x100000\ldots}\right) \cdot \left(\mathtt{0x40}\right)\right)\right) = 0
$$

### Polynomial 49 (original index 49)

$$
A_{6}^{(-1)} + -\left(A_{7}\right) + -\left(\mathtt{0x224698\ldots}\right) = 0
$$

### Polynomial 50 (original index 50)

$$
\left(A_{7}^{(-1)}\right) \cdot \left(A_{6} + -\left(\mathtt{0x100000\ldots}\right)\right) = 0
$$

### Polynomial 51 (original index 51)

$$
\left(A_{7}^{(-1)}\right) \cdot \left(A_{7}^{(+1)}\right) = 0
$$

### Polynomial 52 (original index 52)

$$
\left(\left(\mathtt{0x1} + -\left(A_{7}^{(-1)}\right)\right) \cdot \left(\mathtt{0x1} + -\left(\left(A_{6}\right) \cdot \left(A_{6}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{7}^{(+1)}\right) = 0
$$

## Group 6 (envelope column $F_{21}$, 18 polynomials)

### Polynomial 53 (original index 53)

$$
\left(A_{9}^{(+1)} + -\left(\mathtt{0x2} \cdot \left(A_{9}\right)\right)\right) \cdot \left(\mathtt{0x1} + -\left(A_{9}^{(+1)} + -\left(\mathtt{0x2} \cdot \left(A_{9}\right)\right)\right)\right) = 0
$$

### Polynomial 54 (original index 54)

$$
\left(A_{9}^{(+1)} + -\left(\mathtt{0x2} \cdot \left(A_{9}\right)\right)\right) \cdot \left(A_{0}\right) + \left(\mathtt{0x1} + -\left(A_{9}^{(+1)} + -\left(\mathtt{0x2} \cdot \left(A_{9}\right)\right)\right)\right) \cdot \left(A_{0} + -\left(A_{0}^{(+1)}\right)\right) = 0
$$

### Polynomial 55 (original index 55)

$$
\left(A_{9}^{(+1)} + -\left(\mathtt{0x2} \cdot \left(A_{9}\right)\right)\right) \cdot \left(A_{1}\right) + \left(\mathtt{0x1} + -\left(A_{9}^{(+1)} + -\left(\mathtt{0x2} \cdot \left(A_{9}\right)\right)\right)\right) \cdot \left(A_{1} + A_{1}^{(+1)}\right) = 0
$$

### Polynomial 64 (original index 64)

$$
\left(A_{5}\right) \cdot \left(\mathtt{0x1} + -\left(A_{5}\right)\right) = 0
$$

### Polynomial 65 (original index 65)

$$
\left(A_{4}\right) \cdot \left(A_{4}\right) + -\left(\mathtt{0x1}\right) = 0
$$

### Polynomial 66 (original index 66)

$$
\left(A_{1} + -\left(A_{3}\right)\right) \cdot \left(A_{1} + A_{3}\right) = 0
$$

### Polynomial 67 (original index 67)

$$
\left(A_{4}\right) \cdot \left(A_{1}\right) + -\left(A_{3}\right) = 0
$$

### Polynomial 68 (original index 68)

$$
\left(A_{8}\right) \cdot \left(A_{7}\right) = 0
$$

### Polynomial 69 (original index 69)

$$
\left(A_{8}\right) \cdot \left(A_{7}^{(+1)} + -\left(\left(A_{8}^{(-1)}\right) \cdot \left(\mathtt{0x100000\ldots}\right)\right)\right) = 0
$$

### Polynomial 70 (original index 70)

$$
\left(A_{8}\right) \cdot \left(\left(A_{8}^{(+1)} + -\left(\mathtt{0x8} \cdot \left(A_{7}^{(+1)}\right)\right)\right) \cdot \left(\mathtt{0x1} + -\left(A_{8}^{(+1)} + -\left(\mathtt{0x8} \cdot \left(A_{7}^{(+1)}\right)\right)\right)\right)\right) = 0
$$

### Polynomial 71 (original index 71)

$$
\left(A_{8}\right) \cdot \left(A_{6}^{(+1)}\right) = 0
$$

### Polynomial 72 (original index 72)

$$
\left(\left(\left(A_{7}\right) \cdot \left(\mathtt{0x1} + -\left(A_{7}\right)\right)\right) \cdot \left(\mathtt{0x2} + -\left(A_{7}\right)\right)\right) \cdot \left(\mathtt{0x3} + -\left(A_{7}\right)\right) = 0
$$

### Polynomial 73 (original index 73)

$$
\left(A_{8}\right) \cdot \left(\mathtt{0x1} + -\left(A_{8}\right)\right) = 0
$$

### Polynomial 74 (original index 74)

$$
A_{8}^{(-1)} + -\left(A_{7} + \mathtt{0x4} \cdot \left(A_{8}\right)\right) = 0
$$

### Polynomial 75 (original index 75)

$$
A_{6} + -\left(A_{6}^{(-1)} + -\left(\mathtt{0x100000\ldots} \cdot \left(A_{8}^{(-1)}\right)\right) + \mathtt{0x400000\ldots} + -\left(\mathtt{0x224698\ldots}\right)\right) = 0
$$

### Polynomial 76 (original index 76)

$$
\mathtt{0xab5e5b\ldots} \cdot \left(\left(\left(\left(A_{6} + F_{5}\right) \cdot \left(A_{6} + F_{5}\right)\right) \cdot \left(\left(A_{6} + F_{5}\right) \cdot \left(A_{6} + F_{5}\right)\right)\right) \cdot \left(A_{6} + F_{5}\right)\right) + \mathtt{0x319166\ldots} \cdot \left(\left(\left(\left(A_{7} + F_{6}\right) \cdot \left(A_{7} + F_{6}\right)\right) \cdot \left(\left(A_{7} + F_{6}\right) \cdot \left(A_{7} + F_{6}\right)\right)\right) \cdot \left(A_{7} + F_{6}\right)\right) + \mathtt{0x7c045d\ldots} \cdot \left(\left(\left(\left(A_{8} + F_{7}\right) \cdot \left(A_{8} + F_{7}\right)\right) \cdot \left(\left(A_{8} + F_{7}\right) \cdot \left(A_{8} + F_{7}\right)\right)\right) \cdot \left(A_{8} + F_{7}\right)\right) + -\left(A_{6}^{(+1)}\right) = 0
$$

### Polynomial 77 (original index 77)

$$
\mathtt{0x233162\ldots} \cdot \left(\left(\left(\left(A_{6} + F_{5}\right) \cdot \left(A_{6} + F_{5}\right)\right) \cdot \left(\left(A_{6} + F_{5}\right) \cdot \left(A_{6} + F_{5}\right)\right)\right) \cdot \left(A_{6} + F_{5}\right)\right) + \mathtt{0x25cae2\ldots} \cdot \left(\left(\left(\left(A_{7} + F_{6}\right) \cdot \left(A_{7} + F_{6}\right)\right) \cdot \left(\left(A_{7} + F_{6}\right) \cdot \left(A_{7} + F_{6}\right)\right)\right) \cdot \left(A_{7} + F_{6}\right)\right) + \mathtt{0x22f5b5\ldots} \cdot \left(\left(\left(\left(A_{8} + F_{7}\right) \cdot \left(A_{8} + F_{7}\right)\right) \cdot \left(\left(A_{8} + F_{7}\right) \cdot \left(A_{8} + F_{7}\right)\right)\right) \cdot \left(A_{8} + F_{7}\right)\right) + -\left(A_{7}^{(+1)}\right) = 0
$$

### Polynomial 78 (original index 78)

$$
\mathtt{0x2e29dd\ldots} \cdot \left(\left(\left(\left(A_{6} + F_{5}\right) \cdot \left(A_{6} + F_{5}\right)\right) \cdot \left(\left(A_{6} + F_{5}\right) \cdot \left(A_{6} + F_{5}\right)\right)\right) \cdot \left(A_{6} + F_{5}\right)\right) + \mathtt{0x1d1aab\ldots} \cdot \left(\left(\left(\left(A_{7} + F_{6}\right) \cdot \left(A_{7} + F_{6}\right)\right) \cdot \left(\left(A_{7} + F_{6}\right) \cdot \left(A_{7} + F_{6}\right)\right)\right) \cdot \left(A_{7} + F_{6}\right)\right) + \mathtt{0x3bf763\ldots} \cdot \left(\left(\left(\left(A_{8} + F_{7}\right) \cdot \left(A_{8} + F_{7}\right)\right) \cdot \left(\left(A_{8} + F_{7}\right) \cdot \left(A_{8} + F_{7}\right)\right)\right) \cdot \left(A_{8} + F_{7}\right)\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

## Group 7 (envelope column $F_{22}$, 4 polynomials)

### Polynomial 56 (original index 56)

$$
\left(\left(\left(\left(\left(\left(\left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right) \cdot \left(\mathtt{0x1} + -\left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right)\right) \cdot \left(\mathtt{0x2} + -\left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right)\right) \cdot \left(\mathtt{0x3} + -\left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right)\right) \cdot \left(\mathtt{0x4} + -\left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right)\right) \cdot \left(\mathtt{0x5} + -\left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right)\right) \cdot \left(\mathtt{0x6} + -\left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right)\right) \cdot \left(\mathtt{0x7} + -\left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) = 0
$$

### Polynomial 57 (original index 57)

$$
0 + \left(\mathtt{0x1}\right) \cdot \left(F_{3}\right) + \left(\left(\mathtt{0x1}\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(F_{4}\right) + \left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(F_{5}\right) + \left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(F_{6}\right) + \left(\left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(F_{7}\right) + \left(\left(\left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(F_{8}\right) + \left(\left(\left(\left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(F_{9}\right) + \left(\left(\left(\left(\left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(A_{4} + -\left(\mathtt{0x8} \cdot \left(A_{4}^{(+1)}\right)\right)\right)\right) \cdot \left(F_{10}\right) + -\left(A_{0}\right) = 0
$$

### Polynomial 58 (original index 58)

$$
\left(A_{5}\right) \cdot \left(A_{5}\right) + -\left(A_{1}\right) + -\left(F_{11}\right) = 0
$$

### Polynomial 59 (original index 59)

$$
\left(A_{1}\right) \cdot \left(A_{1}\right) + -\left(\left(\left(A_{0}\right) \cdot \left(A_{0}\right)\right) \cdot \left(A_{0}\right)\right) + -\left(\mathtt{0x5}\right) = 0
$$

## Group 8 (envelope column $F_{23}$, 4 polynomials)

### Polynomial 60 (original index 60)

$$
0 + \left(\mathtt{0x1}\right) \cdot \left(F_{3}\right) + \left(\left(\mathtt{0x1}\right) \cdot \left(A_{4}\right)\right) \cdot \left(F_{4}\right) + \left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(F_{5}\right) + \left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(F_{6}\right) + \left(\left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(F_{7}\right) + \left(\left(\left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(F_{8}\right) + \left(\left(\left(\left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(F_{9}\right) + \left(\left(\left(\left(\left(\left(\left(\left(\mathtt{0x1}\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(A_{4}\right)\right) \cdot \left(F_{10}\right) + -\left(A_{0}\right) = 0
$$

### Polynomial 61 (original index 61)

$$
\left(A_{5}\right) \cdot \left(A_{5}\right) + -\left(A_{1}\right) + -\left(F_{11}\right) = 0
$$

### Polynomial 62 (original index 62)

$$
\left(A_{1}\right) \cdot \left(A_{1}\right) + -\left(\left(\left(A_{0}\right) \cdot \left(A_{0}\right)\right) \cdot \left(A_{0}\right)\right) + -\left(\mathtt{0x5}\right) = 0
$$

### Polynomial 63 (original index 63)

$$
\left(\left(\left(\left(\left(\left(\left(A_{4}\right) \cdot \left(\mathtt{0x1} + -\left(A_{4}\right)\right)\right) \cdot \left(\mathtt{0x2} + -\left(A_{4}\right)\right)\right) \cdot \left(\mathtt{0x3} + -\left(A_{4}\right)\right)\right) \cdot \left(\mathtt{0x4} + -\left(A_{4}\right)\right)\right) \cdot \left(\mathtt{0x5} + -\left(A_{4}\right)\right)\right) \cdot \left(\mathtt{0x6} + -\left(A_{4}\right)\right)\right) \cdot \left(\mathtt{0x7} + -\left(A_{4}\right)\right) = 0
$$

## Group 9 (envelope column $F_{24}$, 11 polynomials)

### Polynomial 79 (original index 79)

$$
\left(\left(\left(A_{6} + F_{5}\right) \cdot \left(A_{6} + F_{5}\right)\right) \cdot \left(\left(A_{6} + F_{5}\right) \cdot \left(A_{6} + F_{5}\right)\right)\right) \cdot \left(A_{6} + F_{5}\right) + -\left(A_{5}\right) = 0
$$

### Polynomial 80 (original index 80)

$$
\left(\left(\left(\mathtt{0xab5e5b\ldots} \cdot \left(A_{5}\right) + \mathtt{0x319166\ldots} \cdot \left(A_{7} + F_{6}\right) + \mathtt{0x7c045d\ldots} \cdot \left(A_{8} + F_{7}\right) + F_{8}\right) \cdot \left(\mathtt{0xab5e5b\ldots} \cdot \left(A_{5}\right) + \mathtt{0x319166\ldots} \cdot \left(A_{7} + F_{6}\right) + \mathtt{0x7c045d\ldots} \cdot \left(A_{8} + F_{7}\right) + F_{8}\right)\right) \cdot \left(\left(\mathtt{0xab5e5b\ldots} \cdot \left(A_{5}\right) + \mathtt{0x319166\ldots} \cdot \left(A_{7} + F_{6}\right) + \mathtt{0x7c045d\ldots} \cdot \left(A_{8} + F_{7}\right) + F_{8}\right) \cdot \left(\mathtt{0xab5e5b\ldots} \cdot \left(A_{5}\right) + \mathtt{0x319166\ldots} \cdot \left(A_{7} + F_{6}\right) + \mathtt{0x7c045d\ldots} \cdot \left(A_{8} + F_{7}\right) + F_{8}\right)\right)\right) \cdot \left(\mathtt{0xab5e5b\ldots} \cdot \left(A_{5}\right) + \mathtt{0x319166\ldots} \cdot \left(A_{7} + F_{6}\right) + \mathtt{0x7c045d\ldots} \cdot \left(A_{8} + F_{7}\right) + F_{8}\right) + -\left(\mathtt{0x2cc057\ldots} \cdot \left(A_{6}^{(+1)}\right) + \mathtt{0x32e7c4\ldots} \cdot \left(A_{7}^{(+1)}\right) + \mathtt{0x2eae5d\ldots} \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

### Polynomial 81 (original index 81)

$$
\mathtt{0x233162\ldots} \cdot \left(A_{5}\right) + \mathtt{0x25cae2\ldots} \cdot \left(A_{7} + F_{6}\right) + \mathtt{0x22f5b5\ldots} \cdot \left(A_{8} + F_{7}\right) + F_{9} + -\left(\mathtt{0x7bf368\ldots} \cdot \left(A_{6}^{(+1)}\right) + \mathtt{0x2aec69\ldots} \cdot \left(A_{7}^{(+1)}\right) + \mathtt{0x952e02\ldots} \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

### Polynomial 82 (original index 82)

$$
\mathtt{0x2e29dd\ldots} \cdot \left(A_{5}\right) + \mathtt{0x1d1aab\ldots} \cdot \left(A_{7} + F_{6}\right) + \mathtt{0x3bf763\ldots} \cdot \left(A_{8} + F_{7}\right) + F_{10} + -\left(\mathtt{0x2fcbba\ldots} \cdot \left(A_{6}^{(+1)}\right) + \mathtt{0x1ec737\ldots} \cdot \left(A_{7}^{(+1)}\right) + \mathtt{0xd0c2ef\ldots} \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

### Polynomial 83 (original index 83)

$$
A_{6}^{(-1)} + A_{6} + -\left(A_{6}^{(+1)}\right) = 0
$$

### Polynomial 84 (original index 84)

$$
A_{7}^{(-1)} + A_{7} + -\left(A_{7}^{(+1)}\right) = 0
$$

### Polynomial 85 (original index 85)

$$
A_{8}^{(-1)} + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 86 (original index 86)

$$
\mathtt{0x2} \cdot \left(F_{3}\right) + -\left(\left(A_{3} + A_{4}\right) \cdot \left(A_{0} + -\left(\left(A_{3}\right) \cdot \left(A_{3}\right) + -\left(A_{0}\right) + -\left(A_{1}\right)\right)\right)\right) = 0
$$

### Polynomial 89 (original index 89)

$$
A_{2} + -\left(\left(A_{4}\right) \cdot \left(A_{1}\right) + \left(\mathtt{0x1} + -\left(A_{4}\right)\right) \cdot \left(A_{0}\right)\right) = 0
$$

### Polynomial 90 (original index 90)

$$
A_{3} + -\left(\left(A_{4}\right) \cdot \left(A_{0}\right) + \left(\mathtt{0x1} + -\left(A_{4}\right)\right) \cdot \left(A_{1}\right)\right) = 0
$$

### Polynomial 91 (original index 91)

$$
\left(A_{4}\right) \cdot \left(\mathtt{0x1} + -\left(A_{4}\right)\right) = 0
$$

## Group 10 (envelope column $F_{25}$, 26 polynomials)

### Polynomial 92 (original index 92)

$$
A_{0} + -\left(\mathtt{0x400} \cdot \left(A_{0}^{(+1)}\right)\right) + -\left(A_{4}^{(+1)}\right) = 0
$$

### Polynomial 93 (original index 93)

$$
A_{0}^{(+1)} + \mathtt{0x100000\ldots} \cdot \left(A_{1} + -\left(\mathtt{0x400} \cdot \left(A_{1}^{(+1)}\right)\right) + \mathtt{0x400} \cdot \left(A_{2}^{(+1)}\right)\right) + -\left(A_{3}\right) = 0
$$

### Polynomial 94 (original index 94)

$$
A_{3}^{(+1)} + \mathtt{0x20} \cdot \left(A_{2}\right) + -\left(A_{4}\right) = 0
$$

### Polynomial 95 (original index 95)

$$
A_{1}^{(+1)} + -\left(A_{2}^{(+1)} + \mathtt{0x20} \cdot \left(A_{3}^{(+1)}\right)\right) = 0
$$

### Polynomial 96 (original index 96)

$$
\mathtt{0x2} \cdot \left(F_{4}\right) + -\left(\left(A_{8} + A_{9}\right) \cdot \left(A_{5} + -\left(\left(A_{8}\right) \cdot \left(A_{8}\right) + -\left(A_{5}\right) + -\left(A_{6}\right)\right)\right)\right) = 0
$$

### Polynomial 106 (original index 106)

$$
\left(A_{4}\right) \cdot \left(\mathtt{0x1} + -\left(A_{4}\right)\right) = 0
$$

### Polynomial 107 (original index 107)

$$
\left(A_{4}^{(+1)}\right) \cdot \left(\mathtt{0x1} + -\left(A_{4}^{(+1)}\right)\right) = 0
$$

### Polynomial 108 (original index 108)

$$
A_{2} + -\left(A_{3} + \mathtt{0x10} \cdot \left(A_{4}\right) + \mathtt{0x20} \cdot \left(A_{5}\right)\right) = 0
$$

### Polynomial 109 (original index 109)

$$
A_{2}^{(+1)} + -\left(A_{3}^{(+1)} + \mathtt{0x200} \cdot \left(A_{4}^{(+1)}\right)\right) = 0
$$

### Polynomial 110 (original index 110)

$$
A_{1} + \mathtt{0x400000\ldots} \cdot \left(A_{3}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{4}\right) + -\left(A_{0}\right) = 0
$$

### Polynomial 111 (original index 111)

$$
A_{5} + \mathtt{0x20} \cdot \left(A_{1}^{(+1)}\right) + \mathtt{0x200000\ldots} \cdot \left(A_{3}^{(+1)}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{4}^{(+1)}\right) + -\left(A_{0}^{(+1)}\right) = 0
$$

### Polynomial 112 (original index 112)

$$
\left(A_{4}\right) \cdot \left(A_{3}\right) = 0
$$

### Polynomial 113 (original index 113)

$$
\left(A_{4}\right) \cdot \left(A_{6}\right) = 0
$$

### Polynomial 114 (original index 114)

$$
A_{1} + \mathtt{0x400000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{7}\right) = 0
$$

### Polynomial 115 (original index 115)

$$
\left(A_{4}\right) \cdot \left(A_{8}\right) = 0
$$

### Polynomial 116 (original index 116)

$$
\left(A_{4}^{(+1)}\right) \cdot \left(A_{3}^{(+1)}\right) = 0
$$

### Polynomial 117 (original index 117)

$$
\left(A_{4}^{(+1)}\right) \cdot \left(A_{6}^{(+1)}\right) = 0
$$

### Polynomial 118 (original index 118)

$$
A_{5} + \mathtt{0x20} \cdot \left(A_{1}^{(+1)}\right) + \mathtt{0x100000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{7}^{(+1)}\right) = 0
$$

### Polynomial 119 (original index 119)

$$
\left(A_{4}^{(+1)}\right) \cdot \left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 120 (original index 120)

$$
\left(A_{8}\right) \cdot \left(\mathtt{0x1} + -\left(A_{8}\right)\right) = 0
$$

### Polynomial 121 (original index 121)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(\mathtt{0x1} + -\left(A_{7}^{(+1)}\right)\right) = 0
$$

### Polynomial 122 (original index 122)

$$
A_{6} + -\left(A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x20} \cdot \left(A_{7}^{(+1)}\right) + \mathtt{0x40} \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

### Polynomial 123 (original index 123)

$$
\left(A_{7}\right) \cdot \left(\mathtt{0x1} + -\left(A_{7}\right)\right) = 0
$$

### Polynomial 124 (original index 124)

$$
\left(A_{8}\right) \cdot \left(\mathtt{0x1} + -\left(A_{8}\right)\right) = 0
$$

### Polynomial 125 (original index 125)

$$
A_{6} + -\left(A_{7} + \mathtt{0x2} \cdot \left(A_{8}\right) + \mathtt{0x4} \cdot \left(A_{7}^{(+1)}\right) + \mathtt{0x400} \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

### Polynomial 126 (original index 126)

$$
A_{6} + -\left(A_{7} + \mathtt{0x40} \cdot \left(A_{8}\right)\right) = 0
$$

## Group 11 (envelope column $F_{26}$, 21 polynomials)

### Polynomial 99 (original index 99)

$$
A_{7} + -\left(\left(A_{9}\right) \cdot \left(A_{6}\right) + \left(\mathtt{0x1} + -\left(A_{9}\right)\right) \cdot \left(A_{5}\right)\right) = 0
$$

### Polynomial 100 (original index 100)

$$
A_{8} + -\left(\left(A_{9}\right) \cdot \left(A_{5}\right) + \left(\mathtt{0x1} + -\left(A_{9}\right)\right) \cdot \left(A_{6}\right)\right) = 0
$$

### Polynomial 101 (original index 101)

$$
\left(A_{9}\right) \cdot \left(\mathtt{0x1} + -\left(A_{9}\right)\right) = 0
$$

### Polynomial 102 (original index 102)

$$
A_{5} + -\left(\mathtt{0x400} \cdot \left(A_{5}^{(+1)}\right)\right) + -\left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 103 (original index 103)

$$
A_{5}^{(+1)} + \mathtt{0x100000\ldots} \cdot \left(A_{6} + -\left(\mathtt{0x400} \cdot \left(A_{6}^{(+1)}\right)\right) + \mathtt{0x400} \cdot \left(A_{7}^{(+1)}\right)\right) + -\left(A_{8}\right) = 0
$$

### Polynomial 104 (original index 104)

$$
A_{8}^{(+1)} + \mathtt{0x20} \cdot \left(A_{7}\right) + -\left(A_{9}\right) = 0
$$

### Polynomial 105 (original index 105)

$$
A_{6}^{(+1)} + -\left(A_{7}^{(+1)} + \mathtt{0x20} \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

### Polynomial 127 (original index 127)

$$
\left(A_{7}\right) \cdot \left(\mathtt{0x1} + -\left(A_{7}\right)\right) = 0
$$

### Polynomial 128 (original index 128)

$$
A_{6} + -\left(A_{7} + \mathtt{0x2} \cdot \left(A_{6}^{(+1)}\right) + \mathtt{0x400} \cdot \left(A_{7}^{(+1)}\right)\right) = 0
$$

### Polynomial 129 (original index 129)

$$
\left(A_{8}\right) \cdot \left(\mathtt{0x1} + -\left(A_{8}\right)\right) = 0
$$

### Polynomial 130 (original index 130)

$$
A_{6} + -\left(A_{7} + \mathtt{0x20} \cdot \left(A_{8}\right)\right) = 0
$$

### Polynomial 131 (original index 131)

$$
A_{8} + \mathtt{0x400000\ldots} \cdot \left(A_{7}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{7}^{(+1)}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 132 (original index 132)

$$
A_{8} + \mathtt{0x400000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 133 (original index 133)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{7}\right) = 0
$$

### Polynomial 134 (original index 134)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}\right) = 0
$$

### Polynomial 135 (original index 135)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 136 (original index 136)

$$
A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{7}^{(+1)}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 137 (original index 137)

$$
A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x100000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 138 (original index 138)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}\right) = 0
$$

### Polynomial 139 (original index 139)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 140 (original index 140)

$$
A_{7} + \mathtt{0x100} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{9}\right) + -\left(A_{6}\right) = 0
$$

## Group 12 (envelope column $F_{27}$, 25 polynomials)

### Polynomial 141 (original index 141)

$$
A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{7}^{(+1)}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 142 (original index 142)

$$
A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x100000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 143 (original index 143)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}\right) = 0
$$

### Polynomial 144 (original index 144)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 145 (original index 145)

$$
A_{7} + \mathtt{0x200} \cdot \left(A_{8}\right) + \mathtt{0x200000\ldots} \cdot \left(A_{6}^{(+1)}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{7}^{(+1)}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 146 (original index 146)

$$
A_{7} + \mathtt{0x200} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 147 (original index 147)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{6}^{(+1)}\right) = 0
$$

### Polynomial 148 (original index 148)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}\right) = 0
$$

### Polynomial 149 (original index 149)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 150 (original index 150)

$$
\left(A_{9}\right) \cdot \left(\mathtt{0x1} + -\left(A_{9}\right)\right) = 0
$$

### Polynomial 151 (original index 151)

$$
A_{5}^{(+1)} + -\left(A_{6} + \mathtt{0x2} \cdot \left(A_{7}\right) + \mathtt{0x400} \cdot \left(A_{6}^{(+1)}\right)\right) = 0
$$

### Polynomial 152 (original index 152)

$$
A_{5} + -\left(A_{5}^{(+1)} + \mathtt{0x400000\ldots} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{9}\right)\right) = 0
$$

### Polynomial 153 (original index 153)

$$
A_{5}^{(+1)} + \mathtt{0x400000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 154 (original index 154)

$$
\left(A_{9}\right) \cdot \left(A_{8}\right) = 0
$$

### Polynomial 155 (original index 155)

$$
\left(A_{9}\right) \cdot \left(A_{7}^{(+1)}\right) = 0
$$

### Polynomial 156 (original index 156)

$$
\left(A_{9}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 157 (original index 157)

$$
\left(A_{8}\right) \cdot \left(\mathtt{0x1} + -\left(A_{8}\right)\right) = 0
$$

### Polynomial 158 (original index 158)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(\mathtt{0x1} + -\left(A_{7}^{(+1)}\right)\right) = 0
$$

### Polynomial 159 (original index 159)

$$
A_{6} + -\left(A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x20} \cdot \left(A_{7}^{(+1)}\right) + \mathtt{0x40} \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

### Polynomial 160 (original index 160)

$$
\left(A_{7}\right) \cdot \left(\mathtt{0x1} + -\left(A_{7}\right)\right) = 0
$$

### Polynomial 161 (original index 161)

$$
\left(A_{8}\right) \cdot \left(\mathtt{0x1} + -\left(A_{8}\right)\right) = 0
$$

### Polynomial 162 (original index 162)

$$
A_{6} + -\left(A_{7} + \mathtt{0x2} \cdot \left(A_{8}\right) + \mathtt{0x4} \cdot \left(A_{7}^{(+1)}\right) + \mathtt{0x400} \cdot \left(A_{8}^{(+1)}\right)\right) = 0
$$

### Polynomial 163 (original index 163)

$$
A_{6} + -\left(A_{7} + \mathtt{0x40} \cdot \left(A_{8}\right)\right) = 0
$$

### Polynomial 164 (original index 164)

$$
\left(A_{7}\right) \cdot \left(\mathtt{0x1} + -\left(A_{7}\right)\right) = 0
$$

### Polynomial 165 (original index 165)

$$
A_{6} + -\left(A_{7} + \mathtt{0x2} \cdot \left(A_{6}^{(+1)}\right) + \mathtt{0x400} \cdot \left(A_{7}^{(+1)}\right)\right) = 0
$$

## Group 13 (envelope column $F_{28}$, 28 polynomials)

### Polynomial 166 (original index 166)

$$
\left(A_{8}\right) \cdot \left(\mathtt{0x1} + -\left(A_{8}\right)\right) = 0
$$

### Polynomial 167 (original index 167)

$$
A_{6} + -\left(A_{7} + \mathtt{0x20} \cdot \left(A_{8}\right)\right) = 0
$$

### Polynomial 168 (original index 168)

$$
A_{8} + \mathtt{0x400000\ldots} \cdot \left(A_{7}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{7}^{(+1)}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 169 (original index 169)

$$
A_{8} + \mathtt{0x400000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 170 (original index 170)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{7}\right) = 0
$$

### Polynomial 171 (original index 171)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}\right) = 0
$$

### Polynomial 172 (original index 172)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 173 (original index 173)

$$
A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{7}^{(+1)}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 174 (original index 174)

$$
A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x100000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 175 (original index 175)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}\right) = 0
$$

### Polynomial 176 (original index 176)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 177 (original index 177)

$$
A_{7} + \mathtt{0x100} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{9}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 178 (original index 178)

$$
A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{7}^{(+1)}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 179 (original index 179)

$$
A_{7} + \mathtt{0x10} \cdot \left(A_{8}\right) + \mathtt{0x100000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 180 (original index 180)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}\right) = 0
$$

### Polynomial 181 (original index 181)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 182 (original index 182)

$$
A_{7} + \mathtt{0x200} \cdot \left(A_{8}\right) + \mathtt{0x200000\ldots} \cdot \left(A_{6}^{(+1)}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{7}^{(+1)}\right) + -\left(A_{6}\right) = 0
$$

### Polynomial 183 (original index 183)

$$
A_{7} + \mathtt{0x200} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 184 (original index 184)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{6}^{(+1)}\right) = 0
$$

### Polynomial 185 (original index 185)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}\right) = 0
$$

### Polynomial 186 (original index 186)

$$
\left(A_{7}^{(+1)}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$

### Polynomial 187 (original index 187)

$$
\left(A_{9}\right) \cdot \left(\mathtt{0x1} + -\left(A_{9}\right)\right) = 0
$$

### Polynomial 188 (original index 188)

$$
A_{5}^{(+1)} + -\left(A_{6} + \mathtt{0x2} \cdot \left(A_{7}\right) + \mathtt{0x400} \cdot \left(A_{6}^{(+1)}\right)\right) = 0
$$

### Polynomial 189 (original index 189)

$$
A_{5} + -\left(A_{5}^{(+1)} + \mathtt{0x400000\ldots} \cdot \left(A_{8}\right) + \mathtt{0x400000\ldots} \cdot \left(A_{9}\right)\right) = 0
$$

### Polynomial 190 (original index 190)

$$
A_{5}^{(+1)} + \mathtt{0x400000\ldots} + -\left(\mathtt{0x224698\ldots}\right) + -\left(A_{8}^{(+1)}\right) = 0
$$

### Polynomial 191 (original index 191)

$$
\left(A_{9}\right) \cdot \left(A_{8}\right) = 0
$$

### Polynomial 192 (original index 192)

$$
\left(A_{9}\right) \cdot \left(A_{7}^{(+1)}\right) = 0
$$

### Polynomial 193 (original index 193)

$$
\left(A_{9}\right) \cdot \left(A_{9}^{(+1)}\right) = 0
$$
