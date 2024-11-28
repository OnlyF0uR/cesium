const SALT_LENGTH: usize = 32;
const CHALLENGE_LENGTH: usize = 32;

#[derive(Clone, Debug)]
pub struct Commitment(pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct Challenge(pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct Response(pub Vec<u8>);

pub mod prover;
pub mod verifier;

#[cfg(test)]
mod tests {
    use prover::ProverProtocol;
    use verifier::VerifierProtocol;

    use crate::mldsa::keypair::SignerPair;

    use super::*;

    #[test]
    fn test_valid_proof() {
        let account = SignerPair::create();

        // Create a secret value
        let secret = b"my secret value";
        // Prover generates commitment
        let (commitment, _salt) = ProverProtocol::generate_commitment(secret).unwrap();

        // Verifier generates challenge
        let challenge = VerifierProtocol::generate_challenge(&commitment);

        // Prover generates response
        let response =
            ProverProtocol::generate_response(&account, &commitment, &challenge).unwrap();

        // Verifier verifies the proof
        let is_valid =
            VerifierProtocol::verify(&account, &commitment, &challenge, &response).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_invalid_account() {
        let account = SignerPair::create();
        let wrong_account = SignerPair::create(); // Generate different account

        // Create a secret value
        let secret = b"my secret value";
        // Prover generates commitment
        let (commitment, _salt) = ProverProtocol::generate_commitment(secret).unwrap();

        // Verifier generates challenge
        let challenge = VerifierProtocol::generate_challenge(&commitment);

        // Prover generates response with wrong secret key
        let response =
            ProverProtocol::generate_response(&account, &commitment, &challenge).unwrap();

        // Verifier verifies the proof
        let is_valid =
            VerifierProtocol::verify(&wrong_account, &commitment, &challenge, &response).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_invalid_proof() {
        let account = SignerPair::create();

        // Create a secret value
        let secret = b"my secret value";
        // Prover generates commitment
        let (commitment, _salt) = ProverProtocol::generate_commitment(secret).unwrap();

        // Verifier generates challenge
        let challenge = VerifierProtocol::generate_challenge(&commitment);

        // Prover generates response
        let response =
            ProverProtocol::generate_response(&account, &commitment, &challenge).unwrap();

        // Verifier verifies the proof with wrong commitment
        let wrong_commitment = Commitment(vec![0; CHALLENGE_LENGTH]);
        let is_valid =
            VerifierProtocol::verify(&account, &wrong_commitment, &challenge, &response).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_commitment_uniqueness() {
        let secret = b"my secret value";
        let (commitment1, _) = ProverProtocol::generate_commitment(secret).unwrap();
        let (commitment2, _) = ProverProtocol::generate_commitment(secret).unwrap();

        // Commitments should be different due to random salt
        assert_ne!(commitment1.0, commitment2.0);
    }

    #[test]
    fn test_non_interactive() {
        let account = SignerPair::create();
        let secret = b"my secret value";

        let (commitment, response) =
            ProverProtocol::generate_non_interactive(&account, secret).unwrap();

        let valid =
            VerifierProtocol::verify_non_interactive(&account, &commitment, &response).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_non_interactive_wrong_account() {
        let account = SignerPair::create();
        let wrong_account = SignerPair::create();
        let secret = b"my secret value";

        let (commitment, response) =
            ProverProtocol::generate_non_interactive(&account, secret).unwrap();

        let valid =
            VerifierProtocol::verify_non_interactive(&wrong_account, &commitment, &response)
                .unwrap();

        assert!(!valid);
    }

    #[test]
    fn test_non_interactive_wrong_commitment() {
        let account = SignerPair::create();
        let secret = b"my secret value";

        let (_, response) = ProverProtocol::generate_non_interactive(&account, secret).unwrap();

        let wrong_commitment = Commitment(vec![0; CHALLENGE_LENGTH]);

        let valid =
            VerifierProtocol::verify_non_interactive(&account, &wrong_commitment, &response)
                .unwrap();

        assert!(!valid);
    }
}
