use std::io::Read;

use rand::{rngs::OsRng, Rng};
use sha3::{
    digest::{ExtendableOutput, Update},
    Shake256,
};

use crate::{errors::CryptoError, sphincsplus::keypair::SignerPair};

use super::{Challenge, Commitment, Response, CHALLENGE_LENGTH, SALT_LENGTH};

/// Functions for the Prover role
pub struct ProverProtocol;

impl ProverProtocol {
    /// Generate a commitment to a secret
    pub fn generate_commitment(secret: &[u8]) -> Result<(Commitment, Vec<u8>), CryptoError> {
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
        account: &SignerPair,
        commitment: &Commitment,
        challenge: &Challenge,
    ) -> Result<Response, CryptoError> {
        // Create message to sign by concatenating commitment and challenge
        let mut message = Vec::new();
        message.extend_from_slice(&commitment.0);
        message.extend_from_slice(&challenge.0);

        // Sign the message using SPHINCS+
        let signature = account.sign(&message);

        Ok(Response(signature))
    }

    /// Generate commitment and response non-interactive
    pub fn generate_non_interactive(
        account: &SignerPair,
        secret: &[u8],
    ) -> Result<(Commitment, Response), CryptoError> {
        let mut rng = OsRng;
        let mut salt = vec![0u8; SALT_LENGTH];
        rng.fill(&mut salt[..]);

        let mut hasher = Shake256::default();
        hasher.update(secret);
        hasher.update(&salt);

        let mut commitment = vec![0u8; CHALLENGE_LENGTH];
        let mut xof = hasher.finalize_xof();
        let _ = xof.read(&mut commitment);

        // Use Fiat-Shamir transform to generate a challenge from commitment
        let mut challenge_hasher = Shake256::default();
        challenge_hasher.update(&commitment);

        let mut challenge = vec![0u8; CHALLENGE_LENGTH];
        let mut xof_challenge = challenge_hasher.finalize_xof();
        let _ = xof_challenge.read(&mut challenge);

        // Create a response by signing the commitment and challenge
        let mut message = Vec::new();
        message.extend_from_slice(&commitment);
        message.extend_from_slice(&challenge);

        let signature = account.sign(&message);

        Ok((Commitment(commitment), Response(signature)))
    }
}
