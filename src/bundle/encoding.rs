//! Read-only views of the byte encodings of actions and bundles.
//!
//! Each trait is implemented by the point type ([`Action`], [`Bundle`]) and its compressed
//! counterpart ([`ActionBytes`], [`BundleBytes`]), so code that only needs encodings
//! (serialization, ZIP-244 digests) is written once for both.

use nonempty::NonEmpty;

use crate::{
    action::{Action, ActionBytes},
    bundle::{Authorization, Bundle, BundleBytes, BundleVersion, Flags},
    note::{ExtractedNoteCommitment, Nullifier, TransmittedNoteCiphertext},
    tree::Anchor,
};

/// The byte encodings of an Action description.
pub trait ActionEncoding {
    /// The spend authorization this action carries.
    type SpendAuth;
    /// The nullifier of the note being spent.
    fn nullifier(&self) -> &Nullifier;
    /// The x-coordinate of the commitment to the output note.
    fn cmx(&self) -> &ExtractedNoteCommitment;
    /// The encrypted output note (`epk` as encoded, plus the ciphertexts).
    fn encrypted_note(&self) -> &TransmittedNoteCiphertext;
    /// `cv_net`, as encoded.
    fn cv_net_bytes(&self) -> [u8; 32];
    /// `rk`, as encoded.
    fn rk_bytes(&self) -> [u8; 32];
    /// The spend authorization.
    fn authorization(&self) -> &Self::SpendAuth;
}

impl<A> ActionEncoding for Action<A> {
    type SpendAuth = A;
    fn nullifier(&self) -> &Nullifier {
        Action::nullifier(self)
    }
    fn cmx(&self) -> &ExtractedNoteCommitment {
        Action::cmx(self)
    }
    fn encrypted_note(&self) -> &TransmittedNoteCiphertext {
        Action::encrypted_note(self)
    }
    fn cv_net_bytes(&self) -> [u8; 32] {
        self.cv_net().to_bytes()
    }
    fn rk_bytes(&self) -> [u8; 32] {
        self.rk().into()
    }
    fn authorization(&self) -> &A {
        Action::authorization(self)
    }
}

impl<A> ActionEncoding for ActionBytes<A> {
    type SpendAuth = A;
    fn nullifier(&self) -> &Nullifier {
        ActionBytes::nullifier(self)
    }
    fn cmx(&self) -> &ExtractedNoteCommitment {
        ActionBytes::cmx(self)
    }
    fn encrypted_note(&self) -> &TransmittedNoteCiphertext {
        ActionBytes::encrypted_note(self)
    }
    fn cv_net_bytes(&self) -> [u8; 32] {
        self.cv_net().to_bytes()
    }
    fn rk_bytes(&self) -> [u8; 32] {
        self.rk().to_bytes()
    }
    fn authorization(&self) -> &A {
        ActionBytes::authorization(self)
    }
}

/// The byte encodings of an Orchard-protocol bundle.
pub trait BundleEncoding<T: Authorization, V> {
    /// This tier's Action description.
    type Action: ActionEncoding<SpendAuth = T::SpendAuth>;
    /// The bundle's actions.
    fn actions(&self) -> &NonEmpty<Self::Action>;
    /// The bundle's flags.
    fn flags(&self) -> &Flags;
    /// The flags as encoded under [`BundleEncoding::bundle_version`].
    fn flag_byte(&self) -> u8;
    /// The net value moved out of the pool.
    fn value_balance(&self) -> &V;
    /// The root of the commitment tree the actions prove membership in.
    fn anchor(&self) -> &Anchor;
    /// The bundle's pool and protocol version.
    fn bundle_version(&self) -> BundleVersion;
    /// The authorizing data.
    fn authorization(&self) -> &T;
}

impl<T: Authorization, V> BundleEncoding<T, V> for Bundle<T, V> {
    type Action = Action<T::SpendAuth>;
    fn actions(&self) -> &NonEmpty<Self::Action> {
        Bundle::actions(self)
    }
    fn flags(&self) -> &Flags {
        Bundle::flags(self)
    }
    fn flag_byte(&self) -> u8 {
        Bundle::flag_byte(self)
    }
    fn value_balance(&self) -> &V {
        Bundle::value_balance(self)
    }
    fn anchor(&self) -> &Anchor {
        Bundle::anchor(self)
    }
    fn bundle_version(&self) -> BundleVersion {
        Bundle::bundle_version(self)
    }
    fn authorization(&self) -> &T {
        Bundle::authorization(self)
    }
}

impl<T: Authorization, V> BundleEncoding<T, V> for BundleBytes<T, V> {
    type Action = ActionBytes<T::SpendAuth>;
    fn actions(&self) -> &NonEmpty<Self::Action> {
        BundleBytes::actions(self)
    }
    fn flags(&self) -> &Flags {
        BundleBytes::flags(self)
    }
    fn flag_byte(&self) -> u8 {
        BundleBytes::flag_byte(self)
    }
    fn value_balance(&self) -> &V {
        BundleBytes::value_balance(self)
    }
    fn anchor(&self) -> &Anchor {
        BundleBytes::anchor(self)
    }
    fn bundle_version(&self) -> BundleVersion {
        BundleBytes::bundle_version(self)
    }
    fn authorization(&self) -> &T {
        BundleBytes::authorization(self)
    }
}
