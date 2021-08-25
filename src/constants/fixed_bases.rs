//! Orchard fixed bases.
use super::{L_ORCHARD_SCALAR, L_VALUE};
use ecc::{chip::compute_lagrange_coeffs, gadget::FixedPoints};

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

#[cfg(feature = "test-ecc")]
#[test]
fn test_orchard_fixed_bases() {
    use ecc::gadget::testing;
    use halo2::dev::MockProver;

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
