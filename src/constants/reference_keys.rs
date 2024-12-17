//! Orchard reference keys, including the spending key and recipient address, used for reference notes.
//!
//! The reference SpendingKey is a placeholder key with all bytes set to zero.
//! Using this SpendingKey, we derive the FullViewingKey, and the recipient address.
//! To avoid repeating the derivation process whenever the recipient address is required, we store
//! its raw encoding.

use crate::{
    address::Address,
    keys::{FullViewingKey, SpendingKey},
};

/// Raw bytes representation of the reference recipient address.
pub const RAW_REFERENCE_RECIPIENT: [u8; 43] = [
    204, 54, 96, 25, 89, 33, 59, 107, 12, 219, 150, 167, 92, 23, 195, 166, 104, 169, 127, 13, 106,
    140, 92, 225, 100, 165, 24, 234, 155, 169, 165, 14, 167, 81, 145, 253, 134, 27, 15, 241, 14,
    98, 176,
];

/// Reference keys (spending key and recipient address) are used for reference notes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ReferenceKeys;

impl ReferenceKeys {
    /// Returns the spending key for reference notes.
    pub fn sk() -> SpendingKey {
        SpendingKey::from_bytes([0; 32]).unwrap()
    }

    /// Returns the recipient address for reference notes.
    pub fn recipient() -> Address {
        Address::from_raw_address_bytes(&RAW_REFERENCE_RECIPIENT).unwrap()
    }

    /// Returns the full viewing key for reference notes.
    pub fn fvk() -> FullViewingKey {
        FullViewingKey::from(&Self::sk())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{FullViewingKey, Scope};

    #[test]
    fn recipient() {
        let sk = SpendingKey::from_bytes([0; 32]).unwrap();
        let fvk = FullViewingKey::from(&sk);
        let recipient = fvk.address_at(0u32, Scope::External);

        assert_eq!(recipient, ReferenceKeys::recipient());
    }
}
