//! Orchard fixed bases.
use super::{L_ORCHARD_SCALAR, L_VALUE};
use halo2_gadgets::ecc::{chip::compute_lagrange_coeffs, FixedPoints};

use pasta_curves::pallas;

pub mod commit_ivk_r;
pub mod note_commit_r;
pub mod nullifier_k;
pub mod spend_auth_g;
pub mod value_commit_r;
pub mod value_commit_v;

/// SWU hash-to-curve personalization for the spending key base point and
/// the nullifier base point K^Orchard
pub const ORCHARD_PERSONALIZATION: &str = "z.cash:Orchard";

/// SWU hash-to-curve personalization for the value commitment generator
pub const VALUE_COMMITMENT_PERSONALIZATION: &str = "z.cash:Orchard-cv";

/// SWU hash-to-curve value for the value commitment generator
pub const VALUE_COMMITMENT_V_BYTES: [u8; 1] = *b"v";

/// SWU hash-to-curve value for the value commitment generator
pub const VALUE_COMMITMENT_R_BYTES: [u8; 1] = *b"r";

/// SWU hash-to-curve personalization for the note commitment generator
pub const NOTE_COMMITMENT_PERSONALIZATION: &str = "z.cash:Orchard-NoteCommit";

/// SWU hash-to-curve personalization for the IVK commitment generator
pub const COMMIT_IVK_PERSONALIZATION: &str = "z.cash:Orchard-CommitIvk";

/// Window size for fixed-base scalar multiplication
pub const FIXED_BASE_WINDOW_SIZE: usize = 3;

/// $2^{`FIXED_BASE_WINDOW_SIZE`}$
pub const H: usize = 1 << FIXED_BASE_WINDOW_SIZE;

/// Number of windows for a full-width scalar
pub const NUM_WINDOWS: usize =
    (L_ORCHARD_SCALAR + FIXED_BASE_WINDOW_SIZE - 1) / FIXED_BASE_WINDOW_SIZE;

/// Number of windows for a short signed scalar
pub const NUM_WINDOWS_SHORT: usize =
    (L_VALUE + FIXED_BASE_WINDOW_SIZE - 1) / FIXED_BASE_WINDOW_SIZE;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// The fixed bases used in the Orchard protocol.
pub enum OrchardFixedBases {
    /// The random base used in CommitIvk. This is multiplied by a full-width
    /// scalar.
    CommitIvkR,
    /// The random base used in NoteCommit. This is multiplied by a full-width
    /// scalar.
    NoteCommitR,
    /// The base used to multiply the trapdoor in ValueCommit. This is multiplied
    /// by a full-width scalar.
    ValueCommitR,
    /// The base used in SpendAuthSig. This is multiplied by a full-width
    /// scalar.
    SpendAuthG,
    /// The base used in DeriveNullifier. This is multiplied by a base-field
    /// element.
    NullifierK,
    /// The base used to multiply the value in ValueCommit. This is multiplied
    /// by a signed 64-bit integer.
    ValueCommitV,
}

impl FixedPoints<pallas::Affine> for OrchardFixedBases {
    fn generator(&self) -> pallas::Affine {
        match self {
            OrchardFixedBases::CommitIvkR => commit_ivk_r::generator(),
            OrchardFixedBases::NoteCommitR => note_commit_r::generator(),
            OrchardFixedBases::ValueCommitR => value_commit_r::generator(),
            OrchardFixedBases::SpendAuthG => spend_auth_g::generator(),
            OrchardFixedBases::NullifierK => nullifier_k::generator(),
            OrchardFixedBases::ValueCommitV => value_commit_v::generator(),
        }
    }

    fn u(&self) -> Vec<[[u8; 32]; H]> {
        match self {
            OrchardFixedBases::CommitIvkR => commit_ivk_r::U.to_vec(),
            OrchardFixedBases::NoteCommitR => note_commit_r::U.to_vec(),
            OrchardFixedBases::ValueCommitR => value_commit_r::U.to_vec(),
            OrchardFixedBases::SpendAuthG => spend_auth_g::U.to_vec(),
            OrchardFixedBases::NullifierK => nullifier_k::U.to_vec(),
            OrchardFixedBases::ValueCommitV => value_commit_v::U_SHORT.to_vec(),
        }
    }

    fn z(&self) -> Vec<u64> {
        match self {
            OrchardFixedBases::CommitIvkR => commit_ivk_r::Z.to_vec(),
            OrchardFixedBases::NoteCommitR => note_commit_r::Z.to_vec(),
            OrchardFixedBases::ValueCommitR => value_commit_r::Z.to_vec(),
            OrchardFixedBases::SpendAuthG => spend_auth_g::Z.to_vec(),
            OrchardFixedBases::NullifierK => nullifier_k::Z.to_vec(),
            OrchardFixedBases::ValueCommitV => value_commit_v::Z_SHORT.to_vec(),
        }
    }

    fn lagrange_coeffs(&self) -> Vec<[pallas::Base; H]> {
        match self {
            OrchardFixedBases::ValueCommitV => {
                compute_lagrange_coeffs(self.generator(), NUM_WINDOWS_SHORT)
            }
            _ => compute_lagrange_coeffs(self.generator(), NUM_WINDOWS),
        }
    }
}

#[cfg(test)]
use pasta_curves::arithmetic::{CurveAffine, FieldExt};

#[cfg(test)]
/// Test that Lagrange interpolation coefficients reproduce the correct x-coordinate
/// for each fixed-base multiple in each window.
fn test_lagrange_coeffs<C: CurveAffine>(base: C, num_windows: usize) {
    use ff::Field;
    use group::Curve;
    use halo2_gadgets::ecc::FIXED_BASE_WINDOW_SIZE;

    fn evaluate<C: CurveAffine>(x: u8, coeffs: &[C::Base]) -> C::Base {
        let x = C::Base::from_u64(x as u64);
        coeffs
            .iter()
            .rev()
            .cloned()
            .reduce(|acc, coeff| acc * x + coeff)
            .unwrap_or_else(C::Base::zero)
    }

    let lagrange_coeffs = compute_lagrange_coeffs(base, num_windows);

    // Check first 84 windows, i.e. `k_0, k_1, ..., k_83`
    for (idx, coeffs) in lagrange_coeffs[0..(num_windows - 1)].iter().enumerate() {
        // Test each three-bit chunk in this window.
        for bits in 0..(1 << FIXED_BASE_WINDOW_SIZE) {
            {
                // Interpolate the x-coordinate using this window's coefficients
                let interpolated_x = evaluate::<C>(bits, coeffs);

                // Compute the actual x-coordinate of the multiple [(k+2)*(8^w)]B.
                let point = base
                    * C::Scalar::from_u64(bits as u64 + 2)
                    * C::Scalar::from_u64(H as u64).pow(&[idx as u64, 0, 0, 0]);
                let x = *point.to_affine().coordinates().unwrap().x();

                // Check that the interpolated x-coordinate matches the actual one.
                assert_eq!(x, interpolated_x);
            }
        }
    }

    // Check last window.
    for bits in 0..(1 << FIXED_BASE_WINDOW_SIZE) {
        // Interpolate the x-coordinate using the last window's coefficients
        let interpolated_x = evaluate::<C>(bits, &lagrange_coeffs[num_windows - 1]);

        // Compute the actual x-coordinate of the multiple [k * (8^84) - offset]B,
        // where offset = \sum_{j = 0}^{83} 2^{3j+1}
        let offset = (0..(num_windows - 1)).fold(C::Scalar::zero(), |acc, w| {
            acc + C::Scalar::from_u64(2).pow(&[
                FIXED_BASE_WINDOW_SIZE as u64 * w as u64 + 1,
                0,
                0,
                0,
            ])
        });
        let scalar = C::Scalar::from_u64(bits as u64)
            * C::Scalar::from_u64(H as u64).pow(&[(num_windows - 1) as u64, 0, 0, 0])
            - offset;
        let point = base * scalar;
        let x = *point.to_affine().coordinates().unwrap().x();

        // Check that the interpolated x-coordinate matches the actual one.
        assert_eq!(x, interpolated_x);
    }
}

#[cfg(test)]
// Test that the z-values and u-values satisfy the conditions:
//      1. z + y = u^2,
//      2. z - y is not a square
// for the y-coordinate of each fixed-base multiple in each window.
fn test_zs_and_us<C: CurveAffine>(base: C, z: &[u64], u: &[[[u8; 32]; H]], num_windows: usize) {
    use ff::Field;
    use halo2_gadgets::ecc::chip::compute_window_table;

    let window_table = compute_window_table(base, num_windows);

    for ((u, z), window_points) in u.iter().zip(z.iter()).zip(window_table) {
        for (u, point) in u.iter().zip(window_points.iter()) {
            let y = *point.coordinates().unwrap().y();
            let u = C::Base::from_bytes(u).unwrap();
            assert_eq!(C::Base::from_u64(*z) + y, u * u); // allow either square root
            assert!(bool::from((C::Base::from_u64(*z) - y).sqrt().is_none()));
        }
    }
}

#[cfg(feature = "gadget-tests")]
#[test]
fn test_orchard_fixed_bases() {
    use halo2::dev::MockProver;
    use halo2_gadgets::ecc::testing;

    struct OrchardTest;
    impl testing::EccTest<OrchardFixedBases> for OrchardTest {
        fn fixed_bases_full() -> Vec<OrchardFixedBases> {
            vec![
                OrchardFixedBases::CommitIvkR,
                OrchardFixedBases::NoteCommitR,
                OrchardFixedBases::SpendAuthG,
                OrchardFixedBases::ValueCommitR,
            ]
        }
        fn fixed_bases_short() -> Vec<OrchardFixedBases> {
            vec![OrchardFixedBases::ValueCommitV]
        }
        fn fixed_bases_base_field() -> Vec<OrchardFixedBases> {
            vec![OrchardFixedBases::NullifierK]
        }
    }

    let k = 13;
    let circuit = testing::MyCircuit::<OrchardTest, OrchardFixedBases>(std::marker::PhantomData);
    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    assert_eq!(prover.verify(), Ok(()))
}
