//! GLV endomorphism-accelerated scalar multiplication for batched trial
//! decryption.
//!
//! Pallas carries the cube-root endomorphism φ(x, y) = (ζ_p·x, y), with
//! φ(P) = λ·P for λ = `Scalar::ZETA` and ζ_p = `Base::ZETA`. A one-time GLV
//! decomposition splits a 255-bit scalar into two signed halves of at most
//! 128 bits each, k = k1 + k2·λ (mod r_P), so that `k·P = k1·P + k2·φ(P)`
//! runs as a Straus ladder with shared doublings: ~128 doublings plus ~52
//! window-4 additions, instead of the ~255 doublings a full-width walk pays.
//!
//! Batched trial decryption multiplies every ephemeral key in a batch by
//! each of the wallet's incoming viewing keys, which makes both halves of
//! the work amortizable:
//!
//! - [`EndoTable`] is the window for one ephemeral key: the odd multiples
//!   {1, 3, 5, 7}·P and {1, 3, 5, 7}·φ(P) in affine coordinates. It is built
//!   once per ephemeral key — [`endo_tables`] normalizes the windows for the
//!   whole batch with a single shared inversion — and is then reused by every
//!   viewing key's multiplication against that key.
//! - [`DecomposedScalar`] is a viewing key's GLV split with both halves
//!   already wNAF-recoded; it is computed once per (viewing key, batch) and
//!   consumed read-only by every ladder run.
//!
//! Correctness does not rest on how the GLV constants below were derived:
//! `decompose_reconstructs` re-verifies k1 + k2·λ ≡ k (mod r_P) from scratch
//! for full-width scalars and edge cases, `endo_map_is_lambda` pins the φ↔λ
//! pairing on the real curve, and the ladder is tested byte-identical to the
//! group's own scalar multiplication.
//!
//! Like the `group::Wnaf`-based preparation this complements, this path is
//! variable-time with respect to the scalar. It is used only for trial
//! decryption with the wallet's own incoming viewing keys, matching the
//! existing `PreparedNonZeroScalar` usage.

use alloc::vec;
use alloc::vec::Vec;

use ff::{PrimeField, WithSmallOrderMulGroup};
use group::{prime::PrimeCurveAffine, Curve, Group};
use pasta_curves::arithmetic::CurveAffine;
use pasta_curves::pallas;

// GLV constants for the Pallas scalar field. Short lattice basis for
// {(a, b) : a + b·λ ≡ 0 (mod r_P)}:
//   v1 = (V1A, −V1B_NEG), v2 = (V2A, V2B)
// The `decompose_reconstructs` test proves these correct independently of
// their derivation: wrong constants cannot reconstruct k.
const V1A: u128 = 0x49e69d1640f049157fcae1c700000001;
const V1B_NEG: u128 = 0x49e69d1640a899538cb1279300000000;
const V2A: u128 = 0x49e69d1640a899538cb1279300000000;
const V2B: u128 = 0x93cd3a2c8198e2690c7c095a00000001;
// Babai rounding coefficients: g1 = round(2^384·v2.b / r_P),
// g2 = round(2^384·(−v1.b) / r_P).
const G1: [u64; 5] = [
    0x111f686111afc293,
    0xc35fbd4d086862e0,
    0x31f0256800000002,
    0x4f34e8b2066389a4,
    0x2,
];
const G2: [u64; 5] = [
    0x4a95a2d972171db4,
    0x61afdea68480fa55,
    0x32c49e4bffffffff,
    0x279a745902a2654e,
    0x1,
];

/// `round((g · k) / 2^384)` for a 5-limb `g` and 4-limb `k` — the Babai
/// coefficient. Fits `u128` (at most ~128 bits by construction).
fn round_mul_shift(g: &[u64; 5], k: &[u64; 4]) -> u128 {
    let mut prod = [0u64; 9];
    for (i, &gi) in g.iter().enumerate() {
        let mut carry = 0u128;
        for (j, &kj) in k.iter().enumerate() {
            let t = u128::from(gi) * u128::from(kj) + u128::from(prod[i + j]) + carry;
            prod[i + j] = t as u64;
            carry = t >> 64;
        }
        prod[i + 4] = prod[i + 4].wrapping_add(carry as u64);
    }
    // Bits >= 384 live in limbs 6..; round on bit 383 (top bit of limb 5).
    let round = prod[5] >> 63;
    (u128::from(prod[6]) | (u128::from(prod[7]) << 64)).wrapping_add(u128::from(round))
}

/// 256-bit product of two `u128`s, as little-endian limbs.
fn mul_u128(a: u128, b: u128) -> [u64; 4] {
    let (a0, a1) = (a as u64, (a >> 64) as u64);
    let (b0, b1) = (b as u64, (b >> 64) as u64);
    let mut out = [0u64; 4];
    let mut acc = |i: usize, v: u128| {
        let mut idx = i;
        let mut carry = v;
        while carry != 0 {
            let t = u128::from(out[idx]) + (carry & u128::from(u64::MAX));
            out[idx] = t as u64;
            carry = (carry >> 64) + (t >> 64);
            idx += 1;
        }
    };
    acc(0, u128::from(a0) * u128::from(b0));
    acc(1, u128::from(a0) * u128::from(b1));
    acc(1, u128::from(a1) * u128::from(b0));
    acc(2, u128::from(a1) * u128::from(b1));
    out
}

/// 256-bit wrapping subtraction (two's complement).
fn sub256(a: [u64; 4], b: [u64; 4]) -> [u64; 4] {
    let mut out = [0u64; 4];
    let mut borrow = 0u64;
    for i in 0..4 {
        let (d, b1) = a[i].overflowing_sub(b[i]);
        let (d, b2) = d.overflowing_sub(borrow);
        out[i] = d;
        borrow = u64::from(b1) + u64::from(b2);
    }
    out
}

/// Interprets a 256-bit two's-complement value with |x| < 2^128 as
/// (is_negative, |x|).
fn signed_halves(x: [u64; 4]) -> (bool, u128) {
    if x[3] >> 63 == 0 {
        debug_assert!(x[2] == 0 && x[3] == 0, "positive half exceeds 128 bits");
        (false, u128::from(x[0]) | (u128::from(x[1]) << 64))
    } else {
        // Negate: !x + 1.
        let mut n = [!x[0], !x[1], !x[2], !x[3]];
        let mut carry = 1u64;
        for limb in &mut n {
            let (v, c) = limb.overflowing_add(carry);
            *limb = v;
            carry = u64::from(c);
            if carry == 0 {
                break;
            }
        }
        debug_assert!(n[2] == 0 && n[3] == 0, "negative half exceeds 128 bits");
        (true, u128::from(n[0]) | (u128::from(n[1]) << 64))
    }
}

/// GLV split: `k = k1 + k2·λ (mod r_P)` with |k1|, |k2| ≤ 2^128, each half
/// returned as (is_negative, magnitude).
fn decompose(k: &pallas::Scalar) -> ((bool, u128), (bool, u128)) {
    let repr = k.to_repr();
    let bytes: &[u8; 32] = repr.as_ref().try_into().expect("32-byte repr");
    let mut kl = [0u64; 4];
    for (i, limb) in kl.iter_mut().enumerate() {
        *limb = u64::from_le_bytes(bytes[i * 8..(i + 1) * 8].try_into().expect("8 bytes"));
    }
    let c1 = round_mul_shift(&G1, &kl);
    let c2 = round_mul_shift(&G2, &kl);
    // k1 = k − c1·V1A − c2·V2A   (two's complement over 256 bits)
    let k1 = sub256(sub256(kl, mul_u128(c1, V1A)), mul_u128(c2, V2A));
    // k2 = c1·V1B_NEG − c2·V2B   (v1.b = −V1B_NEG, v2.b = +V2B)
    let k2 = sub256(mul_u128(c1, V1B_NEG), mul_u128(c2, V2B));
    (signed_halves(k1), signed_halves(k2))
}

/// φ(P) on affine coordinates: (ζ_p·x, y). The identity maps to the identity.
fn endo_affine(p: &pallas::Affine) -> pallas::Affine {
    let coords = p.coordinates();
    if bool::from(coords.is_none()) {
        return pallas::Affine::identity();
    }
    let c = coords.unwrap();
    pallas::Affine::from_xy(pallas::Base::ZETA * c.x(), *c.y()).unwrap()
}

/// The GLV window for one base point: the odd multiples {1, 3, 5, 7}·P and
/// {1, 3, 5, 7}·φ(P) in affine coordinates.
///
/// Built once per ephemeral key ([`endo_tables`] builds a whole batch with
/// one shared normalization) and reused by every viewing key's
/// multiplication against it. 512 bytes.
#[derive(Clone, Debug)]
pub(crate) struct EndoTable {
    /// {1, 3, 5, 7}·P
    t1: [pallas::Affine; 4],
    /// {1, 3, 5, 7}·φ(P)
    t2: [pallas::Affine; 4],
}

#[cfg(test)]
impl EndoTable {
    /// The base point P (= t1[0]) back as a projective point.
    pub(crate) fn point(&self) -> pallas::Point {
        pallas::Point::from(self.t1[0])
    }

    /// Builds the window for a single point (one 4-point normalization).
    pub(crate) fn from_point(p: &pallas::Point) -> Self {
        endo_tables(core::slice::from_ref(p))
            .pop()
            .expect("one table per input point")
    }
}

/// Builds [`EndoTable`]s for a batch of non-identity points with one shared
/// batch normalization across all 4·n odd multiples — a single field
/// inversion for the whole batch, where building each window individually
/// pays one inversion per point.
///
/// Uses projective group operations only (no hand-rolled affine formulas),
/// and on a prime-order curve the odd multiples of a non-identity point are
/// never the identity, so the normalized windows are always well-formed.
/// Callers guarantee non-identity inputs; ephemeral keys are non-identity by
/// construction.
pub(crate) fn endo_tables(points: &[pallas::Point]) -> Vec<EndoTable> {
    let n = points.len();
    if n == 0 {
        return Vec::new();
    }
    // Odd multiples per point, projective (cheap additions, no inversions),
    // interleaved [1·P₀, 3·P₀, 5·P₀, 7·P₀, 1·P₁, ...].
    let mut proj = Vec::with_capacity(n * 4);
    for p in points {
        debug_assert!(
            !bool::from(p.is_identity()),
            "endo_tables contract: non-identity points only"
        );
        let two_p = p.double();
        let mut m = *p;
        proj.push(m);
        for _ in 1..4 {
            m += two_p;
            proj.push(m);
        }
    }
    // One inversion for the whole batch.
    let mut affine = vec![pallas::Affine::identity(); n * 4];
    pallas::Point::batch_normalize(&proj, &mut affine);
    affine
        .chunks_exact(4)
        .map(|c| {
            let t1: [pallas::Affine; 4] = c.try_into().expect("chunks of 4");
            let t2 = [
                endo_affine(&t1[0]),
                endo_affine(&t1[1]),
                endo_affine(&t1[2]),
                endo_affine(&t1[3]),
            ];
            EndoTable { t1, t2 }
        })
        .collect()
}

/// A scalar in GLV-decomposed, wNAF-recoded form, ready for
/// [`mul_with_table`].
///
/// Batched trial decryption multiplies the same viewing key against every
/// ephemeral key in the batch; building this once per (viewing key, batch)
/// hoists the decomposition and digit recoding out of the per-output loop.
#[derive(Clone, Debug)]
pub(crate) struct DecomposedScalar {
    neg1: bool,
    digits1: [i8; 132],
    len1: usize,
    neg2: bool,
    digits2: [i8; 132],
    len2: usize,
}

impl DecomposedScalar {
    /// Decomposes `k` and recodes both halves as width-4 wNAF digits.
    pub(crate) fn new(k: &pallas::Scalar) -> Self {
        let ((neg1, a1), (neg2, a2)) = decompose(k);
        let (digits1, len1) = wnaf_digits(a1);
        let (digits2, len2) = wnaf_digits(a2);
        DecomposedScalar {
            neg1,
            digits1,
            len1,
            neg2,
            digits2,
            len2,
        }
    }
}

/// `k·P` for the P encoded by `table`, via the Straus shared-doubling ladder
/// over the GLV split. Byte-identical to `P * k` (tested).
pub(crate) fn mul_with_table(table: &EndoTable, k: &DecomposedScalar) -> pallas::Point {
    let len = k.len1.max(k.len2);
    let mut acc = pallas::Point::identity();
    for i in (0..len).rev() {
        acc = acc.double();
        let d = if i < k.len1 { k.digits1[i] } else { 0 };
        if d != 0 {
            let mut a = table.t1[(d.unsigned_abs() / 2) as usize];
            if (d < 0) ^ k.neg1 {
                a = -a;
            }
            acc += a;
        }
        let d = if i < k.len2 { k.digits2[i] } else { 0 };
        if d != 0 {
            let mut a = table.t2[(d.unsigned_abs() / 2) as usize];
            if (d < 0) ^ k.neg2 {
                a = -a;
            }
            acc += a;
        }
    }
    acc
}

/// `k·P` for the P encoded by `table`, decomposing `k` on the spot. Used by
/// the per-item multiplication against a batch-prepared ephemeral key; the
/// batched path decomposes once via [`DecomposedScalar`] instead.
pub(crate) fn mul(table: &EndoTable, k: &pallas::Scalar) -> pallas::Point {
    mul_with_table(table, &DecomposedScalar::new(k))
}

/// Width-4 wNAF digits of a u128 magnitude, lowest position first. A
/// magnitude of at most 2^127 yields at most 129 digits; the array is sized
/// with headroom.
fn wnaf_digits(a: u128) -> ([i8; 132], usize) {
    debug_assert!(a >> 127 == 0, "magnitude must be at most 127 bits");
    let mut digits = [0i8; 132];
    let mut n = 0;
    let mut k = a;
    while k != 0 {
        if k & 1 == 1 {
            let low = (k & 0xF) as i8;
            let d = if low >= 8 { low - 16 } else { low };
            digits[n] = d;
            if d >= 0 {
                k -= d as u128;
            } else {
                k += (-d) as u128;
            }
        }
        n += 1;
        k >>= 1;
    }
    (digits, n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff::Field;

    /// Deterministic full-width scalars for the known-answer tests.
    fn scalars(n: u64) -> impl Iterator<Item = pallas::Scalar> {
        (0..n).map(|i| {
            (pallas::Scalar::from(0x9E37_79B9_7F4A_7C15u64 + i).square()
                + pallas::Scalar::from(0x0123_4567_89AB_CDEFu64))
            .square()
                + pallas::Scalar::from(i)
        })
    }

    /// The φ↔λ pairing on the real curve: (ζ_p·x, y) == λ·P.
    #[test]
    fn endo_map_is_lambda() {
        let g = pallas::Point::generator();
        for k in scalars(64) {
            let p = (g * k).to_affine();
            let via_map = endo_affine(&p);
            let via_mul = (pallas::Point::from(p) * pallas::Scalar::ZETA).to_affine();
            assert_eq!(via_map, via_mul, "phi(P) must equal ZETA_scalar * P");
        }
    }

    /// The algebraic gate: k1 + k2·λ ≡ k (mod r_P) with both halves at most
    /// 2^128, for full-width scalars and the edge cases. Wrong GLV constants
    /// cannot pass this.
    #[test]
    fn decompose_reconstructs() {
        let edges = [
            pallas::Scalar::ZERO,
            pallas::Scalar::ONE,
            -pallas::Scalar::ONE,
            pallas::Scalar::ZETA,
            pallas::Scalar::ZETA.square(),
            pallas::Scalar::from(u64::MAX),
        ];
        for k in edges.into_iter().chain(scalars(2_000)) {
            let ((s1, a1), (s2, a2)) = decompose(&k);
            assert!(
                a1 >> 127 == 0 && a2 >> 127 == 0,
                "halves must be at most 127 bits"
            );
            let half = |s: bool, a: u128| {
                let v = pallas::Scalar::from_u128(a);
                if s {
                    -v
                } else {
                    v
                }
            };
            let rec = half(s1, a1) + half(s2, a2) * pallas::Scalar::ZETA;
            assert_eq!(rec, k, "k1 + k2*lambda must reconstruct k");
        }
    }

    /// The ladder over batch-built windows equals the curve's own scalar
    /// multiplication — proves `endo_tables`' odd multiples (φ side
    /// included) and the digit walk together.
    #[test]
    fn tabled_mul_matches_group_mul() {
        let g = pallas::Point::generator();
        let points: Vec<pallas::Point> = (0..24u64)
            .map(|i| g * pallas::Scalar::from(31 + i))
            .collect();
        let tables = endo_tables(&points);
        assert_eq!(tables.len(), points.len());
        for (i, (p, t)) in points.iter().zip(&tables).enumerate() {
            assert_eq!(t.point(), *p, "table {i} must round-trip its base point");
            for k in scalars(12) {
                assert_eq!(mul(t, &k), p * k, "table {i}");
            }
            // Edges per table: 0, ±1, λ.
            assert_eq!(
                mul(t, &pallas::Scalar::ZERO),
                pallas::Point::identity(),
                "table {i}"
            );
            assert_eq!(mul(t, &pallas::Scalar::ONE), *p, "table {i}");
            assert_eq!(mul(t, &-pallas::Scalar::ONE), -p, "table {i}");
            assert_eq!(
                mul(t, &pallas::Scalar::ZETA),
                p * pallas::Scalar::ZETA,
                "table {i}"
            );
        }
    }

    /// Batch-built windows equal individually-built windows: the shared
    /// normalization changes the arithmetic route, not the values.
    #[test]
    fn batch_tables_equal_solo_tables() {
        let g = pallas::Point::generator();
        let points: Vec<pallas::Point> = (0..32u64)
            .map(|i| g * pallas::Scalar::from(97 + i))
            .collect();
        for (p, batch) in points.iter().zip(endo_tables(&points)) {
            let solo = EndoTable::from_point(p);
            assert_eq!(batch.t1, solo.t1);
            assert_eq!(batch.t2, solo.t2);
        }
    }

    /// A precomputed [`DecomposedScalar`] and on-the-spot decomposition run
    /// the same ladder.
    #[test]
    fn decomposed_scalar_reuse_matches_fresh() {
        let g = pallas::Point::generator();
        let points: Vec<pallas::Point> = (0..8u64)
            .map(|i| g * pallas::Scalar::from(211 + i))
            .collect();
        let tables = endo_tables(&points);
        for k in scalars(16) {
            let decomposed = DecomposedScalar::new(&k);
            for (p, t) in points.iter().zip(&tables) {
                assert_eq!(mul_with_table(t, &decomposed), p * k);
                assert_eq!(mul(t, &k), p * k);
            }
        }
    }

    /// Full-width KAT sweep: byte-identical to the curve's own scalar
    /// multiplication across many (point, scalar) pairs.
    #[test]
    fn mul_matches_group_mul() {
        let g = pallas::Point::generator();
        let n = if cfg!(debug_assertions) { 500 } else { 4_000 };
        for (i, k) in scalars(n).enumerate() {
            let base = g * pallas::Scalar::from(31 + i as u64);
            let t = EndoTable::from_point(&base);
            assert_eq!(mul(&t, &k), base * k, "lane {i}");
        }
    }
}
