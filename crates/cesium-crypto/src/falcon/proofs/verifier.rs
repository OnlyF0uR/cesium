use std::io::Read;

use sha3::{
    digest::{ExtendableOutput, Update},
    Shake256,
};

use crate::{
    errors::CryptoError,
    falcon::keypair::{SignerPair, ViewOperations},
};

use super::{Challenge, Commitment, Response, CHALLENGE_LENGTH};

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

    /// Verify a proof using Falcon (fndsa512) signature verification
    pub fn verify(
        account: &SignerPair,
        commitment: &Commitment,
        challenge: &Challenge,
        response: &Response,
    ) -> Result<bool, CryptoError> {
        // Reconstruct message that was signed
        let mut message = Vec::new();
        message.extend_from_slice(&commitment.0);
        message.extend_from_slice(&challenge.0);

        // Convert response back to DetachedSignature
        account.verify(&message, &response.0)
    }

    pub fn verify_non_interactive(
        account: &SignerPair,
        commitment: &Commitment,
        response: &Response,
    ) -> Result<bool, CryptoError> {
        // Deterministically recreate the challenge from the commitment
        let mut hasher = Shake256::default();
        hasher.update(&commitment.0);

        let mut challenge = vec![0u8; CHALLENGE_LENGTH];
        let mut xof = hasher.finalize_xof();
        let _ = xof.read(&mut challenge);

        // Reconstruct the message that was signed
        let mut message = Vec::new();
        message.extend_from_slice(&commitment.0);
        message.extend_from_slice(&challenge);

        // Verify the response signature
        account.verify(&message, &response.0)
    }
}
