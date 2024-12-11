use rand::{CryptoRng, RngCore};

use crate::{keys::SpendAuthorizingKey, primitives::redpallas};

impl super::Action {
    /// Signs the Orchard spend with the given spend authorizing key.
    ///
    /// It is the caller's responsibility to perform any semantic validity checks on the
    /// PCZT (for example, comfirming that the change amounts are correct) before calling
    /// this method.
    pub fn sign<R: RngCore + CryptoRng>(
        &mut self,
        sighash: [u8; 32],
        ask: &SpendAuthorizingKey,
        rng: R,
    ) -> Result<(), SignerError> {
        let alpha = self
            .spend
            .alpha
            .ok_or(SignerError::MissingSpendAuthRandomizer)?;

        let rsk = ask.randomize(&alpha);
        let rk = redpallas::VerificationKey::from(&rsk);

        if self.spend.rk == rk {
            self.spend.spend_auth_sig = Some(rsk.sign(rng, &sighash));
            Ok(())
        } else {
            Err(SignerError::WrongSpendAuthorizingKey)
        }
    }
}

/// Errors that can occur while signing an Orchard action in a PCZT.
#[derive(Debug)]
pub enum SignerError {
    /// The Signer role requires `alpha` to be set.
    MissingSpendAuthRandomizer,
    /// The provided `ask` does not own the action's spent note.
    WrongSpendAuthorizingKey,
}
