use std::io::Read;

use rand::{rngs::OsRng, Rng};
use sha3::{
    digest::{ExtendableOutput, Update},
    Shake256,
};

use crate::keys::Account;

use super::{errors::ZkError, Challenge, Commitment, Response, CHALLENGE_LENGTH, SALT_LENGTH};

/// Functions for the Prover role
pub struct ProverProtocol;

impl ProverProtocol {
    /// Generate a commitment to a secret
    pub fn generate_commitment(secret: &[u8]) -> Result<(Commitment, Vec<u8>), ZkError> {
        let mut rng = OsRng;
        let mut salt = vec![0u8; SALT_LENGTH];
        rng.fill(&mut salt[..]);

        let mut hasher = Shake256::default();
        hasher.update(secret);
        hasher.update(&salt);

        let mut commitment = vec![0u8; CHALLENGE_LENGTH];
        let mut xof = hasher.finalize_xof();
        let _ = xof.read(&mut commitment);

        Ok((Commitment(commitment), salt))
    }

    /// Generate a response using SPHINCS+ signature
    pub fn generate_response(
        account: &Account,
        commitment: &Commitment,
        challenge: &Challenge,
    ) -> Result<Response, ZkError> {
        // Create message to sign by concatenating commitment and challenge
        let mut message = Vec::new();
        message.extend_from_slice(&commitment.0);
        message.extend_from_slice(&challenge.0);

        // Sign the message using SPHINCS+
        let signature = account
            .digest(&message)
            .map_err(|e| ZkError::SigningError(e.to_string()))?;

        Ok(Response(signature))
    }
}
