const SALT_LENGTH: usize = 32;
const CHALLENGE_LENGTH: usize = 32;

#[derive(Clone, Debug)]
pub struct Commitment(pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct Challenge(pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct Response(pub Vec<u8>);

pub mod errors;
pub mod prover;
pub mod verifier;

#[cfg(test)]
mod tests {
    use errors::ZkError;
    use prover::ProverProtocol;
    use verifier::VerifierProtocol;

    use crate::keys::Account;

    use super::*;

    #[test]
    fn test_valid_proof() -> Result<(), ZkError> {
        let account = Account::create();

        // Create a secret value
        let secret = b"my secret value";
        // Prover generates commitment
        let (commitment, _salt) = ProverProtocol::generate_commitment(secret)?;

        // Verifier generates challenge
        let challenge = VerifierProtocol::generate_challenge(&commitment);

        // Prover generates response
        let response = ProverProtocol::generate_response(&account, &commitment, &challenge)?;

        // Verifier verifies the proof
        let is_valid = VerifierProtocol::verify(&account, &commitment, &challenge, &response)?;

        assert!(is_valid);
        Ok(())
    }

    #[test]
    fn test_invalid_account() -> Result<(), ZkError> {
        let account = Account::create();
        let wrong_account = Account::create(); // Generate different account

        // Create a secret value
        let secret = b"my secret value";
        // Prover generates commitment
        let (commitment, _salt) = ProverProtocol::generate_commitment(secret)?;

        // Verifier generates challenge
        let challenge = VerifierProtocol::generate_challenge(&commitment);

        // Prover generates response with wrong secret key
        let response = ProverProtocol::generate_response(&account, &commitment, &challenge)?;

        // Verifier verifies the proof
        let is_valid =
            VerifierProtocol::verify(&wrong_account, &commitment, &challenge, &response)?;

        assert!(!is_valid);
        Ok(())
    }

    #[test]
    fn test_invalid_proof() -> Result<(), ZkError> {
        let account = Account::create();

        // Create a secret value
        let secret = b"my secret value";
        // Prover generates commitment
        let (commitment, _salt) = ProverProtocol::generate_commitment(secret)?;

        // Verifier generates challenge
        let challenge = VerifierProtocol::generate_challenge(&commitment);

        // Prover generates response
        let response = ProverProtocol::generate_response(&account, &commitment, &challenge)?;

        // Verifier verifies the proof with wrong commitment
        let wrong_commitment = Commitment(vec![0; CHALLENGE_LENGTH]);
        let is_valid =
            VerifierProtocol::verify(&account, &wrong_commitment, &challenge, &response)?;

        assert!(!is_valid);
        Ok(())
    }

    #[test]
    fn test_commitment_uniqueness() {
        let secret = b"my secret value";
        let (commitment1, _) = ProverProtocol::generate_commitment(secret).unwrap();
        let (commitment2, _) = ProverProtocol::generate_commitment(secret).unwrap();

        // Commitments should be different due to random salt
        assert_ne!(commitment1.0, commitment2.0);
    }
}
