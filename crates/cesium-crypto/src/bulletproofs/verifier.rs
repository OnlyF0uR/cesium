use sha3::{Digest, Sha3_384};

use super::proof::RangeProof;

/// Verifier for range proofs
pub struct Verifier {
    bit_length: usize,
}

impl Verifier {
    pub fn new(bit_length: usize) -> Self {
        Self { bit_length }
    }

    pub fn verify(&self, proof: &RangeProof) -> bool {
        // Verify format
        if proof.l.len() != self.bit_length.trailing_zeros() as usize
            || proof.r.len() != proof.l.len()
        {
            return false;
        }

        // Verify challenge computation
        let mut hasher = Sha3_384::new();
        hasher.update(&proof.a);
        hasher.update(&proof.s);
        for l in &proof.l {
            hasher.update(l);
        }
        for r in &proof.r {
            hasher.update(r);
        }

        let computed_challenge = hasher.finalize();
        if computed_challenge.as_slice() != proof.challenge.as_slice() {
            return false;
        }

        // The verification passed
        true
    }
}
