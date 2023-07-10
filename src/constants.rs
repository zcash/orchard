//! Constants used in the Orchard protocol.
pub mod fixed_bases;
pub mod sinsemilla;
pub mod util;

pub use fixed_bases::{NullifierK, OrchardFixedBases, OrchardFixedBasesFull, ValueCommitV, H};
pub use sinsemilla::{OrchardCommitDomains, OrchardHashDomains};

/// $\mathsf{MerkleDepth^{Orchard}}$
pub(crate) const MERKLE_DEPTH_ORCHARD: usize = 32;

/// The Pallas scalar field modulus is $q = 2^{254} + \mathsf{t_q}$.
/// <https://github.com/zcash/pasta>
pub(crate) const T_Q: u128 = 45560315531506369815346746415080538113;

/// The Pallas base field modulus is $p = 2^{254} + \mathsf{t_p}$.
/// <https://github.com/zcash/pasta>
pub(crate) const T_P: u128 = 45560315531419706090280762371685220353;

/// $\ell^\mathsf{Orchard}_\mathsf{base}$
pub(crate) const L_ORCHARD_BASE: usize = 255;

/// $\ell^\mathsf{Orchard}_\mathsf{scalar}$
pub(crate) const L_ORCHARD_SCALAR: usize = 255;

/// $\ell_\mathsf{value}$
pub(crate) const L_VALUE: usize = 64;

/// SWU hash-to-curve personalization for the group hash for key diversification
pub const KEY_DIVERSIFICATION_PERSONALIZATION: &str = "z.cash:Orchard-gd";

/// First 64 bytes of the BLAKE2s input during group hash.
/// This is chosen to be some random string that we couldn't have anticipated when we designed
/// the algorithm, for rigidity purposes.
/// We deliberately use an ASCII hex string of 32 bytes here.
pub const GH_FIRST_BLOCK: &[u8; 64] =
    b"096b36a5804bfacef1691e173c366a47ff5ba84a44f26ddd7e8d9f79d5b42df0";

/// Length in bytes of the asset identifier
pub const ASSET_IDENTIFIER_LENGTH: usize = 32;

/// BLAKE2s Personalization for deriving asset identifier from asset name
pub const ASSET_IDENTIFIER_PERSONALIZATION: &[u8; 8] = b"MASP__t_";

/// BLAKE2s Personalization for the value commitment generator for the value
pub const VALUE_COMMITMENT_GENERATOR_PERSONALIZATION: &[u8; 8] = b"MASP__v_";

#[cfg(test)]
mod tests {
    use ff::PrimeField;
    use pasta_curves::pallas;

    #[test]
    // Orchard uses the Pallas base field as its base field.
    fn l_orchard_base() {
        assert_eq!(super::L_ORCHARD_BASE, pallas::Base::NUM_BITS as usize);
    }

    #[test]
    // Orchard uses the Pallas base field as its base field.
    fn l_orchard_scalar() {
        assert_eq!(super::L_ORCHARD_SCALAR, pallas::Scalar::NUM_BITS as usize);
    }

    #[test]
    fn t_q() {
        let t_q = pallas::Scalar::from_u128(super::T_Q);
        let two_pow_254 = pallas::Scalar::from_u128(1 << 127).square();
        assert_eq!(t_q + two_pow_254, pallas::Scalar::zero());
    }

    #[test]
    fn t_p() {
        let t_p = pallas::Base::from_u128(super::T_P);
        let two_pow_254 = pallas::Base::from_u128(1 << 127).square();
        assert_eq!(t_p + two_pow_254, pallas::Base::zero());
    }
}
