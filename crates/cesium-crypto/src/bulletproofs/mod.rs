pub mod fields;
pub mod proof;
pub mod prover;
pub mod verifier;

#[cfg(test)]
mod tests {
    use super::*;
    use prover::Prover;
    use verifier::Verifier;

    #[test]
    fn test_range_proof() {
        let value = 42u64;
        let bit_length = 8;

        let prover = Prover::new(value, bit_length);
        let proof = prover.prove();

        let verifier = Verifier::new(bit_length);
        let is_valid = verifier.verify(&proof);

        assert!(is_valid);
    }

    #[test]
    fn test_invalid_range_proof() {
        let value = 42u64;
        let bit_length = 8;

        let prover: Prover = Prover::new(value, bit_length);
        let mut proof = prover.prove();

        // Modify the proof to make it invalid
        assert_ne!(proof.challenge[0], 0);
        proof.challenge[0] = 0;

        let verifier = Verifier::new(bit_length);
        let is_valid = verifier.verify(&proof);

        assert!(!is_valid);
    }
}
