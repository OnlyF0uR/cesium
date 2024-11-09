use std::io::Read;

use sha3::{
    digest::{ExtendableOutput, Update},
    Shake256,
};

use crate::keys::Account;

use super::{errors::ZkError, Challenge, Commitment, Response, CHALLENGE_LENGTH};

/// Functions for the Verifier role
pub struct VerifierProtocol;

impl VerifierProtocol {
    /// Generate a challenge using SHAKE256
    pub fn generate_challenge(commitment: &Commitment) -> Challenge {
        let mut hasher = Shake256::default();
        hasher.update(&commitment.0);

        let mut challenge = vec![0u8; CHALLENGE_LENGTH];
        let mut xof = hasher.finalize_xof();
        let _ = xof.read(&mut challenge);

        Challenge(challenge)
    }

    /// Verify a proof using SPHINCS+ signature verification
    pub fn verify(
        account: &Account,
        commitment: &Commitment,
        challenge: &Challenge,
        response: &Response,
    ) -> Result<bool, ZkError> {
        // Reconstruct message that was signed
        let mut message = Vec::new();
        message.extend_from_slice(&commitment.0);
        message.extend_from_slice(&challenge.0);

        // Convert response back to DetachedSignature
        match account.verify(&message, &response.0) {
            Ok(b) => Ok(b),
            Err(e) => {
                // TODO: This will chance when proper error handling for
                // the rest of the crypto library is implemented
                if e.to_string().contains("verification failed") {
                    return Ok(false);
                } else {
                    return Err(ZkError::VerificationError(e.to_string()));
                }
            }
        }
    }
}
